-- Phase A3: expand account.type enum.
-- See specs/05-account-types.md and specs/decisions/0002-account-types-mapping.md.
--
-- Pattern: create-new-with-temp-name, copy, drop-old, rename-new. Avoids the
-- SQLite ALTER TABLE RENAME trap where renaming the LIVE table rewrites every
-- referencing FK in other tables to point at the temp name. The temp name we
-- use here (`accounts_new`) cannot possibly be referenced by an existing FK,
-- so no FK rewrites happen.

PRAGMA foreign_keys = OFF;

CREATE TABLE accounts_new (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    type              TEXT NOT NULL CHECK (type IN ('asset','credit_card','liability','revenue','expense')),
    currency          TEXT NOT NULL,
    initial_balance   INTEGER NOT NULL DEFAULT 0,
    archived          INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT,
    sync_version      INTEGER NOT NULL DEFAULT 1,
    device_id         TEXT NOT NULL
);

INSERT INTO accounts_new (
    id, user_id, name, type, currency, initial_balance, archived,
    created_at, updated_at, deleted_at, sync_version, device_id
)
SELECT
    id, user_id, name,
    CASE type
        WHEN 'BANK'    THEN 'asset'
        WHEN 'PAYMENT' THEN 'asset'
        WHEN 'CASH'    THEN 'asset'
        ELSE type
    END AS type,
    currency, initial_balance, archived,
    created_at, updated_at, deleted_at, sync_version, device_id
FROM accounts;

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;

CREATE INDEX accounts_user_updated ON accounts (user_id, updated_at);

CREATE UNIQUE INDEX accounts_user_singleton_revenue
    ON accounts(user_id) WHERE type = 'revenue' AND deleted_at IS NULL;
CREATE UNIQUE INDEX accounts_user_singleton_expense
    ON accounts(user_id) WHERE type = 'expense' AND deleted_at IS NULL;

PRAGMA foreign_keys = ON;
