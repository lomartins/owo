use crate::domain::transaction::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::pagination::{filters_hash, Cursor, Order};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const TX_SELECT_COLS: &str = "id, source_account_id, destination_account_id, category_id, payment_method, \
                              value, currency, fx_rate, description, tx_date, \
                              CAST(paid AS INTEGER) != 0 AS paid, \
                              receipt_url, picture_url, card_id, bill_id, invoice_id, \
                              installment_group_id, installment_number, installment_count, \
                              created_at, updated_at, \
                              (CASE \
                                  WHEN (SELECT type FROM accounts WHERE id = source_account_id) = 'equity' \
                                       OR (SELECT type FROM accounts WHERE id = destination_account_id) = 'equity' THEN 'opening' \
                                  WHEN (SELECT type FROM accounts WHERE id = source_account_id) = 'revenue' \
                                       AND (SELECT type FROM accounts WHERE id = destination_account_id) IN ('asset','credit_card') THEN 'deposit' \
                                  WHEN (SELECT type FROM accounts WHERE id = source_account_id) IN ('asset','credit_card') \
                                       AND (SELECT type FROM accounts WHERE id = destination_account_id) = 'expense' THEN 'withdrawal' \
                                  ELSE 'transfer' \
                              END) AS kind";

#[derive(Deserialize)]
pub struct ListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub order: Option<String>,
    pub month: Option<String>,
    pub category_id: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct PageMeta {
    pub limit: u32,
    pub order: String,
    pub returned: u32,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TransactionPage {
    pub items: Vec<Transaction>,
    pub page: PageMeta,
}

fn month_bounds(month: &str) -> ApiResult<(String, String)> {
    if month.len() != 7 || month.as_bytes()[4] != b'-' {
        return Err(ApiError::Validation {
            field: "month".into(),
            reason: "expected YYYY-MM".into(),
        });
    }
    let year: i32 = month[0..4].parse().map_err(|_| ApiError::Validation {
        field: "month".into(),
        reason: "invalid year".into(),
    })?;
    let mo: u32 = month[5..7].parse().map_err(|_| ApiError::Validation {
        field: "month".into(),
        reason: "invalid month".into(),
    })?;
    if !(1..=12).contains(&mo) {
        return Err(ApiError::Validation {
            field: "month".into(),
            reason: "month must be 01..12".into(),
        });
    }
    let lo = format!("{:04}-{:02}-01", year, mo);
    let (ny, nm) = if mo == 12 { (year + 1, 1) } else { (year, mo + 1) };
    let hi = format!("{:04}-{:02}-01", ny, nm);
    Ok((lo, hi))
}

#[utoipa::path(
    get, path = "/api/v1/transactions", tag = "transactions",
    security(("bearer" = [])),
    responses((status = 200, body = TransactionPage))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<TransactionPage>> {
    let limit = q.limit.unwrap_or(50).min(200);
    let order = match q.order.as_deref() {
        Some("asc") => Order::Asc,
        _ => Order::Desc,
    };
    let fhash = filters_hash(&[
        ("user", &cu.id.to_string()),
        ("month", q.month.as_deref().unwrap_or("")),
        ("category_id", q.category_id.as_deref().unwrap_or("")),
        ("account_id", q.account_id.as_deref().unwrap_or("")),
    ]);

    let cursor = match q.cursor.as_deref() {
        Some(c) => Some(Cursor::decode(c, &fhash, &order)?),
        None => None,
    };

    let (op, dir) = match order {
        Order::Desc => ("<", "DESC"),
        Order::Asc => (">", "ASC"),
    };

    let month_range = match q.month.as_deref() {
        Some(m) => Some(month_bounds(m)?),
        None => None,
    };

    let mut sql = format!(
        "SELECT {cols} FROM transactions WHERE user_id = ? AND deleted_at IS NULL",
        cols = TX_SELECT_COLS
    );
    if month_range.is_some() {
        sql.push_str(" AND tx_date >= ? AND tx_date < ?");
    }
    if q.category_id.is_some() {
        sql.push_str(" AND category_id = ?");
    }
    if q.account_id.is_some() {
        sql.push_str(" AND (source_account_id = ? OR destination_account_id = ?)");
    }
    if cursor.is_some() {
        sql.push_str(&format!(" AND (tx_date {op} ? OR (tx_date = ? AND id {op} ?))"));
    }
    sql.push_str(&format!(" ORDER BY tx_date {dir}, id {dir} LIMIT ?"));

    let take = limit + 1;
    let mut query = sqlx::query_as::<_, Transaction>(&sql).bind(cu.id.to_string());
    if let Some((lo, hi)) = &month_range {
        query = query.bind(lo).bind(hi);
    }
    if let Some(cid) = &q.category_id {
        query = query.bind(cid);
    }
    if let Some(aid) = &q.account_id {
        query = query.bind(aid).bind(aid);
    }
    if let Some(c) = &cursor {
        query = query.bind(&c.tx_date).bind(&c.tx_date).bind(&c.id);
    }
    query = query.bind(take as i64);
    let mut items: Vec<Transaction> = query.fetch_all(&state.pool).await?;

    let has_more = items.len() as u32 > limit;
    if has_more {
        items.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        items.last().map(|t| {
            Cursor {
                tx_date: t.tx_date.clone(),
                id: t.id.to_string(),
                order: order.clone(),
                filters_hash: fhash.clone(),
            }
            .encode()
        })
    } else {
        None
    };

    Ok(Json(TransactionPage {
        page: PageMeta {
            limit,
            order: match order {
                Order::Desc => "desc".into(),
                Order::Asc => "asc".into(),
            },
            returned: items.len() as u32,
            has_more,
            next_cursor,
        },
        items,
    }))
}

/// Look up the (source_type, destination_type) pair to validate the derived transaction type.
async fn leg_types(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    src: &str,
    dst: &str,
) -> ApiResult<(String, String)> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT \
            (SELECT type FROM accounts WHERE id = ?1 AND user_id = ?3 AND deleted_at IS NULL), \
            (SELECT type FROM accounts WHERE id = ?2 AND user_id = ?3 AND deleted_at IS NULL)",
    )
    .bind(src)
    .bind(dst)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (s, d) = row.ok_or(ApiError::NotFound)?;
    if s.is_empty() || d.is_empty() {
        return Err(ApiError::NotFound);
    }
    Ok((s, d))
}

