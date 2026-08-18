use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use gilvave_core::ids::{SessionId, UserId};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Result as JwtResult,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use gilvave_settings::settings;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,
    pub sid: SessionId,
    pub exp: i64,
    pub jti: Uuid,
}

pub fn create_jwt(session_id: SessionId, user_id: UserId) -> JwtResult<String> {
    let claims = Claims {
        sub: user_id,
        sid: session_id,
        exp: settings!().access_token_expire(),
        jti: Uuid::new_v4(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings!().secret.as_bytes()),
    )
}

pub fn verify_jwt(token: &str) -> JwtResult<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(settings!().secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}

pub fn generate_refresh_token() -> String {
    let mut bytes = vec![0u8; 64];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
