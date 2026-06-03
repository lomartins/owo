-- Bills become recurring monthly templates with per-month payment tracking.
-- `due_day` (1-31) replaces the absolute `due_date` for monthly bills.
-- `bill_payments` records each month a bill was paid + which transaction posted it.

PRAGMA foreign_keys = OFF;

ALTER TABLE bills ADD COLUMN due_day INTEGER;

-- Backfill due_day from existing due_date strings (YYYY-MM-DD).
UPDATE bills SET due_day = CAST(substr(due_date, 9, 2) AS INTEGER) WHERE due_day IS NULL;

-- For new bills the migration default recurrence is MONTHLY; existing rows keep
-- whatever they had. The UI now treats every bill as monthly + recurring.
UPDATE bills SET recurrence = 'MONTHLY' WHERE recurrence IS NULL OR recurrence = '';

CREATE TABLE bill_payments (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bill_id         TEXT NOT NULL REFERENCES bills(id) ON DELETE CASCADE,
    month           TEXT NOT NULL,
    transaction_id  TEXT REFERENCES transactions(id),
    paid_at         TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    sync_version    INTEGER NOT NULL DEFAULT 1,
    device_id       TEXT NOT NULL,
    CHECK (length(month) = 7 AND substr(month, 5, 1) = '-')
);

CREATE UNIQUE INDEX bill_payments_bill_month ON bill_payments(bill_id, month);
CREATE INDEX bill_payments_user_month ON bill_payments(user_id, month);

-- Backfill: bills that were already marked paid get a payment row in their due-month.
INSERT INTO bill_payments (id, user_id, bill_id, month, transaction_id, paid_at, created_at, device_id)
SELECT
    lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' ||
    lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6))),
    user_id,
    id,
    substr(due_date, 1, 7),
    paid_transaction_id,
    COALESCE(paid_at, updated_at),
    COALESCE(paid_at, updated_at),
    device_id
FROM bills
WHERE paid = 1 AND deleted_at IS NULL;

PRAGMA foreign_keys = ON;
