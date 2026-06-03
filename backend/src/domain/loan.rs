use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Loan {
    pub id: String,
    pub account_id: String,
    pub description: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub total_value: i64,
    pub currency: String,
    pub interest_rate: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub installment_value: i64,
    pub installment_count: i64,
    pub paid_installments: i64,
    pub start_date: String,
    pub first_due_date: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateLoan {
    pub account_id: String,
    pub description: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub total_value: i64,
    pub currency: String,
    pub interest_rate: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub installment_value: i64,
    pub installment_count: i64,
    pub start_date: String,
    pub first_due_date: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PayInstallment {
    pub account_id: String,
    pub tx_date: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub value: i64,
}
