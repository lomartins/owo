//! Idempotent per-user provisioning: revenue + expense bucket accounts (phase A3) and
//! the MVP seed categories (phase B7). Called from auth::register, and safe to re-invoke
//! as a self-heal (every INSERT is guarded by a NOT EXISTS check or a unique index).
//!
//! See specs/05-account-types.md and specs/06-budgets.md.

use crate::ids::{new_id, now_iso};
use sqlx::SqliteConnection;
use uuid::Uuid;

const SEED_CATEGORIES: &[&str] = &[
    "Food",
    "Leisure",
    "Transport",
    "Health",
    "Education",
    "Clothes",
    "Home",
    "Pet",
    "Subscriptions",
    "Other",
];

pub async fn provision_user(
    conn: &mut SqliteConnection,
    user_id: Uuid,
    default_currency: &str,
    device_id: &str,
) -> Result<(), sqlx::Error> {
    let now = now_iso();
    ensure_bucket(conn, user_id, "revenue", "Income", default_currency, device_id, &now).await?;
    ensure_bucket(conn, user_id, "expense", "Expense", default_currency, device_id, &now).await?;
    ensure_default_asset(conn, user_id, "Carteira", default_currency, device_id, &now).await?;
    ensure_seed_categories(conn, user_id, device_id, &now).await?;
    Ok(())
}

/// Provision a single default user-facing asset account (e.g. a Carteira/wallet)
/// idempotently by name. Skips insert when a non-archived asset account with the
/// exact name already exists.
async fn ensure_default_asset(
    conn: &mut SqliteConnection,
    user_id: Uuid,
    name: &str,
    currency: &str,
    device_id: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    let exists: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM accounts WHERE user_id = ? AND type = 'asset' AND name = ? AND deleted_at IS NULL",
    )
    .bind(user_id.to_string())
    .bind(name)
    .fetch_optional(&mut *conn)
    .await?;
    if exists.is_some() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO accounts (id, user_id, name, type, currency, initial_balance, archived, \
            created_at, updated_at, device_id) \
         VALUES (?, ?, ?, 'asset', ?, 0, 0, ?, ?, ?)",
    )
    .bind(new_id().to_string())
    .bind(user_id.to_string())
    .bind(name)
    .bind(currency)
    .bind(now)
    .bind(now)
    .bind(device_id)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn ensure_bucket(
    conn: &mut SqliteConnection,
    user_id: Uuid,
    bucket_type: &str,
    name: &str,
    currency: &str,
    device_id: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    let exists: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM accounts WHERE user_id = ? AND type = ? AND deleted_at IS NULL",
    )
    .bind(user_id.to_string())
    .bind(bucket_type)
    .fetch_optional(&mut *conn)
    .await?;
    if exists.is_some() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO accounts (id, user_id, name, type, currency, initial_balance, archived, \
            created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, 0, 0, ?, ?, ?)",
    )
    .bind(new_id().to_string())
    .bind(user_id.to_string())
    .bind(name)
    .bind(bucket_type)
    .bind(currency)
    .bind(now)
    .bind(now)
    .bind(device_id)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn ensure_seed_categories(
    conn: &mut SqliteConnection,
    user_id: Uuid,
    device_id: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    for name in SEED_CATEGORIES {
        let exists: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM categories WHERE user_id = ? AND name = ? AND deleted_at IS NULL",
        )
        .bind(user_id.to_string())
        .bind(name)
        .fetch_optional(&mut *conn)
        .await?;
        if exists.is_some() {
            continue;
        }
        sqlx::query(
            "INSERT INTO categories (id, user_id, name, kind, archived, created_at, updated_at, device_id) \
             VALUES (?, ?, ?, 'EXPENSE', 0, ?, ?, ?)",
        )
        .bind(new_id().to_string())
        .bind(user_id.to_string())
        .bind(name)
        .bind(now)
        .bind(now)
        .bind(device_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
