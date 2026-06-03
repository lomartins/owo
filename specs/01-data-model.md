# 01 — Data model

Storage: SQLite (current). Postgres is the future target; design choices avoid lock-in. All money columns are `INTEGER` cents in storage and `Decimal` at the API boundary (see `03-money.md`).

## Entities

### account

Anywhere money sits, enters, or leaves.

| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUID. |
| `user_id` | TEXT FK | Owner. |
| `name` | TEXT | Human label. |
| `type` | TEXT | One of `asset`, `credit_card`, `liability`, `revenue`, `expense`. See `05-account-types.md`. |
| `currency` | TEXT | Default `BRL`. |
| `initial_balance` | INTEGER | Cents at account creation. Always 0 for `revenue` / `expense` / `credit_card` / `liability`. |
| `archived` | INTEGER (bool) | Soft hide. |
| `created_at`, `updated_at`, `deleted_at` | TEXT | ISO-8601. |
| `sync_version`, `device_id` | — | Existing sync fields, kept. |

Per user, the system auto-provisions on register:
- one `revenue` account ("Income") — destination of all deposits.
- one `expense` account ("Expense") — destination of all withdrawals.

These two are the destination legs that the spreadsheet model implicitly used.

### transaction

One movement of money with **two legs**. See `04-two-legged-transactions.md` for full derivation rules.

| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `user_id` | TEXT FK | |
| `source_account_id` | TEXT FK accounts | Where the money leaves. |
| `destination_account_id` | TEXT FK accounts | Where it arrives. |
| `category_id` | TEXT FK categories NULL | NULL only for transfers between own accounts (incl. invoice payments). |
| `payment_method` | TEXT | `PIX, CASH, BOLETO, CARD, VA, DEBIT, CREDIT, TED`. CHECK enforced. |
| `value` | INTEGER | Cents. Always positive; direction is encoded in source/destination. |
| `currency` | TEXT | |
| `fx_rate` | TEXT NULL | Reserved for multi-currency. |
| `description` | TEXT | |
| `tx_date` | TEXT | YYYY-MM-DD. |
| `paid` | INTEGER (bool) | Mirrors the spreadsheet "paid" flag for variable expenses and fixed bills. |
| `card_id` | TEXT FK cards NULL | Metadata pointer when source is a `credit_card` account. |
| `invoice_id` | TEXT FK card_invoices NULL | When this tx is the invoice payment, or belongs to an invoice. |
| `bill_id` | TEXT FK bills NULL | If posted from a fixed expense. |
| `receipt_url`, `picture_url` | TEXT NULL | |
| `created_at`, `updated_at`, `deleted_at` | TEXT | |
| `sync_version`, `device_id` | — | |

Derived `type` (not stored):
- `deposit` — source is `revenue`, destination is `asset` (or `credit_card` for refunds).
- `withdrawal` — source is `asset` (or `credit_card`), destination is `expense`.
- `transfer` — source and destination are both user accounts (`asset`, `credit_card`, `liability`). No category.

### category

| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `user_id` | TEXT FK | |
| `name` | TEXT | |
| `kind` | TEXT | `INCOME` / `EXPENSE` / `BOTH`. Kept from current schema. |
| `archived` | INTEGER | |

Seed on register (kind = EXPENSE unless noted): Food, Leisure, Transport, Health, Education, Clothes, Home, Pet, Subscriptions, Other.

### budget

| Field | Type | Notes |
|---|---|---|
| `id` | TEXT PK | |
| `user_id` | TEXT FK | |
| `category_id` | TEXT FK categories | |
| `month` | TEXT | YYYY-MM. Unique on `(user_id, category_id, month)`. |
| `estimated_amount` | INTEGER | Cents. |
| `currency` | TEXT | |
| `created_at`, `updated_at` | TEXT | |

`spent` and `difference` are **never stored**. They are SQL aggregates at read time: see `06-budgets.md`.

### payment_method

Enum CHECK constraint (no separate table for MVP): `PIX, CASH, BOLETO, CARD, VA, DEBIT, CREDIT, TED`.

`VA` (vale-alimentação) is income tied to spending categories; it stays an enum value rather than a special account type.

### cards / card_invoices

Kept as-is from existing schema. Now each `cards.account_id` points to a `credit_card`-type `accounts` row (see migration in phase C). Card-specific metadata (last 4 digits, brand, limit, close/due days) lives in `cards`. Statements live in `card_invoices`.

Out of MVP: `installment_plan` table. Stays a future addition once the foundations are in place.

### fixed expenses

Modeled via the existing `bills` table — `bills` has `value` (estimated), `paid`, `paid_at`, `paid_transaction_id`, `recurrence`. Decision: keep this. See `00-mvp-scope.md`. The "actual" is whatever transaction the bill points at via `paid_transaction_id`.

### Other tables (preserve, do not delete)

`investments`, `loans`, `goals`, `sync_state`, `audit_log`, `tags`, `transaction_tags`, `sessions`, `users` — all stay as-is. They are forward-compat infrastructure or already-built support.

## Indexes (preserved)

- `tx_user_date_id (user_id, tx_date DESC, id DESC) WHERE deleted_at IS NULL`
- `tx_account_date (account_id, tx_date DESC)` — to be replaced by index on `(source_account_id, tx_date)` and `(destination_account_id, tx_date)` after migration.
- `tx_category (category_id)`

## Atomicity

- Any operation that writes more than one row uses `pool.begin()`.
- A future installment plan (out of MVP) will insert N rows in one transaction.
- The two-legged transaction model is **one row**, not two — so a single insert is atomic by definition. No twin-row drift.
