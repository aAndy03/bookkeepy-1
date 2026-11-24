# 03 — Architecture

## Repo topology: monorepo + one mirror sub-repo (ADR-0001, ADR-0011/0012)

- **Main repo** `aAndy03/bookkeepy-1`: one Cargo workspace (ADR-0001 rationale stands).
- **Rates mirror sub-repo** `aAndy03/bookkeepy-1-rates`: automation pulls FX rates from
  public sources (BNR first) and publishes immutable checksummed packages (spec:
  `bookkeepy-1-rates/docs/package-format.md`). **The app never connects to rate sources
  directly — only to this sub-repo** (ADR-0011). Docs: [11-rates-mirror-and-live-data.md](11-rates-mirror-and-live-data.md).

```
bookkeepy-1/                     workspace root
├── Cargo.toml · rust-toolchain.toml (1.100.0)
├── crates/
│   ├── core/                    Money, Currency (from primary data), ids, time, error-code type. No I/O.
│   ├── primary-data/            loads data/*.toml via include_str!; typed accessors (ADR-0008)
│   ├── contracts/               DTOs + domain events; UI- and storage-agnostic
│   ├── bus/                     typed event bus + durable local outbox (crash-safe reactions)
│   ├── jobs/                    engine_jobs + broker_jobs: durable job state machines, auto-retry, pause/resume/cancel (RULE-JOB-001, ADR-0015)
│   ├── reminders/               local reminders/notifications engine (F16) — dismissible, never blocking
│   ├── storage/                 rusqlite: connection (background thread), migrations, repositories, hash-chained journal
│   ├── audit/                   append-only audit log + hash-chain verification
│   ├── ledger/                  double-entry postings, derived balances, reversals
│   ├── transactions/            user transactions → postings; splits; schedules (durable state machines)
│   ├── catalog/                 categories, tags, parties
│   ├── rates/                   engine_rates: package download/verify/import, source registry, manual rates, fallback defaults
│   ├── fx/                      valuation/conversion at stored rates, rounding account
│   ├── import/                  pipeline_import: parse → map → validate → dedup → post → index
│   ├── export/                  projections → CSV/JSON/PDF
│   ├── budget/ · reporting/     envelopes; golden-tested reports
│   ├── search/                  tantivy wrapper, write-through + rebuild (ADR-0006)
│   ├── documents/               attachments
│   ├── backup/                  snapshot / restore / verify + retention
│   ├── loadpolicy/              engine_loadpolicy: what loads when (data/load-policies.toml) — hot/warm/cold (point 12)
│   ├── diagnostics/             local system context (displays/DPI/GPU/CPU/RAM/OS/tz/net state) — local-only (doc 08)
│   ├── settings/ · i18n/        config, locales (EN/RO)
│   ├── security/                app lock, key derivation, optional at-rest encryption
│   ├── updater/                 app updates from main repo Releases (allowed HTTP #1)
│   ├── app-services/            composition root + brokers; the `api` the UI uses
│   └── ui/                      gpui-kit views, widgets (Figma-model), presenters, theme
├── app/                         thin binary: window, DI, run; **single instance + focus IPC** (RULE-APP-001, ADR-0016)
├── xtask/                       governor tree (doc 04 §4.9)
├── data/                        primary data (currencies, errors, defaults, load-policies, perf-budgets, theme…)
├── tests/                       integration + old-version fixtures (upgrade-path CI)
└── packaging/                   installers
```

## Dependency direction (RULE-ARCH-001)

`core ← primary-data ← contracts ← bus ← storage ← domain engines ← app-services ← ui ← app`;
`rates` and `updater` are leaves with the only HTTP deps (RULE-NET-001); `diagnostics` is a
leaf read-only over the OS. No cycles; UI never touches storage (`updater`/`rates` never
touch domain engines directly — they go through `app-services`). Enforced by
`xtask govern deps`.

## Layers — vertical and horizontal (point 6)

Engines/pipelines/APIs grow on **layers**, vertically and horizontally:

- **Vertical layers** (call direction is strictly downward; upward only via events):
  `L0 platform (OS/GPU/window)` → `L1 ui shell` → `L2 feature views` → `L3 presenters` →
  `L4 app-services api + brokers` → `L5 engines` → `L6 pipelines` → `L7 storage` →
  `L8 primary data`.
