use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header, request::Parts},
};

use crate::{jwt::verify_jwt, service::user_service::UserService};
use gilvave_core::model::User;

#[derive(Clone)]
pub struct AuthUser {
    pub user: User,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    UserService: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        let token = auth_header.strip_prefix("Bearer ").ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid authorization header format",
        ))?;

        let payload =
            verify_jwt(token).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        let user_service = UserService::from_ref(state);

        if user_service.is_token_blacklisted(&payload.jti).await {
            return Err((StatusCode::UNAUTHORIZED, "Token is blacklisted"));
        }
        
        let user = user_service
            .find_by_id(payload.sub.into())
            .await
            .ok_or((StatusCode::FORBIDDEN, "User not found"))?;

        if !user.is_active {
            return Err((StatusCode::FORBIDDEN, "User is inactive"));
        }

        Ok(AuthUser { user })
    }
}
