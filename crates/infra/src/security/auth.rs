use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};

use crate::{jwt::verify_jwt, service::user::UserService};
use gilvave_core::{dto::user::User, error::CoreError};

#[derive(Clone)]
pub struct AuthUser(pub User);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    UserService: FromRef<S>,
{
    type Rejection = CoreError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let token = if let Some(header_val) = auth_header {
            header_val
                .strip_prefix("Bearer ")
                .ok_or(CoreError::Unauthorized(
                    "Invalid authorization header format".to_string(),
                ))?
                .to_string()
        } else if let Some(query) = parts.uri.query() {
            let mut found = None;
            for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    if k == "token" || k == "access_token" {
                        found = Some(v.to_string());
                        break;
                    }
                }
            }
            found.ok_or(CoreError::Unauthorized(
                "Missing authorization header".to_string(),
            ))?
        } else if let Some(proto) = parts
            .headers
            .get("sec-websocket-protocol")
            .and_then(|h| h.to_str().ok())
        {
            let p = proto
                .split(',')
                .map(|s| s.trim())
                .find(|s| s.starts_with("ey"))
                .unwrap_or(proto);
            p.to_string()
        } else {
            return Err(CoreError::Unauthorized(
                "Missing authorization header".to_string(),
            ));
        };

        let payload =
            verify_jwt(&token).map_err(|_| CoreError::Unauthorized("Invalid token".to_string()))?;

        let user_service = UserService::from_ref(state);

        let is_blacklisted = user_service
            .is_token_blacklisted(&payload.jti)
            .await
            .map_err(|_| {
                CoreError::InternalServerError(
                    "Error occurred while checking token blacklist".to_string(),
                )
            })?;
        if is_blacklisted {
            return Err(CoreError::Unauthorized("Token is blacklisted".to_string()));
        }

        let user = user_service
            .find_by_id(payload.sub)
            .await
            .map_err(|_| {
                CoreError::InternalServerError("Error occurred while fetching user".to_string())
            })?
            .ok_or(CoreError::Forbidden("User not found".to_string()))?;

        if !user.is_active {
            return Err(CoreError::Unauthorized("User is inactive".to_string()));
        }

        Ok(Self(user))
    }
}
