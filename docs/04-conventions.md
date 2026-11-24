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
- Primary data also includes **data-as-dependencies**: IANA tz database (via `jiff`), ISO
  4217, later ISO 3166/639 — referenced as dependencies or vendored files, never re-typed.
- Files carry a `[source]` header; the `primary-data` crate loads them via `include_str!`,
  parses once at startup, validates, exposes typed accessors. Invalid primary data fails at
  startup, loudly.
- **Referential integrity (RULE-DATA-002):** primary data cross-references by ISO code.
- **Primary data is modularized too (RULE-DATA-003):** a growing scope becomes a directory —
  `data/<scope>/index.toml` is the SSOT entry listing the scope's files; each file owns one
  small concern; overrides sit adjacent (e.g. `data/currencies/overrides.toml`). The loader
  merges + validates the whole scope.
- Shipped: `currencies.toml`, `errors.toml`, `defaults.toml`, `load-policies.toml`,
  `perf-budgets.toml`, `shortcuts.toml` (keyboard map — SSOT, remappable, RULE-INPUT-001),
  `theme/` (directory scope, RULE-DATA-003: `index.toml` + colors/spacing/radii/typography/
  elevation tokens — the design-language variables). Planned:
  `countries.toml`, `languages.toml`, `category-presets.toml`,
  `default-accounts.toml`.
- Changing primary data = changelog entry; schema change = ADR. User-level data is *not*
  primary data; it lives in the DB.

## 4.2 Modularization, single source of truth, recursive engines (RULE-MOD-001, RULE-SSOT-001, ADR-0013)

- **Folders group; folders nest freely.** File per responsibility. No artificial line limit,
  but files stay small and reusable: split proactively as a file grows.
- **Variants and edge cases live adjacent to the thing they vary**: `posting/variant_transfer.rs`,
  `posting/edge_case_zero_amount.rs`, …
- **SSOT (RULE-SSOT-001):** every concept is defined in exactly one file; everything else
  `use`/`pub use`es it and links to it. Applies to code, primary data (4.1), and UI (4.9).
- **Engines are recursive:** an engine is not a monolith — it is a folder of parts that are
  themselves engines. Some parts are **shared** (live in shared crates/helpers), some are
  **localized** to that engine's purpose (live inside its folder). A domain slice = one
  crate composed of many engines, not "one engine".
- Crate skeleton (shape, not file-cramming):

```
crates/<domain>/
├── README.md                 ownership, invariants, ADR links
└── src/
    ├── lib.rs                only `pub mod` re-exports
    ├── api/mod.rs            public API — the only import other crates use
    ├── api/<operation>.rs    one file per operation as the API grows
    ├── <thing>/              one folder per domain concept
    │   ├── mod.rs · model.rs · engine/ (engine is itself a folder of sub-engines)
    │   ├── pipeline/ (stage_01_*.rs …) · guard.rs · validate.rs · error.rs · events.rs
    │   ├── variant_*.rs · edge_*.rs · tests/
    └── …
```

- **Engine roles:** domain engines, helper engines (`helper_*`, e.g. validation), guard
  engines (`guard_*`), brokers (`broker_*` — the only engine↔engine path), pipelines
  (`pipeline_*`, staged + resumable).
