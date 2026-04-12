-- Initialize PostgreSQL for owo
-- This script is automatically run by Docker when the container first starts.

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Set timezone to UTC for all timestamps
SET timezone = 'UTC';

-- Users table (base for all other data)
CREATE TABLE IF NOT EXISTS users (
    id            BIGSERIAL    PRIMARY KEY,
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name     VARCHAR(255),
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Grant permissions
GRANT CONNECT ON DATABASE owo_dev TO owo;
GRANT USAGE   ON SCHEMA public TO owo;
GRANT CREATE  ON SCHEMA public TO owo;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES    IN SCHEMA public TO owo;
GRANT USAGE, SELECT                  ON ALL SEQUENCES IN SCHEMA public TO owo;

-- Note: _sqlx_migrations is managed by sqlx — do not create it here.
-- sqlx will create it automatically with the correct schema on first run.

DO $$ BEGIN
    RAISE NOTICE 'Database initialized successfully';
END $$;
