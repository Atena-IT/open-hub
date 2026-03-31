/// Jobs API — mocked stub endpoints.
///
/// This module provides drop-in stub implementations for the Hugging Face Jobs API so
/// that the `huggingface_hub` Python client CLI (`hf jobs …`) can target this server
/// without errors.  No real compute is performed: jobs are stored in-memory and
/// immediately placed in the `"QUEUED"` stage.  The full implementation will replace
/// these stubs once the batch-7 delivery is complete.
///
/// Covered endpoints (all under `/api/`):
///
/// One-off jobs
/// - POST   /api/jobs/{owner}                        — run_job / run_uv_job
/// - GET    /api/jobs/{owner}                        — list_jobs
/// - GET    /api/jobs/{owner}/{job_id}               — inspect_job
/// - GET    /api/jobs/{owner}/{job_id}/logs          — fetch_job_logs (SSE)
/// - GET    /api/jobs/{owner}/{job_id}/metrics       — fetch_job_metrics (SSE)
/// - POST   /api/jobs/{owner}/{job_id}/cancel        — cancel_job
///
/// Hardware catalogue
/// - GET    /api/jobs/hardware                        — list_jobs_hardware
///
/// Scheduled jobs
/// - POST   /api/scheduled-jobs/{owner}                             — create_scheduled_job
/// - GET    /api/scheduled-jobs/{owner}                             — list_scheduled_jobs
/// - GET    /api/scheduled-jobs/{owner}/{scheduled_job_id}          — inspect_scheduled_job
/// - POST   /api/scheduled-jobs/{owner}/{scheduled_job_id}/suspend  — suspend_scheduled_job
/// - POST   /api/scheduled-jobs/{owner}/{scheduled_job_id}/resume   — resume_scheduled_job
/// - DELETE /api/scheduled-jobs/{owner}/{scheduled_job_id}          — delete_scheduled_job
use crate::auth;
use crate::state::HubState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
    Json,
};
use chrono::Utc;
use common::AppError;
use serde_json::{json, Value};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn new_id() -> String {
    // 24 hex chars matches the MongoDB-style ObjectId length used by the HF Jobs API
    uuid::Uuid::new_v4().simple().to_string()[..24].to_string()
}

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn make_owner(user_id: &str, username: &str) -> Value {
    json!({ "id": user_id, "name": username, "type": "user" })
}

// ─── One-off Jobs ─────────────────────────────────────────────────────────────

/// POST /api/jobs/{owner}
/// Body: the job_spec built by huggingface_hub._create_job_spec
pub async fn run_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(owner): Path<String>,
    Json(job_spec): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only create jobs in your own namespace".into(),
        ));
    }

    let job_id = new_id();
    let created_at = now_iso();

    let record = json!({
        "id": job_id,
        "createdAt": created_at,
        "dockerImage": job_spec.get("dockerImage"),
        "spaceId": job_spec.get("spaceId"),
        "command": job_spec.get("command").cloned().unwrap_or(json!([])),
        "arguments": job_spec.get("arguments").cloned().unwrap_or(json!([])),
        "environment": job_spec.get("environment").cloned().unwrap_or(json!({})),
        "secrets": job_spec.get("secrets").cloned().unwrap_or(json!({})),
        "flavor": job_spec.get("flavor").cloned().unwrap_or(json!("cpu-basic")),
        "labels": job_spec.get("labels"),
        "volumes": job_spec.get("volumes"),
        "status": { "stage": "QUEUED", "message": null },
        "owner": make_owner(&user_id.to_string(), &user.username),
    });

    state
        .jobs
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("jobs lock poisoned".into()))?
        .insert(format!("{}/{}", owner, job_id), record.clone());

    Ok(Json(record))
}

/// GET /api/jobs/{owner}
pub async fn list_jobs(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(owner): Path<String>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only list jobs in your own namespace".into(),
        ));
    }

    let prefix = format!("{}/", owner);
    let guard = state
        .jobs
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("jobs lock poisoned".into()))?;
    let jobs: Vec<Value> = guard
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v.clone())
        .collect();

    Ok(Json(json!(jobs)))
}

/// GET /api/jobs/{owner}/{job_id}
pub async fn inspect_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, job_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only inspect jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, job_id);
    let guard = state
        .jobs
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("jobs lock poisoned".into()))?;
    let record = guard
        .get(&key)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("job {job_id} not found")))?;

    Ok(Json(record))
}

