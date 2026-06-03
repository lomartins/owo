# owo — REST API Specification

- **Version**: v1 (draft)
- **Status**: Proposed
- **Date**: 2026-05-10
- **Base path**: `/api/v1`
- **Format**: JSON, UTF-8
- **Auth**: `Authorization: Bearer <session-token>` (see ADR-0002)

---

## Conventions

### Versioning

Path-based: `/api/v1/...`. Breaking changes bump to `/api/v2`. v1 maintained ≥6 months after v2 ships.

### Identifiers

UUID v7 strings.

### Money

All monetary fields are integer minor units (cents) as JSON numbers. Currency is ISO 4217 string. Example:
```json
{ "value": 1599, "currency": "BRL" }   // R$ 15.99
```

### Timestamps

ISO-8601 UTC with `Z` suffix. Example: `"2026-05-10T14:32:11.123Z"`.

### Field storage

All fields stored as plaintext server-side per ADR-0003 (v1 self-hosted-first). HTTPS in transit is the only confidentiality layer over the network.

### Pagination

Cursor-based. Query params:
- `limit` — default 50, max 200
- `cursor` — opaque string

Response:
```json
{
  "items": [...],
  "next_cursor": "eyJ0eC...="
}
```

`next_cursor` absent → end of stream.

### Error format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "human-readable",
    "details": { "field": "email", "reason": "format" },
    "request_id": "01HX..."
  }
}
```

Codes: `UNAUTHENTICATED`, `FORBIDDEN`, `NOT_FOUND`, `VALIDATION_ERROR`, `CONFLICT`, `RATE_LIMITED`, `INTERNAL`, `FK_VIOLATION`, `STALE_WRITE`.

### Status codes

- `200` GET / mutation success with body
- `201` resource created
- `204` mutation success no body
- `400` validation
- `401` missing/invalid token
- `403` authenticated but not allowed
- `404` not found
- `409` conflict (stale `updated_at`, FK violation)
- `429` rate limited
- `500` internal

### Concurrency control

Mutations on existing rows require `If-Match: <updated_at>` header. Mismatch → `409 STALE_WRITE` with current row in body.

### Rate limits

- `POST /auth/login`: 5 / 15min / (ip, email)
- `POST /sync/push`: 60 / min / user
- Other authed endpoints: 600 / min / user

Headers: `X-RateLimit-Remaining`, `X-RateLimit-Reset`.

---

## Auth

### `POST /auth/register`

Self-host signup. Backend may disable via env (`OWO_ALLOW_REGISTRATION=false`).

```json
// request
{
  "email": "luis@example.com",
  "password": "...",            // min 12 chars
  "display_name": "Luis",
  "default_currency": "BRL",
  "locale": "pt-BR",
  "device_id": "uuid",
  "device_name": "Pixel 8"
}

// 201
{
  "user": { "id": "...", "email": "...", "display_name": "...", "default_currency": "BRL" },
  "session": { "id": "...", "token": "<raw>", "expires_at": "..." }
}
```

### `POST /auth/login`

```json
// request
{ "email": "...", "password": "...", "device_id": "...", "device_name": "..." }

