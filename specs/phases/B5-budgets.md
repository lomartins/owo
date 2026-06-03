# Phase B5 — Budgets refactor

Status: shipped (2026-05-27)

## What shipped

- Migration `0005_budgets_month.sql`:
  - Schema becomes `(user_id, category_id, month YYYY-MM, estimated_amount, currency)`.
  - Backfills `month = substr(start_date, 1, 7)`, copies `"limit" → estimated_amount`.
  - Drops `period`, `start_date`, `end_date`.
  - Unique partial index on `(user_id, category_id, month)` where `deleted_at IS NULL`.
  - CHECK enforces `length(month) = 7 AND month[4] = '-'`.
- `domain::budget`:
  - `Budget` carries `month` and `estimated_amount`.
  - `CreateBudget` / `UpdateBudget` introduced.
  - `BudgetRow` and `BudgetMonth` are the read-shapes for the dashboard endpoint.
- `api/budgets.rs`:
  - `GET /budgets?month=YYYY-MM` returns one `BudgetRow` per non-archived expense/both category, including categories with no budget row (estimated = 0). `spent` computed from transactions where `destination.type = 'expense'`, `source.type IN (asset, credit_card)`. `difference = estimated - spent`.
  - `POST /budgets` upserts on `(user_id, category_id, month)`.
  - `PATCH /budgets/:id` updates `estimated_amount` only.
  - `DELETE /budgets/:id` soft-deletes.
- `api/mod.rs`:
  - Registered new routes and schemas.

## Verification

- `cargo build` passes (release-quality linkage).
- Manual round-trip TBD via web SPA.

## Deviations

- None vs `specs/06-budgets.md`.

## Follow-ups

- INCOME-side budgets (e.g. "expect at least X from salary") are not modelled. Out of MVP — spreadsheet doesn't have them either.
