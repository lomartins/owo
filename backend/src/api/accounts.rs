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

/// Balance = initial_balance + sum(incoming) - sum(outgoing), counted up to an
/// exclusive cutoff date. `excl_sql` is a SQL date expression: `date('now','+1 day')`
/// for "as of today", or a quoted `'YYYY-MM-01'` literal for end-of-month views.
/// Subqueries are filtered on (source|destination)_account_id matching the row's id.
/// initial_balance is now always 0 (opening value lives in a transaction), but it
/// is kept in the formula for forward-compatibility.
fn balance_expr(excl_sql: &str) -> String {
    format!(
        "a.initial_balance \
         + COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = a.id AND deleted_at IS NULL AND tx_date < {e}), 0) \
         - COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id      = a.id AND deleted_at IS NULL AND tx_date < {e}), 0)",
        e = excl_sql
    )
}

const TODAY_EXCL: &str = "date('now','+1 day')";

/// Exclusive cutoff for "end of month `month` (YYYY-MM)": a quoted `'YYYY-MM-01'`
/// literal for the first day of the FOLLOWING month. Validated server-side, so safe
/// to inline.
fn month_end_excl(month: &str) -> ApiResult<String> {
    if month.len() != 7 || month.as_bytes()[4] != b'-' {
        return Err(ApiError::Validation { field: "month".into(), reason: "expected YYYY-MM".into() });
    }
    let y: i32 = month[0..4].parse().map_err(|_| ApiError::Validation { field: "month".into(), reason: "invalid year".into() })?;
    let mo: u32 = month[5..7].parse().map_err(|_| ApiError::Validation { field: "month".into(), reason: "invalid month".into() })?;
    if !(1..=12).contains(&mo) {
        return Err(ApiError::Validation { field: "month".into(), reason: "month must be 01..12".into() });
    }
    let (ny, nm) = if mo == 12 { (y + 1, 1) } else { (y, mo + 1) };
    Ok(format!("'{:04}-{:02}-01'", ny, nm))
}

#[derive(Deserialize)]
pub struct AccountQuery {
    /// End-of-month basis (YYYY-MM). When omitted, balances are computed as of today.
    pub month: Option<String>,
}

/// Signed value of the account's opening-balance transaction (the leg paired with
/// the user's equity bucket). Positive = money started in the account.
const OPENING_EXPR: &str = "COALESCE((\
    SELECT CASE WHEN t.destination_account_id = a.id THEN t.value ELSE -t.value END \
    FROM transactions t \
    JOIN accounts e ON e.user_id = a.user_id AND e.type = 'equity' AND e.deleted_at IS NULL \
    WHERE t.deleted_at IS NULL \
      AND ((t.source_account_id = e.id AND t.destination_account_id = a.id) \
        OR (t.source_account_id = a.id AND t.destination_account_id = e.id)) \
    LIMIT 1), 0)";

