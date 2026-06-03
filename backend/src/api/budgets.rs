use crate::domain::budget::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct MonthQuery {
    pub month: String,
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
    get, path = "/api/v1/budgets", tag = "budgets",
    security(("bearer" = [])),
    params(("month" = String, Query, description = "YYYY-MM")),
    responses((status = 200, body = BudgetMonth))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Query(q): Query<MonthQuery>,
) -> ApiResult<Json<BudgetMonth>> {
    let (lo, hi) = month_bounds(&q.month)?;

    let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, i64, i64)> = sqlx::query_as(
        "SELECT \
            c.id              AS category_id, \
            c.name            AS category_name, \
            c.icon            AS icon, \
            c.color           AS color, \
            b.id              AS budget_id, \
            COALESCE(b.estimated_amount, 0) AS estimated_cents, \
            COALESCE(( \
                SELECT SUM(t.value) \
                FROM transactions t \
                JOIN accounts dst ON dst.id = t.destination_account_id \
                JOIN accounts src ON src.id = t.source_account_id \
                WHERE t.user_id = ? \
                  AND t.deleted_at IS NULL \
                  AND t.tx_date >= ? AND t.tx_date < ? \
                  AND t.category_id = c.id \
                  AND dst.type = 'expense' \
                  AND src.type IN ('asset','credit_card') \
            ), 0) AS spent_cents \
         FROM categories c \
         LEFT JOIN budgets b \
                ON b.category_id = c.id \
               AND b.user_id     = ? \
               AND b.month       = ? \
               AND b.deleted_at IS NULL \
         WHERE c.user_id = ? \
           AND c.archived = 0 \
           AND c.deleted_at IS NULL \
           AND c.kind IN ('EXPENSE','BOTH') \
         ORDER BY c.name",
    )
    .bind(cu.id.to_string())
    .bind(&lo)
    .bind(&hi)
    .bind(cu.id.to_string())
    .bind(&q.month)
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(category_id, category_name, icon, color, budget_id, estimated, spent)| BudgetRow {
            category_id,
            category_name,
            icon,
            color,
            budget_id,
            estimated,
            spent,
            difference: estimated - spent,
        })
        .collect();

    Ok(Json(BudgetMonth {
        month: q.month,
        items,
    }))
}

#[utoipa::path(
    post, path = "/api/v1/budgets", tag = "budgets",
    security(("bearer" = [])),
    request_body = CreateBudget,
    responses((status = 201, body = Budget))
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateBudget>,
) -> ApiResult<(StatusCode, Json<Budget>)> {
    let _ = month_bounds(&req.month)?;

    let now = now_iso();
    let mut tx = state.pool.begin().await?;

    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM budgets WHERE user_id = ? AND category_id = ? AND month = ? AND deleted_at IS NULL",
    )
    .bind(cu.id.to_string())
    .bind(&req.category_id)
    .bind(&req.month)
    .fetch_optional(&mut *tx)
    .await?;

    let id = if let Some((existing_id,)) = existing {
        sqlx::query(
            "UPDATE budgets SET estimated_amount = ?, currency = ?, updated_at = ?, sync_version = sync_version + 1 \
             WHERE id = ?",
        )
        .bind(req.estimated_amount)
        .bind(&req.currency)
        .bind(&now)
        .bind(&existing_id)
        .execute(&mut *tx)
        .await?;
        existing_id
    } else {
        let id = new_id().to_string();
        sqlx::query(
            "INSERT INTO budgets (id, user_id, category_id, month, estimated_amount, currency, \
                created_at, updated_at, device_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(cu.id.to_string())
        .bind(&req.category_id)
        .bind(&req.month)
        .bind(req.estimated_amount)
        .bind(&req.currency)
        .bind(&now)
        .bind(&now)
        .bind(&cu.device_id)
        .execute(&mut *tx)
        .await?;
        id
    };

    crate::audit::write(&mut *tx, cu.id, "Budget", &id, "CREATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Budget = sqlx::query_as(
        "SELECT id, category_id, month, estimated_amount, currency, created_at, updated_at \
         FROM budgets WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    patch, path = "/api/v1/budgets/{id}", tag = "budgets",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    request_body = UpdateBudget,
    responses((status = 200, body = Budget))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBudget>,
) -> ApiResult<Json<Budget>> {
    let now = now_iso();
    let mut tx = state.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE budgets SET estimated_amount = ?, updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(req.estimated_amount)
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    crate::audit::write(&mut *tx, cu.id, "Budget", &id.to_string(), "UPDATE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;

    let row: Budget = sqlx::query_as(
        "SELECT id, category_id, month, estimated_amount, currency, created_at, updated_at \
         FROM budgets WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

#[utoipa::path(
    delete, path = "/api/v1/budgets/{id}", tag = "budgets",
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
    let res = sqlx::query(
        "UPDATE budgets SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
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
    crate::audit::write(&mut *tx, cu.id, "Budget", &id.to_string(), "DELETE", None, Some(&cu.device_id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
