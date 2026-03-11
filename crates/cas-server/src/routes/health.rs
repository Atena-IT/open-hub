use crate::state::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // Quick DB ping
    let db_ok = sqlx::query("SELECT 1")
        .execute(state.pool.as_ref())
        .await
        .is_ok();

    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "db":     if db_ok { "connected" } else { "error" },
    }))
}
