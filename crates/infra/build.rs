fn main() {
    dotenvy::dotenv().ok();

    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        println!("cargo:rustc-env=DATABASE_URL={}", db_url);
    }
}
