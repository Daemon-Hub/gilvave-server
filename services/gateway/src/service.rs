use redis::{/*AsyncTypedCommands,*/ Client, RedisResult, aio::MultiplexedConnection};
use std::sync::Arc;

use gilvave_core::ids::{ServerId, UserId};
use gilvave_settings::settings;

#[derive(Clone)]
pub struct RedisService {
    redis: Arc<Client>,
}

impl RedisService {
    pub fn new() -> Self {
        Self {
            redis: Arc::new(Client::open(settings!().redis_url).unwrap()),
        }
    }

    async fn get_connection(&self) -> RedisResult<MultiplexedConnection> {
        self.redis.clone().get_multiplexed_async_connection().await
    }

    /// Добавить пользователя в глобальный список онлайн
    pub async fn set_user_online(
        &self,
        user_id: UserId,
        server_ids: Vec<ServerId>,
    ) -> RedisResult<()> {
        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.sadd("global:online", user_id.to_string()).ignore();

        for sid in server_ids {
            pipe.sadd(format!("server:{}:online", sid), user_id.to_string())
                .ignore();
        }

        let mut con = self.get_connection().await?;
        pipe.query_async::<()>(&mut con).await
    }

    /// Удалить пользователя из глобального списка онлайн
    pub async fn del_user_online(
        &self,
        user_id: UserId,
        server_ids: Vec<ServerId>,
    ) -> RedisResult<()> {
        let mut pipe = redis::pipe();
        pipe.atomic();

        pipe.srem("global:online", user_id.to_string()).ignore();

        for sid in server_ids {
            pipe.srem(format!("server:{}:online", sid), user_id.to_string())
                .ignore();
        }

        let mut con = self.get_connection().await?;
        pipe.query_async::<()>(&mut con).await
    }
}
