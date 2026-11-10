use async_trait::async_trait;
use bytes::Bytes;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use pingora::upstreams::peer::Peer;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::auth::{extract_token, is_public_path};
use crate::circuit_breaker::CircuitBreaker;
use crate::rate_limiter::RateLimiter;
use gilvave_redis::RedisService;
use gilvave_security::jwt::decode_jwt;

const MAX_PAYLOAD_SIZE: usize = 105 * 1024 * 1024;

const RATE_LIMIT_BODY: &[u8] = b"{\"error\":\"Too Many Requests: Rate limit exceeded\"}";
const RATE_LIMIT_LEN: &str = "48";

const PAYLOAD_TOO_LARGE_BODY: &[u8] = b"{\"error\":\"Payload Too Large: Maximum size is 100MB\"}";
const PAYLOAD_TOO_LARGE_LEN: &str = "52";

const CIRCUIT_BREAKER_BODY: &[u8] = b"{\"error\":\"Service temporarily unavailable. Circuit breaker is open. Please try again in 5 seconds.\"}";
const CIRCUIT_BREAKER_LEN: &str = "105";

const UNAUTHORIZED_BODY: &[u8] = b"{\"error\":\"Unauthorized: Invalid or revoked token\"}";
const UNAUTHORIZED_LEN: &str = "49";

#[inline]
fn format_ip(ip: IpAddr, buf: &mut [u8; 45]) -> &str {
    match ip {
        IpAddr::V4(v4) => {
            use std::io::Write;
            let mut cursor = std::io::Cursor::new(&mut buf[..]);
            let _ = write!(cursor, "{}", v4);
            let len = cursor.position() as usize;
            std::str::from_utf8(&buf[..len]).unwrap_or("127.0.0.1")
        }
        IpAddr::V6(v6) => {
            use std::io::Write;
            let mut cursor = std::io::Cursor::new(&mut buf[..]);
            let _ = write!(cursor, "{}", v6);
            let len = cursor.position() as usize;
            std::str::from_utf8(&buf[..len]).unwrap_or("::1")
        }
    }
}

fn attach_cors_headers(resp: &mut ResponseHeader, session: &Session) {
    if let Some(origin) = session.req_header().headers.get("Origin") {
        let _ = resp.append_header("Access-Control-Allow-Origin", origin.as_bytes());
        let _ = resp.append_header("Access-Control-Allow-Credentials", "true");
        let _ = resp.append_header("Vary", "Origin");
    }
}

pub struct GilvaveApiGateway {
    http_upstream: String,
    ws_upstream: String,
    http_breaker: CircuitBreaker,
    ws_breaker: CircuitBreaker,
    rate_limiter: RateLimiter,
    redis: OnceCell<RedisService>,
}

impl GilvaveApiGateway {
    pub fn new(http_upstream: String, ws_upstream: String) -> Self {
        Self {
            http_upstream,
            ws_upstream,
            http_breaker: CircuitBreaker::new("HTTP-API", 5, 5),
            ws_breaker: CircuitBreaker::new("WS-GATEWAY", 5, 5),
            rate_limiter: RateLimiter::new(10, Duration::from_secs(1)),
            redis: OnceCell::new(),
        }
    }

    async fn get_redis(&self) -> Result<&RedisService, ()> {
        self.redis
            .get_or_try_init(|| async {
                RedisService::new().await.map_err(|e| {
                    tracing::error!("[GATEWAY REDIS] Failed to initialize Redis connection: {e}");
                })
            })
            .await
    }
}

pub struct GatewayContext {
    pub start_time: Instant,
    pub user_id: Option<String>,
    pub is_ws: bool,
    pub request_id: Uuid,
    pub client_ip: IpAddr,
}

#[async_trait]
impl ProxyHttp for GilvaveApiGateway {
    type CTX = GatewayContext;

