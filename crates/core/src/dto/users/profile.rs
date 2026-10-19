use bytes::Bytes;
use serde::Serialize;

pub struct Avatar {
    pub filename: String,
    pub bytes: Bytes,
}

#[derive(Serialize)]
pub struct AvatarUrl {
    pub url: String,
}
