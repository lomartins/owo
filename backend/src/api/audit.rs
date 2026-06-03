use crate::error::ApiResult;
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;

#[utoipa::path(get, path = "/api/v1/audit", tag = "audit", security(("bearer" = [])), responses((status = 501)))]
pub async fn list(_state: State<AppState>, _cu: Extension<CurrentUser>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