    fn new_ctx(&self) -> Self::CTX {
        GatewayContext {
            start_time: Instant::now(),
            user_id: None,
            is_ws: false,
            request_id: Uuid::new_v4(),
            client_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        }
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        ctx.is_ws = path.starts_with("/ws");
        let target_addr = if ctx.is_ws {
            &self.ws_upstream
        } else {
            &self.http_upstream
        };

        let mut peer = Box::new(HttpPeer::new(target_addr, false, String::new()));
        peer.options.connection_timeout = Some(Duration::from_secs(5));
        if ctx.is_ws {
            peer.options.read_timeout = None;
            peer.options.write_timeout = None;
            peer.options.idle_timeout = None;
        } else {
            peer.options.read_timeout = Some(Duration::from_secs(15));
            peer.options.write_timeout = Some(Duration::from_secs(10));
            peer.options.idle_timeout = Some(Duration::from_secs(60));
        }

        Ok(peer)
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        if let Some(client_addr) = session.client_addr()
            && let Some(inet) = client_addr.as_inet()
        {
            ctx.client_ip = inet.ip();
        }

        let mut ip_buf = [0u8; 45];
        let ip_str = format_ip(ctx.client_ip, &mut ip_buf);

        let mut req_id_buf = [0u8; 36];
        let req_id_str = ctx.request_id.hyphenated().encode_lower(&mut req_id_buf);

        {
            let req_mut = session.req_header_mut();
            let _ = req_mut.remove_header("x-user-id");
            let _ = req_mut.remove_header("x-session-id");
            let _ = req_mut.remove_header("x-user-data");
            let _ = req_mut.remove_header("x-user-claims");

            let _ = req_mut.append_header("X-Real-IP", ip_str);
            let _ = req_mut.append_header("X-Forwarded-For", ip_str);
            let _ = req_mut.append_header("X-Request-ID", &*req_id_str);
        }

        if !self.rate_limiter.check(ctx.client_ip) {
            tracing::warn!(
                "[RATE LIMIT] IP {} exceeded limit on {}",
                ctx.client_ip,
                session.req_header().uri.path()
            );

            let mut resp = ResponseHeader::build(429, None)?;
            attach_cors_headers(&mut resp, session);
            let _ = resp.append_header("Content-Type", "application/json");
            let _ = resp.append_header("Retry-After", "1");
            let _ = resp.append_header("X-Request-ID", &*req_id_str);
            let body = Bytes::from_static(RATE_LIMIT_BODY);
            let _ = resp.append_header("Content-Length", RATE_LIMIT_LEN);

            session.write_response_header(Box::new(resp), false).await?;
            session.write_response_body(Some(body), true).await?;
            return Ok(true);
        }

        if let Some(cl_val) = session.req_header().headers.get("Content-Length")
            && let Ok(cl_str) = cl_val.to_str()
            && let Ok(cl) = cl_str.parse::<usize>()
            && cl > MAX_PAYLOAD_SIZE
        {
            let mut resp = ResponseHeader::build(413, None)?;
            attach_cors_headers(&mut resp, session);
            let _ = resp.append_header("Content-Type", "application/json");
            let _ = resp.append_header("X-Request-ID", &*req_id_str);
            let body = Bytes::from_static(PAYLOAD_TOO_LARGE_BODY);
            let _ = resp.append_header("Content-Length", PAYLOAD_TOO_LARGE_LEN);

            session.write_response_header(Box::new(resp), false).await?;
            session.write_response_body(Some(body), true).await?;
            return Ok(true);
        }

        let is_ws = session.req_header().uri.path().starts_with("/ws");

        if is_ws {
            session.as_downstream_mut().set_read_timeout(None);
            session.as_downstream_mut().set_write_timeout(None);
        }

        let breaker = if is_ws {
            &self.ws_breaker
        } else {
            &self.http_breaker
        };
        let (allowed, status) = breaker.can_execute();

        if !allowed {
            tracing::warn!(
                "[CIRCUIT BREAKER] Request to {} blocked by {:?} breaker!",
                session.req_header().uri.path(),
                status
            );

            let mut resp = ResponseHeader::build(503, None)?;
            attach_cors_headers(&mut resp, session);
            let _ = resp.append_header("Content-Type", "application/json");
            let _ = resp.append_header("Retry-After", "5");
            let _ = resp.append_header("X-Request-ID", &*req_id_str);
            let body = Bytes::from_static(CIRCUIT_BREAKER_BODY);
            let _ = resp.append_header("Content-Length", CIRCUIT_BREAKER_LEN);

            session.write_response_header(Box::new(resp), false).await?;
            session.write_response_body(Some(body), true).await?;
            return Ok(true);
        }

        if session.req_header().method == "OPTIONS" {
            let origin = session
                .req_header()
                .headers
                .get("Origin")
                .and_then(|o| o.to_str().ok())
                .unwrap_or("*")
                .to_string();

            let mut resp = ResponseHeader::build(204, None)?;
            let _ = resp.append_header("Access-Control-Allow-Origin", &origin);
            let _ = resp.append_header("Access-Control-Allow-Credentials", "true");
            let _ = resp.append_header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, PATCH, OPTIONS",
            );
            let _ = resp.append_header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, X-Requested-With, Accept, Sec-WebSocket-Protocol, X-Request-ID",
            );
            let _ = resp.append_header("Access-Control-Max-Age", "86400");
            let _ = resp.append_header("X-Request-ID", &*req_id_str);
            let _ = resp.append_header("Content-Length", "0");

