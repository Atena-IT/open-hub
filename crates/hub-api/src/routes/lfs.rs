use crate::auth;
use crate::state::HubState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use common::AppError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub struct LfsBatchRequest {
    pub operation: String,
    pub transfers: Option<Vec<String>>,
    pub objects: Vec<LfsBatchObject>,
}

#[derive(Deserialize, Serialize)]
pub struct LfsBatchObject {
    pub oid: String,
    pub size: i64,
}

#[derive(Serialize)]
pub struct LfsBatchResponse {
    pub transfer: String,
    pub objects: Vec<LfsBatchResponseObject>,
}

#[derive(Serialize)]
pub struct LfsBatchResponseObject {
    pub oid: String,
    pub size: i64,
    pub authenticated: bool,
    pub actions: Option<LfsActions>,
}

#[derive(Serialize)]
pub struct LfsActions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<LfsAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<LfsAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<LfsAction>,
}

#[derive(Serialize)]
pub struct LfsAction {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<std::collections::HashMap<String, String>>,
    pub expires_in: u64,
}

/// POST /:owner/:repo/info/lfs/objects/batch
pub async fn lfs_batch(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Json(req): Json<LfsBatchRequest>,
) -> Result<Json<LfsBatchResponse>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;

    let repo_clean = repo.strip_suffix(".git").unwrap_or(&repo);
    let full_name = format!("{}/{}", owner, repo_clean);
    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;

    match req.operation.as_str() {
        "download" => auth::ensure_repo_read_access(&repo_row, &full_name, Some(user_id))?,
        "upload" => auth::ensure_repo_write_access(&repo_row, user_id)?,
        _ => return Err(AppError::BadRequest("invalid operation".into())),
    }

    let base_url = &state.config.hub_base_url;
    let mut objects = Vec::new();

    let transfer = if let Some(transfers) = &req.transfers {
        if transfers.iter().any(|t| t == "xet") {
            "xet".to_string()
        } else {
            "basic".to_string()
        }
    } else {
        "basic".to_string()
    };

    for obj in &req.objects {
        let existing =
            db_layer::queries::lfs_objects::find_lfs_object(&state.pool, repo_row.id, &obj.oid)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

        let actions = match req.operation.as_str() {
            "download" => {
                if let Some(lfs_obj) = &existing {
                    if let Some(s3_key) = &lfs_obj.s3_key {
                        let url = state
                            .s3
                            .presign_get(s3_key, Duration::from_secs(3600))
                            .await
                            .map_err(|e| AppError::Internal(format!("presign failed: {e}")))?;
                        Some(LfsActions {
                            download: Some(LfsAction {
                                href: url,
                                header: None,
                                expires_in: 3600,
                            }),
                            upload: None,
                            verify: None,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "upload" => {
                if existing.is_some() && existing.as_ref().unwrap().s3_key.is_some() {
                    // Already uploaded — no actions needed
                    None
                } else {
                    let s3_key = s3_storage::lfs_key(&obj.oid);
                    let upload_url = state
                        .s3
                        .presign_put(&s3_key, Duration::from_secs(3600))
                        .await
                        .map_err(|e| AppError::Internal(format!("presign PUT failed: {e}")))?;

                    // Record placeholder
                    db_layer::queries::lfs_objects::upsert_lfs_object(
                        &state.pool,
                        repo_row.id,
                        &obj.oid,
                        obj.size,
                        None,
                    )
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                    Some(LfsActions {
                        download: None,
                        upload: Some(LfsAction {
                            href: upload_url,
                            header: None,
                            expires_in: 3600,
                        }),
                        verify: Some(LfsAction {
                            href: format!("{}/{}/info/lfs/verify", base_url, full_name),
                            header: None,
                            expires_in: 3600,
                        }),
                    })
                }
            }
            _ => return Err(AppError::BadRequest("invalid operation".into())),
        };

        objects.push(LfsBatchResponseObject {
            oid: obj.oid.clone(),
            size: obj.size,
            authenticated: true,
            actions,
        });
    }

    Ok(Json(LfsBatchResponse { transfer, objects }))
}

/// GET /:owner/:repo/info/lfs/objects/:oid — redirect to presigned download
pub async fn lfs_download(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, repo, oid)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    let repo_clean = repo.strip_suffix(".git").unwrap_or(&repo);
    let full_name = format!("{}/{}", owner, repo_clean);
    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;
    let requester_id = auth::resolve_optional_bearer_token(&state.pool, &headers).await?;
    auth::ensure_repo_read_access(&repo_row, &full_name, requester_id)?;

    let lfs_obj = db_layer::queries::lfs_objects::find_lfs_object(&state.pool, repo_row.id, &oid)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("LFS object not found".into()))?;

    let s3_key = lfs_obj
        .s3_key
        .ok_or_else(|| AppError::NotFound("LFS object not uploaded yet".into()))?;

    let url = state
        .s3
        .presign_get(&s3_key, Duration::from_secs(3600))
        .await
        .map_err(|e| AppError::Internal(format!("presign failed: {e}")))?;

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", &url)
        .body(Body::empty())
        .unwrap())
}

/// PUT /:owner/:repo/info/lfs/objects/:oid — direct upload (fallback)
pub async fn lfs_upload(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, repo, oid)): Path<(String, String, String)>,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;

    let repo_clean = repo.strip_suffix(".git").unwrap_or(&repo);
    let full_name = format!("{}/{}", owner, repo_clean);
    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;
    auth::ensure_repo_write_access(&repo_row, user_id)?;

    let s3_key = s3_storage::lfs_key(&oid);
    let body_len = body.len() as i64;
    state
        .s3
        .put_object(&s3_key, body)
        .await
        .map_err(|e| AppError::Internal(format!("S3 upload failed: {e}")))?;

    db_layer::queries::lfs_objects::upsert_lfs_object(
        &state.pool,
        repo_row.id,
        &oid,
        body_len,
        Some(&s3_key),
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::OK)
}

/// POST /:owner/:repo/info/lfs/verify
#[derive(Deserialize)]
pub struct LfsVerifyRequest {
    pub oid: String,
    pub size: i64,
}

pub async fn lfs_verify(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Json(req): Json<LfsVerifyRequest>,
) -> Result<StatusCode, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;

    let repo_clean = repo.strip_suffix(".git").unwrap_or(&repo);
    let full_name = format!("{}/{}", owner, repo_clean);
    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;
    auth::ensure_repo_write_access(&repo_row, user_id)?;

    // Update s3_key now that upload is complete
    let s3_key = s3_storage::lfs_key(&req.oid);
    let exists = state
        .s3
        .object_exists(&s3_key)
        .await
        .map_err(|e| AppError::Internal(format!("S3 check failed: {e}")))?;

    if !exists {
        return Err(AppError::NotFound("LFS object not found in storage".into()));
    }

    db_layer::queries::lfs_objects::upsert_lfs_object(
        &state.pool,
        repo_row.id,
        &req.oid,
        req.size,
        Some(&s3_key),
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::OK)
}
