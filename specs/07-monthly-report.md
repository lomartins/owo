# 07 — Monthly report

`GET /reports/monthly?month=YYYY-MM` returns the spreadsheet's monthly summary.

## Response

```json
{
  "month": "2026-05",
  "income_total":   "5400.00",
  "spent_total":    "3210.45",
  "budget_balance": "189.55",
  "carry_over_in":  "1230.00",
  "carry_over_out": "3419.55"
}
```

## Definitions

- `income_total` = sum of `value` over all non-deleted transactions in the month whose **destination** is an `asset` (or `credit_card`, for refunds) and **source** is `revenue`.
- `spent_total` = sum of `value` over all non-deleted transactions in the month whose **destination** is `expense`.
- `budget_balance` = `sum(budgets.estimated_amount) - spent_total` for the month. Positive = under budget.
- `carry_over_in` = sum of `current_balance` over all asset accounts as of `month-01 - 1 day`.
- `carry_over_out` = sum of `current_balance` over all asset accounts as of `month_end`.

Carry-over uses **asset** accounts only. `credit_card` (negative liability) and `liability` are excluded from carry-over so that a maxed-out card doesn't make the user "richer" next month.

## SQL sketch

```sql
WITH m AS (
  SELECT
    :month || '-01' AS lo,
    date(:month || '-01', '+1 month') AS hi
),
income AS (
  SELECT COALESCE(SUM(t.value), 0) AS v
  FROM transactions t
  JOIN accounts s ON s.id = t.source_account_id
  WHERE t.user_id = :user_id AND t.deleted_at IS NULL
    AND t.tx_date >= (SELECT lo FROM m) AND t.tx_date < (SELECT hi FROM m)
    AND s.type = 'revenue'
),
spent AS (
  SELECT COALESCE(SUM(t.value), 0) AS v
  FROM transactions t
  JOIN accounts d ON d.id = t.destination_account_id
  WHERE t.user_id = :user_id AND t.deleted_at IS NULL
    AND t.tx_date >= (SELECT lo FROM m) AND t.tx_date < (SELECT hi FROM m)
    AND d.type = 'expense'
),
budget_total AS (
  SELECT COALESCE(SUM("limit"), 0) AS v
  FROM budgets
  WHERE user_id = :user_id AND month = :month AND deleted_at IS NULL
),
carry_in AS (
  SELECT COALESCE(SUM(
    a.initial_balance
    + COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = a.id AND tx_date < (SELECT lo FROM m) AND deleted_at IS NULL), 0)
    - COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id      = a.id AND tx_date < (SELECT lo FROM m) AND deleted_at IS NULL), 0)
  ), 0) AS v
  FROM accounts a
  WHERE a.user_id = :user_id AND a.type = 'asset' AND a.archived = 0 AND a.deleted_at IS NULL
),
carry_out AS (
  SELECT COALESCE(SUM(
    a.initial_balance
    + COALESCE((SELECT SUM(value) FROM transactions WHERE destination_account_id = a.id AND tx_date < (SELECT hi FROM m) AND deleted_at IS NULL), 0)
    - COALESCE((SELECT SUM(value) FROM transactions WHERE source_account_id      = a.id AND tx_date < (SELECT hi FROM m) AND deleted_at IS NULL), 0)
  ), 0) AS v
  FROM accounts a
  WHERE a.user_id = :user_id AND a.type = 'asset' AND a.archived = 0 AND a.deleted_at IS NULL
)
SELECT
  (SELECT v FROM income) AS income_cents,
  (SELECT v FROM spent)  AS spent_cents,
  (SELECT v FROM budget_total) - (SELECT v FROM spent) AS budget_balance_cents,
  (SELECT v FROM carry_in)  AS carry_in_cents,
  (SELECT v FROM carry_out) AS carry_out_cents;
```

One round trip. Handler converts cents → `Decimal`.

## Edge cases

- Month with no transactions: all zeros except `carry_over_in` / `carry_over_out`.
- New user, no asset accounts: carry-over zeros.
- Future month: report still computes but carry-over uses today's transactions and may not reflect the user's intent. Frontend warns when `month > current month`.
