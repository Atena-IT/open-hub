//! GET /v1/reconstructions/{file_id}
//!
//! Returns the QueryReconstructionResponse JSON so clients can download
//! the file directly from S3 via pre-signed URLs.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, time::Duration};
use tracing::instrument;

use crate::state::AppState;
use common::{api_string_to_hash, AppError};
use db_layer::queries::file_mappings::get_file_mapping;
use s3_storage::xorb_key;

#[derive(Debug, Deserialize)]
pub struct ReconPath {
    file_id: String,
}

#[derive(Deserialize)]
pub struct RangeQuery {
    // Optional byte range (from Range header — we support it via query param too)
}

#[instrument(skip(state), fields(file_id = %p.file_id))]
pub async fn get_reconstruction(
    State(state): State<AppState>,
    Path(p): Path<ReconPath>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    // Decode file_id from API hash encoding
    let file_hash = api_string_to_hash(&p.file_id)
        .ok_or_else(|| AppError::BadRequest("invalid file_id".into()))?;

    // Query DB for reconstruction terms
    let mapping = get_file_mapping(&state.pool, &file_hash)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("file {} not found", p.file_id)))?;

    let terms = mapping
        .parse_terms()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Parse optional Range header: "bytes=start-end"
    let (range_start, _range_end) = parse_range_header(&headers);

    // Generate pre-signed S3 URLs for each unique xorb referenced by the terms
    let presign_expiry = Duration::from_secs(3600); // 1 hour
    let mut fetch_info: HashMap<String, Vec<Value>> = HashMap::new();

    for term in &terms {
        if fetch_info.contains_key(&term.xorb_hash) {
            continue; // already presigned
        }

        // term.xorb_hash is already in API encoding (64-char hex)
        // We need to convert to raw bytes to build the S3 key
        let raw = api_string_to_hash(&term.xorb_hash)
            .ok_or_else(|| AppError::Internal("bad xorb_hash in DB".into()))?;
        let plain_hex = hex::encode(raw);
        let s3_key = xorb_key(&plain_hex);

        let url = state
            .s3
            .presign_get(&s3_key, presign_expiry)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // For simplicity, one fetch_info entry covers the full xorb
        // (clients will use the appropriate chunk range)
        let start_chunk = term.chunk_index_start;
        let end_chunk = term.chunk_index_end;

        // We need the xorb byte size to provide url_range.
        // Use 0-MAX as a conservative fallback; clients specify their own Range anyway.
        let _xorb_size: i64 =
            sqlx::query_scalar::<_, i64>("SELECT size_bytes FROM xorbs WHERE hash = $1")
                .bind(raw.as_slice())
                .fetch_optional(state.pool.as_ref())
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .unwrap_or(0);

        fetch_info
            .entry(term.xorb_hash.clone())
            .or_default()
            .push(json!({
                "range":     { "start": start_chunk, "end": end_chunk },
                "url":       url

            }));
    }

    // Build terms array
    let terms_json: Vec<Value> = terms
        .iter()
        .map(|t| {
            json!({
                "hash":            t.xorb_hash,
                "range":           { "start": t.chunk_index_start, "end": t.chunk_index_end },
                "unpacked_length": t.unpacked_length,
            })
        })
        .collect();

    let offset = if range_start > 0 {
        // TODO: compute exact offset_into_first_range from chunk boundaries
        // For now return 0 (full-file download only fully supported)
        0
    } else {
        0
    };

    tracing::info!(num_terms = terms_json.len(), "reconstruction served");

    Ok(Json(json!({
        "offset_into_first_range": offset,
        "terms":      terms_json,
        "fetch_info": fetch_info,
    })))
}

/// Parse `Range: bytes=start-end` header.
/// Returns (start, end_opt) where end_opt = None means "to EOF".
fn parse_range_header(headers: &axum::http::HeaderMap) -> (u64, Option<u64>) {
    let value = headers
        .get(axum::http::header::RANGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let stripped = value.strip_prefix("bytes=").unwrap_or("");
    let mut parts = stripped.splitn(2, '-');
    let start = parts
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let end = parts.next().and_then(|s| s.parse::<u64>().ok());
    (start, end)
}
