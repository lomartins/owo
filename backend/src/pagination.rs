use crate::error::{ApiError, ApiResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Order::Desc
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cursor {
    pub tx_date: String,
    pub id: String,
    pub order: Order,
    pub filters_hash: String,
}

impl Cursor {
    pub fn encode(&self) -> String {
        let json = serde_json::to_vec(self).expect("cursor serialize");
        URL_SAFE_NO_PAD.encode(json)
    }

    pub fn decode(
        raw: &str,
        expected_filters_hash: &str,
        expected_order: &Order,
    ) -> ApiResult<Self> {
        let bytes = URL_SAFE_NO_PAD
            .decode(raw)
            .map_err(|_| ApiError::CursorFilterMismatch)?;
        let cursor: Cursor =
            serde_json::from_slice(&bytes).map_err(|_| ApiError::CursorFilterMismatch)?;
        if cursor.filters_hash != expected_filters_hash || &cursor.order != expected_order {
            return Err(ApiError::CursorFilterMismatch);
        }
        Ok(cursor)
    }
}

pub fn filters_hash(parts: &[(&str, &str)]) -> String {
    let mut hasher = Sha256::new();
    for (k, v) in parts {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        hasher.update(v.as_bytes());
        hasher.update(b"&");
    }
    let digest = hasher.finalize();
    hex::encode(&digest[..6])
}
