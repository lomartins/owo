use crate::domain::investment::*;
use crate::error::ApiResult;
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct InvestmentList { pub items: Vec<Investment> }

#[utoipa::path(get, path = "/api/v1/investments", tag = "investments", security(("bearer" = [])),
    responses((status = 200, body = InvestmentList)))]
pub async fn list(State(state): State<AppState>, Extension(cu): Extension<CurrentUser>) -> ApiResult<Json<InvestmentList>> {
    let items: Vec<Investment> = sqlx::query_as(
        "SELECT id, account_id, name, type, principal, current_value, currency, rate_index, rate_spread, \
                purchase_date, expiration_date, CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM investments WHERE user_id = ? AND deleted_at IS NULL ORDER BY purchase_date DESC",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(InvestmentList { items }))
}

#[utoipa::path(post, path = "/api/v1/investments", tag = "investments", security(("bearer" = [])),
    request_body = CreateInvestment, responses((status = 501)))]
pub async fn create(_state: State<AppState>, _cu: Extension<CurrentUser>, _body: Json<CreateInvestment>)
    -> ApiResult<StatusCode> { Ok(StatusCode::NOT_IMPLEMENTED) }
