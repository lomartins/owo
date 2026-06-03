# Phase A3 — Account types

Status: shipped (2026-05-27)

## What shipped

- Migration `0002_account_types.sql`:
  - Rewrote `accounts.type` CHECK constraint to `('asset','credit_card','liability','revenue','expense')`.
  - Mapped legacy `BANK / PAYMENT / CASH → asset`.
  - Added unique partial indexes enforcing one `revenue` and one `expense` bucket per user (where `deleted_at IS NULL`).
- `services/user_provisioning.rs`:
  - `provision_user(conn, user_id, currency, device_id)` — idempotent.
  - Inserts a `revenue` bucket ("Income") and an `expense` bucket ("Expense") per user.
  - Seeds the 10 MVP categories (Food, Leisure, Transport, Health, Education, Clothes, Home, Pet, Subscriptions, Other) as kind `EXPENSE`. (Also covers Phase B7.)
- `api/auth.rs::register` wraps the user-insert + provisioning in a single `pool.begin()` transaction.
- `api/accounts.rs`:
  - `POST /accounts` rejects non-`asset` types with `RESERVED_TYPE`.
  - `GET /accounts` filters out `revenue`/`expense` buckets.
  - `PUT` / `DELETE` reject mutations on bucket accounts.

## Verification

- `cargo check` clean.
- New unique partial indexes ensure DB-level singleton.
- The provisioning function is idempotent — safe to invoke as a self-heal startup hook later.

## Deviations

- None vs `specs/05-account-types.md`.

## Follow-ups

- The Phase C9 card-binding migration will create `credit_card` accounts via an internal flow; the public API stays restricted to `asset`.
- Optional: add a startup self-heal that scans existing users and ensures buckets exist (covers DB rows imported from before this phase).