- **Shared engines (RULE-SHARED-001):** anything generic is implemented **once** in the
  shared crate `crates/shared/` and *localized* per feature via thin adapter/variant files in
  the feature crate — never re-implemented per feature. Before writing anything in a feature,
  check the shared registry below; a new generic capability goes into `crates/shared/`
  (generic over entity kind via an `ActionableEntity`-style trait). `govern meta` fails if a
  feature re-implements a registered shared engine without an adapter marker. Localization
  means: the shared engine owns the *rule* (audit, undo-scope, validation pipeline, job
  wrapping); the feature adapter owns only the *entity-specific constraint* (e.g. "account
  names must be unique in this profile") and the localized strings.
- `api/` is the contract; internal refactors never change it without an ADR.

### Shared engine registry (SSOT — doc 04 §4.2; grow it, never duplicate it)

`crates/shared/` is organized as `actions/`, `helpers/`, `guards/`:

| Category | Shared engines | Why shared |
|---|---|---|
| **actions/** (generic over entity; audited; job-wrapped for bulk) | `rename` · `archive`/`restore` · `duplicate` · `delete` (reversal for posted/audited entities, hard delete only for non-audited) · `move` (to another parent/container) · `merge` (e.g. merge two payees/categories — re-points all references, audited) · `add_tag`/`remove_tag` · `attach`/`detach` · `set_note` · `change_status` · `pin`/`unpin` · `set_color_icon` · `set_order` (reorder within a list) · `export_selection` · `link`/`unlink` (e.g. reversal links) · `bulk_apply` (any of the above over a selection, one job) | identical semantics for every entity kind; one audit/undo/job behavior; SSOT |
| **helpers/** | `validate` (format validators: text, algebra-number, date, currency code vs primary data, color, id) · `parse`/`format` (locale-aware pair — the SSOT for how money/dates are read and displayed; money always shows the ISO code, RULE-FMT-001) · `query` (generic filter/sort over any list; feeds tables + the timeline filters; built on the search DSL) · `paginate` (page/virtualize per `data/load-policies.toml`) · `time` (scope math: day/week/month/trimester/year/custom, DST-safe via jiff; feeds the timeline) · `recurrence` ✅ M2b IMPLEMENTED (`shared::api::recurrence`: anchor-based bounded expansion, exceptions, reconciled-by-default past projection; RULE-RECUR-001…003) · `diff` (compare two datasets/scopes — timeline compare mode) · `selection` (bulk-selection state in `ui_state`) · `undo` (non-audited state only, RULE-UNDO-001) · `i18n` (locale catalogs, EN fallback, message/placeholder lookup) · `theme` (tokens from `data/theme/*.toml`) · `notify` (severity routing → toast / dialog / fatal + copy-diagnostic, RULE-ERR) · `export` (versioned serialization, doc 05 §5.11) | one behavior everywhere; prevents per-feature drift of formatting, filtering, parsing, scheduling |
| **guards/** | `ledger_balance` (debits = credits) · `hash_chain` (tamper evidence) · `audit_immutable` (no UPDATE/DELETE paths) · `money` (no floats, minor units, ISO codes) · `iso_ref` (primary-data referential integrity, RULE-DATA-002) · `schema` (migrations applied) | invariants enforced in one place, reused by every crate's `guard.rs` |

Example of localization: renaming an account is `shared::actions::rename` + an adapter
supplying the uniqueness check; renaming a profile's display name is the *same* engine + a
different adapter. The engine owns the rule; the adapter owns the constraint and strings.

## 4.3 Validation (RULE-VAL-001)

All external input (user input, imported files, rates packages, settings) is validated at
the boundary before any engine runs; engines re-assert preconditions. Prefer making invalid
states unrepresentable (smart constructors, newtypes, enums). **Number/date input parsing is
locale-aware** (decimal comma vs dot, date order per locale, fiscal periods — doc 05 §5.12);
the validation engine owns the parsing rules (one SSOT module); invalid input gets a
BK-coded localized message.

## 4.4 Errors, warnings, user-facing diagnostics (RULE-ERR-001…004)

- Each crate has a `thiserror` enum; every variant carries a stable code `BK-<AREA>-<NNNN>`;
  `?` propagates; `anyhow` at the binary edge.
- **Registry (SSOT):** `data/errors.toml` lists every code with severity + i18n key.
  `xtask govern errors` fails CI if a code in code is missing from the registry or lacks a
  message in every shipped locale. Messages render in the chosen language (EN/RO first).
- **Severity:** `warning` (toast, app continues) → `error` (dialog, app continues) →
  `fatal` (dialog + one-click **“Copy diagnostic”**: code + localized message + context +
  version/OS — paste into a GitHub issue). Engines keep working until a *fatal* error; all
  fatals are surfaced for reporting.
- No `unwrap()`/`expect()`/`panic!` outside tests; `dbg!`/`todo!` denied; `unsafe_code =
  forbid`.

## 4.5 Money (INV-MONEY-001, ADR-0004)

`Money { amount_minor: i64, currency: CurrencyCode }`; minor units from ISO 4217 primary
data (not always 2 decimals); no floats in money code; serialize as string or minor-unit
integer, never a bare JSON number.

## 4.6 Events & ownership (RULE-EVT-001)

Every state change emits a typed domain event (what / when / who-what / why). The `bus`
crate dispatches to subscribers (`audit`, `search`, `rates`, `jobs`, `ui`); subscribers are
idempotent; crash-surviving reactions go through a durable local outbox and replay. One crate
owns each table; others use its `api`.

**Kind registry mechanics (M2-hardening, ADR-0025):** dotted kind strings are defined ONLY
as consts in `bookkeepy_contracts::kinds` and registered in `data/events.toml` (SSOT, loaded
by `primary-data::events`). Engines emit via consts, so a rename is a compile error, not
runtime drift. `cargo xtask govern events` fails on any stray kind literal in code and on
any const/registry mismatch.

## 4.7 Async-first (RULE-ASYNC-001)

Primarily async. All engine/pipeline/broker APIs are async; UI drives them with `cx.spawn`;
storage runs on the background executor / dedicated DB thread behind a bounded channel; the
UI thread never blocks on I/O. Long flows emit progress events and are cancellable (4.8).

## 4.8 Jobs orchestration (RULE-JOB-001, ADR-0015)

All potentially long/failing async operations are **Jobs** managed centrally by the `jobs`
crate (`engine_jobs` + `broker_jobs`) — never inline per call site. A Job is a durable state
machine in the `jobs` table: `queued|running|paused|done|failed|cancelled`, progress,
attempts, `max_attempts`, payload, result, BK error code. **Auto-retry** with exponential
backoff + jitter up to `max_attempts` (`data/defaults.toml [jobs]`), then **manual retry
surfaced** to the user (toast + Jobs panel). Progress events drive **one shared indicator
with pause / resume / cancel**; a startup driver resumes interrupted jobs (full
resumability). Job types are persisted pipelines (exact resume). UI: one non-modal Jobs
drawer (RULE-UX-001).

**RULE-JOB-002 (backoff column, M2-hardening, ADR-0025):** the backoff is persisted, not
in-memory — `jobs.not_before` (schema v4) is set by `fail` to `now + backoff_delay(attempts,
id)` (exponential 2s→…→cap 300s + deterministic id-hash jitter, no RNG) and `claim_next`
only picks jobs whose `not_before` has passed. A poison job can never spin in a hot
retry loop; after `max_attempts` it lands in `failed` for manual retry.

## 4.9 UI system — gallery-first, Atomic Design, capability traits (RULE-UI-001…009, ADR-0017)

- gpui-kit styled components, catalog-first (ADR-0007), per the vendored `skills/`.
- **Gallery-first workflow (RULE-UI-006):** every interface component is built and reviewed
  **in the `gallery` first**; when approved there, it is used in the app. There is **one set
  of components only** — the gallery is a harness/viewer over the same `ui` widgets crate,
  never a second copy. The gallery showcases **all** gpui-kit styled components by default,
  arranged by atomic layers, with our variants/edge cases alongside.
- **Gallery versioning (RULE-UI-008):** every widget folder carries a `meta.toml`
  (component id, **version**, capability traits, variants, edge cases, changelog). The
  gallery renders the version history; any change to a component/variant/trait bumps its
  version and appends to its changelog. `govern meta` cross-checks `meta.toml` against the
  code (traits actually enabled, variants actually linked) — the same registry governors
  track (4.11).
- **Atomic Design (Brad Frost) governs the interface structure:** atoms = our wrappers over
  gpui-kit styled components (`ui/src/widgets/atoms/<component>/`); molecules = composed
  controls (`widgets/molecules/`); organisms = feature blocks (`widgets/organisms/`);
  templates = layouts; pages = views (`ui/src/views/<feature>/<screen>/`). Shared, reusable
  parts build ever more complex interfaces.
- **Figma model:** variables = theme tokens as primary data (`data/theme/` — directory scope
  per RULE-DATA-003, shipped: `index.toml` + colors/spacing/radii/typography/elevation; the
  Bookkeepy v1 design language, dark theme native + light mirror; semantic tokens `bg`/`fg`/
  `accent`/`danger`/`success`/`warning`/`border`/…, never raw values in code); components
  = source + props + variants **linked to the SSOT source** — a variant (e.g. a mini calendar)
  is `variant_mini.rs` extending the source, never a copy.
- **Capability traits (RULE-UI-007):** capabilities are registered once in
  `ui/src/capabilities/` (SSOT registry). **Variants are built only by enabling/disabling
  registered traits on the source component.** Examples: Input's `algebra` capability
  (`+ − × ÷` evaluation) — enabled on amount fields, not on text fields; Table's
  `selectable` capability — enables the checkbox column + bulk bar (F20); Timeline-bound
  components carry the `timeline` trait — they render according to the global time scope
  (RULE-TL-001) and switch to a comparison variant when the `compare` trait is on (F19).
- **Placeholders (RULE-UI-005):** every input and every interface element has a localized
  placeholder (plus label / helper / error text). `xtask govern ui` fails on any input
  without a placeholder i18n key.
- **Input modalities (RULE-INPUT-001):** (a) mouse, (b) trackpad/touchpad gestures,
  (c) keyboard shortcuts — all usable **in combination at the same time**. Shortcuts are
  primary data (shipped: `data/shortcuts.toml` — flat entries: `id`, `keys`, `action` (dotted
  engine action id), `label_i18n_key`, `scope` (global/screen), `when` guard; SSOT,
  remappable, localized; no hardcoded keys anywhere else), bound via gpui
  actions/keybindings. (d) **Voice control via AI is parked** (doc 10); when un-parked it
  must run **on-device** (a cloud model would need an ADR amending ADR-0011).
- **Presentation defaults (RULE-UI-008):** data is shown **as tables (rows/columns) by
  default** — the Table atom is the standard data presentation; cards, kanban, charts are
  variants enabled per context via traits. A shared **context-menu** atom (right-click /
  long-press / menu key) lists the applicable shared actions per entity (F21). Managers are
  centralized screens (F21) built from the same atoms/organisms.
- **Visual language (RULE-UI-009):** the interface is rich and interactive — charts,
  interactive graphics, the timeline — designed to invite mouse interaction; not just cards
  with numbers. Use gpui-kit's chart/graphic components + custom canvas drawing where needed.
- `ui` depends only on `app-services::api` + `contracts` (RULE-UI-004); presenters are the
  SSOT for view models; session state per RULE-STATE-001 (persisted in `ui_state`, intuitively
  clearable, never touching posted data).

## 4.10 Audit vs app history vs undo (RULE-AUD-001, RULE-UNDO-001) + never-block (RULE-UX-001, ADR-0014)

- **`audit_log` — sacred:** append-only, hash-chained, tamper-evident; what / when /
  who-what / why for every **posted** change; never editable or deletable through the app.
- **`app_history`:** UI-level tracking, separate table, user-configured retention (keep /
  N days / on exit / off), clearable anytime; never money truth.
- **Undo/redo (RULE-UNDO-001)** only affects non-audited state (drafts, inputs, filters, UI
  state). Posted changes are changed only via linked corrections/reversals, which post new
  audit events (record → event A; edit → correction posts event B).
- **Never block the user (RULE-UX-001):** reminders are dismissible; background jobs never
  obstruct user actions; no modal lockouts.
- **Soft delete only (RULE-LIFE-001, ADR-0024):** no hard delete of audited/catalog data.
  Catalog entities use `archived` (excluded from pickers/forward projections, kept in
  history); transactions use `voided`, which posts a linked reversal — computed money metrics
  never filter posted rows by a flag, exclusion happens inside the journal itself
  (INV-LEDGER-001). SSOT: `bookkeepy-ledger::api::lifecycle`.

## 4.11 Governors — repo tree + metadata registries (RULE-GOV-001)

`cargo xtask govern` runs a tree of governors (each may nest):

- `deps` — dependency direction (RULE-ARCH-001); `net` — HTTP only in `updater`/`rates`,
  host allowlist (RULE-NET-001); `lints`; `adr` — numbering/status; `perf` —
  `data/perf-budgets.toml` vs criterion benches; `errors` — registry ↔ code ↔ locales;
  `events` — event-kind consts ↔ `data/events.toml`, no stray literals (RULE-EVT-001);
  `data` — primary data validates, ISO integrity (RULE-DATA-002/003); `ui` — every input has
  a placeholder key, every widget/variant declares only registered capability traits,
  variants link to their source (RULE-UI-005/007); `docs` — see below; `meta` — see below;
  **`seams` — RULE-SEAM-001 (ADR-0026): cross-layer seams are guarded statically** —
  S1 risky deps (rusqlite/fs2/sha2/hex only in their allowed crates), S2 every
  `tx::post`/`tx::post_with` in ledger/shared maps errors through `from_storage` (BK-codes
  survive the `StorageError::Foreign` boundary), S3 literal SQL touches only the crate's
  own tables (storage: its tables + CREATE TRIGGER/INDEX guards; ledger: its 9; shared:
  none — dynamic via the `Actionable` allowlist), S4 only storage constructs connections.
  Paired with boundary contract tests (`crates/ledger/tests/seams.rs`).

- **Metadata registries (RULE-GOV-001):** governors track, per engine/widget/crate — via
  code annotations (e.g. a `#[bookkeepy(meta…)]`-style attribute or per-crate manifest) —
  **which optimizations, edge cases, variants, capability traits, and input-handling
  behaviors have been applied**, plus other metadata. If implementation is inconsistent
  with the registry (missing edge case, unregistered trait, untracked optimization),
  `govern` warns/errors in `cargo xtask govern` and CI — so everything is coded
  consistently and drift is visible.
- **Bidirectional docs↔code traceability:** every `RULE-`/`INV-`/`ADR-` ID in docs must
  exist as an annotation in code, and vice versa; `govern docs` fails otherwise. **What is
  already written in the docs stays**: documented content is never silently removed — doc
  changes are explicit edits in the canonical file (RULE-DOC-001), and the governor diffs
  the rule/ID registry to catch accidental loss.
- **Logic governors (in-app):** every engine folder has `guard.rs` — invariant checks before/
  after operations; violations emit BK-codes and are audit-logged.

## 4.12 Documentation conventions (RULE-DOC-001)

Modular files; `docs/README.md` is the index; **one canonical file per topic** — cross-link,
never duplicate; split by subtopic when a file outgrows its topic. ADRs are for decisions
only, numbered, immutable once accepted (supersede via new ADR). Routine changes →
CHANGELOG. Every pub item documented; every crate has a README.

## 4.13 Git & CI hygiene (RULE-PROC-001/002)

`main` protected: PRs only, required CI + review, conventional commits referencing the
issue; branch `<type>/<short-name>`.

**Frontend delegation (RULE-PROC-002):** backend work (engines/pipelines/storage/docs) is
done in this session; when UI work is needed, this session produces a **short prompt** for a
separate frontend AI agent, which implements in this same repo following these docs
(gallery-first, one component set, capability traits). The frontend agent never changes
backend contracts without an ADR.

## 4.14 Change propagation (RULE-SYS-001)

Bookkeepy is a **system**: each small change propagates proportionally to its size through
the relationships between system elements (crates, rules, ADRs, docs, primary data, governor
registries, tests, locales). Before merging any change, **trace its propagation**: affected
crates (`govern deps`), rules/ADRs touched, docs canonical places, primary data, governor
registries (4.11), tests, locales, CHANGELOG. This section is the mind model for maintaining
docs and code together.

## 4.15 Build channels (RULE-CHANNEL-001, ADR-0021)

One codebase, two build channels, fixed at **build time** via `BOOKKEEPY_CHANNEL`:
`cargo xtask dev` builds `dev` (extra tooling: perf monitoring overlays, debug screens,
verbose logging); release builds (`release.yml`) are `release`. Dev-only code must be
behind the `dev-channel` cargo feature so it is **not compiled into shipped binaries**.
The channel is embedded (`bookkeepy-build-info::CHANNEL`), shown in About and in every
diagnostic bundle; the updater never crosses channels.

## 4.16 Code registry — tracking everything we code (RULE-META-001)

`docs/registry/code-map.toml` is the SSOT for what exists in the codebase: every crate,
engine, shared engine, widget family and dev-only module is registered at creation
(`cargo xtask new-crate` appends automatically). Statuses: `active | deprecated | removed`
— a `deprecated`/`removed` entry must name a `successor` or `removal_note`, so deprecating,
replacing, refactoring or removing anything is a one-place, auditable edit that old docs,
ADRs and changelogs still resolve against. `cargo xtask govern meta` fails on unregistered
crates, dangling paths, invalid statuses or missing successors.

## 4.17 Logging, time, error-surfacing rules (RULE-LOG-001, RULE-TIME-001, RULE-ERR-005)

- **RULE-LOG-001:** every engine log line carries structured fields: `engine`, `job_id`,
  `profile_id`, `event` kind — logs stay greppable and diagnostic bundles can correlate.
- **RULE-TIME-001:** store UTC (`jiff::Timestamp`) everywhere; convert to the profile
  timezone only at render; DST handled by the IANA tzdb (jiff). Never store local time.
- **RULE-ERR-005 (severity → UI mapping):** `warning` → toast · `error` → dialog (app
  continues) · `fatal` → dialog with **Copy diagnostic** (`bookkeepy-diagnostic@1`) —
  surfaced via the shared `notify` helper only, so every screen behaves identically.

## 4.18 Change checklist (every PR)

1. Fits an existing crate/thing folder (4.2)? New concept → new folder; SSOT respected?
   Architecture-affecting → ADR first.
2. New/changed rule or invariant? Register the ID; governor if mechanical; docs↔code
   traceability intact (4.11).
3. Primary data touched? → 4.1 process (RULE-DATA-002/003).
4. Boundary validation (4.3)? BK-coded errors + registry + locales (4.4)? Money (4.5)?
5. Events (4.6)? Async + Job if long-running (4.7/4.8)? UI: placeholder + capability traits
   + variant links (4.9)? Session state via ui-state? Audit vs history respected (4.10)?
   Guards updated (4.11)?
6. Tests (doc 09) incl. property/golden for money, retry/resume injection for jobs; edge
   cases enumerated in `edge_*.rs` against the doc-10 register; propagation traced (4.14);
   docs updated in the canonical place; CHANGELOG entry; SPDX header present (`govern
   lints`); new crates/engines registered in `docs/registry/code-map.toml` (4.16).
