# 04 — Conventions: how every piece of Bookkeepy 1 code is shaped

Goal: additions, refactors, subtractions and deprecations are easy and safe. Everything is
standardized, patterned, pipelined, grouped, owned, contextualized, API'd, validated and
engine'd. Rules have IDs (`RULE-<AREA>-<NNN>`); invariants have IDs (`INV-<AREA>-<NNN>`).
Repo governors (later: `cargo xtask govern …`, CI) enforce the mechanical ones; guard
engines + tests enforce the invariants.

## 4.1 Primary data (RULE-DATA-001/002, ADR-0008)

- Sets of primary data (constants, rules, variables the app is built on) live as data files
  in `data/*.toml` — e.g. `data/currencies.toml` (ISO 4217). **No such value is ever
  hardcoded in source code**; code imports it from the `primary-data` crate.
- Primary data also includes **libraries/dependencies that are data**: the IANA time-zone
  database (via `jiff`), ISO 4217 (our `currencies.toml`), and later ISO 3166 countries /
  ISO 639 languages — referenced as dependencies or vendored files, never re-typed in code.
- Files carry a `[source]` header. The `primary-data` crate loads them via `include_str!`,
  parses once at startup, validates, and exposes typed accessors. Invalid primary data fails
  at startup, loudly.
- **Referential integrity (RULE-DATA-002):** primary data cross-references by ISO code.
  Live-rates packages must use codes present in `currencies.toml`; categories reference
  currency codes; etc. Producers and the app both validate.
- Shipped/planned: `currencies.toml` (done), `errors.toml` (error-code registry), `defaults.toml`
  (fallback policies), `load-policies.toml` (what loads when), `perf-budgets.toml` (CI
  budgets), `theme/*.toml` (UI variables), plus planned `countries.toml`, `languages.toml`,
  `category-presets.toml`, `default-accounts.toml`.
- Changing primary data = changelog entry; schema change = ADR.
- User-level customization (their categories, rates, …) is *not* primary data; it lives in
  the user's database.

## 4.2 Modularization and single source of truth (RULE-MOD-001, RULE-SSOT-001, ADR-0013)

Modularization and orchestration are **principles**, not afterthoughts.

- **Folders group; folders nest freely.** File per responsibility. There is no artificial
  line limit, but files stay small and reusable: as a file grows, split it proactively into
  a folder of smaller files (3.1).
- **Variants and edge cases live adjacent to the thing they vary** (3.2): e.g.
  `src/ledger/posting/mod.rs`, `posting/engine.rs`, `posting/variant_transfer.rs`,
  `posting/variant_split.rs`, `posting/edge_case_zero_amount.rs`, `posting/tests.rs`.
  Small files *extend* the base thing; they never duplicate it.
- **Single source of truth (RULE-SSOT-001, 3.4):** every concept is defined in exactly one
  file; everything else `use` / `pub use`es it and links to it. No copied constants, no
  parallel definitions. UI components follow the same rule: one source component, with
  variants/properties/edge cases linked to it (see 4.8).
- Crate skeleton (the *shape*, not a file-cramming rule):

```
crates/<domain>/
├── README.md                 ownership, invariants, ADR links
└── src/
    ├── lib.rs                only `pub mod` re-exports
    ├── api/mod.rs            the public API (pub fns/traits, DTO re-exports) — the only import other crates use
    ├── api/<operation>.rs    one file per operation when the API grows
    ├── <thing>/              one folder per domain concept
    │   ├── mod.rs            re-exports
    │   ├── model.rs          types + smart constructors (invalid states unrepresentable)
    │   ├── engine.rs         the engine (or engine/<sub>.rs as it grows)
    │   ├── pipeline/         a pipeline = folder of stage files: stage_01_parse.rs, stage_02_map.rs, …
    │   ├── guard.rs          guard engine: invariant checks before/after (4.10)
    │   ├── validate.rs       boundary validation (RULE-VAL-001)
    │   ├── error.rs          thiserror enum with BK-codes (4.4)
    │   ├── events.rs         domain events (4.6)
    │   ├── variant_*.rs      variants, adjacent (3.2)
    │   ├── edge_*.rs         edge cases, adjacent (3.2)
    │   └── tests/            unit tests per file group
    └── … more <thing>/ folders
```

- **Engine roles (3.3):** engines are specialized — domain engines (`engine_*`), helper
  engines (`helper_*` — e.g. the validation engine), guard engines (`guard_*` — enforce
  invariants), broker/facade engines (`broker_*` — the only allowed path for engine↔engine
  talk, 6/03-architecture), pipelines (`pipeline_*` — staged, resumable flows).
- `api/` is the crate's contract. Internal refactors never change it without an ADR.
- Engines take validated input + handles, return `Result<_, DomainError>`; pipelines persist
  progress per stage so they resume after a crash (fintecheng "full resumability").

## 4.3 Validation (RULE-VAL-001)

- All external input (user input, imported files, packages, settings) is validated at the
  boundary before any engine runs; engines assert preconditions again. Prefer making
  invalid states unrepresentable (smart constructors, newtypes, enums).

## 4.4 Errors, warnings, user-facing diagnostics (RULE-ERR-001…004)

- Each crate defines a `thiserror` enum; every variant carries a **stable code**
  `BK-<AREA>-<NNNN>`; `?` propagates; the binary converts to `anyhow` at the edge.
- **Error registry (SSOT):** `data/errors.toml` lists every code with severity and i18n key.
  A governor (`xtask govern errors`) fails CI if a code in code is missing from the
  registry, or lacks a message in every shipped locale. Messages render in the chosen
  language (EN/RO first).
