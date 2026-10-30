use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use gilvave_core::{error::CoreError, security::auth::extract_claims_from_headers};
use std::net::{IpAddr, SocketAddr};

use crate::state::AppState;

pub async fn client_ip_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
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
    let req_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| "-".to_string());

    let res = next.run(req).await;

    let status = res.status();
    if status.is_server_error() {
        tracing::error!("[HTTP] [{req_id}] {method} {uri} -> {status}");
    } else {
        tracing::info!("[HTTP] [{req_id}] {method} {uri} -> {status}");
    }

    res
}

pub async fn auth_middleware(
    State(_state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, CoreError> {
    let claims = extract_claims_from_headers(req.headers(), req.uri().query())?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
