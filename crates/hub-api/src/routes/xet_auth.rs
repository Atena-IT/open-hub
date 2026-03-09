use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use common::AppError;
use crate::state::HubState;
use crate::auth;

#[derive(Serialize)]
pub struct XetTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub exp: u64,
    #[serde(rename = "casUrl")]
    pub cas_url: String,
}

pub async fn get_xet_read_token(
    state: State<HubState>,
    headers: HeaderMap,
    params: Path<std::collections::HashMap<String, String>>,
) -> Result<Response, AppError> {
    get_xet_token_impl(state, headers, params, "read").await
}

pub async fn get_xet_write_token(
    state: State<HubState>,
    headers: HeaderMap,
    params: Path<std::collections::HashMap<String, String>>,
) -> Result<Response, AppError> {
    get_xet_token_impl(state, headers, params, "write").await
}

async fn get_xet_token_impl(
    State(state): State<HubState>,
    headers: HeaderMap,
    Path(params): Path<std::collections::HashMap<String, String>>,
    token_type: &str,
) -> Result<Response, AppError> {
    let token = auth::extract_bearer(&headers)
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    // Resolve user (ensure they exist and have access)
    let user_id = auth::resolve_bearer_token(&state.pool, token).await?;

    let owner = params.get("owner").ok_or_else(|| AppError::BadRequest("missing owner".into()))?;
    let repo = params.get("repo").ok_or_else(|| AppError::BadRequest("missing repo".into()))?;

    let full_name = format!("{}/{}", owner, repo);
    let _repo_row = db_layer::queries::repositories::find_repo_by_full_name(&state.pool, &full_name)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("repo '{}' not found", full_name)))?;

    // Validate token_type
    if token_type != "read" && token_type != "write" {
        return Err(AppError::BadRequest("invalid token type".into()));
    }

    let now = chrono::Utc::now().timestamp() as u64;
    let exp = now + state.config.jwt_expiry_secs;

    use jsonwebtoken::{encode, Header, EncodingKey};
    #[derive(serde::Serialize)]
    struct Claims {
        sub: String,
        scope: String,
        revision: String,
        exp: u64,
    }

    let claims = Claims {
        sub: full_name.clone(),
        scope: token_type.to_string(),
        revision: params.get("revision").cloned().unwrap_or_else(|| "main".to_string()),
        exp,
    };

    let cas_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes())
    ).map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    let json_body = Json(XetTokenResponse {
        access_token: cas_token.clone(),
        exp,
        cas_url: state.config.cas_base_url.clone(), // cas_base_url is where /v1/xorbs lives
    });

    let mut response = json_body.into_response();
    response.headers_mut().insert(
        axum::http::header::HeaderName::from_static("x-xet-cas-url"),
        axum::http::HeaderValue::from_str(&state.config.cas_base_url).unwrap(),
    );
    response.headers_mut().insert(
        axum::http::header::HeaderName::from_static("x-xet-access-token"),
        axum::http::HeaderValue::from_str(&cas_token).unwrap(),
    );
    response.headers_mut().insert(
        axum::http::header::HeaderName::from_static("x-xet-token-expiration"),
        axum::http::HeaderValue::from_str(&exp.to_string()).unwrap(),
    );

    Ok(response)
}
