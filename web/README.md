# owo web SPA

Mobile-first responsive SPA for owo. Built with **SolidJS + Vite + TypeScript + Tailwind v4**. Solid gives signal-based reactivity with a tiny runtime (no virtual DOM cost), which fits a 5-screen finance app far better than React's overhead; Tailwind v4 is used purely as a token system for the bold terracotta aesthetic — every utility class on the page is in service of a design decision in `src/styles/app.css`, not a substitute for one.

## Build

```bash
cd web
npm install
npm run build
```

The bundle is emitted to `web/dist/`. Axum serves it via `tower_http::services::ServeDir` at `/`, falling back to `index.html` so client-side routes survive a refresh.

## Dev loop

```bash
npm run dev
```

Starts Vite on port 5173 and proxies `/api/*` to the backend on `localhost:3000`. To run end-to-end, launch the backend first (`cargo run` in `/backend`) and then `npm run dev` here.

## What ships

- Five screens: monthly dashboard, transaction entry, transaction list, budget editor, month switcher (global header element).
- Auth: bearer token from `/auth/login` kept in `localStorage`. App state is **never** cached locally — every screen reads from the API.
- Money is always treated as decimal strings (per `specs/03-money.md`): parsed to BigInt cents for arithmetic and formatted via `Intl.NumberFormat`.

See `specs/phases/D-web-spa.md` for design choices, the localStorage trade-off, and known API mismatches the backend still needs to close.
