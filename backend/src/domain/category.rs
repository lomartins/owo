use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub kind: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateCategory {
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub archived: Option<bool>,
}
