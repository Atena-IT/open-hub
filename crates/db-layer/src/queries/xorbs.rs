use sqlx::PgPool;

/// Insert or ignore a new xorb record. Returns true if newly inserted.
pub async fn upsert_xorb(
    pool: &PgPool,
    hash: &[u8; 32],
    s3_key: &str,
    size_bytes: i64,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        r#"
        INSERT INTO xorbs (hash, s3_key, size_bytes)
        VALUES ($1, $2, $3)
        ON CONFLICT (hash) DO NOTHING
        "#,
    )
    .bind(hash.as_slice())
    .bind(s3_key)
    .bind(size_bytes)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Check if a xorb exists.
pub async fn xorb_exists(pool: &PgPool, hash: &[u8; 32]) -> anyhow::Result<bool> {
    let count: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1) FROM xorbs WHERE hash = $1",
    )
    .bind(hash.as_slice())
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

/// Get xorb s3_key by hash.
pub async fn get_xorb_s3_key(pool: &PgPool, hash: &[u8; 32]) -> anyhow::Result<Option<String>> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT s3_key FROM xorbs WHERE hash = $1",
    )
    .bind(hash.as_slice())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Batch-insert chunk records (on conflict ignore).
pub async fn upsert_chunks(
    pool: &PgPool,
    chunks: &[ChunkRecord],
) -> anyhow::Result<()> {
    for c in chunks {
        sqlx::query(
            r#"
            INSERT INTO chunks (hash, xorb_hash, chunk_index_in_xorb, byte_range_start, unpacked_segment_bytes)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (hash) DO NOTHING
            "#,
        )
        .bind(c.hash.as_slice())
        .bind(c.xorb_hash.as_slice())
        .bind(c.chunk_index_in_xorb)
        .bind(c.byte_range_start)
        .bind(c.unpacked_segment_bytes)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Look up a chunk by its hash. Returns the chunk record if found.
pub async fn find_chunk(pool: &PgPool, hash: &[u8; 32]) -> anyhow::Result<Option<ChunkRecord>> {
    let row = sqlx::query_as::<_, ChunkRecord>(
        r#"
        SELECT hash, xorb_hash, chunk_index_in_xorb, byte_range_start, unpacked_segment_bytes
        FROM chunks WHERE hash = $1
        "#,
    )
    .bind(hash.as_slice())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Get all chunks belonging to a specific xorb.
pub async fn chunks_for_xorb(pool: &PgPool, xorb_hash: &[u8; 32]) -> anyhow::Result<Vec<ChunkRecord>> {
    let rows = sqlx::query_as::<_, ChunkRecord>(
        r#"
        SELECT hash, xorb_hash, chunk_index_in_xorb, byte_range_start, unpacked_segment_bytes
        FROM chunks WHERE xorb_hash = $1
        ORDER BY chunk_index_in_xorb ASC
        "#,
    )
    .bind(xorb_hash.as_slice())
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChunkRecord {
    pub hash:                   Vec<u8>,
    pub xorb_hash:              Vec<u8>,
    pub chunk_index_in_xorb:    i32,
    pub byte_range_start:       i32,
    pub unpacked_segment_bytes: i32,
}
