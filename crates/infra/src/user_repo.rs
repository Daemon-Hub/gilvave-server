use sqlx::PgPool;
use uuid::Uuid;

use gilvave_core::dto::user::RegisterResponse;
use gilvave_core::ids::UserId;
use gilvave_core::model::user::User;

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> anyhow::Result<RegisterResponse> {
    let id = Uuid::now_v7();

    let rec = sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash)

        VALUES ($1, $2, $3)
        RETURNING id, username
        "#,
        id,
        username,
        password_hash
    )
    .fetch_one(pool)
    .await?;

    Ok(RegisterResponse {
        id: UserId(rec.id),
        username: rec.username,
    })
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> anyhow::Result<User> {
    let rec = sqlx::query!(
        r#"
        SELECT * FROM users
        WHERE username = $1;
        "#,
        username
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: UserId(rec.id),
        username: rec.username,
        password_hash: rec.password_hash,
    })
}
