use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LfsObjectRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub oid: String,
    pub size: i64,
    pub s3_key: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upsert_lfs_object(
    pool: &PgPool,
    repo_id: Uuid,
    oid: &str,
    size: i64,
    s3_key: Option<&str>,
) -> anyhow::Result<LfsObjectRow> {
    let row = sqlx::query_as::<_, LfsObjectRow>(
        r#"INSERT INTO lfs_objects (repo_id, oid, size, s3_key)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (repo_id, oid)
           DO UPDATE SET size = $3, s3_key = COALESCE($4, lfs_objects.s3_key)
           RETURNING id, repo_id, oid, size, s3_key, created_at"#,
    )
    .bind(repo_id)
    .bind(oid)
    .bind(size)
    .bind(s3_key)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn find_lfs_object(pool: &PgPool, repo_id: Uuid, oid: &str) -> anyhow::Result<Option<LfsObjectRow>> {
    let row = sqlx::query_as::<_, LfsObjectRow>(
        "SELECT id, repo_id, oid, size, s3_key, created_at FROM lfs_objects WHERE repo_id = $1 AND oid = $2",
    )
    .bind(repo_id)
    .bind(oid)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_lfs_objects(pool: &PgPool, repo_id: Uuid) -> anyhow::Result<Vec<LfsObjectRow>> {
    let rows = sqlx::query_as::<_, LfsObjectRow>(
        "SELECT id, repo_id, oid, size, s3_key, created_at FROM lfs_objects WHERE repo_id = $1 ORDER BY created_at DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
