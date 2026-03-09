use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RepoFileRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub path: String,
    pub size: i64,
    pub sha256: Option<String>,
    pub s3_key: Option<String>,
    pub is_lfs: bool,
    pub lfs_oid: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upsert_file(
    pool: &PgPool,
    repo_id: Uuid,
    path: &str,
    size: i64,
    sha256: Option<&str>,
    s3_key: Option<&str>,
    is_lfs: bool,
    lfs_oid: Option<&str>,
) -> anyhow::Result<RepoFileRow> {
    let row = sqlx::query_as::<_, RepoFileRow>(
        r#"INSERT INTO repo_files (repo_id, path, size, sha256, s3_key, is_lfs, lfs_oid)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (repo_id, path)
           DO UPDATE SET size = $3, sha256 = $4, s3_key = $5, is_lfs = $6, lfs_oid = $7, updated_at = NOW()
           RETURNING id, repo_id, path, size, sha256, s3_key, is_lfs, lfs_oid, created_at, updated_at"#,
    )
    .bind(repo_id)
    .bind(path)
    .bind(size)
    .bind(sha256)
    .bind(s3_key)
    .bind(is_lfs)
    .bind(lfs_oid)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_file(pool: &PgPool, repo_id: Uuid, path: &str) -> anyhow::Result<Option<RepoFileRow>> {
    let row = sqlx::query_as::<_, RepoFileRow>(
        "SELECT id, repo_id, path, size, sha256, s3_key, is_lfs, lfs_oid, created_at, updated_at FROM repo_files WHERE repo_id = $1 AND path = $2",
    )
    .bind(repo_id)
    .bind(path)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_files(pool: &PgPool, repo_id: Uuid, prefix: Option<&str>) -> anyhow::Result<Vec<RepoFileRow>> {
    let rows = if let Some(prefix) = prefix {
        let pattern = if prefix.is_empty() {
            "%".to_string()
        } else {
            format!("{}/%", prefix)
        };
        // List direct children of the prefix
        sqlx::query_as::<_, RepoFileRow>(
            "SELECT id, repo_id, path, size, sha256, s3_key, is_lfs, lfs_oid, created_at, updated_at FROM repo_files WHERE repo_id = $1 AND path LIKE $2 ORDER BY path",
        )
        .bind(repo_id)
        .bind(&pattern)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, RepoFileRow>(
            "SELECT id, repo_id, path, size, sha256, s3_key, is_lfs, lfs_oid, created_at, updated_at FROM repo_files WHERE repo_id = $1 ORDER BY path",
        )
        .bind(repo_id)
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

pub async fn delete_file(pool: &PgPool, repo_id: Uuid, path: &str) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM repo_files WHERE repo_id = $1 AND path = $2")
        .bind(repo_id)
        .bind(path)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