// 200
{
  "user": { ... },
  "session": { "id": "...", "token": "<raw>", "expires_at": "..." }
}
```

`401 UNAUTHENTICATED` on bad credentials. Generic message; never reveal "user not found" vs "wrong password".

### `POST /auth/logout`

Revokes the current session. `204`.

### `GET /auth/me`

Returns current user.
```json
{ "user": { "id": "...", "email": "...", "display_name": "...", "default_currency": "BRL", "locale": "pt-BR" } }
```

### `PATCH /auth/me`

Update profile fields.
```json
{ "display_name": "Luis M.", "default_currency": "USD", "locale": "en-US" }
```

### `POST /auth/password`

Change password. Revokes all *other* sessions.
```json
{ "current_password": "...", "new_password": "..." }
```

### `GET /auth/sessions`

List active sessions for current user.
```json
{
  "items": [
    { "id": "...", "device_name": "Pixel 8", "ip_last_seen": "203.0.113.5",
      "user_agent": "...", "created_at": "...", "last_used_at": "...",
      "expires_at": "...", "current": true }
  ]
}
```

### `DELETE /auth/sessions/{id}`

Revoke specific session. `204`. `DELETE /auth/sessions` revokes all *except current*.

---

## Health

### `GET /health`

Public, no auth.
```json
{ "status": "ok", "version": "0.1.0", "db": "ok" }
```

### `GET /health/ready`

Readiness for orchestrators. Same shape; `503` if DB unreachable.

---

## Resource Endpoints — Common Shape

For every resource `R` in: `accounts`, `cards`, `categories`, `tags`, `transactions`, `bills`, `investments`, `loans`, `budgets`, `goals`:

| Method | Path                    | Purpose                          |
|--------|-------------------------|----------------------------------|
| GET    | `/api/v1/{R}`           | List (paginated, filterable)     |
| POST   | `/api/v1/{R}`           | Create                           |
| GET    | `/api/v1/{R}/{id}`      | Read one                         |
| PATCH  | `/api/v1/{R}/{id}`      | Partial update (If-Match)        |
| DELETE | `/api/v1/{R}/{id}`      | Soft-delete (sets `deleted_at`)  |

Common list query params:
- `archived` — `true` / `false` / `all` (default `false`)
- `updated_since` — ISO timestamp (delta queries)
- `limit`, `cursor`

---

## Accounts

### `GET /api/v1/accounts`

Filter: `type=BANK|PAYMENT|CASH`, `currency=BRL`.

```json
{
  "items": [
    {
      "id": "...",
      "name": "Itaú Conta Corrente",
      "type": "BANK",
      "currency": "BRL",
      "initial_balance": 0,
      "current_balance": 145032,        // computed
      "archived": false,
      "created_at": "...", "updated_at": "..."
    }
  ]
}
```

### `POST /api/v1/accounts`

```json
{ "name": "...", "type": "BANK", "currency": "BRL", "initial_balance": 0 }
```

### `GET /api/v1/accounts/{id}/balance`

Computed balance with breakdown.
```json
{
  "account_id": "...",
  "currency": "BRL",
  "initial_balance": 0,
  "transactions_total": 145032,
  "current_balance": 145032,
  "as_of": "2026-05-10T14:32:11.123Z"
}
```

---

## Cards

### `GET /api/v1/cards`

Filter: `account_id`, `type=CREDIT|DEBIT`.

```json
{
  "items": [
    {
      "id": "...",
      "account_id": "...",
      "last_four_digits": "1234",
      "brand": "VISA",
      "type": "CREDIT",
      "limit": 500000,
      "remaining": 423100,           // computed from open invoice
      "close_day": 25,
      "due_day": 5,
      "archived": false
    }
  ]
}
```

### `GET /api/v1/cards/{id}/invoices`

```json
{
  "items": [
    { "id": "...", "period_start": "...", "period_end": "...", "due_date": "...",
      "total": 76900, "paid_at": null, "paid_transaction_id": null }
  ]
}
```

### `POST /api/v1/cards/{id}/invoices/{invoice_id}/pay`

Marks invoice paid. Creates linked Transaction.
```json
// request
{ "account_id": "...", "tx_date": "2026-05-05", "value": 76900 }

// 201
{ "invoice": { ... }, "transaction": { ... } }
```

---

## Categories

### `GET /api/v1/categories`

Filter: `kind=INCOME|EXPENSE|BOTH`, `parent_id`.

```json
{
  "items": [
    { "id": "...", "name": "Food", "parent_id": null, "icon": "utensils",
      "color": "#f59e0b", "kind": "EXPENSE", "archived": false }
  ]
}
```

---

## Tags

### `GET /api/v1/tags`

```json
{ "items": [ { "id": "...", "name": "vacation", "color": "#3b82f6" } ] }
```

---

## Transactions

### `GET /api/v1/transactions`

Cursor-paginated. Newest first by default.

**Query params**:

| Param            | Type     | Default      | Notes                                                       |
|------------------|----------|--------------|-------------------------------------------------------------|
| `cursor`         | string   | —            | Opaque cursor from previous response. Omit for first page.  |
| `limit`          | int      | 50           | Max 200.                                                    |
| `order`          | enum     | `desc`       | `desc` \| `asc` — by `(tx_date, id)`. Must match across page calls. |
| `account_id`     | uuid     | —            | Filter.                                                     |
| `card_id`        | uuid     | —            | Filter.                                                     |
| `category_id`    | uuid     | —            | Filter.                                                     |
| `tag_id`         | uuid[]   | —            | Repeatable. Matches if tx has any of the listed tags.       |
| `type`           | enum     | —            | `INCOME` \| `EXPENSE` \| `TRANSFER`                          |
| `payment_method` | enum     | —            | `PIX` \| `DEBIT` \| `CREDIT` \| `TED` \| `BOLETO` \| `CASH`  |
| `from`           | date     | —            | `tx_date >= from` (ISO `YYYY-MM-DD`).                       |
| `to`             | date     | —            | `tx_date <= to`.                                            |
| `min_value`      | int      | —            | Minor units, signed.                                        |
| `max_value`      | int      | —            | Minor units, signed.                                        |
| `q`              | string   | —            | Text search on description. Capped at 200 results.          |

**Cursor encoding**:

Opaque to clients. Server-side: base64url of `{ "tx_date": "YYYY-MM-DD", "id": "<uuid>", "order": "desc", "filters_hash": "<sha256-prefix>" }`.

- `(tx_date, id)` is the seek key — stable composite ordering, no ties.
- `filters_hash` is a hash of the active filter set; if client re-uses a cursor with different filters → `400 VALIDATION_ERROR` (`code: CURSOR_FILTER_MISMATCH`). Forces explicit re-paginate.
- `order` reuse must match.

**Seek query** (server, `desc` order):
```sql
SELECT *
FROM transaction
WHERE user_id = $1
  AND deleted_at IS NULL
  AND (tx_date, id) < ($cursor_tx_date, $cursor_id)
  -- + filters
