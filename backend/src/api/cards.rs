use crate::domain::card::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

const CARD_COLS: &str = "id, account_id, payment_account_id, last_four_digits, brand, type, \
    \"limit\", close_day, due_day, CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at";

#[derive(Serialize, utoipa::ToSchema)]
pub struct CardList {
    pub items: Vec<Card>,
}

#[utoipa::path(get, path = "/api/v1/cards", tag = "cards", security(("bearer" = [])),
    responses((status = 200, body = CardList)))]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<Json<CardList>> {
    let items: Vec<Card> = sqlx::query_as(&format!(
        "SELECT {CARD_COLS} FROM cards WHERE user_id = ? AND deleted_at IS NULL ORDER BY created_at"
    ))
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(CardList { items }))
}

/// Creating a card also creates the underlying `credit_card`-type ledger account
/// that holds the card's debt. `payment_account_id` records which asset account
/// the card is attached to and paid from. Card purchases source from the ledger
/// account; invoice payments transfer asset -> credit_card.
#[utoipa::path(post, path = "/api/v1/cards", tag = "cards", security(("bearer" = [])),
    request_body = CreateCard, responses((status = 201, body = Card)))]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateCard>,
) -> ApiResult<(StatusCode, Json<Card>)> {
    if req.r#type != "CREDIT" {
        return Err(ApiError::Validation {
            field: "type".into(),
            reason: "only CREDIT cards bind to a credit_card account; DEBIT cards are out of MVP".into(),
        });
    }
    let now = now_iso();
    let card_id = new_id().to_string();
    let credit_account_id = new_id().to_string();
    let account_name = format!("{} •••• {}", req.brand, req.last_four_digits);

    // The card is attached to an asset account; inherit its currency.
    let parent: Option<(String,)> = sqlx::query_as(
        "SELECT currency FROM accounts WHERE id = ? AND user_id = ? AND type = 'asset' AND deleted_at IS NULL",
    )
    .bind(&req.payment_account_id)
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((currency,)) = parent else {
        return Err(ApiError::Validation {
            field: "payment_account_id".into(),
            reason: "must reference an asset account the card is attached to".into(),
        });
    };

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO accounts (id, user_id, name, type, currency, initial_balance, archived, \
            created_at, updated_at, device_id) \
         VALUES (?, ?, ?, 'credit_card', ?, 0, 0, ?, ?, ?)",
    )
    .bind(&credit_account_id)
    .bind(cu.id.to_string())
    .bind(&account_name)
    .bind(&currency)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO cards (id, user_id, account_id, payment_account_id, last_four_digits, brand, \
            type, \"limit\", close_day, due_day, archived, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)",
    )
    .bind(&card_id)
    .bind(cu.id.to_string())
    .bind(&credit_account_id)
    .bind(&req.payment_account_id)
    .bind(&req.last_four_digits)
    .bind(&req.brand)
    .bind(&req.r#type)
    .bind(req.limit)
    .bind(req.close_day)
    .bind(req.due_day)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&mut *tx)
    .await?;

    crate::audit::write(&mut *tx, cu.id, "Card", &card_id, "CREATE", None, Some(&cu.device_id)).await?;
    crate::audit::write(&mut *tx, cu.id, "Account", &credit_account_id, "CREATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Card = sqlx::query_as(&format!("SELECT {CARD_COLS} FROM cards WHERE id = ?"))
        .bind(&card_id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(patch, path = "/api/v1/cards/{id}", tag = "cards", security(("bearer" = [])),
    params(("id" = Uuid, Path,)), request_body = UpdateCard,
    responses((status = 200, body = Card)))]
pub async fn update(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCard>,
) -> ApiResult<Json<Card>> {
    let now = now_iso();
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT account_id FROM cards WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((credit_account_id,)) = existing else {
        return Err(ApiError::NotFound);
    };

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE cards SET \
            payment_account_id = COALESCE(?, payment_account_id), \
            last_four_digits   = COALESCE(?, last_four_digits), \
            brand              = COALESCE(?, brand), \
            \"limit\"          = COALESCE(?, \"limit\"), \
            close_day          = COALESCE(?, close_day), \
            due_day            = COALESCE(?, due_day), \
            archived           = COALESCE(?, archived), \
            updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ?",
    )
    .bind(req.payment_account_id.as_deref())
    .bind(req.last_four_digits.as_deref())
    .bind(req.brand.as_deref())
    .bind(req.limit)
    .bind(req.close_day)
    .bind(req.due_day)
    .bind(req.archived.map(|b| if b { 1 } else { 0 }))
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&mut *tx)
    .await?;

    // Keep the ledger account name in sync with brand / last four.
    if req.brand.is_some() || req.last_four_digits.is_some() {
        let cur: (String, String) = sqlx::query_as(
            "SELECT brand, last_four_digits FROM cards WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_one(&mut *tx)
        .await?;
        let name = format!("{} •••• {}", cur.0, cur.1);
        sqlx::query("UPDATE accounts SET name = ?, updated_at = ? WHERE id = ? AND user_id = ?")
            .bind(&name)
            .bind(&now)
            .bind(&credit_account_id)
            .bind(cu.id.to_string())
            .execute(&mut *tx)
            .await?;
    }
    crate::audit::write(&mut *tx, cu.id, "Card", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Card = sqlx::query_as(&format!("SELECT {CARD_COLS} FROM cards WHERE id = ?"))
        .bind(id.to_string())
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(row))
}

