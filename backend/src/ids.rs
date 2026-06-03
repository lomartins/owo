use uuid::Uuid;

pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
