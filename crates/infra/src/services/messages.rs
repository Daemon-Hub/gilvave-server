use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{
    dto::messages::*,
    error::DatabaseError,
    ids::{ChannelId, MessageId, UserId},
};

#[derive(Clone)]
pub struct MessagesService {
    pub db: Arc<Database>,
}

impl MessagesService {
    pub async fn create(&self, info: CreateInfo) -> Result<MessageView, DatabaseError> {
        let reply_to_uuid = info.reply_to_id.as_ref().map(|id| id.0);
        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO messages (id, channel_id, author_id, author_name, content, reply_to_id, forwarded_from)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at;
                "#,
                &[
                    &MessageId::default().0,
                    &info.channel_id.0,
                    &info.author_id.0,
                    &info.author_name,
                    &info.content,
                    &reply_to_uuid,
                    &info.forwarded_from,
                ],
            )
            .await?;

        MessageView::from_row(&row)
    }

    pub async fn update(
        &self,
        id: MessageId,
        channel_id: ChannelId,
        author_id: UserId,
        content: String,
    ) -> Result<MessageView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                UPDATE messages
                SET content = $1, edited = TRUE
                WHERE id = $2 AND channel_id = $3 AND author_id = $4
                RETURNING id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at;
                "#,
                &[&content, &id.0, &channel_id.0, &author_id.0],
            )
            .await?;

        MessageView::from_row(&row)
    }

    pub async fn delete(
        &self,
        id: MessageId,
        channel_id: ChannelId,
        author_id: UserId,
    ) -> Result<bool, DatabaseError> {
        let count = self
            .db
            .execute(
                r#"
                DELETE FROM messages
                WHERE id = $1 AND channel_id = $2 AND author_id = $3;
                "#,
                &[&id.0, &channel_id.0, &author_id.0],
            )
            .await?;

        Ok(count > 0)
    }

    pub async fn get_history_before(
        &self,
        info: GetHistoryInfo,
    ) -> Result<Vec<MessageView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at 
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
                SELECT id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at 
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
