fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    if let Ok(secret) = std::env::var("SECRET") {
        println!("cargo:rustc-env=SECRET={}", secret);
    }
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        println!("cargo:rustc-env=DATABASE_URL={}", db_url);
    }
    if let Ok(rmq_url) = std::env::var("RABBITMQ_DEFAULT_URL") {
        println!("cargo:rustc-env=RABBITMQ_DEFAULT_URL={}", rmq_url);
    }
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        println!("cargo:rustc-env=REDIS_URL={}", redis_url);
    }
    Ok(())
}
