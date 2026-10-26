pub mod constants;

use std::sync::OnceLock;

pub struct Settings {
    // SECRET KEY
    pub secret: &'static str,

    // PostgreSQL
    pub database_url: &'static str,

    // RabbitMQ (Deprecated / Replaced by Apache Iggy)
    pub rmq_url: &'static str,

    // Apache Iggy
    pub iggy_url: &'static str,

    // Redis
    pub redis_url: &'static str,

    // S3
    pub s3_url: &'static str,
    pub s3_access_key: &'static str,
    pub s3_secret_key: &'static str,

    // JWT
    pub access_token_expire_minutes: time::Duration,
    pub refresh_token_expire_days: time::Duration,

    // E-MAIL
    pub email_validation_ttl: time::Duration,
    pub smtp_login: &'static str,
    pub smtp_secret: &'static str,
}

impl Settings {
    /// Возвращает время жизни access токена согласно установленным настройкам
    pub fn access_token_expire(&self) -> i64 {
        (time::OffsetDateTime::now_utc() + self.access_token_expire_minutes).unix_timestamp()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            secret: std::env!("SECRET"),
            database_url: std::env!("DATABASE_URL"),
            rmq_url: option_env!("RABBITMQ_DEFAULT_URL").unwrap_or(""),
            iggy_url: option_env!("IGGY_URL").unwrap_or("iggy://iggy:iggy_pass@127.0.0.1:8090"),
            redis_url: std::env!("REDIS_URL"),
            s3_url: std::env!("S3_URL"),
            s3_access_key: std::env!("S3_ACCESS_KEY"),
            s3_secret_key: std::env!("S3_SECRET_KEY"),
            access_token_expire_minutes: time::Duration::minutes(20),
            refresh_token_expire_days: time::Duration::days(30),
            email_validation_ttl: time::Duration::minutes(20),
            smtp_login: std::env!("SMTP_LOGIN"),
            smtp_secret: std::env!("SMTP_SECRET"),
        }
    }
}

pub fn setup_settings() {
    SETTINGS.set(Settings::default()).ok();
}

pub static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Макрос предоставляющий доступ к предустановленным переменным приложения
#[macro_export]
macro_rules! settings {
    () => {
        $crate::SETTINGS.get().unwrap()
    };
}
