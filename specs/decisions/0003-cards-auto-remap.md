# ADR 0003 — Cards: auto-create credit_card account per card and remap

Status: Accepted (2026-05-27)

## Context

The current schema has `cards` rows pointing at `accounts.id` of an `asset` account (the user's checking). Per the spec, card purchases should source from a `credit_card` account, not from checking. We need to bridge the gap without losing existing card / invoice / transaction history.

## Decision

A migration auto-creates one `credit_card`-type `accounts` row per existing `cards` row and remaps:

1. New `accounts` row per card. Name = `brand + " " + last_four_digits`. `initial_balance = 0`. Currency inherited.
2. `cards.account_id` updated to point at the new credit_card account.
3. For every transaction with `card_id IS NOT NULL`: `source_account_id = new_credit_card_account_id`, `destination_account_id = user.expense_account_id`, `category_id` left untouched.
4. For every invoice-payment transaction: `source_account_id = checking`, `destination_account_id = new_credit_card_account_id`, `category_id = NULL`. Ambiguous cases (no clear checking, no `paid_transaction_id` chain) are flagged via `audit_log` action `MIGRATION_NEEDS_REVIEW` for manual fix-up.

## Why

- Single migration, no user action. Auto-rebinds existing data to the spec-correct shape.
- Card metadata (last 4, brand, limit, close/due days) stays put — we don't lose it.
- The cards code path is preserved; the only visible behaviour change is that card-tx queries now naturally aggregate per credit_card account instead of via a `card_id` filter on top of an asset account.

## Trade-offs

- Migration is irreversible in practice. Backup the SQLite file before running. Document in `phases/C-cards.md`.
- Ambiguous invoice payments need manual review. Expected to be rare; audit_log captures them so they aren't silently lost.

## Rejected alternatives

- **Add a second account pointer on `cards`** (`credit_card_account_id` alongside `account_id`). Two pointers per card is wiring debt forever, and downstream queries need to remember which one to use. Worse than the migration.
- **Defer to manual remap via API.** Slows MVP; user would need a separate "fix my cards" flow before card features work.
