-- Phase A2: two-legged transactions.
-- Create-new-then-drop-old pattern (see 0002).
-- See specs/04-two-legged-transactions.md.

PRAGMA foreign_keys = OFF;

CREATE TABLE transactions_new (
    id                       TEXT PRIMARY KEY,
    user_id                  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source_account_id        TEXT NOT NULL REFERENCES accounts(id),
    destination_account_id   TEXT NOT NULL REFERENCES accounts(id),
    category_id              TEXT REFERENCES categories(id),
    payment_method           TEXT NOT NULL CHECK (payment_method IN ('PIX','DEBIT','CREDIT','TED','BOLETO','CASH')),
    value                    INTEGER NOT NULL CHECK (value > 0),
    currency                 TEXT NOT NULL,
    fx_rate                  TEXT,
    description              TEXT NOT NULL,
    tx_date                  TEXT NOT NULL,
    paid                     INTEGER NOT NULL DEFAULT 1,
    receipt_url              TEXT,
    picture_url              TEXT,
    card_id                  TEXT REFERENCES cards(id),
    bill_id                  TEXT,
    invoice_id               TEXT REFERENCES card_invoices(id),
    created_at               TEXT NOT NULL,
    updated_at               TEXT NOT NULL,
    deleted_at               TEXT,
    sync_version             INTEGER NOT NULL DEFAULT 1,
    device_id                TEXT NOT NULL,
    CHECK (source_account_id <> destination_account_id)
);

-- Best-effort backfill from the legacy single-legged schema. For early-dev
-- empty state this is a no-op; if rows exist and lack a derivable leg pair
-- (TRANSFER without bucket lookup), the INSERT aborts on NOT NULL.
INSERT INTO transactions_new (
    id, user_id, source_account_id, destination_account_id, category_id,
    payment_method, value, currency, fx_rate, description, tx_date, paid,
    receipt_url, picture_url, card_id, bill_id, invoice_id,
    created_at, updated_at, deleted_at, sync_version, device_id
)
SELECT
    t.id, t.user_id,
    CASE t.type
        WHEN 'INCOME'  THEN (SELECT id FROM accounts WHERE user_id = t.user_id AND type = 'revenue' AND deleted_at IS NULL LIMIT 1)
        WHEN 'EXPENSE' THEN t.account_id
    END,
    CASE t.type
        WHEN 'INCOME'  THEN t.account_id
        WHEN 'EXPENSE' THEN (SELECT id FROM accounts WHERE user_id = t.user_id AND type = 'expense' AND deleted_at IS NULL LIMIT 1)
    END,
    t.category_id,
    t.payment_method, t.value, t.currency, t.fx_rate, t.description, t.tx_date,
    1,
    t.receipt_url, t.picture_url, t.card_id, t.bill_id, t.invoice_id,
    t.created_at, t.updated_at, t.deleted_at, t.sync_version, t.device_id
FROM transactions t
WHERE t.type IN ('INCOME', 'EXPENSE');

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

CREATE INDEX tx_user_date_id ON transactions (user_id, tx_date DESC, id DESC) WHERE deleted_at IS NULL;
CREATE INDEX tx_source_date  ON transactions (source_account_id, tx_date DESC);
CREATE INDEX tx_dest_date    ON transactions (destination_account_id, tx_date DESC);
CREATE INDEX tx_category     ON transactions (category_id);
CREATE INDEX tx_user_updated ON transactions (user_id, updated_at);

PRAGMA foreign_keys = ON;
