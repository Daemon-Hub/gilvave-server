use axum::http::{HeaderMap, header};

use crate::{dto::users::auth::Claims, error::CoreError, security::jwt::decode_jwt};

pub fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    for val in headers.get_all(header::COOKIE) {
        if let Ok(cookie_str) = val.to_str() {
            for pair in cookie_str.split(';') {
                let pair = pair.trim();
                if let Some((k, v)) = pair.split_once('=')
                    && k.trim() == name
                    && !v.trim().is_empty()
                {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_query_token(query: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=')
            && (k == "token" || k == "access_token")
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

pub fn extract_token_from_headers(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    if let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        && let Some(token) = auth_header.strip_prefix("Bearer ")
    {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(cookie_token) = extract_cookie(headers, "access_token") {
        return Some(cookie_token);
    }

    if let Some(q) = query
        && let Some(query_token) = extract_query_token(q)
    {
        return Some(query_token);
    }

    if let Some(proto) = headers
        .get("sec-websocket-protocol")
        .and_then(|h| h.to_str().ok())
    {
        let p = proto
            .split(',')
            .map(|s| s.trim())
            .find(|s| s.starts_with("ey"))
            .unwrap_or(proto);
        return Some(p.to_string());
    }

    None
}

/// Извлечение проверенных Claims: сначала из X-User-Claims (шлюз), иначе fallback на верификацию JWT
pub fn extract_claims_from_headers(
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<Claims, CoreError> {
    if let Some(claims_hdr) = headers.get("x-user-claims")
        && let Ok(claims_str) = claims_hdr.to_str()
        && let Ok(claims) = serde_json::from_str::<Claims>(claims_str)
    {
        return Ok(claims);
    }

    let token = extract_token_from_headers(headers, query)
        .ok_or_else(|| CoreError::Unauthorized("Missing authorization token".to_string()))?;

    decode_jwt(&token).map_err(|_| CoreError::Unauthorized("Invalid token".to_string()))
}

// #[allow(dead_code)]
// #[derive(Clone)]
// pub struct AuthUser(pub User, pub Claims);

// impl<S> FromRequestParts<S> for AuthUser
// where
//     S: Send + Sync,
//     UsersService: FromRef<S>,
// {
//     type Rejection = CoreError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let auth_header = parts
//             .headers
//             .get(header::AUTHORIZATION)
//             .and_then(|header| header.to_str().ok());

//         let token = if let Some(header_val) = auth_header {
//             header_val
//                 .strip_prefix("Bearer ")
//                 .ok_or(CoreError::Unauthorized(
//                     "Invalid authorization header format".to_string(),
//                 ))?
//                 .to_string()
//         } else if let Some(cookie_token) = extract_cookie(&parts.headers, "access_token") {
//             cookie_token
//         } else if let Some(query_token) = parts.uri.query().and_then(extract_query_token) {
//             query_token
//         } else if let Some(proto) = parts
//             .headers
//             .get("sec-websocket-protocol")
//             .and_then(|h| h.to_str().ok())
//         {
//             let p = proto
//                 .split(',')
//                 .map(|s| s.trim())
//                 .find(|s| s.starts_with("ey"))
//                 .unwrap_or(proto);
//             p.to_string()
//         } else {
//             return Err(CoreError::Unauthorized(
//                 "Missing authorization header".to_string(),
//             ));
//         };

//         let payload =
//             decode_jwt(&token).map_err(|_| CoreError::Unauthorized("Invalid token".to_string()))?;

//         let users = UsersService::from_ref(state);

//         let is_blacklisted = users
//             .sessions
//             .is_token_blacklisted(&payload.jti)
//             .await
//             .map_err(|_| {
//                 CoreError::InternalServerError(
//                     "Error occurred while checking token blacklist".to_string(),
//                 )
//             })?;
//         if is_blacklisted {
//             return Err(CoreError::Unauthorized("Token is blacklisted".to_string()));
//         }

//         let user = users
//             .find_by_id(payload.sub)
//             .await
//             .map_err(|_| {
//                 CoreError::InternalServerError("Error occurred while fetching user".to_string())
//             })?
//             .ok_or(CoreError::Forbidden("User not found".to_string()))?;

//         if !user.is_active {
//             return Err(CoreError::Unauthorized("User is inactive".to_string()));
//         }

//         Ok(Self(user, payload))
//     }
// }
