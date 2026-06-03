use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Page {
    pub limit: u32,
    pub order: String,
    pub returned: u32,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ListEnvelope<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Page>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct Money(
    #[serde(with = "rust_decimal::serde::str")]
    #[schema(value_type = String, example = "1234.56")]
    pub Decimal,
);

impl Money {
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }

    pub fn to_cents(self) -> i64 {
        let scaled = self.0 * Decimal::from(100);
        scaled.round().mantissa() as i64
    }

    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }
}

impl From<i64> for Money {
    fn from(cents: i64) -> Self {
        Self::from_cents(cents)
    }
}

/// Serde adapter: i64 cents in storage <-> Decimal string on the wire.
/// Use as `#[serde(with = "crate::domain::common::cents_as_decimal")]` on i64 money fields
/// in API request/response structs. Storage rows keep i64 untouched.
pub mod cents_as_decimal {
    use rust_decimal::Decimal;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(cents: &i64, s: S) -> Result<S::Ok, S::Error> {
        let d = Decimal::new(*cents, 2);
        s.serialize_str(&d.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
        let raw = String::deserialize(d)?;
        let dec: Decimal = raw.parse().map_err(de::Error::custom)?;
        let scaled = dec * Decimal::from(100);
        Ok(scaled.round().mantissa() as i64)
    }
}

/// Optional variant of [`cents_as_decimal`] for `Option<i64>` fields.
pub mod cents_as_decimal_opt {
    use rust_decimal::Decimal;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(cents: &Option<i64>, s: S) -> Result<S::Ok, S::Error> {
        match cents {
            Some(c) => {
                let d = Decimal::new(*c, 2);
                s.serialize_some(&d.to_string())
            }
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
        let raw: Option<String> = Option::deserialize(d)?;
        match raw {
            Some(s) => {
                let dec: Decimal = s.parse().map_err(de::Error::custom)?;
                let scaled = dec * Decimal::from(100);
                Ok(Some(scaled.round().mantissa() as i64))
            }
            None => Ok(None),
        }
    }
}
