# 05 — Account types

```
type IN ('asset', 'credit_card', 'liability', 'revenue', 'expense')
```

| Type | Active in MVP? | Meaning |
|---|---|---|
| `asset` | yes | Checking, savings, cash, wallet, payment apps. |
| `revenue` | yes | System bucket. One per user. Source of all deposits. |
| `expense` | yes | System bucket. One per user. Destination of all withdrawals. |
| `credit_card` | reserved | Used in phase C (preserve-and-verify). One per existing card row. |
| `liability` | reserved | Loans. Not built in MVP. |

## Why the buckets

The two-legged model needs **both** legs to be real accounts. A withdrawal isn't "money disappears" — it's "money leaves an asset and lands in an `expense` bucket". The `expense` and `revenue` accounts are the accounting counterparties.

In the UI the buckets are invisible: dashboards show "income" as `sum(deposits into asset)` and "spend" as `sum(withdrawals from asset/credit_card)`. The bucket accounts never appear in the user-facing accounts list.

## Provisioning

On `POST /auth/register`, the same transaction that creates the `users` row inserts:
- one `revenue` account named `"Income"`,
- one `expense` account named `"Expense"`.

Both with `archived = false`, `initial_balance = 0`. Their `id` is stable per user; the API stores them as `state.revenue_account_id(user_id)` and `state.expense_account_id(user_id)`, fetched on demand (cached per request).

## Migration of existing accounts

Existing rows have `type IN ('BANK', 'PAYMENT', 'CASH')`. Migration `0004_account_types.sql`:

1. Rewrite the CHECK constraint to the new enum.
2. `UPDATE accounts SET type = 'asset' WHERE type IN ('BANK', 'PAYMENT', 'CASH');`
3. For each existing user, INSERT `revenue` and `expense` accounts if not present.

## Constraints

- Only one `revenue` and one `expense` account per user. Enforced by unique partial index:
  ```sql
  CREATE UNIQUE INDEX accounts_user_singleton_revenue
    ON accounts(user_id) WHERE type = 'revenue' AND deleted_at IS NULL;
  CREATE UNIQUE INDEX accounts_user_singleton_expense
    ON accounts(user_id) WHERE type = 'expense' AND deleted_at IS NULL;
  ```
- `revenue` and `expense` accounts are not deletable and not user-renamable via API in MVP.
- `credit_card` accounts are created only by the card-creation path (phase C). They are not creatable through `POST /accounts` in MVP — the API rejects with `RESERVED_TYPE`.
