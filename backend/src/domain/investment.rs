use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Investment {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub r#type: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub principal: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub current_value: i64,
    pub currency: String,
    pub rate_index: Option<String>,
    pub rate_spread: Option<String>,
    pub purchase_date: String,
    pub expiration_date: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateInvestment {
    pub account_id: String,
    pub name: String,
    pub r#type: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub principal: i64,
    pub currency: String,
    pub rate_index: Option<String>,
    pub rate_spread: Option<String>,
    pub purchase_date: String,
    pub expiration_date: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateValue {
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub current_value: i64,
}
