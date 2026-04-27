use std::sync::OnceLock;

pub struct Settings {
    // SECRET KEY
    pub secret: &'static str,

    // PostgreSQL
    pub database_url: &'static str,

    // RabbitMQ
    pub rmq_url: &'static str,

    // Redis
    pub redis_url: &'static str,

    // JWT
    pub access_token_expire_minutes: time::Duration,
    pub refresh_token_expire_days: time::Duration,
}

impl Settings {
    /// Возвращает время жизни access токена согласно установленным настройкам
    pub fn access_token_expire(&self) -> usize {
        (time::OffsetDateTime::now_utc() + self.access_token_expire_minutes).unix_timestamp()
            as usize
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            secret: std::env!("SECRET"),
            database_url: std::env!("DATABASE_URL"),
            rmq_url: std::env!("RABBITMQ_DEFAULT_URL"),
            redis_url: std::env!("REDIS_URL"),

            access_token_expire_minutes: time::Duration::minutes(20),
            refresh_token_expire_days: time::Duration::days(30),
        }
    }
}

pub fn setup_settings() {
    SETTINGS.set(Settings::default()).ok();
}

pub static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Макрос предоставляющий доступ к предустановленным настройкам системы
#[macro_export]
macro_rules! settings {
    () => {
        $crate::SETTINGS.get().unwrap()
    };
}
