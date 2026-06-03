use crate::domain::bill::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, utoipa::ToSchema)]
pub struct BillList {
    pub items: Vec<Bill>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateBill {
    pub description: Option<String>,
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>)]
    pub value: Option<i64>,
    pub due_day: Option<i64>,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    /// "this_month" | "this_and_next" | "all".
    /// Default: "all" (legacy behavior, patches the template).
    pub scope: Option<String>,
    /// Reference month for scoped edits (YYYY-MM). Defaults to current.
    pub month: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PayBillRequest {
    pub source_account_id: String,
    /// Override the tx date (defaults to the bill's due day in the resolved month).
    pub tx_date: Option<String>,
    /// Defaults to PIX.
    pub payment_method: Option<String>,
    /// Override the amount for this month (e.g. variable utility bill).
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>)]
    pub amount: Option<i64>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    /// YYYY-MM. If omitted, defaults to the server's current month.
    pub month: Option<String>,
}

fn validate_month(m: &str) -> ApiResult<()> {
    if m.len() == 7 && m.as_bytes()[4] == b'-' {
        if let (Ok(_y), Ok(mo)) = (m[0..4].parse::<i32>(), m[5..7].parse::<u32>()) {
            if (1..=12).contains(&mo) {
                return Ok(());
            }
        }
    }
    Err(ApiError::Validation {
        field: "month".into(),
        reason: "expected YYYY-MM".into(),
    })
}

fn current_month() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m").to_string()
}

fn validate_due_day(d: i64) -> ApiResult<()> {
    if (1..=31).contains(&d) {
        Ok(())
    } else {
        Err(ApiError::Validation {
            field: "due_day".into(),
            reason: "must be between 1 and 31".into(),
        })
    }
}

/// SELECT clause that LEFT JOINs bill_payments (for the per-month paid flag) and
/// bill_overrides (for per-month value/due_day/category/etc overrides). Override
/// fields win when set, template falls back. `paid` / `paid_at` reflect the
/// queried month.
const BILL_COLS_WITH_PAY: &str = "b.id, \
    COALESCE(o.account_id,  b.account_id)  AS account_id, \
    COALESCE(o.category_id, b.category_id) AS category_id, \
    COALESCE(o.description, b.description) AS description, \
    COALESCE(o.value,       b.value)       AS value, \
    b.currency, \
    COALESCE(o.due_day, b.due_day)         AS due_day, \
    (p.id IS NOT NULL) AS paid, p.paid_at, p.transaction_id AS paid_transaction_id, \
    b.created_at, b.updated_at";

/// JOIN fragment used by both `list` and `load_bill_for_month`. The query that
/// includes it binds the month parameter TWICE (once for payments, once for
/// overrides), then the rest of the WHERE bindings.
const BILL_JOINS: &str = "FROM bills b \
    LEFT JOIN bill_payments  p ON p.bill_id  = b.id AND p.month = ? AND p.user_id = b.user_id \
    LEFT JOIN bill_overrides o ON o.bill_id  = b.id AND o.month = ?";

#[utoipa::path(get, path = "/api/v1/bills", tag = "bills", security(("bearer" = [])),
    params(("month" = Option<String>, Query, description = "YYYY-MM (defaults to current)")),
    responses((status = 200, body = BillList)))]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<BillList>> {
    let month = q.month.unwrap_or_else(current_month);
    validate_month(&month)?;

    let sql = format!(
        "SELECT {cols} {joins} \
         WHERE b.user_id = ? AND b.deleted_at IS NULL \
         ORDER BY COALESCE(o.due_day, b.due_day), b.description",
        cols = BILL_COLS_WITH_PAY,
        joins = BILL_JOINS,
    );
    let items: Vec<Bill> = sqlx::query_as(&sql)
        .bind(&month) // bill_payments.month
        .bind(&month) // bill_overrides.month
        .bind(cu.id.to_string())
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(BillList { items }))
}

