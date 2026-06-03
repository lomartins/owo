use crate::domain::category::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, utoipa::ToSchema)]
pub struct CategoryList {
    pub items: Vec<Category>,
}

#[utoipa::path(get, path = "/api/v1/categories", tag = "categories", security(("bearer" = [])),
    responses((status = 200, body = CategoryList)))]
pub async fn list(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<Json<CategoryList>> {
    let items: Vec<Category> = sqlx::query_as(
        "SELECT id, name, parent_id, icon, color, kind, CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM categories WHERE user_id = ? AND deleted_at IS NULL ORDER BY name",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(CategoryList { items }))
}

#[utoipa::path(post, path = "/api/v1/categories", tag = "categories", security(("bearer" = [])),
    request_body = CreateCategory, responses((status = 201, body = Category)))]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateCategory>,
) -> ApiResult<(StatusCode, Json<Category>)> {
    let id = new_id();
    let now = now_iso();
    sqlx::query(
        "INSERT INTO categories (id, user_id, name, parent_id, icon, color, kind, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .bind(&req.name)
    .bind(req.parent_id.map(|u| u.to_string()))
    .bind(&req.icon)
    .bind(&req.color)
    .bind(&req.kind)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&state.pool)
    .await?;
    let cat: Category = sqlx::query_as(
        "SELECT id, name, parent_id, icon, color, kind, CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM categories WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(cat)))
}

#[utoipa::path(patch, path = "/api/v1/categories/{id}", tag = "categories", security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    request_body = UpdateCategory, responses((status = 200, body = Category)))]
pub async fn update(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCategory>,
) -> ApiResult<Json<Category>> {
    let now = now_iso();
    let res = sqlx::query(
        "UPDATE categories SET \
            name     = COALESCE(?, name), \
            kind     = COALESCE(?, kind), \
            icon     = COALESCE(?, icon), \
            color    = COALESCE(?, color), \
            archived = COALESCE(?, archived), \
            updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(req.name)
    .bind(req.kind)
    .bind(req.icon)
    .bind(req.color)
    .bind(req.archived.map(|b| if b { 1_i64 } else { 0_i64 }))
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&state.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    let cat: Category = sqlx::query_as(
        "SELECT id, name, parent_id, icon, color, kind, CAST(archived AS INTEGER) != 0 AS archived, created_at, updated_at \
         FROM categories WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(cat))
}

#[utoipa::path(delete, path = "/api/v1/categories/{id}", tag = "categories", security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    responses((status = 204)))]
pub async fn delete(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let now = now_iso();
    let res = sqlx::query(
        "UPDATE categories SET deleted_at = ?, updated_at = ?, sync_version = sync_version + 1 \
         WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(&now)
    .bind(&now)
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .execute(&state.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
