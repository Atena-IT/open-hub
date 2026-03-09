use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use common::AppError;
use crate::state::HubState;
use crate::auth;

/// GET /api/{type}s/:owner/:repo/tree/:revision[/*path]
pub async fn tree_list(
    State(state): State<HubState>,
    Path(params): Path<Vec<(String, String)>>,
) -> Result<Json<Vec<TreeEntry>>, AppError> {
    let params: std::collections::HashMap<_, _> = params.into_iter().collect();
    let owner = params.get("owner").ok_or_else(|| AppError::BadRequest("missing owner".into()))?;
    let repo = params.get("repo").ok_or_else(|| AppError::BadRequest("missing repo".into()))?;
    let path = params.get("path");

    let full_name = format!("{}/{}", owner, repo);
    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;

    let files = db_layer::queries::repo_files::list_files(
        &state.pool,
        repo_row.id,
        path.map(|s| s.as_str()),
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let prefix = path.map(|s| s.as_str()).unwrap_or("");
    let entries = build_tree_entries(&files, prefix);

    Ok(Json(entries))
}

#[derive(Serialize)]
pub struct TreeEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub oid: String,
    pub size: i64,
    pub path: String,
    pub lfs: Option<LfsInfo>,
}

#[derive(Serialize)]
pub struct LfsInfo {
    pub oid: String,
    pub size: i64,
}

fn build_tree_entries(
    files: &[db_layer::queries::repo_files::RepoFileRow],
    prefix: &str,
) -> Vec<TreeEntry> {
    use std::collections::HashSet;
    let mut entries = Vec::new();
    let mut seen_dirs: HashSet<String> = HashSet::new();

    for f in files {
        let rel_path = if prefix.is_empty() {
            &f.path
        } else if let Some(stripped) = f.path.strip_prefix(&format!("{}/", prefix)) {
            stripped
        } else {
            continue;
        };

        if let Some(slash_pos) = rel_path.find('/') {
            let dir_name = &rel_path[..slash_pos];
            let dir_path = if prefix.is_empty() {
                dir_name.to_string()
            } else {
                format!("{}/{}", prefix, dir_name)
            };
            if seen_dirs.insert(dir_path.clone()) {
                entries.push(TreeEntry {
                    entry_type: "directory".to_string(),
                    oid: String::new(),
                    size: 0,
                    path: dir_path,
                    lfs: None,
                });
            }
        } else {
            entries.push(TreeEntry {
                entry_type: "file".to_string(),
                oid: f.sha256.clone().unwrap_or_default(),
                size: f.size,
                path: f.path.clone(),
                lfs: if f.is_lfs {
                    f.lfs_oid.as_ref().map(|oid| LfsInfo {
                        oid: oid.clone(),
                        size: f.size,
                    })
                } else {
                    None
                },
            });
        }
    }
    entries
}

/// POST /api/{type}s/:owner/:repo/preupload/:revision
#[derive(Deserialize)]
pub struct PreuploadRequest {
    pub files: Vec<PreuploadFile>,
}

#[derive(Deserialize)]
pub struct PreuploadFile {
    pub path: String,
    pub size: i64,
    pub sample: Option<String>,
}

#[derive(Serialize)]
pub struct PreuploadResponse {
    pub files: Vec<PreuploadFileResponse>,
}

#[derive(Serialize)]
pub struct PreuploadFileResponse {
    pub path: String,
    #[serde(rename = "uploadMode")]
    pub upload_mode: String,
    #[serde(rename = "shouldIgnore")]
    pub should_ignore: bool,
}

pub async fn preupload(
    State(_state): State<HubState>,
    Path(_params): Path<Vec<(String, String)>>,
    Json(req): Json<PreuploadRequest>,
) -> Result<Json<PreuploadResponse>, AppError> {
    let files: Vec<PreuploadFileResponse> = req.files.iter().map(|f| {
        let upload_mode = if f.size > 10 * 1024 * 1024 {
            "lfs".to_string()
        } else {
            "regular".to_string()
        };
        PreuploadFileResponse {
            path: f.path.clone(),
            upload_mode,
            should_ignore: false,
        }
    }).collect();

    Ok(Json(PreuploadResponse { files }))
}

