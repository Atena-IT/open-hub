pub mod auth_routes;
pub mod files;
pub mod lfs;
pub mod repos;
pub mod tokens;
pub mod whoami;
pub mod xet_auth;

use crate::state::HubState;
use axum::{
    routing::{delete, get, post},
    Router,
};

/// Hub API routes — all under /api/...
pub fn hub_api_router(state: HubState) -> Router {
    Router::new()
        // Auth
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login))
        // Whoami
        .route("/api/whoami", get(whoami::whoami))
        .route("/api/whoami-v2", get(whoami::whoami))
        // Repos
        .route("/api/models", get(repos::list_models))
        .route("/api/datasets", get(repos::list_datasets))
        .route("/api/repos/create", post(repos::create_repo))
        .route("/api/models/:owner/:repo", get(repos::repo_info))
        .route("/api/models/:owner/:repo/settings", axum::routing::put(repos::update_repo_settings))
        .route(
            "/api/models/:owner/:repo/revision/:revision",
            get(repos::repo_info_revision),
        )
        .route("/api/datasets/:owner/:repo", get(repos::repo_info))
        .route("/api/datasets/:owner/:repo/settings", axum::routing::put(repos::update_repo_settings))
        .route(
            "/api/datasets/:owner/:repo/revision/:revision",
            get(repos::repo_info_revision),
        )
        .route("/api/spaces/:owner/:repo", get(repos::repo_info))
        .route("/api/spaces/:owner/:repo/settings", axum::routing::put(repos::update_repo_settings))
        .route(
            "/api/spaces/:owner/:repo/revision/:revision",
            get(repos::repo_info_revision),
        )
        // Tree listing
        .route(
            "/api/models/:owner/:repo/tree/:revision",
            get(files::tree_list),
        )
        .route(
            "/api/models/:owner/:repo/tree/:revision/*path",
            get(files::tree_list),
        )
        .route(
            "/api/datasets/:owner/:repo/tree/:revision",
            get(files::tree_list),
        )
        .route(
            "/api/datasets/:owner/:repo/tree/:revision/*path",
            get(files::tree_list),
        )
        .route(
            "/api/spaces/:owner/:repo/tree/:revision",
            get(files::tree_list),
        )
        .route(
            "/api/spaces/:owner/:repo/tree/:revision/*path",
            get(files::tree_list),
        )
        // Preupload
        .route(
            "/api/models/:owner/:repo/preupload/:revision",
            post(files::preupload),
        )
        .route(
            "/api/datasets/:owner/:repo/preupload/:revision",
            post(files::preupload),
        )
        .route(
            "/api/spaces/:owner/:repo/preupload/:revision",
            post(files::preupload),
        )
        // Commit
        .route(
            "/api/models/:owner/:repo/commit/:revision",
            post(files::commit),
        )
        .route(
            "/api/datasets/:owner/:repo/commit/:revision",
            post(files::commit),
        )
        .route(
            "/api/spaces/:owner/:repo/commit/:revision",
            post(files::commit),
        )
        // Resolve (download) — with type prefix
        .route(
            "/models/:owner/:repo/resolve/:revision/*path",
            get(files::resolve_file),
        )
        .route(
            "/datasets/:owner/:repo/resolve/:revision/*path",
            get(files::resolve_file),
        )
        .route(
            "/spaces/:owner/:repo/resolve/:revision/*path",
            get(files::resolve_file),
        )
        // Resolve (download) — without type prefix (used by huggingface_hub for models)
        .route(
            "/:owner/:repo/resolve/:revision/*path",
            get(files::resolve_file),
        )
        // Tokens (combined GET + POST on same path)
        .route(
            "/api/tokens",
            get(tokens::list_tokens).post(tokens::create_token),
        )
        .route("/api/tokens/:token_id", delete(tokens::delete_token))
        // Xet CAS Auth
        .route(
            "/api/models/:owner/:repo/xet-read-token",
            get(xet_auth::get_xet_read_token),
        )
        .route(
            "/api/models/:owner/:repo/xet-read-token/:revision",
            get(xet_auth::get_xet_read_token),
        )
        .route(
            "/api/models/:owner/:repo/xet-write-token",
            get(xet_auth::get_xet_write_token),
        )
        .route(
            "/api/models/:owner/:repo/xet-write-token/:revision",
            get(xet_auth::get_xet_write_token),
        )
        .route(
            "/api/datasets/:owner/:repo/xet-read-token",
            get(xet_auth::get_xet_read_token),
        )
        .route(
            "/api/datasets/:owner/:repo/xet-read-token/:revision",
            get(xet_auth::get_xet_read_token),
        )
        .route(
            "/api/datasets/:owner/:repo/xet-write-token",
            get(xet_auth::get_xet_write_token),
        )
        .route(
            "/api/datasets/:owner/:repo/xet-write-token/:revision",
            get(xet_auth::get_xet_write_token),
        )
        .route(
            "/api/spaces/:owner/:repo/xet-read-token",
            get(xet_auth::get_xet_read_token),
        )
        .route(
            "/api/spaces/:owner/:repo/xet-read-token/:revision",
            get(xet_auth::get_xet_read_token),
        )
        .route(
            "/api/spaces/:owner/:repo/xet-write-token",
            get(xet_auth::get_xet_write_token),
        )
        .route(
            "/api/spaces/:owner/:repo/xet-write-token/:revision",
            get(xet_auth::get_xet_write_token),
        )
        // Delete repo
        .route("/api/validate-yaml", post(repos::validate_yaml))
        .route("/api/repos/delete", delete(repos::delete_repo))
        .with_state(state)
}

/// LFS routes — /:owner/:repo/info/lfs/...
pub fn lfs_api_router(state: HubState) -> Router {
    Router::new()
        .route(
            "/models/:owner/:repo/info/lfs/objects/batch",
            post(lfs::lfs_batch),
        )
        .route(
            "/datasets/:owner/:repo/info/lfs/objects/batch",
            post(lfs::lfs_batch),
        )
        .route(
            "/spaces/:owner/:repo/info/lfs/objects/batch",
            post(lfs::lfs_batch),
        )
        .route("/:owner/:repo/info/lfs/objects/batch", post(lfs::lfs_batch))
        // Combined GET + PUT on same path
        .route(
            "/models/:owner/:repo/info/lfs/objects/:oid",
            get(lfs::lfs_download).put(lfs::lfs_upload),
        )
        .route(
            "/datasets/:owner/:repo/info/lfs/objects/:oid",
            get(lfs::lfs_download).put(lfs::lfs_upload),
        )
        .route(
            "/spaces/:owner/:repo/info/lfs/objects/:oid",
            get(lfs::lfs_download).put(lfs::lfs_upload),
        )
        .route(
            "/:owner/:repo/info/lfs/objects/:oid",
            get(lfs::lfs_download).put(lfs::lfs_upload),
        )
        .route(
            "/models/:owner/:repo/info/lfs/verify",
            post(lfs::lfs_verify),
        )
        .route(
            "/datasets/:owner/:repo/info/lfs/verify",
            post(lfs::lfs_verify),
        )
        .route(
            "/spaces/:owner/:repo/info/lfs/verify",
            post(lfs::lfs_verify),
        )
        .route("/:owner/:repo/info/lfs/verify", post(lfs::lfs_verify))
        .with_state(state)
}
