use gilvave_core::ids::UserId;
use gilvave_settings::settings;
use redis::{AsyncTypedCommands, Client};
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionService {
    redis: Arc<Client>,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            redis: Arc::new(Client::open(settings!().redis_url).unwrap()),
        }
    }

    /// Добавить пользователя в список онлайн
    pub async fn set_user_online(&self, user_id: UserId, node_id: &str) -> anyhow::Result<()> {
        let mut con = self
            .redis
            .clone()
            .get_multiplexed_async_connection()
            .await?;
        con.set(format!("user:{}", user_id.0), node_id).await?;
        Ok(())
    }

    /// Удалить пользователя из списка онлайн
    pub async fn remove_user(&self, user_id: UserId) -> anyhow::Result<()> {
        let mut con = self
            .redis
            .clone()
            .get_multiplexed_async_connection()
            .await?;
        con.del(format!("user:{}", user_id.0)).await?;
        Ok(())
    }
}
