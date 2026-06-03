# 02 — API

REST + JSON. All paths prefixed `/api/v1`. Auth = Bearer token (existing scheme).

All amounts in request/response bodies are JSON strings holding a `Decimal` (e.g. `"1234.56"`), not numbers, not cents. See `03-money.md`.

## MVP surface

### Accounts

- `GET  /accounts` — list. Each item includes `current_balance` computed as `initial_balance + sum(deposits) - sum(withdrawals)` over non-deleted transactions.
- `POST /accounts` — create.
- `GET  /accounts/:id` — show, with `current_balance`.
- `PUT  /accounts/:id` — update.
- `DELETE /accounts/:id` — soft-delete.
- `GET  /accounts/:id/balance?as_of=YYYY-MM-DD` — balance as of date. Optional `as_of` filter on `tx_date <= as_of`.

### Transactions

- `GET  /transactions?month=YYYY-MM` — list, filterable by month (`tx_date BETWEEN month-01 AND month-end`), by `category_id`, by `account_id` (matches either leg). Cursor pagination preserved.
- `POST /transactions` — create. Body: `{ source_account_id, destination_account_id, category_id?, payment_method, value (Decimal), currency, description, tx_date, paid?, card_id?, invoice_id?, bill_id? }`. Single row insert.
- `PUT  /transactions/:id` — update (same body shape, optional fields).
- `DELETE /transactions/:id` — soft-delete.
- `POST /transactions/transfer` — convenience wrapper for transfers (no category, asserts both legs are user-owned accounts).

### Budgets

- `GET  /budgets?month=YYYY-MM` — returns one row per category with `{ category_id, name, estimated, spent, difference }`. Includes categories with no budget row yet (estimated = 0).
- `POST /budgets` — body: `{ category_id, month, estimated_amount }`. Upserts on `(user_id, category_id, month)`.
- `PUT  /budgets/:id` — update `estimated_amount`.
- `DELETE /budgets/:id` — remove a budget row.

### Categories

- `GET  /categories` — list (filter `archived`).
- `POST /categories` — create.
- `PUT  /categories/:id` — update.
- `DELETE /categories/:id` — archive (soft).

### Reports

- `GET /reports/monthly?month=YYYY-MM` — see `07-monthly-report.md`. Returns:
  ```json
  {
    "month": "2026-05",
    "income_total": "5400.00",
    "spent_total": "3210.45",
    "budget_balance": "189.55",
    "carry_over_in": "1230.00",
    "carry_over_out": "3419.55"
  }
  ```

## Preserved endpoints (not built or extended in MVP)

Existing endpoints for `cards`, `bills`, `loans`, `investments`, `goals`, `sync`, `audit`, `backup`, `tags` stay registered. Any 501 stub stays 501. Card endpoints are touched only in Phase C (preserve-and-verify): see `08-cards-preserve.md`.

## Errors

Existing `ApiError` shape stands. New cases:
- `INVALID_LEG_PAIR` — source and destination accounts produce no valid derived type (e.g. `revenue → revenue`).
- `INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY` — invoice-payment transactions must not carry a `category_id`.
- `MONTH_FORMAT` — `month` query/param is not `YYYY-MM`.

## Static SPA

Axum mounts the built SPA at `/` from `web/dist/`. API stays under `/api/v1`. Single port, single binary.
