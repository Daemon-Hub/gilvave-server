pub mod auth;
pub mod profile;
pub mod sessions;

use std::sync::Arc;

use gilvave_core::{dto::users::User, error::*, ids::UserId};
use crate::db::Database;

#[derive(Clone)]
pub struct UsersService {
    pub db: Arc<Database>,
    pub auth: auth::AuthService,
    pub profile: profile::ProfileService,
    pub sessions: sessions::SessionsService,
}

impl UsersService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db: db.clone(),
            auth: auth::AuthService { db: db.clone() },
            profile: profile::ProfileService { db: db.clone() },
            sessions: sessions::SessionsService::new(db),
        }
    }

    pub async fn find_by_id(&self, user_id: UserId) -> Result<Option<User>, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT id, username, email, password_hash, is_active, joined_at
                FROM users
                WHERE id = $1;
                "#,
                &[&user_id.0],
            )
            .await?;

        row.map(|r| User::from_row(&r)).transpose()
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT id, username, email, password_hash, is_active, joined_at
                FROM users
                WHERE LOWER(username) = LOWER($1);
                "#,
                &[&username],
            )
            .await?;

        row.map(|r| User::from_row(&r)).transpose()
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT id, username, email, password_hash, is_active, joined_at
                FROM users
                WHERE email = $1;
                "#,
                &[&email],
            )
            .await?;

        row.map(|r| User::from_row(&r)).transpose()
    }
}
