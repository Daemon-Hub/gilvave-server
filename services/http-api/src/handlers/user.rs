use axum::{Json, extract::State};

use gilvave_core::dto::user::{
    FooData, LoginRequest, LoginResponse, RegisterRequest, RegisterResponse,
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
) -> Json<RegisterResponse> {
    if state
        .user_service
        .find_by_email(&body.email)
        .await
        .is_some()
    {
        panic!("The user with this email address exists.");
    }

    if state
        .user_service
        .find_by_username(&body.username)
        .await
        .is_some()
    {
        panic!("The user with this username exists.");
    }

    let user = state
        .user_service
        .create_user(&body.username, &body.email, &body.password)
        .await
        .unwrap();

    Json(user)
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = state
        .user_service
        .find_by_email(&body.email)
        .await
        .ok_or_else(|| {
            AppError::Unauthorized(
                "The user with this email address has not been found!".to_string(),
            )
        })?;

    if !state
        .user_service
        .verify_password(&user.password_hash, &body.password)
    {
        return Err(AppError::Unauthorized("Invalid password".to_string()));
    }

    let access_token = create_jwt(&user.id.0.to_string()).unwrap();
    let refresh_token = generate_refresh_token();

    state
        .ref_token_service
        .sync_refresh_token(user.id, &refresh_token)
        .await?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
    }))
}

pub async fn foo(AuthUser { user }: AuthUser) -> Json<FooData> {
    Json(FooData {
        message: String::from("foo"),
    })
}
