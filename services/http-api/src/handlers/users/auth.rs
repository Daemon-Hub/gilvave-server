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
        User,
        auth::{CreateUserInfo, LoginInfo},
        sessions::SessionCreateInfo,
    },
    error::CoreError,
    validation::validate_username,
};
use gilvave_infra::security::jwt::{
    Claims, create_jwt, generate_refresh_token, hash_refresh_token,
};

use crate::{handlers::users::build_tokens_response, state::AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CreateUserInfo>,
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

    state.users.auth.create(body).await?;

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
    Extension(user): Extension<User>,
    Extension(clm): Extension<Claims>,
) -> Result<Response, CoreError> {
    let (_, res) = tokio::join!(
        state.redis.revoke_session(clm.sid, clm.exp),
        state.users.sessions.delete_by_id(user.id, clm.sid),
    );
    res?;

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
