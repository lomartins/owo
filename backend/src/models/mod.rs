//! Core domain models
//!
//! Structs in this module map 1-to-1 to database tables via `sqlx::FromRow`.
//! They are also serialized/deserialized to JSON via `serde`.

use serde::{Deserialize, Serialize};

// ============================================================================
// Response envelope types (Phase 3.4 — Claude)
// ============================================================================

/// Wraps a single-item API response: `{ "data": T }`
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

// ============================================================================
// USER TASK 3.2 — Account model  (Easy)
//
// Write two structs below: `Account` and `CreateAccountRequest`.
//
// You will also need:
//   rust_decimal::Decimal  ← use the full path, no extra `use` statement needed
//
// ── Account ──────────────────────────────────────────────────────────────────
// Maps to the `accounts` table. Required derives:
//   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
//
// Fields:
//   pub id:                   i64
//   pub user_id:              i64
//   pub name:                 String
//   pub account_type:         String
//   pub balance:              rust_decimal::Decimal   ← NEVER use f64 for money
//   pub currency_code:        String
//   pub external_account_id:  Option<String>
//   pub is_active:            bool
//   pub created_at:           chrono::DateTime<chrono::Utc>
//   pub updated_at:           chrono::DateTime<chrono::Utc>
//
// ── CreateAccountRequest ─────────────────────────────────────────────────────
// Required derives: #[derive(Debug, Deserialize)]
//
// Fields:
//   pub name:            String
//   pub account_type:    String
//   pub currency_code:   Option<String>                    — service defaults to "BRL" if None
//   pub initial_balance: Option<rust_decimal::Decimal>
//
// Run `cargo check` when done.
// ============================================================================

// ← Your structs go here (Task 3.2)

// ============================================================================
// USER TASK 3.3 — Transaction model + status enum  (Medium)
//
// Write three types below: `TransactionStatus`, `Transaction`, and
// `CreateTransactionRequest`.
//
// ── TransactionStatus ────────────────────────────────────────────────────────
// An enum that maps to the `status` VARCHAR column in `transactions`.
// Required derives:
//   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
// Required attribute (tells sqlx how to map enum variants to DB strings):
//   #[sqlx(type_name = "varchar", rename_all = "lowercase")]
//
// Variants: Posted, Pending, Failed
//
// ── Transaction ──────────────────────────────────────────────────────────────
// Maps to the `transactions` table. Required derives:
//   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
//
// Fields:
//   pub id:               i64
//   pub user_id:          i64
//   pub account_id:       i64
//   pub amount:           rust_decimal::Decimal
//   pub category_id:      Option<i64>
//   pub description:      Option<String>
//   pub transaction_date: chrono::NaiveDate          ← DATE column → NaiveDate, NOT DateTime<Utc>
//   pub recorded_at:      chrono::DateTime<chrono::Utc>
//   pub status:           TransactionStatus
//
// ── CreateTransactionRequest ─────────────────────────────────────────────────
// Required derives: #[derive(Debug, Deserialize)]
//
// Fields:
//   pub amount:           rust_decimal::Decimal
//   pub description:      Option<String>
//   pub transaction_date: chrono::NaiveDate
//   pub category_id:      Option<i64>
//
// Note: account_id and user_id come from the URL path, not the request body.
//
// Run `cargo check` when done.
// ============================================================================

// ← Your types go here (Task 3.3)
