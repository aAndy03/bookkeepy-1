# 04 — Conventions: how every piece of Bookkeepy 1 code is shaped

Goal: additions, refactors, subtractions and deprecations are easy and safe. Everything is
standardized, patterned, pipelined, grouped, owned, contextualized, API'd, validated and
engine'd. Rules have IDs (`RULE-<AREA>-<NNN>`); invariants have IDs (`INV-<AREA>-<NNN>`).
Repo governors (`cargo xtask govern …`, CI) enforce the mechanical ones; guard engines +
tests enforce the invariants.

## 4.1 Primary data (RULE-DATA-001/002/003, ADR-0008)

- Sets of primary data (constants, rules, variables the app is built on) live as data files
  under `data/` — e.g. `data/currencies.toml` (ISO 4217). **No such value is ever hardcoded
  in source code**; code imports it from the `primary-data` crate.
- Primary data also includes **data-as-dependencies**: the IANA time-zone database (via
  `jiff`), ISO 4217, later ISO 3166/639 — referenced as dependencies or vendored files,
  never re-typed in code.
- Files carry a `[source]` header. The `primary-data` crate loads them via `include_str!`,
  parses once at startup, validates, and exposes typed accessors. Invalid primary data fails
  at startup, loudly.
- **Referential integrity (RULE-DATA-002):** primary data cross-references by ISO code.
  Rates packages must use codes present in `currencies.toml`; etc. Producers and the app
  both validate.
- **Primary data is modularized too (RULE-DATA-003):** a scope that will grow becomes a
  directory, not one crammed file: `data/<scope>/index.toml` is the single entry point
  listing the scope's files; each file owns one small concern; variants/overrides sit in
  adjacent files (e.g. `data/currencies/overrides.toml`). The loader merges + validates the
  whole scope. Same modularization/SSOT/variants principles as code (point 1). Example as
  scopes grow: `errors.toml` → `data/errors/{index,storage,rates,import,fx}.toml`.
- Shipped: `currencies.toml`, `errors.toml`, `defaults.toml`, `load-policies.toml`,
  `perf-budgets.toml`. Planned: `theme/*.toml`, `countries.toml`, `languages.toml`,
  `category-presets.toml`, `default-accounts.toml`.
- Changing primary data = changelog entry; schema change = ADR.
- User-level data (their categories, rates, …) is *not* primary data; it lives in the DB.

## 4.2 Modularization and single source of truth (RULE-MOD-001, RULE-SSOT-001, ADR-0013)

Modularization and orchestration are **principles**, not afterthoughts.

- **Folders group; folders nest freely.** File per responsibility. No artificial line limit,
  but files stay small and reusable: as a file grows, split it proactively into a folder of
  smaller files.
- **Variants and edge cases live adjacent to the thing they vary**: e.g.
  `src/ledger/posting/mod.rs`, `posting/engine.rs`, `posting/variant_transfer.rs`,
  `posting/variant_split.rs`, `posting/edge_case_zero_amount.rs`, `posting/tests.rs`.
- **Single source of truth (RULE-SSOT-001):** every concept is defined in exactly one file;
  everything else `use` / `pub use`es it and links to it. No copied constants, no parallel
  definitions. Applies to code, primary data (4.1), and UI (4.9).
- Crate skeleton (shape, not file-cramming):

```
crates/<domain>/
├── README.md                 ownership, invariants, ADR links
└── src/
    ├── lib.rs                only `pub mod` re-exports
    ├── api/mod.rs            public API — the only import other crates use
    ├── api/<operation>.rs    one file per operation as the API grows
    ├── <thing>/              one folder per domain concept
    │   ├── mod.rs · model.rs · engine.rs · pipeline/ (stage_01_*.rs …) · guard.rs
    │   ├── validate.rs · error.rs · events.rs · variant_*.rs · edge_*.rs · tests/
    └── …
```

- **Engine roles:** domain engines (`engine_*`), helper engines (`helper_*` — e.g.
  validation), guard engines (`guard_*` — invariants), broker engines (`broker_*` — the
  only engine↔engine path), pipelines (`pipeline_*` — staged, resumable).
- `api/` is the contract; internal refactors never change it without an ADR. Engines take
  validated input + handles and return `Result<_, DomainError>`; pipelines persist progress
  per stage (resume after crash).

## 4.3 Validation (RULE-VAL-001)

