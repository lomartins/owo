use crate::error::ApiError;
use crate::ids::{new_id, now_iso};
use crate::state::CurrentUser;
use anyhow::Result;
use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

pub const LIFETIME_DAYS: i64 = 90;
pub const HARD_CAP_DAYS: i64 = 365;

pub struct Issued {
    pub session_id: Uuid,
    pub token: String,
    pub expires_at: String,
}

pub fn hash_token(token: &str) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    h.finalize().to_vec()
}

fn random_token() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(buf)
}

pub async fn issue(
    pool: &SqlitePool,
    user_id: Uuid,
    device_id: &str,
    device_name: &str,
    user_agent: Option<&str>,
    ip: Option<&str>,
) -> Result<Issued> {
    let session_id = new_id();
    let token = random_token();
    let token_hash = hash_token(&token);
    let now = now_iso();
    let expires = (Utc::now() + Duration::days(LIFETIME_DAYS))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, device_id, device_name, user_agent, ip_last_seen, created_at, last_used_at, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(session_id.to_string())
    .bind(user_id.to_string())
    .bind(token_hash)
    .bind(device_id)
    .bind(device_name)
    .bind(user_agent)
    .bind(ip)
    .bind(&now)
    .bind(&now)
    .bind(&expires)
    .execute(pool)
    .await?;

    Ok(Issued {
        session_id,
        token,
        expires_at: expires,
    })
}

pub async fn verify(pool: &SqlitePool, token: &str) -> Result<Option<CurrentUser>, ApiError> {
    let token_hash = hash_token(token);
    let row: Option<(String, String, i64, String, String, Option<String>)> = sqlx::query_as(
        "SELECT s.id, s.user_id, u.is_admin, s.device_id, s.expires_at, s.revoked_at \
         FROM sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.token_hash = ?",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    let Some((sid, uid, is_admin, device_id, expires_at, revoked_at)) = row else {
        return Ok(None);
    };
    if revoked_at.is_some() {
        return Ok(None);
    }
    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&expires_at) {
        if parsed <= Utc::now() {
            return Ok(None);
        }
    }

    // sliding window: bump last_used_at if last update >60s ago
    let _ = sqlx::query(
        "UPDATE sessions SET last_used_at = ?, expires_at = MIN(\
            datetime(?, '+90 days'), \
            datetime(created_at, '+365 days')\
         ) WHERE id = ? AND last_used_at < datetime(?, '-60 seconds')",
    )
    .bind(now_iso())
    .bind(now_iso())
    .bind(&sid)
    .bind(now_iso())
    .execute(pool)
    .await;

    Ok(Some(CurrentUser {
        id: Uuid::parse_str(&uid).map_err(|e| ApiError::Internal(e.into()))?,
        is_admin: is_admin != 0,
        session_id: Uuid::parse_str(&sid).map_err(|e| ApiError::Internal(e.into()))?,
        device_id,
    }))
}

pub async fn revoke(pool: &SqlitePool, session_id: Uuid) -> Result<bool> {
    let res = sqlx::query("UPDATE sessions SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL")
        .bind(now_iso())
        .bind(session_id.to_string())
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn revoke_all_except(
    pool: &SqlitePool,
    user_id: Uuid,
    keep: Option<Uuid>,
) -> Result<u64> {
    let res = sqlx::query(
        "UPDATE sessions SET revoked_at = ? \
         WHERE user_id = ? AND revoked_at IS NULL AND id != ?",
    )
    .bind(now_iso())
    .bind(user_id.to_string())
    .bind(keep.map(|u| u.to_string()).unwrap_or_default())
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}
