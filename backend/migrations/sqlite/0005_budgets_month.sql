-- Phase B5: budgets keyed by (category_id, month YYYY-MM, estimated_amount).
-- See specs/06-budgets.md.

PRAGMA foreign_keys = OFF;

CREATE TABLE budgets_new (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id        TEXT NOT NULL REFERENCES categories(id),
    month              TEXT NOT NULL,
    estimated_amount   INTEGER NOT NULL DEFAULT 0,
    currency           TEXT NOT NULL,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    deleted_at         TEXT,
    sync_version       INTEGER NOT NULL DEFAULT 1,
    device_id          TEXT NOT NULL,
    CHECK (length(month) = 7 AND substr(month, 5, 1) = '-')
);

INSERT INTO budgets_new (
    id, user_id, category_id, month, estimated_amount, currency,
    created_at, updated_at, deleted_at, sync_version, device_id
)
SELECT
    id, user_id, category_id,
    substr(start_date, 1, 7),
    "limit",
    currency,
    created_at, updated_at, deleted_at, sync_version, device_id
FROM budgets;

DROP TABLE budgets;
ALTER TABLE budgets_new RENAME TO budgets;

CREATE UNIQUE INDEX budgets_user_category_month
    ON budgets(user_id, category_id, month)
    WHERE deleted_at IS NULL;

PRAGMA foreign_keys = ON;
