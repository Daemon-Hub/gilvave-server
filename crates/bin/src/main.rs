use axum::Router;
use jsonwebtoken::crypto::{rust_crypto::DEFAULT_PROVIDER, CryptoProvider};
use mimalloc::MiMalloc;
use std::sync::Arc;
use tokio::net::TcpListener;

use gilvave_gateway;
use gilvave_http_api;
use gilvave_infra;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;
    CryptoProvider::install_default(&DEFAULT_PROVIDER).unwrap();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = gilvave_infra::db::init_db(&db_url).await?;
    let pool = Arc::new(pool);

    let realtime = Arc::new(gilvave_realtime::Realtime::new());

    let app = Router::new()
        .merge(gilvave_gateway::router(realtime))
        .merge(gilvave_http_api::router(pool.clone()));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     tracing_subscriber::fmt().init();

//     let realtime = Arc::new(Realtime::new());
//     let app = router(realtime);

//     let listener = TcpListener::bind("0.0.0.0:3000").await?;
//     println!("Gateway running on ws://localhost:3000/ws");

//     axum::serve(listener, app).await.unwrap();

//     Ok(())
// }
