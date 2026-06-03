use crate::error::ApiResult;
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::Value;
use uuid::Uuid;

#[utoipa::path(post, path = "/api/v1/backup/create", tag = "backup", security(("bearer" = [])), responses((status = 501)))]
pub async fn create(_state: State<AppState>, _cu: Extension<CurrentUser>, _body: Json<Value>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[utoipa::path(get, path = "/api/v1/backup/jobs/{id}", tag = "backup", security(("bearer" = [])),
    params(("id" = Uuid, Path,)), responses((status = 501)))]
pub async fn status(_state: State<AppState>, _cu: Extension<CurrentUser>, _id: Path<Uuid>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[utoipa::path(get, path = "/api/v1/backup/download/{id}", tag = "backup", security(("bearer" = [])),
    params(("id" = Uuid, Path,)), responses((status = 501)))]
pub async fn download(_state: State<AppState>, _cu: Extension<CurrentUser>, _id: Path<Uuid>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[utoipa::path(post, path = "/api/v1/backup/restore", tag = "backup", security(("bearer" = [])), responses((status = 501)))]
pub async fn restore(_state: State<AppState>, _cu: Extension<CurrentUser>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
