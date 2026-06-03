# owo Mobile — Main Screens

Spec for the KMP mobile client. Each screen lists purpose, primary actions,
backend endpoints consumed, and offline behavior.

Backend contract: [API.md](API.md). Sync model: [ADR-0001](adr/0001-sync-strategy.md).

## Conventions

- All screens read from local Room DB (offline-first). `pull` endpoint refreshes; `push` uploads queued changes.
- Money displayed via locale formatter; stored as `i64` cents.
- Optimistic UI: writes apply locally, sync-queued, rolled back on `409 STALE_WRITE`.
- All authenticated requests carry `Authorization: Bearer <token>` from Keystore.

## Screen index

| # | Screen | Module | Backend touch points |
|---|--------|--------|----------------------|
| 1 | Onboarding / Welcome | `feature:onboarding` | none |
| 2 | Register | `feature:onboarding` | `POST /auth/register` |
| 3 | Login | `feature:onboarding` | `POST /auth/login` |
| 4 | Dashboard | `feature:dashboard` | `GET /reports/net-worth`, `GET /reports/cash-flow` |
| 5 | Accounts list | `feature:accounts` | `GET /accounts` |
| 6 | Account detail | `feature:accounts` | `GET /accounts/{id}`, `GET /accounts/{id}/balance`, `GET /transactions?account_id=` |
| 7 | Account create / edit | `feature:accounts` | `POST /accounts`, `PATCH /accounts/{id}` |
| 8 | Cards list | `feature:cards` | `GET /cards` |
| 9 | Card detail / invoice | `feature:cards` | `GET /cards/{id}/invoices`, `POST /cards/{id}/invoices/{inv}/pay` |
| 10 | Transactions list | `feature:transactions` | `GET /transactions` (cursor) |
| 11 | Transaction detail | `feature:transactions` | `GET /transactions/{id}` |
| 12 | Add / edit transaction | `feature:transactions` | `POST /transactions`, `PATCH /transactions/{id}` |
| 13 | Transfer | `feature:transactions` | `POST /transactions/transfer` |
| 14 | Bills list | `feature:bills` (new) | `GET /bills` |
| 15 | Bill detail / pay | `feature:bills` (new) | `POST /bills/{id}/pay` |
| 16 | Investments list | `feature:investments` (new) | `GET /investments` |
| 17 | Loans list | `feature:loans` (new) | `GET /loans`, `POST /loans/{id}/pay-installment` |
| 18 | Budgets | `feature:budgets` (new) | `GET /budgets` |
| 19 | Goals | `feature:goals` (new) | `GET /goals` |
| 20 | Reports | `feature:reports` (new) | `GET /reports/cash-flow`, `GET /reports/by-category` |
| 21 | Categories management | `feature:categories` (new) | `GET /categories`, `POST /categories` |
| 22 | Tags management | `feature:tags` (new) | `GET /tags`, `POST /tags` |
| 23 | Settings | `feature:settings` (new) | `GET /auth/me`, `PATCH /auth/me` |
| 24 | Sessions / devices | `feature:settings` (new) | `GET /auth/sessions`, `DELETE /auth/sessions/{id}` |
| 25 | Backup | `feature:settings` (new) | `POST /backup/create`, `GET /backup/jobs/{id}` |
| 26 | Audit log | `feature:settings` (new) | `GET /audit` |

---

## 1. Onboarding / Welcome

**Purpose**: first-launch tour, brand intro, choose Register or Login.
**Elements**: 3-page horizontal pager (track / plan / sync), CTA "Get started" → Register, "I have an account" → Login.
**State**: Remembers completion via DataStore (`onboarding_seen=true`), skips on next launch.
**Offline**: fully local.

## 2. Register

**Purpose**: create account on the self-hosted server.
**Inputs**: server URL, email, password (min 12), display name, default currency (BRL/USD/EUR), locale.
**Generates**: `device_id` (UUID v7 stored in Keystore) + `device_name` (model name).
**Backend**: `POST /api/v1/auth/register` → stores returned token in Keystore.
**Errors**: `409` email exists; `403` registration disabled; network → retry.

## 3. Login

**Purpose**: re-authenticate on a known server.
**Inputs**: server URL (cached), email, password, optional "remember device" (keeps `device_id`).
**Backend**: `POST /api/v1/auth/login`.
**Errors**: `401` generic; `429` lockout (show countdown).

## 4. Dashboard (home)

