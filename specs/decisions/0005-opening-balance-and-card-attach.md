# ADR 0005 — Opening balance as a transaction, credit cards attached to an asset account

Status: Accepted (2026-06-04)

## Context

Three product asks landed together:
1. Accounts must be editable, and their **initial value must itself be a transaction** (so it shows in the ledger and can be edited like any other).
2. A **credit card** should be **attached to an asset account** so the user can preview the open invoice (accrued to date + full cycle) and its due date, and pay it from that account.
3. Credit-card purchases need **installments across months** (parcelas).
4. The dashboard needs a **"money left to spend"** simulation that treats card debt as committed and subtracts this month's unpaid bills.

The pre-existing model: `accounts.initial_balance` (i64 cents) column; two-legged transactions (ADR none, see `specs/04`); cards auto-create their own `credit_card` ledger account (ADR 0003).

## Decision

### Opening balance → `equity` bucket transaction
- New account type **`equity`**: a per-user singleton bucket (like `revenue`/`expense`), hidden from the public account list.
- An account's opening value is the transaction paired with the equity bucket: `equity → account` (positive) or `account → equity` (negative). Derived `kind = "opening"`.
- `accounts.initial_balance` column is kept but always `0`; balance = pure sum of transactions. Create/PATCH `/accounts` upsert the opening transaction via `set_opening_balance`. Migration `0011` converted existing `initial_balance` values into opening transactions.
- `PATCH /accounts` now also edits `currency` and `opening_balance`.

### Credit card "attached to" an asset account
- The card keeps its own `credit_card` ledger account (holds the debt) — preserves clean invoice modelling.
- New column `cards.payment_account_id` records the **asset account it is attached to / paid from**. The UI presents the card nested under that asset account, not as a standalone account.
- `GET /cards/invoice-preview` computes, per card, the open billing cycle from `close_day`/`due_day`: `accrued` (charges up to today), `cycle_total` (whole cycle incl. future-dated installments), `due_date`, `outstanding` (all charges − payments), and `limit`.

### Installments
- `POST /transactions` accepts `installments: N`. When `N > 1` it writes N rows sharing an `installment_group_id`, one per month (`tx_date + k months`), value split evenly with the remainder on the first, `installment_number`/`installment_count` set.
- `DELETE /transactions/{id}?scope=following` soft-deletes this row and every later installment in the group (`installment_number >= this`). Default `scope=this`.

### Spendable simulation
- `GET /reports/spendable?month=` returns `asset_total − card_outstanding − pending_bills = spendable`, where `card_outstanding` is the magnitude of `credit_card + liability` debt and `pending_bills` is unpaid bills (override-aware) for the month.

## Why

- "Initial value is a transaction" is satisfied literally without a schema-wide rewrite: the balance formula already sums transactions, so routing the opening value through an `equity` leg keeps every existing query correct (column stays `0`).
- Keeping the card's own ledger account preserves the invoice/debt math the preview needs, while `payment_account_id` delivers the requested "attached to an account" UX.
- N real rows for installments keeps each parcela independently visible, editable, and exact; the group id enables the "exclude the following ones too" flow.

## Trade-offs

- `equity` is a fourth hidden bucket type; provisioning + the leg-pair validator + the `kind` expression all had to learn about it.
- Net-worth still uses its prior formula and is out of scope here.
- The card is, under the hood, still a separate ledger account even though the UI calls it a sub-ledger of the asset account.

## SQLite migration note

Adding `equity` to the `accounts` type CHECK required the create-new-then-drop-old rewrite (CLAUDE.md rule 5). The `0007` invoice-payment triggers reference `accounts`, so `0011` drops them before the rewrite and recreates them after.
