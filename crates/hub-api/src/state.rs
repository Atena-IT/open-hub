use common::AppConfig;
use db_layer::PgPool;
use s3_storage::S3Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory store for the Jobs API stubs.
pub struct JobsStore {
    pub jobs: Mutex<HashMap<String, Value>>,
    pub scheduled_jobs: Mutex<HashMap<String, Value>>,
}

impl JobsStore {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
            scheduled_jobs: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone)]
pub struct HubState {
    pub pool: Arc<PgPool>,
    pub s3: Arc<S3Client>,
    pub config: Arc<AppConfig>,
    pub jobs: Arc<JobsStore>,
}

impl HubState {
    pub fn new(pool: PgPool, s3: S3Client, config: AppConfig) -> Self {
        Self {
            pool: Arc::new(pool),
            s3: Arc::new(s3),
            config: Arc::new(config),
            jobs: Arc::new(JobsStore::new()),
        }
    }

    /// Create from existing Arcs (shared with CAS server).
    pub fn from_arcs(pool: Arc<PgPool>, s3: Arc<S3Client>, config: Arc<AppConfig>) -> Self {
        Self {
            pool,
            s3,
            config,
            jobs: Arc::new(JobsStore::new()),
        }
    }
}
