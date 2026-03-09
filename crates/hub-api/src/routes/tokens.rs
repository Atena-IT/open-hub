use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use common::AppError;
use crate::state::HubState;
use crate::auth;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub scopes: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub id: String,
    pub name: String,
    pub token: Option<String>,
    pub created_at: String,
}

pub async fn create_token(
    State(state): State<HubState>,
    headers: HeaderMap,
    Json(req): Json<CreateTokenRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let bearer = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, bearer).await?;

    let scopes = req.scopes.unwrap_or_else(|| vec!["repo.read".to_string(), "repo.write".to_string()]);
    let (token, token_hash) = auth::generate_api_token();

    let row = db_layer::queries::access_tokens::create_token(
        &state.pool,
        user_id,
        &req.name,
        &token_hash,
        &scopes,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(TokenResponse {
        id: row.id.to_string(),
        name: row.name,
        token: Some(token),
        created_at: row.created_at.to_rfc3339(),
    }))
}

pub async fn list_tokens(
    State(state): State<HubState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TokenResponse>>, AppError> {
    let bearer = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, bearer).await?;

    let rows = db_layer::queries::access_tokens::list_tokens_for_user(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let tokens: Vec<TokenResponse> = rows.iter().map(|r| TokenResponse {
        id: r.id.to_string(),
        name: r.name.clone(),
        token: None,
        created_at: r.created_at.to_rfc3339(),
    }).collect();

    Ok(Json(tokens))
}

pub async fn delete_token(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let bearer = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;
    let user_id = auth::resolve_bearer_token(&state.pool, bearer).await?;

    let token_uuid = Uuid::parse_str(&token_id)
        .map_err(|_| AppError::BadRequest("invalid token id".into()))?;

    let deleted = db_layer::queries::access_tokens::delete_token(&state.pool, token_uuid, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !deleted {
        return Err(AppError::NotFound("token not found".into()));
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}
