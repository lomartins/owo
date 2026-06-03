# ADR-0003: Encryption Strategy

- **Status**: Accepted (v2 — simplified)
- **Date**: 2026-05-10
- **Deciders**: Luis Martins
- **Context tags**: security, privacy, encryption, backup
- **Supersedes**: ADR-0003 v1 (envelope/DEK/master-key design)

---

## Context

owo v1 ships **self-hosted-first**. The trust boundary is the host operating system: anyone with shell access to the account that runs `owo server` is the operator and is trusted by definition. Up to 100 users, single dev, low-cost server (one VPS).

In v1 owo does not aim to defend against a malicious or compromised host operator. That assumption changes the cryptographic surface area dramatically: server-side field encryption with a server-held key adds complexity (master key, per-user DEK, envelope wrap, blob ciphertext, key rotation, restore re-wrap) without raising the bar against the threats v1 actually targets.

What still needs cryptographic protection:
1. **User passwords** — must survive a stolen DB dump.
2. **Session tokens** — must not be readable from server logs or DB dumps.
3. **Network traffic** — must not be readable on the wire.
4. **Backups that leave the host** — optional, operator-driven.

Non-goals for v1:
- Defense against malicious operator / root.
- Server-side field-level encryption.
- Zero-knowledge / E2E mode.
- Hardware-key-backed master key.
- Searchable encryption.

---

## Decision

**Cryptography in v1 is limited to authentication material plus optional backup wrap.** Application data is stored in plaintext on disk. The host OS and (optional) full-disk encryption are the only at-rest barriers for application data.

### Layer 1 — In transit

- TLS 1.3, terminated at a reverse proxy (Caddy / nginx) deployed by the operator. Backend listens on localhost.
- Mobile clients connect over HTTPS only. Plain HTTP refused in production builds.
- No certificate pinning in v1 (operational risk for self-hosters; revisit if a hosted product ships).

### Layer 2 — At rest (deployment, not application)

- LUKS or equivalent full-disk encryption documented as **recommended** in the operator runbook (`docs/SECURITY.md`).
- Postgres-style filesystem encryption is the responsibility of the operator. Backend assumes plaintext disk.
- Application performs no field-level encryption. Receipts and photos are stored as plaintext files under `<data_dir>/blobs/<user_id>/<uuid>` with random UUID filenames; OS permissions (`0600` files, `0700` dirs, owned by the service user) are the only barrier.

### Layer 3 — Authentication material (required encryption)

**Passwords**:
- `argon2id` with parameters `m=64MiB, t=3, p=1` (OWASP 2024 recommendation).
- Stored as a PHC string in `User.password_hash`.
- Re-hash on successful login if stored params are weaker than current code defaults (`needs_rehash`).

**Session tokens**:
- 32 bytes from `OsRng`, base64url-encoded.
- Server stores **SHA-256 digest** of the token in `Session.token_hash`. The raw token is returned exactly once in the login response.
- A token leak from the DB is not a session takeover; the attacker still needs the raw token (hash preimage).

**Tracing redaction**:
- `Authorization`, `password`, `passphrase`, `token`, `password_hash`, `token_hash` keys stripped from all log records.

### Layer 4 — Backups (optional encryption)

`owo` produces backups as a single tar+gzip archive containing the SQLite file plus the blob directory. Two modes:

- **Default — plain tar+gzip.** Suitable when the backup never leaves the host (e.g. a separate disk on the same machine).
- **Operator-encrypted — `--encrypt`.** When the operator passes `--encrypt`, the bundle is wrapped with `AES-256-GCM` using a key derived from a passphrase via `argon2id` (`m=128MiB, t=4, p=1`). Salt + KDF params are stored in a manifest header inside the encrypted bundle.

The backup passphrase is independent of any user login password. Lost passphrase = unrecoverable bundle. Documented as such; intentional.

Restore is symmetric: `owo backup restore [--passphrase ...]` detects the mode from the manifest and proceeds.

---

## Threat coverage

| Threat                              | Coverage                                                  |
|-------------------------------------|-----------------------------------------------------------|
| Network eavesdrop                   | TLS 1.3                                                   |
| Stolen DB dump                      | Passwords (Argon2id) + tokens (SHA-256 digests) safe. **All other data readable.** |
| Stolen plain backup                 | All data readable. Operator decision to leave host.       |
| Stolen `--encrypt` backup           | Argon2id passphrase wrap; brute force infeasible with strong passphrase |
| Lost mobile device                  | OS Keystore/Keychain protects token; remote session revoke. Local Room DB visible if device unlocked — out of scope. |
| Brute-force login attempts          | `5 failures / 15 min` lockout per `(email, ip)` (services/user.rs) |
| Compromised host / root / operator  | **Not covered.** Documented v1 limit.                    |
| Forgotten login password            | Admin reset only (`owo user passwd`).                    |
| Forgotten backup passphrase         | Bundle unrecoverable. Documented.                         |

---

## Consequences

**Positive**:
- Implementation is small. No master key plumbing, no DEK envelope, no per-row nonces. Maintenance burden roughly halved versus v1 of this ADR.
- DB dumps reveal application data — this is honest about the threat model and surfaces the trade-off explicitly to operators choosing where to deploy.
- Backups are portable across hosts without re-wrapping anything.
- Login password reset doesn't destroy data (no DEK to lose).

**Negative**:
- A stolen DB file leaks every user's transactions, balances, and descriptions. Mitigated by host-level controls (LUKS, OS permissions, access discipline) — same posture as `~/.config/Xyz` apps and personal SQLite tools.
- Receipt blobs are plaintext on disk. Same mitigation.
- Field-level search (e.g. across descriptions) works but reveals plaintext to anyone with DB access. Acceptable for v1's threat model.

---

## Migration triggers

Re-evaluate envelope / E2E encryption if **any** of the following ship:

- Hosted multi-tenant deployment (we host on behalf of users) — operator becomes adversarial.
- Shared / family accounts — multiple human owners share a row, weakening per-user trust.
- Regulatory requirement (LGPD enforcement action, banking compliance) demanding ciphertext at rest.
- A reported incident showing real-world data leak via stolen DB dumps.

Until one of those triggers fires, the v2 design holds.

---

## Implementation Checklist

- [ ] Remove `aes-gcm`, `keyring`, `base64` from `backend/Cargo.toml`.
- [ ] Delete `backend/src/crypto/` (whole directory).
- [ ] Strip `Repo.master_key`, `enc()`, `dec()` helpers from `db/repo.rs`.
- [ ] Drop `User.wrapped_dek`, `User.dek_nonce`, `User.crypto_version` columns from the schema (consolidated migration replaces them).
- [ ] Confirm `services/password.rs` Argon2id params match `m=64MiB, t=3, p=1`.
- [ ] Confirm `services/session.rs` stores `sha256(token)` only.
- [ ] Tracing filter excludes `Authorization`, `password`, `passphrase`, `token`, `password_hash`, `token_hash`.
- [ ] `services/backup.rs` (new) — tar+gzip with optional Argon2id+AES-GCM wrap.
- [ ] `docs/SECURITY.md` — rewrite operator runbook (LUKS recommended, TLS proxy, backup hygiene).

---

## References

- ADR-0001 — sync moves plaintext over HTTPS. No transparency-decryption step needed at the sync boundary.
- ADR-0002 — login password is independent of any data-encryption key (there is none). Reset is safe.
- OWASP Password Storage Cheat Sheet (2024).
- Argon2 RFC 9106.
- `docs/SECURITY.md` — operational runbook.
