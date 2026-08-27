macro_rules! forward_env_var {
    ($var:expr) => {
        if let Ok(value) = std::env::var($var) {
            println!("cargo:rustc-env={}={}", $var, value);
        }
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    forward_env_var!("SECRET");
    forward_env_var!("DATABASE_URL");
    forward_env_var!("RABBITMQ_DEFAULT_URL");
    forward_env_var!("REDIS_URL");
    forward_env_var!("S3_URL");
    forward_env_var!("S3_ACCESS_KEY");
    forward_env_var!("S3_SECRET_KEY");
    Ok(())
}
