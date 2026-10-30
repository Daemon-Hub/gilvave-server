use axum::{
    Extension, Json,
    extract::{Query, State},
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    response::{Html, IntoResponse, Response},
};
use std::net::IpAddr;

use gilvave_core::{
    dto::users::{
        auth::{Claims, ConfirmParams, CreateUserInfo, LoginInfo}, sessions::SessionCreateInfo,
    }, error::CoreError, security::jwt::{create_jwt, generate_url_safe_token, hash_refresh_token}, validation::{validate_email, validate_username},
};

use crate::{handlers::users::build_tokens_response, state::AppState};

pub async fn register_validate(
    State(state): State<AppState>,
    Json(mut body): Json<CreateUserInfo>,
) -> Result<(), CoreError> {
    validate_username(&body.username).map_err(|e| CoreError::BadRequest(e.to_string()))?;

    let email = validate_email(&body.email).map_err(CoreError::BadRequest)?;

    if state.users.find_by_email(&email).await?.is_some() {
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

    body.email = email;

    let token = generate_url_safe_token();
    state
        .redis
        .set_email_validation_info(&token, &body.to_slice())
        .await?;

    let state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = state
            .email
            .send_email_validate_msg(&body.email, token)
            .await
        {
            tracing::error!("Failed to send confirmation email: {e}");
        }
    });

    Ok(())
}

pub async fn register_confirm(
    State(state): State<AppState>,
    Query(params): Query<ConfirmParams>,
) -> Result<Html<&'static str>, CoreError> {
    let raw_info = state.redis.get_email_validated_info(&params.token).await?;
    let user_info = CreateUserInfo::from_slice(raw_info);

    state.users.auth.create(user_info).await?;

    Ok(Html(gilvave_settings::constants::CONFIRM_SUCCESS_HTML))
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
        .await
    {
        return Err(CoreError::Unauthorized("Invalid password".to_string()));
    }

    let is_web = body.device_info.get("client").and_then(|v| v.as_str()) == Some("web");

    let refresh_token = generate_url_safe_token();
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

    let access_token = create_jwt(session_id, user.id)
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    Ok(build_tokens_response(access_token, refresh_token, is_web))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, CoreError> {
    let (_, res) = tokio::join!(
        state.redis.revoke_session(claims.sid, claims.exp),
        state.users.sessions.delete_by_id(claims.sub, claims.sid),
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
