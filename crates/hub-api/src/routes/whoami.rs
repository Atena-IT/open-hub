use crate::auth;
use crate::state::HubState;
use axum::http::HeaderMap;
use axum::{extract::State, Json};
use common::AppError;
use serde::Serialize;

#[derive(Serialize)]
pub struct WhoamiResponse {
    #[serde(rename = "type")]
    pub user_type: String,
    pub id: String,
    pub name: String,
    pub fullname: String,
    pub email: Option<String>,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    #[serde(rename = "canPay")]
    pub can_pay: bool,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub orgs: Vec<serde_json::Value>,
}

pub async fn whoami(
    State(state): State<HubState>,
    headers: HeaderMap,
) -> Result<Json<WhoamiResponse>, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;

    let user = db_layer::queries::users::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    Ok(Json(WhoamiResponse {
        user_type: "user".to_string(),
        id: user.id.to_string(),
        name: user.username.clone(),
        fullname: if user.full_name.is_empty() {
            user.username
        } else {
            user.full_name
        },
        email: user.email,
        email_verified: false,
        can_pay: false,
        avatar_url: user.avatar_url,
        orgs: vec![],
    }))
}
