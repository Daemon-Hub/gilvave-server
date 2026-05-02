use sqlx::PgPool;

use gilvave_core::{
    dto::server::*,
    ids::{ServerId, UserId},
};

const ICON_URL_DEFAULT: &str = "http://cdn.gilvave.ru/i/123";

#[derive(Clone)]
pub struct ServerService {
    pub db: PgPool,
}

impl ServerService {
    pub async fn create(
        &self,
        info: ServerCreateInfo,
        owner_id: UserId,
    ) -> anyhow::Result<ServerView> {
        Ok(sqlx::query_as!(
            ServerView,
            r#"
            INSERT INTO servers (name, owner_id, icon_url, is_public)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, icon_url, created_at;
            "#,
            info.name,
            owner_id.0,
            info.icon_url.unwrap_or(ICON_URL_DEFAULT.into()),
            info.is_public
        )
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn get_all_public(&self) -> anyhow::Result<Vec<ServerView>> {
        Ok(sqlx::query_as!(
            ServerView,
            r#"
            SELECT id, name, icon_url, created_at
            FROM servers
            WHERE is_public = true; 
            "#
        )
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn get_owned(&self, user_id: UserId) -> anyhow::Result<Vec<ServerView>> {
        Ok(sqlx::query_as!(
            ServerView,
            r#"
            SELECT id, name, icon_url, created_at
            FROM servers
            WHERE owner_id = $1; 
            "#,
            user_id.0
        )
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn get_member(&self, user_id: UserId) -> anyhow::Result<Vec<ServerView>> {
        Ok(sqlx::query_as!(
            ServerView,
            r#"
            SELECT s.id, s.name, s.icon_url, s.created_at
            FROM servers s
            JOIN server_members sm ON sm.server_id = s.id
            WHERE sm.user_id = $1; 
            "#,
            user_id.0
        )
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn get_all_by_user(&self, user_id: UserId) -> anyhow::Result<Vec<ServerView>> {
        let mut owned = self.get_owned(user_id).await.unwrap_or_default();
        let member = self.get_member(user_id).await.unwrap_or_default();
        owned.extend(member);
        // Может понадобится!
        // Убирает дубликаты по ID сервера
        // owned.dedup_by(|a, b| a.id == b.id);
        Ok(owned)
    }

    pub async fn is_user_owned(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> anyhow::Result<bool> {
        Ok(sqlx::query!(
            r#"
            SELECT name
            FROM servers
            WHERE id = $1 AND owner_id = $2; 
            "#,
            server_id.0,
            user_id.0
        )
        .fetch_one(&self.db)
        .await
        .is_ok())
    }
}
