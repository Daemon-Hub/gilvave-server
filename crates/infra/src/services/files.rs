use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{
    dto::files::{CreateFileInfo, File},
    error::DatabaseError,
    ids::{FileId, MessageId, UserId},
};

#[derive(Clone)]
pub struct FilesService {
    pub db: Arc<Database>,
}

impl FilesService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Создание записи файла
    pub async fn create(
        &self,
        info: CreateFileInfo,
    ) -> Result<File, DatabaseError> {
        let message_uuid = info.message_id.map(|id| id.0);
        let row = self
            .db
            .query_one(
                r#"
                INSERT INTO files (id, name, url, message_id, size_kb, mime_type, uploader_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, name, url, message_id, size_kb, mime_type, uploader_id, created_at;
                "#,
                &[
                    &info.file_id.0,
                    &info.name,
                    &info.url,
                    &message_uuid,
                    &info.size_kb,
                    &info.mime_type,
                    &info.uploader_id.0,
                ],
            )
            .await?;

        File::from_row(&row)
    }

    /// Получение файла по ID
    pub async fn get_by_id(&self, file_id: FileId) -> Result<Option<File>, DatabaseError> {
        let row_opt = self
            .db
            .query_opt(
                r#"
                SELECT id, name, url, message_id, size_kb, mime_type, uploader_id, created_at
                FROM files
                WHERE id = $1;
                "#,
                &[&file_id.0],
            )
            .await?;

        row_opt.map(|r| File::from_row(&r)).transpose()
    }

    /// Получение файлов сообщения
    pub async fn get_by_message_id(&self, message_id: MessageId) -> Result<Vec<File>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                SELECT id, name, url, message_id, size_kb, mime_type, uploader_id, created_at
                FROM files
                WHERE message_id = $1;
                "#,
                &[&message_id.0],
            )
            .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in &rows {
            list.push(File::from_row(row)?);
        }
        Ok(list)
    }

    /// Удаление файла по ID
    pub async fn delete(&self, file_id: FileId, user_id: UserId) -> Result<Option<File>, DatabaseError> {
        let row_opt = self
            .db
            .query_opt(
                r#"
                DELETE FROM files
                WHERE id = $1 AND (uploader_id = $2 OR uploader_id IS NULL)
                RETURNING id, name, url, message_id, size_kb, mime_type, uploader_id, created_at;
                "#,
                &[&file_id.0, &user_id.0],
            )
            .await?;

        row_opt.map(|r| File::from_row(&r)).transpose()
    }

    /// Удаление файлов сообщения
    pub async fn delete_by_message_id(&self, message_id: MessageId) -> Result<Vec<File>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                DELETE FROM files
                WHERE message_id = $1
                RETURNING id, name, url, message_id, size_kb, mime_type, uploader_id, created_at;
                "#,
                &[&message_id.0],
            )
            .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in &rows {
            list.push(File::from_row(row)?);
        }
        Ok(list)
    }

    /// Привязка файла к сообщению
    pub async fn link_to_message(
        &self,
        file_id: FileId,
        message_id: MessageId,
        user_id: UserId,
    ) -> Result<bool, DatabaseError> {
        let count = self
            .db
            .execute(
                r#"
                UPDATE files
                SET message_id = $2
                WHERE id = $1 AND (uploader_id = $3 OR uploader_id IS NULL);
                "#,
                &[&file_id.0, &message_id.0, &user_id.0],
            )
            .await?;

        Ok(count > 0)
    }
}
