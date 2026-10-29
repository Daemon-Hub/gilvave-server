use async_trait::async_trait;
use bytes::Bytes;
use jsonwebtoken::{DecodingKey, Validation, decode};
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use pingora::upstreams::peer::Peer;
use std::time::Instant;

use gilvave_core::dto::users::auth::{Claims, GatewayUserData};

use crate::auth::{extract_token, is_public_path};
use crate::circuit_breaker::CircuitBreaker;

pub struct GilvaveApiGateway {
    http_upstream: String,
    ws_upstream: String,
    decoding_key: DecodingKey,
    http_breaker: CircuitBreaker,
    ws_breaker: CircuitBreaker,
}

impl GilvaveApiGateway {
    pub fn new(http_upstream: String, ws_upstream: String, secret: &[u8]) -> Self {
        Self {
            http_upstream,
            ws_upstream,
            decoding_key: DecodingKey::from_secret(secret),
            http_breaker: CircuitBreaker::new("HTTP-API", 5, 5), // 5 ошибок -> 5 секунд кулдаун
            ws_breaker: CircuitBreaker::new("WS-GATEWAY", 5, 5),
        }
    }
}

pub struct GatewayContext {
    pub start_time: Instant,
    pub user_id: Option<String>,
    pub is_ws: bool,
}

#[async_trait]
impl ProxyHttp for GilvaveApiGateway {
    type CTX = GatewayContext;

    fn new_ctx(&self) -> Self::CTX {
        GatewayContext {
            start_time: Instant::now(),
            user_id: None,
            is_ws: false,
        }
    }

    /// Маршрутизация
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

        Ok(Box::new(HttpPeer::new(target_addr, false, String::new())))
    }

    /// Перехват и фильтрация запроса на входе
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {

        // Защита от спуфинга: удаляем клиентские заголовки
        {
            let req_mut = session.req_header_mut();
            let _ = req_mut.remove_header("x-user-id");
            let _ = req_mut.remove_header("x-session-id");
            let _ = req_mut.remove_header("x-user-data");
        }

        let path = session.req_header().uri.path();
        let is_ws = path.starts_with("/ws");


        // Circuit Breaker
        let breaker = if is_ws {
            &self.ws_breaker
        } else {
            &self.http_breaker
        };
        let (allowed, status) = breaker.can_execute();

        if !allowed {
            tracing::warn!(
                "[CIRCUIT BREAKER] Request to {} blocked by {:?} breaker!",
                path,
                status
            );

            let mut resp = ResponseHeader::build(503, None)?;
            let _ = resp.append_header("Content-Type", "application/json");
            let _ = resp.append_header("Retry-After", "5");
            let body = Bytes::from_static(
                b"{\"error\":\"Service temporarily unavailable. Circuit breaker is open. Please try again in 5 seconds.\"}",
            );
            let _ = resp.append_header("Content-Length", body.len().to_string());

            session.write_response_header(Box::new(resp), false).await?;
            session.write_response_body(Some(body), true).await?;
            return Ok(true);
        }


        // Обработка CORS Preflight (OPTIONS)
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
                "Content-Type, Authorization, X-Requested-With, Accept, Sec-WebSocket-Protocol",
            );
            let _ = resp.append_header("Access-Control-Max-Age", "86400");
            let _ = resp.append_header("Content-Length", "0");

            session.write_response_header(Box::new(resp), true).await?;
            return Ok(true);
        }


        // Проброс реального IP клиента
        if let Some(client_addr) = session.client_addr() {
            let ip_str = client_addr.to_string();
            let _ = session.req_header_mut().append_header("X-Real-IP", &ip_str);
            let _ = session
                .req_header_mut()
                .append_header("X-Forwarded-For", &ip_str);
        }


        // Edge Authentication (JWT Валидация + инжекция X-User-Data)
        let is_public = is_public_path(session.req_header().uri.path());

        if !is_public {
            let token_opt = extract_token(session);

            let auth_result = match token_opt {
                Some(token_str) => {
                    match decode::<Claims>(token_str, &self.decoding_key, &Validation::default()) {
                        Ok(data) => {
                            let user_id_str = data.claims.sub.0.to_string();
                            let session_id_str = data.claims.sid.0.to_string();
                            let user_data_json = serde_json::to_string(&GatewayUserData {
                                id: data.claims.sub,
                                session_id: data.claims.sid,
                                jti: data.claims.jti,
                                username: data.claims.username,
                                email: data.claims.email,
                            })
                            .unwrap_or_default();

                            Ok((user_id_str, session_id_str, user_data_json))
                        }
                        Err(e) => Err(format!("Invalid JWT: {e}")),
                    }
                }
                None => Err("Missing authentication token".to_string()),
            };

            match auth_result {
                Ok((user_id_str, session_id_str, user_data_json)) => {
                    let _ = session
                        .req_header_mut()
                        .append_header("X-User-Id", &user_id_str);
                    let _ = session
                        .req_header_mut()
                        .append_header("X-Session-Id", &session_id_str);
                    let _ = session
                        .req_header_mut()
                        .append_header("X-User-Data", &user_data_json);

                    ctx.user_id = Some(user_id_str);
                }
                Err(err_msg) => {
                    tracing::warn!(
                        "[GATEWAY AUTH] Rejected on {}: {}",
                        session.req_header().uri.path(),
                        err_msg
                    );
                    session
                        .respond_error_with_body(
                            401,
                            Bytes::from_static(
                                b"{\"error\":\"Unauthorized: Invalid or missing token\"}",
                            ),
                        )
                        .await?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Хук Pingora: вызывается при сбое подключения к апстриму!
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

    /// Ответ от апстрима получен успешно -> информируем Circuit Breaker и проставляем CORS
    async fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
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

    /// Добавление CORS-заголовков в исходящие ответы
    async fn response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        if let Some(origin) = session.req_header().headers.get("Origin") {
            let _ =
                upstream_response.append_header("Access-Control-Allow-Origin", origin.as_bytes());
            let _ = upstream_response.append_header("Access-Control-Allow-Credentials", "true");
            let _ = upstream_response.append_header("Vary", "Origin");
        }

        Ok(())
    }

    /// Структурированное логирование запроса с задержкой и ID пользователя
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
            "[GATEWAY] {} {} [{}] -> status: {} ({:?})",
            method,
            path,
            user_info,
            status,
            duration
        );
    }
}
