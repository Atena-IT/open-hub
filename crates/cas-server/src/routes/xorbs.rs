//! POST /v1/xorbs/default/{xorb_hash}

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use http_body_util::BodyExt;
use serde_json::json;
use tracing::instrument;

use common::{api_string_to_hash, AppError};
use db_layer::queries::xorbs::upsert_xorb;
use s3_storage::xorb_key;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct XorbPath {
    prefix: String,
    hash:   String,
}

#[instrument(skip(state, body), fields(xorb_hash = %p.hash))]
pub async fn upload_xorb(
    State(state): State<AppState>,
    Path(p): Path<XorbPath>,
    body: Body,
) -> Result<impl IntoResponse, AppError> {
    // Validate prefix
    if p.prefix != "default" && p.prefix != "default-merkledb" {
        return Err(AppError::BadRequest(format!("unknown prefix '{}'", p.prefix)));
    }

    // Decode hash from API encoding
    let hash_bytes = api_string_to_hash(&p.hash)
        .ok_or_else(|| AppError::BadRequest("invalid xorb hash".into()))?;

    // Collect body
    let data: Bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_bytes();

    let size_bytes = data.len() as i64;

    // Build S3 key using plain hex of the raw hash bytes
    let plain_hex = hex::encode(hash_bytes);
    let s3_key = xorb_key(&plain_hex);

    // Upload to S3
    state.s3.put_object(&s3_key, data).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Record in DB (idempotent upsert)
    let was_inserted = upsert_xorb(&state.pool, &hash_bytes, &s3_key, size_bytes)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!(was_inserted, s3_key, size_bytes, "xorb uploaded");

    Ok((StatusCode::OK, Json(json!({ "was_inserted": was_inserted }))))
}
