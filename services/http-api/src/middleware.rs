use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use gilvave_core::error::CoreError;
use gilvave_infra::security::auth::AuthUser;
use std::net::{IpAddr, SocketAddr};

use crate::state::AppState;

pub async fn client_ip_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Приоритет: X-Forwarded-For (если за доверенным прокси) → X-Real-IP → сокет
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
        })
        .unwrap_or_else(|| addr.ip());

    req.extensions_mut().insert(ip);
    next.run(req).await
}

pub async fn tracing_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let res = next.run(req).await;

    let status = res.status();
    if status.is_server_error() {
        tracing::error!("[HTTP] {method} {uri} -> {status}");
    } else {
        tracing::info!("[HTTP] {method} {uri} -> {status}");
    }

    res
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    AuthUser(user, claims): AuthUser,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, CoreError> {
    let is_revoked = state
        .redis
        .get_session_revocation_status(claims.sid)
        .await?;

    if is_revoked {
        return Err(CoreError::Unauthorized("Session revoked".to_string()));
    }

    req.extensions_mut().insert(claims.sid);
    req.extensions_mut().insert(user);
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
