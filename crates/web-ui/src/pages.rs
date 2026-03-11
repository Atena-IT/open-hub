use crate::WebState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use tera::Context;

fn render(state: &WebState, template: &str, ctx: &Context) -> Response {
    match state.tera.render(template, ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template render error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {e}"),
            )
                .into_response()
        }
    }
}

pub async fn home(State(state): State<WebState>) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", "Home");
    render(&state, "home.html", &ctx)
}

pub async fn login_page(State(state): State<WebState>) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", "Login");
    render(&state, "login.html", &ctx)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_submit(State(state): State<WebState>, Form(_form): Form<LoginForm>) -> Response {
    // For now, redirect to home after login attempt
    // Full session management would require cookies
    let mut ctx = Context::new();
    ctx.insert("title", "Login");
    ctx.insert(
        "message",
        "Login functionality coming soon. Use the API with tokens.",
    );
    render(&state, "login.html", &ctx)
}

pub async fn signup_page(State(state): State<WebState>) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", "Sign Up");
    render(&state, "signup.html", &ctx)
}

#[derive(Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

pub async fn signup_submit(
    State(state): State<WebState>,
    Form(_form): Form<SignupForm>,
) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", "Sign Up");
    ctx.insert(
        "message",
        "Registration functionality coming soon. Use the API.",
    );
    render(&state, "signup.html", &ctx)
}

pub async fn logout() -> Redirect {
    Redirect::to("/-/login")
}

pub async fn new_repo_page(State(state): State<WebState>) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", "New Repository");
    render(&state, "new_repo.html", &ctx)
}

#[derive(Deserialize)]
pub struct NewRepoForm {
    pub name: String,
    pub private: Option<String>,
}

pub async fn new_repo_submit(
    State(state): State<WebState>,
    Form(_form): Form<NewRepoForm>,
) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", "New Repository");
    ctx.insert(
        "message",
        "Repository creation via web coming soon. Use the API.",
    );
    render(&state, "new_repo.html", &ctx)
}

pub async fn user_profile(State(state): State<WebState>, Path(owner): Path<String>) -> Response {
    let mut ctx = Context::new();
    ctx.insert("title", &owner);
    ctx.insert("owner", &owner);

    // List repos for this owner
    if let Ok(Some(user)) =
        db_layer::queries::users::find_user_by_username(&state.hub.pool, &owner).await
    {
        if let Ok(repos) =
            db_layer::queries::repositories::list_repos_for_owner(&state.hub.pool, user.id).await
        {
            let repo_list: Vec<serde_json::Value> = repos
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "full_name": r.full_name,
                        "description": r.description,
                        "repo_type": r.repo_type,
                        "private": r.private,
                    })
                })
                .collect();
            ctx.insert("repos", &repo_list);
        }
    }

    render(&state, "user_profile.html", &ctx)
}

pub async fn repo_detail(
    State(state): State<WebState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Response {
    let full_name = format!("{}/{}", owner, repo);
    let mut ctx = Context::new();
    ctx.insert("title", &full_name);
    ctx.insert("owner", &owner);
    ctx.insert("repo", &repo);

    if let Ok(Some(repo_row)) =
        db_layer::queries::repositories::find_repo_by_full_name(&state.hub.pool, &full_name).await
    {
        ctx.insert("description", &repo_row.description);
        if let Ok(files) =
            db_layer::queries::repo_files::list_files(&state.hub.pool, repo_row.id, None).await
        {
            let file_list: Vec<serde_json::Value> = files
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "path": f.path,
                        "size": f.size,
                        "is_lfs": f.is_lfs,
                    })
                })
                .collect();
            ctx.insert("files", &file_list);
        }
    }

    render(&state, "repo_detail.html", &ctx)
}

pub async fn repo_tree(
    State(state): State<WebState>,
    Path(params): Path<Vec<(String, String)>>,
) -> Response {
    let params: std::collections::HashMap<_, _> = params.into_iter().collect();
    let owner = params.get("owner").map(|s| s.as_str()).unwrap_or("");
    let repo = params.get("repo").map(|s| s.as_str()).unwrap_or("");
    let path = params.get("path").map(|s| s.as_str());
    let full_name = format!("{}/{}", owner, repo);

    let mut ctx = Context::new();
    ctx.insert("title", &full_name);
    ctx.insert("owner", owner);
    ctx.insert("repo", repo);
    ctx.insert("current_path", path.unwrap_or(""));

    if let Ok(Some(repo_row)) =
        db_layer::queries::repositories::find_repo_by_full_name(&state.hub.pool, &full_name).await
    {
        if let Ok(files) =
            db_layer::queries::repo_files::list_files(&state.hub.pool, repo_row.id, path).await
        {
            let file_list: Vec<serde_json::Value> = files
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "path": f.path,
                        "size": f.size,
                        "is_lfs": f.is_lfs,
                    })
                })
                .collect();
            ctx.insert("files", &file_list);
        }
    }

    render(&state, "repo_tree.html", &ctx)
}
