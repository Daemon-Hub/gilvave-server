use anyhow::Ok;
use sqlx::PgPool;

use gilvave_core::{
    dto::channel::*,
    ids::{ChannelId, ServerId},
};

#[derive(Clone)]
pub struct ChannelService {
    pub db: PgPool,
}

impl ChannelService {
    pub async fn create(
        &self,
        server_id: ServerId,
        info: CreateInfo,
    ) -> anyhow::Result<ChannelView> {
        Ok(sqlx::query_as!(
            ChannelView,
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
                    ),
                    0
                ) + 1
            )
            RETURNING id, name, type as "type: ChannelType", position;
            "#,
            server_id.0,
            info.name,
            info.r#type as ChannelType
        )
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn update_name(
        &self,
        server_id: ServerId,
        channel_id: ChannelId,
        name: NameUpdate,
    ) -> anyhow::Result<ChannelView> {
        Ok(sqlx::query_as!(
            ChannelView,
            r#"
            UPDATE channels
            SET name = $1
            WHERE server_id = $2 AND id = $3
            RETURNING id, name, type as "type: ChannelType", position;
            "#,
            name.0,
            server_id.0,
            channel_id.0
        )
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn update_position(
        &self,
        server_id: ServerId,
        channel_id: ChannelId,
        position: PositionUpdate,
    ) -> anyhow::Result<ChannelView> {
        Ok(sqlx::query_as!(
            ChannelView,
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
            RETURNING id, name, type as "type: ChannelType", position;
            "#,
            server_id.0,
            channel_id.0,
            position.old,
            position.new
        )
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn get_server_channels(
        &self,
        server_id: ServerId,
    ) -> anyhow::Result<Vec<ChannelView>> {
        Ok(sqlx::query_as!(
            ChannelView,
            r#"
            SELECT id, name, type as "type: ChannelType", position
            FROM channels
            WHERE server_id = $1;
            "#,
            server_id.0
        )
        .fetch_all(&self.db)
        .await?)
    }
}
