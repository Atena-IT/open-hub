use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use common::AppError;
use db_layer::PgPool;
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Hash a password with Argon2id.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("password hash failed: {e}")))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a new ox_* API token. Returns (plaintext_token, sha256_hash).
pub fn generate_api_token() -> (String, Vec<u8>) {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    let token = format!("ox_{}", hex::encode(random_bytes));
    let hash = sha256_hash(token.as_bytes());
    (token, hash)
}

/// SHA-256 hash of raw bytes.
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Resolve a Bearer token to a user ID.
/// Supports: ox_* API tokens.
pub async fn resolve_bearer_token(pool: &PgPool, token: &str) -> Result<Uuid, AppError> {
    if token.starts_with("ox_") {
        // API token: hash and look up
        let hash = sha256_hash(token.as_bytes());
        let row = db_layer::queries::access_tokens::find_token_by_hash(pool, &hash)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::Unauthorized("invalid token".into()))?;

        // Touch last_used (fire-and-forget is fine)
        let _ = db_layer::queries::access_tokens::touch_token(pool, row.id).await;

        Ok(row.user_id)
    } else {
        Err(AppError::Unauthorized("unsupported token format".into()))
    }
}

/// Extract and resolve Bearer token from request headers.
pub fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    value.strip_prefix("Bearer ")
}
