# Phase A2 — Two-legged transactions

Status: shipped (2026-05-27)

## What shipped

- Migration `0003_two_legged_tx.sql`:
  - Rewrote `transactions` to use `source_account_id` + `destination_account_id` (both NOT NULL, FK to accounts).
  - Dropped stored `type` column and `transfer_pair_id`.
  - Added `paid INTEGER DEFAULT 1` (mirrors the spreadsheet's paid flag for variable expenses and bills).
  - Best-effort backfill: legacy INCOME → source = user's revenue bucket, destination = original `account_id`. EXPENSE → source = original account, destination = user's expense bucket. TRANSFER rows skipped (assumed early-dev empty state).
  - Added indexes `tx_source_date`, `tx_dest_date`. Dropped legacy `tx_account_date`.
  - CHECK ensures `source_account_id <> destination_account_id` and `value > 0`.
- `domain::transaction`:
  - `Transaction` carries source/destination + `paid`. Removed stored `type`.
  - `CreateTransaction` requires both legs; supports optional `paid`, `card_id`, `invoice_id`, `bill_id`.
  - `CreateTransfer` renamed `from/to` → `source/destination`, requires `payment_method`.
- `api/transactions.rs`:
  - `create` looks up `(source.type, destination.type)`, computes derived type, rejects `INVALID_LEG_PAIR` and enforces "transfer ⇒ no category, withdrawal/deposit ⇒ category required".
  - `transfer` (was 501) now creates a category-less row between two user-owned accounts; rejects bucket accounts as legs.
  - `list` supports `?month=YYYY-MM`, `?category_id=`, `?account_id=` (matches either leg). Cursor pagination preserved with updated `filters_hash`.
  - All SELECTs use a new `TX_SELECT_COLS` constant matching the new column order.
- `api/accounts.rs`:
  - Balance expression uses both legs: `initial_balance + sum(incoming) - sum(outgoing)`.
  - `GET /accounts/:id/balance?as_of=YYYY-MM-DD` filters `tx_date <= as_of` on both subqueries. (Also covers Phase B8.)

## Verification

- `cargo check` clean.
- Manual round-trip TBD post-Phase-B (web SPA enables faster checking).

## Deviations

- The migration's TRANSFER backfill is intentionally lossy in the early-dev state — see migration header comment. Documented; not a production issue because no users exist yet.

## Follow-ups

- A `PUT /transactions/:id` and `DELETE /transactions/:id` handler still need to be added (currently only `create`/`list`/`transfer` exist). Tracking under a new task if user wants editing in MVP — the web SPA spec includes edit/delete so this should land before Phase D.
- Trigger-based enforcement of the invoice-payment no-category rule is Phase C10.
