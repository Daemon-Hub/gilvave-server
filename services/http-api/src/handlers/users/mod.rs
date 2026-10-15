pub mod auth;
pub mod profile;
pub mod sessions;

pub use {auth::*, profile::*, sessions::*};

use axum::{
    Json,
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
};

use gilvave_core::dto::users::auth::AuthTokensResponse;
use gilvave_settings::settings;

pub fn build_tokens_response(
    access_token: String,
    refresh_token: String,
    is_web: bool,
) -> Response {
    if is_web {
        let access_max_age = settings!().access_token_expire_minutes.whole_seconds();
        let refresh_max_age = settings!().refresh_token_expire_days.whole_seconds();

        let access_cookie = format!(
            "access_token={access_token}; HttpOnly; Path=/; Max-Age={access_max_age}; SameSite=Lax"
        );
        let refresh_cookie = format!(
            "refresh_token={refresh_token}; HttpOnly; Path=/; Max-Age={refresh_max_age}; SameSite=Lax"
        );

        let mut headers = HeaderMap::new();
        if let Ok(val) = HeaderValue::from_str(&access_cookie) {
            headers.append(header::SET_COOKIE, val);
        }
        if let Ok(val) = HeaderValue::from_str(&refresh_cookie) {
            headers.append(header::SET_COOKIE, val);
        }

        (StatusCode::OK, headers).into_response()
    } else {
        Json(AuthTokensResponse {
            access_token,
            refresh_token,
        })
        .into_response()
    }
}