**Purpose**: financial overview at a glance.
**Sections (vertical scroll)**:
- Net worth card (`/reports/net-worth`) — accounts + investments − loans − card debt.
- Month cash-flow chart (`/reports/cash-flow?from=...&to=...&granularity=MONTH`).
- Quick add FAB (`+ Income`, `+ Expense`, `↔ Transfer`).
- Upcoming bills (next 7 days) — local query on `bills WHERE due_date <= now+7d AND paid=false`.
- Goals progress (top 3).
- Recent transactions (last 5, local).
**Offline**: derives everything from Room. Pull-to-refresh triggers `sync/pull`.

## 5. Accounts list

**Purpose**: enumerate all owned accounts.
**Items**: name, type icon, current balance, currency.
**Actions**: tap → detail; FAB → create; long-press → archive/delete (soft).
**Backend**: `GET /api/v1/accounts` (initial sync); subsequent reads local.

## 6. Account detail

**Purpose**: balance + transaction history for one account.
**Header**: name, type, current balance large, "as of" timestamp.
**Tabs**: Transactions | Stats | Settings.
**Transactions tab**: cursor-paginated (`GET /transactions?account_id=...`), grouped by day.
**Stats tab**: 30-day inflow/outflow bar, top 5 categories.
**Settings tab**: rename, archive, delete.

## 7. Account create / edit

**Inputs**: name, type (`BANK`/`PAYMENT`/`CASH`), currency picker, opening balance.
**Backend create**: `POST /accounts`.
**Backend edit**: `PATCH /accounts/{id}` with `If-Match` header (last `updated_at`).
**Conflict**: `409 STALE_WRITE` → re-fetch and ask user to retry.

## 8. Cards list

**Purpose**: credit + debit cards.
**Items**: brand logo, last4, type chip, available limit (credit) or linked account (debit).
**Actions**: tap → card detail; FAB → add card.

## 9. Card detail / invoice

**Purpose**: track credit-card cycle.
**Sections**:
- Open invoice card: total accumulated this cycle, period dates, days until close.
- Past invoices: list with status (paid/open), totals.
- Pay invoice button: opens sheet → pick account, amount → `POST /cards/{id}/invoices/{inv}/pay`.

## 10. Transactions list

**Purpose**: full transaction stream.
**Filters bar**: account, date range, type, payment method, category chips, tag chips, text search.
**List**: virtualized, cursor-paginated. Each row: date, description, amount (color-coded), category icon, tags.
**Backend**: `GET /transactions?cursor=...&limit=50` with active filters. Cursor encoded server-side; client passes through.
**Empty state**: "No transactions yet — tap + to add."

## 11. Transaction detail

**Purpose**: full record of one transaction.
**Fields**: date, account, card, type, payment method, value (with FX rate if cross-currency), description, category, tags, receipt thumbnail, photo thumbnail.
**Actions**: edit, delete, view receipt full-screen, jump to transfer pair (if TRANSFER).

## 12. Add / edit transaction

**Form** (single screen, sectioned):
- Type segmented control: Income / Expense / Transfer.
- Account picker.
- Card picker (only when method = CREDIT/DEBIT).
- Value (numeric pad with currency).
- Date picker (default today).
- Payment method chip row: PIX / DEBIT / CREDIT / TED / BOLETO / CASH.
- Category picker (with parent hierarchy).
- Tags multi-select.
- Description.
- Attach receipt (camera/picker — uploads via multipart `POST /transactions/{id}/receipt` after save).
**Backend**: `POST /transactions` then optional `POST /transactions/{id}/receipt`.

## 13. Transfer

**Purpose**: move funds between own accounts.
**Inputs**: from-account, to-account, value, currency (auto-from-source), `fx_rate` (required when accounts differ in currency), date, description.
**Backend**: `POST /transactions/transfer` → returns linked outbound + inbound; both rendered in lists with link icon.

## 14. Bills list

**Purpose**: scheduled / recurring expenses (utilities, subscriptions).
**Sections**: Overdue · Due this week · Upcoming · Paid (collapsed).
**Items**: description, value, due-date pill, recurrence chip (`MONTHLY` etc).
**Actions**: tap → detail; long-press → mark paid quickly.

## 15. Bill detail / pay

**Header**: description, value, next due date, recurrence summary.
**Pay action**: pick account + tx_date + value (default = bill value) → `POST /bills/{id}/pay`. Creates linked Transaction; if recurring, server returns `next_bill` and UI rolls forward.
**Edit**: change value, due date, recurrence.

## 16. Investments list

**Purpose**: track CDB / Tesouro / stocks.
**Items**: name, type chip (FIXED/VARIABLE), current value, gain since purchase (color).
**Actions**: tap → detail (read-only stats screen — historical chart from local `current_value` snapshots); FAB → add.
**Update value**: `PATCH /investments/{id}/value` for variable instruments.

