//! Token vending machine — mirrors HF Hub's xet-{read|write}-token API.

use axum::{
    extract::{Path, State},
    Json,
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{middleware::Claims, state::AppState};
use common::AppError;

#[derive(Deserialize)]
pub struct TokenPath {
    pub repo_type: String,
    pub namespace: String,
    pub repo: String,
    pub revision: String,
}

pub async fn issue_read_token(
    State(state): State<AppState>,
    Path(p): Path<TokenPath>,
) -> Result<Json<Value>, AppError> {
    issue_token(state, p, "read").await
}

pub async fn issue_write_token(
    State(state): State<AppState>,
    Path(p): Path<TokenPath>,
) -> Result<Json<Value>, AppError> {
    issue_token(state, p, "write").await
}

async fn issue_token(state: AppState, p: TokenPath, scope: &str) -> Result<Json<Value>, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let exp = now + state.config.jwt_expiry_secs;

    let claims = Claims {
        sub: format!("{}/{}/{}", p.repo_type, p.namespace, p.repo),
        scope: scope.to_owned(),
        revision: p.revision.clone(),
        exp: exp as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "accessToken": token,
        "exp":         exp,
        "casUrl":      state.config.cas_base_url,
    })))
}