#[utoipa::path(
    get, path = "/api/v1/accounts", tag = "accounts",
    security(("bearer" = [])),
    responses((status = 200, body = AccountList))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Query(q): Query<AccountQuery>,
) -> ApiResult<Json<AccountList>> {
    let excl = match q.month {
        Some(m) => month_end_excl(&m)?,
        None => TODAY_EXCL.to_string(),
    };
    // revenue / expense / equity are accounting buckets; never in the user list.
    let sql = format!(
        "SELECT a.id, a.name, a.type, a.currency, a.initial_balance, \
                {opening} AS opening_balance, \
                {balance} AS current_balance, \
                CAST(a.archived AS INTEGER) != 0 AS archived, a.created_at, a.updated_at \
         FROM accounts a \
         WHERE a.user_id = ? AND a.deleted_at IS NULL AND a.type NOT IN ('revenue','expense','equity') \
         ORDER BY a.created_at",
        opening = OPENING_EXPR,
        balance = balance_expr(&excl)
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
    // initial_balance lives in the ledger as an opening transaction, so the column
    // stays 0 and the value is fully editable later.
    sqlx::query(
        "INSERT INTO accounts (id, user_id, name, type, currency, initial_balance, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .bind(&req.name)
    .bind(&req.r#type)
    .bind(&req.currency)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await
    .map_err(map_constraint)?;
    if initial != 0 {
        set_opening_balance(&mut tx, cu.id, &id.to_string(), &req.currency, initial, &cu.device_id, &now).await?;
    }
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
    let existing: Option<(String, String, String)> = sqlx::query_as(
        "SELECT updated_at, type, currency FROM accounts WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    let Some((current_ts, acct_type, currency)) = existing else {
        return Err(ApiError::NotFound);
    };
    if matches!(acct_type.as_str(), "revenue" | "expense" | "equity") {
        return Err(ApiError::Validation {
            field: "type".into(),
            reason: "bucket accounts (revenue/expense/equity) cannot be modified".into(),
        });
    }
    if let Some(im) = if_match {
        if im != current_ts {
            return Err(ApiError::StaleWrite);
        }
    }
    sqlx::query(
        "UPDATE accounts SET name = COALESCE(?, name), currency = COALESCE(?, currency), \
            archived = COALESCE(?, archived), updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ?",
    )
    .bind(req.name)
    .bind(req.currency.as_deref())
    .bind(req.archived.map(|b| if b { 1 } else { 0 }))
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&mut *tx)
    .await?;
    if let Some(opening) = req.opening_balance {
        let cur = req.currency.as_deref().unwrap_or(&currency);
        set_opening_balance(&mut tx, cu.id, &id.to_string(), cur, opening, &cu.device_id, &now).await?;
    }
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
        if matches!(t.as_str(), "revenue" | "expense" | "equity") {
            return Err(ApiError::Validation {
                field: "type".into(),
                reason: "bucket accounts (revenue/expense/equity) cannot be deleted".into(),
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
                {opening} AS opening_balance, \
                {balance} AS current_balance, \
                CAST(a.archived AS INTEGER) != 0 AS archived, a.created_at, a.updated_at \
         FROM accounts a WHERE a.id = ? AND a.user_id = ? AND a.deleted_at IS NULL",
        opening = OPENING_EXPR,
        balance = balance_expr(TODAY_EXCL)
    );
    let row: Option<Account> = sqlx::query_as(&sql)
        .bind(id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;
    row.ok_or(ApiError::NotFound)
}

/// Fetch the user's equity (opening-balance) bucket, self-healing by creating it
/// when missing (e.g. accounts predating the equity migration).
async fn equity_bucket(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: Uuid,
    currency: &str,
    device_id: &str,
    now: &str,
) -> ApiResult<String> {
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM accounts WHERE user_id = ? AND type = 'equity' AND deleted_at IS NULL",
    )
    .bind(user_id.to_string())
    .fetch_optional(&mut **tx)
    .await?;
    if let Some((id,)) = existing {
        return Ok(id);
    }
    let id = new_id().to_string();
    sqlx::query(
        "INSERT INTO accounts (id, user_id, name, type, currency, initial_balance, archived, \
            created_at, updated_at, device_id) \
         VALUES (?, ?, 'Opening balance', 'equity', ?, 0, 0, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id.to_string())
    .bind(currency)
    .bind(now)
    .bind(now)
    .bind(device_id)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

/// Set an account's opening balance to `cents` (signed) by upserting the
/// opening-balance transaction paired with the equity bucket. `cents == 0`
/// removes the opening transaction.
async fn set_opening_balance(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: Uuid,
    account_id: &str,
    currency: &str,
    cents: i64,
    device_id: &str,
    now: &str,
) -> ApiResult<()> {
    let equity = equity_bucket(tx, user_id, currency, device_id, now).await?;

    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM transactions \
         WHERE user_id = ? AND deleted_at IS NULL \
           AND ((source_account_id = ? AND destination_account_id = ?) \
             OR (source_account_id = ? AND destination_account_id = ?)) \
         LIMIT 1",
    )
    .bind(user_id.to_string())
    .bind(&equity)
    .bind(account_id)
    .bind(account_id)
    .bind(&equity)
    .fetch_optional(&mut **tx)
    .await?;

    if cents == 0 {
        if let Some((tx_id,)) = existing {
            sqlx::query(
                "UPDATE transactions SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 WHERE id = ?",
            )
            .bind(now)
            .bind(now)
            .bind(&tx_id)
            .execute(&mut **tx)
            .await?;
        }
        return Ok(());
    }

    let (src, dst, value) = if cents > 0 {
        (equity.clone(), account_id.to_string(), cents)
    } else {
        (account_id.to_string(), equity.clone(), -cents)
    };

    if let Some((tx_id,)) = existing {
        sqlx::query(
            "UPDATE transactions SET source_account_id = ?, destination_account_id = ?, \
                value = ?, updated_at = ?, sync_version = sync_version + 1 WHERE id = ?",
        )
        .bind(&src)
        .bind(&dst)
        .bind(value)
        .bind(now)
        .bind(&tx_id)
        .execute(&mut **tx)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO transactions \
                (id, user_id, source_account_id, destination_account_id, category_id, \
                 payment_method, value, currency, description, tx_date, paid, \
                 created_at, updated_at, device_id) \
             VALUES (?, ?, ?, ?, NULL, 'CASH', ?, ?, 'Opening balance', ?, 1, ?, ?, ?)",
        )
        .bind(new_id().to_string())
        .bind(user_id.to_string())
        .bind(&src)
        .bind(&dst)
        .bind(value)
        .bind(currency)
        .bind(&now[..10.min(now.len())])
        .bind(now)
        .bind(now)
        .bind(device_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

fn map_constraint(e: sqlx::Error) -> ApiError {
    if let Some(db_err) = e.as_database_error() {
        if db_err.message().contains("FOREIGN KEY") {
            return ApiError::FkViolation(db_err.message().to_string());
        }
    }
    ApiError::Sqlx(e)
}