/// POST /api/jobs/{owner}/{job_id}/cancel
pub async fn cancel_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, job_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only cancel jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, job_id);
    let mut guard = state
        .jobs
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("jobs lock poisoned".into()))?;
    let record = guard
        .get_mut(&key)
        .ok_or_else(|| AppError::NotFound(format!("job {job_id} not found")))?;

    if let Some(status) = record.get_mut("status") {
        *status = json!({ "stage": "CANCELLED", "message": null });
    }

    Ok(Json(record.clone()))
}

/// GET /api/jobs/{owner}/{job_id}/logs
///
/// Returns a minimal SSE stream.  The Python client iterates lines looking for
/// `data: {…}` events; an empty stream (204 / empty body) causes it to return
/// immediately with no log lines, which is the correct behaviour for a stub job
/// that never actually runs.
pub async fn fetch_job_logs(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, job_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only fetch logs for jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, job_id);
    let exists = state
        .jobs
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("jobs lock poisoned".into()))?
        .contains_key(&key);
    if !exists {
        return Err(AppError::NotFound(format!("job {job_id} not found")));
    }

    // Return an SSE stream with a single stub log line so that the client
    // has at least one complete event to parse.
    let body = format!(
        "data: {{\"timestamp\":\"{}\",\"data\":\"[stub] job queued, no compute available\"}}\n\n",
        now_iso()
    );
    let response = axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .body(axum::body::Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}

/// GET /api/jobs/{owner}/{job_id}/metrics
///
/// Returns a minimal SSE stream with one stub metrics snapshot.
pub async fn fetch_job_metrics(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, job_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only fetch metrics for jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, job_id);
    let exists = state
        .jobs
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("jobs lock poisoned".into()))?
        .contains_key(&key);
    if !exists {
        return Err(AppError::NotFound(format!("job {job_id} not found")));
    }

    let metrics_payload = json!({
        "cpu_usage_pct": 0,
        "cpu_millicores": 0,
        "memory_used_bytes": 0,
        "memory_total_bytes": 0,
        "rx_bps": 0,
        "tx_bps": 0,
        "gpus": {},
        "replica": "stub"
    });
    let body = format!("data: {}\n\n", metrics_payload);
    let response = axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .body(axum::body::Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}

// ─── Hardware Catalogue ───────────────────────────────────────────────────────

/// GET /api/jobs/hardware
pub async fn list_jobs_hardware(headers: HeaderMap) -> Result<Json<Value>, AppError> {
    // Require any bearer token (no user lookup needed for the catalogue).
    let _token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    Ok(Json(json!([
        {
            "name": "cpu-basic",
            "prettyName": "CPU Basic",
            "cpu": "2 vCPU",
            "ram": "16 GB",
            "accelerator": null,
            "unitCostMicroUSD": 167,
            "unitCostUSD": 0.000167,
            "unitLabel": "minute"
        },
        {
            "name": "cpu-upgrade",
            "prettyName": "CPU Upgrade",
            "cpu": "8 vCPU",
            "ram": "32 GB",
            "accelerator": null,
            "unitCostMicroUSD": 667,
            "unitCostUSD": 0.000667,
            "unitLabel": "minute"
        },
        {
            "name": "t4-small",
            "prettyName": "Nvidia T4 - small",
            "cpu": "4 vCPU",
            "ram": "15 GB",
            "accelerator": {
                "type": "gpu",
                "model": "T4",
                "quantity": "1",
                "vram": "16 GB",
                "manufacturer": "Nvidia"
            },
            "unitCostMicroUSD": 6000,
            "unitCostUSD": 0.006,
            "unitLabel": "minute"
        },
        {
            "name": "a10g-small",
            "prettyName": "Nvidia A10G - small",
            "cpu": "4 vCPU",
            "ram": "15 GB",
            "accelerator": {
                "type": "gpu",
                "model": "A10G",
                "quantity": "1",
                "vram": "24 GB",
                "manufacturer": "Nvidia"
            },
            "unitCostMicroUSD": 15000,
            "unitCostUSD": 0.015,
            "unitLabel": "minute"
        },
        {
            "name": "a100-large",
            "prettyName": "Nvidia A100 - large",
            "cpu": "12 vCPU",
            "ram": "142 GB",
            "accelerator": {
                "type": "gpu",
                "model": "A100",
                "quantity": "1",
                "vram": "80 GB",
                "manufacturer": "Nvidia"
            },
            "unitCostMicroUSD": 68000,
            "unitCostUSD": 0.068,
            "unitLabel": "minute"
        }
    ])))
}

