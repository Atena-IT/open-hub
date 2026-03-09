use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr:             String,
    pub database_url:          String,
    pub s3_endpoint:           Option<String>,
    pub s3_bucket:             String,
    pub s3_region:             String,
    pub jwt_secret:            String,
    pub jwt_expiry_secs:       u64,
    pub cas_base_url:          String,
    /// Secret key for hashing ox_* API tokens (SHA-256 HMAC).
    pub hub_token_secret:      String,
    /// Base URL for the Hub API (used in LFS batch responses, etc.)
    pub hub_base_url:          String,
    /// Public S3 endpoint for presigned URLs (reachable by external clients).
    /// Defaults to S3_ENDPOINT if not set.
    pub s3_public_endpoint:    Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        use std::env::var;

        let cas_base_url = var("CAS_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".into());

        Ok(Self {
            bind_addr: var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            database_url: var("DATABASE_URL")
                .map_err(|_| AppError::Config("DATABASE_URL not set".into()))?,
            s3_endpoint: var("S3_ENDPOINT").ok(),
            s3_bucket: var("S3_BUCKET").unwrap_or_else(|_| "xet-storage".into()),
            s3_region: var("S3_REGION").unwrap_or_else(|_| "us-east-1".into()),
            jwt_secret: var("JWT_SECRET")
                .map_err(|_| AppError::Config("JWT_SECRET not set".into()))?,
            jwt_expiry_secs: var("JWT_EXPIRY_SECS")
                .unwrap_or_else(|_| "3600".into())
                .parse()
                .unwrap_or(3600),
            hub_base_url: var("HUB_BASE_URL")
                .unwrap_or_else(|_| cas_base_url.clone()),
            cas_base_url,
            hub_token_secret: var("HUB_TOKEN_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".into()),
            s3_public_endpoint: var("S3_PUBLIC_ENDPOINT").ok()
                .or_else(|| var("S3_ENDPOINT").ok()),
        })
    }
}
