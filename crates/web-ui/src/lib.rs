pub mod pages;

use axum::{routing::get, Router};
use hub_api::state::HubState;
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct WebState {
    pub hub: HubState,
    pub tera: Arc<Tera>,
}

/// Build the Web UI router.
pub fn web_router(hub_state: HubState, template_dir: &str) -> Router {
    let tera = match Tera::new(&format!("{}/**/*", template_dir)) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("Tera template error (non-fatal): {e}");
            Tera::default()
        }
    };

    let state = WebState {
        hub: hub_state,
        tera: Arc::new(tera),
    };

    Router::new()
        .route("/-/login", get(pages::login_page).post(pages::login_submit))
        .route(
            "/-/signup",
            get(pages::signup_page).post(pages::signup_submit),
        )
        .route("/-/logout", get(pages::logout))
        .route(
            "/-/new",
            get(pages::new_repo_page).post(pages::new_repo_submit),
        )
        .route("/", get(pages::home))
        .route("/{owner}", get(pages::user_profile))
        .route("/{owner}/{repo}", get(pages::repo_detail))
        .route("/{owner}/{repo}/tree/{revision}", get(pages::repo_tree))
        .route(
            "/{owner}/{repo}/tree/{revision}/{*path}",
            get(pages::repo_tree),
        )
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
