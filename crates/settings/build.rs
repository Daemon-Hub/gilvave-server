macro_rules! forward_env_vars {
    ($($var:expr),* $(,)?) => {
        $(
            if let Ok(value) = std::env::var($var) {
                println!("cargo:rustc-env={}={}", $var, value);
            }
        )*
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    forward_env_vars!(
        "SECRET",
        "DATABASE_URL",
        "RABBITMQ_DEFAULT_URL",
        "REDIS_URL",
        "S3_URL",
        "S3_ACCESS_KEY",
        "S3_SECRET_KEY",
        "SMTP_LOGIN",
        "SMTP_SECRET",
    );
    Ok(())
}
