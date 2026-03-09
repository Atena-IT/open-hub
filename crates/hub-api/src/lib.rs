pub mod auth;
pub mod routes;
pub mod state;

use axum::Router;
use state::HubState;

/// Build the Hub API router (mounted at /api/... and /:owner/:repo/...).
pub fn hub_router(state: HubState) -> Router {
    routes::hub_api_router(state)
}

/// Build the LFS router (mounted at /:owner/:repo/info/lfs/...).
pub fn lfs_router(state: HubState) -> Router {
    routes::lfs_api_router(state)
}
