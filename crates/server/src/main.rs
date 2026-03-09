use common::AppConfig;
use s3_storage::S3Client;
use tracing::info;
use std::sync::Arc;
use std::net::SocketAddr;

use cas_server::state::AppState as CasState;
use hub_api::state::HubState;

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
    let config = AppConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Config error: {e}"))?;

    info!(
        bind_addr = %config.bind_addr,
        cas_base_url = %config.cas_base_url,
        hub_base_url = %config.hub_base_url,
        "Starting xet-server"
    );

    // ── Database ──────────────────────────────────────────────────────────────
    let pool = db_layer::create_pool(&config.database_url).await
        .map_err(|e| anyhow::anyhow!("DB connection failed: {e}"))?;
    info!("Database connected and migrations applied");

    // ── S3 / MinIO ────────────────────────────────────────────────────────────
    let s3 = S3Client::with_public_endpoint(
        config.s3_endpoint.as_deref(),
        config.s3_public_endpoint.as_deref(),
        &config.s3_region,
        &config.s3_bucket,
        &std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "minioadmin".into()),
        &std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".into()),
    )
    .await?;
    info!(bucket = %config.s3_bucket, "S3 client ready");

    // ── Shared state ─────────────────────────────────────────────────────────
    let pool_arc = Arc::new(pool);
    let s3_arc = Arc::new(s3);
    let config_arc = Arc::new(config.clone());

    let cas_state = CasState::new(
        (*pool_arc).clone(),
        (*s3_arc).clone(),
        config.clone(),
    );

    let hub_state = HubState::from_arcs(
        pool_arc.clone(),
        s3_arc.clone(),
        config_arc.clone(),
    );

    // ── Build unified router ─────────────────────────────────────────────────
    // Order matters: more specific routes first
    let template_dir = std::env::var("TEMPLATE_DIR").unwrap_or_else(|_| "templates".into());

    let app = axum::Router::new()
        .merge(cas_server::router::build_router(cas_state))
        .merge(hub_api::hub_router(hub_state.clone()))
        .merge(hub_api::lfs_router(hub_state.clone()))
        .merge(web_ui::web_router(hub_state, &template_dir))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = config.bind_addr.parse()?;
    info!(%addr, "Listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
