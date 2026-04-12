-- Initialize PostgreSQL for owo
-- This script is automatically run by Docker when the container starts

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Set timezone to UTC for all timestamps
SET timezone = 'UTC';

-- Create users table (base for all other data)
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Grant permissions
GRANT CONNECT ON DATABASE owo_dev TO owo;
GRANT USAGE ON SCHEMA public TO owo;
GRANT CREATE ON SCHEMA public TO owo;

-- Create a migration tracking table for sqlx
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL,
    execution_time BIGINT NOT NULL
);

GRANT SELECT, INSERT ON _sqlx_migrations TO owo;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO owo;

PRINT 'Database initialized successfully';