- **Horizontal slices** = feature domains (ledger, rates, budget…). A slice may span all
  layers, but slices talk to each other **only through `app-services` brokers and the
  event bus** — never directly (when/why: a change in one slice must not ripple; events
  decouple; brokers make interactions explicit and optimized).
- New capability → new vertical layer or a new module in an existing one; new feature → new
  horizontal slice. Both are async (RULE-ASYNC-001), use bounded channels (backpressure),
  and never block the UI thread.

## Runtime data flow

```
UI action → api (L4) → validate → guard → engine (L5) → storage tx (L7) → events on bus
   → audit (append) → search (write-through) → rates/index updates → ui notify
```

DB write = source of truth; balances, index, caches = derived.

## Async (point 5)

Primarily async everywhere (RULE-ASYNC-001, doc 04 §4.7): gpui `cx.spawn` in the UI,
background executor / dedicated DB thread for storage, progress events for long flows.

## Loading, caching, prefetch — hot / warm / cold (point 12)

Bookkeepy becomes an **archival record** over years of use; it must stay fast, so loading is
demand-driven, governed by one SSOT: `data/load-policies.toml` (consumed by
`engine_loadpolicy`):

- **Hot cache:** in-memory LRU + live gpui entities (current view models, accounts summary).
- **Warm:** SQLite (WAL, prepared statements, indexes, paginated/virtualized lists — never
  load the whole journal), tantivy index.
- **Cold:** attachment files, downloaded rates packages, old reports — read on demand only.
- **On app open:** shell + settings + accounts summary + last view only (INV-LOAD-001: the
  user can add a transaction within the budget in `data/perf-budgets.toml` on a cold start).
  A background self-check job runs `quick_check` + hash-chain verify + invariant guards
  (ADR-0016); drafts restore with a "Restore draft?" affordance.
- **Scale:** materialized aggregations (per-account-period balances, category totals) are
  kept by event-bus subscribers as background jobs and recomputed on demand as fallback;
  lists are virtualized/paginated; budgets cover 100k+ transaction books.
- **On navigation/idle:** prefetch per the policy table (next page, rates manifest if
  enabled, index warm-up for the current book).
- **Minimal local usage tracking:** a local `usage_stats` table (which views, when) used
  *only* to tune prefetching, contextualized to this machine; local-only, visible and
  deletable in Settings → Privacy (doc 08).

## Optimization everywhere — no drift (point 7, RULE-PERF-001)

Every engine/pipeline/layer registers its budgets in `data/perf-budgets.toml` (SSOT) and has
criterion benches; `xtask govern perf` + CI compare measurements against budgets, so no part
of the backend drifts unoptimized. Budgets are contextualized by `diagnostics` (this machine's
CPU/RAM/GPU/display) where relevant.

## Jobs orchestration (RULE-JOB-001, ADR-0015)

All async long-running work (import, rates sync, backup, exports, index rebuild, migrations)
is a **Job** in the `jobs` crate — durable, auto-retried (backoff, max attempts), resumable
after restart, with progress + pause/resume/cancel surfaced in one non-modal Jobs drawer.
Built once, used everywhere — never inline `cx.spawn` per call site. Details: doc 04 §4.8.

## UI session state (RULE-STATE-001)

Interface state (window geometry per display, last view, filters, drafts…) persists in the
`ui_state` table via one `ui-state` module, with intuitive per-view and global clearing.
Never blocks, never touches posted/audited data. Details: doc 04 §4.9.

## UI stack (ADR-0007)

`gpui-kit = { version = "0.11.0", features = ["gpui-fast"] }` — the single UI dependency;
all styled components catalog-first; our widgets follow the Figma model (components +
variables + variants linked to SSOT, doc 04 §4.8) on top of gpui-kit's styled components.

## Search & storage (ADR-0005/0006)

Tantivy index = derived cache next to the DB, write-through, rebuildable (ADR-0006). One
user-owned SQLite file via rusqlite, WAL, migrations, hash-chained append-only journal
(ADR-0005/0010, doc 05).
