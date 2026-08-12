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
    ) -> Result<ServerView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO servers (name, owner_id, icon_url, is_public)
                VALUES ($1, $2, $3, $4)
                RETURNING id, name, icon_url, created_at;
                "#,
                &[&info.name, &owner_id.0, &info.icon_url, &info.is_public],
            )
            .await?;

        let server = ServerView::from_row(&row)?;

        self.add_user(JoinInfo {
            server_id: server.id,
            user_id: owner_id,
        })
        .await?;

        Ok(server)
    }

    pub async fn get_all_public(&self) -> Result<Vec<ServerView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, name, icon_url, created_at
                FROM servers
                WHERE is_public = true; 
                "#,
                &[],
            )
            .await?
            .iter()
            .map(ServerView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_owned(&self, user_id: UserId) -> Result<Vec<ServerView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT id, name, icon_url, created_at
                FROM servers
                WHERE owner_id = $1; 
                "#,
                &[&user_id.0],
            )
            .await?
            .iter()
            .map(ServerView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_member(&self, user_id: UserId) -> Result<Vec<ServerView>, DatabaseError> {
        self.db
            .query(
                r#"
                SELECT s.id, s.name, s.icon_url, s.created_at
                FROM servers s
                JOIN server_members sm ON sm.server_id = s.id
                WHERE sm.user_id = $1; 
                "#,
                &[&user_id.0],
            )
            .await?
            .iter()
            .map(ServerView::from_row)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_all_by_user(&self, user_id: UserId) -> Result<Vec<ServerView>, DatabaseError> {
        //let mut owned = self.get_owned(user_id).await.unwrap_or_default();
        let member = self.get_member(user_id).await.unwrap_or_default();
        //owned.extend(member);
        Ok(member)
    }

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
