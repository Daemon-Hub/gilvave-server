use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{
    dto::channel::*,
    error::DatabaseError,
    ids::{ChannelId, ServerId},
};

#[derive(Clone)]
pub struct ChannelService {
    pub db: Arc<Database>,
}

impl ChannelService {
    pub async fn create(
        &self,
        server_id: ServerId,
        info: CreateInfo,
    ) -> Result<ChannelView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO channels (
                    server_id,
                    name,
                    type,
                    position
                )
                VALUES ($1, $2, $3,
                    COALESCE(
                        (
                            SELECT max(position)
                            FROM channels
                            WHERE server_id = $1 
                              AND type = $3
                        ),
                        0
                    ) + 1
                )
                RETURNING id, name, type, position;
                "#,
                &[
                    &server_id.0,
                    &info.name,
                    &info.r#type, // Предполагается, что ChannelType реализует ToSql
                ],
            )
            .await?;

        ChannelView::from_row(&row)
    }

    pub async fn update_name(
        &self,
        server_id: ServerId,
        channel_id: ChannelId,
        name: NameUpdate,
    ) -> Result<ChannelView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                UPDATE channels
                SET name = $1
                WHERE server_id = $2 AND id = $3
                RETURNING id, name, type, position;
                "#,
                &[&name.0, &server_id.0, &channel_id.0],
            )
            .await?;

        ChannelView::from_row(&row)
    }

    pub async fn update_position(
        &self,
        server_id: ServerId,
        channel_id: ChannelId,
        position: PositionUpdate,
    ) -> Result<ChannelView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                WITH shift AS (
                UPDATE channels
                SET position = CASE
                    WHEN $3 < $4 THEN position - 1
                    WHEN $3 > $4 THEN position + 1
                    ELSE position
                END
                WHERE server_id = $1 AND id != $2
                    AND CASE
                        WHEN $3 < $4::int THEN position > $3
                        AND position <= $4
                        WHEN $3 > $4 THEN position >= $4
                        AND position < $3
                    END
                )
                UPDATE channels
                SET position = $4
                WHERE id = $2 AND server_id = $1
                RETURNING id, name, type, position;
                "#,
                &[&server_id.0, &channel_id.0, &position.old, &position.new],
            )
            .await?;

        ChannelView::from_row(&row)
    }

    pub async fn get_server_channels(
        &self,
        server_id: ServerId,
    ) -> Result<Vec<ChannelView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, name, type, position
                FROM channels
                WHERE server_id = $1;
                "#,
                &[&server_id.0],
            )
            .await?
            .iter()
            .map(ChannelView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }
}
