# 04 — Two-legged transactions

The single most important forward-compat decision. Locks in card / loan / multi-account semantics later without reshaping the core.

## Shape

A transaction is **one row** with two account FKs.

```
transactions
  id
  source_account_id      -- where money leaves
  destination_account_id -- where money arrives
  category_id NULL       -- required for withdrawals & deposits; NULL for transfers
  value INTEGER          -- cents, always positive
  ...
```

Direction is **not** in the sign of `value`. Direction is in the pair of accounts.

## Derived type

Computed (never stored) from the `type` of each leg:

| source.type | dest.type | derived |
|---|---|---|
| `revenue` | `asset` | `deposit` |
| `revenue` | `credit_card` | `deposit` (refund) |
| `asset` | `expense` | `withdrawal` |
| `credit_card` | `expense` | `withdrawal` (card purchase) |
| `asset` | `asset` | `transfer` |
| `asset` | `credit_card` | `transfer` (invoice payment) |
| `asset` | `liability` | `transfer` (loan payment) |
| any other | any other | `INVALID_LEG_PAIR` |

## Account balance

For any account `a`, balance as of date `d`:

```
balance(a, d) =
    a.initial_balance
  + sum(value FROM transactions WHERE destination_account_id = a.id AND tx_date <= d AND deleted_at IS NULL)
  - sum(value FROM transactions WHERE source_account_id      = a.id AND tx_date <= d AND deleted_at IS NULL)
```

For asset / credit_card / liability this is the user-facing balance. For `revenue` and `expense` accounts the absolute value is the period total (income / spend); they are accounting buckets.

## Category semantics

- Withdrawals **must** have a category (it's the whole point: which envelope did the money leave through).
- Deposits should have a category (income source — e.g. "Salary", "VA"). Required at the API layer unless the source is the auto-revenue account from a generic deposit.
- Transfers **must not** have a category. Enforced by service-layer guard + CHECK. Categorising a transfer double-counts the spend.

## Invariants the API enforces

1. `source_account_id != destination_account_id`.
2. Both accounts belong to the same `user_id`.
3. Both accounts share `currency` (multi-currency comes later via `fx_rate`).
4. Derived type is in the allowed table above.
5. Transfer → `category_id IS NULL`. Non-transfer → `category_id IS NOT NULL`.
6. `value > 0`.

## Migration from current schema

The existing schema is single-legged with `transfer_pair_id` (linked-row hack). Migration:

1. Add `source_account_id`, `destination_account_id` (nullable initially).
2. Backfill:
   - `INCOME` rows → `source = user.revenue_account`, `destination = account_id`.
   - `EXPENSE` rows → `source = account_id`, `destination = user.expense_account`.
   - `TRANSFER` rows that have `transfer_pair_id`: pick the row with negative direction as source-side; set `source = its account_id`, `destination = its pair's account_id`; drop the pair row.
3. Set NOT NULL.
4. Drop `transfer_pair_id`, drop the stored `type` CHECK column.
5. Add new indexes on `(source_account_id, tx_date)` and `(destination_account_id, tx_date)`.

Backfill runs inside one transaction. If any user has a malformed `TRANSFER` (no pair, missing counterparty), the migration aborts; surfaces to a CLI subcommand we can run manually to inspect.

## API impact

`POST /transactions` body changes from `{ account_id, type, value, ... }` to `{ source_account_id, destination_account_id, category_id?, value, ... }`. The old client behaviour is **lost**, no shim. Web SPA is built fresh against the new shape.

Convenience: `POST /transactions/transfer` accepts `{ source_account_id, destination_account_id, value, tx_date, description, payment_method }` and asserts both legs are user-owned non-bucket accounts.
