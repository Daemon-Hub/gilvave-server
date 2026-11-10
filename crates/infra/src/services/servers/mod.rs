pub mod channels;

use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{
    dto::servers::*,
    error::DatabaseError,
    ids::{ServerId, UserId},
};

#[derive(Clone)]
pub struct ServersService {
    pub db: Arc<Database>,
    pub channels: channels::ChannelsService,
}

impl ServersService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db: db.clone(),
            channels: channels::ChannelsService { db: db.clone() },
        }
    }

    pub async fn create(
        &self,
        info: ServerCreateInfo,
        owner_id: UserId,
    ) -> Result<Server, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO servers (owner_id, name, is_public)
                VALUES ($1, $2, $3)
                RETURNING *;
                "#,
                &[&owner_id.0, &info.name, &info.is_public],
            )
            .await?;

        let server = Server::from_row(&row)?;

        self.add_user(JoinInfo {
            server_id: server.id,
            user_id: owner_id,
        })
        .await?;

        Ok(server)
    }

    pub async fn update(
        &self,
        server_id: ServerId,
        user_id: UserId,
        info: UpdateServerInfo,
    ) -> Result<Server, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                UPDATE servers
                SET
                    name = COALESCE($3, name),
                    description = COALESCE($4, description),
                    icon_url = COALESCE($5, icon_url),
                    cover = COALESCE($6, cover),
                    is_public = COALESCE($7, is_public)
                WHERE id = $1 AND owner_id = $2
                RETURNING *;
                "#,
                &[
                    &server_id.0,
                    &user_id.0,
                    &info.name,
                    &info.description,
                    &info.icon_url,
                    &info.cover,
                    &info.is_public,
                ],
            )
            .await?;

        Server::from_row(&row)
    }

    /// Получить полную информацию о сервере по его ID
    pub async fn get_by_id(
        &self,
        server_id: ServerId,
        user_id: UserId,
    ) -> Result<Server, DatabaseError> {
        Server::from_row(
            &self
                .db
                .query_one(
                    r#"SELECT * FROM servers
                        WHERE id = $1
                        AND (
                            is_public = TRUE
                            OR EXISTS (
                                SELECT 1
                                FROM server_members
                                WHERE server_id = servers.id
                                    AND user_id = $2
                            )
                        );"#,
                    &[&server_id, &user_id],
                )
                .await?,
        )
    }

    /// Получить список публичных серверов
    pub async fn get_public(&self, offset: i64) -> Result<(Vec<Server>, bool), DatabaseError> {
        let mut rows = self
            .db
            .query(
                r#"
                SELECT * FROM servers
                WHERE is_public = true
                ORDER BY created_at DESC
                LIMIT 21
                OFFSET $1;
                "#,
                &[&offset],
            )
            .await?;

        let has_more = rows.len() > 20;

        if has_more {
            rows.truncate(20);
        }

        let mut servers = Vec::with_capacity(rows.len());
        for row in &rows {
            servers.push(Server::from_row(row)?);
        }

        Ok((servers, has_more))
    }

    /// Получить список серверов, в которых состоит пользователь
    pub async fn retrieve_user_servers(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ServerSmallPart>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                SELECT s.id, s.name, s.icon_url, ($1 = s.owner_id) as is_owner
                FROM servers s
                JOIN server_members sm ON sm.server_id = s.id
                WHERE sm.user_id = $1
                ORDER BY sm.joined_at ASC NULLS LAST;
                "#,
                &[&user_id.0],
            )
            .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in &rows {
            list.push(ServerSmallPart::from_row(row)?);
        }
        Ok(list)
    }

    /// Проверить, является ли пользователь владельцем сервера
    pub async fn is_user_owned(
        &self,
        user_id: UserId,
        server_id: ServerId,
    ) -> Result<bool, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT 1
                FROM servers
                WHERE id = $1 AND owner_id = $2; 
                "#,
                &[&server_id.0, &user_id.0],
            )
            .await?;

        Ok(row.is_some())
    }

    /// Добавить пользователя на сервер
    pub async fn add_user(&self, info: JoinInfo) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                INSERT INTO server_members (server_id, user_id)
                VALUES ($1, $2)
                ON CONFLICT (server_id, user_id) DO NOTHING;
                "#,
                &[&info.server_id.0, &info.user_id.0],
            )
            .await?;
        Ok(())
    }

    /// Получить список участников сервера
    pub async fn get_members(&self, server_id: ServerId) -> Result<Vec<MemberView>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                SELECT users.id, users.username, COALESCE(p.avatar, '') AS avatar 
                FROM users
                JOIN server_members sm ON users.id = sm.user_id
                LEFT JOIN profile p ON p.user_id = users.id
                WHERE sm.server_id = $1;
                "#,
                &[&server_id.0],
            )
            .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in &rows {
            list.push(MemberView::from_row(row)?);
        }
        Ok(list)
    }
}
