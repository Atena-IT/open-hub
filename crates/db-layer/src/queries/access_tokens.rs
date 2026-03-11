use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccessTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub token_hash: Vec<u8>,
    pub scopes: Vec<String>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_token(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    token_hash: &[u8],
    scopes: &[String],
) -> anyhow::Result<AccessTokenRow> {
    let row = sqlx::query_as::<_, AccessTokenRow>(
        r#"INSERT INTO access_tokens (user_id, name, token_hash, scopes)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, name, token_hash, scopes, last_used, created_at"#,
    )
    .bind(user_id)
    .bind(name)
    .bind(token_hash)
    .bind(scopes)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_token_by_hash(
    pool: &PgPool,
    token_hash: &[u8],
) -> anyhow::Result<Option<AccessTokenRow>> {
    let row = sqlx::query_as::<_, AccessTokenRow>(
        "SELECT id, user_id, name, token_hash, scopes, last_used, created_at FROM access_tokens WHERE token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_tokens_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<AccessTokenRow>> {
    let rows = sqlx::query_as::<_, AccessTokenRow>(
        "SELECT id, user_id, name, token_hash, scopes, last_used, created_at FROM access_tokens WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn delete_token(pool: &PgPool, token_id: Uuid, user_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM access_tokens WHERE id = $1 AND user_id = $2")
        .bind(token_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn touch_token(pool: &PgPool, token_id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE access_tokens SET last_used = NOW() WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}
