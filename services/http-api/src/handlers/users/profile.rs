use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
};

use gilvave_core::{
    dto::users::{UserView, auth::Claims, profile::*}, error::CoreError,
};
use crate::state::AppState;

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserView>, CoreError> {
    let user_view = state
        .users
        .profile
        .get_user_view(claims.sub)
        .await?
        .ok_or_else(|| CoreError::Unauthorized("User not found".to_string()))?;

    Ok(Json(user_view))
}

pub async fn update_avatar(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
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

    let (processed_bytes, mime_type) = state.users.profile.process_avatar(avatar).await?;
    let url = state
        .s3
        .send_static(
            &format!("avatar/{}/{}.webp", claims.sub, uuid::Uuid::new_v4()),
            processed_bytes.into(),
            Some(&mime_type),
        )
        .await
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    state.users.profile.update_avatar(claims.sub, &url).await?;

    Ok(Json(AvatarUrl { url }))
}

pub async fn get_themes_list(
    Path(page): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<(Vec<Theme>, bool)>, CoreError> {
    Ok(Json(
        state.users.profile.get_themes_list((page - 1) * 20).await?,
    ))
}

pub async fn create_theme(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateThemeInfo>,
) -> Result<(), CoreError> {
    let author = state
        .users
        .find_by_id(claims.sub)
        .await?
        .ok_or_else(|| CoreError::Unauthorized("User not found".to_string()))?
        .username;

    state.users.profile.create_theme(author, body).await?;
    Ok(())
}
