use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{
    dto::server::*,
    error::DatabaseError,
    ids::{ServerId, UserId},
};

#[derive(Clone)]
pub struct ServerService {
    pub db: Arc<Database>,
}

impl ServerService {
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
    pub async fn get_public(&self, offset: i64) -> Result<Vec<Server>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT * FROM servers
                WHERE is_public = true
                LIMIT 20
                OFFSET $1;
                "#,
                &[&offset],
            )
            .await?
            .iter()
            .map(Server::from_row)
            .collect()
    }

    /// Получить список серверов, в которых состоит пользователь
    pub async fn retrieve_user_servers(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ServerSmallPart>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT s.id, s.name, s.icon_url
                FROM servers s
                JOIN server_members sm ON sm.server_id = s.id
                WHERE sm.user_id = $1; 
                "#,
                &[&user_id.0],
            )
            .await?
            .iter()
            .map(ServerSmallPart::from_row)
            .collect()
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
                VALUES ($1, $2);
                "#,
                &[&info.server_id.0, &info.user_id.0],
            )
            .await?;
        Ok(())
    }

    /// Получить список участников сервера
    pub async fn get_members(&self, server_id: ServerId) -> Result<Vec<MemberView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, username, avatar FROM users
                JOIN server_members sm ON users.id = sm.user_id
                WHERE sm.server_id = $1;
                "#,
                &[&server_id.0],
            )
            .await?
            .iter()
            .map(MemberView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }
}