- **Severity model:** `warning` (toast, app continues) → `error` (dialog, app continues) →
  `fatal` (dialog with one-click **“Copy diagnostic”**: code + localized message + context +
  app version + OS; the user can paste it into a GitHub issue). Internally, engines/pipelines
  keep working until a **fatal** error; all fatal errors are surfaced to the user for
  reporting (point 8).
- No `unwrap()`/`expect()`/`panic!` outside tests; `dbg!`/`todo!` denied; `unsafe_code =
  forbid`. Errors are actionable: what failed, which input, what to fix.

## 4.5 Money (INV-MONEY-001, ADR-0004)

- Never `f64`/`f32`. `Money { amount_minor: i64, currency: CurrencyCode }`; minor units from
  `data/currencies.toml` (ISO 4217 — not always 2 decimals). Currency codes validated
  against the primary data at every boundary.
- Serialization: string or integer minor units, never a bare JSON number.
- `clippy::float_arithmetic` denied in `core` and all money crates.

## 4.6 Events & ownership (RULE-EVT-001)

- Every state change emits a typed domain event (what / when / who-what / why — the audit
  fields, doc 06). A `bus` crate dispatches to subscribers: `audit` (append-only),
  `search` (tantivy write-through), `rates` (index new packages), `ui` (cx.notify).
  Subscribers are idempotent and durable where needed (local outbox: reactions survive a
  crash and replay — fintecheng outbox pattern, adapted to one machine).
- One crate owns each table/entity set; others go through its `api`.

## 4.7 Async-first (RULE-ASYNC-001)

- The app works **primarily async**; gpui-kit favors async orchestration and we follow.
- All engine/pipeline/broker APIs are async (`async fn` / `impl Future`); the UI drives
  them with `cx.spawn(...)`; storage work runs on the background executor / a dedicated DB
  thread behind a bounded channel — the UI thread never blocks on I/O or DB.
- Long-running flows (import, rates download, backup, rebuild) emit progress events and are
  cancellable; they are resumable (4.2 pipelines).

## 4.8 UI conventions — the Figma model (RULE-UI-001…004, ADR-0007)

- UI uses **all** gpui-kit styled components (catalog-first, ADR-0007); usage follows the
  vendored skills in `skills/gpui-kit/` and `skills/gpui-kit-design-guides/`.
- We manage our interface the way a large Figma file is managed (point 9): **components,
  variables, properties, variants linked to a single source of truth.**
  - *Variables* = theme tokens as primary data (`data/theme/*.toml`) — one file per token
    group, imported everywhere; never re-typed.
  - *Components* = our wrappers in `ui/src/widgets/<component>/` — `mod.rs` (source),
    `variant_*.rs`, `props.rs`, `edge_*.rs` — built on the gpui-kit styled component.
  - *Properties/variants* map to gpui-kit component props; edge cases sit in adjacent files.
- `ui` depends only on `app-services::api` + `contracts` (RULE-UI-004); presenters
  (`ui/src/presenters/<feature>/`) are the SSOT for view models — one per feature, with
  variants adjacent.
- One view module per screen: `ui/src/views/<feature>/<screen>/`.

## 4.9 Governors (point 4)

- **Repo governors:** `cargo xtask govern` runs a tree of governors — each governor may have
  sub-governors: `govern deps` (dependency direction RULE-ARCH-001, RULE-NET-001 — only
  `updater`/`rates` crates may depend on HTTP), `govern lints`, `govern docs` (docs index
  accurate, links valid, one canonical doc per topic), `govern adr` (numbering, status),
  `govern perf` (budgets in `data/perf-budgets.toml` vs criterion benches), `govern errors`
  (registry ↔ code ↔ locales), `govern data` (primary data validates; ISO referential
  integrity). CI runs the whole tree.
- **Logic governors (in the app):** every engine folder has `guard.rs` — a guard engine
  that checks the crate's invariants before/after operations (fintecheng: runtime checks +
  post-factum). Guards emit error codes on violation; the audit log records them. The
  `governor` orchestrates guards per operation.

## 4.10 Documentation conventions (point 16)

- Docs are modular too: `docs/README.md` is the index; **one canonical file per topic** —
  cross-link, never duplicate content (RULE-DOC-001). When a topic outgrows its file, split
  by subtopic and link from the index.
- ADRs are for **decisions only**, numbered, immutable once accepted (supersede, never edit
  except status). Don't create ADRs for routine changes — those go in the CHANGELOG.
- Every public item has doc comments; every crate has a README; `CHANGELOG.md` (Keep a
  Changelog) gets an entry per user-visible change.

## 4.11 Git & CI hygiene (RULE-PROC-001)

- `main` protected: PRs only, required CI + review, conventional commits (`feat:`, `fix:`,
  `refactor:`, `docs:`, `adr:` …) referencing the issue. Branch: `<type>/<short-name>`.

## 4.12 Change checklist (every PR)

1. Fits an existing crate/thing folder (4.2)? New concept → new folder, SSOT respected? If
   architecture-affecting → ADR first.
2. New/changed rule or invariant? → register the ID here; governor if mechanical.
3. Primary data touched? → 4.1 process + RULE-DATA-002 checks.
4. Boundary validation (4.3)? BK-coded errors + registry + locales (4.4)? Money rules (4.5)?
5. Events emitted (4.6)? Async (4.7)? Guards updated (4.9)?
6. Tests (doc 09) incl. property/golden where money is touched; docs updated, single
   canonical place; CHANGELOG entry.
