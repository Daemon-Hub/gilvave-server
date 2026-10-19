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
                SELECT u.id, u.username, u.email, u.password_hash, u.is_active, COALESCE(p.avatar, '') AS avatar, COALESCE(p.bio, '') AS bio
                FROM users u
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE u.id = $1;
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
                SELECT u.id, u.username, u.email, u.password_hash, u.is_active, COALESCE(p.avatar, '') AS avatar, COALESCE(p.bio, '') AS bio
                FROM users u
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE LOWER(u.username) = LOWER($1);
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
                SELECT u.id, u.username, u.email, u.password_hash, u.is_active, COALESCE(p.avatar, '') AS avatar, COALESCE(p.bio, '') AS bio
                FROM users u
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE u.email = $1;
                "#,
                &[&email],
            )
            .await?;

        row.map(|r| User::from_row(&r)).transpose()
    }
}
