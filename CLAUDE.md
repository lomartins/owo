# CLAUDE.md

Guidance for Claude Code working in this repo. Read it before editing.

## Project

**owo** (owó, Yoruba for "money") is a self-hosted personal finance app for Luisa & Mavê. Single user (token-auth, multi-user-ready), one binary serves API + SPA. Brand is rooted in Afro-Brazilian culture (cowrie shells / búzios, terracotta on warm cream).

## Tech stack (actual, not aspirational)

- **Backend**: Rust + Axum + SQLx + **SQLite**. Migrations under `backend/migrations/sqlite/`. Single Tokio runtime, one process binds `0.0.0.0:8080` for LAN access.
- **Web SPA**: **SolidJS** + Vite + TypeScript + Tailwind v4 + **Geist** (Roman 400 / Medium 500 — never 700) + Material Symbols Rounded. Served as static from `web/dist/` via Axum `ServeDir`.
- **i18n**: PT-BR + EN-US. Dictionary in `web/src/lib/i18n/{en,pt}.ts`. Provider in `index.tsx`. Browser-detected default with manual toggle. Curly-brace placeholders `{{name}}`.
- **Tests**: Vitest + jsdom + `@solidjs/testing-library`. Files `*.test.{ts,tsx}` under `web/src/`. `npm test` runs the suite.
- **Mobile (KMP)**: scaffolded but deferred (see `specs/decisions/0004-mobile-deferred.md`). Web SPA is the only client for MVP+.
- **CLI**: not built. Web SPA covers user flows.

## Repo layout

```
owo/
├── backend/
│   ├── src/
│   │   ├── api/          # axum handlers (one file per resource)
│   │   ├── domain/       # request/response structs + sqlx::FromRow rows
│   │   ├── services/     # password, session, user_provisioning
│   │   ├── db/           # SqlitePool factory
│   │   └── …             # auth_middleware, audit, error, ids, pagination, state, server
│   ├── migrations/sqlite/  # 0001 → 0010 sequential, SQLite-safe
│   └── Cargo.toml
├── web/
│   ├── src/
│   │   ├── api/          # typed wrappers + bearer-token client
│   │   ├── components/   # Avatar, Cowrie, NetWorthPanel, BillEditModal, etc.
│   │   ├── lib/          # i18n, money, month, categories, categoryIcons, session
│   │   ├── routes/       # Dashboard, Transactions, Bills, AddTransaction, …
│   │   ├── styles/app.css
│   │   └── test/setup.ts
│   ├── public/brand/     # owo_mark*.svg (rounded variant is the favicon)
│   ├── package.json
│   ├── vite.config.ts    # also holds the vitest config block
│   └── vitest.config.ts  # mirrors vite block — leave both in place
├── specs/                # spec-driven docs (read these before changing a non-trivial area)
│   ├── 00..10-*.md
│   ├── decisions/ADRs
│   └── phases/A1..D
├── scripts/
│   └── import_june_2026.mjs  # idempotent importer, takes OWO_TOKEN env
├── uploads/              # profile photos (created at runtime; served at /uploads/*)
└── owo.db                # dev SQLite file (gitignored)
```

## Non-negotiables (architectural)

These are load-bearing decisions. Don't touch without ADR + user check-in.

1. **Money: `rust_decimal::Decimal` at API boundary, `i64` cents in SQLite storage.** Wire format is JSON string (`"1234.56"`). See `specs/03-money.md` and `specs/decisions/0001-money-at-api-boundary.md`. Conversion goes through `domain::common::Money` and `cents_as_decimal` serde adapters. No floats anywhere near financial values.

2. **Two-legged transactions.** One row, `source_account_id` + `destination_account_id` (both FK to `accounts`). Direction is the pair, never the sign of `value`. Derived type computed at read-time from source.type × destination.type (deposit / withdrawal / transfer). Backend infers the missing leg when the SPA sends only one + `category_id`, falling back to the user's `revenue` / `expense` bucket account. See `specs/04-two-legged-transactions.md`.

