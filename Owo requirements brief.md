# owo: Requirements and Gap Analysis Brief

## Purpose of this document

This is a requirements specification for `owo`, a self-hosted personal finance manager. The project is already in progress. This document is written to be handed to Claude Code as a working brief.

Claude Code: your job is not to start from scratch. Read the existing codebase first, then compare it against the requirements below, then report what exists, what is missing, and what must change. The exact task and the expected output format are in the final section, "Task for Claude Code". Read the whole document before acting.

## 1. Product context

`owo` replaces an envelope-budgeting spreadsheet. The spreadsheet works for monthly budgeting and categorized spending, but it breaks on three things that are the real reason this software exists:

1. Credit card spending loses its category. Card invoices appear as single lines in the month they are paid, so a grocery purchase made on a card stops being "groceries" and becomes "other".
2. Loans are recorded as a fixed monthly value with no shrinking balance and no interest split.
3. Investments are not modeled at all, and there is no way to project future contributions.

`owo` must solve all three. They are the differentiators; generic budgeting is the baseline.

## 2. Architecture

Thin clients, logic on the server. CLI, web, and mobile are all clients of one API. All business logic (budget math, amortization, invoice closing, projections) lives in the backend so it is never duplicated across clients.

```
+-----------+   +-----------+   +-----------+
|    CLI    |   |    Web    |   |  Mobile   |
+-----+-----+   +-----+-----+   +-----+-----+
      |               |               |
      +-------+-------+-------+-------+
                      |
              REST/JSON API
                      |
            +---------+---------+
            |  Backend (Rust)   |
            |  rules + math     |
            +---------+---------+
                      |
              +-------+-------+
              |  PostgreSQL   |
              +---------------+
```

Stack: Rust + Axum, SQLx with PostgreSQL. Phase 1 ships backend and CLI. Web SPA and KMP/Android come later as additional clients of the same API.

### Non-negotiable backend rules

- Money is always a decimal type, never a float. Use `rust_decimal::Decimal` in Rust and `NUMERIC(14,2)` in Postgres. Floats lose cents.
- Any operation touching two accounts is atomic in a single SQL transaction. An installment purchase that generates 12 rows is all-or-nothing.
- All financial rules live server-side. Clients render and submit; they do not compute balances, interest, or projections.
- Authentication is token-based. Single-user self-hosted is acceptable for the first version; design so multi-user can be added without a rewrite.

## 3. Core domain model

The key modeling principle: separate "when you spend" from "when you pay". For cash and PIX these are simultaneous. For credit cards they are separated in time (buy in February, pay the invoice in March). For loans the spend happened once in the past and the payments are the monthly installments. Two-legged transactions over an account model capture all of this.

### 3.1 Base entities

**account**: anywhere money sits, enters, or leaves.
- Fields: `id`, `name`, `type`, `currency` (default BRL), `opening_balance`, `archived`
- `type`: `asset`, `credit_card`, `liability`, `revenue`, `expense`
- Credit card is a `credit_card` type (a liability subtype), and loans are `liability`. Do not model a credit card as an asset.

**transaction**: one movement of money with two legs.
- Fields: `id`, `date`, `description`, `amount`, `source_account_id`, `destination_account_id`, `category_id`, `payment_method`, `paid` (bool), `notes`
- Derived `type`: `withdrawal` (asset to expense), `deposit` (revenue to asset), `transfer` (between own accounts)

**category**: configurable. Seed with Food, Leisure, Transport, Health, Education, Clothes, Home, Pet, Subscriptions, Other.

**budget**: monthly allocation per category.
- Fields: `id`, `category_id`, `month` (YYYY-MM), `estimated_amount`
- `spent_amount` and `difference` are computed, never stored.

**payment_method**: PIX, Cash, Boleto, Card, VA (meal voucher). Enum or table. VA is income tied to a specific spending category.

### 3.2 Month-to-month continuity

Do not store "previous month balance" as a field. Compute an account balance at a date as `opening_balance + sum(transactions up to that date)`. The carry-over is a query, not a stored value. This removes the manual-entry error the spreadsheet has.

### 3.3 Credit cards (solves problem 1)

Each card purchase is a normal transaction with a category, but its source leg is the `credit_card` account, not the checking account. The invoice is an aggregation. Paying the invoice is a transfer from checking to the card.

**credit_card_statement**:
- Fields: `id`, `card_account_id`, `closing_date`, `due_date`, `reference_month`, `total_amount` (computed), `paid` (bool), `paid_date`

