# Phase A4 — Payment methods: add VA

Status: shipped (2026-05-27)

## What shipped

- Migration `0004_payment_method_va.sql`: rewrote `transactions.payment_method` CHECK to include `'VA'` (vale-alimentação). SQLite cannot ALTER CHECK in place; the migration recreates the table preserving rows.
- Domain types unchanged — `payment_method` was already a free-form String at the Rust layer; only the DB constraint changed.

## Verification

- `cargo check` clean.

## Deviations

- DEBIT, CREDIT, TED kept in the enum even though `01-data-model.md` mentions an optional collapse to `CARD`. The collapse can happen later without breaking anything; leaving the granular set avoids data loss for any existing rows.

## Follow-ups

- Web SPA should still surface only `{PIX, CASH, BOLETO, CARD, VA}` as the user-facing chip set. The extra DB-level values (`DEBIT`, `CREDIT`, `TED`) are accepted but not advertised.
