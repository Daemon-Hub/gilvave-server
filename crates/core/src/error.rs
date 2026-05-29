use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum CoreError {
    /// 401 Несанкционированный доступ
    Unauthorized(String),
    /// 400 неверный запрос
    BadRequest(String),
    /// 403 Запрещено
    Forbidden(String),
    /// 409 Конфликт
    Conflict(String),
    /// 413 Полезная нагрузка слишком велика
    PayloadTooLarge(String),
    /// 415 Неподдерживаемый тип носителя
    UnsupportedMediaType(String),
    /// 422 Необрабатываемая сущность
    UnprocessableEntity(String),

    /// 500 Внутренняя ошибка сервера
    InternalServerError(String),
}

impl IntoResponse for CoreError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            CoreError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            CoreError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            CoreError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            CoreError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            CoreError::UnsupportedMediaType(msg) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg),
            CoreError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            CoreError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg),
            CoreError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

impl From<anyhow::Error> for CoreError {
    fn from(err: anyhow::Error) -> Self {
        CoreError::InternalServerError(err.to_string())
    }
}