ORDER BY tx_date DESC, id DESC
LIMIT $limit + 1;
```

Server reads `limit + 1` to detect more pages without `COUNT(*)`. If `limit + 1` rows returned, drop the last and emit `next_cursor` from the new last item.

**Response**:
```json
{
  "items": [
    {
      "id": "01HX...",
      "account_id": "...",
      "card_id": null,
      "category_id": "...",
      "type": "EXPENSE",
      "payment_method": "PIX",
      "value": -4500,
      "currency": "BRL",
      "fx_rate": null,
      "description": "Almoço",         // plaintext per ADR-0003
      "tx_date": "2026-05-09",
      "receipt_url": null,
      "picture_url": null,
      "transfer_pair_id": null,
      "bill_id": null,
      "invoice_id": null,
      "tag_ids": ["..."],
      "created_at": "...",
      "updated_at": "..."
    }
  ],
  "page": {
    "limit": 50,
    "order": "desc",
    "returned": 50,
    "has_more": true,
    "next_cursor": "eyJ0eF9kYXRlIjoiMjAyNi0wNS0wOSIsImlkIjoiMDFIWC4uLiIsIm9yZGVyIjoiZGVzYyIsImZpbHRlcnNfaGFzaCI6ImE5ZjIifQ"
  }
}
```

`page.next_cursor` absent (or `null`) when end of stream. `page.has_more` mirrors that boolean for clients that don't want to inspect the cursor.

**Required index** (DB):
```sql
CREATE INDEX tx_user_date_id ON transaction (user_id, tx_date DESC, id DESC)
WHERE deleted_at IS NULL;
```

**Why cursor and not offset**:
- New rows arriving between page calls don't shift existing pages (offset would skip/duplicate).
- `O(log n)` per page; offset is `O(offset)`.
- Aligns with sync protocol cursor semantics (ADR-0001).

**Failure modes**:
- Stale cursor (row referenced by cursor was deleted): seek still works — predicate is `<` on `(tx_date, id)`, doesn't require the cursor row to exist.
- Filter set changed: `400 CURSOR_FILTER_MISMATCH`. Client must restart pagination.
- Cursor older than retention horizon (90d post-tombstone): still safe — only data missing is fully-purged tombstones, which the seek wouldn't return anyway.

### `POST /api/v1/transactions`

```json
{
  "account_id": "...",
  "card_id": null,
  "category_id": "...",
  "type": "EXPENSE",
  "payment_method": "PIX",
  "value": -4500,
  "currency": "BRL",
  "description": "Almoço",
  "tx_date": "2026-05-09",
  "tag_ids": ["..."]
}
```

### `POST /api/v1/transactions/transfer`

Atomic two-leg transfer. Creates two linked Transactions (`type=TRANSFER`) with mutual `transfer_pair_id`.

```json
// request
{
  "from_account_id": "...",
  "to_account_id": "...",
  "value": 50000,                       // positive; sign applied per leg
  "currency": "BRL",
  "fx_rate": null,                      // required if accounts differ in currency
  "tx_date": "2026-05-09",
  "description": "Transfer to savings"
}

