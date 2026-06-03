# 10 — Change plan

Ordered. Foundational refactors first, then MVP features, then card preserve-and-verify, then the web SPA.

Each step lists the artifacts it produces and the verification gate it must pass before the next step starts.

## Phase A — Foundational

### A1 — Money at the API boundary
- Spec: `03-money.md`. ADR: `decisions/0001-money-at-api-boundary.md`.
- Code: add `rust_decimal` to `backend/Cargo.toml`. New `domain::common::Money` wrapper.
- Touch: `domain/{account,transaction,budget,bill,card,investment,loan,goal,common}.rs`. API request/response structs. Every `bind(i64)` for a money column → bind from `Money::to_cents()`. Every `FromRow` → wrap into `Money`.
- Verify: `cargo clippy --all-targets`, `cargo test`, plus `grep -rn 'f64' backend/src/` returns no money-related hits.

### A2 — Two-legged transactions
- Spec: `04-two-legged-transactions.md`.
- Migration: `0002_two_legged_tx.sql` — add `source_account_id`, `destination_account_id`, backfill, drop `transfer_pair_id`. See spec for backfill rules.
- Code: `domain::transaction::{Transaction,CreateTransaction}` shape changes. `api::transactions::{create,update,list,transfer}` rewritten. Derived `type` becomes computed, not stored.
- Verify: a manual round-trip — create asset → create deposit → create withdrawal → balance reflects both. Cursor pagination still works on the renamed column index.

### A3 — Account types
- Spec: `05-account-types.md`. ADR: `decisions/0002-account-types-mapping.md`.
- Migration: `0003_account_types.sql` — rewrite CHECK; map BANK/PAYMENT/CASH → asset; add unique partial indexes for the `revenue`/`expense` singletons; insert the two bucket accounts per existing user.
- Code: `api::accounts::create` rejects `RESERVED_TYPE` for `credit_card`/`liability`/`revenue`/`expense` in MVP. Auth register handler also inserts the two buckets.
- Verify: list accounts excludes `revenue`/`expense` from the user-facing list (handler filters them out).

### A4 — Payment methods
- Migration: `0004_payment_methods.sql` — extend CHECK to include `VA`. Done.

## Phase B — Spreadsheet parity features

### B5 — Budgets refactor
- Spec: `06-budgets.md`.
- Migration: `0005_budgets_month.sql` — add `month TEXT`, backfill `month = substr(start_date, 1, 7)`, drop `period`, `start_date`, `end_date`. Unique on `(user_id, category_id, month)`.
- Code: domain + handlers (list, upsert via POST, PUT, DELETE). Spent/difference computed in SQL.
- Verify: a category with no budget row returns `estimated = "0.00"`. A category over budget returns negative `difference`.

### B6 — Monthly report
- Spec: `07-monthly-report.md`.
- Code: replace `reports::cash_flow` 501 with the unified `/reports/monthly`. Keep the other endpoints registered (stub or remove from router; prefer stub so future phases can fill them).

### B7 — Category seed
- Code: on `auth::register`, insert the 10 MVP categories within the same tx as the user/buckets creation.

### B8 — Balance as-of-date
- Code: `accounts::balance` accepts `?as_of=`. SQL adds `WHERE tx_date <= ?`.

## Phase C — Card preserve-and-verify

### C9 — Auto-create credit_card account per card
- Spec: `08-cards-preserve.md`.
- Migration: `0006_card_account_binding.sql`. Backfill logic per the spec. Flag ambiguous rows to `audit_log`.
- Code: card creation API also inserts the underlying `credit_card` account. Existing tests run.

### C10 — Invoice-payment guard
- Migration: `0007_invoice_payment_guard.sql` — INSERT and UPDATE triggers.
- Code: service-layer guard returns 400 `INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY` ahead of the trigger.

## Phase D — Web SPA

### D — Five screens, responsive
- Spec: `09-web-spa.md`.
- Pre-step: read `frontend-design` SKILL.md, commit to a direction.
- Build: `web/` directory with chosen stack. Output to `web/dist/`.
- Wire: Axum `ServeDir` at `/` with SPA fallback to `index.html`.
- Verify: open in a browser, log in, navigate all 5 screens at phone width + desktop width.

## Out-of-MVP, intentionally untouched

- `cards` / `card_invoices` / `loans` / `investments` / `goals` / `sync` / `backup` endpoints: stay 501 or stay as-is. Do not delete tables. See `00-mvp-scope.md` preserve clause.

## Verification gates (per phase)

After each phase: `cargo clippy --all-targets`, `cargo test`, manual API round-trip on the new endpoints. The web SPA additionally requires a real browser session for the golden-path flow (login → enter income → enter expense → see dashboard update).
