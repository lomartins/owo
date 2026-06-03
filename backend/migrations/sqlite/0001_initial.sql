-- owo v1 — consolidated schema (SQLite).
-- See docs/DATA_MODEL.md and docs/adr/0001-sync-strategy.md.

PRAGMA foreign_keys = ON;

------------------------------------------------------------
-- Users + auth
------------------------------------------------------------

CREATE TABLE users (
    id                  TEXT PRIMARY KEY,
    email               TEXT NOT NULL UNIQUE,
    password_hash       TEXT NOT NULL,
    display_name        TEXT NOT NULL,
    default_currency    TEXT NOT NULL,
    locale              TEXT NOT NULL,
    is_admin            INTEGER NOT NULL DEFAULT 0,
    failed_login_count  INTEGER NOT NULL DEFAULT 0,
    lockout_until       TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT,
    sync_version        INTEGER NOT NULL DEFAULT 1,
    device_id           TEXT NOT NULL DEFAULT 'server'
);

CREATE TABLE sessions (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      BLOB NOT NULL UNIQUE,
    device_id       TEXT NOT NULL,
    device_name     TEXT NOT NULL,
    user_agent      TEXT,
    ip_last_seen    TEXT,
    created_at      TEXT NOT NULL,
    last_used_at    TEXT NOT NULL,
    expires_at      TEXT NOT NULL,
    revoked_at      TEXT
);
CREATE INDEX sessions_user_revoked ON sessions (user_id, revoked_at);

------------------------------------------------------------
-- Core entities
------------------------------------------------------------

CREATE TABLE accounts (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    type              TEXT NOT NULL CHECK (type IN ('BANK','PAYMENT','CASH')),
    currency          TEXT NOT NULL,
    initial_balance   INTEGER NOT NULL DEFAULT 0,
    archived          INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT,
    sync_version      INTEGER NOT NULL DEFAULT 1,
    device_id         TEXT NOT NULL
);
CREATE INDEX accounts_user_updated ON accounts (user_id, updated_at);

CREATE TABLE cards (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id        TEXT NOT NULL REFERENCES accounts(id),
    last_four_digits  TEXT NOT NULL,
    brand             TEXT NOT NULL CHECK (brand IN ('VISA','MASTERCARD','ELO','AMEX','OTHER')),
    type              TEXT NOT NULL CHECK (type IN ('CREDIT','DEBIT')),
    "limit"           INTEGER,
    close_day         INTEGER,
    due_day           INTEGER,
    archived          INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT,
    sync_version      INTEGER NOT NULL DEFAULT 1,
    device_id         TEXT NOT NULL
);
CREATE INDEX cards_user_updated ON cards (user_id, updated_at);

CREATE TABLE card_invoices (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    card_id             TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    period_start        TEXT NOT NULL,
    period_end          TEXT NOT NULL,
    due_date            TEXT NOT NULL,
    total               INTEGER NOT NULL DEFAULT 0,
    paid_at             TEXT,
    paid_transaction_id TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT,
    sync_version        INTEGER NOT NULL DEFAULT 1,
    device_id           TEXT NOT NULL
);
CREATE INDEX card_invoices_card_period ON card_invoices (card_id, period_end);

CREATE TABLE categories (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    parent_id     TEXT REFERENCES categories(id),
    icon          TEXT,
    color         TEXT,
    kind          TEXT NOT NULL CHECK (kind IN ('INCOME','EXPENSE','BOTH')),
    archived      INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT,
    sync_version  INTEGER NOT NULL DEFAULT 1,
    device_id     TEXT NOT NULL
);
CREATE INDEX categories_user_updated ON categories (user_id, updated_at);

CREATE TABLE tags (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    color         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT,
    sync_version  INTEGER NOT NULL DEFAULT 1,
    device_id     TEXT NOT NULL
);
CREATE INDEX tags_user_updated ON tags (user_id, updated_at);

CREATE TABLE transactions (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id          TEXT NOT NULL REFERENCES accounts(id),
    card_id             TEXT REFERENCES cards(id),
    category_id         TEXT REFERENCES categories(id),
    type                TEXT NOT NULL CHECK (type IN ('INCOME','EXPENSE','TRANSFER')),
    payment_method      TEXT NOT NULL CHECK (payment_method IN ('PIX','DEBIT','CREDIT','TED','BOLETO','CASH')),
    value               INTEGER NOT NULL,
    currency            TEXT NOT NULL,
    fx_rate             TEXT,
    description         TEXT NOT NULL,
    tx_date             TEXT NOT NULL,
    receipt_url         TEXT,
    picture_url         TEXT,
    transfer_pair_id    TEXT REFERENCES transactions(id),
    bill_id             TEXT,
    invoice_id          TEXT REFERENCES card_invoices(id),
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT,
    sync_version        INTEGER NOT NULL DEFAULT 1,
    device_id           TEXT NOT NULL
);
CREATE INDEX tx_user_date_id ON transactions (user_id, tx_date DESC, id DESC) WHERE deleted_at IS NULL;
CREATE INDEX tx_account_date ON transactions (account_id, tx_date DESC);
CREATE INDEX tx_category ON transactions (category_id);
CREATE INDEX tx_user_updated ON transactions (user_id, updated_at);