All external input (user input, imported files, rates packages, settings) is validated at
the boundary before any engine runs; engines re-assert preconditions. Prefer making invalid
states unrepresentable (smart constructors, newtypes, enums). **Number/date input parsing is
locale-aware** (decimal comma vs dot, date order per locale, fiscal periods — doc 05 §5.12);
the validation engine owns the parsing rules (one SSOT module), and invalid input gets a
BK-coded localized message.

## 4.4 Errors, warnings, user-facing diagnostics (RULE-ERR-001…004)

- Each crate has a `thiserror` enum; every variant carries a stable code `BK-<AREA>-<NNNN>`;
  `?` propagates; `anyhow` at the binary edge.
- **Registry (SSOT):** `data/errors.toml` lists every code with severity + i18n key.
  `xtask govern errors` fails CI if a code in code is missing from the registry or lacks a
  message in every shipped locale. Messages render in the chosen language (EN/RO first).
- **Severity:** `warning` (toast, app continues) → `error` (dialog, app continues) →
  `fatal` (dialog + one-click **“Copy diagnostic”**: code + localized message + context +
  version/OS — paste into a GitHub issue). Internally, engines keep working until a *fatal*
  error; all fatals are surfaced for reporting.
- No `unwrap()`/`expect()`/`panic!` outside tests; `dbg!`/`todo!` denied; `unsafe_code =
  forbid`. Errors say what failed, which input, what to fix.

## 4.5 Money (INV-MONEY-001, ADR-0004)

`Money { amount_minor: i64, currency: CurrencyCode }`; minor units from ISO 4217 primary
data (not always 2 decimals); no floats in money code (`float_arithmetic` denied); serialize
as string or minor-unit integer, never a bare JSON number.

## 4.6 Events & ownership (RULE-EVT-001)

Every state change emits a typed domain event (what / when / who-what / why). The `bus`
crate dispatches to subscribers (`audit`, `search`, `rates`, `jobs`, `ui`); subscribers are
idempotent; reactions that must survive a crash go through a durable local outbox and
replay. One crate owns each table; others use its `api`.

## 4.7 Async-first (RULE-ASYNC-001)

The app works **primarily async**. All engine/pipeline/broker APIs are async; the UI drives
them with `cx.spawn`; storage runs on the background executor / a dedicated DB thread behind
a bounded channel; the UI thread never blocks on I/O. Long flows emit progress events and are
cancellable — see 4.8.

## 4.8 Jobs orchestration (RULE-JOB-001, ADR-0015)

Async means jobs — and jobs are orchestrated **once, centrally**, never inline per call site
(gpui-kit provides the async primitives; we wrap them in our own engine so behavior is
uniform everywhere):

- Every potentially long/failing async operation is a **Job** handled by the `jobs` crate
  (`engine_jobs` + `broker_jobs`): import, rates sync, backup/restore, report export,
  index rebuild, migrations, etc.
- A Job is a **durable state machine** persisted in the `jobs` table: `id, type, state
  (queued|running|paused|done|failed|cancelled), progress (fraction + message), attempts,
  max_attempts, payload, result, error_code, timestamps`. On startup a driver resumes
  interrupted jobs (fintecheng full resumability).
- **Auto-retry:** on failure, retry automatically with exponential backoff + jitter up to
  `max_attempts` (policy per job type in `data/defaults.toml` `[jobs]`). After max attempts:
  `failed` — **manual retry is surfaced to the user** (toast + Jobs panel entry with a Retry
  button). The error carries its BK-code.
- **Progress & control:** every job reports progress events on the bus; the UI shows one
  shared progress indicator with **pause / resume / cancel** (job control API; cancel is
  cooperative at stage boundaries). Job types are pipelines (stages persisted), so resume
  is exact.
- **UI:** a single Jobs drawer widget (Figma-model component, per-type variant files) —
  never modal, never blocking (RULE-UX-001, 4.10).

## 4.9 UI conventions — the Figma model + session state (RULE-UI-001…004, RULE-STATE-001)

- gpui-kit styled components, catalog-first (ADR-0007), per the vendored `skills/`.
- Figma-style management: **variables** = theme tokens as primary data (`data/theme/*.toml`);
  **components** = our wrappers in `ui/src/widgets/<component>/` (`mod.rs` source +
  `props.rs` + `variant_*.rs` + `edge_*.rs`); **presenters** are the SSOT for view models.
