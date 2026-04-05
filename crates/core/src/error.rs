use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal,
}
