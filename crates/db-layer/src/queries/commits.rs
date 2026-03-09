use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CommitRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub sha: String,
    pub message: String,
    pub author_id: Option<Uuid>,
    pub parent_sha: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_commit(
    pool: &PgPool,
    repo_id: Uuid,
    sha: &str,
    message: &str,
    author_id: Option<Uuid>,
    parent_sha: Option<&str>,
) -> anyhow::Result<CommitRow> {
    let row = sqlx::query_as::<_, CommitRow>(
        r#"INSERT INTO commits (repo_id, sha, message, author_id, parent_sha)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, repo_id, sha, message, author_id, parent_sha, created_at"#,
    )
    .bind(repo_id)
    .bind(sha)
    .bind(message)
    .bind(author_id)
    .bind(parent_sha)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_latest_commit(pool: &PgPool, repo_id: Uuid) -> anyhow::Result<Option<CommitRow>> {
    let row = sqlx::query_as::<_, CommitRow>(
        "SELECT id, repo_id, sha, message, author_id, parent_sha, created_at FROM commits WHERE repo_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(repo_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_commit_by_sha(
    pool: &PgPool,
    repo_id: Uuid,
    sha: &str,
) -> anyhow::Result<Option<CommitRow>> {
    let row = sqlx::query_as::<_, CommitRow>(
        "SELECT id, repo_id, sha, message, author_id, parent_sha, created_at FROM commits WHERE repo_id = $1 AND sha = $2",
    )
    .bind(repo_id)
    .bind(sha)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
