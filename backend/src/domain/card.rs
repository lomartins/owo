use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Card {
    pub id: String,
    pub account_id: String,
    pub last_four_digits: String,
    pub brand: String,
    pub r#type: String,
    #[serde(with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "5000.00")]
    pub limit: Option<i64>,
    pub close_day: Option<i64>,
    pub due_day: Option<i64>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateCard {
    pub account_id: String,
    pub last_four_digits: String,
    pub brand: String,
    pub r#type: String,
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "5000.00")]
    pub limit: Option<i64>,
    pub close_day: Option<i64>,
    pub due_day: Option<i64>,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct CardInvoice {
    pub id: String,
    pub card_id: String,
    pub period_start: String,
    pub period_end: String,
    pub due_date: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub total: i64,
    pub paid_at: Option<String>,
    pub paid_transaction_id: Option<String>,
}
