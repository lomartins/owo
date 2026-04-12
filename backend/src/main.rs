use anyhow::Context;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env (ok to fail if the file doesn't exist)
    dotenvy::dotenv().ok();

    // Initialize structured logging — level controlled by RUST_LOG env var
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create the database connection pool
    let pool = owo_backend::db::create_pool()
        .await
        .context("Failed to connect to the database. Is Docker running?")?;

    // Apply any pending database migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    info!("Database ready");

    // Build the router — routes will be added in Phase 4
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool);

    // Resolve server address from environment variables
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .context("SERVER_PORT must be a valid port number (0–65535)")?;

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {addr}"))?;

    info!("Server listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .context("Server crashed")?;

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
