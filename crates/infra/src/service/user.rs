use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use bytes::Bytes;
use image::ImageFormat;
use sqlx::PgPool;
use std::io::Cursor;

use gilvave_core::{
    dto::user::{Avatar, UpdateAvatarInfo},
    error::CoreError,
    ids::UserId,
    model::user::User,
};

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

    pub async fn create(&self, username: &str, email: &str, password: &str) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4);
            "#,
            UserId::default().0,
            username,
            email,
            self.hash_password(&password),
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, user_id: UserId) -> anyhow::Result<Option<User>> {
        Ok(sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE id = $1;
            "#,
            user_id.0
        )
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        Ok(sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE username = $1;
            "#,
            username
        )
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        Ok(sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE email = $1;
            "#,
            email
        )
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn update_avatar(&self, info: UpdateAvatarInfo) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE users SET avatar = $2
            WHERE id = $1;
            "#,
            info.user_id.0,
            info.url,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Обрабатывает изображение: проверяет размер, формат, сжимает
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

    pub async fn is_token_blacklisted(&self, _jti: &uuid::Uuid) -> bool {
        false
    }
}
