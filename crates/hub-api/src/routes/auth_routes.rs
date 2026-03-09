use crate::auth;
use crate::state::HubState;
use axum::{extract::State, Json};
use common::AppError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}

pub async fn register(
    State(state): State<HubState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password required".into(),
        ));
    }
    if req.username.len() < 3 || req.username.len() > 64 {
        return Err(AppError::BadRequest(
            "username must be 3-64 characters".into(),
        ));
    }

    // Check if username taken
    let existing = db_layer::queries::users::find_user_by_username(&state.pool, &req.username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if existing.is_some() {
        return Err(AppError::Conflict("username already taken".into()));
    }

    let password_hash = auth::hash_password(&req.password)?;
    let user = db_layer::queries::users::create_user(
        &state.pool,
        &req.username,
        req.email.as_deref(),
        &password_hash,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Auto-create an API token for the new user
    let (token, token_hash) = auth::generate_api_token();
    db_layer::queries::access_tokens::create_token(
        &state.pool,
        user.id,
        "default",
        &token_hash,
        &["repo.read".to_string(), "repo.write".to_string()],
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        username: user.username,
    }))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<HubState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = db_layer::queries::users::find_user_by_username(&state.pool, &req.username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    if !auth::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    // Generate a new token
    let (token, token_hash) = auth::generate_api_token();
    db_layer::queries::access_tokens::create_token(
        &state.pool,
        user.id,
        "login",
        &token_hash,
        &["repo.read".to_string(), "repo.write".to_string()],
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        username: user.username,
    }))
}
