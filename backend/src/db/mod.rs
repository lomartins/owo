use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

/// Creates and returns a PostgreSQL connection pool.
///
/// Reads `DATABASE_URL` from the environment (loaded from `.env` by main).
/// Returns an error if the variable is missing or the connection fails.
pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL environment variable is not set")?;

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")
}
