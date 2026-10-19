use maxminddb::{self as mmdb, geoip2};
use std::{net::IpAddr, sync::Arc};

use crate::{db::Database, security::jwt::verify_jwt};
use gilvave_core::{
    dto::users::sessions::{BlacklistInfo, SessionCreateInfo, SessionIds, SessionView},
    error::DatabaseError,
    ids::{SessionId, UserId},
};
use gilvave_settings::settings;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct SessionsService {
    pub db: Arc<Database>,
    pub geo_reader: Option<Arc<mmdb::Reader<Vec<u8>>>>,
}

impl SessionsService {
    pub fn new(db: Arc<Database>) -> Self {
        let reader = mmdb::Reader::open_readfile(r"C:\Users\KIM\GeoLite2-City.mmdb");

        let geo_reader = if let Ok(r) = reader {
            tracing::info!("✓ GeoIP база успешно загружена");
            Some(Arc::new(r))
        } else {
            tracing::warn!("ℹ GeoLite2-City.mmdb не найден, определение локаций отключено");
            None
        };

        Self { db, geo_reader }
    }

    pub fn resolve_location(&self, ip: IpAddr) -> Option<String> {
        if match ip {
            IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local(),
            IpAddr::V6(ipv6) => ipv6.is_loopback(),
        } {
            return Some("Локальная сеть".to_string());
        }

        let reader = self.geo_reader.as_ref()?;
        let result = reader.lookup(ip).ok()?;
        let city = result.decode::<geoip2::City>().ok()??;

        let country_name = city.country.names.russian.or(city.country.names.english);
        let city_name = city.city.names.russian.or(city.city.names.english);

        match (city_name, country_name) {
            (Some(city), Some(country)) => Some(format!("{city}, {country}")),
            (None, Some(country)) => Some(country.to_string()),
            (Some(city), None) => Some(city.to_string()),
            (None, None) => None,
        }
    }

    pub async fn get_unexpired(&self, refresh_token_hash: &str) -> Option<SessionIds> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT id, user_id
                FROM sessions
                WHERE refresh_token_hash = $1
                    AND expires_at > NOW();
                "#,
                &[&refresh_token_hash.as_bytes()],
            )
            .await
            .ok()??;
        SessionIds::from_row(&row).ok()
    }

    pub async fn create<'a>(
        &self,
        info: SessionCreateInfo<'a>,
    ) -> Result<SessionId, DatabaseError> {
        let now = time::OffsetDateTime::now_utc();
        let expires_at = now + settings!().refresh_token_expire_days;
        let location = self.resolve_location(info.ip_address);

        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO sessions (
                    id, 
                    user_id, 
                    refresh_token_hash, 
                    device_info, 
                    ip_address,
                    location,
                    last_used_at,
                    created_at, 
                    expires_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id;
                "#,
                &[
                    &SessionId::default(),
                    &info.user_id,
                    &info.refresh_token_hash,
                    &info.device_info,
                    &info.ip_address,
                    &location,
                    &now,
                    &now,
                    &expires_at,
                ],
            )
            .await?;
        Ok(row.try_get(0)?)
    }

    pub async fn delete_by_id(
        &self,
        user_id: UserId,
        session_id: SessionId,
    ) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                DELETE FROM sessions
                WHERE user_id = $1 AND id = $2;
                "#,
                &[&user_id, &session_id],
            )
            .await?;
        Ok(())
    }

    pub async fn delete_all_other(
        &self,
        user_id: UserId,
        session_id: SessionId,
    ) -> Result<Vec<SessionId>, DatabaseError> {
        Ok(self
            .db
            .query(
                r#"
                DELETE FROM sessions
                WHERE user_id = $1 AND id != $2
                RETURNING id;
                "#,
                &[&user_id, &session_id],
            )
            .await?
            .iter()
            .map(|r| r.get(0))
            .collect::<Vec<SessionId>>())
    }

    pub async fn update(
        &self,
        user_id: UserId,
        session_id: SessionId,
        refresh_token_hash: String,
        new_ip: IpAddr,
    ) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                UPDATE sessions 
                SET refresh_token_hash = $3, last_used_at = $4, ip_address = $5
                WHERE id = $1 AND user_id = $2;
                "#,
                &[
                    &session_id,
                    &user_id,
                    &refresh_token_hash.as_bytes(),
                    &OffsetDateTime::now_utc(),
                    &new_ip,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn get_list(
        &self,
        user_id: UserId,
        session_id: SessionId,
    ) -> Result<Vec<(SessionView, bool)>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                SELECT id, device_info, HOST(ip_address) AS ip_address, location, COALESCE(last_used_at, created_at) AS last_used_at, created_at
                FROM sessions
                WHERE user_id = $1
                ORDER BY created_at DESC;
                "#,
                &[&user_id],
            )
            .await?;
        let mut sessions = Vec::new();
        for row in rows {
            let session = SessionView::from_row(&row)?;
            let sid = session.id;
            sessions.push((session, session_id == sid));
        }
        Ok(sessions)
    }

    pub async fn blacklist_token(&self, info: BlacklistInfo) -> Result<(), DatabaseError> {
        let claims = verify_jwt(&info.token).unwrap();
        let exp = OffsetDateTime::from_unix_timestamp(claims.exp).unwrap();

        self.db
            .execute(
                r#"
                INSERT INTO token_blacklist (jti, user_id, expires_at)
                VALUES ($1, $2, $3);
                "#,
                &[&claims.jti, &info.user_id.0, &exp],
            )
            .await?;
        Ok(())
    }

    pub async fn is_token_blacklisted(&self, jti: &uuid::Uuid) -> Result<bool, DatabaseError> {
        let row = self
            .db
            .query_opt(r#"SELECT 1 FROM token_blacklist WHERE jti = $1"#, &[jti])
            .await?;

        Ok(row.is_some())
    }
}
