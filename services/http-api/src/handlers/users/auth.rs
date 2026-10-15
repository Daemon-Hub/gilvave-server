use axum::{
    Extension, Json,
    extract::State,
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
};
use std::net::IpAddr;

use gilvave_core::{
    dto::users::{
        auth::{LoginInfo, RegisterRequest},
        sessions::{BlacklistInfo, SessionCreateInfo},
    },
    error::CoreError,
    validation::validate_username,
};
use gilvave_infra::{
    jwt::{create_jwt, generate_refresh_token, hash_refresh_token},
    security::auth::{AuthUser, extract_cookie},
};

use crate::{handlers::users::build_tokens_response, state::AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(), CoreError> {
    if let Err(msg) = validate_username(&body.username) {
        return Err(CoreError::BadRequest(msg.to_string()));
    }

    if state.users.find_by_email(&body.email).await?.is_some() {
        return Err(CoreError::Conflict(
            "The user with this email address exists.".to_string(),
        ));
    }

    if state
        .users
        .find_by_username(&body.username)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "The user with this username exists.".to_string(),
        ));
    }

    state
        .users
        .auth
        .create(&body.username, &body.email, &body.password)
        .await?;

    Ok(())
}

pub async fn login(
    State(state): State<AppState>,
    Extension(ip_address): Extension<IpAddr>,
    Json(body): Json<LoginInfo>,
) -> Result<Response, CoreError> {
    let user = match state.users.find_by_email(&body.email).await? {
        Some(u) => u,
        None => {
            return Err(CoreError::Unauthorized(
                "The user with this email address has not been found!".to_string(),
            ));
        }
    };

    if !state
        .users
        .auth
        .verify_password(&user.password_hash, &body.password)
    {
        return Err(CoreError::Unauthorized("Invalid password".to_string()));
    }

    let is_web = body.device_info.get("client").and_then(|v| v.as_str()) == Some("web");

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    let session_id = state
        .users
        .sessions
        .create(SessionCreateInfo {
            user_id: user.id,
            refresh_token_hash: refresh_token_hash.as_bytes(),
            device_info: body.device_info,
            ip_address,
        })
        .await?;

    let access_token = create_jwt(session_id, user.id).unwrap();

    Ok(build_tokens_response(access_token, refresh_token, is_web))
}

pub async fn logout(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
) -> Result<Response, CoreError> {
    state.users.sessions.delete(user.id).await?;
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(String::from)
        .or_else(|| extract_cookie(&headers, "access_token"))
        .ok_or_else(|| CoreError::Unauthorized("Missing authorization token".to_string()))?;

    state
        .users
        .sessions
        .blacklist_token(BlacklistInfo {
            token,
            user_id: user.id,
        })
        .await?;

    let mut resp_headers = HeaderMap::new();
    if let Ok(val) =
        HeaderValue::from_str("access_token=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax")
    {
        resp_headers.append(header::SET_COOKIE, val);
    }
    if let Ok(val) =
        HeaderValue::from_str("refresh_token=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax")
    {
        resp_headers.append(header::SET_COOKIE, val);
    }

    Ok((StatusCode::OK, resp_headers).into_response())
}
