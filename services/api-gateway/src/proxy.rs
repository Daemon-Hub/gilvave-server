use async_trait::async_trait;
use bytes::Bytes;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use pingora::upstreams::peer::Peer;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;

use crate::auth::{extract_token, is_public_path};
use crate::circuit_breaker::CircuitBreaker;
use crate::rate_limiter::RateLimiter;
use gilvave_core::security::jwt::decode_jwt;
use gilvave_redis::RedisService;

const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;

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
    pub request_id: String,
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
            request_id: uuid::Uuid::new_v4().to_string(),
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

        let ip_str = ctx.client_ip.to_string();
        {
            let req_mut = session.req_header_mut();
            let _ = req_mut.remove_header("x-user-id");
            let _ = req_mut.remove_header("x-session-id");
            let _ = req_mut.remove_header("x-user-data");
            let _ = req_mut.remove_header("x-user-claims");

            let _ = req_mut.append_header("X-Real-IP", &ip_str);
            let _ = req_mut.append_header("X-Forwarded-For", &ip_str);
            let _ = req_mut.append_header("X-Request-ID", &ctx.request_id);
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
            let _ = resp.append_header("X-Request-ID", &ctx.request_id);
            let body =
                Bytes::from_static(b"{\"error\":\"Too Many Requests: Rate limit exceeded\"}");
            let _ = resp.append_header("Content-Length", body.len().to_string());

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
            let _ = resp.append_header("X-Request-ID", &ctx.request_id);
            let body =
                Bytes::from_static(b"{\"error\":\"Payload Too Large: Maximum size is 10MB\"}");
            let _ = resp.append_header("Content-Length", body.len().to_string());

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
            let _ = resp.append_header("X-Request-ID", &ctx.request_id);
            let body = Bytes::from_static(
                b"{\"error\":\"Service temporarily unavailable. Circuit breaker is open. Please try again in 5 seconds.\"}",
            );
            let _ = resp.append_header("Content-Length", body.len().to_string());

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
            let _ = resp.append_header("X-Request-ID", &ctx.request_id);
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
                        if let Ok(redis) = self.get_redis().await {
                            match redis.get_session_revocation_status(claims.sid).await {
                                Ok(true) => {
                                    tracing::warn!("[GATEWAY AUTH] Session {} revoked", claims.sid);
                                    Err("Session revoked".to_string())
                                }
                                Ok(false) => {
                                    let user_id_str = claims.sub.0.to_string();
                                    let claims_json =
                                        serde_json::to_string(&claims).unwrap_or_default();
                                    Ok((user_id_str, claims_json))
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[GATEWAY REDIS] Error checking session revocation: {e}"
                                    );
                                    let user_id_str = claims.sub.0.to_string();
                                    let claims_json =
                                        serde_json::to_string(&claims).unwrap_or_default();
                                    Ok((user_id_str, claims_json))
                                }
                            }
                        } else {
                            let user_id_str = claims.sub.0.to_string();
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
                    let _ = resp.append_header("X-Request-ID", &ctx.request_id);
                    let body = Bytes::from_static(
                        b"{\"error\":\"Unauthorized: Invalid or revoked token\"}",
                    );
                    let _ = resp.append_header("Content-Length", body.len().to_string());

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
        let _ = upstream_response.append_header("X-Request-ID", &ctx.request_id);

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

        tracing::info!(
            "[GATEWAY] [{}] {} {} [{}] from {} -> status: {} ({:?})",
            ctx.request_id,
            method,
            path,
            user_info,
            ctx.client_ip,
            status,
            duration
        );
    }
}
