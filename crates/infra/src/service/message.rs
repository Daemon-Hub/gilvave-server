use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{dto::message::*, error::DatabaseError, ids::MessageId};

#[derive(Clone)]
pub struct MessageService {
    pub db: Arc<Database>,
}

impl MessageService {
    pub async fn create(&self, info: CreateInfo) -> Result<MessageView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO messages (id, channel_id, author_id, author_name, content)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, channel_id, author_id, author_name, content, created_at;
                "#,
                &[
                    &MessageId::default().0,
                    &info.channel_id.0,
                    &info.author_id.0,
                    &info.author_name,
                    &info.content,
                ],
            )
            .await?;

        MessageView::from_row(&row)
    }

    pub async fn get_history_before(
        &self,
        info: GetHistoryInfo,
    ) -> Result<Vec<MessageView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, channel_id, author_id, author_name, content, created_at 
                FROM messages
                WHERE channel_id = $1 AND created_at < $2
                ORDER BY created_at DESC
                LIMIT 20;
                "#,
                &[&info.channel_id.0, &info.timestamp],
            )
            .await?
            .iter()
            .map(MessageView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_history_after(
        &self,
        info: GetHistoryInfo,
    ) -> Result<Vec<MessageView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, channel_id, author_id, author_name, content, created_at 
                FROM messages
                WHERE channel_id = $1 AND created_at > $2
                ORDER BY created_at ASC
                LIMIT 20;
                "#,
                &[&info.channel_id.0, &info.timestamp],
            )
            .await?
            .iter()
            .map(MessageView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }
}
