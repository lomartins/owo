# 08 — Cards: preserve and verify

Existing card code stays. **Do not delete.** This document defines the verification rules and the minimal adjustments needed to make it spec-correct without rebuilding it.

## Rules to verify

1. **Card purchases preserve their real category.** A grocery buy on a card stays `Food`, not `Other`. Enforced by the same `category_id IS NOT NULL` rule that applies to all withdrawals.
2. **Source leg of a card purchase is a `credit_card` account.** Not the checking account.
3. **Invoice payment is a category-less transfer.** Asset → credit_card. `category_id IS NULL` enforced.
4. **Installments**: out of MVP; foundation must keep it cheap to add.
5. **Money is Decimal at API boundary, INTEGER cents in storage.** Same rule as everything else.
6. **Atomicity**: any multi-row write (future: installment generator) wraps in `pool.begin()`.

## Adjustments

### C9 — auto-create credit_card account per card

Migration `0006_card_account_binding.sql`:

1. For each existing row in `cards`:
   - INSERT INTO accounts a row of `type = 'credit_card'`, `name = cards.brand || ' ' || cards.last_four_digits`, `currency = cards.currency` (fall back to user default), `initial_balance = 0`, `archived = cards.archived`.
   - UPDATE `cards.account_id` to point at the new row.
   - Save mapping (old `account_id` → new `credit_card` account_id) in a temp table.
2. For every transaction whose `card_id IS NOT NULL` and whose current `account_id` is the old (asset) account:
   - Set `source_account_id` = new credit_card account.
   - Set `destination_account_id` = user's `expense` account (since these are card purchases).
   - Leave `card_id` alone (it stays as metadata pointing at the card row).
3. For every transaction that is an invoice payment (`invoice_id IS NOT NULL` and was previously a transfer):
   - Set `source_account_id` = checking (the user's asset account that paid the invoice — taken from `card_invoices.paid_transaction_id` lookup if available; otherwise from the user's first asset account; otherwise leave the migration row flagged for manual review).
   - Set `destination_account_id` = the new credit_card account.
   - Force `category_id = NULL`.

If any user has invoice payments the migration cannot disambiguate, the migration succeeds but writes those tx ids to `audit_log` with action `MIGRATION_NEEDS_REVIEW`. We surface them in `GET /audit?action=MIGRATION_NEEDS_REVIEW`.

### C10 — invoice-payment guard

A service-layer check + SQL CHECK:

- API: `POST /transactions` (and the bills/cards code paths) reject `category_id IS NOT NULL` when destination is a `credit_card` account and source is an `asset` account.
- DB: trigger:
  ```sql
  CREATE TRIGGER trg_tx_invoice_payment_no_category
  BEFORE INSERT ON transactions
  FOR EACH ROW
  WHEN NEW.category_id IS NOT NULL
       AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.destination_account_id AND type = 'credit_card')
       AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.source_account_id      AND type = 'asset')
  BEGIN
    SELECT RAISE(ABORT, 'INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY');
  END;
  ```

A symmetric trigger on UPDATE.

## What stays untouched

- `cards` schema (brand, last_four_digits, close_day, due_day, limit) — unchanged.
- `card_invoices` schema — unchanged.
- Card listing / creation API surface — unchanged signature; the implementation now also creates the underlying `credit_card` account.

## What this enables later (still out of MVP)

- `installment_plan` table feeding N transactions sharing a category, all sourced from the same `credit_card` account, generated atomically.
- A "committed future spending" view that sums future-dated card-sourced transactions.
- Per-card balance via the standard account-balance query (no card-specific logic).
