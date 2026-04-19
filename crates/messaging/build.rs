fn main() {
    dotenvy::dotenv().ok();

    if let Ok(rmq_url) = std::env::var("RABBITMQ_DEFAULT_URI") {
        println!("cargo:rustc-env=RABBITMQ_DEFAULT_URI={}", rmq_url);
    }
}
