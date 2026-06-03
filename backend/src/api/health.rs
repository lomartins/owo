use axum::Json;
use serde_json::{json, Value};

#[utoipa::path(
    get, path = "/health", tag = "health",
    responses((status = 200, description = "service healthy"))
)]
pub async fn check() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}
