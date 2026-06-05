use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .init();

    let config = owo::config::Config::from_env()?;
    // Migrations run on their own FK-disabled connection (see db::run_migrations).
    owo::db::run_migrations(&config.database_url).await?;
    let pool = owo::db::pool(&config.database_url).await?;

    let state = owo::state::AppState::new(pool, config.clone());
    let app = owo::server::router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on http://{}", config.bind_addr);
    tracing::info!("Swagger UI at http://{}/docs", config.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
