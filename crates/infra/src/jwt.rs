use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gilvave_settings::settings;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub jti: Uuid,
}

pub fn create_jwt(user_id: &str) -> anyhow::Result<String> {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: settings!().access_token_expire(),
        jti: Uuid::new_v4(),
    };

    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings!().secret.as_bytes()),
    )?)
}

pub fn verify_jwt(token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(settings!().secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}

fn token_urlsafe(byte_count: usize) -> String {
    let mut bytes = vec![0u8; byte_count];
    OsRng.fill_bytes(&mut bytes[..]);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn generate_refresh_token() -> String {
    token_urlsafe(64)
}
