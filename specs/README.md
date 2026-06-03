# owo specs

Spec-driven development index. Read these before code. If code disagrees with a spec, the spec wins unless a deviation is documented under `decisions/`.

## Layout

- `00-mvp-scope.md` — what is in MVP, what is explicitly out.
- `01-data-model.md` — entities, fields, types, constraints.
- `02-api.md` — REST surface for the MVP.
- `03-money.md` — money representation rule.
- `04-two-legged-transactions.md` — transaction shape and derived type.
- `05-account-types.md` — five-type enum and reservation rule.
- `06-budgets.md` — month-keyed budgets and computed spent/difference.
- `07-monthly-report.md` — monthly aggregate report.
- `08-cards-preserve.md` — preserve-and-verify rule for existing card code.
- `09-web-spa.md` — responsive web UI screens and design direction.
- `10-change-plan.md` — ordered execution plan.
- `decisions/` — ADRs for choices that deviate from a strict reading of the brief.
- `phases/` — per-phase implementation notes (filled in as work lands).

## Source documents

- `/Owo requirements brief.md` — the long-form brief (covers phases 1–3).
- `CLAUDE.md` — repo-level guidance.

The MVP scope here is narrower than the brief on purpose: spreadsheet parity first, differentiators (loans, investments, projections, full installments) later. The brief stays the long-term roadmap.
