use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub currency: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub initial_balance: i64,
    /// Signed value of the account's opening-balance transaction (equity leg).
    /// This is the editable "initial value" surfaced to the user.
    #[sqlx(default)]
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub opening_balance: i64,
    #[sqlx(default)]
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub current_balance: i64,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateAccount {
    pub name: String,
    pub r#type: String,
    pub currency: String,
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "0.00")]
    pub initial_balance: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub currency: Option<String>,
    pub archived: Option<bool>,
    /// New signed opening balance (the "initial value"). Adjusts the account's
    /// opening-balance transaction. Omit to leave it unchanged.
    #[serde(default, with = "crate::domain::common::cents_as_decimal_opt")]
    #[schema(value_type = Option<String>, example = "0.00")]
    pub opening_balance: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AccountBalance {
    pub account_id: String,
    pub currency: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub initial_balance: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub transactions_total: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub current_balance: i64,
    pub as_of: String,
}