fn validate_leg_pair(src: &str, dst: &str, category_id: Option<&str>) -> ApiResult<()> {
    let derived = match (src, dst) {
        ("equity", "asset" | "credit_card" | "liability")
        | ("asset" | "credit_card" | "liability", "equity") => "opening",
        ("revenue", "asset" | "credit_card") => "deposit",
        ("asset" | "credit_card", "expense") => "withdrawal",
        (s, d)
            if matches!(s, "asset" | "credit_card" | "liability")
                && matches!(d, "asset" | "credit_card" | "liability") =>
        {
            "transfer"
        }
        _ => {
            return Err(ApiError::Validation {
                field: "source_account_id+destination_account_id".into(),
                reason: format!("INVALID_LEG_PAIR: {src} -> {dst}"),
            });
        }
    };
    let has_cat = category_id.map(|s| !s.is_empty()).unwrap_or(false);
    if (derived == "transfer" || derived == "opening") && has_cat {
        return Err(ApiError::Validation {
            field: "category_id".into(),
            reason: "transfers and opening balances must not carry a category".into(),
        });
    }
    if (derived == "withdrawal" || derived == "deposit") && !has_cat {
        return Err(ApiError::Validation {
            field: "category_id".into(),
            reason: format!("{derived} requires a category"),
        });
    }
    Ok(())
}

