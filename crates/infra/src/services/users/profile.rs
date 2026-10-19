use bytes::Bytes;
use image::ImageFormat;
use std::{io::Cursor, sync::Arc};

use gilvave_core::{dto::users::profile::Avatar, error::*, ids::UserId};

use crate::db::Database;

#[derive(Clone)]
pub struct ProfileService {
    pub db: Arc<Database>,
}

impl ProfileService {
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
}
