use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};
use gilvave_core::{
    dto::{
        files::{CreateFileInfo, File, FileType, FileUploadResponse},
        users::auth::Claims,
    },
    error::CoreError,
    files::{detect_file_type, get_file_extension, parse_s3_url},
    ids::FileId,
};

use crate::state::AppState;

/// Загрузка файла
pub async fn upload_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, CoreError> {
    let mut file_data = None;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name_attr = field.name().unwrap_or("").to_string();
        let raw_file_name = field
            .file_name()
            .map(|f| f.to_string())
            .unwrap_or_else(|| format!("file_{}", time::OffsetDateTime::now_utc().unix_timestamp()));

        if name_attr == "file" || file_data.is_none() {
            let mut bytes = Vec::new();
            let mut total_size: u64 = 0;

            while let Ok(Some(chunk)) = field.chunk().await {
                total_size += chunk.len() as u64;
                if total_size > gilvave_core::files::MAX_FILE_SIZE_BYTES {
                    return Err(CoreError::BadRequest(format!(
                        "Размер файла превышает максимальный лимит {} МБ",
                        gilvave_core::files::MAX_FILE_SIZE_MB
                    )));
                }
                bytes.extend_from_slice(&chunk);
            }

            if bytes.is_empty() {
                return Err(CoreError::BadRequest("Uploaded file is empty".to_string()));
            }

            let safe_file_name = std::path::Path::new(&raw_file_name)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.replace('\0', "").trim().to_string())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| format!("file_{}", time::OffsetDateTime::now_utc().unix_timestamp()));

            let safe_file_name = if safe_file_name.chars().count() > 255 {
                safe_file_name.chars().take(255).collect::<String>()
            } else {
                safe_file_name
            };

            file_data = Some((safe_file_name, bytes));
            break;
        }
    }

    let (file_name, bytes) = file_data
        .ok_or_else(|| CoreError::BadRequest("No file found in the request".to_string()))?;

    let (file_type, mime_type) = detect_file_type(&file_name, &bytes);
    let ext = get_file_extension(&file_name, mime_type);
    let file_id = FileId::default();
    let key = format!("{}/{}.{}", claims.sub, file_id, ext);

    let url = match file_type {
        FileType::Image => {
            state
                .s3
                .send_image(&key, bytes.clone().into(), Some(mime_type))
                .await
        }
        FileType::Video => {
            state
                .s3
                .send_video(&key, bytes.clone().into(), Some(mime_type))
                .await
        }
        FileType::File => {
            state
                .s3
                .send_file(&key, bytes.clone().into(), Some(mime_type))
                .await
        }
    }
    .map_err(|e| CoreError::InternalServerError(format!("S3 upload error: {e}")))?;

    let size_kb = (bytes.len() as f64 / 1024.0).ceil() as i32;

    state
        .files
        .create(CreateFileInfo {
            file_id,
            uploader_id: claims.sub,
            name: file_name.clone(),
            size_kb,
            mime_type: mime_type.to_string(),
            url: url.clone(),
            message_id: None,
        })
        .await?;

    let public_url = format!("/files/{}", file_id);

    Ok(Json(FileUploadResponse {
        id: file_id,
        name: file_name,
        url: public_url,
        size_kb,
        mime_type: mime_type.to_string(),
        file_type,
        is_previewable: file_type.is_previewable(),
    }))
}

/// Скачивание файла
pub async fn download_file(
    State(state): State<AppState>,
    Path(file_id): Path<FileId>,
) -> Result<impl IntoResponse, CoreError> {
    let file = state
        .files
        .get_by_id(file_id)
        .await?
        .ok_or_else(|| CoreError::NotFound("File not found".to_string()))?;

    let (bucket, key) = parse_s3_url(&file.url)
        .ok_or_else(|| CoreError::InternalServerError("Invalid file URL in database".to_string()))?;

    let data = state
        .s3
        .get_by_bucket(bucket, key)
        .await
        .map_err(|e| CoreError::InternalServerError(format!("Failed to retrieve file from S3: {e}")))?;

    let mut headers = HeaderMap::new();
    let ct = HeaderValue::from_str(&file.mime_type)
        .unwrap_or(HeaderValue::from_static("application/octet-stream"));
    headers.insert(header::CONTENT_TYPE, ct);

    let is_preview = file.mime_type.starts_with("image/") || file.mime_type.starts_with("video/");
    let disposition_type = if is_preview { "inline" } else { "attachment" };

    let safe_ascii_name: String = file
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // RFC 5987 / RFC 6266 percent-encoded UTF-8 filename for full Unicode (Cyrillic, emojis, etc.)
    let mut encoded_name = String::with_capacity(file.name.len() * 3);
    for b in file.name.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(*b, b'.' | b'-' | b'_' | b'~') {
            encoded_name.push(*b as char);
        } else {
            use std::fmt::Write;
            let _ = write!(encoded_name, "%{:02X}", b);
        }
    }

    let disposition = format!(
        "{disposition_type}; filename=\"{safe_ascii_name}\"; filename*=UTF-8''{encoded_name}"
    );
    let cd = HeaderValue::from_str(&disposition)
        .unwrap_or(HeaderValue::from_static("inline"));
    headers.insert(header::CONTENT_DISPOSITION, cd);
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

    Ok((headers, data.to_vec()))
}

/// Получение метаданных файла
pub async fn get_file_info(
    State(state): State<AppState>,
    Path(file_id): Path<FileId>,
) -> Result<Json<File>, CoreError> {
    let file = state
        .files
        .get_by_id(file_id)
        .await?
        .ok_or_else(|| CoreError::NotFound("File not found".to_string()))?;

    Ok(Json(file))
}

/// Удаление файла
pub async fn delete_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<FileId>,
) -> Result<Json<()>, CoreError> {
    let file = state
        .files
        .get_by_id(file_id)
        .await?
        .ok_or_else(|| CoreError::NotFound("File not found".to_string()))?;

    if let Some(uploader_id) = file.uploader_id
        && uploader_id != claims.sub
    {
        return Err(CoreError::Forbidden(
            "You are not allowed to delete this file".to_string(),
        ));
    }

    if let Some((bucket, key)) = parse_s3_url(&file.url)
        && let Err(e) = state.s3.delete_object(bucket, key).await
    {
        tracing::warn!("[S3] Failed to delete object {bucket}/{key}: {e}");
    }

    state.files.delete(file_id, claims.sub).await?;

    Ok(Json(()))
}
