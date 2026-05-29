fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    if let Ok(secret) = std::env::var("SECRET") {
        println!("{secret}");
        println!("cargo:rustc-env=SECRET={}", secret);
    }
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        println!("{db_url}");
        println!("cargo:rustc-env=DATABASE_URL={}", db_url);
    }
    if let Ok(rmq_url) = std::env::var("RABBITMQ_DEFAULT_URL") {
        println!("{rmq_url}");
        println!("cargo:rustc-env=RABBITMQ_DEFAULT_URL={}", rmq_url);
    }
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        println!("{redis_url}");
        println!("cargo:rustc-env=REDIS_URL={}", redis_url);
    }
    if let Ok(s3_url) = std::env::var("S3_URL") {
        println!("{s3_url}");
        println!("cargo:rustc-env=S3_URL={}", s3_url);
    }
    if let Ok(s3_access_key) = std::env::var("S3_ACCESS_KEY") {
        println!("cargo:rustc-env=S3_ACCESS_KEY={}", s3_access_key);
    }
    if let Ok(s3_secret_key) = std::env::var("S3_SECRET_KEY") {
        println!("cargo:rustc-env=S3_SECRET_KEY={}", s3_secret_key);
    }
    Ok(())
}
