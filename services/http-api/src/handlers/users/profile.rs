use axum::{
    Json,
    extract::{Multipart, State},
};

use gilvave_core::{
    dto::users::{UserView, profile::*},
    error::CoreError,
};
use gilvave_infra::security::auth::AuthUser;

use crate::state::AppState;

pub async fn get_profile(AuthUser(user): AuthUser) -> Json<UserView> {
    Json(UserView {
        id: user.id,
        username: user.username,
        email: user.email,
        is_active: user.is_active,
        avatar: user.avatar,
    })
}

pub async fn update_avatar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    mut multipart: Multipart,
) -> Result<Json<AvatarUrl>, CoreError> {
    let avatar = if let Some(field) = multipart.next_field().await.unwrap_or(None)
        && (field.name().unwrap_or("") == "avatar")
    {
        Avatar {
            filename: field
                .file_name()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "avatar".to_string()),
            bytes: match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    return Err(CoreError::BadRequest(format!("Failed to read file: {}", e)));
                }
            },
        }
    } else {
        return Err(CoreError::BadRequest("Missing field 'avatar'".to_string()));
    };

    let (processed_bytes, mime_type) = state.users.profile.process_avatar(&avatar)?;
    let url = state
        .s3
        .send_static(
            &format!("avatar/{}/{}.webp", user.id, uuid::Uuid::new_v4()),
            processed_bytes.into(),
            Some(&mime_type),
        )
        .await
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    state
        .users
        .profile
        .update_avatar(UpdateAvatarInfo {
            user_id: user.id,
            url: url.to_owned(),
        })
        .await?;

    Ok(Json(AvatarUrl { url }))
}
