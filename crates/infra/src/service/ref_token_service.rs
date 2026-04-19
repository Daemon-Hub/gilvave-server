use sqlx::PgPool;

use gilvave_core::ids::{RefreshTokenId, UserId};

#[derive(Clone)]
pub struct RefTokenService {
    pub db: PgPool,
}

impl RefTokenService {
    pub async fn create_token(&self, user_id: UserId, token: &str) -> anyhow::Result<()> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query!(
            r#"
            INSERT INTO refresh_token (id, user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5);
            "#,
            RefreshTokenId::default().0,
            user_id.0,
            token,
            now + time::Duration::days(30),
            now
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_token(&self, user_id: UserId) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM refresh_token
            WHERE user_id = $1;
            "#,
            user_id.0
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn sync_refresh_token(&self, user_id: UserId, token: &str) -> anyhow::Result<()> {
        // sqlx::query!(
        //     r#"
        //     UPDATE refresh_token
        //     SET token=$1, expires_at=$2, created_at=$3
        //     WHERE user_id=$4;
        //     "#,
        //     token,
        //     time::OffsetDateTime::now_utc() + time::Duration::days(30),
        //     time::OffsetDateTime::now_utc(),
        //     user_id.0
        // )
        // .execute(&self.db)
        // .await?;
        self.delete_token(user_id).await?;
        self.create_token(user_id, token).await?;
        Ok(())
    }
}
