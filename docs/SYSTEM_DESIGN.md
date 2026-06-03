# owo — System Design

Snapshot of the current repository structure as of 2026-05-08. All diagrams are
[Mermaid](https://mermaid.js.org/) — edit them directly in this file; GitHub,
Obsidian, IntelliJ, and VS Code (with the Mermaid extension) render them inline.

> **Legend** — Solid arrow = compile/runtime dependency. Dashed arrow = future /
> planned. Stadium nodes (`([ ])`) = external systems. Hex (`{{ }}`) = build /
> tooling artifacts.

---

## 1. System context (C4 L1)

```mermaid
flowchart LR
    user([User])
    bank([Bank Open Finance API])
    ai([AI Provider — optional])

    subgraph owo["owo platform"]
        mobile[Mobile App<br/>KMP · Compose MP]
        web[Web Dashboard<br/>TBD]:::planned
        cli[CLI<br/>TBD]:::planned
        backend[Backend API<br/>Rust · axum]
        db[(PostgreSQL)]
    end

    user -->|Android / iOS| mobile
    user -.->|browser| web
    user -.->|terminal| cli
    mobile -->|REST| backend
    web -.->|REST| backend
    cli -.->|REST| backend
    backend --> db
    backend -->|Open Banking| bank
    backend -.->|optional| ai

    classDef planned stroke-dasharray: 5 5,fill:#fafafa,color:#888
```

---

## 2. Repository monorepo layout

```mermaid
flowchart TB
    root[owo/]
    root --> backend[backend/<br/>Rust · axum · sqlx]
    root --> mobile[mobile/<br/>KMP · Gradle]
    root --> docs[docs/]
    root --> scripts[scripts/<br/>SQL bootstrap]
    root --> ops{{Dockerfile<br/>docker-compose.yml<br/>Makefile}}
    root -.-> web[web/ — TBD]:::planned
    root -.-> cli[cli/ — TBD]:::planned

    backend --> b_src[src/]
    b_src --> b_api[api/]
    b_src --> b_models[models/]
    b_src --> b_services[services/]
    b_src --> b_db[db/]
    b_src --> b_int[integrations/]
    backend --> b_mig[migrations/<br/>0001_initial_schema.sql]

    mobile --> m_app[app/<br/>Android entry]
    mobile --> m_shared[shared/<br/>KMP entry · iOS framework]
    mobile --> m_core[core/]
    mobile --> m_feat[feature/]

    m_core --> c_dom[domain]
    m_core --> c_db[database<br/>Room]
    m_core --> c_ui[ui<br/>Compose]
    m_core --> c_nav[navigation<br/>api · impl]

    m_feat --> f_on[onboarding<br/>api · impl]
    m_feat --> f_dash[dashboard<br/>api · impl]
    m_feat --> f_acc[accounts<br/>api · impl]
    m_feat --> f_cards[cards<br/>api · impl]
    m_feat --> f_tx[transactions<br/>api · impl]
    m_feat -.-> f_home[home<br/>api · impl ⚠ not in settings.gradle]:::orphan

    classDef planned stroke-dasharray: 5 5,fill:#fafafa,color:#888
    classDef orphan stroke:#c33,fill:#fee,color:#c33
```

> ⚠ `mobile/feature/home/{api,impl}` exists on disk but is **not registered** in
> `mobile/settings.gradle.kts`. Either include it or delete it.

---

## 3. Mobile module dependency graph (KMP)

api/impl split: `:feature:X:api` exposes navigation routes + public contracts;
`:feature:X:impl` carries Compose UI + ViewModels + Koin wiring. Features depend
on **other features' api modules only** — never on impl. `:shared` is the
composition root that wires every impl together for the iOS XCFramework and
Android entry.

```mermaid
flowchart TB
    app[":app<br/>Android"]
    shared[":shared<br/>iOS XCFramework + composition root"]

    subgraph core["core/"]
        c_domain[":core:domain"]
        c_db[":core:database<br/>Room + KSP"]
        c_ui[":core:ui<br/>Compose MP"]
        c_nav_api[":core:navigation:api"]
        c_nav_impl[":core:navigation:impl"]
    end

    subgraph feature["feature/ — api/impl"]
        on_api[":feature:onboarding:api"]
        on_impl[":feature:onboarding:impl"]
        dash_api[":feature:dashboard:api"]
        dash_impl[":feature:dashboard:impl"]
        acc_api[":feature:accounts:api"]
        acc_impl[":feature:accounts:impl"]
        card_api[":feature:cards:api"]
        card_impl[":feature:cards:impl"]
        tx_api[":feature:transactions:api"]
        tx_impl[":feature:transactions:impl"]
    end

    app --> shared
    shared --> on_impl
    shared --> dash_impl
    shared --> acc_impl
    shared --> card_impl
    shared --> tx_impl
    shared --> c_db
    shared --> c_domain
    shared --> c_ui
    shared --> c_nav_api
    shared --> c_nav_impl

    c_nav_impl --> c_nav_api
    c_db --> c_domain
    c_ui --> c_nav_api

    on_api --> c_domain
    on_api --> c_nav_api
    dash_api --> c_domain
    dash_api --> c_nav_api
    acc_api --> c_domain
    acc_api --> c_nav_api
    card_api --> c_domain
    card_api --> c_nav_api
    tx_api --> c_domain
    tx_api --> c_nav_api

    on_impl --> on_api
    dash_impl --> dash_api
    acc_impl --> acc_api
    card_impl --> card_api
    tx_impl --> tx_api

    dash_impl --> tx_api
    dash_impl --> acc_api
    dash_impl --> card_api

    on_impl --> c_ui
    dash_impl --> c_ui
    acc_impl --> c_ui
    card_impl --> c_ui
    tx_impl --> c_ui
```

---

## 4. Mobile clean-architecture layering (per feature)

```mermaid
flowchart TB
    subgraph ui["feature/X/impl — UI layer"]
        screen[Composable Screen]
        vm[ViewModel<br/>Koin · lifecycle]
    end

    subgraph contract["feature/X/api"]
        route[NavRoute · @Serializable]
        contract_iface[Public contracts]
    end

    subgraph domain["core/domain"]
        usecase[UseCase]
        repo_iface[Repository interface]
        model[Domain model]
    end

    subgraph data["core/database"]
        repo_impl[Repository impl]
        dao[Room DAO]
        entity[Room Entity]
    end

    api([Backend REST]):::ext

    screen --> vm
    vm --> usecase
    vm --> route
    usecase --> repo_iface
    repo_impl -.implements.-> repo_iface
    repo_impl --> dao
    dao --> entity
    repo_impl -.->|sync| api

    classDef ext fill:#eef,stroke:#557
```

---

## 5. Backend internal structure (Rust)

Cargo crate `owo-backend` (edition 2024). Stack: **axum** HTTP, **sqlx**
Postgres + migrations, **tokio**, **tracing**, **tower-http**, **rust_decimal**
for money, **argon2** for passwords.

```mermaid
flowchart LR
    main[main.rs<br/>bootstrap]
    lib[lib.rs<br/>app factory]
    api[api/<br/>routes · handlers]
    services[services/<br/>business logic]
    models[models/<br/>domain types · DTOs]
    db[db/<br/>pool · queries]
    integrations[integrations/<br/>Open Finance · AI]
    tests[tests/]
    mig[(migrations/<br/>0001_initial_schema.sql)]
    pg[(PostgreSQL)]
    bank([Bank API]):::ext

    main --> lib
    lib --> api
    api --> services
    services --> models
    services --> db
    services --> integrations
    db --> pg
    db -.applies.-> mig
    integrations --> bank
    tests --> api

    classDef ext fill:#eef,stroke:#557
```

---

## 6. Build & run pipeline

```mermaid
flowchart LR
    dev[Developer]
    make{{Makefile}}
    dc{{docker-compose}}
    cargo{{cargo}}
    gradle{{gradle}}

    pg[(Postgres container)]
    apibin[owo-backend binary]
    apk[Android APK]
    xcf[iOS XCFramework]

    dev --> make
    make --> cargo
    make --> dc
    cargo --> apibin
    dc --> pg
    apibin --> pg

    dev --> gradle
    gradle -->|:app:assembleDebug| apk
    gradle -->|:shared:linkXCFramework| xcf
    apk --> apibin
    xcf --> apibin
```

---

## 7. Status matrix

| Component       | Path                  | State                 | Notes                                    |
|-----------------|-----------------------|-----------------------|------------------------------------------|
| Backend         | `backend/`            | Scaffolded            | mod.rs stubs, deps chosen, 1 migration   |
| Mobile shared   | `mobile/shared/`      | Wiring complete       | All feature impls aggregated             |
| Mobile features | `mobile/feature/*`    | UI scaffolded         | onboarding, dashboard, accounts, cards, transactions |
| `feature/home`  | `mobile/feature/home/`| **Orphan**            | Not in `settings.gradle.kts`             |
| Web             | `web/`                | Not started           | Mentioned in CLAUDE.md only              |
| CLI             | `cli/`                | Not started           | Mentioned in CLAUDE.md only              |
| Docs            | `docs/`               | Minimal               | `database.md`, `logo.svg`, this file     |

---

## 8. How to edit

- Open this file in any Mermaid-aware viewer (GitHub renders inline).
- Live editor: <https://mermaid.live> — paste a single fenced block to iterate.
- For richer canvas-style editing: import the Mermaid into draw.io or Excalidraw
  (both have Mermaid import).
- Keep the **Status matrix** in sync when modules land or get removed.
