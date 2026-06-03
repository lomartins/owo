# Phase B8 — Balance as of date

Status: shipped (2026-05-27)

## What shipped

`GET /accounts/:id/balance` now accepts an optional `?as_of=YYYY-MM-DD` query. When provided, both subqueries (incoming + outgoing) add `AND tx_date <= ?`. The returned `as_of` field is the user-supplied date when set, otherwise the current ISO timestamp.

The same balance expression is exported via the module-level `BALANCE_EXPR` constant and reused by `list` and `load` for consistency.

## Verification

- `cargo check` clean.

## Follow-ups

- The monthly report (Phase B6) reuses an inlined version of this expression for asset-only carry-over. If we land a `services::accounts::balance_at(date)` Rust function later, both call sites should consume it.
