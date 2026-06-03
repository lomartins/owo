# 06 — Budgets

## Shape

`budgets(user_id, category_id, month, estimated_amount, currency, created_at, updated_at)`.

- `month` is `YYYY-MM` text. Unique on `(user_id, category_id, month)`.
- No `period`, no `start_date`, no `end_date`. Yearly budgets are out of MVP.
- `estimated_amount` is INTEGER cents.
- `spent_amount` and `difference` are **never columns**. They are computed in `GET /budgets`.

## GET /budgets?month=YYYY-MM

Returns one row per category (including ones with no `budgets` row for the month — `estimated = 0`):

```json
{
  "month": "2026-05",
  "items": [
    {
      "category_id": "...",
      "category_name": "Food",
      "budget_id": "..." | null,
      "estimated": "800.00",
      "spent": "612.30",
      "difference": "187.70"
    },
    ...
  ]
}
```

`difference = estimated - spent`. Positive = under budget. Negative = over.

## SQL sketch

```sql
WITH month_bounds AS (
  SELECT
    :month || '-01' AS lo,
    date(:month || '-01', '+1 month') AS hi
),
spent_per_cat AS (
  SELECT
    t.category_id,
    COALESCE(SUM(t.value), 0) AS spent_cents
  FROM transactions t
  JOIN accounts src ON src.id = t.source_account_id
  JOIN accounts dst ON dst.id = t.destination_account_id
  WHERE t.user_id = :user_id
    AND t.deleted_at IS NULL
    AND t.tx_date >= (SELECT lo FROM month_bounds)
    AND t.tx_date <  (SELECT hi FROM month_bounds)
    AND dst.type = 'expense'         -- only withdrawals count as spend
    AND src.type IN ('asset','credit_card')
  GROUP BY t.category_id
)
SELECT
  c.id              AS category_id,
  c.name            AS category_name,
  b.id              AS budget_id,
  COALESCE(b."limit", 0) AS estimated_cents,
  COALESCE(s.spent_cents, 0) AS spent_cents
FROM categories c
LEFT JOIN budgets b
       ON b.category_id = c.id
      AND b.user_id     = :user_id
      AND b.month       = :month
      AND b.deleted_at IS NULL
LEFT JOIN spent_per_cat s ON s.category_id = c.id
WHERE c.user_id = :user_id
  AND c.archived = 0
  AND c.kind IN ('EXPENSE','BOTH')
ORDER BY c.name;
```

Conversion to `Decimal` happens in the handler, not in SQL.

## POST /budgets

Upsert. If a row exists for `(user_id, category_id, month)`, update its `estimated_amount`. Otherwise insert.

Body:

```json
{ "category_id": "...", "month": "2026-05", "estimated_amount": "800.00", "currency": "BRL" }
```

## PUT /budgets/:id

Updates `estimated_amount` only. `category_id` and `month` are immutable (a different cell entirely).

## DELETE /budgets/:id

Soft-delete. Computed `spent` is unaffected (it reads transactions, not budgets).

## Spreadsheet parity check

The user's spreadsheet shows, per category, three columns:

| Estimated | Spent | Difference |
|---|---|---|

`GET /budgets?month=…` returns exactly these three fields per category. Dashboards render them as bars.