CREATE TABLE transaction_tags (
    transaction_id  TEXT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    tag_id          TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (transaction_id, tag_id)
);

CREATE TABLE bills (
    id                    TEXT PRIMARY KEY,
    user_id               TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id            TEXT REFERENCES accounts(id),
    category_id           TEXT REFERENCES categories(id),
    description           TEXT NOT NULL,
    value                 INTEGER NOT NULL,
    currency              TEXT NOT NULL,
    due_date              TEXT NOT NULL,
    recurrence            TEXT NOT NULL CHECK (recurrence IN ('NONE','WEEKLY','MONTHLY','YEARLY','CUSTOM')),
    recurrence_interval   INTEGER,
    recurrence_end        TEXT,
    paid                  INTEGER NOT NULL DEFAULT 0,
    paid_at               TEXT,
    payment_receipt_url   TEXT,
    paid_transaction_id   TEXT REFERENCES transactions(id),
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL,
    deleted_at            TEXT,
    sync_version          INTEGER NOT NULL DEFAULT 1,
    device_id             TEXT NOT NULL
);
CREATE INDEX bills_user_due ON bills (user_id, due_date) WHERE paid = 0;

CREATE TABLE investments (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id        TEXT NOT NULL REFERENCES accounts(id),
    name              TEXT NOT NULL,
    type              TEXT NOT NULL CHECK (type IN ('FIXED','VARIABLE')),
    principal         INTEGER NOT NULL,
    current_value     INTEGER NOT NULL,
    currency          TEXT NOT NULL,
    rate_index        TEXT CHECK (rate_index IN ('NONE','CDI','IPCA','SELIC','FIXED_RATE')),
    rate_spread       TEXT,
    purchase_date     TEXT NOT NULL,
    expiration_date   TEXT,
    archived          INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT,
    sync_version      INTEGER NOT NULL DEFAULT 1,
    device_id         TEXT NOT NULL
);

CREATE TABLE loans (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id          TEXT NOT NULL REFERENCES accounts(id),
    description         TEXT NOT NULL,
    total_value         INTEGER NOT NULL,
    currency            TEXT NOT NULL,
    interest_rate       TEXT NOT NULL,
    installment_value   INTEGER NOT NULL,
    installment_count   INTEGER NOT NULL,
    paid_installments   INTEGER NOT NULL DEFAULT 0,
    start_date          TEXT NOT NULL,
    first_due_date      TEXT NOT NULL,
    archived            INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT,
    sync_version        INTEGER NOT NULL DEFAULT 1,
    device_id           TEXT NOT NULL
);

CREATE TABLE budgets (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id   TEXT NOT NULL REFERENCES categories(id),
    period        TEXT NOT NULL CHECK (period IN ('MONTHLY','YEARLY')),
    "limit"       INTEGER NOT NULL,
    currency      TEXT NOT NULL,
    start_date    TEXT NOT NULL,
    end_date      TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT,
    sync_version  INTEGER NOT NULL DEFAULT 1,
    device_id     TEXT NOT NULL
);

CREATE TABLE goals (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    target_value  INTEGER NOT NULL,
    currency      TEXT NOT NULL,
    current_value INTEGER NOT NULL DEFAULT 0,
    account_id    TEXT REFERENCES accounts(id),
    target_date   TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT,
    sync_version  INTEGER NOT NULL DEFAULT 1,
    device_id     TEXT NOT NULL
);

------------------------------------------------------------
-- Sync + audit
------------------------------------------------------------

CREATE TABLE sync_state (
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id       TEXT NOT NULL,
    last_pulled_at  TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    last_pushed_at  TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    PRIMARY KEY (user_id, device_id)
);

CREATE TABLE audit_log (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type   TEXT NOT NULL,
    entity_id     TEXT NOT NULL,
    action        TEXT NOT NULL CHECK (action IN ('CREATE','UPDATE','DELETE','LOGIN','LOGIN_FAILED','REVOKE')),
    diff          TEXT,
    timestamp     TEXT NOT NULL,
    device_id     TEXT
);
CREATE INDEX audit_entity ON audit_log (entity_type, entity_id);
CREATE INDEX audit_user_time ON audit_log (user_id, timestamp DESC);
