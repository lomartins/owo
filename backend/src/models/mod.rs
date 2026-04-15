//! Core domain models
//!
//! Structs in this module map 1-to-1 to database tables via `sqlx::FromRow`.
//! They are also serialized/deserialized to JSON via `serde`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

/// Wraps a paginated list response: `{ "data": [...], "total": N, "page": N, "per_page": N }`
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub account_type: String,
    pub balance: rust_decimal::Decimal,
    pub currency_code: String,
    pub external_account_id: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub account_type: String,
    pub currency_code: Option<String>,
    pub initial_balance: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum TransactionStatus {
    Posted,
    Pending,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i64,
    pub user_id: i64,
    pub account_id: i64,
    pub amount: rust_decimal::Decimal,
    pub category_id: Option<i64>,
    pub description: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
    pub status: TransactionStatus,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub amount: rust_decimal::Decimal,
    pub description: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub category_id: Option<i64>,
}
