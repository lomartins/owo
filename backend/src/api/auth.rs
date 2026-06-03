use crate::domain::session::Session;
use crate::domain::user::*;
use crate::error::{ApiError, ApiResult};
use crate::ids::{new_id, now_iso};
use crate::services::{password, session, user_provisioning};
use crate::state::{AppState, CurrentUser};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use uuid::Uuid;

#[utoipa::path(
    post, path = "/api/v1/auth/register", tag = "auth",
    request_body = RegisterRequest,
    responses((status = 201, body = AuthResponse))
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    if !state.config.allow_registration {
        return Err(ApiError::Forbidden);
    }
    if req.password.len() < 12 {
        return Err(ApiError::Validation {
            field: "password".into(),
            reason: "min 12 chars".into(),
        });
    }
    let pool = &state.pool;
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(pool)
        .await?;
    if exists.is_some() {
        return Err(ApiError::Conflict);
    }
    let user_id = new_id();
    let hash = password::hash_password(&req.password)?;
    let now = now_iso();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, default_currency, locale, created_at, updated_at, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'server')",
    )
    .bind(user_id.to_string())
    .bind(&req.email)
    .bind(&hash)
    .bind(&req.display_name)
    .bind(&req.default_currency)
    .bind(&req.locale)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
    user_provisioning::provision_user(&mut *tx, user_id, &req.default_currency, &req.device_id).await?;
    tx.commit().await?;

    let issued = session::issue(pool, user_id, &req.device_id, &req.device_name, None, None).await?;
    let user = load_user(pool, user_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user,
            session: SessionToken {
                id: issued.session_id.to_string(),
                token: issued.token,
                expires_at: issued.expires_at,
            },
        }),
    ))
}

#[utoipa::path(
    post, path = "/api/v1/auth/login", tag = "auth",
    request_body = LoginRequest,
    responses((status = 200, body = AuthResponse))
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let pool = &state.pool;
    let row: Option<(String, String, Option<String>, i64)> = sqlx::query_as(
        "SELECT id, password_hash, lockout_until, failed_login_count FROM users WHERE email = ? AND deleted_at IS NULL",
    )
    .bind(&req.email)
    .fetch_optional(pool)
    .await?;

    let Some((id, hash, lockout, _failed)) = row else {
        return Err(ApiError::Unauthenticated);
    };
    if let Some(lock) = lockout {
        if let Ok(t) = chrono::DateTime::parse_from_rfc3339(&lock) {
            if t > chrono::Utc::now() {
                return Err(ApiError::RateLimited);
            }
        }
    }
    if !password::verify_password(&req.password, &hash)? {
        let _ = sqlx::query(
            "UPDATE users SET failed_login_count = failed_login_count + 1, \
             lockout_until = CASE WHEN failed_login_count + 1 >= 5 THEN datetime('now','+15 minutes') ELSE lockout_until END \
             WHERE email = ?",
        )
        .bind(&req.email)
        .execute(pool)
        .await;
        return Err(ApiError::Unauthenticated);
    }
    let _ = sqlx::query("UPDATE users SET failed_login_count = 0, lockout_until = NULL WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await;

    let user_id = Uuid::parse_str(&id).map_err(|e| ApiError::Internal(e.into()))?;
    let issued = session::issue(pool, user_id, &req.device_id, &req.device_name, None, None).await?;
    let user = load_user(pool, user_id).await?;
    Ok(Json(AuthResponse {
        user,
        session: SessionToken {
            id: issued.session_id.to_string(),
            token: issued.token,
            expires_at: issued.expires_at,
        },
    }))
}

