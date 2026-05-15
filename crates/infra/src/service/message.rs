use sqlx::PgPool;

use gilvave_core::{
    dto::message::*,
    ids::{MessageId, UserId},
};

#[derive(Clone)]
pub struct MessageService {
    pub db: PgPool,
}

impl MessageService {
    pub async fn create(&self, info: CreateInfo) -> anyhow::Result<MessageView> {
        let res = sqlx::query_as!(
            MessageView,
            r#"
            INSERT INTO messages (id, channel_id, author_id, author_name, content)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, channel_id, author_id as "author_id: UserId", author_name, content, created_at;
            "#,
            MessageId::default().0,
            info.channel_id.0,
            info.author_id.0,
            info.author_name,
            info.content,
        )
        .fetch_one(&self.db)
        .await?;
        Ok(res)
    }

    pub async fn get_history_by_time(&self, info: GetHistoryInfo) -> anyhow::Result<Vec<MessageView>> {
        let res = sqlx::query_as!(
            MessageView,
            r#"
            SELECT id, channel_id, author_id as "author_id: UserId", author_name, content, created_at 
            FROM messages
            WHERE channel_id = $1 AND created_at >= $2
            LIMIT 20;
            "#,
            info.channel_id.0,
            info.from
        )
        .fetch_all(&self.db)
        .await?;
        Ok(res)
    }
}
