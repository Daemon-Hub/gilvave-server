use fred::error::Error as RedisError;
use fred::prelude::*;

use gilvave_core::ids::{ServerId, SessionId, UserId};
use gilvave_settings::settings;

#[inline]
fn encode_blacklist_key(session_id: SessionId, buf: &mut [u8; 46]) -> &str {
    buf[..10].copy_from_slice(b"blacklist:");
    session_id.encode_hyphenated(&mut buf[10..46].try_into().unwrap());
    std::str::from_utf8(buf).unwrap_or("")
}

#[inline]
fn encode_server_online_key(server_id: ServerId, buf: &mut [u8; 50]) -> &str {
    buf[..7].copy_from_slice(b"server:");
    server_id.encode_hyphenated(&mut buf[7..43].try_into().unwrap());
    buf[43..50].copy_from_slice(b":online");
    std::str::from_utf8(buf).unwrap_or("")
}

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
        let mut uid_buf = [0u8; 36];
        let uid = user_id.encode_hyphenated(&mut uid_buf);

        trx.sadd::<(), &str, _>("global:online", &*uid).await?;

        for sid in server_ids {
            let mut key_buf = [0u8; 50];
            let key = encode_server_online_key(*sid, &mut key_buf);
            trx.sadd::<(), &str, _>(key, &*uid).await?;
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
        let mut uid_buf = [0u8; 36];
        let uid = user_id.encode_hyphenated(&mut uid_buf);

        trx.srem::<(), &str, _>("global:online", &*uid).await?;

        for sid in server_ids {
            let mut key_buf = [0u8; 50];
            let key = encode_server_online_key(*sid, &mut key_buf);
            trx.srem::<(), &str, _>(key, &*uid).await?;
        }

        trx.exec::<()>(true).await?;
        Ok(())
    }

    /// Получить список пользователей онлайн на конкретном сервере
    pub async fn get_server_online(&self, sid: ServerId) -> Result<Vec<UserId>, RedisError> {
        let mut key_buf = [0u8; 50];
        let key = encode_server_online_key(sid, &mut key_buf);
        let users = self.pool.smembers::<Vec<UserId>, _>(key).await?;
        Ok(users)
    }

    /// Получить список пользователей онлайн на конкретных серверах
    pub async fn get_users_online(&self, sids: &[ServerId]) -> Result<Vec<UserId>, RedisError> {
        if sids.is_empty() {
            return Ok(Vec::new());
        }

        let mut keys: Vec<String> = Vec::with_capacity(sids.len());
        for sid in sids {
            let mut key_buf = [0u8; 50];
            keys.push(encode_server_online_key(*sid, &mut key_buf).to_string());
        }

        let users = self.pool.sunion::<Vec<UserId>, _>(keys).await?;
        Ok(users)
    }

    /// Добавить сессию в черный список
    pub async fn revoke_session(&self, session_id: SessionId, exp: i64) -> Result<(), RedisError> {
        let mut key_buf = [0u8; 46];
        let key = encode_blacklist_key(session_id, &mut key_buf);

        self.pool
            .set::<(), &str, _>(key, 1, Some(Expiration::EXAT(exp)), None, false)
            .await?;

        Ok(())
    }

    /// Добавить массив сессий в черный список
    pub async fn revoke_sessions_list(&self, sids: &[SessionId]) -> Result<(), RedisError> {
        let trx = self.pool.multi();
        let exp = settings!().access_token_expire();

        for sid in sids {
            let mut key_buf = [0u8; 46];
            let key = encode_blacklist_key(*sid, &mut key_buf);
            trx.set::<(), &str, _>(
                key,
                1,
                Some(Expiration::EXAT(exp)),
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
        let mut key_buf = [0u8; 46];
        let key = encode_blacklist_key(session_id, &mut key_buf);
        let res = self.pool.exists::<bool, _>(key).await?;
        Ok(res)
    }

    pub async fn set_email_validation_info(
        &self,
        token: &str,
        user_info: &[u8],
    ) -> Result<(), RedisError> {
        let mut key_buf = [0u8; 96];
        let key = if token.len() <= 75 {
            key_buf[..17].copy_from_slice(b"email_validation:");
            key_buf[17..17 + token.len()].copy_from_slice(token.as_bytes());
            std::str::from_utf8(&key_buf[..17 + token.len()]).unwrap_or("")
        } else {
            &format!("email_validation:{token}")
        };

        self.pool
            .set::<(), &str, &[u8]>(
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
        let mut key_buf = [0u8; 96];
        let key = if token.len() <= 75 {
            key_buf[..17].copy_from_slice(b"email_validation:");
            key_buf[17..17 + token.len()].copy_from_slice(token.as_bytes());
            std::str::from_utf8(&key_buf[..17 + token.len()]).unwrap_or("")
        } else {
            &format!("email_validation:{token}")
        };

        let user_info = self.pool.get::<Vec<u8>, &str>(key).await?;
        self.pool.del::<(), &str>(key).await?;
        Ok(user_info)
    }
}
