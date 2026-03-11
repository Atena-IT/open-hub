use common::AppConfig;
use s3_storage::S3Client;
use std::net::SocketAddr;
use tracing::info;

use cas_server::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.parse().unwrap()),
        )
        .init();

    // ── Config ────────────────────────────────────────────────────────────────
    let config = AppConfig::from_env().map_err(|e| anyhow::anyhow!("Config error: {e}"))?;

    info!(bind_addr = %config.bind_addr, cas_base_url = %config.cas_base_url, "Starting cas-server");

    // ── Database ──────────────────────────────────────────────────────────────
    let pool = db_layer::create_pool(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("DB connection failed: {e}"))?;
    info!("Database connected and migrations applied");

    // ── S3 / MinIO ────────────────────────────────────────────────────────────
    let s3 = S3Client::new(
        config.s3_endpoint.as_deref(),
        &config.s3_region,
        &config.s3_bucket,
        &std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "minioadmin".into()),
        &std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".into()),
    )
    .await?;
    info!(bucket = %config.s3_bucket, "S3 client ready");

    // ── HTTP Server ───────────────────────────────────────────────────────────
    let app_state = AppState::new(pool, s3, config.clone());
    let app = cas_server::router::build_router(app_state);

    let addr: SocketAddr = config.bind_addr.parse()?;
    info!(%addr, "Listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
