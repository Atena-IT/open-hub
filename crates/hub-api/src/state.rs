use common::AppConfig;
use db_layer::PgPool;
use s3_storage::S3Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct HubState {
    pub pool: Arc<PgPool>,
    pub s3: Arc<S3Client>,
    pub config: Arc<AppConfig>,
}

impl HubState {
    pub fn new(pool: PgPool, s3: S3Client, config: AppConfig) -> Self {
        Self {
            pool: Arc::new(pool),
            s3: Arc::new(s3),
            config: Arc::new(config),
        }
    }

    /// Create from existing Arcs (shared with CAS server).
    pub fn from_arcs(pool: Arc<PgPool>, s3: Arc<S3Client>, config: Arc<AppConfig>) -> Self {
        Self { pool, s3, config }
    }
}
