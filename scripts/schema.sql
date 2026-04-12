-- Core owo schema
-- Financial data tables with strong integrity constraints

SET timezone = 'UTC';

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    account_type VARCHAR(50) NOT NULL,
    balance NUMERIC(19, 2) NOT NULL DEFAULT 0.00,
    currency_code CHAR(3) NOT NULL DEFAULT 'USD',
    external_account_id VARCHAR(255),
    external_account_type VARCHAR(255),
    last_synced_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_accounts_user_active ON accounts(user_id, is_active);
CREATE INDEX idx_accounts_external_id ON accounts(external_account_id);

-- Transaction categories
CREATE TABLE IF NOT EXISTS transaction_categories (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7),
    icon VARCHAR(50),
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_categories_user ON transaction_categories(user_id);

-- Transactions (immutable ledger)
CREATE TABLE IF NOT EXISTS transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    amount NUMERIC(19, 2) NOT NULL,
    category_id BIGINT REFERENCES transaction_categories(id) ON DELETE SET NULL,
    description VARCHAR(512),
    transaction_date DATE NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'posted',
    CHECK (status IN ('posted', 'pending', 'failed'))
);

CREATE INDEX idx_transactions_user_date ON transactions(user_id, transaction_date DESC);
CREATE INDEX idx_transactions_account_date ON transactions(account_id, transaction_date DESC);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_category ON transactions(category_id);

-- Audit log (immutable)
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    entity_type VARCHAR(50) NOT NULL,
    entity_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent VARCHAR(512)
);

CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_user_date ON audit_log(user_id, changed_at DESC);

-- Sync queue for mobile offline support
CREATE TABLE IF NOT EXISTS sync_queue (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id BIGINT NOT NULL,
    operation VARCHAR(20) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,
    CHECK (status IN ('pending', 'synced', 'failed'))
);

CREATE INDEX idx_sync_queue_user_status ON sync_queue(user_id, status);
CREATE INDEX idx_sync_queue_created ON sync_queue(created_at DESC);

-- Sync conflicts detection
CREATE TABLE IF NOT EXISTS sync_conflicts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id BIGINT NOT NULL,
    remote_version JSONB,
    local_version JSONB,
    resolution VARCHAR(20),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_conflicts_user ON sync_conflicts(user_id);

-- Grant all permissions to owo user
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO owo;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO owo;

PRINT 'Schema created successfully';
