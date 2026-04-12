# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**owo** is a personal finance management system designed to be reliable, expansible, and user-friendly. The project uses a monorepo structure with Rust backend, Kotlin Multiplatform mobile, and web/CLI frontends.

### Core Values
- **Reliability**: Financial data integrity is paramount—all code must be production-grade
- **Expansibility**: Plugin architecture and API-first design enable third-party extensions
- **Privacy**: Self-hosted capable, minimal data collection, encryption by default

## Tech Stack

### Backend
- **Language**: Rust
- **Role**: Core API, financial calculations, data processing, Open Finance integration
- **Key Libs** (TBD): Database ORM, async runtime, API framework

### Mobile
- **KMP (Kotlin Multiplatform)**: Shared logic for iOS/Android with platform-specific UI layers
- **Goal**: Single codebase with native performance

### Web
- **Type**: Browser-based dashboard and settings (framework TBD)
- **Role**: Primary user interface for transaction management, reports, settings

### CLI
- **Language**: Rust (native binary)
- **Role**: Power-user interface, automation, scripting

## Monorepo Structure

```
owo/
├── backend/                 # Rust backend
│   ├── src/
│   │   ├── api/            # REST/GraphQL endpoints
│   │   ├── models/         # Core domain models
│   │   ├── services/       # Business logic
│   │   ├── integrations/   # Open Finance, AI, etc.
│   │   └── db/             # Database migrations, models
│   ├── tests/
│   └── Cargo.toml
├── mobile/                  # KMP project
│   ├── shared/             # Shared Kotlin code
│   ├── android/
│   ├── ios/
│   └── build.gradle.kts
├── web/                     # Web frontend
│   ├── src/
│   ├── public/
│   └── package.json
├── cli/                     # Rust CLI tool
│   ├── src/
│   └── Cargo.toml
├── docs/                    # Architecture docs, ADRs, API spec
└── README.md

```

## Development Commands (to be established)

As the project develops, add common commands here:

```bash
# Backend
cargo build --release              # Build Rust backend
cargo test                         # Run all tests
cargo clippy                       # Lint with clippy
cargo fmt                          # Format code

# Mobile (KMP)
./gradlew build                   # Build all targets
./gradlew :shared:build           # Build shared code
./gradlew :android:assembleDebug  # Build Android APK

# Web
npm run dev                       # Local development server
npm run build                     # Production build
npm test                          # Run tests

# CLI
cargo build --release -p owo-cli  # Build CLI binary
```

## Key Architectural Decisions

### API Design
- REST API as primary interface (future GraphQL optional)
- Versions all endpoints for backward compatibility
- Error codes and messages follow consistent structure

### Database
- Single source of truth for financial data
- Migration-based schema management
- Audit logging for all financial transactions

### Open Finance Integration
- Adapter pattern for different bank APIs (e.g., Open Banking standards)
- Secure credential storage (encrypted, no plaintext tokens)
- Scheduled sync with manual override capability

### AI Integration (Optional)
- Plugin-style architecture—AI disabled by default
- Privacy-conscious: can run locally or via API
- Explainable recommendations with confidence scores

### Mobile Architecture (KMP)
- Shared business logic and API clients in `shared/`
- Platform-specific UI in `android/` and `ios/`
- Local database (SQLite via SQLDelight or similar)

## Important Guidelines

### Financial Data Handling
- All financial calculations must use fixed-point arithmetic (no floating-point rounding errors)
- Comprehensive audit logging for all transactions
- Encryption for sensitive data at rest and in transit
- Test thoroughly with edge cases (e.g., currency conversions, negative balances)

### Integration Testing
- Open Finance integrations require sandbox testing before production
- AI responses should be validated against real financial scenarios
- Cross-platform mobile testing essential (iOS and Android have different behaviors)

### Code Quality
- All financial logic requires unit tests
- Integration tests for API endpoints
- Performance: backend responses < 200ms for typical queries
- Security: input validation, SQL injection prevention, CORS/CSRF protection

## Expanding the Monorepo

When adding new packages/crates:
1. Add to appropriate directory
2. Update workspace configuration (Cargo.toml/settings.gradle.kts)
3. Document in CLAUDE.md
4. Include tests from the start

## External Integrations

### Open Finance APIs
- Document credentials management approach
- Test sandbox before production
- Handle rate limiting and retries gracefully

### AI Systems
- Optional feature flag to enable/disable
- API key management via environment variables
- Graceful degradation if AI service unavailable

## Further Documentation

- `/docs/ARCHITECTURE.md` - Detailed design decisions
- `/docs/API.md` - API endpoint specification
- `/docs/INTEGRATIONS.md` - Open Finance and AI setup
- Individual `README.md` files in each workspace package

---

This CLAUDE.md will evolve as the project develops. Update it with specific commands, architectural decisions, and patterns as they're established.
