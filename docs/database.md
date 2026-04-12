# Database Design & Implementation

This document outlines the database architecture for the owo personal finance management system, including technology choices, schema design principles, and implementation guidelines.

## Overview

owo uses a **centralized database model** with the Rust backend as the source of truth. Mobile and web clients access data exclusively through the REST API to maintain consistency and security.

```
┌─────────────────┐
│  Web/CLI/Mobile │
│    Clients      │
└────────┬────────┘
         │ (REST API)
         ▼
┌─────────────────┐
│  Rust Backend   │
│   (Business     │
│   Logic)        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Database     │
│  (PostgreSQL)   │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│ Local Cache     │
│  (Mobile/Web)   │
└─────────────────┘
```

## Technology Recommendations

### Backend: Rust

#### Primary Database: PostgreSQL

**Why PostgreSQL:**
- Strong ACID guarantees for financial data
- Native JSON support (for flexible transaction metadata)
- Excellent full-text search (for transaction search)
- Reliable, mature, battle-tested in finance
- Strong encryption capabilities (pgcrypto)
- Great support for audit logging and versioning

**Recommended ORM/Query Libraries:**

1. **SQLx** (Recommended for strict type safety)
   - Compile-time query verification via macros
   - Zero runtime overhead
   - Manual query writing (more control)
   - Excellent for financial calculations where correctness matters
   ```toml
   sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "macros"] }
   ```

2. **Diesel** (Alternative - full-featured ORM)
   - Type-safe query builder
   - Good for rapid development
   - Migration management built-in
   - Less manual control than SQLx
   ```toml
   diesel = { version = "2.1", features = ["postgres"] }
   diesel_migrations = "2.1"
   ```

3. **SeaORM** (Modern alternative)
   - Async-first design
   - Good for API-heavy applications
   - Migrations via Rust
   ```toml
   sea-orm = { version = "0.12", features = ["postgres", "macros"] }
   ```

**Recommendation:** Use **SQLx** for critical financial modules (transactions, accounts, calculations) and **Diesel** or **SeaORM** for general data access. Start with SQLx for the core.

#### Connection Pooling

```toml
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
# Or use deadpool for more control
deadpool-postgres = "0.14"
```

### Mobile: Kotlin Multiplatform

#### Local Database: SQLite via SQLDelight

**Why SQLDelight:**
- Shared Kotlin code between iOS/Android
- Type-safe SQL at compile-time
- Zero-cost abstraction
- Excellent for offline-first mobile apps
- Native performance

```kotlin
// build.gradle.kts
plugins {
    id("app.cash.sqldelight") version "2.0.1"
}

sqldelight {
    databases {
        create("OwoDatabase") {
            packageName.set("com.owo.db")
        }
    }
}
```

**Key Features:**
- Local SQLite database for offline support
- Sync queue for pending transactions (to send to backend when online)
- Periodic sync with backend
- Local encryption (SQLCipher via SQLDelight)

### Web Frontend

#### Local Storage / Cache

```json
{
  "dependencies": {
    "@tanstack/query": "^5.0",  // Server state sync
    "idb": "^8.0"               // IndexedDB wrapper for offline
  }
}
```

- Use **TanStack Query** (@tanstack/query) for server state management
- **IndexedDB** for local caching of transactions
- Automatic sync when reconnected

---

## Schema Design Principles

### 1. Core Financial Data Integrity

**Decimal Arithmetic:**
```sql
-- Always use NUMERIC/DECIMAL for money (never FLOAT/DOUBLE)
CREATE TABLE accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    balance NUMERIC(19, 2) NOT NULL,  -- PostgreSQL: supports up to 131,072 digits before decimal
    currency_code CHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Transaction Ledger (Immutable):**
```sql
CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    account_id BIGINT NOT NULL REFERENCES accounts(id),
    amount NUMERIC(19, 2) NOT NULL,
    category_id BIGINT REFERENCES transaction_categories(id),
    description VARCHAR(512),
    transaction_date DATE NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Metadata for integrations (bank transaction ID, reconciliation status)
    metadata JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'posted',  -- posted, pending, failed
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_transactions_user_date ON transactions(user_id, transaction_date);
CREATE INDEX idx_transactions_account_date ON transactions(account_id, transaction_date);
CREATE INDEX idx_transactions_status ON transactions(status);
```

### 2. Audit Logging (Critical for Finance)

```sql
-- Immutable audit log for all financial changes
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    entity_type VARCHAR(50) NOT NULL,  -- 'account', 'transaction', etc.
    entity_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    action VARCHAR(50) NOT NULL,  -- 'created', 'updated', 'deleted'
    old_values JSONB,
    new_values JSONB,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent VARCHAR(512),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT
);

CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_user_date ON audit_log(user_id, changed_at);
```

### 3. Accounts & Categories

```sql
CREATE TABLE accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    account_type VARCHAR(50) NOT NULL,  -- 'checking', 'savings', 'credit', 'investment'
    balance NUMERIC(19, 2) NOT NULL,
    currency_code CHAR(3) NOT NULL DEFAULT 'USD',
    -- Integration tracking
    external_account_id VARCHAR(255),  -- From Open Finance API
    external_account_type VARCHAR(255),  -- Bank, CreditCard, etc.
    last_synced_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, name)
);

CREATE TABLE transaction_categories (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,  -- NULL = system default
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7),  -- Hex color
    icon VARCHAR(50),
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

### 4. Syncing & Conflicts (Mobile)

```sql
-- Track sync state for mobile clients
CREATE TABLE sync_queue (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id BIGINT NOT NULL,
    operation VARCHAR(20) NOT NULL,  -- 'create', 'update', 'delete'
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, synced, failed
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Detect conflicts (e.g., same transaction modified on mobile + backend)
CREATE TABLE sync_conflicts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id BIGINT NOT NULL,
    remote_version JSONB,
    local_version JSONB,
    resolution VARCHAR(20),  -- 'remote_wins', 'local_wins', 'merge'
    resolved_at TIMESTAMPTZ,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

---

## Migration Management

### Rust Backend (SQLx or Diesel)

**Using Diesel:**
```bash
diesel setup  # Create database
diesel migration generate create_accounts
diesel migration run  # Apply migrations
```

**Using SQLx:**
```bash
sqlx migrate add create_accounts
sqlx migrate run
```

**Migration Structure:**
```
backend/migrations/
├── 20260412000000_create_users.sql
├── 20260412000001_create_accounts.sql
├── 20260412000002_create_transactions.sql
├── 20260412000003_create_audit_log.sql
└── 20260412000004_create_indexes.sql
```

### Mobile (SQLDelight)

```kotlin
// mobile/shared/src/sqliteShared/kotlin/migrations.sql
sqldelight {
    databases {
        create("OwoDatabase") {
            schemaOutputDirectory = "src/sqliteShared/sqldelight"
        }
    }
}
```

---

## Implementation Checklist

### Phase 1: Core Setup
- [ ] Choose ORM (recommend SQLx + Diesel combo)
- [ ] Set up PostgreSQL locally and in CI/CD
- [ ] Create initial schema with transactions, accounts, audit log
- [ ] Implement database migrations
- [ ] Set up connection pooling

### Phase 2: Mobile Integration
- [ ] Set up SQLDelight for KMP
- [ ] Implement local transaction storage
- [ ] Create sync queue mechanism
- [ ] Handle offline-first scenarios

### Phase 3: Advanced Features
- [ ] Audit logging for compliance
- [ ] Full-text search on transactions
- [ ] Reporting database views
- [ ] Backup and recovery procedures

### Phase 4: Performance
- [ ] Add indexes for common queries
- [ ] Query performance monitoring
- [ ] Caching strategy (Redis for backend)
- [ ] Database optimization & VACUUM

---

## Key Considerations

### Financial Data Security
- Encrypt sensitive fields at rest (SSN, bank account numbers)
- Use prepared statements to prevent SQL injection
- Implement row-level security (PostgreSQL RLS) if needed
- Audit all access to financial data

### Backup & Recovery
- Automated daily backups
- Point-in-time recovery capability
- Test recovery procedures regularly
- Consider encrypting backups

### Performance Targets
- Account balance queries: < 50ms
- Transaction list (paginated): < 100ms
- Reports aggregation: < 500ms
- Sync operations: < 2s

### Compliance
- GDPR: Data deletion & export capabilities
- CCPA: Audit trail for all data access
- PCI-DSS: If handling payment cards directly
- Immutable audit log retention (7+ years for finance)

---

## Recommended Development Workflow

```bash
# Backend
cd backend
cargo sqlx database create
cargo sqlx migrate run
cargo test

# Mobile
cd mobile
./gradlew :shared:generateSqlDelight
./gradlew build

# Local database
docker run -d -p 5432:6432 -e POSTGRES_PASSWORD=local postgres:16
psql -h localhost -U postgres -f scripts/init.sql
```

---

## Resources

- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Diesel Guide](https://diesel.rs/)
- [SeaORM](https://www.sea-ql.org/)
- [SQLDelight](https://cashapp.github.io/sqldelight/)
- [PostgreSQL Best Practices](https://www.postgresql.org/docs/)
- [Financial Data Handling](https://en.wikipedia.org/wiki/Decimal_datatype)
