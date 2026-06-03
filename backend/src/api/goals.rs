use crate::domain::goal::*;
use crate::error::ApiResult;
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct GoalList { pub items: Vec<Goal> }

#[utoipa::path(get, path = "/api/v1/goals", tag = "goals", security(("bearer" = [])),
    responses((status = 200, body = GoalList)))]
pub async fn list(State(state): State<AppState>, Extension(cu): Extension<CurrentUser>) -> ApiResult<Json<GoalList>> {
    let items: Vec<Goal> = sqlx::query_as(
        "SELECT id, name, target_value, currency, current_value, account_id, target_date, created_at, updated_at \
         FROM goals WHERE user_id = ? AND deleted_at IS NULL",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(GoalList { items }))
}

#[utoipa::path(post, path = "/api/v1/goals", tag = "goals", security(("bearer" = [])),
    request_body = CreateGoal, responses((status = 501)))]
pub async fn create(_state: State<AppState>, _cu: Extension<CurrentUser>, _body: Json<CreateGoal>)
    -> ApiResult<StatusCode> { Ok(StatusCode::NOT_IMPLEMENTED) }
