use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A bill is a recurring monthly template. `due_day` is the day-of-month (1-31).
/// `paid` and `paid_at` reflect the payment status for a specific month — see
/// `bill_payments` table. The bill row itself has no persistent paid flag any more.
#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Bill {
    pub id: String,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub description: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
    pub currency: String,
    pub due_day: i64,
    /// Per-month payment flag, computed by the list handler when ?month=… is set.
    pub paid: bool,
    /// ISO timestamp of the per-month payment, or NULL if unpaid this month.
    pub paid_at: Option<String>,
    /// Transaction id that posted the per-month payment, or NULL if unpaid.
    pub paid_transaction_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateBill {
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub description: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
    pub currency: String,
    pub due_day: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct PayBill {
    pub account_id: String,
    pub tx_date: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
}
