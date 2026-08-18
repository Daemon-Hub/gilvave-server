use std::{net::IpAddr, sync::Arc};

use crate::db::Database;
use gilvave_core::{
    dto::user::{SessionCheckInfo, SessionCreateInfo},
    error::DatabaseError,
    ids::{SessionId, UserId},
};
use gilvave_settings::settings;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct SessionService {
    pub db: Arc<Database>,
}

impl SessionService {
    pub async fn get_unexpired(&self, refresh_token_hash: &str) -> Option<(SessionId, UserId)> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT id, user_id, expires_at
                FROM sessions
                WHERE refresh_token_hash = $1
                "#,
                &[&refresh_token_hash.as_bytes()],
            )
            .await
            .ok()??;
        if let Ok(session_info) = SessionCheckInfo::from_row(&row)
            && OffsetDateTime::now_utc() < session_info.expires_at
        {
            Some((session_info.id, session_info.user_id))
        } else {
            None
        }
    }

    pub async fn create<'a>(
        &self,
        info: SessionCreateInfo<'a>,
    ) -> Result<SessionId, DatabaseError> {
        let now = time::OffsetDateTime::now_utc();
        let expires_at = now + settings!().refresh_token_expire_days;

        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO sessions (
                    id, 
                    user_id, 
                    refresh_token_hash, 
                    device_id,
                    device_info, 
                    ip_address, 
                    created_at, 
                    expires_at,
                    last_used_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id;
                "#,
                &[
                    &SessionId::default(),
                    &info.user_id,
                    &info.refresh_token_hash,
                    &info.device_id,
                    &info.device_info,
                    &info.ip_address,
                    &now,
                    &expires_at,
                    &now,
                ],
            )
            .await?;
        Ok(row.try_get(0)?)
    }

    pub async fn delete(&self, user_id: UserId) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                DELETE FROM refresh_token
                WHERE user_id = $1;
                "#,
                &[&user_id],
            )
            .await?;
        Ok(())
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
}
