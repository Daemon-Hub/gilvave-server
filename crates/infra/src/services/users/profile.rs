use bytes::Bytes;
use image::ImageFormat;
use std::{io::Cursor, sync::Arc};

use crate::db::Database;
use gilvave_core::{
    dto::users::profile::{Avatar, CreateThemeInfo, ProfileView, Theme},
    error::*,
    ids::UserId,
};

#[derive(Clone)]
pub struct ProfileService {
    pub db: Arc<Database>,
}

impl ProfileService {
    pub async fn get_by_user_id(&self, user_id: UserId) -> Result<ProfileView, DatabaseError> {
        let row = self
            .db
            .query_one(
                r#"
                SELECT name, bio, avatar, settings
                FROM profile
                WHERE user_id = $1;
                "#,
                &[&user_id],
            )
            .await?;
        ProfileView::from_row(&row)
    }

    pub async fn get_themes_list(&self, offset: i64) -> Result<(Vec<Theme>, bool), DatabaseError> {
        let mut rows = self
            .db
            .query(
                r#"
                SELECT * FROM themes
                ORDER BY created_at DESC
                LIMIT 21 OFFSET $1;
                "#,
                &[&offset],
            )
            .await?;

        let has_more = rows.len() > 20;

        if has_more {
            rows.truncate(20);
        }

        let servers = rows
            .iter()
            .map(Theme::from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((servers, has_more))
    }

    /// TODO: Закончить функцию создания темы
    pub async fn create_theme(
        &self,
        author: String,
        info: CreateThemeInfo,
    ) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                INSERT INTO
                    themes (
                        name,
                        author,
                        description,
                        config,
                        created_at,
                        updated_at
                    )
                VALUES ($1, $2, $3, $4, NOW(), NOW());
                "#,
                &[&info.name, &author, &info.description, &info.config],
            )
            .await?;
        Ok(())
    }

    pub async fn update_avatar(&self, user_id: UserId, url: &str) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                INSERT INTO profile (user_id, name, avatar, settings)
                VALUES ($1, '', $2, '{}'::jsonb)
                ON CONFLICT (user_id)
                DO UPDATE SET avatar = EXCLUDED.avatar;
                "#,
                &[&user_id, &url],
            )
            .await?;
        Ok(())
    }

    /// Синхронная обработка изображения
    pub fn process_avatar_sync(avatar: &Avatar) -> Result<(Bytes, String), CoreError> {
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

    /// Асинхронная обработка аватара: перенос тяжелого ресайза и сжатия в WebP на spawn_blocking пул
    pub async fn process_avatar(&self, avatar: Avatar) -> Result<(Bytes, String), CoreError> {
        tokio::task::spawn_blocking(move || Self::process_avatar_sync(&avatar))
            .await
            .map_err(|_| CoreError::InternalServerError("Image processing worker failed".to_string()))?
    }
}
