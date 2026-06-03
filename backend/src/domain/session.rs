use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub device_id: String,
    pub device_name: String,
    pub user_agent: Option<String>,
    pub ip_last_seen: Option<String>,
    pub created_at: String,
    pub last_used_at: String,
    pub expires_at: String,
    #[sqlx(default)]
    pub current: bool,
}
