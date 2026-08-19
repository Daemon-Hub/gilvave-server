use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::header::HeaderMap,
};
use std::net::IpAddr;

use gilvave_core::{
    dto::user::{
        AuthTokensResponse, Avatar, AvatarUrl, BlacklistInfo, LoginInfo, RefreshTokenRequest,
        RegisterRequest, SessionCreateInfo, UpdateAvatarInfo, UserView,
    },
    error::CoreError,
};
use gilvave_infra::{
    jwt::{create_jwt, generate_refresh_token, hash_refresh_token},
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
    Extension(ip_address): Extension<IpAddr>,
    Json(body): Json<LoginInfo>,
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

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    let session_id = state
        .session_service
        .create(SessionCreateInfo {
            user_id: user.id,
            refresh_token_hash: refresh_token_hash.as_bytes(),
            device_info: body.device_info,
            ip_address,
        })
        .await?;

    let access_token = create_jwt(session_id, user.id).unwrap();

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
    state.session_service.delete(user.id).await?;
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
    Extension(ip_address): Extension<IpAddr>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<Json<AuthTokensResponse>, CoreError> {
    let refresh_token_hash = hash_refresh_token(&body.refresh_token);
    let (session_id, user_id) = state
        .session_service
        .get_unexpired(&refresh_token_hash)
        .await
        .ok_or(CoreError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ))?;

    let access_token = create_jwt(session_id, user_id)
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    state
        .session_service
        .update(user_id, session_id, refresh_token_hash, ip_address)
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
