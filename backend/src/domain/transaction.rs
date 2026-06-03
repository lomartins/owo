use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A transaction is one row with two account legs. Direction is encoded in the pair
/// of accounts, never in the sign of `value`. The derived type (deposit / withdrawal /
/// transfer) is computed from `source.type` × `destination.type` at read time.
#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub source_account_id: String,
    pub destination_account_id: String,
    pub category_id: Option<String>,
    pub payment_method: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
    pub currency: String,
    pub fx_rate: Option<String>,
    pub description: String,
    pub tx_date: String,
    pub paid: bool,
    pub receipt_url: Option<String>,
    pub picture_url: Option<String>,
    pub card_id: Option<String>,
    pub bill_id: Option<String>,
    pub invoice_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Derived from source.type × destination.type: deposit | withdrawal | transfer.
    pub kind: String,
}

/// Create a transaction.
///
/// The spreadsheet model treats "income" and "expense" as actions on a single account
/// (the user's asset account) with a category. The backend internally uses two-legged
/// transactions where the other leg is a system bucket (`revenue` or `expense`).
///
/// To keep the API ergonomic, exactly one of `source_account_id` / `destination_account_id`
/// MAY be omitted, in which case the missing leg is derived from `category_id`:
///   - source provided + category present → destination = user.expense bucket (withdrawal).
///   - destination provided + category present → source = user.revenue bucket (deposit).
///   - both provided → leg pair validated as-is (covers transfers and explicit posts).
#[derive(Deserialize, ToSchema)]
pub struct CreateTransaction {
    pub source_account_id: Option<String>,
    pub destination_account_id: Option<String>,
    pub category_id: Option<String>,
    pub payment_method: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
    pub currency: String,
    pub description: String,
    pub tx_date: String,
    #[serde(default)]
    pub paid: Option<bool>,
    pub fx_rate: Option<String>,
    pub card_id: Option<String>,
    pub invoice_id: Option<String>,
    pub bill_id: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTransaction {
    pub source_account_id: Option<String>,
    pub destination_account_id: Option<String>,
    pub category_id: Option<String>,
    pub payment_method: Option<String>,
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "0.00")]
    pub value: Option<i64>,
    pub description: Option<String>,
    pub tx_date: Option<String>,
    pub paid: Option<bool>,
    pub fx_rate: Option<String>,
    pub card_id: Option<String>,
    pub invoice_id: Option<String>,
    pub bill_id: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTransfer {
    pub source_account_id: Uuid,
    pub destination_account_id: Uuid,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
    pub currency: String,
    pub fx_rate: Option<String>,
    pub tx_date: String,
    pub description: String,
    pub payment_method: String,
}
