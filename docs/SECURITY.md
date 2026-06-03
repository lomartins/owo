# owo — Security & Privacy

> **Scope**: v1, self-hosted-first. The `owo server` binary is intended to run
> on a host the operator owns. The trust boundary is the host OS.

This document is the operator runbook + threat-model statement. The crypto
contract lives in [ADR-0003](adr/0003-encryption-strategy.md); this file
describes how to deploy, harden, and recover an `owo` instance.

---

## Threat model

`owo` is a personal-finance app deployed on a single host (VPS, home server, or
local machine). Anyone with shell access to the account that runs the binary is
the **operator** and is trusted by definition.

### What `owo` defends against

- **Stolen DB dump alone** — passwords are Argon2id PHC strings; session tokens
  are stored as SHA-256 digests. An attacker with the SQLite file cannot
  impersonate users or recover passwords by brute force.
- **Network eavesdroppers** — clients connect over HTTPS via the operator's
  reverse proxy. Plain HTTP refused.
- **Brute-force login attempts** — `5 failures / 15 min` lockout per
  `(email, ip)`.
- **Bearer token leak from logs** — `Authorization`, `password`, `passphrase`,
  `token`, `password_hash`, `token_hash` are redacted in tracing output.
- **Stolen encrypted backups** — when produced with `--encrypt`, bundles are
  wrapped with AES-256-GCM keyed by an Argon2id-derived key from the operator's
  passphrase.
- **Lost mobile device** — sessions are revocable individually; user can list
  active sessions via the API and revoke remotely.

### What `owo` does **not** defend against

- A compromised running `owo server` process. Auth tokens are in memory.
- Root access on the host.
- A malicious or coerced operator. The operator can read all application data.
- Supply-chain compromise of the binary or its dependencies.
- Network adversaries against an unproxied `owo server`. TLS termination is
  the operator's responsibility.
- Plaintext backups that leave the host without `--encrypt`.

If your threat model includes any of those, run `owo` on a host you fully
control and apply the hardening below.

---

## Authentication contract

- **Passwords**: Argon2id, parameters `m=64MiB, t=3, p=1` (OWASP 2024). Stored
  as PHC strings in `User.password_hash`. Re-hashed on successful login if
  weaker params are detected.
- **Sessions**: 32-byte CSPRNG tokens (`OsRng`), base64url-encoded. The DB
  stores `sha256(token)` only. The raw token is returned exactly once at login
  / register and saved in the OS Keystore (Android) / Keychain (iOS) on the
  client.
- **Default lifetime**: 90 days, sliding (`last_used_at` extends `expires_at`).
  Hard cap 1 year from `created_at` — forces re-auth even on a daily-use
  device.
- **Lockout**: 5 failed attempts per `(email, ip)` within 15 minutes triggers
  a 15-minute lockout. Counters reset on successful auth.

See [ADR-0002](adr/0002-auth-strategy.md) for the full session-table shape and
revocation semantics.

---

## Operator runbook

### 1. Pick the host

- Run on a host you own or rent from a provider you trust at the
  hypervisor/disk level.
- Use a non-root system user dedicated to `owo`. The home directory of that
  user owns `<data_dir>` (database file + blob directory).
- File permissions: `<data_dir>` and everything inside is `0700` / `0600`
  owned by the service user.

### 2. Enable disk encryption

- Recommended: LUKS on the data partition (`/var/lib/owo` or `~/.local/share/owo`).
- Without LUKS, a stolen disk yields all application data in plaintext.
- Cloud VPS: enable provider-side volume encryption *and* application-level
  LUKS for defense in depth.

### 3. Run behind a TLS-terminating reverse proxy

- Caddy with automatic Let's Encrypt is the smallest setup that works.
  ```caddy
  finance.example.com {
      reverse_proxy 127.0.0.1:8080
  }
  ```
- `owo server` listens on localhost only by default. Do not expose it to the
  public internet without TLS in front.
- HSTS recommended at the proxy.

### 4. Backup hygiene

- **Default**: `owo backup create` produces `owo-YYYYMMDD-HHMMSS.tar.gz`. Do
  **not** copy plain bundles off the host.
- **Off-host backups**: always pass `--encrypt` and supply a strong
  passphrase (≥12-word diceware recommended).
  ```bash
  owo backup create --encrypt --output /mnt/external/owo-2026-05-10.tar.gz.enc
  ```
- Lost passphrase = unrecoverable bundle. Store it in a password manager
  separate from the host.

### 5. Account hygiene

- First boot: register the admin user immediately, then disable open
  registration via env: `OWO_ALLOW_REGISTRATION=false`.
- Periodically review `GET /api/v1/auth/sessions` and revoke unrecognized
  devices.
- Rotate operator passwords annually or after any suspected exposure.

---

## Password reset

There is **no** automated reset flow in v1. Recovery options:

1. **Admin reset**: an admin user runs `owo user passwd --user <email>` and
   communicates the new password through a side channel (signal, in-person,
   etc.).
2. **Direct DB access**: the operator can update `User.password_hash` with a
   freshly-hashed value.

Email-based reset is intentionally absent — it would require an SMTP
dependency and a recovery-token surface that's larger than the current attack
surface for a 100-user, self-hosted product.

---

## Reporting issues

Security issues should be reported privately to the maintainer. Do not open
a public GitHub issue for a vulnerability disclosure. Include:

- Affected version (`owo --version`).
- Reproduction steps.
- Expected vs observed behavior.
- Suggested mitigation if known.

---

## References

- [ADR-0001](adr/0001-sync-strategy.md) — sync protocol; influences tombstone
  retention and audit log immutability.
- [ADR-0002](adr/0002-auth-strategy.md) — session shape, lockout, revocation.
- [ADR-0003](adr/0003-encryption-strategy.md) — what is and is not encrypted.
- [DATA_MODEL.md](DATA_MODEL.md) — entity definitions, audit log schema.
- [API.md](API.md) — auth + session management endpoints.
