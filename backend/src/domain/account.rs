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
    pub archived: Option<bool>,
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
