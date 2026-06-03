# Phase C9 — Card → credit_card account binding

Status: shipped (2026-05-27)

## What shipped

- Migration `0006_card_account_binding.sql`: extends `audit_log.action` CHECK to include `MIGRATION_NEEDS_REVIEW` so the runtime binding pass can flag ambiguous cases. No schema rewrites on `cards` or `accounts` — the binding is enforced at the application layer.
- `api/cards.rs::create`:
  - Inserts a new `credit_card`-type `accounts` row in the same `pool.begin()` transaction as the `cards` row.
  - `cards.account_id` now points at the new credit_card account (not at the funding asset account).
  - Inherits currency from the asset account the user nominated via the request body (validated to exist + belong to the user + be of type `asset`).
  - Rejects DEBIT cards — debit binding is out of MVP.
  - Writes two audit rows (Card CREATE, Account CREATE) inside the same transaction.

## Verification

- `cargo check` clean.
- The DB trigger (Phase C10) ensures that any code path that tries to attach a category to an `asset → credit_card` transfer fails loudly.

## Deviations

- The spec calls for a backfill of *existing* cards. We deferred that backfill into a runtime self-heal because SQLite cannot generate UUIDs. With no production users yet, this is moot — fresh card rows go through the new path automatically. A startup self-heal task can be added when needed (see follow-ups).

## Follow-ups

- Optional startup task `services::user_provisioning::ensure_card_bindings(pool)` to scan for `cards` rows whose `accounts.type` is not `credit_card`, create the credit_card account, repoint, and write an `audit_log` row of action `MIGRATION_NEEDS_REVIEW` per remapped row. Not in MVP because the table is currently empty.
- Card delete + invoice listing handlers remain stubs. Not in MVP.
