# ADR-0002: Authentication & Session Strategy

- **Status**: Proposed
- **Date**: 2026-05-10
- **Deciders**: Luis Martins
- **Context tags**: auth, security, sessions, mobile

---

## Context

owo is self-hostable with up to 100 users. Clients: KMP mobile (primary), future web + CLI. Backend: Rust + axum + Postgres. Offline-first — clients hold long-lived credentials and sync periodically.

Requirements:
1. Password-based login (email + password).
2. Long-lived sessions on trusted devices (months) — friction-free finance app.
3. Per-device session revocation (lost phone scenario).
4. Survives offline use: client must operate without re-auth on every request.
5. Resistant to common attacks: credential stuffing, replay, token theft from logs.
6. No third-party identity provider in v1 (self-hosted, privacy).
7. Low operational cost — no Redis, no separate auth service.

Non-requirements (v1):
- OAuth / social login.
- 2FA / TOTP (deferred to v2).
- SSO.
- Passwordless / magic links.

---

## Options Considered

### Option A — Opaque session tokens in Postgres

- Login: verify password, generate 256-bit random token, store SHA-256 hash in `session` table, return raw token to client.
- Each request: `Authorization: Bearer <token>`. Server hashes and looks up.
- Revocation: `DELETE FROM session WHERE id=…` — instant.
- Refresh: extend `expires_at` on use (sliding window).

**Pros**: Trivial revocation. No client-side validation logic. Hash leak ≠ token leak. Single DB. Fits sqlx + Postgres directly.
**Cons**: DB hit per request (mitigated by Postgres index + connection pool).

### Option B — JWT (signed, stateless)

- Server signs JWT with HS256/EdDSA; clients send in `Authorization` header.
- Verification is stateless — no DB hit.

**Pros**: Stateless, scales horizontally.
**Cons**: Revocation requires a denylist (defeats statelessness). Long-lived JWTs are a footgun. Algorithm-confusion attacks. Token in logs = full account access until expiry. Overkill for 100 users on one server. Refresh-token dance adds complexity.

### Option C — Refresh + access token pair (OAuth-style)

- Short-lived access JWT (15 min) + long-lived opaque refresh token in DB.
- Client refreshes silently when access expires.

**Pros**: Best-of-both for large scale.
**Cons**: Two token types, refresh endpoint, race conditions on parallel requests, extra surface area. Justified at >10k users, not 100.

---

## Decision

**Adopt Option A — opaque session tokens in Postgres.**

### Specifics

**Password storage**: `argon2id` with parameters `m=64MiB, t=3, p=1` (OWASP 2024 recommendation). Stored as PHC string in `User.password_hash`. Re-hash on login if params upgraded.

**Token generation**:
- 32 bytes from `OsRng` → base64url-encoded → ~43 chars.
- Server stores **SHA-256 hash** of the token, never the raw value.
- Returned to client exactly once, in login response body.

**Session table** (already in DATA_MODEL.md, refined here):
```
Session(
    id: UUID,
    user_id: FK User,
    token_hash: BYTEA UNIQUE,        // SHA-256 of raw token
    device_id: String,               // client-generated stable id
    device_name: String,             // user-visible: "Pixel 8 — Luis"
    user_agent: String?,
    ip_last_seen: INET?,
    created_at: Timestamp,
    last_used_at: Timestamp,
    expires_at: Timestamp,
    revoked_at: Timestamp?
)
```

Index: `(token_hash)` for lookup; `(user_id, revoked_at)` for "list my sessions".

**Lifetime**:
- Default `expires_at = now() + 90 days`. Sliding window: each successful auth bumps `last_used_at` and extends `expires_at` to `last_used_at + 90 days` (capped at `created_at + 365 days`).
- Hard cap = 1 year. Forces re-auth even on a phone that's used daily.

**Revocation paths**:
- User: `DELETE /sessions/{id}` or `DELETE /sessions` (all but current) → set `revoked_at`. Auth middleware rejects if `revoked_at IS NOT NULL`.
- Server-initiated (password change): revoke all sessions for user except the one performing the change.
- Server-initiated (suspected compromise): same as above.

**Login flow**:
1. `POST /auth/login` `{ email, password, device_id, device_name }`
2. Server: lookup user, verify argon2, generate token, insert session.
3. Response: `{ token, user: {...}, session: { id, expires_at } }`.
4. Rate limit: 5 attempts / 15 min / IP+email. After 5 failures, lock email for 15 min.

**Auth middleware**:
- Extract `Authorization: Bearer <token>`.
- SHA-256 hash, query `SELECT user_id FROM session WHERE token_hash=$1 AND revoked_at IS NULL AND expires_at > now()`.
- Update `last_used_at` async (skip if updated <60s ago to reduce write amp).
- Reject with `401` on miss; never specify why.

**Client storage**:
- Mobile: Keystore (Android) / Keychain (iOS) via KMP `multiplatform-settings` + platform-specific encrypted backend.
- Token is the only sync credential — no separate refresh.
- On `401`: redirect to login. Sync queue preserved locally.

**Logging**:
- Never log raw tokens, never log password fields. Tracing filter strips `Authorization` and `password` keys.
- Log `session.id` instead of token for audit traces.

**Audit**:
- Login success/failure → `audit_log` (entity=`Session`, action=`CREATE`/`LOGIN_FAILED`).
- Session revoke → `audit_log` (action=`DELETE`).

---

## Consequences

**Positive**:
- One round-trip to DB per request — acceptable at <100 users, indexed lookup is sub-millisecond.
- Revocation is immediate and trivial.
- Token theft from logs impossible (tokens never logged; hash-only storage).
- Argon2id with current params is GPU-resistant.
- No JWT footguns.

**Negative**:
- Every authed request hits Postgres. At 100 users this is irrelevant; at 10k+ would benefit from a small in-memory cache (LRU keyed on `token_hash`).
- Session table grows unbounded without GC — needs periodic `DELETE WHERE revoked_at < now() - interval '30 days' OR expires_at < now()`.
- No offline auth verification — but every request is already online (it's a sync request). Mobile reads Room locally anyway; auth only matters at sync boundary.

**Deferred to future ADRs**:
- TOTP / WebAuthn 2FA.
- Hardware-key-backed session binding.
- Anti-CSRF for browser web client (cookie-based) — current Bearer scheme is XSS-vulnerable in browser; revisit when web ships.

---

## Implementation Checklist

- [ ] `argon2` crate in `backend/Cargo.toml`, params constant.
- [ ] `services/password.rs` — `hash`, `verify`, `needs_rehash`.
- [ ] `services/session.rs` — `create`, `lookup`, `revoke`, `revoke_all_for_user`, `gc`.
- [ ] `api/auth_middleware.rs` — extract bearer, lookup, attach `User` to request extensions.
- [ ] Login rate limiter — `tower-governor` or in-process token bucket keyed on (ip, email).
- [ ] Tracing filter — redact `Authorization`, `password`, `token_hash`.
- [ ] Mobile: encrypted token storage via KMP wrapper around Keystore/Keychain.
- [ ] Background GC: cron-style job in backend — daily session cleanup.

---

## References

- OWASP Password Storage Cheat Sheet (2024).
- argon2 RFC 9106.
- ADR-0001 — sync flow uses session token at the sync boundary.
- ADR-0003 — encryption strategy (sibling).
