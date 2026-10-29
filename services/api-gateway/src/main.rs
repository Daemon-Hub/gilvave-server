mod auth;
mod circuit_breaker;
mod proxy;

use mimalloc::MiMalloc;
use pingora::{proxy::http_proxy_service, server::Server};

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
    tracing::info!("[GATEWAY] Edge CORS:        Enabled (instant 204 preflight)");
    tracing::info!("[GATEWAY] Edge Auth:        Enabled (JWT verification on entry)");
    tracing::info!("[GATEWAY] Circuit Breaker:  Enabled (Closed / Open / Half-Open)");

    let mut server = Server::new(None)?;
    server.bootstrap();

    let gateway = GilvaveApiGateway::new(
        settings!().http_api_upstream.to_string(),
        settings!().ws_gateway_upstream.to_string(),
        settings!().secret.as_bytes(),
    );
    let mut proxy = http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp(settings!().gateway_listen_addr);

    server.add_service(proxy);
    server.run_forever();
}
