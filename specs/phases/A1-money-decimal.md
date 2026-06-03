# Phase A1 — Money at the API boundary

Status: shipped (2026-05-27)

## What shipped

- Added `rust_decimal = "1"` and `utoipa` `decimal` feature to `backend/Cargo.toml`.
- Replaced the legacy `domain::common::Money { value: i64, currency }` schema-only struct with:
  - `Money(Decimal)` newtype, JSON-string serialized via `rust_decimal::serde::str`.
  - `cents_as_decimal` serde adapter module (i64 ↔ Decimal-string).
  - `cents_as_decimal_opt` for `Option<i64>` fields.
- Annotated every money field across all `backend/src/domain/*.rs` files with the adapter, so request/response JSON carries decimal strings like `"1234.56"` while the in-memory + storage representation stays i64 cents:
  - `account.rs`: `initial_balance`, `current_balance`, `transactions_total`, optional `initial_balance` on create.
  - `transaction.rs`: `value` on `Transaction`, `CreateTransaction`, `CreateTransfer`.
  - `budget.rs`: `limit` on `Budget`, `CreateBudget`.
  - `bill.rs`: `value` on `Bill`, `CreateBill`, `PayBill`.
  - `card.rs`: `limit` (Option) on `Card`, `CreateCard`; `total` on `CardInvoice`.
  - `loan.rs`: `total_value`, `installment_value` on `Loan`, `CreateLoan`; `value` on `PayInstallment`.
  - `investment.rs`: `principal`, `current_value` on `Investment`, `CreateInvestment`; `current_value` on `UpdateValue`.
  - `goal.rs`: `target_value`, `current_value`.
- OpenAPI schemas declare these fields as `String` with a `1234.56`-style example.

## Verification

- `cargo check` clean (only existing unrelated warnings if any; build succeeded).
- No SQL or storage changes — i64 cents in DB stay i64.

## Deviations

- Per ADR `decisions/0001-money-at-api-boundary.md`: storage stays `INTEGER` cents instead of `NUMERIC(14, 2)` / TEXT. The spec calls for full Decimal end-to-end; the deviation is documented and the wire format complies.

## Follow-ups for later phases

- When Postgres lands, columns become `NUMERIC(14, 2)`; bind/extract converts to/from `Decimal` directly. The serde adapter is unaffected.
- Multi-currency: `currency` is already a field; the FX-rate conversion will fan out where money is summed across currencies. Out of MVP.