/// POST /api/{type}s/:owner/:repo/commit/:revision
/// Body is NDJSON (newline-delimited JSON)
pub async fn commit(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(params): Path<Vec<(String, String)>>,
    body: axum::body::Bytes,
) -> Result<Json<CommitResponse>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;

    let params: std::collections::HashMap<_, _> = params.into_iter().collect();
    let owner = params.get("owner").ok_or_else(|| AppError::BadRequest("missing owner".into()))?;
    let repo = params.get("repo").ok_or_else(|| AppError::BadRequest("missing repo".into()))?;
    let full_name = format!("{}/{}", owner, repo);

    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;

    // Parse NDJSON body
    let body_str = String::from_utf8(body.to_vec())
        .map_err(|_| AppError::BadRequest("invalid UTF-8 body".into()))?;

    let mut commit_message = String::from("Upload files");
    let mut operations: Vec<(String, serde_json::Value)> = Vec::new();

    for line in body_str.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        let val: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| AppError::BadRequest(format!("invalid NDJSON line: {e}")))?;

        let key = val.get("key").and_then(|k| k.as_str()).unwrap_or("");
        let value = val.get("value").cloned().unwrap_or(serde_json::Value::Null);

        match key {
            "header" => {
                if let Some(summary) = value.get("summary").and_then(|s| s.as_str()) {
                    commit_message = summary.to_string();
                }
            }
            "file" | "deletedFile" | "deletedFolder" | "lfsFile" => {
                operations.push((key.to_string(), value));
            }
            _ => {
                // Also handle legacy format: {"header": {"summary": "..."}}
                if let Some(header) = val.get("header") {
                    if let Some(summary) = header.get("summary").and_then(|s| s.as_str()) {
                        commit_message = summary.to_string();
                    }
                }
            }
        }
    }

    // Process each operation
    for (key, value) in &operations {
        let path = value.get("path").and_then(|p| p.as_str()).unwrap_or("");
        if path.is_empty() { continue; }

        match key.as_str() {
            "file" => {
                let content = value.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if !content.is_empty() {
                    let encoding = value.get("encoding").and_then(|e| e.as_str());
                    let file_bytes = if encoding == Some("base64") {
                        use base64::Engine;
                        base64::engine::general_purpose::STANDARD.decode(content)
                            .map_err(|e| AppError::BadRequest(format!("invalid base64: {e}")))?
                    } else {
                        content.as_bytes().to_vec()
                    };

                    let sha256 = hex::encode(sha2::Sha256::digest(&file_bytes));
                    let s3_key = s3_storage::repo_file_key(&full_name, &sha256);

                    state.s3.put_object(&s3_key, bytes::Bytes::from(file_bytes.clone()))
                        .await
                        .map_err(|e| AppError::Internal(format!("S3 upload failed: {e}")))?;

                    db_layer::queries::repo_files::upsert_file(
                        &state.pool,
                        repo_row.id,
                        path,
                        file_bytes.len() as i64,
                        Some(&sha256),
                        Some(&s3_key),
                        false,
                        None,
                    )
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "lfsFile" => {
                let oid = value.get("oid").and_then(|o| o.as_str()).unwrap_or("");
                let size = value.get("size").and_then(|s| s.as_i64()).unwrap_or(0);
                if !oid.is_empty() {
                    let s3_key = s3_storage::lfs_key(oid);
                    db_layer::queries::repo_files::upsert_file(
                        &state.pool,
                        repo_row.id,
                        path,
                        size,
                        Some(oid),
                        Some(&s3_key),
                        true,
                        Some(oid),
                    )
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "deletedFile" | "deletedFolder" => {
                db_layer::queries::repo_files::delete_file(&state.pool, repo_row.id, path)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
            _ => {}
        }
    }

    // Create commit record
    let sha = format!("{:x}", sha2::Sha256::digest(
        format!("{}:{}", chrono::Utc::now().timestamp(), commit_message).as_bytes()
    ));
    let commit_sha = &sha[..40];

    let parent_sha = repo_row.head_sha.as_deref();
    db_layer::queries::commits::create_commit(
        &state.pool,
        repo_row.id,
        commit_sha,
        &commit_message,
        Some(user_id),
        parent_sha,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    db_layer::queries::repositories::update_head_sha(&state.pool, repo_row.id, commit_sha)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(CommitResponse {
        commit_oid: commit_sha.to_string(),
        commit_url: format!("{}/{}/commit/{}", state.config.hub_base_url, full_name, commit_sha),
    }))
}

#[derive(Serialize)]
pub struct CommitResponse {
    #[serde(rename = "commitOid")]
    pub commit_oid: String,
    #[serde(rename = "commitUrl")]
    pub commit_url: String,
}

/// GET /{type}s/:owner/:repo/resolve/:revision/*path — download a file
pub async fn resolve_file(
    State(state): State<HubState>,
    method: Method,
    Path(params): Path<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let params: std::collections::HashMap<_, _> = params.into_iter().collect();
    let owner = params.get("owner").ok_or_else(|| AppError::BadRequest("missing owner".into()))?;
    let repo = params.get("repo").ok_or_else(|| AppError::BadRequest("missing repo".into()))?;
    let path = params.get("path").ok_or_else(|| AppError::BadRequest("missing path".into()))?;

    let full_name = format!("{}/{}", owner, repo);
    let repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;

    let file = db_layer::queries::repo_files::find_file(&state.pool, repo_row.id, path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("file '{}' not found", path)))?;

    let etag = file.sha256.as_deref().unwrap_or("");
    let commit_sha = repo_row.head_sha.as_deref().unwrap_or("main");

    if file.is_lfs {
        let oid = file.lfs_oid.as_deref().unwrap_or("");

        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", file.size.to_string())
            .header("ETag", format!("\"{}\"", etag))
            .header("X-Linked-ETag", format!("\"{}\"", etag))
            .header("X-Linked-Size", file.size.to_string())
            .header("X-Repo-Commit", commit_sha)
            .header("X-Xet-Hash", oid)
            .header(
                "X-Xet-Refresh-Route",
                format!("{}/api/models/{}/{}/xet-read-token/main", state.config.hub_base_url, owner, repo)
            );

        if method == Method::HEAD {
            return Ok(builder.body(Body::empty()).unwrap());
        } else {
            // For a GET, ideally it's downloaded via xet-core.
            // If they still try to GET via HTTP, we could proxy or redirect.
            // Since it's Xet, the bytes aren't in a single S3 object!
            // We'll just return a 400 or empty, assuming huggingface_hub uses hf_xet.
            // Or we could redirect to `/api/.../reconstructions` if they wanted to parse it, but that's not standard.
            // Wait, what if it was uploaded via "basic" LFS, not Xet?
            // Then it IS in S3! Let's check if the s3_key actually exists in S3 as a full file.
            // For now, if xet is enabled, `huggingface_hub` intercepts at HEAD anyway.
            if let Some(s3_key) = &file.s3_key {
                let url = state.s3.presign_get(s3_key, std::time::Duration::from_secs(3600))
                    .await
                    .map_err(|e| AppError::Internal(format!("presign failed: {e}")))?;
                return Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header("Location", &url)
                    .header("X-Linked-ETag", etag)
                    .header("X-Xet-Hash", oid)
                    .body(Body::empty())
                    .unwrap());
            }
            return Ok(builder.body(Body::empty()).unwrap());
        }
    }

    // Regular file: proxy content
    let s3_key = file.s3_key.as_deref()
        .ok_or_else(|| AppError::NotFound("file content not available".into()))?;

    let data = if method == Method::HEAD {
        bytes::Bytes::new()
    } else {
        state.s3.get_object(s3_key)
            .await
            .map_err(|e| AppError::Internal(format!("download failed: {e}")))?
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header("Content-Length", file.size.to_string())
        .header("ETag", format!("\"{}\"", etag))
        .header("X-Linked-ETag", format!("\"{}\"", etag))
        .header("X-Linked-Size", file.size.to_string())
        .header("X-Repo-Commit", commit_sha)
        .body(Body::from(data))
        .unwrap())
}
