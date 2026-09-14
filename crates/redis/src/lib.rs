use fred::error::Error as RedisError;
use fred::prelude::*;

use gilvave_core::ids::{ServerId, UserId};
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
        server_ids: Vec<ServerId>,
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
        server_ids: Vec<ServerId>,
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
}
