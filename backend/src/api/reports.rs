use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize)]
pub struct MonthQuery {
    pub month: String,
}

#[derive(Serialize, ToSchema)]
pub struct MonthlyReport {
    pub month: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub income_total: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub spent_total: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub budget_balance: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub carry_over_in: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub carry_over_out: i64,
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
    get, path = "/api/v1/reports/monthly", tag = "reports",
    security(("bearer" = [])),
    params(("month" = String, Query, description = "YYYY-MM")),
    responses((status = 200, body = MonthlyReport))
)]
pub async fn monthly(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Query(q): Query<MonthQuery>,
) -> ApiResult<Json<MonthlyReport>> {
    let (lo, hi) = month_bounds(&q.month)?;

    // Income: source.type = 'revenue', within the month.
    let (income_cents,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(t.value), 0) AS v \
         FROM transactions t \
         JOIN accounts s ON s.id = t.source_account_id \
         WHERE t.user_id = ? AND t.deleted_at IS NULL \
           AND t.tx_date >= ? AND t.tx_date < ? \
           AND s.type = 'revenue'",
    )
    .bind(cu.id.to_string())
    .bind(&lo)
    .bind(&hi)
    .fetch_one(&state.pool)
    .await?;

    // Spent: destination.type = 'expense', within the month.
    let (spent_cents,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(t.value), 0) AS v \
         FROM transactions t \
         JOIN accounts d ON d.id = t.destination_account_id \
         WHERE t.user_id = ? AND t.deleted_at IS NULL \
           AND t.tx_date >= ? AND t.tx_date < ? \
           AND d.type = 'expense'",
    )
    .bind(cu.id.to_string())
    .bind(&lo)
    .bind(&hi)
    .fetch_one(&state.pool)
    .await?;

    // Sum of all estimated budgets for this month.
    let (budget_total_cents,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(estimated_amount), 0) FROM budgets \
         WHERE user_id = ? AND month = ? AND deleted_at IS NULL",
    )
    .bind(cu.id.to_string())
    .bind(&q.month)
    .fetch_one(&state.pool)
    .await?;

    // Carry-over: sum of asset balances at month boundaries.
    let carry_in_cents: i64 = asset_balances_at(&state.pool, &cu.id.to_string(), &lo).await?;
    let carry_out_cents: i64 = asset_balances_at(&state.pool, &cu.id.to_string(), &hi).await?;

    Ok(Json(MonthlyReport {
        month: q.month,
        income_total: income_cents,
        spent_total: spent_cents,
        budget_balance: budget_total_cents - spent_cents,
        carry_over_in: carry_in_cents,
        carry_over_out: carry_out_cents,
    }))
}

async fn asset_balances_at(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    before: &str,
) -> ApiResult<i64> {
    balances_for_types_at(pool, user_id, &["asset"], before).await
}

/// Sum balances at a given date for the given account types.
/// Balance = initial + Σ(incoming where tx_date < before) - Σ(outgoing where tx_date < before).
async fn balances_for_types_at(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    types: &[&str],
    before: &str,
) -> ApiResult<i64> {
    if types.is_empty() {
        return Ok(0);
    }
    // Build dynamic IN (...) with placeholders.
    let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT COALESCE(SUM( \
              a.initial_balance \
            + COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = a.id AND deleted_at IS NULL AND tx_date < ?), 0) \
            - COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id      = a.id AND deleted_at IS NULL AND tx_date < ?), 0) \
         ), 0) \
         FROM accounts a \
         WHERE a.user_id = ? AND a.archived = 0 AND a.deleted_at IS NULL AND a.type IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, (i64,)>(&sql)
        .bind(before)
        .bind(before)
        .bind(user_id);
    for t in types {
        q = q.bind(*t);
    }
    let (total,) = q.fetch_one(pool).await?;
    Ok(total)
}

