-- Phase A4: extend payment_method enum to include 'VA' (vale-alimentação).
-- Create-new-then-drop-old pattern (see 0002).

PRAGMA foreign_keys = OFF;

CREATE TABLE transactions_new (
    id                       TEXT PRIMARY KEY,
    user_id                  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source_account_id        TEXT NOT NULL REFERENCES accounts(id),
    destination_account_id   TEXT NOT NULL REFERENCES accounts(id),
    category_id              TEXT REFERENCES categories(id),
    payment_method           TEXT NOT NULL CHECK (payment_method IN ('PIX','DEBIT','CREDIT','TED','BOLETO','CASH','VA')),
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

INSERT INTO transactions_new SELECT * FROM transactions;
DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

CREATE INDEX tx_user_date_id ON transactions (user_id, tx_date DESC, id DESC) WHERE deleted_at IS NULL;
CREATE INDEX tx_source_date  ON transactions (source_account_id, tx_date DESC);
CREATE INDEX tx_dest_date    ON transactions (destination_account_id, tx_date DESC);
CREATE INDEX tx_category     ON transactions (category_id);
CREATE INDEX tx_user_updated ON transactions (user_id, updated_at);

PRAGMA foreign_keys = ON;
