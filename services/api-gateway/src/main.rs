mod auth;
mod circuit_breaker;
mod proxy;
mod rate_limiter;

use mimalloc::MiMalloc;
use pingora::{proxy::http_proxy_service, server::Server};
use std::path::Path;

use crate::proxy::GilvaveApiGateway;
use gilvave_settings::{settings, setup_settings};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .compact()
        .init();

    setup_settings();

    tracing::info!("╔════════════════════════════════════════════════════════════════╗");
    tracing::info!("║            ⚡ GILVAVE API GATEWAY (Pingora / Rust) ⚡           ║");
    tracing::info!("╚════════════════════════════════════════════════════════════════╝");
    tracing::info!(
        "[GATEWAY] Listening on:     {}",
        settings!().gateway_listen_addr
    );
    tracing::info!(
        "[GATEWAY] Upstream HTTP:    {}",
        settings!().http_api_upstream
    );
    tracing::info!(
        "[GATEWAY] Upstream WS:      {}",
        settings!().ws_gateway_upstream
    );
    tracing::info!("[GATEWAY] Edge CORS:        Enabled");
    tracing::info!("[GATEWAY] Edge Auth:        Enabled (JWT + Redis Revocation Check)");
    tracing::info!("[GATEWAY] Rate Limiter:     Enabled (10 req/s per IP)");
    tracing::info!("[GATEWAY] Payload Limit:    Enabled (10 MB)");
    tracing::info!("[GATEWAY] Circuit Breaker:  Enabled");

    let mut server = Server::new(None)?;
    server.bootstrap();

    let gateway = GilvaveApiGateway::new(
        settings!().http_api_upstream.to_string(),
        settings!().ws_gateway_upstream.to_string(),
    );
    let mut proxy = http_proxy_service(&server.configuration, gateway);

    let cert_candidates = [
        ("certs/cert.pem", "certs/key.pem"),
        (
            "services/api-gateway/certs/cert.pem",
            "services/api-gateway/certs/key.pem",
        ),
        ("../certs/cert.pem", "../certs/key.pem"),
    ];

    let found_cert = cert_candidates
        .iter()
        .find(|(c, k)| Path::new(c).exists() && Path::new(k).exists());

    if let Some((cert, key)) = found_cert {
        proxy.add_tls(settings!().gateway_listen_addr, cert, key)?;
        tracing::info!(
            "[GATEWAY] TLS/HTTPS enabled on {}",
            settings!().gateway_listen_addr
        );
    } else {
        proxy.add_tcp(settings!().gateway_listen_addr);
        tracing::info!(
            "[GATEWAY] Plain TCP/HTTP enabled on {}",
            settings!().gateway_listen_addr
        );
    }

    server.add_service(proxy);
    server.run_forever();
}
