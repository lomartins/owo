use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Budget {
    pub id: String,
    pub category_id: String,
    pub month: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub estimated_amount: i64,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateBudget {
    pub category_id: String,
    pub month: String,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub estimated_amount: i64,
    pub currency: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateBudget {
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub estimated_amount: i64,
}

#[derive(Serialize, ToSchema)]
pub struct BudgetRow {
    pub category_id: String,
    pub category_name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub budget_id: Option<String>,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub estimated: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub spent: i64,
    #[serde(with = "crate::domain::common::cents_as_decimal")]
    #[schema(value_type = String, example = "0.00")]
    pub difference: i64,
}

#[derive(Serialize, ToSchema)]
pub struct BudgetMonth {
    pub month: String,
    pub items: Vec<BudgetRow>,
}
