use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use crate::state::AppState;
use crate::routes::{health, auth, xorbs, shards, chunks, reconstructions};
use crate::middleware::jwt_auth;

pub fn build_router(state: AppState) -> Router {
    // Public routes (no auth required)
    let public = Router::new()
        .route("/health", get(health::health_check))
        // Token vending: repo_id = ":namespace/:repo" → split as two path params
        .route(
            "/api/:repo_type/:namespace/:repo/xet-read-token/:revision",
            get(auth::issue_read_token),
        )
        .route(
            "/api/:repo_type/:namespace/:repo/xet-write-token/:revision",
            get(auth::issue_write_token),
        )
        .route(
            "/api/:repo_type/:namespace/:repo/xet-write-token/:revision",
            post(auth::issue_write_token),
        );

    // Protected CAS routes (require JWT bearer token)
    let cas = Router::new()
        .route("/v1/xorbs/:prefix/:hash",          post(xorbs::upload_xorb))
        .route("/v1/shards",                       post(shards::upload_shard))
        .route("/shards",                          post(shards::upload_shard))
        .route("/v1/chunks/:prefix/:hash",         get(chunks::dedup_query))
        .route("/v1/reconstructions/:file_id",     get(reconstructions::get_reconstruction))
        .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

    Router::new()
        .merge(public)
        .merge(cas)
        .with_state(state)
}
