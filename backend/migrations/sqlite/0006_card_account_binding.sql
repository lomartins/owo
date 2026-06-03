-- Phase C9: extend audit_log.action so the binding pass can record ambiguous rows.
-- Create-new-then-drop-old pattern (see 0002).
-- See specs/08-cards-preserve.md.

PRAGMA foreign_keys = OFF;

CREATE TABLE audit_log_new (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type   TEXT NOT NULL,
    entity_id     TEXT NOT NULL,
    action        TEXT NOT NULL CHECK (action IN ('CREATE','UPDATE','DELETE','LOGIN','LOGIN_FAILED','REVOKE','MIGRATION_NEEDS_REVIEW')),
    diff          TEXT,
    timestamp     TEXT NOT NULL,
    device_id     TEXT
);

INSERT INTO audit_log_new SELECT * FROM audit_log;
DROP TABLE audit_log;
ALTER TABLE audit_log_new RENAME TO audit_log;

CREATE INDEX audit_entity ON audit_log (entity_type, entity_id);
CREATE INDEX audit_user_time ON audit_log (user_id, timestamp DESC);

PRAGMA foreign_keys = ON;