3. **Account types**: `asset, credit_card, liability, revenue, expense`. `revenue` and `expense` are per-user bucket accounts auto-provisioned at register (the "where income comes from / where spend goes" abstraction). Filtered out of the public account list. `credit_card` / `liability` are reserved for card/loan flows. Public `POST /accounts` only accepts `asset`. See `specs/05-account-types.md`.

4. **Atomicity.** Anything that writes >1 row uses `pool.begin()`. Single-row inserts (now the norm thanks to two-legged tx) are atomic by SQLite default.

5. **SQLite ALTER TABLE trap.** Never `ALTER TABLE x RENAME TO x_old` — it bends FK references in other tables to `x_old`. Always **create-new-with-suffix → drop-old → ALTER new RENAME TO x**. The new-name has no inbound FKs so nothing gets rewritten. Verified in migrations 0002–0006.

6. **Bill semantics**: every bill is a **monthly recurring template** with per-month payment state in `bill_payments`, and per-month patches in `bill_overrides`. Editing a bill takes a `scope` field (`this_month` | `this_and_next` | `all`); `all` clears overrides + patches the template + retroactively updates past `transactions` linked via `bills.id`. See `specs/06-budgets.md` (related) and migrations 0009 + 0010.

7. **All financial rules live server-side.** Clients render and submit. No client-side balance/interest/projection math.

8. **Spec wins.** `specs/` is source of truth. If code disagrees, the spec is right unless an ADR documents the deviation.

## Visual identity (brand)

Built on the owo design system bundle (terracotta `#ad4f32`, warm cream surfaces, Geist medium 500). The canonical tokens live in `web/src/styles/app.css` `@theme` block — match these names, don't reinvent.