## 17. Loans list

**Purpose**: track outstanding debt.
**Items**: description, remaining balance, installment progress (e.g. `12/60`), next due.
**Actions**: tap → detail; "Pay installment" sheet → `POST /loans/{id}/pay-installment`.

## 18. Budgets

**Purpose**: monthly / yearly category caps.
**List**: each row shows category, period, limit bar (spent vs cap, % filled), remaining.
**Tap**: drill-down list of transactions in that category for the period.
**Create**: pick category, period, limit, currency, start date.

## 19. Goals

**Purpose**: savings targets.
**List**: name, progress ring (current/target), days to deadline.
**Detail**: linked account (optional), manual contribution button (creates `INCOME` transaction tagged with goal).

## 20. Reports

**Tabs**:
- **Cash flow**: stacked bar (income vs expense) per month, line for net.
- **By category**: donut for current month + table; toggle EXPENSE/INCOME.
- **Net worth**: line chart of cumulative monthly snapshots (server returns latest only; historical computed locally from monthly aggregates).
**Filters**: date range, account multiselect, currency.

## 21. Categories management

**Purpose**: tree view of user categories.
**Actions**: create root, create child, rename, change icon/color, archive.
**Hierarchy**: drag to re-parent (writes `parent_id` via PATCH).

## 22. Tags management

**Purpose**: flat list of tags.
**Actions**: create, rename, change color, delete (cascades via M:N).

## 23. Settings

**Sections**:
- **Profile**: display name, default currency, locale → `PATCH /auth/me`.
- **Security**: change password (`POST /auth/password`), revoke all other sessions.
- **Devices**: → screen 24.
- **Backup**: → screen 25.
- **Audit log**: → screen 26.
- **About**: server URL, app version, log out (`POST /auth/logout`).

## 24. Sessions / devices

**Purpose**: see and revoke active sessions.
**List**: device_name, ip_last_seen, last_used_at, "current" badge.
**Actions**: revoke individual (`DELETE /auth/sessions/{id}`); revoke all others (`DELETE /auth/sessions`).

## 25. Backup

**Purpose**: trigger encrypted backup bundle.
**Form**: optional passphrase (toggle "Encrypt"), output destination = device download.
**Flow**:
1. `POST /backup/create { encrypt, passphrase }` → `job_id`.
2. Poll `GET /backup/jobs/{id}` until `status=DONE`.
3. Tap "Download" → `GET /backup/download/{id}` → save via SAF (Android Storage Access Framework) / share-sheet (iOS).
**Restore**: not in mobile v1 (CLI only). Show note.

## 26. Audit log

**Purpose**: review what changed.
**List**: timestamp, entity type, entity id (linkable), action, device.
**Filters**: entity type, action, date range.
**Backend**: `GET /audit?...` cursor-paginated.

---

## Cross-cutting components

- **Bottom navigation** (3 tabs): Home (Dashboard) · Transactions · Settings. Accounts, Cards, Bills, Reports reachable from Home cards or a global drawer.
- **Quick-add FAB**: persistent on Home and Transactions, expands to Income/Expense/Transfer.
- **Sync indicator**: top-app-bar status — green dot (synced), yellow (pending push), red (error).
- **Empty states** and **error states** are first-class screens for every list.

## Data flow diagram (per screen)

```
[Compose Screen]  →  [ViewModel]  →  [UseCase]  →  [Repository]
                                                      │
                                       ┌──────────────┴──────────────┐
                                       │                             │
                                  [Room DAO]                  [HTTP client → owo backend]
```

UI reads local Room only. The repository pushes / pulls via the sync endpoints
on a coalesced background timer (default 30s) and on explicit pull-to-refresh.

## Phasing for implementation

| Phase | Screens |
|-------|---------|
| M1 (auth + skeleton) | 1, 2, 3, 4, 23 |
| M2 (core money) | 5, 6, 7, 10, 11, 12 |
| M3 (cards + transfer) | 8, 9, 13 |
| M4 (planning) | 14, 15, 18, 19 |
| M5 (long-term assets) | 16, 17 |
| M6 (insights + admin) | 20, 21, 22, 24, 25, 26 |

## References

- [API.md](API.md) — endpoint contract.
- [DATA_MODEL.md](DATA_MODEL.md) — entity fields.
- [SYSTEM_DESIGN.md](SYSTEM_DESIGN.md) — module dependency graph.
- Figma designs (TBD — link when available).
