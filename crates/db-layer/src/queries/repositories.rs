use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RepoRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub full_name: String,
    pub repo_type: String,
    pub private: bool,
    pub head_sha: Option<String>,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_repo(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
    full_name: &str,
    repo_type: &str,
    private: bool,
) -> anyhow::Result<RepoRow> {
    let row = sqlx::query_as::<_, RepoRow>(
        r#"INSERT INTO repositories (owner_id, name, full_name, repo_type, private)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, owner_id, name, full_name, repo_type, private, head_sha, description, created_at, updated_at"#,
    )
    .bind(owner_id)
    .bind(name)
    .bind(full_name)
    .bind(repo_type)
    .bind(private)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_repo_by_full_name(
    pool: &PgPool,
    full_name: &str,
) -> anyhow::Result<Option<RepoRow>> {
    let row = sqlx::query_as::<_, RepoRow>(
        "SELECT id, owner_id, name, full_name, repo_type, private, head_sha, description, created_at, updated_at FROM repositories WHERE full_name = $1",
    )
    .bind(full_name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_repo_by_id(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<RepoRow>> {
    let row = sqlx::query_as::<_, RepoRow>(
        "SELECT id, owner_id, name, full_name, repo_type, private, head_sha, description, created_at, updated_at FROM repositories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_repos_for_owner(pool: &PgPool, owner_id: Uuid) -> anyhow::Result<Vec<RepoRow>> {
    let rows = sqlx::query_as::<_, RepoRow>(
        "SELECT id, owner_id, name, full_name, repo_type, private, head_sha, description, created_at, updated_at FROM repositories WHERE owner_id = $1 ORDER BY updated_at DESC",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update_head_sha(pool: &PgPool, repo_id: Uuid, sha: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE repositories SET head_sha = $1, updated_at = NOW() WHERE id = $2")
        .bind(sha)
        .bind(repo_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_repo(pool: &PgPool, repo_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM repositories WHERE id = $1")
        .bind(repo_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_repos_by_type(pool: &PgPool, repo_type: &str) -> anyhow::Result<Vec<RepoRow>> {
    let rows = sqlx::query_as::<_, RepoRow>(
        "SELECT id, owner_id, name, full_name, repo_type, private, head_sha, description, created_at, updated_at FROM repositories WHERE repo_type = $1 ORDER BY updated_at DESC",
    )
    .bind(repo_type)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
