use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

use gilvave_core::{dto::user::RegisterResponse, ids::UserId, model::user::User};

#[derive(Clone)]
pub struct UserService {
    pub db: PgPool,
}

impl UserService {
    pub fn hash_password(&self, password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);

        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    pub fn verify_password(&self, hash: &str, password: &str) -> bool {
        let parsed = PasswordHash::new(hash).unwrap();

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }

    pub async fn create(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> anyhow::Result<RegisterResponse> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email;
            "#,
            UserId::default().0,
            username,
            email,
            self.hash_password(&password)
        )
        .fetch_one(&self.db)
        .await?;

        Ok(RegisterResponse {
            id: UserId(rec.id),
            username: rec.username,
            email: rec.email,
        })
    }

    pub async fn find_by_id(&self, user_id: UserId) -> Option<User> {
        if let Ok(Some(res)) = sqlx::query!(
            r#"
            SELECT * FROM users
            WHERE id = $1;
            "#,
            user_id.0
        )
        .fetch_optional(&self.db)
        .await
        {
            return Some(User {
                id: UserId(res.id),
                username: res.username,
                email: res.email,
                password_hash: res.password_hash,
                is_active: res.is_active,
            });
        }
        None
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        if let Ok(Some(res)) = sqlx::query!(
            r#"
            SELECT * FROM users
            WHERE username = $1;
            "#,
            username
        )
        .fetch_optional(&self.db)
        .await
        {
            return Some(User {
                id: UserId(res.id),
                username: res.username,
                email: res.email,
                password_hash: res.password_hash,
                is_active: res.is_active,
            });
        }
        None
    }

    pub async fn find_by_email(&self, email: &str) -> Option<User> {
        if let Ok(Some(res)) = sqlx::query!(
            r#"
            SELECT * FROM users
            WHERE email = $1;
            "#,
            email
        )
        .fetch_optional(&self.db)
        .await
        {
            return Some(User {
                id: UserId(res.id),
                username: res.username,
                email: res.email,
                password_hash: res.password_hash,
                is_active: res.is_active,
            });
        }
        None
    }

    pub async fn is_token_blacklisted(&self, _jti: &uuid::Uuid) -> bool {
        false
    }
}
