use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
};
use std::net::IpAddr;

use gilvave_core::{
    dto::user::{
        AuthTokensResponse, Avatar, AvatarUrl, BlacklistInfo, LoginInfo, RefreshTokenRequest,
        RegisterRequest, SessionCreateInfo, UpdateAvatarInfo, UserView,
    },
    error::CoreError,
    validation::validate_username,
};
use gilvave_infra::{
    jwt::{create_jwt, generate_refresh_token, hash_refresh_token},
    security::auth::{AuthUser, extract_cookie},
};
use gilvave_settings::settings;

use crate::state::AppState;

fn build_tokens_response(access_token: String, refresh_token: String, is_web: bool) -> Response {
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

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(), CoreError> {
    if let Err(msg) = validate_username(&body.username) {
        return Err(CoreError::BadRequest(msg.to_string()));
    }

    if state
        .user_service
        .find_by_email(&body.email)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "The user with this email address exists.".to_string(),
        ));
    }

    if state
        .user_service
        .find_by_username(&body.username)
        .await?
        .is_some()
    {
        return Err(CoreError::Conflict(
            "The user with this username exists.".to_string(),
        ));
    }

    state
        .user_service
        .create(&body.username, &body.email, &body.password)
        .await?;

    Ok(())
}

pub async fn login(
    State(state): State<AppState>,
    Extension(ip_address): Extension<IpAddr>,
    Json(body): Json<LoginInfo>,
) -> Result<Response, CoreError> {
    let user = match state.user_service.find_by_email(&body.email).await? {
        Some(u) => u,
        None => {
            return Err(CoreError::Unauthorized(
                "The user with this email address has not been found!".to_string(),
            ));
        }
    };

    if !state
        .user_service
        .verify_password(&user.password_hash, &body.password)
    {
        return Err(CoreError::Unauthorized("Invalid password".to_string()));
    }

    let is_web = body
        .device_info
        .get("client")
        .and_then(|v| v.as_str())
        == Some("web");

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    let session_id = state
        .session_service
        .create(SessionCreateInfo {
            user_id: user.id,
            refresh_token_hash: refresh_token_hash.as_bytes(),
            device_info: body.device_info,
            ip_address,
        })
        .await?;

    let access_token = create_jwt(session_id, user.id).unwrap();

    Ok(build_tokens_response(access_token, refresh_token, is_web))
}

pub async fn logout(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
) -> Result<Response, CoreError> {
    state.session_service.delete(user.id).await?;
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(String::from)
        .or_else(|| extract_cookie(&headers, "access_token"))
        .ok_or_else(|| CoreError::Unauthorized("Missing authorization token".to_string()))?;

    state
        .user_service
        .blacklist_token(BlacklistInfo {
            token,
            user_id: user.id,
        })
        .await?;

    let mut resp_headers = HeaderMap::new();
    if let Ok(val) =
        HeaderValue::from_str("access_token=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax")
    {
        resp_headers.append(header::SET_COOKIE, val);
    }
    if let Ok(val) =
        HeaderValue::from_str("refresh_token=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax")
    {
        resp_headers.append(header::SET_COOKIE, val);
    }

    Ok((StatusCode::OK, resp_headers).into_response())
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Extension(ip_address): Extension<IpAddr>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<Response, CoreError> {
    let (raw_refresh_token, is_web) =
        if let Some(cookie_token) = extract_cookie(&headers, "refresh_token") {
            (cookie_token, true)
        } else if let Some(Json(req)) = body
            && !req.refresh_token.is_empty()
        {
            (req.refresh_token, false)
        } else {
            return Err(CoreError::Unauthorized(
                "Missing refresh token".to_string(),
            ));
        };

    let refresh_token_hash = hash_refresh_token(&raw_refresh_token);
    let (session_id, user_id) = state
        .session_service
        .get_unexpired(&refresh_token_hash)
        .await
        .ok_or(CoreError::Unauthorized(
            "Invalid or expired refresh token".to_string(),
        ))?;

    let access_token = create_jwt(session_id, user_id)
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    state
        .session_service
        .update(user_id, session_id, refresh_token_hash, ip_address)
        .await?;

    Ok(build_tokens_response(access_token, refresh_token, is_web))
}

pub async fn get_profile(AuthUser(user): AuthUser) -> Json<UserView> {
    Json(UserView {
        id: user.id,
        username: user.username,
        email: user.email,
        is_active: user.is_active,
        avatar: user.avatar,
    })
}

pub async fn update_avatar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    mut multipart: Multipart,
) -> Result<Json<AvatarUrl>, CoreError> {
    let avatar = if let Some(field) = multipart.next_field().await.unwrap_or(None)
        && (field.name().unwrap_or("") == "avatar")
    {
        Avatar {
            filename: field
                .file_name()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "avatar".to_string()),
            bytes: match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    return Err(CoreError::BadRequest(format!("Failed to read file: {}", e)));
                }
            },
        }
    } else {
        return Err(CoreError::BadRequest("Missing field 'avatar'".to_string()));
    };

    let (processed_bytes, mime_type) = state.user_service.process_avatar(&avatar)?;
    let url = state
        .s3
        .send_static(
            &format!("avatar/{}/{}.webp", user.id, uuid::Uuid::new_v4()),
            processed_bytes.into(),
            Some(&mime_type),
        )
        .await
        .map_err(|e| CoreError::InternalServerError(e.to_string()))?;

    state
        .user_service
        .update_avatar(UpdateAvatarInfo {
            user_id: user.id,
            url: url.to_owned(),
        })
        .await?;

    Ok(Json(AvatarUrl { url }))
}