- Dominant color: `--color-primary-600` (#ad4f32). Pink tint `--color-primary-50` (#fbeee9) for category thumbs / tinted cards.
- Surfaces: `--color-background` (#f4f1ec), `--color-surface` (white), `--color-surface-muted` (#f7f4f0), `--color-surface-sunken` (#ebe6df). **No cool grays.** Neutral ramp shares a red-yellow bias.
- Type: weight 500 is the workhorse, never 700. Tabular figures on money (`.money`, `.money-display`, `.tabular`). PT-BR formatting (`R$ 1.234,56`).
- Icons: Material Symbols Rounded only. Brand-spec category placeholder = monogram square (`.tx-icon` class — uppercase letter centered in 32×32 with primary-50 background). The custom icon picker (32 curated Material Symbols + 8 brand tints) overrides the monogram per category.
- Mark: 7 cowrie shells on terracotta disc. SVG at `web/public/brand/owo_mark{_rounded,_dark,}.svg`. Favicon uses rounded variant.
- Motion: 180ms ease-out. Stagger-enter on lists. `prefers-reduced-motion` always respected. CSS classes `.enter`, `.enter-fade`, `.stagger-enter`, `.pulse-soft`, `.route-transition`, `.swipe-feedback` in `app.css`.
- No emoji as iconography (the cowrie + Material Symbols cover everything).

## Workflow

### Dev loop

```bash
# Backend (Rust)
cd backend
cargo build --release          # ~12s incremental
cargo check --message-format=short
cargo test

# SPA
cd web
npm install
npm run build                  # builds to web/dist/ (60-70 KB gzipped)
npm test                       # Vitest, 41+ tests
npx tsc --noEmit               # quick TS-only check

# Run the server (binds 0.0.0.0:8080 by default — LAN reachable)
cd /home/lomartins/projects/owo
DATABASE_URL="sqlite://$PWD/owo.db?mode=rwc" \
  BIND_ADDR="0.0.0.0:8080" \
  OWO_WEB_DIST="$PWD/web/dist" \
  OWO_UPLOADS_DIR="$PWD/uploads" \
  ./backend/target/release/owo
```

### Resetting dev state

```bash
pkill -f 'target/release/owo'
rm -f owo.db owo.db-wal owo.db-shm
```

### Migrations

Migrations are embedded at compile time via `sqlx::migrate!("./migrations/sqlite")`. **Edit a migration file → rebuild the backend binary.** Forgetting this gives you stale schema. Migration files are sequential `00NN_*.sql`; never reorder or rewrite a shipped one for an existing DB without bumping the version.

### Importing historical data

`scripts/import_june_2026.mjs` is the template for importing spreadsheet-style data: idempotent, takes a bearer token via `OWO_TOKEN`, uses the public API (no DB writes). Copy + adapt per month.

## Common pitfalls

- **Material Symbols glyphs aren't always centered in their em-box.** `sell`, `event_repeat` etc. anchor to upper-left. Prefer letter monograms or check each glyph visually before using one in `.tx-icon`.
- **Vitest 4 environment**: per-file `// @vitest-environment jsdom` works; project-level `environment: "jsdom"` in `vite.config.ts` `test` block is also set up. Don't both override.
- **localStorage in node**: jsdom build here doesn't expose Storage in all paths; guard tests with `if (typeof localStorage !== "undefined")`.
- **Categories table** already has `icon` and `color` columns (since `0001_initial.sql`). The seed-icon backfill happens in `Categories.tsx` on first list-load, not in a backend migration.
- **The backend binary embeds migrations + bundled OpenAPI doc.** Editing migration SQL without `cargo build --release` is a silent no-op against the running process.
- **`PRAGMA legacy_alter_table` does NOT save you on its own.** Rewrites still require the create-new-then-drop-old pattern (rule 5 above). Don't trust the pragma alone.

## Specs index (read before non-trivial changes)

- `specs/00-mvp-scope.md` — what's in / what's out
- `specs/01-data-model.md` — entities + fields
- `specs/02-api.md` — REST surface
- `specs/03-money.md` — Decimal at boundary
- `specs/04-two-legged-transactions.md` — leg model
- `specs/05-account-types.md` — 5-type enum
- `specs/06-budgets.md` — month-keyed budgets
- `specs/07-monthly-report.md` — dashboard aggregates
- `specs/08-cards-preserve.md` — card preserve-and-verify
- `specs/09-web-spa.md` — SPA spec
- `specs/10-change-plan.md` — ordered execution
- `specs/decisions/` — ADRs (money, account types, card remap, mobile deferral)
- `specs/phases/` — per-phase shipment notes (A1 → D)

## Net-worth & dashboard panels

The dashboard (`web/src/routes/Dashboard.tsx`) has three sections in order:
1. Tinted greeting card with `<Avatar>` + "Olá, Luisa & Mavê" + current month label
2. `<NetWorthPanel>` (dark hero, 7 month bars, % change pill) — `GET /reports/net-worth?to=YYYY-MM&months=7`
3. Summary tiles (Receitas / Gastos / Saldo do orçamento / Δ saldo do mês) + per-category budget rows with bars

Net worth = `Σ asset balances − Σ (credit_card + liability) balances` at month-end. See `backend/src/api/reports.rs`.

## Bill semantics quick reference

- A bill is a template row in `bills` with `value`, `due_day` (1-31), `account_id`, `category_id`, `currency`. Recurrence is always MONTHLY.
- Each month a user pays → `bill_payments` row (unique on `bill_id, month`) + a real `transactions` row tied to the bill.
- Per-month edits (e.g. variable utility) → `bill_overrides` row (unique on `bill_id, month`); `GET /bills?month=…` COALESCEs override over template.
- Edit scope:
  - `this_month` → upsert one override
  - `this_and_next` → upsert two
  - `all` → patch template + delete all overrides + retroactively patch linked transactions (irreversible — warn user)

## Open follow-ups (intentional)

- Mobile (KMP) — see `specs/phases/E-mobile-resync.md` (deferred)
- Loans, investments, projections, IR — beyond MVP, see `Owo requirements brief.md`
- Pluggy / CSV-OFX import — Phase 3
- Postgres swap — covered by the Decimal-at-boundary design (one-day change), not on the roadmap

## When asked about ultrareview or running it
`/code-review ultra` is user-triggered cloud review — billed. Cannot be launched by Claude. Needs a git repo.

---

This file evolves with the project. Update it when an ADR lands, a new top-level pattern is set, or a workflow command changes.
