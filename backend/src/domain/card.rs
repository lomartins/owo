use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Card {
    pub id: String,
    /// The card's own credit_card-type ledger account (holds the debt).
    pub account_id: String,
    /// The asset account this card is attached to / paid from.
    pub payment_account_id: Option<String>,
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
    /// The asset account this credit card is attached to (funds the invoice).
    pub payment_account_id: String,
    pub last_four_digits: String,
    pub brand: String,
    pub r#type: String,
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "5000.00")]
    pub limit: Option<i64>,
    pub close_day: Option<i64>,
    pub due_day: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCard {
    pub payment_account_id: Option<String>,
    pub last_four_digits: Option<String>,
    pub brand: Option<String>,
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "5000.00")]
    pub limit: Option<i64>,
    pub close_day: Option<i64>,
    pub due_day: Option<i64>,
    pub archived: Option<bool>,
}

/// Current open-invoice snapshot for a card: how much is accrued so far in the
/// open billing cycle, the full cycle total (incl. future-dated charges such as
/// later installments within the cycle), the due date, and limit usage.
#[derive(Serialize, ToSchema)]
pub struct InvoicePreview {
    pub card_id: String,
    pub period_start: String,
    pub period_end: String,
    pub due_date: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub accrued: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub cycle_total: i64,
    /// Total outstanding debt on the card across all cycles (charges − payments).
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub outstanding: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "5000.00")]
    pub limit: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct InvoicePreviewList {
    pub items: Vec<InvoicePreview>,
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
