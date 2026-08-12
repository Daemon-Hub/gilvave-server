use std::sync::Arc;

use crate::db::Database;
use gilvave_core::{
    error::DatabaseError,
    ids::{RefreshTokenId, UserId},
};

#[derive(Clone)]
pub struct RefTokenService {
    pub db: Arc<Database>,
}

impl RefTokenService {
    pub async fn get(&self, token: &str) -> Option<UserId> {
        let row = self
            .db
            .query_opt(
                r#"
                SELECT user_id FROM refresh_token
                WHERE token = $1;
                "#,
                &[&token],
            )
            .await
            .ok()??;

        row.try_get::<_, Option<_>>(0).ok().flatten().map(UserId)
    }

    pub async fn create(&self, user_id: UserId, token: &str) -> Result<(), DatabaseError> {
        let now = time::OffsetDateTime::now_utc();
        let expires_at = now + time::Duration::days(30);

        self.db
            .execute(
                r#"
                INSERT INTO refresh_token (id, user_id, token, expires_at, created_at)
                VALUES ($1, $2, $3, $4, $5);
                "#,
                &[
                    &RefreshTokenId::default().0,
                    &user_id.0,
                    &token,
                    &expires_at,
                    &now,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn delete(&self, user_id: UserId) -> Result<(), DatabaseError> {
        self.db
            .execute(
                r#"
                DELETE FROM refresh_token
                WHERE user_id = $1;
                "#,
                &[&user_id.0],
            )
            .await?;
        Ok(())
    }

    pub async fn sync(&self, user_id: UserId, token: &str) -> Result<(), DatabaseError> {
        self.delete(user_id).await?;
        self.create(user_id, token).await?;
        Ok(())
    }
}