// 201
{
  "outbound": { /* Transaction value=-50000 */ },
  "inbound":  { /* Transaction value=+50000 */ }
}
```

### `POST /api/v1/transactions/{id}/receipt`

Multipart upload of receipt image/PDF. Backend encrypts and stores blob.
```
Content-Type: multipart/form-data
file: <binary>
```
```json
// 200
{ "transaction": { "id": "...", "receipt_url": "/api/v1/blobs/<id>", ... } }
```

### `GET /api/v1/blobs/{id}`

Streams blob to authorized owner. `Content-Type` from manifest.

---

## Bills

### `GET /api/v1/bills`

Filters: `paid=true|false`, `due_before`, `due_after`, `recurrence`.

```json
{
  "items": [
    {
      "id": "...",
      "account_id": "...",
      "category_id": "...",
      "description": "Netflix",
      "value": 5590,
      "currency": "BRL",
      "due_date": "2026-05-15",
      "recurrence": "MONTHLY",
      "recurrence_interval": 1,
      "recurrence_end": null,
      "paid": false,
      "paid_at": null,
      "payment_receipt_url": null,
      "paid_transaction_id": null
    }
  ]
}
```

### `POST /api/v1/bills/{id}/pay`

Marks bill paid; creates linked Transaction; if recurring, generates next instance.

```json
// request
{ "account_id": "...", "tx_date": "2026-05-15", "value": 5590 }

// 201
{ "bill": { ... }, "transaction": { ... }, "next_bill": { ... } | null }
```

---

## Investments

### `GET /api/v1/investments`

Filter: `account_id`, `type=FIXED|VARIABLE`, `archived`.

```json
{
  "items": [
    {
      "id": "...",
      "account_id": "...",
      "name": "CDB Banco X",
      "type": "FIXED",
      "principal": 1000000,
      "current_value": 1023400,
      "currency": "BRL",
      "rate_index": "CDI",
      "rate_spread": "1.05",          // decimal string
      "purchase_date": "2026-01-15",
      "expiration_date": "2027-01-15"
    }
  ]
}
```

### `PATCH /api/v1/investments/{id}/value`

Update `current_value` (variable instruments).
```json
{ "current_value": 1024100 }
```

---

## Loans

### `GET /api/v1/loans`

```json
{
  "items": [
    {
      "id": "...",
      "account_id": "...",
      "description": "Car loan",
      "total_value": 5000000,
      "currency": "BRL",
      "interest_rate": "1.99",
      "installment_value": 89500,
      "installment_count": 60,
      "paid_installments": 12,
      "remaining_balance": 4297600,    // computed
      "start_date": "2025-05-15",
      "first_due_date": "2025-06-15"
    }
  ]
}
```

### `POST /api/v1/loans/{id}/pay-installment`

```json
// request
{ "account_id": "...", "tx_date": "2026-05-15", "value": 89500 }

// 201
{ "loan": { ..., "paid_installments": 13 }, "transaction": { ... } }
```

---

## Budgets

### `GET /api/v1/budgets`

Filter: `period=MONTHLY|YEARLY`, `category_id`.

```json
{
  "items": [
    {
      "id": "...",
      "category_id": "...",
      "period": "MONTHLY",
      "limit": 80000,
      "currency": "BRL",
      "start_date": "2026-05-01",
      "end_date": null,
      "spent_current_period": 32100,   // computed
      "remaining_current_period": 47900
    }
  ]
}
```

---

## Goals

### `GET /api/v1/goals`

```json
{
  "items": [
    {
      "id": "...",
      "name": "Emergency fund",
      "target_value": 3000000,
      "currency": "BRL",
      "current_value": 1450320,
      "account_id": "...",
      "target_date": "2027-01-01",
      "progress_pct": 48.34
    }
  ]
}
```

---

## Reports

### `GET /api/v1/reports/cash-flow`

Query: `from`, `to`, `granularity=DAY|WEEK|MONTH`, `account_id?`, `currency?`.

```json
{
  "currency": "BRL",
  "granularity": "MONTH",
  "buckets": [
    { "period": "2026-04", "income": 850000, "expense": -612300, "net": 237700 },
    { "period": "2026-05", "income": 0,      "expense": -41200,  "net": -41200 }
  ]
}
```

### `GET /api/v1/reports/by-category`

Query: `from`, `to`, `kind=INCOME|EXPENSE`.

```json
{
  "currency": "BRL",
  "from": "...", "to": "...",
  "items": [
    { "category_id": "...", "category_name": "Food", "total": -41200, "count": 18 }
  ]
}
```

### `GET /api/v1/reports/net-worth`

```json
{
  "currency": "BRL",
  "as_of": "2026-05-10T...",
  "accounts_total": 1450320,
  "investments_total": 1023400,
  "loans_total": -4297600,
  "credit_card_debt": -76900,
  "net_worth": -1900780
}
```

---

## Sync

Per ADR-0001. Topological order required on push.

### `POST /api/v1/sync/push`

```json
// request
{
  "device_id": "...",
  "batch": {
    "users":         [],
    "accounts":      [ { "id": "...", "updated_at": "...", ... } ],
    "categories":    [],
    "tags":          [],
    "cards":         [],
    "card_invoices": [],
    "transactions":  [],
    "transaction_tags": [],
    "bills":         [],
    "investments":   [],
    "loans":         [],
    "budgets":       [],
    "goals":         []
  }
}

