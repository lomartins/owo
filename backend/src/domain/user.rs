use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub default_currency: String,
    pub locale: String,
    pub is_admin: bool,
    pub partner_name: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub default_currency: String,
    pub locale: String,
    pub device_id: String,
    pub device_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub device_id: String,
    pub device_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ProfileUpdate {
    pub display_name: Option<String>,
    pub default_currency: Option<String>,
    pub locale: Option<String>,
    pub partner_name: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct PhotoUploadRequest {
    /// data URL with base64 payload, e.g. `"data:image/jpeg;base64,..."`.
    pub data_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct PhotoUploadResponse {
    pub photo_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: User,
    pub session: SessionToken,
}

#[derive(Serialize, ToSchema)]
pub struct SessionToken {
    pub id: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub user: User,
}
