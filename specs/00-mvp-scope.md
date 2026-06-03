# 00 — MVP scope

Goal: spreadsheet parity. The user's existing budgeting spreadsheet is the source of truth for MVP scope. Nothing more, nothing less.

## In scope

1. Multiple monthly income entries per month.
2. Monthly budget per category: `estimated`, `spent` (computed), `difference` (computed).
3. Variable expense entries: date, description, category, payment method, paid flag, amount.
4. Fixed expenses: estimated vs actual, with payment method.
5. Month carry-over: previous month's closing balance feeds the next month.
6. Monthly summary: total income, total spent, budget balance, carry-over in/out.
7. Responsive web SPA covering: monthly dashboard, transaction entry, transaction list, budget editor, month switcher.

## Out of scope (do NOT build now)

- Loans (amortization, Price/SAC, shrinking balance).
- Investments (tracking, projection, IR).
- Bank integration (Pluggy / Open Finance), CSV/OFX import.
- Multi-user features beyond the existing single-user-with-token design.
- TUI dashboard.
- CLI (mobile / web are the clients).

## Preserve-and-verify (do NOT delete if present)

Existing repo contains schema and stubs for credit cards, card invoices, installments-on-loans, bills, investments, loans, goals, sync, backup, audit. These are kept because they encode forward-compat. They may be stubbed (501) but tables and domain types stay. See `08-cards-preserve.md` for the specific verification rule on cards.

## Non-negotiables (architectural)

- Money is `rust_decimal::Decimal` at every API boundary. Storage stays `INTEGER` cents per ADR `decisions/0001-money-at-api-boundary.md`.
- Multi-row writes are atomic (one SQL transaction).
- All financial rules live server-side. Clients render and submit; they never compute balances, totals, or differences.
- Token-based auth (existing scheme stands).
- Single binary serves API + static SPA on one port.
