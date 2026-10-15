use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::sync::Arc;

use gilvave_core::{error::*, ids::UserId};

use crate::db::Database;

#[derive(Clone)]
pub struct AuthService {
    pub db: Arc<Database>,
}

impl AuthService {
    pub fn hash_password(&self, password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(64_000, 3, 4, None).unwrap(),
        );
        argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    pub fn verify_password(&self, hash: &str, password: &str) -> bool {
        let parsed = PasswordHash::new(hash).unwrap();
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(64_000, 3, 4, None).unwrap(),
        );
        argon2.verify_password(password.as_bytes(), &parsed).is_ok()
    }

    pub async fn create(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                INSERT INTO users (id, username, email, password_hash)
                VALUES ($1, $2, $3, $4);
                "#,
                &[
                    &UserId::default().0,
                    &username,
                    &email,
                    &self.hash_password(password),
                ],
            )
            .await?;
        Ok(())
    }
}
