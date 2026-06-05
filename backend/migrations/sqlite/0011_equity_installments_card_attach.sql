-- Phase F: editable accounts, opening-balance-as-transaction, credit-card
-- attach + installments.
--
-- 1. Add the `equity` account type (opening-balance bucket, singleton per user,
--    hidden from the user-facing list like revenue/expense).
-- 2. Provision an equity bucket for every existing user.
-- 3. Convert each account's initial_balance into a real "Opening balance"
--    transaction (equity -> account, or account -> equity for negatives), then
--    zero the column. Balance is now purely the sum of transactions, so the
--    opening value shows in the ledger and can be edited like any transaction.
-- 4. Add installment grouping columns to transactions.
-- 5. Add cards.payment_account_id (the asset account a card is attached to /
--    paid from). The card keeps its own credit_card ledger account.
--
-- The accounts rewrite uses the create-new-with-temp-name → drop-old → rename
-- pattern (see 0002) so no FK in cards/transactions/etc. is rewritten.

-- The accounts table rewrite below drops + recreates `accounts`, which is
-- referenced by FKs from cards/transactions/bills/etc. This is only safe with
-- foreign-key enforcement OFF. `PRAGMA foreign_keys` is a no-op inside a
-- transaction (and sqlx wraps each migration in one), so enforcement is disabled
-- at the connection level by db::run_migrations before the transaction begins.
-- The PRAGMA below is kept for parity with sibling migrations / direct sqlite3 use.
PRAGMA foreign_keys = OFF;

-- The 0007 triggers reference `accounts`; the table rewrite below briefly drops
-- it. Drop the triggers first and recreate them at the end.
DROP TRIGGER IF EXISTS trg_tx_invoice_payment_no_category_insert;
DROP TRIGGER IF EXISTS trg_tx_invoice_payment_no_category_update;

------------------------------------------------------------
-- 1. accounts: extend type CHECK with 'equity'
------------------------------------------------------------

CREATE TABLE accounts_new (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    type              TEXT NOT NULL CHECK (type IN ('asset','credit_card','liability','revenue','expense','equity')),
    currency          TEXT NOT NULL,
    initial_balance   INTEGER NOT NULL DEFAULT 0,
    archived          INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT,
    sync_version      INTEGER NOT NULL DEFAULT 1,
    device_id         TEXT NOT NULL
);

INSERT INTO accounts_new
SELECT id, user_id, name, type, currency, initial_balance, archived,
       created_at, updated_at, deleted_at, sync_version, device_id
FROM accounts;

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;

CREATE INDEX accounts_user_updated ON accounts (user_id, updated_at);
CREATE UNIQUE INDEX accounts_user_singleton_revenue
    ON accounts(user_id) WHERE type = 'revenue' AND deleted_at IS NULL;
CREATE UNIQUE INDEX accounts_user_singleton_expense
    ON accounts(user_id) WHERE type = 'expense' AND deleted_at IS NULL;
CREATE UNIQUE INDEX accounts_user_singleton_equity
    ON accounts(user_id) WHERE type = 'equity' AND deleted_at IS NULL;

------------------------------------------------------------
-- 2. provision an equity bucket per user
------------------------------------------------------------

INSERT INTO accounts (id, user_id, name, type, currency, initial_balance,
                      archived, created_at, updated_at, device_id)
SELECT
    lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(6))),
    u.id, 'Opening balance', 'equity', u.default_currency, 0, 0,
    strftime('%Y-%m-%dT%H:%M:%SZ','now'),
    strftime('%Y-%m-%dT%H:%M:%SZ','now'),
    'server'
FROM users u
WHERE u.deleted_at IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM accounts a
    WHERE a.user_id = u.id AND a.type = 'equity' AND a.deleted_at IS NULL
  );

------------------------------------------------------------
-- 3. initial_balance -> opening transaction, then zero the column
------------------------------------------------------------

INSERT INTO transactions (
    id, user_id, source_account_id, destination_account_id, category_id,
    payment_method, value, currency, description, tx_date, paid,
    created_at, updated_at, device_id
)
SELECT
    lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(6))),
    a.user_id,
    CASE WHEN a.initial_balance >= 0
         THEN (SELECT e.id FROM accounts e WHERE e.user_id = a.user_id AND e.type = 'equity' AND e.deleted_at IS NULL)
         ELSE a.id END,
    CASE WHEN a.initial_balance >= 0
         THEN a.id
         ELSE (SELECT e.id FROM accounts e WHERE e.user_id = a.user_id AND e.type = 'equity' AND e.deleted_at IS NULL) END,
    NULL,
    'CASH',
    abs(a.initial_balance),
    a.currency,
    'Opening balance',
    substr(a.created_at, 1, 10),
    1,
    strftime('%Y-%m-%dT%H:%M:%SZ','now'),
    strftime('%Y-%m-%dT%H:%M:%SZ','now'),
    'server'
FROM accounts a
WHERE a.initial_balance <> 0
  AND a.type IN ('asset','credit_card','liability');

UPDATE accounts SET initial_balance = 0
WHERE type IN ('asset','credit_card','liability') AND initial_balance <> 0;

------------------------------------------------------------
-- 4. installment grouping on transactions
------------------------------------------------------------

ALTER TABLE transactions ADD COLUMN installment_group_id TEXT;
ALTER TABLE transactions ADD COLUMN installment_number   INTEGER;
ALTER TABLE transactions ADD COLUMN installment_count    INTEGER;
CREATE INDEX tx_installment_group ON transactions (installment_group_id);

------------------------------------------------------------
-- 5. card payment (funding) account
------------------------------------------------------------

ALTER TABLE cards ADD COLUMN payment_account_id TEXT REFERENCES accounts(id);

------------------------------------------------------------
-- recreate the 0007 invoice-payment guard triggers
------------------------------------------------------------

CREATE TRIGGER trg_tx_invoice_payment_no_category_insert
BEFORE INSERT ON transactions
FOR EACH ROW
WHEN NEW.category_id IS NOT NULL
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.destination_account_id AND type = 'credit_card')
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.source_account_id      AND type = 'asset')
BEGIN
    SELECT RAISE(ABORT, 'INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY');
END;

CREATE TRIGGER trg_tx_invoice_payment_no_category_update
BEFORE UPDATE OF category_id, source_account_id, destination_account_id ON transactions
FOR EACH ROW
WHEN NEW.category_id IS NOT NULL
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.destination_account_id AND type = 'credit_card')
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.source_account_id      AND type = 'asset')
BEGIN
    SELECT RAISE(ABORT, 'INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY');
END;

PRAGMA foreign_keys = ON;