Flow:
1. Card purchase: transaction with source = card, a real category, linked to the statement of its closing month. Category is preserved.
2. Statement closes: sum of transactions within the cycle.
3. Payment: a checking-to-card transfer that zeros the card balance. This payment has no spending category, otherwise the spend is double-counted. It is a movement between accounts only.

Result: you see how much went to each category on the card, and separately how much left the bank to pay the invoice, with no double counting.

**installment_plan** (solves "items plus installments"):
- Fields: `id`, `card_account_id`, `description`, `total_amount`, `installments_count`, `first_due_month`, `category_id`
- Generates N future transactions, one per statement, each `amount = total / N`, all sharing the category. Handle rounding so the installments sum exactly to the total (assign the remainder cent to the last or first installment).
- Enables a view of committed future spending.

### 3.4 Loans (solves problem 2)

**loan**:
- Fields: `id`, `name`, `principal`, `annual_interest_rate`, `installments_count`, `start_date`, `payment_method`, `current_balance` (computed)

**loan_installment**:
- Fields: `id`, `loan_id`, `number`, `due_date`, `amount`, `principal_portion`, `interest_portion`, `paid`, `paid_date`

Support both amortization and fixed installments. Default to the Price table (fixed installment). Fixed installment formula:

```
PMT = PV * [ i * (1+i)^n ] / [ (1+i)^n - 1 ]
```

`PV` principal, `i` monthly rate, `n` number of installments. Per installment, interest portion is `outstanding_balance * i` and amortization portion is `PMT - interest`. The outstanding balance shrinks each payment. Also support SAC (constant amortization) as an alternative method, selectable per loan.

### 3.5 Investments: tracking and projection (solves problem 3)

Two distinct functions. Do not conflate them.

- Tracking looks backward: how much I hold now, how much it has yielded.
- Projection looks forward: how much I will have if I contribute X per month at Y per year.

**investment** (a holding you have today):
- Fields: `id`, `name`, `account_id` (linked asset account so it counts toward net worth), `invested_amount`, `current_value`, `remuneration_type`, `start_date`, `maturity_date` (optional), `liquidity` (D+0, D+1, at maturity), `tax_exempt` (bool)

**remuneration** (how it yields; embed or separate table):
- `kind`: `prefixed` (fixed rate), `cdi_percent` (percent of CDI), `ipca_spread` (IPCA plus spread)
- `rate`: the fixed rate, the CDI percentage, or the spread
- Each investment selects its own remuneration type.

**index_rate** (reference rates for indexed holdings):
- Fields: `index` (CDI, IPCA, SELIC), `reference_date`, `annual_value`
- Manually maintained for now. For projection, the user supplies an assumed future CDI/IPCA; no one knows the future index, so a projection is a scenario, not a forecast.

**projection** (a simulation scenario):
- Fields: `id`, `name`, `initial_amount`, `monthly_contribution`, `months`, `assumed_annual_rate` (or assumed CDI/IPCA), `created_at`
- The monthly result series is computed. Optionally store a snapshot so scenarios can be compared.

Tracking math: `current_value` is updated by periodic reconciliation (monthly, manual at first). The difference versus last period minus contributions is the yield. The linked asset account feeds the existing net-worth calculation, so investments show in net worth without polluting the month's cash flow. Keep contributions as transactions so new money is never confused with yield.

Projection math, future value with constant monthly contributions:

```
FV = P * (1+i)^n + PMT * [ ((1+i)^n - 1) / i ]
```

`P` initial amount, `PMT` monthly contribution, `i` effective monthly rate, `n` months. This is the same compound-interest module as the Price table, inverted. Reuse it.

Rate conversion, a common bug source:
- Annual to monthly: `i_monthly = (1 + i_annual)^(1/12) - 1`. Never divide by 12; that is simple interest and understates the result.
- For indexed holdings in a projection, convert the assumed index to the holding's effective rate before projecting.

Output the projection month by month as a time series, splitting contributed principal from accrued yield, so it can feed an evolution chart.

Regressive income tax (gross vs net): on taxable Brazilian fixed income the rate falls with holding time. Tax applies only to yield, never to principal.

```
up to 180 days:       22.5%
181 to 360 days:      20.0%
361 to 720 days:      17.5%
over 720 days:        15.0%
```

For a first version, applying the full-term rate to total yield is an acceptable approximation; document it as such. A more accurate version tracks each contribution's holding time separately. Respect the `tax_exempt` flag (savings, LCI, LCA, some bonds pay no IR). IOF in the first 30 days is negligible for long-term projection; omit it in v1 and document the simplification. Always present gross and net side by side.

