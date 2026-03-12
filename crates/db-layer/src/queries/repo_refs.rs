use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RepoRefRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub ref_type: String,
    pub target_sha: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_refs(pool: &PgPool, repo_id: Uuid) -> anyhow::Result<Vec<RepoRefRow>> {
    let rows = sqlx::query_as::<_, RepoRefRow>(
        "SELECT id, repo_id, name, ref_type, target_sha, created_at, updated_at FROM repo_refs WHERE repo_id = $1 ORDER BY ref_type, name",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_ref_by_name(
    pool: &PgPool,
    repo_id: Uuid,
    name: &str,
) -> anyhow::Result<Option<RepoRefRow>> {
    let row = sqlx::query_as::<_, RepoRefRow>(
        "SELECT id, repo_id, name, ref_type, target_sha, created_at, updated_at FROM repo_refs WHERE repo_id = $1 AND name = $2",
    )
    .bind(repo_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_ref(
    pool: &PgPool,
    repo_id: Uuid,
    name: &str,
    ref_type: &str,
    target_sha: &str,
) -> anyhow::Result<RepoRefRow> {
    let row = sqlx::query_as::<_, RepoRefRow>(
        r#"INSERT INTO repo_refs (repo_id, name, ref_type, target_sha)
           VALUES ($1, $2, $3, $4)
           RETURNING id, repo_id, name, ref_type, target_sha, created_at, updated_at"#,
    )
    .bind(repo_id)
    .bind(name)
    .bind(ref_type)
    .bind(target_sha)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_ref(
    pool: &PgPool,
    repo_id: Uuid,
    name: &str,
    ref_type: &str,
) -> anyhow::Result<bool> {
    let result =
        sqlx::query("DELETE FROM repo_refs WHERE repo_id = $1 AND name = $2 AND ref_type = $3")
            .bind(repo_id)
            .bind(name)
            .bind(ref_type)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}
