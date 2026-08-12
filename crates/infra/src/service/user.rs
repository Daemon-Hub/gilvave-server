use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use bytes::Bytes;
use image::ImageFormat;
use std::{io::Cursor, sync::Arc};
use time::OffsetDateTime;

use gilvave_core::{
    dto::user::{Avatar, BlacklistInfo, UpdateAvatarInfo, User},
    error::*,
    ids::UserId,
};

use crate::{jwt::verify_jwt, db::Database};

#[derive(Clone)]
pub struct UserService {
    pub db: Arc<Database>,
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

    pub async fn find_by_id(&self, user_id: UserId) -> Result<Option<User>, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT * FROM users
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
                SELECT * FROM users
                WHERE username = $1;
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
                SELECT * FROM users
                WHERE email = $1;
                "#,
                &[&email],
            )
            .await?;

        row.map(|r| User::from_row(&r)).transpose()
    }

    pub async fn update_avatar(&self, info: UpdateAvatarInfo) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                UPDATE users SET avatar = $2
                WHERE id = $1;
                "#,
                &[&info.user_id.0, &info.url],
            )
            .await?;
        Ok(())
    }

    /// Обрабатывает изображение: проверяет размер, формат и сжимает в WebP
    pub fn process_avatar(&self, avatar: &Avatar) -> Result<(Bytes, String), CoreError> {
        // Проверяем размер не более 5 MB
        if avatar.bytes.len() > 5 * 1024 * 1024 {
            return Err(CoreError::PayloadTooLarge(
                "Avatar too large. Max 5 MB".to_string(),
            ));
        }

        // Определяем формат изображения
        _ = match image::ImageFormat::from_extension(
            std::path::Path::new(&avatar.filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""),
        ) {
            Some(f)
                if f == ImageFormat::Jpeg || f == ImageFormat::Png || f == ImageFormat::WebP =>
            {
                f
            }
            _ => {
                // Пробуем определить по содержимому
                match image::guess_format(&avatar.bytes) {
                    Ok(f)
                        if f == ImageFormat::Jpeg
                            || f == ImageFormat::Png
                            || f == ImageFormat::WebP =>
                    {
                        f
                    }
                    _ => {
                        return Err(CoreError::UnsupportedMediaType(
                            "Only JPEG, PNG, WebP images are supported".to_string(),
                        ));
                    }
                }
            }
        };

        // Декодируем изображение
        let img = match image::load_from_memory(&avatar.bytes) {
            Ok(img) => img,
            Err(e) => {
                return Err(CoreError::UnprocessableEntity(format!(
                    "Invalid image data: {}",
                    e
                )));
            }
        };

        // Изменяем размер до 512x512 (максимальный размер аватара)
        let resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);

        // Кодируем обратно в WebP (хорошее сжатие)
        let mut output_bytes = Vec::new();
        match resized.write_to(&mut Cursor::new(&mut output_bytes), ImageFormat::WebP) {
            Ok(_) => (),
            Err(e) => {
                return Err(CoreError::InternalServerError(format!(
                    "Failed to process image: {}",
                    e
                )));
            }
        };

        let mime_type = "image/webp".to_string();
        Ok((Bytes::from(output_bytes), mime_type))
    }

    pub async fn blacklist_token(&self, info: BlacklistInfo) -> Result<(), DatabaseError> {
        let claims = verify_jwt(&info.token).unwrap();
        let exp = OffsetDateTime::from_unix_timestamp(claims.exp).unwrap();

        self.db
            .execute(
                r#"
                INSERT INTO token_blacklist (jti, user_id, expires_at)
                VALUES ($1, $2, $3);
                "#,
                &[&claims.jti, &info.user_id.0, &exp],
            )
            .await?;
        Ok(())
    }

    pub async fn is_token_blacklisted(&self, jti: &uuid::Uuid) -> Result<bool, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT id FROM token_blacklist
                WHERE jti = $1;
                "#,
                &[jti],
            )
            .await?;

        Ok(row.is_some())
    }
}
