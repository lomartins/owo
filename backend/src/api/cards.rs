use crate::domain::card::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

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
    let items: Vec<Card> = sqlx::query_as(
        "SELECT id, account_id, last_four_digits, brand, type, \"limit\", close_day, due_day, \
                CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM cards WHERE user_id = ? AND deleted_at IS NULL ORDER BY created_at",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(CardList { items }))
}

/// Creating a card also creates the underlying `credit_card`-type account. The
/// card row's `account_id` points to that account. Card purchases source from
/// it; invoice payments transfer asset -> credit_card. See specs/08-cards-preserve.md.
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

    // The user must specify which checking/asset account this card "belongs" to operationally,
    // i.e. which currency to inherit. We pull currency from that account but the card's
    // funding account becomes the new credit_card account.
    let parent: Option<(String,)> = sqlx::query_as(
        "SELECT currency FROM accounts WHERE id = ? AND user_id = ? AND type = 'asset' AND deleted_at IS NULL",
    )
    .bind(&req.account_id)
    .bind(cu.id.to_string())
    .fetch_optional(&state.pool)
    .await?;
    let Some((currency,)) = parent else {
        return Err(ApiError::Validation {
            field: "account_id".into(),
            reason: "must reference an asset account that the card draws funding from".into(),
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
        "INSERT INTO cards (id, user_id, account_id, last_four_digits, brand, type, \"limit\", \
            close_day, due_day, archived, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)",
    )
    .bind(&card_id)
    .bind(cu.id.to_string())
    .bind(&credit_account_id)
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

    let row: Card = sqlx::query_as(
        "SELECT id, account_id, last_four_digits, brand, type, \"limit\", close_day, due_day, \
                CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM cards WHERE id = ?",
    )
    .bind(&card_id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}
