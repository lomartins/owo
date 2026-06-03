# 09 — Web SPA

Responsive (mobile-first), served as static from Axum on the same port as `/api/v1`.

## Design direction

Read `/mnt/skills/public/frontend-design/SKILL.md` before writing CSS. Commit to one bold cohesive aesthetic — no generic AI-default styling.

- Dominant colour: terracotta `#ad4f32`. Use it confidently on primary surfaces, not as an accent stripe.
- Brand motif: cowrie shell. Used as logo, as the "carry-over" glyph, as the empty-state illustration.
- Typography: one distinctive display face (suggested: a high-contrast serif or a confident geometric like *Fraunces* / *DM Serif Display* / *Söhne Breit*) + one clean body face (`Inter`, `Geist`, or `IBM Plex Sans`). Pick the pairing in the design-pass; document the choice in `phases/D-web-spa.md`.
- Numbers: tabular-figure body face for amounts. Negative numbers in a desaturated red, not the same terracotta.
- Empty states get a small cowrie illustration, not a generic icon.

## Stack

To be decided in `phases/D-web-spa.md`. Constraints:

- Output is a static bundle in `web/dist/`.
- No browser storage APIs for state (per spec). All state from the API.
- Reasonable build pipeline (Vite + a thin framework). Recommend **SolidJS** or **Svelte** — small bundle, no runtime virtual DOM cost, fine ergonomics for a 5-screen app. React works too but the bundle is heavier than needed.
- Tailwind is fine but not required; design-direction commits matter more than utility-class choice.

## Screens (mobile-first)

### 1. Monthly dashboard

Top cards: `Income`, `Spent`, `Budget balance`, `Carry-over` (in/out).

Below, one bar per expense category:

```
Food          ████████░░░░  R$ 612.30  /  R$ 800.00   (+ R$ 187.70)
Transport     ███████████░  R$ 320.00  /  R$ 350.00   (+ R$  30.00)
Leisure       █████████████ R$ 410.00  /  R$ 300.00   (− R$ 110.00)
```

Over-budget bars in desaturated red. Positive `difference` rendered with a soft `+`, negative with `−`.

### 2. Transaction entry

Fast form for thumb-typing. Field order:

1. Amount (big numeric input, formatted on blur).
2. Category (chip row, most-used first, "more" expands).
3. Source account (defaults to user's most-recently-used asset account).
4. Payment method (chip row: PIX, Card, Cash, Boleto, VA).
5. Paid flag (toggle, default ON for variable expenses).
6. Date (defaults to today, tappable to change).
7. Description (optional, single line).

`destination_account_id` is inferred: if `category_id` set → the user's `expense` account. Same for income via the "log income" mode of the same screen.

### 3. Transaction list

Filterable by month, category, account. Default = current month. Each row: date, description, category chip, amount, payment-method icon. Tap to edit, swipe (or long-press on desktop) to delete.

### 4. Budget editor

Per category: estimated input field. Save updates upsert on `(category_id, month)`. Live preview of `spent` and `difference` next to the input. Month switcher in header.

### 5. Month switcher

Global header element. `<` / current / `>`. Sub-line shows `carry_over_in → carry_over_out`. Clicking the current month opens a year-grid picker.

## Wiring

- Axum: `tower-http::services::ServeDir` at `/`. Fall-back to `index.html` so client-side routing works on refresh.
- API base URL: same origin, `/api/v1`. No CORS.
- Auth: existing Bearer token; store in a `httpOnly` cookie set by the login endpoint, or accept localStorage trade-off and document it. Decide in phase D.
