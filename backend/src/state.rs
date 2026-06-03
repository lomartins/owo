use crate::config::Config;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: SqlitePool, config: Config) -> Self {
        Self { pool, config }
    }
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub is_admin: bool,
    pub session_id: Uuid,
    pub device_id: String,
}
