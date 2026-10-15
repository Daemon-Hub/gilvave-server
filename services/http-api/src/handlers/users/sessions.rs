use axum::{Extension, Json, extract::State, http::header::HeaderMap, response::Response};
use std::net::IpAddr;

use gilvave_core::{dto::users::sessions::RefreshTokenRequest, error::CoreError};
use gilvave_infra::{
    jwt::{create_jwt, generate_refresh_token, hash_refresh_token},
    security::auth::extract_cookie,
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

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    state
        .users
        .sessions
        .update(session.user_id, session.id, refresh_token_hash, ip_address)
        .await?;

    Ok(build_tokens_response(access_token, refresh_token, is_web))
}
