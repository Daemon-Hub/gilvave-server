use axum::{
    Extension, Json,
    extract::{Multipart, State},
};

use gilvave_core::{
    dto::users::{User, UserView, profile::*},
    error::CoreError,
};

use crate::state::AppState;

pub async fn get_profile(Extension(user): Extension<User>) -> Json<UserView> {
    Json(UserView {
        id: user.id,
        username: user.username,
        email: user.email,
        is_active: user.is_active,
        avatar: user.avatar,
        bio: user.bio,
    })
}

pub async fn update_avatar(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
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

    state.users.profile.update_avatar(user.id, &url).await?;

    Ok(Json(AvatarUrl { url }))
}
