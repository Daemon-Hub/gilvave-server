use sqlx::{PgPool, postgres::PgPoolOptions};

use gilvave_settings::settings;

pub async fn init_db() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(settings!().database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    Ok(pool)
}
