use deadpool_postgres::{
    self as pgpool, Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime,
};
use tokio_postgres::{NoTls, Row};

use gilvave_core::error::DatabaseError;
use gilvave_settings::settings;

refinery::embed_migrations!("../../migrations");

macro_rules! db_methods {
    (
        $(
            $name:ident: $return_type:ty
        ),* $(,)?
    ) => {
        $(
            pub async fn $name(
                &self,
                sql: &'static str,
                params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
            ) -> Result<$return_type, DatabaseError> {
                let client = self.pool.get().await?;
                let stmt = client.prepare_cached(sql).await?;
                Ok(client.$name(&stmt, params).await?)
            }
        )*
    };
}

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn get_client(&self) -> Result<pgpool::Object, DatabaseError> {
        Ok(self.pool.get().await?)
    }

    /// Pre-warm: создаём соединения заранее, чтобы первые запросы не ждали.
    pub async fn warmup(&self, n: usize) {
        let mut handles = Vec::with_capacity(n);
        for _ in 0..n {
            let pool = self.pool.clone();
            handles.push(tokio::spawn(async move {
                let _ = pool.get().await;
            }));
        }
        for h in handles {
            let _ = h.await;
        }
        tracing::info!("[DB] Database pool pre-warmed with {n} connections");
    }

    db_methods! {
        query: Vec<Row>,
        query_one: Row,
        query_opt: Option<Row>,
        execute: u64,
    }
}

pub async fn init_db() -> Result<Database, Box<dyn std::error::Error + Send + Sync>> {
    let mut cfg = Config::new();
    cfg.url = Some(settings!().database_url.to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Verified,
    });
    // TCP Keepalive - чтобы сеть не рубила простаивающие соединения
    // cfg.keepalives = Some(true);
    // TCP PING каждые 30 сек
    // cfg.keepalives_idle = Some(std::time::Duration::from_secs(30));

    let pool_cfg = PoolConfig {
        max_size: 20,
        ..Default::default()
    };
    cfg.pool = Some(pool_cfg);

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    let (mut client, connection) = tokio_postgres::connect(settings!().database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("[DB] Database connection error: {e}");
        }
    });
    migrations::runner().run_async(&mut client).await?;

    Ok(Database { pool })
}
