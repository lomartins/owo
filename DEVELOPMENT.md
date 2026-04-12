# Development Setup Guide

This guide explains how to set up your local development environment for the owo project.

## Prerequisites

- **Docker** & **Docker Compose** (for PostgreSQL and services)
- **Rust** 1.75+ ([Install](https://rustup.rs/))
- **Node.js** 18+ (for web frontend)
- **Android Studio** or **Xcode** (for mobile development)

## Quick Start

### 1. Clone & Setup

```bash
git clone https://github.com/yourusername/owo.git
cd owo
cp .env.example .env
```

### 2. Start PostgreSQL with Docker

```bash
docker-compose up -d
```

This will:
- Start PostgreSQL 16 Alpine on port 5432
- Initialize the database with users table
- Create core schema (accounts, transactions, audit log, etc.)
- Persist data in `postgres_data` volume

**Verify it's running:**
```bash
docker-compose ps
```

### 3. Connect to the Database

```bash
# Using psql
psql -h localhost -U owo -d owo_dev

# Using Docker
docker-compose exec postgres psql -U owo -d owo_dev

# Connection string for Rust
DATABASE_URL=postgres://owo:owo_dev_password@localhost:5432/owo_dev
```

## Backend Development

### Setup

```bash
cd backend
cargo build
```

### Environment Variables

Create a `.env` file (already included):
```bash
cat .env
```

Key variables:
- `DB_HOST`: localhost
- `DB_PORT`: 5432
- `DATABASE_URL`: postgres://owo:owo_dev_password@localhost:5432/owo_dev
- `RUST_LOG`: debug

### Running the Backend

```bash
cd backend
cargo run
```

The API will be available at `http://localhost:3000`

### Common Commands

```bash
# Build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy

# Format code
cargo fmt

# Run with logging
RUST_LOG=debug cargo run

# Run specific binary
cargo run --bin owo-backend
```

### Database Migrations

Migrations are handled via SQL files in the `/scripts` folder.

**When you need to add a schema change:**

1. Create a new migration file:
   ```bash
   touch scripts/migrations/YYYYMMDD_HHMMSS_description.sql
   ```

2. Add your SQL (example):
   ```sql
   -- migrations/20260412_140000_add_user_preferences.sql
   CREATE TABLE user_preferences (
       id BIGSERIAL PRIMARY KEY,
       user_id BIGINT NOT NULL UNIQUE REFERENCES users(id),
       theme VARCHAR(20) DEFAULT 'light',
       currency VARCHAR(3) DEFAULT 'USD',
       created_at TIMESTAMPTZ DEFAULT NOW()
   );
   ```

3. Restart Docker to apply:
   ```bash
   docker-compose down
   docker-compose up -d
   ```

## Mobile Development

### Setup Kotlin Multiplatform

```bash
cd mobile
./gradlew build
```

### iOS Development
```bash
cd mobile
open ios/owo.xcworkspace
```

### Android Development
```bash
cd mobile
# In Android Studio, open the mobile/ folder
```

## Web Frontend Development

```bash
cd web
npm install
npm run dev
```

Access at `http://localhost:5173` (or as shown in terminal)

## Testing

### Backend Tests

```bash
cd backend
cargo test                    # Run all tests
cargo test --lib             # Run unit tests only
cargo test -- --nocapture    # Show println! output
cargo test -- --test-threads=1  # Run sequentially
```

### Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        // Test code
    }
}
```

## Docker Compose Commands

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f postgres

# Rebuild images
docker-compose build --no-cache

# Remove volumes (WARNING: deletes data)
docker-compose down -v

# Access PostgreSQL shell
docker-compose exec postgres psql -U owo -d owo_dev
```

## Troubleshooting

### "Connection refused" error

```bash
# Check if postgres is running
docker-compose ps

# Restart it
docker-compose restart postgres

# Check logs
docker-compose logs postgres
```

### Database migration failed

```bash
# Reset database (WARNING: loses all data)
docker-compose down -v
docker-compose up -d
```

### Port already in use (5432)

```bash
# Change port in .env
DB_PORT=5433

# Update docker-compose.yml
# ports:
#   - "5433:5432"
```

### psql not found

```bash
# Use Docker to access database
docker-compose exec postgres psql -U owo -d owo_dev

# Or install PostgreSQL client locally
# macOS: brew install postgresql
# Ubuntu: sudo apt install postgresql-client
```

## IDE Setup

### VS Code Extensions (Recommended)

```json
// .vscode/extensions.json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "ms-vscode.cpptools",
    "eamodio.gitlens",
    "GitHub.copilot",
    "ms-python.python"
  ]
}
```

### IntelliJ IDEA / Android Studio

- Install Rust plugin
- Install Kotlin plugin
- Install Git plugin

## Useful Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio (async runtime)](https://tokio.rs/)
- [PostgreSQL Docs](https://www.postgresql.org/docs/)
- [Kotlin Multiplatform](https://kotlinlang.org/docs/multiplatform.html)
- [SQLDelight](https://cashapp.github.io/sqldelight/)

## Next Steps

1. ✅ Start PostgreSQL: `docker-compose up -d`
2. ✅ Build backend: `cd backend && cargo build`
3. ✅ Run tests: `cargo test`
4. ✅ Start backend: `cargo run`
5. Test an endpoint: `curl http://localhost:3000/health`

---

**Need help?** Check the project's issues or CLAUDE.md for architecture details.