#[utoipa::path(get, path = "/api/v1/reports/cash-flow", tag = "reports", security(("bearer" = [])), responses((status = 501)))]
pub async fn cash_flow(_state: State<AppState>, _cu: Extension<CurrentUser>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[utoipa::path(get, path = "/api/v1/reports/by-category", tag = "reports", security(("bearer" = [])), responses((status = 501)))]
pub async fn by_category(_state: State<AppState>, _cu: Extension<CurrentUser>) -> ApiResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Serialize, ToSchema)]
pub struct NetWorthPoint {
    pub month: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub net_worth: i64,
}

#[derive(Serialize, ToSchema)]
pub struct NetWorthReport {
    pub points: Vec<NetWorthPoint>,
    /// Current period net worth (last entry in `points`).
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub current: i64,
    /// Percentage change vs previous month, as a Decimal string (e.g. "4.20" for +4.2%).
    /// `null` when there is no prior month or prior was zero.
    pub change_pct: Option<String>,
}

#[derive(Deserialize)]
pub struct NetWorthQuery {
    pub to: Option<String>,
    pub months: Option<u32>,
}

fn shift_month(m: &str, delta: i32) -> String {
    let y: i32 = m[0..4].parse().unwrap();
    let mo: i32 = m[5..7].parse().unwrap();
    let total = y * 12 + (mo - 1) + delta;
    let ny = total.div_euclid(12);
    let nmo = total.rem_euclid(12) + 1;
    format!("{:04}-{:02}", ny, nmo)
}

#[utoipa::path(
    get, path = "/api/v1/reports/net-worth", tag = "reports",
    security(("bearer" = [])),
    params(
        ("to" = Option<String>, Query, description = "Last month to include (YYYY-MM). Defaults to current."),
        ("months" = Option<u32>, Query, description = "Window size (default 7, max 24)."),
    ),
    responses((status = 200, body = NetWorthReport))
)]
pub async fn net_worth(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Query(q): Query<NetWorthQuery>,
) -> ApiResult<Json<NetWorthReport>> {
    let to = q.to.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m").to_string()
    });
    let months = q.months.unwrap_or(7).clamp(1, 24);
    if to.len() != 7 || to.as_bytes()[4] != b'-' {
        return Err(ApiError::Validation {
            field: "to".into(),
            reason: "expected YYYY-MM".into(),
        });
    }

    // Build month list ascending (oldest first → newest last).
    let mut months_list: Vec<String> = (0..months as i32)
        .rev()
        .map(|i| shift_month(&to, -i))
        .collect();
    months_list.dedup();

    let mut points: Vec<NetWorthPoint> = Vec::with_capacity(months_list.len());
    for m in &months_list {
        let next = shift_month(m, 1);
        let lo_next = format!("{}-01", next); // exclusive boundary = start of next month
        let assets = balances_for_types_at(&state.pool, &cu.id.to_string(), &["asset"], &lo_next).await?;
        let liab = balances_for_types_at(&state.pool, &cu.id.to_string(), &["credit_card", "liability"], &lo_next).await?;
        let nw = assets - liab;
        points.push(NetWorthPoint { month: m.clone(), net_worth: nw });
    }

    let current = points.last().map(|p| p.net_worth).unwrap_or(0);
    let change_pct = points.iter().rev().nth(1).and_then(|prev| {
        if prev.net_worth == 0 {
            None
        } else {
            // % change as basis-points / 100, two decimal precision.
            let diff = current - prev.net_worth;
            // (diff / prev) * 100, computed in i64 → decimal string.
            let bp = (diff as i128 * 10_000) / prev.net_worth as i128; // basis-points * 100
            let int_part = bp / 100;
            let frac_part = (bp % 100).abs();
            let sign = if bp < 0 && int_part == 0 { "-" } else { "" };
            Some(format!("{sign}{int_part}.{:02}", frac_part))
        }
    });

    Ok(Json(NetWorthReport {
        points,
        current,
        change_pct,
    }))
}
