use crate::error::ApiResult;
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::Value;

#[utoipa::path(post, path = "/api/v1/sync/push", tag = "sync", security(("bearer" = [])), responses((status = 501)))]
pub async fn push(_state: State<AppState>, _cu: Extension<CurrentUser>, _body: Json<Value>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[utoipa::path(get, path = "/api/v1/sync/pull", tag = "sync", security(("bearer" = [])), responses((status = 501)))]
pub async fn pull(_state: State<AppState>, _cu: Extension<CurrentUser>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
