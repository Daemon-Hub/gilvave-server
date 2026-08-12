use axum::{
    Json,
    extract::{Multipart, State},
    http::header::HeaderMap,
};

use gilvave_core::{
    dto::user::{
        AuthTokensResponse, Avatar, AvatarUrl, BlacklistInfo, LoginRequest, RefreshTokenRequest,
        RegisterRequest, UpdateAvatarInfo, UserView,
    },
    error::CoreError,
};
use gilvave_infra::{
    jwt::{create_jwt, generate_refresh_token},
    security::auth::AuthUser,
};

use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(), CoreError> {
    if state
        .user_service
        .find_by_email(&body.email)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "The user with this email address exists.".to_string(),
        ));
    }

    if state
        .user_service
        .find_by_username(&body.username)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "The user with this username exists.".to_string(),
        ));
    }

    state
        .user_service
        .create(&body.username, &body.email, &body.password)
        .await?;

    Ok(())
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthTokensResponse>, CoreError> {
    let user = match state.user_service.find_by_email(&body.email).await? {
        Some(u) => u,
        None => {
            return Err(CoreError::Unauthorized(
                "The user with this email address has not been found!".to_string(),
            ));
        }
    };

    if !state
        .user_service
        .verify_password(&user.password_hash, &body.password)
    {
        return Err(CoreError::Unauthorized("Invalid password".to_string()));
    }

    let access_token = create_jwt(user.id).unwrap();
    let refresh_token = generate_refresh_token();

    state
        .ref_token_service
        .sync(user.id, &refresh_token)
        .await?;

    Ok(Json(AuthTokensResponse {
        access_token,
        refresh_token,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    header: HeaderMap,
) -> Result<(), CoreError> {
    state.ref_token_service.delete(user.id).await?;
    let token = String::from(
        header
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap()
            .strip_prefix("Bearer ")
            .unwrap(),
    );
    state
        .user_service
        .blacklist_token(BlacklistInfo {
            token,
            user_id: user.id,
        })
        .await?;
    Ok(())
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<Json<AuthTokensResponse>, CoreError> {
    let user_id = state
        .ref_token_service
        .get(&body.refresh_token)
        .await
        .ok_or(CoreError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ))?;

    let access_token =
        create_jwt(user_id).map_err(|e| CoreError::InternalServerError(e.to_string()))?;
    let refresh_token = generate_refresh_token();

    state
        .ref_token_service
        .sync(user_id, &refresh_token)
        .await?;

    Ok(Json(AuthTokensResponse {
        access_token,
        refresh_token,
    }))
}

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

    let (processed_bytes, mime_type) = state.user_service.process_avatar(&avatar)?;
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
        .user_service
        .update_avatar(UpdateAvatarInfo {
            user_id: user.id,
            url: url.to_owned(),
        })
        .await?;

    Ok(Json(AvatarUrl { url }))
}
