use crate::ids::{new_id, now_iso};
use sqlx::SqliteExecutor;
use uuid::Uuid;

pub async fn write<'a, E>(
    exec: E,
    user_id: Uuid,
    entity_type: &str,
    entity_id: &str,
    action: &str,
    diff: Option<serde_json::Value>,
    device_id: Option<&str>,
) -> sqlx::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let id = new_id().to_string();
    let user = user_id.to_string();
    let ts = now_iso();
    let diff_json = diff.map(|v| v.to_string());
    sqlx::query(
        "INSERT INTO audit_log (id, user_id, entity_type, entity_id, action, diff, timestamp, device_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(user)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(diff_json)
    .bind(ts)
    .bind(device_id)
    .execute(exec)
    .await?;
    Ok(())
}
