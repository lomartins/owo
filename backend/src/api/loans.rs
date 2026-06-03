use crate::domain::loan::*;
use crate::error::ApiResult;
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct LoanList { pub items: Vec<Loan> }

#[utoipa::path(get, path = "/api/v1/loans", tag = "loans", security(("bearer" = [])),
    responses((status = 200, body = LoanList)))]
pub async fn list(State(state): State<AppState>, Extension(cu): Extension<CurrentUser>) -> ApiResult<Json<LoanList>> {
    let items: Vec<Loan> = sqlx::query_as(
        "SELECT id, account_id, description, total_value, currency, interest_rate, installment_value, \
                installment_count, paid_installments, start_date, first_due_date, \
                CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM loans WHERE user_id = ? AND deleted_at IS NULL ORDER BY start_date DESC",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(LoanList { items }))
}

#[utoipa::path(post, path = "/api/v1/loans", tag = "loans", security(("bearer" = [])),
    request_body = CreateLoan, responses((status = 501)))]
pub async fn create(_state: State<AppState>, _cu: Extension<CurrentUser>, _body: Json<CreateLoan>)
    -> ApiResult<StatusCode> { Ok(StatusCode::NOT_IMPLEMENTED) }