#[utoipa::path(post, path = "/api/v1/bills", tag = "bills", security(("bearer" = [])),
    request_body = CreateBill, responses((status = 201, body = Bill)))]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateBill>,
) -> ApiResult<(StatusCode, Json<Bill>)> {
    if req.value <= 0 {
        return Err(ApiError::Validation {
            field: "value".into(),
            reason: "must be > 0".into(),
        });
    }
    validate_due_day(req.due_day)?;

    let id = new_id().to_string();
    let now = now_iso();
    // Keep a synthetic due_date for back-compat (some queries / clients still read it).
    let synthetic_due = format!("{}-{:02}", current_month(), req.due_day);

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO bills (id, user_id, account_id, category_id, description, value, currency, \
            due_date, due_day, recurrence, paid, paid_at, \
            created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'MONTHLY', 0, NULL, ?, ?, ?)",
    )
    .bind(&id)
    .bind(cu.id.to_string())
    .bind(req.account_id.as_deref())
    .bind(req.category_id.as_deref())
    .bind(&req.description)
    .bind(req.value)
    .bind(&req.currency)
    .bind(&synthetic_due)
    .bind(req.due_day)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await?;
    crate::audit::write(&mut *tx, cu.id, "Bill", &id, "CREATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Bill = load_bill_for_month(&state.pool, cu.id.to_string().as_str(), &id, &current_month()).await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(patch, path = "/api/v1/bills/{id}", tag = "bills", security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    request_body = UpdateBill, responses((status = 200, body = Bill)))]
pub async fn update(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBill>,
) -> ApiResult<Json<Bill>> {
    if let Some(d) = req.due_day {
        validate_due_day(d)?;
    }
    if let Some(v) = req.value {
        if v <= 0 {
            return Err(ApiError::Validation {
                field: "value".into(),
                reason: "must be > 0".into(),
            });
        }
    }
    // Confirm bill exists + belongs to user.
    let exists: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM bills WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    let scope = req.scope.as_deref().unwrap_or("all");
    let month = req.month.clone().unwrap_or_else(current_month);
    validate_month(&month)?;
    let now = now_iso();
    let mut tx = state.pool.begin().await?;

    match scope {
        "this_month" => {
            upsert_override(&mut *tx, &id.to_string(), &month, &req, &now).await?;
        }
        "this_and_next" => {
            upsert_override(&mut *tx, &id.to_string(), &month, &req, &now).await?;
            let next = shift_month_str(&month, 1)?;
            upsert_override(&mut *tx, &id.to_string(), &next, &req, &now).await?;
        }
        "all" => {
            // Patch the template.
            sqlx::query(
                "UPDATE bills SET \
                    description = COALESCE(?, description), \
                    value       = COALESCE(?, value), \
                    due_day     = COALESCE(?, due_day), \
                    account_id  = COALESCE(?, account_id), \
                    category_id = COALESCE(?, category_id), \
                    updated_at  = ?, sync_version = sync_version + 1 \
                 WHERE id = ? AND user_id = ?",
            )
            .bind(req.description.as_deref())
            .bind(req.value)
            .bind(req.due_day)
            .bind(req.account_id.as_deref())
            .bind(req.category_id.as_deref())
            .bind(&now)
            .bind(id.to_string())
            .bind(cu.id.to_string())
            .execute(&mut *tx)
            .await?;

            // Clear per-month overrides — template is now the source of truth.
            sqlx::query("DELETE FROM bill_overrides WHERE bill_id = ?")
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;

            // Retroactively patch past transactions tied to this bill so all
            // already-paid months reflect the new template. Audit history is
            // preserved via the `bill_payments` row pointing at the tx id.
            sqlx::query(
                "UPDATE transactions SET \
                    description = COALESCE(?, description), \
                    value       = COALESCE(?, value), \
                    category_id = COALESCE(?, category_id), \
                    updated_at  = ?, sync_version = sync_version + 1 \
                 WHERE bill_id = ? AND user_id = ? AND deleted_at IS NULL",
            )
            .bind(req.description.as_deref())
            .bind(req.value)
            .bind(req.category_id.as_deref())
            .bind(&now)
            .bind(id.to_string())
            .bind(cu.id.to_string())
            .execute(&mut *tx)
            .await?;
        }
        other => {
            return Err(ApiError::Validation {
                field: "scope".into(),
                reason: format!("unknown scope: {other} (this_month | this_and_next | all)"),
            });
        }
    }

    crate::audit::write(&mut *tx, cu.id, "Bill", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row = load_bill_for_month(&state.pool, cu.id.to_string().as_str(), &id.to_string(), &month).await?;
    Ok(Json(row))
}

#[utoipa::path(delete, path = "/api/v1/bills/{id}", tag = "bills", security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    responses((status = 204)))]
pub async fn delete(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let now = now_iso();
    let res = sqlx::query(
        "UPDATE bills SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(&now)
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&state.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct PayQuery {
    /// YYYY-MM (defaults to current).
    pub month: Option<String>,
}

/// Pay a bill for a specific month. Inserts a per-month payment row and creates
/// the corresponding transaction. Idempotent: if already paid this month, 409.
#[utoipa::path(post, path = "/api/v1/bills/{id}/pay", tag = "bills", security(("bearer" = [])),
    params(
        ("id" = Uuid, Path,),
        ("month" = Option<String>, Query, description = "YYYY-MM"),
    ),
    request_body = PayBillRequest,
    responses((status = 201, body = crate::domain::transaction::Transaction)))]
pub async fn pay(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(q): Query<PayQuery>,
    Json(req): Json<PayBillRequest>,
) -> ApiResult<(StatusCode, Json<crate::domain::transaction::Transaction>)> {
    let month = q.month.unwrap_or_else(current_month);
    validate_month(&month)?;

    let bill: Option<(String, Option<String>, String, i64, String, i64)> = sqlx::query_as(
        "SELECT account_id, category_id, description, value, currency, due_day \
         FROM bills WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((_bill_account, category_id, description, default_value, currency, due_day)) = bill
    else {
        return Err(ApiError::NotFound);
    };

    // Reject double-pay for the same month.
    let already: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM bill_payments WHERE bill_id = ? AND month = ?",
    )
    .bind(id.to_string())
    .bind(&month)
    .fetch_optional(&state.pool)
    .await?;
    if already.is_some() {
        return Err(ApiError::Conflict);
    }

    let (expense_bucket,): (String,) = sqlx::query_as(
        "SELECT id FROM accounts WHERE user_id = ? AND type = 'expense' AND deleted_at IS NULL",
    )
    .bind(cu.id.to_string())
    .fetch_one(&state.pool)
    .await?;

    if req.source_account_id == expense_bucket {
        return Err(ApiError::Validation {
            field: "source_account_id".into(),
            reason: "must be an asset account, not the expense bucket".into(),
        });
    }

    let value = req.amount.unwrap_or(default_value);
    if value <= 0 {
        return Err(ApiError::Validation {
            field: "amount".into(),
            reason: "must be > 0".into(),
        });
    }
    let tx_date = req
        .tx_date
        .unwrap_or_else(|| format!("{}-{:02}", month, due_day.clamp(1, 28)));
    let payment_method = req.payment_method.unwrap_or_else(|| "PIX".into());

    let tx_id = new_id().to_string();
    let pay_id = new_id().to_string();
    let now = now_iso();

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO transactions \
            (id, user_id, source_account_id, destination_account_id, category_id, \
             payment_method, value, currency, fx_rate, description, tx_date, paid, \
             bill_id, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, 1, ?, ?, ?, ?)",
    )
    .bind(&tx_id)
    .bind(cu.id.to_string())
    .bind(&req.source_account_id)
    .bind(&expense_bucket)
    .bind(category_id.as_deref())
    .bind(&payment_method)
    .bind(value)
    .bind(&currency)
    .bind(&description)
    .bind(&tx_date)
    .bind(id.to_string())
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO bill_payments (id, user_id, bill_id, month, transaction_id, paid_at, created_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&pay_id)
    .bind(cu.id.to_string())
    .bind(id.to_string())
    .bind(&month)
    .bind(&tx_id)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await?;

    crate::audit::write(&mut *tx, cu.id, "Bill", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: crate::domain::transaction::Transaction = sqlx::query_as(
        "SELECT id, source_account_id, destination_account_id, category_id, payment_method, \
                value, currency, fx_rate, description, tx_date, \
                CAST(paid AS INTEGER) != 0 AS paid, \
                receipt_url, picture_url, card_id, bill_id, invoice_id, created_at, updated_at \
         FROM transactions WHERE id = ?",
    )
    .bind(&tx_id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// Undo a payment for a specific month: deletes the bill_payments row and the
/// linked transaction. Caller passes ?month=YYYY-MM.
#[utoipa::path(post, path = "/api/v1/bills/{id}/reset", tag = "bills", security(("bearer" = [])),
    params(
        ("id" = Uuid, Path,),
        ("month" = Option<String>, Query, description = "YYYY-MM"),
    ),
    responses((status = 200, body = Bill)))]
pub async fn reset(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(q): Query<PayQuery>,
) -> ApiResult<Json<Bill>> {
    let month = q.month.unwrap_or_else(current_month);
    validate_month(&month)?;

    let pay: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, transaction_id FROM bill_payments \
         WHERE bill_id = ? AND month = ? AND user_id = ?",
    )
    .bind(id.to_string())
    .bind(&month)
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((pay_id, tx_id_opt)) = pay else {
        return Err(ApiError::NotFound);
    };

    let now = now_iso();
    let mut tx = state.pool.begin().await?;

    if let Some(tx_id) = tx_id_opt {
        sqlx::query(
            "UPDATE transactions SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
             WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
        )
        .bind(&now)
        .bind(&now)
        .bind(&tx_id)
        .bind(cu.id.to_string())
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("DELETE FROM bill_payments WHERE id = ?")
        .bind(&pay_id)
        .execute(&mut *tx)
        .await?;

    crate::audit::write(&mut *tx, cu.id, "Bill", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row = load_bill_for_month(&state.pool, cu.id.to_string().as_str(), &id.to_string(), &month).await?;
    Ok(Json(row))
}

async fn load_bill_for_month(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    bill_id: &str,
    month: &str,
) -> ApiResult<Bill> {
    let sql = format!(
        "SELECT {cols} {joins} \
         WHERE b.id = ? AND b.user_id = ? AND b.deleted_at IS NULL",
        cols = BILL_COLS_WITH_PAY,
        joins = BILL_JOINS,
    );
    let row: Option<Bill> = sqlx::query_as(&sql)
        .bind(month) // payments
        .bind(month) // overrides
        .bind(bill_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    row.ok_or(ApiError::NotFound)
}

fn shift_month_str(m: &str, delta: i32) -> ApiResult<String> {
    if m.len() != 7 || m.as_bytes()[4] != b'-' {
        return Err(ApiError::Validation { field: "month".into(), reason: "expected YYYY-MM".into() });
    }
    let y: i32 = m[0..4].parse().map_err(|_| ApiError::Validation {
        field: "month".into(), reason: "invalid year".into(),
    })?;
    let mo: i32 = m[5..7].parse().map_err(|_| ApiError::Validation {
        field: "month".into(), reason: "invalid month".into(),
    })?;
    let total = y * 12 + (mo - 1) + delta;
    let ny = total.div_euclid(12);
    let nmo = total.rem_euclid(12) + 1;
    Ok(format!("{:04}-{:02}", ny, nmo))
}

/// Upsert a single bill_overrides row. Only sets fields the caller provided
/// (NULL means inherit from template at read time).
async fn upsert_override(
    conn: &mut sqlx::SqliteConnection,
    bill_id: &str,
    month: &str,
    req: &UpdateBill,
    now: &str,
) -> Result<(), sqlx::Error> {
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM bill_overrides WHERE bill_id = ? AND month = ?",
    )
    .bind(bill_id)
    .bind(month)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some((existing_id,)) = existing {
        sqlx::query(
            "UPDATE bill_overrides SET \
                description = COALESCE(?, description), \
                value       = COALESCE(?, value), \
                due_day     = COALESCE(?, due_day), \
                account_id  = COALESCE(?, account_id), \
                category_id = COALESCE(?, category_id), \
                updated_at  = ? WHERE id = ?",
        )
        .bind(req.description.as_deref())
        .bind(req.value)
        .bind(req.due_day)
        .bind(req.account_id.as_deref())
        .bind(req.category_id.as_deref())
        .bind(now)
        .bind(existing_id)
        .execute(&mut *conn)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO bill_overrides \
                (id, bill_id, month, description, value, due_day, account_id, category_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id().to_string())
        .bind(bill_id)
        .bind(month)
        .bind(req.description.as_deref())
        .bind(req.value)
        .bind(req.due_day)
        .bind(req.account_id.as_deref())
        .bind(req.category_id.as_deref())
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