            session.write_response_header(Box::new(resp), true).await?;
            return Ok(true);
        }

        let is_public = is_public_path(session.req_header().uri.path());

        if !is_public {
            let token_opt = extract_token(session);

            let auth_result = match token_opt {
                Some(token_str) => match decode_jwt(token_str) {
                    Ok(claims) => {
                        let is_revoked = if let Ok(redis) = self.get_redis().await {
                            match redis.get_session_revocation_status(claims.sid).await {
                                Ok(true) => true,
                                Ok(false) => false,
                                Err(e) => {
                                    tracing::error!(
                                        "[GATEWAY REDIS] Error checking session revocation: {e}"
                                    );
                                    false
                                }
                            }
                        } else {
                            false
                        };

                        if is_revoked {
                            tracing::warn!("[GATEWAY AUTH] Session {} revoked", claims.sid);
                            Err("Session revoked".to_string())
                        } else {
                            let mut user_id_buf = [0u8; 36];
                            let user_id_str = claims.sub.encode_hyphenated(&mut user_id_buf).to_string();
                            let claims_json = serde_json::to_string(&claims).unwrap_or_default();
                            Ok((user_id_str, claims_json))
                        }
                    }
                    Err(e) => Err(format!("Invalid JWT: {e}")),
                },
                None => Err("Missing authentication token".to_string()),
            };

            match auth_result {
                Ok((user_id_str, claims_json)) => {
                    let req_mut = session.req_header_mut();
                    let _ = req_mut.append_header("X-User-Claims", &claims_json);
                    let _ = req_mut.append_header("X-User-Id", &user_id_str);

                    ctx.user_id = Some(user_id_str);
                }
                Err(err_msg) => {
                    tracing::warn!(
                        "[GATEWAY AUTH] Rejected on {}: {}",
                        session.req_header().uri.path(),
                        err_msg
                    );
                    let mut resp = ResponseHeader::build(401, None)?;
                    attach_cors_headers(&mut resp, session);
                    let _ = resp.append_header("Content-Type", "application/json");
                    let _ = resp.append_header("X-Request-ID", &*req_id_str);
                    let body = Bytes::from_static(UNAUTHORIZED_BODY);
                    let _ = resp.append_header("Content-Length", UNAUTHORIZED_LEN);

                    session.write_response_header(Box::new(resp), false).await?;
                    session.write_response_body(Some(body), true).await?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        peer: &HttpPeer,
        ctx: &mut Self::CTX,
        e: Box<pingora::Error>,
    ) -> Box<pingora::Error> {
        let breaker = if ctx.is_ws {
            &self.ws_breaker
        } else {
            &self.http_breaker
        };
        breaker.record_failure();

        tracing::error!(
            "[GATEWAY CONNECT ERROR] Failed to connect to upstream {} (peer: {}): {}",
            breaker.name(),
            peer.address(),
            e
        );

        e
    }

    async fn upstream_response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        if ctx.is_ws || upstream_response.status.as_u16() == 101 {
            session.as_downstream_mut().set_read_timeout(None);
            session.as_downstream_mut().set_write_timeout(None);
        }

        let breaker = if ctx.is_ws {
            &self.ws_breaker
        } else {
            &self.http_breaker
        };

        if upstream_response.status.as_u16() < 500 {
            breaker.record_success();
        } else if upstream_response.status.as_u16() >= 500 {
            breaker.record_failure();
        }

        Ok(())
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        let mut req_id_buf = [0u8; 36];
        let req_id_str = ctx.request_id.hyphenated().encode_lower(&mut req_id_buf);
        let _ = upstream_response.append_header("X-Request-ID", &*req_id_str);

        if let Some(origin) = session.req_header().headers.get("Origin") {
            let _ = upstream_response.remove_header("Access-Control-Allow-Origin");
            let _ = upstream_response.remove_header("Access-Control-Allow-Credentials");

            let _ =
                upstream_response.append_header("Access-Control-Allow-Origin", origin.as_bytes());
            let _ = upstream_response.append_header("Access-Control-Allow-Credentials", "true");
            let _ = upstream_response.append_header("Vary", "Origin");
        }

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let req = session.req_header();
        let path = req.uri.path();
        let method = req.method.as_str();
        let duration = ctx.start_time.elapsed();
        let status = session
            .response_written()
            .map(|r| r.status.as_u16())
            .unwrap_or(0);

        let user_info = ctx.user_id.as_deref().unwrap_or("anonymous");

        let mut req_id_buf = [0u8; 36];
        let req_id_str = ctx.request_id.hyphenated().encode_lower(&mut req_id_buf);

        tracing::info!(
            "[GATEWAY] [{}] {} {} [{}] from {} -> status: {} ({:?})",
            req_id_str,
            method,
            path,
            user_info,
            ctx.client_ip,
            status,
            duration
        );
    }
}
