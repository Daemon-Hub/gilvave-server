use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};

use crate::{jwt::verify_jwt, service::user::UserService};
use gilvave_core::{dto::user::User, error::CoreError, ids::UserId};

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
            .and_then(|header| header.to_str().ok())
            .ok_or(CoreError::Unauthorized(
                "Missing authorization header".to_string(),
            ))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(CoreError::Unauthorized(
                "Invalid authorization header format".to_string(),
            ))?;

        let payload =
            verify_jwt(token).map_err(|_| CoreError::Unauthorized("Invalid token".to_string()))?;

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
            .find_by_id(UserId(payload.sub))
            .await
            .map_err(|_| CoreError::InternalServerError("Error occurred while fetching user".to_string()))?
            .ok_or(CoreError::Forbidden("User not found".to_string()))?;

        if !user.is_active {
            return Err(CoreError::Unauthorized("User is inactive".to_string()));
        }

        Ok(Self(user))
    }
}