#[utoipa::path(
    post, path = "/api/v1/auth/logout", tag = "auth",
    security(("bearer" = [])),
    responses((status = 204))
)]
pub async fn logout(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<StatusCode> {
    session::revoke(&state.pool, cu.session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get, path = "/api/v1/auth/me", tag = "auth",
    security(("bearer" = [])),
    responses((status = 200, body = UserResponse))
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<Json<UserResponse>> {
    let user = load_user(&state.pool, cu.id).await?;
    Ok(Json(UserResponse { user }))
}

#[utoipa::path(
    patch, path = "/api/v1/auth/me", tag = "auth",
    security(("bearer" = [])),
    request_body = ProfileUpdate,
    responses((status = 200, body = UserResponse))
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(p): Json<ProfileUpdate>,
) -> ApiResult<Json<UserResponse>> {
    let now = now_iso();
    sqlx::query(
        "UPDATE users SET display_name = COALESCE(?, display_name), \
         default_currency = COALESCE(?, default_currency), \
         locale = COALESCE(?, locale), \
         partner_name = COALESCE(?, partner_name), \
         updated_at = ? WHERE id = ?",
    )
    .bind(p.display_name)
    .bind(p.default_currency)
    .bind(p.locale)
    .bind(p.partner_name)
    .bind(now)
    .bind(cu.id.to_string())
    .execute(&state.pool)
    .await?;
    let user = load_user(&state.pool, cu.id).await?;
    Ok(Json(UserResponse { user }))
}

#[utoipa::path(
    post, path = "/api/v1/auth/password", tag = "auth",
    security(("bearer" = [])),
    request_body = PasswordChangeRequest,
    responses((status = 204))
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(p): Json<PasswordChangeRequest>,
) -> ApiResult<StatusCode> {
    if p.new_password.len() < 12 {
        return Err(ApiError::Validation {
            field: "new_password".into(),
            reason: "min 12 chars".into(),
        });
    }
    let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(cu.id.to_string())
        .fetch_optional(&state.pool)
        .await?;
    let Some((hash,)) = row else {
        return Err(ApiError::NotFound);
    };
    if !password::verify_password(&p.current_password, &hash)? {
        return Err(ApiError::Unauthenticated);
    }
    let new_hash = password::hash_password(&p.new_password)?;
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(new_hash)
        .bind(now_iso())
        .bind(cu.id.to_string())
        .execute(&state.pool)
        .await?;
    let _ = session::revoke_all_except(&state.pool, cu.id, Some(cu.session_id)).await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SessionList {
    pub items: Vec<Session>,
}

#[utoipa::path(
    get, path = "/api/v1/auth/sessions", tag = "auth",
    security(("bearer" = [])),
    responses((status = 200, body = SessionList))
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<Json<SessionList>> {
    let rows: Vec<Session> = sqlx::query_as(
        "SELECT id, device_id, device_name, user_agent, ip_last_seen, created_at, last_used_at, expires_at, \
                CAST((id = ?) AS INTEGER) AS current \
         FROM sessions WHERE user_id = ? AND revoked_at IS NULL ORDER BY last_used_at DESC",
    )
    .bind(cu.session_id.to_string())
    .bind(cu.id.to_string())
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(SessionList { items: rows }))
}

#[utoipa::path(
    delete, path = "/api/v1/auth/sessions/{id}", tag = "auth",
    security(("bearer" = [])),
    params(("id" = Uuid, Path,)),
    responses((status = 204))
)]
pub async fn revoke_session(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let row: Option<(String,)> = sqlx::query_as("SELECT user_id FROM sessions WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(&state.pool)
        .await?;
    let Some((uid,)) = row else { return Err(ApiError::NotFound) };
    if uid != cu.id.to_string() {
        return Err(ApiError::Forbidden);
    }
    session::revoke(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete, path = "/api/v1/auth/sessions", tag = "auth",
    security(("bearer" = [])),
    responses((status = 204))
)]
pub async fn revoke_other_sessions(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
) -> ApiResult<StatusCode> {
    session::revoke_all_except(&state.pool, cu.id, Some(cu.session_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn load_user(pool: &sqlx::SqlitePool, id: Uuid) -> ApiResult<User> {
    let row: Option<User> = sqlx::query_as(
        "SELECT id, email, display_name, default_currency, locale, \
                CAST(is_admin AS INTEGER) != 0 AS is_admin, \
                partner_name, photo_url, \
                created_at, updated_at \
         FROM users WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;
    row.ok_or(ApiError::NotFound)
}
