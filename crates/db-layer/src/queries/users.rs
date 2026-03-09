use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub full_name: String,
    pub is_org: bool,
    pub avatar_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: Option<&str>,
    password_hash: &str,
) -> anyhow::Result<UserRow> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"INSERT INTO users (username, email, password_hash)
           VALUES ($1, $2, $3)
           RETURNING id, username, email, password_hash, full_name, is_org, avatar_url, created_at, updated_at"#,
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_user_by_username(pool: &PgPool, username: &str) -> anyhow::Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, full_name, is_org, avatar_url, created_at, updated_at FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, full_name, is_org, avatar_url, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_org(pool: &PgPool, name: &str, creator_id: Uuid) -> anyhow::Result<UserRow> {
    // Create org user entry
    let org = sqlx::query_as::<_, UserRow>(
        r#"INSERT INTO users (username, password_hash, is_org)
           VALUES ($1, '', TRUE)
           RETURNING id, username, email, password_hash, full_name, is_org, avatar_url, created_at, updated_at"#,
    )
    .bind(name)
    .fetch_one(pool)
    .await?;

    // Add creator as owner
    sqlx::query(
        "INSERT INTO org_members (org_id, user_id, role) VALUES ($1, $2, 'owner')"
    )
    .bind(org.id)
    .bind(creator_id)
    .execute(pool)
    .await?;

    Ok(org)
}
