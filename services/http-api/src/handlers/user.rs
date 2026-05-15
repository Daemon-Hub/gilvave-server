use axum::{Json, extract::State};

use gilvave_core::dto::user::{
    AuthTokensResponse, LoginRequest, RefreshTokenRequest, RegisterRequest,
    UserView,
};
use gilvave_infra::{
    jwt::{create_jwt, generate_refresh_token},
    security::auth::AuthUser,
};

use crate::errors::AppError;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(), AppError> {
    if state
        .user_service
        .find_by_email(&body.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
            "The user with this email address exists.".to_string(),
        ));
    }

    if state
        .user_service
        .find_by_username(&body.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
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
) -> Result<Json<AuthTokensResponse>, AppError> {
    let user = match state.user_service.find_by_email(&body.email).await? {
        Some(u) => u,
        None => {
            return Err(AppError::Unauthorized(
                "The user with this email address has not been found!".to_string(),
            ));
        }
    };

    if !state
        .user_service
        .verify_password(&user.password_hash, &body.password)
    {
        return Err(AppError::Unauthorized("Invalid password".to_string()));
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

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<Json<AuthTokensResponse>, AppError> {
    let user_id = state
        .ref_token_service
        .get(&body.refresh_token)
        .await
        .ok_or(AppError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ))?;

    let access_token = create_jwt(user_id)?;
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
    })
}
