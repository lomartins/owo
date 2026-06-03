# ADR-0001: Offline-First Sync Strategy

- **Status**: Proposed
- **Date**: 2026-05-10
- **Deciders**: Luis Martins
- **Context tags**: sync, offline-first, mobile, backend

---

## Context

owo is offline-first. Mobile (KMP, Room) is the primary client; backend (Rust + Postgres) is the authoritative server. Up to 100 users, single-user multi-device (phone + tablet + future web/CLI). All entities carry `created_at`, `updated_at`, `deleted_at?`, `sync_version`, `device_id`.

Requirements:
1. Writes must succeed offline.
2. Reconverge on reconnect without data loss for non-conflicting edits.
3. Conflicts resolve deterministically without user prompts in the common case.
4. Audit log preserves all states (per CLAUDE.md mandate).
5. Low backend cost — no per-tenant compute, no constant push channel.
6. Implementation must fit a small team (one dev) and a small server.

Non-requirements:
- Real-time collaboration (no shared accounts in v1).
- Sub-second propagation across devices.
- Causal consistency across unrelated entities.

---

## Options Considered

### Option A — Last-Write-Wins (LWW) per row, pull-based delta sync

- Each row carries `updated_at` (server-assigned on accept) + `device_id`.
- Client pulls `GET /sync?since=<cursor>` returning rows changed after cursor.
- Client pushes `POST /sync` with batch of local changes; server applies LWW, returns authoritative versions.
- Tombstones via `deleted_at`.
- Conflict rule: highest `updated_at` wins; tie-break by `(device_id, id)` lexicographic.
- Audit log captures every accepted version; losers are recoverable.

**Pros**: Simple. Works with sqlx + Postgres directly. Single endpoint pair. Cheap. Single-user multi-device rarely produces real conflicts.
**Cons**: Concurrent edits to the same row drop one side's field-level changes. No automatic merge of independent fields.

### Option B — CRDT (per-field LWW register, OR-Set for tags)

- Each scalar field is an LWW-register `(value, hlc_timestamp, device_id)`.
- Sets (tags) use OR-Set or Add-Wins.
- Merge is commutative/associative; clients converge without server arbitration.
- Server stores the merged state plus an op log.

**Pros**: No data loss on concurrent edits. Sync direction agnostic. Future P2P possible.
**Cons**: Significant complexity. Schema bloat (timestamp per field). Library choice in Rust + Kotlin is thin (`automerge`, `yrs`). Overkill for single-user multi-device.

### Option C — Event-sourcing / op log with server-side replay

- Clients emit immutable ops; server replays into materialized views.
- Conflicts handled by op semantics.

**Pros**: Strong audit, time-travel for free.
**Cons**: Heaviest complexity. Forces every domain action to be an op. Hard to retrofit.

---

## Decision

**Adopt Option A (LWW row-level + delta pull/push) for v1.** Reserve Option B for entities with proven concurrent-edit conflicts post-launch (likely never, given single-user scope).

### Specifics

**Clock**: Server-assigned `updated_at` (TIMESTAMPTZ from `clock_timestamp()`). Client `updated_at` ignored on accept; server stamps and returns. This avoids relying on client clock skew. Local optimistic UI uses client timestamp until server response replaces it.

**Tie-break**: When two devices push edits for the same row in the same `clock_timestamp()` tick (rare), server compares `device_id` lexicographic ASC; loser is rewritten with new `updated_at` so its next pull receives the canonical version.

**Sync cursor**: Per-device row in `sync_state(user_id, device_id, last_pulled_at, last_pushed_at)`.

**Endpoints**:
- `POST /sync/push` — body: `{ entity: [...rows] }`. Returns accepted rows with server timestamps + any rejected (validation only, never conflicts).
- `GET /sync/pull?since=<iso8601>&limit=N` — returns batches of changed rows per entity, plus `next_cursor`.

**Tombstones**: `deleted_at` set, row retained 90 days, then GC. Pull always returns tombstoned rows updated after cursor so peer devices delete locally.

**Audit log**: Every accepted server write inserts into `audit_log` with full diff. Audit log is server-only — clients do not pull it.

**Foreign keys & ordering**: Push payload may reference rows the server hasn't seen. Solution: client sends rows in topological order per batch (User → Account → Card → CardInvoice → Transaction → TransactionTag, etc.). Server validates FKs after batch insert in a single transaction; rejects whole batch on FK violation with the offending row id.

**Idempotency**: Each pushed row carries client UUID `id`. Re-push of same row with same `updated_at` is a no-op. Re-push with newer `updated_at` overwrites.

**Schema-level invariants** (server-enforced, not LWW):
- `Transaction.transfer_pair_id` symmetry — both legs must reference each other. Server validates on push.
- `CardInvoice.total` recomputed server-side from charges; client value ignored.
- Account `current_balance` always derived; never synced.

**Conflict scenarios accepted**:
- Concurrent edit of same Transaction `description`: last write wins, prior content recoverable from audit log.
- Concurrent edit of different fields on same Transaction: last full-row write wins → field-level loss possible. Mitigation: UI prompts user before overwriting locally-stale row by checking `updated_at` on edit submit.

**Conflict scenarios prevented by design**:
- Two devices creating distinct rows: no conflict (UUIDs differ).
- Tags on same Transaction: M:N rows have separate `(transaction_id, tag_id)` PKs; concurrent tag adds both succeed.

---

## Consequences

**Positive**:
- Implementable in days, not weeks.
- No new dependencies (Postgres + sqlx + serde sufficient).
- Audit log compensates for LWW data loss — nothing is irretrievable.
- Easy to reason about; easy to debug with SQL queries.
- Pull-based sync respects mobile battery (no push channel needed).

**Negative**:
- Field-level concurrent edits lose one side. Acceptable given single-user scope.
- No causal consistency: a Transaction may sync before its Category if pushed in the wrong order. Mitigated by topological batch ordering and FK validation.
- Re-architecting to CRDT later means migration of every entity. Cost is bounded but real.

**Migration triggers** (re-evaluate Option B if any of these occur):
- Shared/family accounts ship → multi-user concurrent edits become common.
- User reports show >1% of edits lost to conflicts.
- P2P sync without backend becomes a goal.

---

## Implementation Checklist

- [ ] Add `sync_state` table (`user_id`, `device_id`, `last_pulled_at`, `last_pushed_at`, PK on first two).
- [ ] Trigger or service-layer hook: every write stamps `updated_at = clock_timestamp()`, server-side only.
- [ ] Implement `POST /sync/push` with per-entity validators and topological order requirement.
- [ ] Implement `GET /sync/pull` with cursor, per-entity pagination, tombstone inclusion.
- [ ] Add `audit_log` insert on every accepted mutation (DB trigger preferred over service hook).
- [ ] Mobile: wrap Room DAOs with sync-queue table; flush on connectivity.
- [ ] Mobile: stale-row guard on edit screen — refetch + diff before submit if `updated_at` older than N seconds.
- [ ] Background GC job: delete tombstones older than 90 days.

---

## References

- CLAUDE.md — audit logging mandate, fixed-point money.
- DATA_MODEL.md — entity sync fields.
- Hybrid Logical Clocks paper (Kulkarni et al., 2014) — fallback if pure server time proves insufficient.
