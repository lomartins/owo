-- Per-month overrides for recurring bills.
-- A bill is a template (in `bills`). An override is a single-month patch
-- that wins over the template when listing for that month.
-- "Edit just this month" creates one row here; "edit this + next" creates two;
-- "edit all" patches the template and clears overrides.

PRAGMA foreign_keys = OFF;

CREATE TABLE bill_overrides (
    id           TEXT PRIMARY KEY,
    bill_id      TEXT NOT NULL REFERENCES bills(id) ON DELETE CASCADE,
    month        TEXT NOT NULL,
    description  TEXT,
    value        INTEGER,
    due_day      INTEGER,
    account_id   TEXT REFERENCES accounts(id),
    category_id  TEXT REFERENCES categories(id),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    CHECK (length(month) = 7 AND substr(month, 5, 1) = '-')
);

CREATE UNIQUE INDEX bill_overrides_bill_month
    ON bill_overrides(bill_id, month);

PRAGMA foreign_keys = ON;
