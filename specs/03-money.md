# 03 — Money representation

## Rule

- **API boundary**: `rust_decimal::Decimal`, serialized as a JSON string (e.g. `"1234.56"`).
- **Storage**: `INTEGER` cents (smallest currency unit). Existing schema stands.
- **In-memory between API and DB**: `Decimal`. Conversion is at the storage adapter only.

This is a deviation from a strict reading of the brief (which calls for `NUMERIC(14,2)` end-to-end). See `decisions/0001-money-at-api-boundary.md` for rationale.

## Code shape

A single `Money` wrapper in `backend/src/domain/common.rs`:

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Money as Decimal at the API boundary, persisted as i64 cents.
/// Construct from cents or from Decimal. Round-trips exactly for currencies
/// with 2 decimal places (the only ones in MVP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(transparent)]
pub struct Money(#[serde(with = "rust_decimal::serde::str")] pub Decimal);

impl Money {
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }
    pub fn to_cents(self) -> i64 {
        // value * 100, rounded half-to-even. Errors only on overflow.
        (self.0 * Decimal::from(100))
            .round_dp(0)
            .mantissa() as i64
    }
}
```

## Conversion sites

- Inbound: `Json<CreateTransaction>` → `Money` → `to_cents()` at the `sqlx::query(...).bind(...)` site.
- Outbound: `SELECT value INTEGER` → `i64` → `Money::from_cents(i64)` in `From<Row>` / `FromRow`.
- No `f64` anywhere. Searchable invariant: `grep -rn 'f64' backend/src/` returns only test or non-financial use.

## What this rule does not allow

- Floats for money: banned.
- Storing fractional cents: banned (truncate or round to cents at the boundary, never silently).
- Computing budget totals client-side: banned. Aggregates are SQL.

## Future migration to NUMERIC

When Postgres lands, the column type changes from `INTEGER` to `NUMERIC(14, 2)`. The `Money` wrapper is unchanged. `to_cents` / `from_cents` are replaced with a direct `Decimal` bind. That is a one-day change.
