//! GET /v1/chunks/default-merkledb/{chunk_hash}
//!
//! Global deduplication endpoint.
//! Returns an MDB shard binary containing the xorb(s) that contain the chunk,
//! or 404 if the chunk is unknown.

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};
use tracing::instrument;

use crate::state::AppState;
use common::{api_string_to_hash, AppError};
use db_layer::queries::xorbs::chunks_for_xorb;
use shard_parser::{build_dedup_response_shard, ChunkInXorb, ParsedXorb};

#[derive(Debug, serde::Deserialize)]
pub struct ChunkPath {
    prefix: String,
    hash: String,
}

#[instrument(skip(state), fields(chunk_hash = %p.hash))]
pub async fn dedup_query(
    State(state): State<AppState>,
    Path(p): Path<ChunkPath>,
) -> Result<Response, AppError> {
    if p.prefix != "default-merkledb" && p.prefix != "default" {
        return Err(AppError::BadRequest(format!(
            "unknown prefix '{}'",
            p.prefix
        )));
    }

    let chunk_hash = api_string_to_hash(&p.hash)
        .ok_or_else(|| AppError::BadRequest("invalid chunk hash".into()))?;

    // Look up the chunk in the DB
    let record = db_layer::queries::xorbs::find_chunk(&state.pool, &chunk_hash)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let record = match record {
        Some(r) => r,
        None => return Err(AppError::NotFound("chunk not found".into())),
    };

    // Get the xorb hash from the chunk record
    let xorb_hash_bytes: [u8; 32] = record
        .xorb_hash
        .try_into()
        .map_err(|_| AppError::Internal("bad xorb_hash in DB".into()))?;

    // Get all chunks in that xorb
    let db_chunks = chunks_for_xorb(&state.pool, &xorb_hash_bytes)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let chunks: Vec<ChunkInXorb> = db_chunks
        .into_iter()
        .map(|c| {
            let mut ch = [0u8; 32];
            let src = c.hash.as_slice();
            ch[..src.len().min(32)].copy_from_slice(&src[..src.len().min(32)]);
            ChunkInXorb {
                chunk_hash: ch,
                chunk_byte_range_start: c.byte_range_start as u32,
                unpacked_segment_bytes: c.unpacked_segment_bytes as u32,
            }
        })
        .collect();

    // We need num_bytes_in_cas and num_bytes_on_disk for the xorb.
    // Fetch from xorbs table (size_bytes is on-disk size).
    let on_disk: u32 = sqlx::query_scalar::<_, i64>("SELECT size_bytes FROM xorbs WHERE hash = $1")
        .bind(xorb_hash_bytes.as_slice())
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .unwrap_or(0) as u32;
    let in_cas: u32 = chunks.iter().map(|c| c.unpacked_segment_bytes).sum();

    let xorb = ParsedXorb {
        xorb_hash: xorb_hash_bytes,
        num_bytes_in_cas: in_cas,
        num_bytes_on_disk: on_disk,
        chunks,
    };

    let shard_bytes = build_dedup_response_shard(&[&xorb]);

    tracing::info!(found = true, "global dedup hit");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(shard_bytes))
        .unwrap())
}
