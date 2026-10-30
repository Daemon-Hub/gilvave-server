use axum::{
    Extension, Json,
    extract::{Path, State},
    http::header::HeaderMap,
    response::Response,
};
use std::net::IpAddr;

use gilvave_core::{
    dto::users::{auth::Claims, sessions::{RefreshTokenRequest, SessionView}}, error::CoreError, ids::SessionId, security::{
        auth::extract_cookie,
        jwt::{create_jwt, generate_url_safe_token, hash_refresh_token},
    },
};

use crate::{handlers::users::build_tokens_response, state::AppState};

pub async fn refresh_token(
    State(state): State<AppState>,
    Extension(ip_address): Extension<IpAddr>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<Response, CoreError> {
    let (raw_refresh_token, is_web) =
        if let Some(cookie_token) = extract_cookie(&headers, "refresh_token") {
            (cookie_token, true)
        } else if let Some(Json(req)) = body
            && !req.refresh_token.is_empty()
        {
            (req.refresh_token, false)
        } else {
            return Err(CoreError::Unauthorized("Missing refresh token".to_string()));
        };

    let refresh_token_hash = hash_refresh_token(&raw_refresh_token);
    let session = state
        .users
        .sessions
        .get_unexpired(&refresh_token_hash)
        .await
        .ok_or(CoreError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ))?;

    let access_token = create_jwt(session.id, session.user_id)
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    let refresh_token = generate_url_safe_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    state
        .users
        .sessions
        .update(session.user_id, session.id, refresh_token_hash, ip_address)
        .await?;

    Ok(build_tokens_response(access_token, refresh_token, is_web))
}

pub async fn get_sessions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<(SessionView, bool)>>, CoreError> {
    Ok(Json(
        state
            .users
            .sessions
            .get_list(claims.sub, claims.sid)
            .await?,
    ))
}

pub async fn delete_session_by_id(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<SessionId>,
) -> Result<(), CoreError> {
    let (_, res) = tokio::join!(
        state.redis.revoke_session(session_id, claims.exp),
        state.users.sessions.delete_by_id(claims.sub, session_id)
    );
    Ok(res?)
}

pub async fn delete_all_other_sessions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<(), CoreError> {
    let sids = state
        .users
        .sessions
        .delete_all_other(claims.sub, claims.sid)
        .await?;
    state.redis.revoke_sessions_list(&sids).await?;
    Ok(())
}
