fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    if let Ok(db_url) = std::env::var("SECRET") {
        println!("cargo:rustc-env=SECRET={}", db_url);
    }
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        println!("cargo:rustc-env=DATABASE_URL={}", db_url);
    }
    if let Ok(db_url) = std::env::var("RABBITMQ_DEFAULT_URI") {
        println!("cargo:rustc-env=RABBITMQ_DEFAULT_URI={}", db_url);
    }
    Ok(())
}
