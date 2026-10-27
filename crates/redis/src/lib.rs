use fred::error::Error as RedisError;
use fred::prelude::*;

use gilvave_core::ids::{ServerId, SessionId, UserId};
use gilvave_settings::settings;

#[derive(Clone)]
pub struct RedisService {
    pool: Pool,
}

impl RedisService {
    /// Асинхронный конструктор
    pub async fn new() -> Result<Self, RedisError> {
        let mut config = Config::from_url(settings!().redis_url)?;
        config.version = fred::types::RespVersion::RESP2;

        let pool = Builder::from_config(config)
            .with_connection_config(|cfg| {
                cfg.connection_timeout = std::time::Duration::from_secs(10);
                cfg.tcp = TcpConfig {
                    nodelay: Some(true),
                    ..Default::default()
                };
            })
            .build_pool(8)?;

        pool.init().await?;

        Ok(Self { pool })
    }

    /// Добавить пользователя в глобальный список онлайн
    pub async fn set_user_online(
        &self,
        user_id: UserId,
        server_ids: &[ServerId],
    ) -> Result<(), RedisError> {
        let trx = self.pool.multi();
        let uid = user_id.to_string();

        trx.sadd::<(), &str, _>("global:online", &uid).await?;

        for sid in server_ids {
            trx.sadd::<(), _, _>(format!("server:{sid}:online"), &uid)
                .await?;
        }

        trx.exec::<()>(true).await?;
        Ok(())
    }

    /// Удалить пользователя из глобального списка онлайн
    pub async fn del_user_online(
        &self,
        user_id: UserId,
        server_ids: &[ServerId],
    ) -> Result<(), RedisError> {
        let trx = self.pool.multi();
        let uid = user_id.to_string();

        trx.srem::<(), &str, _>("global:online", &uid).await?;

        for sid in server_ids {
            trx.srem::<(), _, _>(format!("server:{sid}:online"), &uid)
                .await?;
        }

        trx.exec::<()>(true).await?;
        Ok(())
    }

    /// Добавить сессию в черный список
    pub async fn revoke_session(&self, session_id: SessionId, exp: i64) -> Result<(), RedisError> {
        let key = format!("blacklist:{session_id}");

        self.pool
            .set::<(), String, _>(key, 1, Some(Expiration::EXAT(exp)), None, false)
            .await?;

        Ok(())
    }

    /// Добавить массив сессий в черный список
    pub async fn revoke_sessions_list(&self, sids: &[SessionId]) -> Result<(), RedisError> {
        let trx = self.pool.multi();

        for sid in sids {
            trx.set::<(), String, _>(
                format!("blacklist:{sid}"),
                1,
                Some(Expiration::EXAT(settings!().access_token_expire())),
                None,
                false,
            )
            .await?;
        }

        trx.exec::<()>(true).await?;
        Ok(())
    }

    /// Проверяет есть ли сессия в черном списке и возвращает true/false
    pub async fn get_session_revocation_status(
        &self,
        session_id: SessionId,
    ) -> Result<bool, RedisError> {
        let key = format!("blacklist:{session_id}");
        let res = self.pool.exists::<bool, String>(key).await?;
        Ok(res)
    }

    pub async fn set_email_validation_info(
        &self,
        token: &str,
        user_info: &[u8],
    ) -> Result<(), RedisError> {
        let key = format!("email_validation:{token}");
        self.pool
            .set::<(), String, &[u8]>(
                key,
                user_info,
                Some(Expiration::EX(
                    settings!().email_validation_ttl.whole_seconds(),
                )),
                None,
                true,
            )
            .await?;
        Ok(())
    }

    pub async fn get_email_validated_info(&self, token: &str) -> Result<Vec<u8>, RedisError> {
        let key = format!("email_validation:{token}");
        let user_info = self.pool.get::<Vec<u8>, &str>(&key).await?;
        self.pool.del::<(), &str>(&key).await?;
        Ok(user_info)
    }
}
