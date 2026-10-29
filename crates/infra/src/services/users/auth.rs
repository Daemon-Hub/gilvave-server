use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::sync::{Arc, LazyLock};

use gilvave_core::{dto::users::auth::CreateUserInfo, error::*, ids::UserId};

use crate::db::Database;

static ARGON2: LazyLock<Argon2<'static>> = LazyLock::new(|| {
    let params = Params::new(64_000, 3, 4, None).expect("Argon2 params must be valid");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
});

#[derive(Clone)]
pub struct AuthService {
    pub db: Arc<Database>,
}

impl AuthService {
    /// Синхронное хеширование пароля
    pub fn hash_password_sync(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        ARGON2
            .hash_password(password.as_bytes(), &salt)
            .expect("Password hashing failed")
            .to_string()
    }

    /// Асинхронное хеширование пароля через пул блокирующих задач (предотвращает блокировку потоков Tokio)
    pub async fn hash_password(&self, password: &str) -> String {
        let password = password.to_string();
        tokio::task::spawn_blocking(move || Self::hash_password_sync(&password))
            .await
            .unwrap_or_default()
    }

    /// Синхронная проверка пароля без паники на некорректном хеше
    pub fn verify_password_sync(hash: &str, password: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(hash) else {
            return false;
        };
        ARGON2.verify_password(password.as_bytes(), &parsed).is_ok()
    }

    /// Асинхронная проверка пароля через пул блокирующих задач
    pub async fn verify_password(&self, hash: &str, password: &str) -> bool {
        let hash = hash.to_string();
        let password = password.to_string();
        tokio::task::spawn_blocking(move || Self::verify_password_sync(&hash, &password))
            .await
            .unwrap_or(false)
    }

    pub async fn create(&self, info: CreateUserInfo) -> Result<(), DatabaseError> {
        let user_id = UserId::default();
        let password_hash = self.hash_password(&info.password).await;

        self.db
            .execute(
                r#"
                INSERT INTO users (id, username, email, password_hash)
                VALUES ($1, $2, $3, $4);
                "#,
                &[&user_id, &info.username, &info.email, &password_hash],
            )
            .await?;

        self.db
            .execute(
                r#"
                INSERT INTO profile (user_id, name, bio, avatar, settings)
                VALUES ($1, $2, '', '', '{}'::jsonb);
                "#,
                &[&user_id, &info.username],
            )
            .await?;

        Ok(())
    }
}
