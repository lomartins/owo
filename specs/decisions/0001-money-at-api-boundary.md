# ADR 0001 — Money: Decimal at API boundary, i64 cents in storage

Status: Accepted (2026-05-27)

## Context

The owo brief mandates `rust_decimal::Decimal` end-to-end with `NUMERIC(14,2)` storage. The repo is currently SQLite (ADR-0003 v2 of the original project deferred Postgres). SQLite has no native decimal; the column type maps to either `INTEGER` (current: cents) or `TEXT` (Decimal as string).

## Decision

- API boundary uses `rust_decimal::Decimal` for all money values.
- Storage stays `INTEGER` cents.
- A `Money` wrapper handles `Decimal ↔ i64 cents` at the storage adapter.

## Why

- The brief's hard rule is "no floats". `Decimal` at the boundary satisfies that for clients, the network, all in-memory math, and serialization.
- Existing schema and existing handlers all use `INTEGER` cents. Migrating storage to TEXT in SQLite buys us nothing for the MVP (sqlx Decimal-as-TEXT is workable but slower for SUM aggregates and forces query reshape).
- When Postgres lands, columns become `NUMERIC(14, 2)`. The `Money` wrapper survives unchanged; only the bind/extract conversions change. That's a one-day swap.

## Trade-offs

- Aggregate SUMs still compute in cents and convert at the handler. Acceptable.
- Two units to think about (Decimal vs cents). Mitigated by routing every conversion through `Money`.

## Rejected alternatives

- **Decimal as TEXT now.** Spec-strict but high disruption to existing handlers and aggregate queries, with no payoff before Postgres lands.
- **Skip the refactor.** Leaves the `f64` risk only theoretical (no floats are currently in use), but violates the non-negotiable rule of the brief and locks us into the cents-only API forever.

## Verification

- `grep -rn 'f64' backend/src/` returns no money-related hits.
- API responses for money fields are JSON strings (e.g. `"1234.56"`), not numbers.
