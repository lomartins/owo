# ADR 0002 — Account types: collapse legacy BANK/PAYMENT/CASH → asset

Status: Accepted (2026-05-27)

## Context

The existing schema defines `accounts.type IN ('BANK', 'PAYMENT', 'CASH')`. The spec calls for `('asset', 'credit_card', 'liability', 'revenue', 'expense')`. The legacy enum is finer-grained on the "everyday money" side but lacks the accounting buckets (`revenue` / `expense`) the two-legged transaction model requires.

## Decision

- Legacy BANK / PAYMENT / CASH all map to `asset`.
- The "kind of asset" distinction (bank account vs. payment app vs. cash drawer) moves out of the type column. If the user wants to see "wallet" vs "checking", it's the account's `name`, optionally an `icon` we can add later — not a typed enum.
- `credit_card` / `liability` are reserved (created only via internal flows — card binding in Phase C; loans later).
- `revenue` / `expense` are system buckets, one per user, hidden from the user-facing account list.

## Why

- The MVP doesn't render anything that branches on bank-vs-payment-vs-cash.
- Future features (multi-currency dashboards, fee tracking) treat any asset uniformly.
- Adding subtype as data (an `icon` or `subtype` field) is cheaper than carrying a type-flavoured enum forever.

## Migration

`UPDATE accounts SET type = 'asset' WHERE type IN ('BANK','PAYMENT','CASH');` then rewrite CHECK constraint. Reversible by re-inferring from `name` (low fidelity) if we ever need it.

## Trade-offs

- Loses the small UI affordance of "this is a payment app" vs "this is a bank". Acceptable; spreadsheet doesn't make that distinction either.

## Rejected alternatives

- **Add new types alongside legacy.** Keeps 8 enum values (3 legacy + 5 new) forever. Confusing, no benefit.
- **Add `subtype` column now.** Premature. Land it when a UI actually needs it.
