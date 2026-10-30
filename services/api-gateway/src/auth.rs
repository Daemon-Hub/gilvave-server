use pingora::prelude::Session;

/// Проверка: является ли путь публичным
pub fn is_public_path(path: &str) -> bool {
    matches!(
        path,
        "/users/login"
            | "/api/users/login"
            | "/users/auth/login"
            | "/api/users/auth/login"
            | "/users/register/validate"
            | "/api/users/register/validate"
            | "/users/auth/register/validate"
            | "/api/users/auth/register/validate"
            | "/users/register/confirm"
            | "/api/users/register/confirm"
            | "/users/auth/register/confirm"
            | "/api/users/auth/register/confirm"
            | "/health"
            | "/ping"
    )
}

/// Извлечение токена из заголовков (Authorization, Cookie, Sec-WebSocket-Protocol) или Query-параметров.
pub fn extract_token(session: &Session) -> Option<&str> {
    let req_header = session.req_header();

    // Authorization: Bearer <token>
    if let Some(auth_val) = req_header.headers.get("Authorization")
        && let Ok(auth_str) = auth_val.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    // Cookie: access_token=<token>
    if let Some(cookie_val) = req_header.headers.get("Cookie")
        && let Ok(cookie_str) = cookie_val.to_str()
    {
        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if let Some((k, v)) = pair.split_once('=')
                && k.trim() == "access_token"
            {
                let trimmed = v.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
    }

    // Query params: ?token=... или ?access_token=...
    if let Some(query) = req_header.uri.query() {
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=')
                && (k == "token" || k == "access_token")
                && !v.is_empty()
            {
                return Some(v);
            }
        }
    }

    // Sec-WebSocket-Protocol (WebSocket handshake)
    if let Some(proto_val) = req_header.headers.get("Sec-WebSocket-Protocol")
        && let Ok(proto_str) = proto_val.to_str()
    {
        for item in proto_str.split(',') {
            let item = item.trim();
            if item.starts_with("ey") {
                return Some(item);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_paths() {
        assert!(is_public_path("/users/login"));
        assert!(is_public_path("/api/users/login"));
        assert!(is_public_path("/users/auth/login"));
        assert!(is_public_path("/api/users/auth/login"));
        assert!(is_public_path("/users/register/validate"));
        assert!(is_public_path("/users/auth/register/validate"));
        assert!(is_public_path("/users/register/confirm"));
        assert!(is_public_path("/users/auth/register/confirm"));
        assert!(is_public_path("/health"));
        assert!(is_public_path("/ping"));

        assert!(!is_public_path("/users/@me"));
        assert!(!is_public_path("/servers"));
        assert!(!is_public_path("/ws"));
    }
}
