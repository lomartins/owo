# Phase B6 — Monthly report

Status: shipped (2026-05-27)

## What shipped

- `GET /api/v1/reports/monthly?month=YYYY-MM` (replaces the 501 stubs for cash-flow/by-category/net-worth — those endpoints remain registered as stubs for future phases).
- Returns `{ month, income_total, spent_total, budget_balance, carry_over_in, carry_over_out }`.
- Income: sum of `value` where `source.type = 'revenue'` within `[month-01, next-month-01)`.
- Spent: sum where `destination.type = 'expense'`.
- Budget balance: `sum(estimated_amount) - spent` for the month.
- Carry-over in/out: sum of asset balances at the month-open and month-close timestamps, computed via the standard two-legged balance expression filtered on `tx_date < boundary`.
- Implementation in `api/reports.rs::monthly` + helper `asset_balances_at`.

## Verification

- `cargo build` clean.
- Manual end-to-end TBD via web SPA.

## Deviations

- One round-trip in the spec (single SQL with CTE) is split into 4 cheap queries in the implementation. Cleaner Rust; SQLite handles them all in micros. If a SQL profiler later flags this, collapse to one CTE.

## Follow-ups

- The other reports endpoints (cash-flow, by-category, net-worth) stay 501. They are not in MVP — re-enable when Phase 1.5 lands.