// 200
{
  "accepted": {
    "accounts": [ { "id": "...", "updated_at": "<server-stamped>" } ]
  },
  "rejected": [
    { "entity": "transactions", "id": "...", "code": "FK_VIOLATION", "detail": "category_id ... not found" }
  ],
  "server_time": "2026-05-10T14:32:11.123Z"
}
```

Whole-batch reject on FK violation: response `409` with `rejected` populated and no `accepted`.

### `GET /api/v1/sync/pull`

Query: `since` (ISO timestamp), `limit` (default 500, max 2000), `cursor?`.

```json
{
  "server_time": "...",
  "next_cursor": null,
  "changes": {
    "accounts":      [ { ... } ],
    "categories":    [],
    "tags":          [],
    "cards":         [],
    "card_invoices": [],
    "transactions":  [],
    "transaction_tags": [],
    "bills":         [],
    "investments":   [],
    "loans":         [],
    "budgets":       [],
    "goals":         []
  }
}
```

Tombstones included with `deleted_at` set.

---

## Backup

### `POST /api/v1/backup/create`

Generates a backup bundle (per ADR-0003). Long-running — returns job id.

```json
// request — encrypt is optional; default plain tar+gzip
{ "encrypt": false }
// or with operator passphrase
{ "encrypt": true, "passphrase": "...." }

// 202
{ "job_id": "...", "status_url": "/api/v1/backup/jobs/..." }
```

### `GET /api/v1/backup/jobs/{id}`

```json
{ "id": "...", "status": "RUNNING|DONE|FAILED", "download_url": null, "expires_at": null }
```

### `GET /api/v1/backup/download/{job_id}`

Streams encrypted bundle. `Content-Type: application/octet-stream`. One-shot URL; expires 1h after creation.

### `POST /api/v1/backup/restore`

Multipart with bundle + passphrase. Replaces current user data. Destructive — requires fresh login + confirmation header `X-Confirm-Restore: yes`.

---

## CSV Import (deferred — placeholder)

### `POST /api/v1/import/csv`

Multipart CSV + mapping config. Returns staged Transactions for user review before commit.

```json
// 200
{ "import_id": "...", "preview_count": 120, "preview_url": "/api/v1/import/{id}" }
```

### `POST /api/v1/import/{id}/commit`

Commits staged rows.

---

## Open Finance (deferred)

Out of scope v1. Reserved namespace: `/api/v1/integrations/open-finance/...`.

---

## Audit Log

### `GET /api/v1/audit`

Read-only. Filters: `entity_type`, `entity_id`, `action`, `from`, `to`.

```json
{
  "items": [
    { "id": "...", "entity_type": "Transaction", "entity_id": "...",
      "action": "UPDATE", "diff": { "description": ["old","new"] },
      "timestamp": "...", "device_id": "..." }
  ],
  "next_cursor": "..."
}
```

---

## Endpoint Summary

| Group         | Count |
|---------------|-------|
| Auth          | 9     |
| Accounts      | 6     |
| Cards         | 7     |
| Categories    | 5     |
| Tags          | 5     |
| Transactions  | 7     |
| Bills         | 6     |
| Investments   | 6     |
| Loans         | 6     |
| Budgets       | 5     |
| Goals         | 5     |
| Reports       | 3     |
| Sync          | 2     |
| Backup        | 4     |
| Import        | 2     |
| Audit         | 1     |
| Health        | 2     |
| **Total**     | **81** |

---

## References

- ADR-0001 — sync protocol drives `/sync/*` shape.
- ADR-0002 — auth drives `/auth/*` and `Authorization` header.
- ADR-0003 — encryption drives blob streaming + backup endpoints.
- DATA_MODEL.md — entity field definitions.
