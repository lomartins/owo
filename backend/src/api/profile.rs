use crate::domain::user::{PhotoUploadRequest, PhotoUploadResponse};
use crate::error::{ApiError, ApiResult};
use crate::ids::now_iso;
use crate::state::{AppState, CurrentUser};
use axum::extract::State;
use axum::{Extension, Json};
use base64::Engine;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const MAX_BYTES: usize = 5 * 1024 * 1024; // 5 MiB

fn uploads_root() -> PathBuf {
    PathBuf::from(std::env::var("OWO_UPLOADS_DIR").unwrap_or_else(|_| "uploads".to_string()))
}

/// Accepts a data URL, writes the payload to `{OWO_UPLOADS_DIR}/u/<user>.<ext>`,
/// updates `users.photo_url`, returns the URL the SPA should load.
#[utoipa::path(
    post, path = "/api/v1/profile/photo", tag = "auth",
    security(("bearer" = [])),
    request_body = PhotoUploadRequest,
    responses((status = 200, body = PhotoUploadResponse))
)]
pub async fn upload_photo(
    State(state): State<AppState>,
    Extension(cu): Extension<CurrentUser>,
    Json(req): Json<PhotoUploadRequest>,
) -> ApiResult<Json<PhotoUploadResponse>> {
    let (mime, b64) = parse_data_url(&req.data_url)?;
    let ext = match mime.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => {
            return Err(ApiError::Validation {
                field: "data_url".into(),
                reason: format!("unsupported mime type: {mime}"),
            });
        }
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.as_bytes())
        .map_err(|e| ApiError::Validation {
            field: "data_url".into(),
            reason: format!("invalid base64: {e}"),
        })?;
    if bytes.len() > MAX_BYTES {
        return Err(ApiError::Validation {
            field: "data_url".into(),
            reason: format!("payload too large ({} bytes; max {MAX_BYTES})", bytes.len()),
        });
    }

    let root = uploads_root();
    let dir = root.join("u");
    tokio::fs::create_dir_all(&dir).await.map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    // Cache-bust via timestamp suffix so refreshing the SPA picks up new uploads.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let filename = format!("{}-{stamp}.{ext}", cu.id);
    let path = dir.join(&filename);
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    file.write_all(&bytes).await.map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    file.flush().await.map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let photo_url = format!("/uploads/u/{}", filename);
    let now = now_iso();
    sqlx::query("UPDATE users SET photo_url = ?, updated_at = ? WHERE id = ?")
        .bind(&photo_url)
        .bind(&now)
        .bind(cu.id.to_string())
        .execute(&state.pool)
        .await?;

    Ok(Json(PhotoUploadResponse { photo_url }))
}

fn parse_data_url(s: &str) -> ApiResult<(String, String)> {
    // Expected: data:<mime>;base64,<payload>
    let Some(rest) = s.strip_prefix("data:") else {
        return Err(ApiError::Validation {
            field: "data_url".into(),
            reason: "must start with 'data:'".into(),
        });
    };
    let Some((meta, payload)) = rest.split_once(',') else {
        return Err(ApiError::Validation {
            field: "data_url".into(),
            reason: "missing payload separator".into(),
        });
    };
    let mut mime = meta.to_string();
    if let Some((m, params)) = meta.split_once(';') {
        if !params.contains("base64") {
            return Err(ApiError::Validation {
                field: "data_url".into(),
                reason: "only base64-encoded payloads supported".into(),
            });
        }
        mime = m.to_string();
    } else {
        return Err(ApiError::Validation {
            field: "data_url".into(),
            reason: "missing ;base64 declaration".into(),
        });
    }
    Ok((mime, payload.to_string()))
}
