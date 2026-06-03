# owo backend — implementation status

Tracks the refactor planned in
`~/.claude/plans/update-encryption-strategy-it-ll-functional-wozniak.md`.
Specs:
[DATA_MODEL.md](../docs/DATA_MODEL.md),
[API.md](../docs/API.md),
[ADR-0001](../docs/adr/0001-sync-strategy.md),
[ADR-0002](../docs/adr/0002-auth-strategy.md),
[ADR-0003 v2](../docs/adr/0003-encryption-strategy.md),
[SECURITY.md](../docs/SECURITY.md).

## Phase status

| # | Phase | Status |
|---|-------|--------|
| 1 | Doc updates (ADR-0003 v2, SECURITY.md, API.md sync, this file) | ✅ |
| 2 | Crate diet + module strip (cli, daemon, crypto, dual-DB) | ✅ |
| 3 | Migration rewrite — single consolidated `0001_initial.sql` | ✅ |
| 4 | Domain types + `ToSchema` (`domain/`) | ✅ |
| 5 | Auth + sessions (register, login, logout, me, sessions list/revoke, password change) | ✅ |
| 6 | Resource CRUD — accounts (full), categories/tags (list+create), others (list-only stubs) | 🚧 |
| 7 | Reports (cash-flow, by-category, net-worth) | ⬜ stubs |
| 8 | Sync (push topological + pull delta + tombstones) | ⬜ stubs |
| 9 | Pagination + audit endpoint (cursor wired on transactions) | 🚧 partial |
| 10 | Backup endpoints | ⬜ stubs |
| 11 | Hardening (rate limit, tracing redaction, clippy) | ⬜ |

## Verified end-to-end (2026-05-10)

```
POST /api/v1/auth/register  → 201 + bearer token
GET  /api/v1/auth/me        → 200 user
POST /api/v1/accounts       → 201 created
GET  /api/v1/accounts       → 200 list with current_balance
GET  /docs                  → Swagger UI
GET  /api-docs/openapi.json → 31 paths
```

## What's intentionally not in v1

Per ADR-0003 v2 + plan:
- Field-level / envelope / E2E encryption
- Postgres support
- Backend CLI (mobile is the primary client)
- TUI dashboard
- 2FA / TOTP
- Open Finance integration

## Verification (post-implementation)

```bash
cd backend
cargo build
cargo clippy --all-targets -- -D warnings
cargo test

# Swagger UI
curl -fs http://127.0.0.1:8080/api-docs/openapi.json | jq '.paths | keys | length'
# expect 81
```

See plan file for full smoke checklist.
