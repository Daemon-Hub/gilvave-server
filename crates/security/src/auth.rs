use axum::http::{HeaderMap, header};
use gilvave_core::{dto::users::auth::Claims, error::CoreError};

use crate::jwt::decode_jwt;

pub fn extract_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    for val in headers.get_all(header::COOKIE) {
        if let Ok(cookie_str) = val.to_str() {
            for pair in cookie_str.split(';') {
                let pair = pair.trim();
                if let Some((k, v)) = pair.split_once('=')
                    && k.trim() == name
                    && !v.trim().is_empty()
                {
                    return Some(v.trim());
                }
            }
        }
    }
    None
}

fn extract_query_token(query: &str) -> Option<&str> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=')
            && (k == "token" || k == "access_token")
            && !v.is_empty()
        {
            return Some(v);
        }
    }
    None
}

pub fn extract_token_from_headers<'a>(
    headers: &'a HeaderMap,
    query: Option<&'a str>,
) -> Option<&'a str> {
    if let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        && let Some(token) = auth_header.strip_prefix("Bearer ")
    {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
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
        return Some(p);
    }

    None
}

pub fn extract_claims_from_headers(
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<Claims, CoreError> {
    if let Some(claims_hdr) = headers.get("x-user-claims")
        && let Ok(claims) = serde_json::from_slice::<Claims>(claims_hdr.as_bytes())
    {
        return Ok(claims);
    }

    let token = extract_token_from_headers(headers, query)
        .ok_or_else(|| CoreError::Unauthorized("Missing authorization token".to_string()))?;

    decode_jwt(token).map_err(|_| CoreError::Unauthorized("Invalid token".to_string()))
}
