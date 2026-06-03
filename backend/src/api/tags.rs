use crate::domain::tag::*;
use crate::error::ApiResult;
use crate::ids::{new_id, now_iso};
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct TagList { pub items: Vec<Tag> }

#[utoipa::path(get, path = "/api/v1/tags", tag = "tags", security(("bearer" = [])),
    responses((status = 200, body = TagList)))]
pub async fn list(State(state): State<AppState>, Extension(cu): Extension<CurrentUser>) -> ApiResult<Json<TagList>> {
    let items: Vec<Tag> = sqlx::query_as(
        "SELECT id, name, color, created_at, updated_at FROM tags WHERE user_id = ? AND deleted_at IS NULL ORDER BY name",
    )
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(TagList { items }))
}

#[utoipa::path(post, path = "/api/v1/tags", tag = "tags", security(("bearer" = [])),
    request_body = CreateTag, responses((status = 201, body = Tag)))]
pub async fn create(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<CreateTag>,
) -> ApiResult<(StatusCode, Json<Tag>)> {
    let id = new_id();
    let now = now_iso();
    sqlx::query(
        "INSERT INTO tags (id, user_id, name, color, created_at, updated_at, device_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(cu.id.to_string())
    .bind(&req.name)
    .bind(&req.color)
    .bind(&now)
    .bind(&now)
    .bind(&cu.device_id)
    .execute(&state.pool)
    .await?;
    let tag: Tag = sqlx::query_as("SELECT id, name, color, created_at, updated_at FROM tags WHERE id = ?")
        .bind(id.to_string())
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(tag)))
}
