use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{ConnectOptions, Connection, SqlitePool};
use std::str::FromStr;

/// Run embedded migrations on a dedicated connection with foreign-key enforcement
/// OFF. Table-rebuild migrations (changing a CHECK constraint requires DROP +
/// recreate of a table that other tables reference by FK) cannot run with FK
/// enforcement on: `PRAGMA foreign_keys` is a no-op inside a transaction, and
/// `defer_foreign_keys` does not reconcile the deferred-violation counter across a
/// parent-table DROP. FK enforcement must therefore be disabled *before* the
/// migration transaction begins — i.e. at connection setup. The app pool keeps
/// foreign_keys ON for normal operation.
pub async fn run_migrations(url: &str) -> Result<()> {
    let mut conn = SqliteConnectOptions::from_str(url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true)
        .foreign_keys(false)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .connect()
        .await
        .with_context(|| format!("failed to connect for migrations: {url}"))?;
    sqlx::migrate!("./migrations/sqlite").run(&mut conn).await?;
    conn.close().await?;
    Ok(())
}

pub async fn pool(url: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(opts)
        .await
        .with_context(|| format!("failed to connect to {url}"))?;
    Ok(pool)
}
