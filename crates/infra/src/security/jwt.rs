use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use gilvave_core::ids::{SessionId, UserId};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Result as JwtResult,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use gilvave_settings::settings;

pub use gilvave_core::dto::users::auth::Claims;

pub fn create_jwt(session_id: SessionId, user_id: UserId) -> JwtResult<String> {
    create_jwt_with_user(session_id, user_id, "", "")
}

pub fn create_jwt_with_user(
    session_id: SessionId,
    user_id: UserId,
    username: &str,
    email: &str,
) -> JwtResult<String> {
    let claims = Claims {
        sub: user_id,
        sid: session_id,
        exp: settings!().access_token_expire(),
        jti: Uuid::new_v4(),
        username: username.to_string(),
        email: email.to_string(),
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

pub fn generate_url_safe_token() -> String {
    let mut bytes = vec![0u8; 64];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
