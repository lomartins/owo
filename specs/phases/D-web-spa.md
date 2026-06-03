# Phase D — Web SPA

Status: scaffolded, buildable. Not yet wired end-to-end against a running backend.

## What shipped

- `web/` SolidJS + Vite + TypeScript + Tailwind v4 SPA.
- Five screens, all routed:
  - `/` — monthly dashboard (`Dashboard.tsx`): four summary cards (Income, Spent, Budget balance, Carry-over Δ) + per-category gauge rows. Over-budget bars render in desaturated `--color-loss-500`. Signed differences use a soft `+` / `−` per spec.
  - `/transactions` — list (`Transactions.tsx`): month-scoped, plus account and category filters. Renders amount with sign derived from leg pair (revenue→ inflow, → expense outflow), payment-method glyph, pending pill.
  - `/add` — entry (`AddTransaction.tsx`): big amount input (decimal locale-aware), category chips, account select, payment-method chip row, paid toggle, date, description. Spend / Log-income toggle at the top.
  - `/budgets` — editor (`Budgets.tsx`): per-category estimated input with live preview of `spent - estimated` (positive = under budget, terracotta gain tone).
  - Global month switcher lives in `AppHeader.tsx`; `<` / month label / `>` with `carry-over Δ` sub-line.
- `/login` route for sign-in / register.
- Cowrie shell motif inline-SVG: brand mark (`Cowrie` compact mode) and empty-state illustration (`Cowrie illustration` mode).
- Toast host for transient feedback.

## Stack decision

**SolidJS + Vite + TypeScript + Tailwind v4.**

- SolidJS over React: 5 screens, hot signal-graph reactivity, ~10 kB runtime vs React's ~45. The financial-data UI is read-heavy with frequent re-renders of computed money fields; Solid's fine-grained updates avoid React's reconciliation cost on bar gauges and amount previews.
- SolidJS over Svelte: Solid's JSX keeps the engineering surface aligned with the wider TS/React ecosystem we already touch, and it has the smallest router (`@solidjs/router`) for our 4 protected routes + login.
- Vite for the build; native ESM, fast HMR, single config.
- Tailwind v4 via `@tailwindcss/vite`: tokens live in `@theme` (terracotta scale, bone, ink, loss/gain, fonts, radii, shadow). Utility classes are used in service of the tokens, not as a styling substitute — `app.css` defines `.card`, `.btn`, `.chip`, `.gauge`, `.brand-bar` so screens compose layout with named primitives rather than ad-hoc soup.

## Typography

- Display: **Fraunces** (high-contrast variable serif). Used for the wordmark, monthly totals, and section titles.
- Body: **Inter** (variable). Tabular figures enabled site-wide via `.num`, `input[inputmode=decimal]`, and `input[type=number]`.

Both fonts are pulled from CDNs at load time so the build stays static. If you want them self-hosted later, drop the woff2 into `public/fonts/` and update `@font-face` in `src/styles/app.css`.

## Money handling

- All money in and out of the API is a JSON decimal string per `specs/03-money.md`.
- `src/lib/money.ts` parses to `bigint` cents (`parseCents`), formats via `Intl.NumberFormat`, and serialises back with `centsToApi`. No JS `number` is used for amounts anywhere — verified by grepping `src/`.
- `inputToApi` accepts both `1.234,56` (pt-BR) and `1,234.56` (en-US) user typing.

## Auth + storage trade-off

- The bearer token is kept in `localStorage` under `owo.token` and attached as `Authorization: Bearer …` on every request. A cached `{ id, email, display_name, default_currency, locale }` is also kept for instant cold start.
- **App state — transactions, accounts, categories, budgets, reports — is never cached.** Every screen fetches from the API.
- Trade-off vs spec preference for `httpOnly` cookies: localStorage is XSS-exposed. The backend currently issues bearer tokens via JSON, so accepting localStorage is the only way the SPA ships today. To upgrade, the backend would need to set a `Set-Cookie: HttpOnly; Secure; SameSite=Strict` on `POST /auth/login` and the SPA would drop the `Authorization` header in favour of `credentials: "include"`. That is a backend change; flagged below.

## API mismatches found (backend follow-ups)

These are not blockers for the build but will surface as runtime errors when wired end-to-end:

1. **No `PUT` / `DELETE /transactions/:id`** in `backend/src/api/mod.rs` despite `specs/02-api.md` listing them. The list view shows tap-to-edit / swipe-to-delete affordances in spec; the current code intentionally does not call these endpoints. Need to add `transactions::update` + `transactions::delete` handlers + routes.
2. **Bucket-account ids not exposed.** `POST /transactions` requires `source_account_id` and `destination_account_id`, but `GET /accounts` filters out `revenue` and `expense` bucket accounts (see `accounts::list` SQL). The SPA cannot post a withdrawal without the expense bucket id (or a deposit without the revenue bucket id). Options:
   - Expose them on `GET /auth/me` as `{ revenue_account_id, expense_account_id }`.
   - Or accept `category_id` alone on `POST /transactions` and let the backend derive the bucket leg. The current `AddTransaction.tsx` surfaces a clear error when both ids end up empty.
3. **Accounts uses `PATCH`, not `PUT`.** `specs/02-api.md` says `PUT /accounts/:id` but `mod.rs` registers `PATCH`. The SPA does not yet hit account update, but worth aligning either the spec or the route name.
4. **No `/categories/:id`** for update/delete in `backend/src/api/mod.rs` — only `GET` list and `POST` create. Spec lists PUT and DELETE.
5. `transactions::list` does not accept `paid` filter (out of spec scope; noted only).

## Files created

```
web/
  package.json
  tsconfig.json
  vite.config.ts
  index.html
  README.md
  src/
    main.tsx
    App.tsx
    AppShell.tsx
    api/
      client.ts        # fetch wrapper + token handling
      index.ts         # typed endpoint methods
      types.ts         # wire shapes
    components/
      AppHeader.tsx
      Cowrie.tsx       # logo + empty-state illustration
      EmptyState.tsx
      MonthSwitcher.tsx
      Toast.tsx
    lib/
      money.ts         # parse / format / cents arithmetic
      month.ts         # YYYY-MM helpers
      session.ts       # auth signal
      useMonth.ts      # month + refresh context
    routes/
      Login.tsx
      Dashboard.tsx
      Transactions.tsx
      AddTransaction.tsx
      Budgets.tsx
    styles/
      app.css          # tokens + base + components layer
```

## How to build + serve

```bash
cd web
npm install
npm run build           # emits web/dist/
# then run the backend; Axum mounts dist/ at /
```

Dev loop: `npm run dev` proxies `/api/*` to `localhost:3000` so the SPA can run alongside `cargo run`.

## Open follow-ups for later phases

- Wire transaction edit + delete once the backend exposes the routes.
- Add the year-grid month picker (the current-month label is already a button; the popover is the remaining work).
- Self-host the two webfonts to drop the runtime CDN dependency.
- Consider switching the bearer-token storage to httpOnly cookies once the backend grows a cookie-issuing branch on `/auth/login`.
