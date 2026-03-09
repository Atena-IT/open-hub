//! POST /v1/shards

use axum::{body::Body, extract::State, http::StatusCode, response::IntoResponse, Json};
use bytes::Bytes;
use http_body_util::BodyExt;
use serde_json::json;
use tracing::instrument;

use crate::state::AppState;
use common::{hash_to_api_string, AppError};
use db_layer::queries::{
    file_mappings::{upsert_file_mapping, ReconstructionTerm},
    xorbs::{upsert_chunks, ChunkRecord},
};
use s3_storage::shard_key;
use shard_parser::parse_shard;

#[instrument(skip(state, body))]
pub async fn upload_shard(
    State(state): State<AppState>,
    body: Body,
) -> Result<impl IntoResponse, AppError> {
    let data: Bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_bytes();

    // Parse the MDB shard binary
    let shard = parse_shard(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;

    tracing::info!(
        num_files = shard.files.len(),
        num_xorbs = shard.xorbs.len(),
        "shard received"
    );

    // Verify all referenced xorbs are already uploaded
    for xorb in &shard.xorbs {
        let plain_hex = hex::encode(xorb.xorb_hash);
        let s3_key = s3_storage::xorb_key(&plain_hex);
        let exists = state
            .s3
            .object_exists(&s3_key)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if !exists {
            return Err(AppError::BadRequest(format!(
                "referenced xorb {} not yet uploaded",
                plain_hex
            )));
        }
    }

    // Persist xorb + chunk metadata from CAS Info section
    for xorb in &shard.xorbs {
        let xorb_hex = hex::encode(xorb.xorb_hash);
        let s3_key = s3_storage::xorb_key(&xorb_hex);

        // Upsert xorb record (size_bytes from disk size)
        db_layer::queries::xorbs::upsert_xorb(
            &state.pool,
            &xorb.xorb_hash,
            &s3_key,
            xorb.num_bytes_on_disk as i64,
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // Upsert chunk records
        let chunk_records: Vec<ChunkRecord> = xorb
            .chunks
            .iter()
            .enumerate()
            .map(|(idx, c)| ChunkRecord {
                hash: c.chunk_hash.to_vec(),
                xorb_hash: xorb.xorb_hash.to_vec(),
                chunk_index_in_xorb: idx as i32,
                byte_range_start: c.chunk_byte_range_start as i32,
                unpacked_segment_bytes: c.unpacked_segment_bytes as i32,
            })
            .collect();

        upsert_chunks(&state.pool, &chunk_records)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Persist file reconstruction info from File Info section
    for file in &shard.files {
        let terms: Vec<ReconstructionTerm> = file
            .terms
            .iter()
            .map(|t| ReconstructionTerm {
                xorb_hash: hash_to_api_string(&t.xorb_hash),
                chunk_index_start: t.chunk_index_start as i32,
                chunk_index_end: t.chunk_index_end as i32,
                unpacked_length: t.unpacked_length as i64,
            })
            .collect();

        let sha256 = file
            .sha256
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("shard missing FileMetadataExt sha256".into()))?;

        upsert_file_mapping(&state.pool, &file.file_hash, sha256, &terms)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Store raw shard in S3 for archival
    // Use blake3 hash of the data as key (not strictly required by spec)
    let shard_hash = blake3_hash(&data);
    let shard_hex = hex::encode(shard_hash);
    let s3_key = shard_key(&shard_hex);
    state
        .s3
        .put_object(&s3_key, data)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((StatusCode::OK, Json(json!({ "result": 1 }))))
}

fn blake3_hash(data: &[u8]) -> [u8; 32] {
    // Simple non-keyed blake3 just for content-addressing the shard in S3
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}
