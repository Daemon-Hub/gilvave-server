use axum::{extract::ConnectInfo, http::Request, middleware::Next, response::Response};
use std::net::{IpAddr, SocketAddr};

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
