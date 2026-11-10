use bytes::Bytes;
use image::ImageFormat;
use std::{io::Cursor, sync::Arc};

use crate::db::Database;
use gilvave_core::{
    dto::users::{
        UserView,
        profile::{Avatar, CreateThemeInfo, UpdateProfileInfo, ProfileView, Theme},
    },
    error::*,
    ids::UserId,
};

#[derive(Clone)]
pub struct ProfileService {
    pub db: Arc<Database>,
}

impl ProfileService {
    pub async fn get_user_view(&self, user_id: UserId) -> Result<Option<UserView>, DatabaseError> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT 
                    u.id, 
                    u.username, 
                    u.email, 
                    u.is_active, 
                    u.joined_at,
                    COALESCE(p.name, u.username) AS profile_name,
                    COALESCE(p.bio, '') AS profile_bio,
                    COALESCE(p.avatar, '') AS profile_avatar,
                    COALESCE(p.settings, '{}'::jsonb) AS profile_settings
                FROM users u
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE u.id = $1;
                "#,
                &[&user_id],
            )
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let user_view = UserView::from_row(&row)?;

        Ok(Some(user_view))
    }

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

    pub async fn update_profile(
        &self,
        user_id: UserId,
        info: UpdateProfileInfo,
    ) -> Result<Option<UserView>, DatabaseError> {
        if let Some(ref username) = info.username {
            let trimmed = username.trim();
            if !trimmed.is_empty() {
                self.db
                    .execute(
                        "UPDATE users SET username = $1 WHERE id = $2;",
                        &[&trimmed, &user_id],
                    )
                    .await?;
            }
        }

        let name_val = info.name.or(info.username);
        self.db
            .execute(
                r#"
                INSERT INTO profile (user_id, name, bio, avatar, settings)
                VALUES ($1, COALESCE($2, ''), COALESCE($3, ''), COALESCE($4, ''), '{}'::jsonb)
                ON CONFLICT (user_id)
                DO UPDATE SET
                    name = COALESCE(NULLIF($2, ''), profile.name),
                    bio = COALESCE($3, profile.bio),
                    avatar = COALESCE($4, profile.avatar);
                "#,
                &[
                    &user_id,
                    &name_val,
                    &info.bio,
                    &info.avatar,
                ],
            )
            .await?;

        self.get_user_view(user_id).await
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

        let mut themes = Vec::with_capacity(rows.len());
        for row in &rows {
            themes.push(Theme::from_row(row)?);
        }

        Ok((themes, has_more))
    }

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
        if avatar.bytes.len() > 5 * 1024 * 1024 {
            return Err(CoreError::PayloadTooLarge(
                "Avatar too large. Max 5 MB".to_string(),
            ));
        }

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

        let img = match image::load_from_memory(&avatar.bytes) {
            Ok(img) => img,
            Err(e) => {
                return Err(CoreError::UnprocessableEntity(format!(
                    "Invalid image data: {}",
                    e
                )));
            }
        };

        let resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);

        let mut output_bytes = Vec::with_capacity(32 * 1024);
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
            .map_err(|_| {
                CoreError::InternalServerError("Image processing worker failed".to_string())
            })?
    }
}