#[utoipa::path(
    post, path = "/api/v1/transactions", tag = "transactions",
    security(("bearer" = [])),
    request_body = CreateTransaction,
    responses((status = 201, body = Transaction))
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateTransaction>,
) -> ApiResult<(StatusCode, Json<Transaction>)> {
    if req.value <= 0 {
        return Err(ApiError::Validation {
            field: "value".into(),
            reason: "must be > 0".into(),
        });
    }

    let (source_id, destination_id) =
        resolve_legs(&state.pool, &cu.id.to_string(), &req).await?;

    if source_id == destination_id {
        return Err(ApiError::Validation {
            field: "destination_account_id".into(),
            reason: "must differ from source_account_id".into(),
        });
    }

    let (s_type, d_type) =
        leg_types(&state.pool, &cu.id.to_string(), &source_id, &destination_id).await?;
    validate_leg_pair(&s_type, &d_type, req.category_id.as_deref())?;

    let now = now_iso();
    let paid = req.paid.unwrap_or(true);
    let n = req.installments.unwrap_or(1).max(1);
    let group_id: Option<String> = if n > 1 { Some(new_id().to_string()) } else { None };
    let total = req.value;
    let base = total / n as i64;
    let remainder = total - base * n as i64; // goes on the first installment

    let mut tx = state.pool.begin().await?;
    let mut first_id = String::new();
    for k in 0..n {
        let id = new_id().to_string();
        if k == 0 {
            first_id = id.clone();
        }
        let value = base + if k == 0 { remainder } else { 0 };
        let date = if n > 1 { add_months_str(&req.tx_date, k as i32)? } else { req.tx_date.clone() };
        let desc = if n > 1 {
            format!("{} ({}/{})", req.description, k + 1, n)
        } else {
            req.description.clone()
        };
        let number: Option<i64> = if n > 1 { Some((k + 1) as i64) } else { None };
        let count: Option<i64> = if n > 1 { Some(n as i64) } else { None };
        sqlx::query(
            "INSERT INTO transactions \
                (id, user_id, source_account_id, destination_account_id, category_id, \
                 payment_method, value, currency, fx_rate, description, tx_date, paid, \
                 card_id, bill_id, invoice_id, installment_group_id, installment_number, \
                 installment_count, created_at, updated_at, device_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(cu.id.to_string())
        .bind(&source_id)
        .bind(&destination_id)
        .bind(req.category_id.as_deref())
        .bind(&req.payment_method)
        .bind(value)
        .bind(&req.currency)
        .bind(req.fx_rate.as_deref())
        .bind(&desc)
        .bind(&date)
        .bind(paid as i64)
        .bind(req.card_id.as_deref())
        .bind(req.bill_id.as_deref())
        .bind(req.invoice_id.as_deref())
        .bind(group_id.as_deref())
        .bind(number)
        .bind(count)
        .bind(&now)
        .bind(&now)
        .bind(&cu.device_id)
        .execute(&mut *tx)
        .await?;
        crate::audit::write(&mut *tx, cu.id, "Transaction", &id, "CREATE", None, Some(&cu.device_id)).await?;
    }

    if let Some(tag_ids) = &req.tag_ids {
        for tid in tag_ids {
            sqlx::query("INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?, ?)")
                .bind(&first_id)
                .bind(tid.to_string())
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;

    let row: Transaction = sqlx::query_as(&format!(
        "SELECT {cols} FROM transactions WHERE id = ?",
        cols = TX_SELECT_COLS
    ))
    .bind(&first_id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    post, path = "/api/v1/transactions/transfer", tag = "transactions",
    security(("bearer" = [])),
    request_body = CreateTransfer,
    responses((status = 201, body = Transaction))
)]
pub async fn transfer(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateTransfer>,
) -> ApiResult<(StatusCode, Json<Transaction>)> {
    let src = req.source_account_id.to_string();
    let dst = req.destination_account_id.to_string();
    if src == dst {
        return Err(ApiError::Validation {
            field: "destination_account_id".into(),
            reason: "must differ from source_account_id".into(),
        });
    }
    if req.value <= 0 {
        return Err(ApiError::Validation {
            field: "value".into(),
            reason: "must be > 0".into(),
        });
    }
    let (s_type, d_type) = leg_types(&state.pool, &cu.id.to_string(), &src, &dst).await?;
    // Transfers must be between user-owned non-bucket accounts.
    let is_user_account = |t: &str| matches!(t, "asset" | "credit_card" | "liability");
    if !is_user_account(&s_type) || !is_user_account(&d_type) {
        return Err(ApiError::Validation {
            field: "source_account_id+destination_account_id".into(),
            reason: "transfer legs must both be user-owned accounts (asset/credit_card/liability)".into(),
        });
    }

    let id = new_id();
    let now = now_iso();
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO transactions \
            (id, user_id, source_account_id, destination_account_id, category_id, \
             payment_method, value, currency, fx_rate, description, tx_date, paid, \
             created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .bind(&src)
    .bind(&dst)
    .bind(&req.payment_method)
    .bind(req.value)
    .bind(&req.currency)
    .bind(req.fx_rate.as_deref())
    .bind(&req.description)
    .bind(&req.tx_date)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await?;
    crate::audit::write(&mut *tx, cu.id, "Transaction", &id.to_string(), "CREATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Transaction = sqlx::query_as(&format!(
        "SELECT {cols} FROM transactions WHERE id = ?",
        cols = TX_SELECT_COLS
    ))
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// Resolve missing source/destination from `category_id` using the user's revenue/expense
/// buckets. See [`CreateTransaction`] for the rule table.
async fn resolve_legs(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    req: &CreateTransaction,
) -> ApiResult<(String, String)> {
    let src = req.source_account_id.clone();
    let dst = req.destination_account_id.clone();
    match (src, dst) {
        (Some(s), Some(d)) => Ok((s, d)),
        (Some(s), None) => {
            if req.category_id.is_none() {
                return Err(ApiError::Validation {
                    field: "category_id".into(),
                    reason: "required when destination_account_id is omitted".into(),
                });
            }
            let bucket = bucket_id(pool, user_id, "expense").await?;
            Ok((s, bucket))
        }
        (None, Some(d)) => {
            if req.category_id.is_none() {
                return Err(ApiError::Validation {
                    field: "category_id".into(),
                    reason: "required when source_account_id is omitted".into(),
                });
            }
            let bucket = bucket_id(pool, user_id, "revenue").await?;
            Ok((bucket, d))
        }
        (None, None) => Err(ApiError::Validation {
            field: "source_account_id+destination_account_id".into(),
            reason: "at least one leg must be provided".into(),
        }),
    }
}

async fn bucket_id(pool: &sqlx::SqlitePool, user_id: &str, bucket_type: &str) -> ApiResult<String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM accounts WHERE user_id = ? AND type = ? AND deleted_at IS NULL",
    )
    .bind(user_id)
    .bind(bucket_type)
    .fetch_optional(pool)
    .await?;
    row.map(|(id,)| id).ok_or_else(|| ApiError::Validation {
        field: bucket_type.into(),
        reason: format!("user has no {bucket_type} bucket account; re-register or self-heal"),
    })
}

#[utoipa::path(
    patch, path = "/api/v1/transactions/{id}", tag = "transactions",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    request_body = UpdateTransaction,
    responses((status = 200, body = Transaction))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTransaction>,
) -> ApiResult<Json<Transaction>> {
    if let Some(v) = req.value {
        if v <= 0 {
            return Err(ApiError::Validation {
                field: "value".into(),
                reason: "must be > 0".into(),
            });
        }
    }
    let now = now_iso();
    let mut tx = state.pool.begin().await?;
    let existing: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT source_account_id, destination_account_id, category_id \
         FROM transactions WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    let Some((cur_src, cur_dst, cur_cat)) = existing else {
        return Err(ApiError::NotFound);
    };
    let eff_src = req.source_account_id.clone().unwrap_or(cur_src);
    let eff_dst = req.destination_account_id.clone().unwrap_or(cur_dst);
    if eff_src == eff_dst {
        return Err(ApiError::Validation {
            field: "destination_account_id".into(),
            reason: "must differ from source_account_id".into(),
        });
    }
    let eff_cat = match req.category_id.as_ref() {
        Some(c) if c.is_empty() => None,
        Some(c) => Some(c.clone()),
        None => cur_cat,
    };
    let (s_type, d_type) = leg_types(&state.pool, &cu.id.to_string(), &eff_src, &eff_dst).await?;
    validate_leg_pair(&s_type, &d_type, eff_cat.as_deref())?;

    sqlx::query(
        "UPDATE transactions SET \
            source_account_id      = COALESCE(?, source_account_id), \
            destination_account_id = COALESCE(?, destination_account_id), \
            category_id            = COALESCE(?, category_id), \
            payment_method         = COALESCE(?, payment_method), \
            value                  = COALESCE(?, value), \
            description            = COALESCE(?, description), \
            tx_date                = COALESCE(?, tx_date), \
            paid                   = COALESCE(?, paid), \
            fx_rate                = COALESCE(?, fx_rate), \
            card_id                = COALESCE(?, card_id), \
            bill_id                = COALESCE(?, bill_id), \
            invoice_id             = COALESCE(?, invoice_id), \
            updated_at             = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(req.source_account_id.as_deref())
    .bind(req.destination_account_id.as_deref())
    .bind(req.category_id.as_deref())
    .bind(req.payment_method.as_deref())
    .bind(req.value)
    .bind(req.description.as_deref())
    .bind(req.tx_date.as_deref())
    .bind(req.paid.map(|b| if b { 1_i64 } else { 0_i64 }))
    .bind(req.fx_rate.as_deref())
    .bind(req.card_id.as_deref())
    .bind(req.bill_id.as_deref())
    .bind(req.invoice_id.as_deref())
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&mut *tx)
    .await?;

    crate::audit::write(&mut *tx, cu.id, "Transaction", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Transaction = sqlx::query_as(&format!(
        "SELECT {cols} FROM transactions WHERE id = ?",
        cols = TX_SELECT_COLS
    ))
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
pub struct DeleteQuery {
    /// For installment transactions: "this" (default) deletes only this row,
    /// "following" deletes this row and every later installment in the group.
    pub scope: Option<String>,
}

#[utoipa::path(
    delete, path = "/api/v1/transactions/{id}", tag = "transactions",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,), ("scope" = Option<String>, Query, description = "this | following")),
    responses((status = 204))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(q): Query<DeleteQuery>,
) -> ApiResult<StatusCode> {
    let now = now_iso();
    let mut tx = state.pool.begin().await?;

    let following = q.scope.as_deref() == Some("following");
    if following {
        // Resolve this row's installment group + position, then soft-delete it and
        // every later installment in the same group.
        let info: Option<(Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT installment_group_id, installment_number FROM transactions \
             WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
        )
        .bind(id.to_string())
        .bind(cu.id.to_string())
        .fetch_optional(&mut *tx)
        .await?;
        let Some((group, number)) = info else {
            return Err(ApiError::NotFound);
        };
        if let (Some(group), Some(number)) = (group, number) {
            sqlx::query(
                "UPDATE transactions SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
                 WHERE user_id = ? AND deleted_at IS NULL \
                   AND installment_group_id = ? AND installment_number >= ?",
            )
            .bind(&now)
            .bind(&now)
            .bind(cu.id.to_string())
            .bind(&group)
            .bind(number)
            .execute(&mut *tx)
            .await?;
            crate::audit::write(&mut *tx, cu.id, "Transaction", &id.to_string(), "DELETE", None, Some(&cu.device_id)).await?;
            tx.commit().await?;
            return Ok(StatusCode::NO_CONTENT);
        }
        // Not an installment row → fall through to single-row delete.
    }

    let res = sqlx::query(
        "UPDATE transactions SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
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
    crate::audit::write(&mut *tx, cu.id, "Transaction", &id.to_string(), "DELETE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add `n` months to a "YYYY-MM-DD" date string, clamping the day to month length.
fn add_months_str(date: &str, n: i32) -> ApiResult<String> {
    let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| ApiError::Validation {
        field: "tx_date".into(),
        reason: "expected YYYY-MM-DD".into(),
    })?;
    use chrono::Datelike;
    let total = d.year() * 12 + d.month0() as i32 + n;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) as u32 + 1;
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let dim = chrono::NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day();
    let day = d.day().min(dim);
    Ok(chrono::NaiveDate::from_ymd_opt(y, m, day).unwrap().to_string())
}
