use crate::audit;
use crate::domain::account::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, utoipa::ToSchema)]
pub struct AccountList {
    pub items: Vec<Account>,
}

/// Balance = initial_balance + sum(incoming) - sum(outgoing).
/// Subqueries are filtered on (source|destination)_account_id matching the row's id.
const BALANCE_EXPR: &str = "a.initial_balance \
    + COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = a.id AND deleted_at IS NULL), 0) \
    - COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id      = a.id AND deleted_at IS NULL), 0)";

#[utoipa::path(
    get, path = "/api/v1/accounts", tag = "accounts",
    security(("bearer" = [])),
    responses((status = 200, body = AccountList))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<Json<AccountList>> {
    // Revenue and expense are accounting buckets; they never appear in the user-facing list.
    let sql = format!(
        "SELECT a.id, a.name, a.type, a.currency, a.initial_balance, \
                {balance} AS current_balance, \
                CAST(a.archived AS INTEGER) != 0 AS archived, a.created_at, a.updated_at \
         FROM accounts a \
         WHERE a.user_id = ? AND a.deleted_at IS NULL AND a.type NOT IN ('revenue','expense') \
         ORDER BY a.created_at",
        balance = BALANCE_EXPR
    );
    let items: Vec<Account> = sqlx::query_as(&sql)
        .bind(cu.id.to_string())
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(AccountList { items }))
}

#[utoipa::path(
    post, path = "/api/v1/accounts", tag = "accounts",
    security(("bearer" = [])),
    request_body = CreateAccount,
    responses((status = 201, body = Account))
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateAccount>,
) -> ApiResult<(StatusCode, Json<Account>)> {
    // Public API may only create user-facing accounts. credit_card is created via the card
    // binding flow (phase C); liability via the loans flow. revenue and expense are bucket
    // accounts provisioned at register and not user-creatable.
    if !matches!(req.r#type.as_str(), "asset") {
        return Err(ApiError::Validation {
            field: "type".into(),
            reason: "RESERVED_TYPE: public API only creates 'asset' accounts".into(),
        });
    }
    let id = new_id();
    let now = now_iso();
    let initial = req.initial_balance.unwrap_or(0);
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO accounts (id, user_id, name, type, currency, initial_balance, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .bind(&req.name)
    .bind(&req.r#type)
    .bind(&req.currency)
    .bind(initial)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await
    .map_err(map_constraint)?;
    audit::write(&mut *tx, cu.id, "Account", &id.to_string(), "CREATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;
    let acct = load(&state.pool, cu.id, id).await?;
    Ok((StatusCode::CREATED, Json(acct)))
}

#[utoipa::path(
    get, path = "/api/v1/accounts/{id}", tag = "accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    responses((status = 200, body = Account))
)]
pub async fn show(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Account>> {
    Ok(Json(load(&state.pool, cu.id, id).await?))
}

#[utoipa::path(
    patch, path = "/api/v1/accounts/{id}", tag = "accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    request_body = UpdateAccount,
    responses((status = 200, body = Account), (status = 409, description = "stale write"))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateAccount>,
) -> ApiResult<Json<Account>> {
    let if_match = headers.get("if-match").and_then(|h| h.to_str().ok());
    let now = now_iso();
    let mut tx = state.pool.begin().await?;
    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT updated_at, type FROM accounts WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    let Some((current_ts, acct_type)) = existing else {
        return Err(ApiError::NotFound);
    };
    if matches!(acct_type.as_str(), "revenue" | "expense") {
        return Err(ApiError::Validation {
            field: "type".into(),
            reason: "bucket accounts (revenue/expense) cannot be modified".into(),
        });
    }
    if let Some(im) = if_match {
        if im != current_ts {
            return Err(ApiError::StaleWrite);
        }
    }
    sqlx::query(
        "UPDATE accounts SET name = COALESCE(?, name), archived = COALESCE(?, archived), updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ?",
    )
    .bind(req.name)
    .bind(req.archived.map(|b| if b { 1 } else { 0 }))
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&mut *tx)
    .await?;
    audit::write(&mut *tx, cu.id, "Account", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;
    Ok(Json(load(&state.pool, cu.id, id).await?))
}

#[utoipa::path(
    delete, path = "/api/v1/accounts/{id}", tag = "accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    responses((status = 204))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let now = now_iso();
    let mut tx = state.pool.begin().await?;
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT type FROM accounts WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((t,)) = existing {
        if matches!(t.as_str(), "revenue" | "expense") {
            return Err(ApiError::Validation {
                field: "type".into(),
                reason: "bucket accounts (revenue/expense) cannot be deleted".into(),
            });
        }
    } else {
        return Err(ApiError::NotFound);
    }
    let res = sqlx::query(
        "UPDATE accounts SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(&now)
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    audit::write(&mut *tx, cu.id, "Account", &id.to_string(), "DELETE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct BalanceQuery {
    pub as_of: Option<String>,
}

#[utoipa::path(
    get, path = "/api/v1/accounts/{id}/balance", tag = "accounts",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    responses((status = 200, body = AccountBalance))
)]
pub async fn balance(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(q): Query<BalanceQuery>,
) -> ApiResult<Json<AccountBalance>> {
    let as_of = q.as_of.clone();
    let as_of_filter = if as_of.is_some() { " AND tx_date <= ?" } else { "" };
    let sql = format!(
        "SELECT a.currency, a.initial_balance, \
                COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = a.id AND deleted_at IS NULL{filter}), 0) \
              - COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id      = a.id AND deleted_at IS NULL{filter}), 0) AS tx_total \
         FROM accounts a WHERE a.id = ? AND a.user_id = ? AND a.deleted_at IS NULL",
        filter = as_of_filter
    );
    let mut query = sqlx::query_as::<_, (String, i64, i64)>(&sql);
    if let Some(d) = &as_of {
        query = query.bind(d).bind(d);
    }
    let row = query
        .bind(id.to_string())
        .bind(cu.id.to_string())
        .fetch_optional(&state.pool)
        .await?;
    let Some((currency, initial, tx_total)) = row else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(AccountBalance {
        account_id: id.to_string(),
        currency,
        initial_balance: initial,
        transactions_total: tx_total,
        current_balance: initial + tx_total,
        as_of: as_of.unwrap_or_else(now_iso),
    }))
}

async fn load(pool: &sqlx::SqlitePool, user_id: Uuid, id: Uuid) -> ApiResult<Account> {
    let sql = format!(
        "SELECT a.id, a.name, a.type, a.currency, a.initial_balance, \
                {balance} AS current_balance, \
                CAST(a.archived AS INTEGER) != 0 AS archived, a.created_at, a.updated_at \
         FROM accounts a WHERE a.id = ? AND a.user_id = ? AND a.deleted_at IS NULL",
        balance = BALANCE_EXPR
    );
    let row: Option<Account> = sqlx::query_as(&sql)
        .bind(id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;
    row.ok_or(ApiError::NotFound)
}

fn map_constraint(e: sqlx::Error) -> ApiError {
    if let Some(db_err) = e.as_database_error() {
        if db_err.message().contains("FOREIGN KEY") {
            return ApiError::FkViolation(db_err.message().to_string());
        }
    }
    ApiError::Sqlx(e)
}