#[utoipa::path(delete, path = "/api/v1/cards/{id}", tag = "cards", security(("bearer" = [])),
    params(("id" = Uuid, Path,)), responses((status = 204)))]
pub async fn delete(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let now = now_iso();
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT account_id FROM cards WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((credit_account_id,)) = existing else {
        return Err(ApiError::NotFound);
    };
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE cards SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 WHERE id = ? AND user_id = ?")
        .bind(&now).bind(&now).bind(id.to_string()).bind(cu.id.to_string())
        .execute(&mut *tx).await?;
    sqlx::query("UPDATE accounts SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 WHERE id = ? AND user_id = ?")
        .bind(&now).bind(&now).bind(&credit_account_id).bind(cu.id.to_string())
        .execute(&mut *tx).await?;
    crate::audit::write(&mut *tx, cu.id, "Card", &id.to_string(), "DELETE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/cards/invoice-preview", tag = "cards", security(("bearer" = [])),
    responses((status = 200, body = InvoicePreviewList)))]
pub async fn invoice_preview(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<Json<InvoicePreviewList>> {
    let cards: Vec<(String, String, Option<i64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT id, account_id, close_day, due_day, \"limit\" \
         FROM cards WHERE user_id = ? AND deleted_at IS NULL AND archived = 0",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let today = Utc::now().date_naive();
    let mut items = Vec::with_capacity(cards.len());
    for (card_id, account_id, close_day, due_day, limit) in cards {
        let close = close_day.unwrap_or(1).clamp(1, 31) as u32;
        let due = due_day.unwrap_or(10).clamp(1, 31) as u32;
        let (period_start, period_end, due_date) = billing_cycle(today, close, due);

        // Charges = purchases sourced from the card's ledger account, tagged with card_id.
        let accrued = sum_charges(&state.pool, &cu.id.to_string(), &account_id, &period_start, &min_date(today, period_end)).await?;
        let cycle_total = sum_charges(&state.pool, &cu.id.to_string(), &account_id, &period_start, &period_end).await?;
        let outstanding = card_outstanding(&state.pool, &cu.id.to_string(), &account_id).await?;

        items.push(InvoicePreview {
            card_id,
            period_start: period_start.to_string(),
            period_end: period_end.to_string(),
            due_date: due_date.to_string(),
            accrued,
            cycle_total,
            outstanding,
            limit,
        });
    }
    Ok(Json(InvoicePreviewList { items }))
}

/// Sum card charges (ledger-account-sourced purchases) within [start, end] inclusive.
async fn sum_charges(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    credit_account_id: &str,
    start: &NaiveDate,
    end: &NaiveDate,
) -> ApiResult<i64> {
    if end < start {
        return Ok(0);
    }
    let (v,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(value), 0) FROM transactions \
         WHERE user_id = ? AND deleted_at IS NULL AND source_account_id = ? \
           AND tx_date >= ? AND tx_date <= ?",
    )
    .bind(user_id)
    .bind(credit_account_id)
    .bind(start.to_string())
    .bind(end.to_string())
    .fetch_one(pool)
    .await?;
    Ok(v)
}

/// Outstanding debt = charges (out of card) − payments (into card).
async fn card_outstanding(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    credit_account_id: &str,
) -> ApiResult<i64> {
    let (v,): (i64,) = sqlx::query_as(
        "SELECT COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id = ? AND deleted_at IS NULL), 0) \
              - COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = ? AND deleted_at IS NULL), 0)",
    )
    .bind(credit_account_id)
    .bind(credit_account_id)
    .fetch_one(pool)
    .await?;
    let _ = user_id;
    Ok(v)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let first_next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
    first_next.pred_opt().unwrap().day()
}

fn ymd_clamp(year: i32, month: u32, day: u32) -> NaiveDate {
    let d = day.min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, d).unwrap()
}

fn add_months(d: NaiveDate, n: i32) -> NaiveDate {
    let total = d.year() * 12 + (d.month0() as i32) + n;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) as u32 + 1;
    ymd_clamp(y, m, d.day())
}

fn min_date(a: NaiveDate, b: NaiveDate) -> NaiveDate {
    if a < b { a } else { b }
}

/// Current open billing cycle for the given close/due days, relative to `today`.
fn billing_cycle(today: NaiveDate, close_day: u32, due_day: u32) -> (NaiveDate, NaiveDate, NaiveDate) {
    let close_this = ymd_clamp(today.year(), today.month(), close_day);
    let period_end = if today <= close_this {
        close_this
    } else {
        add_months(close_this, 1)
    };
    let prev_close = add_months(period_end, -1);
    let period_start = prev_close.succ_opt().unwrap();
    let due_month = add_months(period_end, 1);
    let due_date = ymd_clamp(due_month.year(), due_month.month(), due_day);
    (period_start, period_end, due_date)
}
