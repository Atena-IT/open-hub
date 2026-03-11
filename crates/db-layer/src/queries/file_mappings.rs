use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// A single term in a file reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionTerm {
    /// Xorb hash as 64-char hex (API encoding).
    pub xorb_hash: String,
    pub chunk_index_start: i32,
    pub chunk_index_end: i32,
    pub unpacked_length: i64,
}

pub async fn upsert_file_mapping(
    pool: &PgPool,
    file_hash: &[u8; 32],
    sha256: &[u8; 32],
    terms: &[ReconstructionTerm],
) -> anyhow::Result<()> {
    let terms_json = serde_json::to_value(terms)?;
    sqlx::query(
        r#"
        INSERT INTO file_mappings (file_hash, sha256, reconstruction_terms)
        VALUES ($1, $2, $3)
        ON CONFLICT (file_hash) DO NOTHING
        "#,
    )
    .bind(file_hash.as_slice())
    .bind(sha256.as_slice())
    .bind(terms_json)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_file_mapping(
    pool: &PgPool,
    file_hash: &[u8; 32],
) -> anyhow::Result<Option<FileMappingRow>> {
    let row = sqlx::query_as::<_, FileMappingRow>(
        r#"SELECT file_hash, sha256, reconstruction_terms FROM file_mappings WHERE file_hash = $1 OR sha256 = $1"#,
    )
    .bind(file_hash.as_slice())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

#[derive(Debug, sqlx::FromRow)]
pub struct FileMappingRow {
    pub file_hash: Vec<u8>,
    pub sha256: Vec<u8>,
    pub reconstruction_terms: Option<Value>,
}

impl FileMappingRow {
    pub fn parse_terms(&self) -> anyhow::Result<Vec<ReconstructionTerm>> {
        match &self.reconstruction_terms {
            Some(v) => Ok(serde_json::from_value(v.clone())?),
            None => Ok(vec![]),
        }
    }
}