// ─── Scheduled Jobs ───────────────────────────────────────────────────────────

/// POST /api/scheduled-jobs/{owner}
pub async fn create_scheduled_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(owner): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only create scheduled jobs in your own namespace".into(),
        ));
    }

    let job_id = new_id();
    let created_at = now_iso();

    let record = json!({
        "id": job_id,
        "createdAt": created_at,
        "jobSpec": body.get("jobSpec").cloned().unwrap_or(json!({})),
        "schedule": body.get("schedule"),
        "suspend": body.get("suspend").and_then(|v| v.as_bool()).unwrap_or(false),
        "concurrency": body.get("concurrency").and_then(|v| v.as_bool()).unwrap_or(false),
        "status": { "lastJob": null, "nextJobRunAt": null },
        "owner": make_owner(&user_id.to_string(), &user.username),
    });

    state
        .jobs
        .scheduled_jobs
        .lock()
        .map_err(|_| AppError::Internal("scheduled_jobs lock poisoned".into()))?
        .insert(format!("{}/{}", owner, job_id), record.clone());

    Ok(Json(record))
}

/// GET /api/scheduled-jobs/{owner}
pub async fn list_scheduled_jobs(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(owner): Path<String>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only list scheduled jobs in your own namespace".into(),
        ));
    }

    let prefix = format!("{}/", owner);
    let guard = state
        .jobs
        .scheduled_jobs
        .lock()
        .map_err(|_| AppError::Internal("scheduled_jobs lock poisoned".into()))?;
    let jobs: Vec<Value> = guard
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v.clone())
        .collect();

    Ok(Json(json!(jobs)))
}

/// GET /api/scheduled-jobs/{owner}/{scheduled_job_id}
pub async fn inspect_scheduled_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, scheduled_job_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only inspect scheduled jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, scheduled_job_id);
    let guard = state
        .jobs
        .scheduled_jobs
        .lock()
        .map_err(|_| AppError::Internal("scheduled_jobs lock poisoned".into()))?;
    let record = guard
        .get(&key)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("scheduled job {scheduled_job_id} not found")))?;

    Ok(Json(record))
}

/// POST /api/scheduled-jobs/{owner}/{scheduled_job_id}/suspend
pub async fn suspend_scheduled_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, scheduled_job_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    set_scheduled_job_suspend(&state, &headers, &owner, &scheduled_job_id, true).await
}

/// POST /api/scheduled-jobs/{owner}/{scheduled_job_id}/resume
pub async fn resume_scheduled_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, scheduled_job_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    set_scheduled_job_suspend(&state, &headers, &owner, &scheduled_job_id, false).await
}

async fn set_scheduled_job_suspend(
    state: &HubState,
    headers: &HeaderMap,
    owner: &str,
    scheduled_job_id: &str,
    suspend: bool,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only modify scheduled jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, scheduled_job_id);
    let mut guard = state
        .jobs
        .scheduled_jobs
        .lock()
        .map_err(|_| AppError::Internal("scheduled_jobs lock poisoned".into()))?;
    let record = guard
        .get_mut(&key)
        .ok_or_else(|| AppError::NotFound(format!("scheduled job {scheduled_job_id} not found")))?;

    if let Some(obj) = record.as_object_mut() {
        obj.insert("suspend".to_string(), serde_json::Value::Bool(suspend));
    }

    Ok(Json(record.clone()))
}

/// DELETE /api/scheduled-jobs/{owner}/{scheduled_job_id}
pub async fn delete_scheduled_job(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path((owner, scheduled_job_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;
    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    if user.username != owner {
        return Err(AppError::Forbidden(
            "can only delete scheduled jobs in your own namespace".into(),
        ));
    }

    let key = format!("{}/{}", owner, scheduled_job_id);
    let removed = state
        .jobs
        .scheduled_jobs
        .lock()
        .map_err(|_| AppError::Internal("scheduled_jobs lock poisoned".into()))?
        .remove(&key);

    if removed.is_none() {
        return Err(AppError::NotFound(format!(
            "scheduled job {scheduled_job_id} not found"
        )));
    }

    Ok(Json(json!({})))
}
