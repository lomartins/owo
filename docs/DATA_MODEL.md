# owo — Data Model

All monetary values stored as `i64 minor_units` (cents). No floats.
All timestamps `TIMESTAMPTZ` (UTC).
All entities include sync fields: `created_at`, `updated_at`, `deleted_at?`, `sync_version`, `device_id`.

---

## User

```
User(
    id: UUID,
    email: String UNIQUE,
    password_hash: String,           // Argon2id
    display_name: String,
    default_currency: String,        // ISO 4217 (BRL, USD)
    locale: String,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Account

```
Account(
    id: UUID,
    user_id: FK User,
    name: String,
    type: Enum(BANK, PAYMENT, CASH),
    currency: String,                // ISO 4217
    initial_balance: i64,            // opening balance
    archived: Bool,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```
Current balance = `initial_balance + sum(transactions)`. Computed, never stored.

## Card

```
Card(
    id: UUID,
    user_id: FK User,
    account_id: FK Account,
    last_four_digits: String,
    brand: Enum(VISA, MASTERCARD, ELO, AMEX, OTHER),
    type: Enum(CREDIT, DEBIT),
    limit: i64,                      // null for debit
    close_day: u8,                   // day of month invoice closes
    due_day: u8,                     // day of month invoice due
    archived: Bool,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```
`remaining` computed from open invoice cycle.

## Category

```
Category(
    id: UUID,
    user_id: FK User,
    name: String,
    parent_id: FK Category?,         // hierarchy
    icon: String?,
    color: String?,                  // hex
    kind: Enum(INCOME, EXPENSE, BOTH),
    archived: Bool,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Tag

```
Tag(
    id: UUID,
    user_id: FK User,
    name: String,
    color: String?,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## TransactionTag (M:N)

```
TransactionTag(
    transaction_id: FK Transaction,
    tag_id: FK Tag,
    PRIMARY KEY (transaction_id, tag_id)
)
```

## Transaction

```
Transaction(
    id: UUID,
    user_id: FK User,
    account_id: FK Account,
    card_id: FK Card?,
    category_id: FK Category?,
    type: Enum(INCOME, EXPENSE, TRANSFER),
    payment_method: Enum(PIX, DEBIT, CREDIT, TED, BOLETO, CASH),
    value: i64,                      // signed: +income, -expense
    currency: String,
    fx_rate: Decimal?,               // if currency != account.currency
    description: String,
    tx_date: Date,                   // when it happened
    receipt_url: String?,
    picture_url: String?,
    transfer_pair_id: FK Transaction?, // links the other leg of transfer
    bill_id: FK Bill?,               // if paying a bill
    invoice_id: FK CardInvoice?,     // if credit card charge
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Transfer

Modeled as two linked Transactions (`type=TRANSFER`) with mutual `transfer_pair_id`. No separate table.

## CardInvoice

```
CardInvoice(
    id: UUID,
    user_id: FK User,
    card_id: FK Card,
    period_start: Date,              // close_day prev cycle
    period_end: Date,                // close_day this cycle
    due_date: Date,
    total: i64,                      // sum of charges
    paid_at: Timestamp?,
    paid_transaction_id: FK Transaction?,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Bill

Covers utilities, subscriptions, any recurring/one-off scheduled expense.

```
Bill(
    id: UUID,
    user_id: FK User,
    account_id: FK Account?,         // pays from
    category_id: FK Category?,
    description: String,
    value: i64,
    currency: String,
    due_date: Date,                  // current/next due
    recurrence: Enum(NONE, WEEKLY, MONTHLY, YEARLY, CUSTOM),
    recurrence_interval: u8?,        // every N units
    recurrence_end: Date?,
    paid: Bool,
    paid_at: Timestamp?,
    payment_receipt_url: String?,
    paid_transaction_id: FK Transaction?,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Investment

```
Investment(
    id: UUID,
    user_id: FK User,
    account_id: FK Account,
    name: String,
    type: Enum(FIXED, VARIABLE),
    principal: i64,                  // amount invested
    current_value: i64,              // for variable; updated periodically
    currency: String,
    rate_index: Enum(NONE, CDI, IPCA, SELIC, FIXED_RATE)?,
    rate_spread: Decimal?,           // e.g. CDI + 2%
    purchase_date: Date,
    expiration_date: Date?,
    archived: Bool,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Loan

```
Loan(
    id: UUID,
    user_id: FK User,
    account_id: FK Account,
    description: String,
    total_value: i64,
    currency: String,
    interest_rate: Decimal,          // annual %
    installment_value: i64,
    installment_count: u16,
    paid_installments: u16,
    start_date: Date,
    first_due_date: Date,
    archived: Bool,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```
`remaining_balance` computed from `total_value - sum(paid)`.

## Budget

```
Budget(
    id: UUID,
    user_id: FK User,
    category_id: FK Category,
    period: Enum(MONTHLY, YEARLY),
    limit: i64,
    currency: String,
    start_date: Date,
    end_date: Date?,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Goal

```
Goal(
    id: UUID,
    user_id: FK User,
    name: String,
    target_value: i64,
    currency: String,
    current_value: i64,              // computed from linked account or manual
    account_id: FK Account?,         // optional: track via account
    target_date: Date?,
    created_at, updated_at, deleted_at?,
    sync_version, device_id
)
```

## Session (auth)

```
Session(
    id: UUID,
    user_id: FK User,
    token_hash: String,
    device_id: String,
    expires_at: Timestamp,
    created_at, last_used_at
)
```

## AuditLog

```
AuditLog(
    id: UUID,
    user_id: FK User,
    entity_type: String,             // "Transaction", "Account", etc.
    entity_id: UUID,
    action: Enum(CREATE, UPDATE, DELETE),
    diff: JSON,                      // before/after
    timestamp: Timestamp,
    device_id: String
)
```

---

## Indexes (initial)

- `Transaction(user_id, tx_date DESC)`
- `Transaction(account_id, tx_date DESC)`
- `Transaction(category_id)`
- `Bill(user_id, due_date)` WHERE `paid=false`
- `CardInvoice(card_id, period_end)`
- `AuditLog(entity_type, entity_id)`
- All `(user_id, updated_at)` for sync delta queries

## Locked Decisions (v1)

1. **Conflict resolution**: Row-level last-write-wins on server-stamped `updated_at`. Tie-break by `device_id` ASC. See ADR-0001.
2. **FX rates**: `fx_rate` stored per-Transaction. No global rate table. Historical accuracy preserved; no drift on later rate updates.
3. **Soft delete**: All synced entities use `deleted_at` tombstones. Hard delete only via 90-day GC job. Required by ADR-0001 sync protocol.
4. **Multi-user shared accounts**: Out of scope v1. Every entity scoped by `user_id`. Sharing deferred until single-user UX ships.
5. **No application-level field encryption (v1)** per ADR-0003 v2. All fields stored plaintext. Confidentiality at rest is host-level (LUKS recommended). Confidentiality in transit is HTTPS.
6. **Money type**: `i64` minor units (cents). No floats anywhere. Per CLAUDE.md.
7. **Time type**: `TIMESTAMPTZ` UTC. Server stamps `updated_at` on accept; client value is hint only.
8. **IDs**: UUID v7 (time-ordered) preferred over v4 — gives index locality without exposing creation time precisely.
9. **Currency duplication**: `currency` stored on `Account`, `Transaction`, `Investment`, `Loan`. Transaction may differ from account (cross-currency); `fx_rate` records the conversion at tx time.
10. **Auth session shape**: Refined in ADR-0002 — `Session` table includes `token_hash BYTEA UNIQUE`, `device_name`, `user_agent`, `ip_last_seen`, `revoked_at`. Supersedes the minimal Session block above.
11. **User table — auth fields only** per ADR-0003 v2. No `wrapped_dek` / `dek_nonce` / `crypto_version`. Auth-related state retained: `password_hash`, `failed_login_count`, `lockout_until`, `is_admin`.

## Sync Infrastructure Tables

Required by ADR-0001:

```
sync_state(
    user_id: FK User,
    device_id: String,
    last_pulled_at: Timestamp,
    last_pushed_at: Timestamp,
    PRIMARY KEY (user_id, device_id)
)
```

`audit_log` already defined above. Server-only — never synced to clients.
