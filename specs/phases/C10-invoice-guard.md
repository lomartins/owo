# Phase C10 — Invoice-payment must be category-less

Status: shipped (2026-05-27)

## What shipped

- Migration `0007_invoice_payment_guard.sql`: BEFORE INSERT and BEFORE UPDATE triggers on `transactions`. If `category_id IS NOT NULL` AND `destination` is a `credit_card` account AND `source` is an `asset` account, the trigger aborts with the message `INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY`.
- Service-layer guard: `api::transactions::validate_leg_pair` already rejects a category on any transfer (asset/credit_card/liability ↔ asset/credit_card/liability) with `400 Validation`. The trigger is a backstop.

## Verification

- `cargo check` clean.
- The trigger uses `RAISE(ABORT, ...)` so the message surfaces in the SQLx error chain as a constraint violation; clients see a generic 500 (the service-layer guard is the user-facing path).

## Deviations

- The trigger only fires on `(asset → credit_card)`. Transfers `credit_card → credit_card` and `liability → credit_card` are not the spec's invoice-payment case; they are ordinary transfers and the service-layer no-category-on-transfer rule still applies.

## Follow-ups

- A `Conflict`-style ApiError variant could be added to map SQLite constraint messages to clean 400s, but the service-layer guard already returns 400 with a clear field/reason, so this is cosmetic.