Important framing: `owo` informs, it does not give investment advice. Projections depend on an assumed future rate, which is a hypothesis, not a prediction. Surface this in the UI copy.

## 4. API surface

```
GET    /accounts                    list accounts with current balance
POST   /accounts                    create account
GET    /transactions?month=YYYY-MM  transactions for a month
POST   /transactions                create transaction
POST   /transactions/installment    create installment purchase (generates N)
GET    /budgets?month=YYYY-MM       budget vs spent per category
PUT    /budgets/:id                 adjust estimated amount
GET    /cards/:id/statement?month=  card statement
POST   /cards/:id/pay               pay statement (transfer)
GET    /loans                       loans with outstanding balance
POST   /loans                       create loan (generates Price or SAC schedule)
GET    /loans/:id/schedule          amortization schedule
GET    /investments                 holdings with current value and yield
POST   /investments                 register a holding
POST   /investments/:id/reconcile   update current value (tracking)
POST   /projections                 simulate scenario, return monthly gross and net series
GET    /projections/:id             retrieve a saved scenario
GET    /reports/monthly?month=      summary: income, spending, balance
GET    /reports/networth            net worth (assets minus liabilities, incl. investments)
```

## 5. CLI (Phase 1 client)

Use `clap`. The CLI consumes the API rather than touching Postgres directly, so rules are never duplicated. Default to a readable table; provide `--json` for scripting.

```
owo add 52.97 --cat food --method card --desc "lunch"
owo add 9000 --income --account bank --desc "salary"
owo budget set food 230 --month 2026-02
owo budget show --month 2026-02
owo card statement nubank --month 2026-03
owo card pay nubank --from bank
owo loan add "Loan X" --principal 10000 --rate 2.5 --installments 12 --method price
owo loan schedule 1
owo invest add "CDB X" --value 5000 --kind cdi_percent --rate 110
owo invest reconcile 1 --value 5180
owo project --initial 1000 --monthly 500 --months 60 --annual-rate 10.5
owo report month --month 2026-02
owo networth
```

## 6. Phasing

- Phase 1 (backend + CLI): accounts, transactions, categories, budgets, balance-by-date, monthly budget report, credit card statements and invoice payment.
- Phase 1.5 (the differentiators): installments, loans with Price/SAC and shrinking balance, net worth, investment tracking and projection with gross/net via regressive IR.
- Phase 2 (rich clients): web SPA served by Axum on a single port/container; KMP/Android with offline entry and sync.
- Phase 3 (optional): bank data integration (Pluggy), CSV/OFX import for history migration.

## 7. Task for Claude Code

Do this in order and do not write feature code until the analysis is delivered and reviewed.

### Step 1: Inventory the existing code

Read the current `owo` repository. Produce a concise map of what is there: crate and module structure, the data model as it currently exists (tables, migrations, Rust structs), which endpoints exist, what the CLI currently does, and what tests exist. Do not assume; read the actual code, including the SQLx migrations and any schema.

### Step 2: Compare against this spec and produce a gap report

For each area below, classify the current state as one of: Present and correct, Present but needs change, or Missing. When you say "needs change" or "missing", be specific about what and why, and cite the file or migration involved.

Areas to assess:
1. Account model and the five account types, with credit card and loan as liabilities.
2. Two-legged transaction model and derived transaction types.
3. Categories and monthly budgets, with spent/difference computed not stored.
4. Balance-by-date and month carry-over as a query.
5. Credit card statements, the buy-vs-pay separation, and no double counting on invoice payment.
6. Installment plans generating N transactions with exact rounding.
7. Loans with Price and SAC, shrinking balance, interest/principal split.
8. Investment tracking by reconciliation, linked to net worth.
9. Projection math (future value with contributions), correct annual-to-monthly conversion.
10. Regressive IR, gross vs net, tax-exempt flag.
11. Decimal money types end to end (no floats), and atomicity of multi-account operations.
12. API surface coverage.
13. CLI command coverage.

### Step 3: Recommend a change plan

After the gap report, propose an ordered plan: what to refactor before adding features (especially anything touching the money type or the transaction model, since those are foundational and expensive to change later), then the sequence of additions following the phasing in section 6. Flag any place where the current code made an assumption that conflicts with this spec, and explain the tradeoff before changing it.

### Output format

Deliver, in this order: (1) the code inventory, (2) the gap report as a table with the three-state classification and file references, (3) the ordered change plan. Keep it direct and structured. Ask before making large structural changes; for the analysis steps, no permission is needed, just read and report.