- `ui` depends only on `app-services::api` + `contracts` (RULE-UI-004).
- **Session/UI state persistence (RULE-STATE-001):** interface state the user changes in a
  session persists across restarts: window geometry per display (uses `system_context`),
  last route/view, list filters/sort/page, expanded tree nodes, **form drafts** (debounced
  autosave), open dialog. Stored in the `ui_state` table (namespace.key → JSON) through the
  single `ui-state` module (SSOT schema). Not persisted: anything posted/audited, secrets,
  transient toasts.
- **Clearing is intuitive and non-destructive:** per-view "Reset view", "Clear unsaved
  drafts", Settings → "Reset interface state". Clearing never blocks and never touches
  posted data.

## 4.10 Audit vs app history vs undo (RULE-AUD-001, RULE-UNDO-001) + never-block (RULE-UX-001, ADR-0014)

- **`audit_log` — sacred.** Append-only, hash-chained, tamper-evident. Records what / when /
  who-or-what / why for every **posted** change (ledger postings, corrections/reversals,
  imports, rates imports, settings/security changes). Once posted, an audit event **cannot be
  edited or deleted through the app** — not even by the user (the user's erasure right is
  exercised outside the app, by deleting the whole data file; PII is kept in separate mutable
  stores, doc 05 §5.8). Verified via `verify_chain()` + `quick_check`. *If the audit can be
  tampered with, it has no value.*
- **`app_history` — ordinary tracking.** UI-level actions (views opened, searches run,
  dialogs) for continuity and prefetch tuning. **Not** money truth. User chooses its
  behavior in Settings: keep indefinitely / auto-delete every N days / delete on exit / off;
  clearable anytime.
- **Undo/redo (RULE-UNDO-001) only affects non-audited state** — drafts, input boxes, filters,
  UI state (4.9), unposted forms. What undo/redo may touch is defined incrementally as the
  app grows, but the rule is absolute: **nothing already posted to the audit is ever
  undoable**. Editing a posted transaction is a *correction*: it posts a linked reversal/
  adjustment — which itself posts a new audit event (ADR-0010). Example: user records a
  transaction → audit event A posted; user edits it → correction entry posts audit event B;
  both remain.
- **Never block the user (RULE-UX-001):** the app never obstructs or blocks user actions
  because something hasn't completed (users are lazy; reminders, not force). Backups,
  rates sync, migrations, self-checks run as background jobs (4.8); overdue items produce
  dismissible reminders; no modal lockouts; failures → warning + Jobs entry.

## 4.11 Governors

- **Repo governors:** `cargo xtask govern` runs a tree of governors (each may nest):
  `deps` (dependency direction RULE-ARCH-001; RULE-NET-001 — only `updater`/`rates` may use
  HTTP), `net` (host allowlist scan), `lints`, `docs` (index accurate, links valid, one
  canonical doc per topic), `adr`, `perf` (`data/perf-budgets.toml` vs criterion benches),
  `errors` (registry ↔ code ↔ locales), `data` (primary data validates; ISO referential
  integrity RULE-DATA-002/003). CI runs the whole tree.
- **Logic governors (in-app):** every engine folder has `guard.rs` — a guard engine checking
  the crate's invariants before/after operations (by construction + runtime + post-factum).
  Guards emit BK-codes on violation; the audit log records them.

## 4.12 Documentation conventions (RULE-DOC-001)

Docs are modular too: `docs/README.md` is the index; **one canonical file per topic** —
cross-link, never duplicate; split by subtopic when a file outgrows its topic. ADRs are for
**decisions only**, numbered, immutable once accepted (supersede via new ADR; status edits
allowed). Routine changes → CHANGELOG, not new ADRs. Every pub item documented; every crate
has a README.

## 4.13 Git & CI hygiene (RULE-PROC-001)

`main` protected: PRs only, required CI + review, conventional commits referencing the
issue; branch `<type>/<short-name>`.

## 4.14 Change checklist (every PR)

1. Fits an existing crate/thing folder (4.2)? New concept → new folder; SSOT respected?
   Architecture-affecting → ADR first.
2. New/changed rule or invariant? Register the ID; governor if mechanical.
3. Primary data touched? → 4.1 process (RULE-DATA-002/003).
4. Boundary validation (4.3)? BK-coded errors + registry + locales (4.4)? Money (4.5)?
5. Events emitted (4.6)? Async + Job if long-running (4.7/4.8)? UI state via ui-state (4.9)?
   Audit vs history respected (4.10)? Guards updated (4.11)?
6. Tests (doc 09) incl. property/golden for money and retry/resume injection for jobs;
   edge cases enumerated in `edge_*.rs` against the register in doc 10 (point 9);
   docs updated in the single canonical place; CHANGELOG entry.
