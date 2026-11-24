# 10 — Process, roadmap, decisions log

## How we work (long-term collaboration)

1. **You prompt a surface-level wish.**
2. **I interpret it as an engineer**: feature domain (doc 02), owning crate/slice (doc 03),
   rules (doc 04), quoted fintech principles (doc 06), tests (doc 09).
3. Architecture/rules/invariants change → **ADR first**, then issue, then PR with tests +
   docs + CHANGELOG.
4. Nothing merges without CI green + review. Nothing ships except via tagged GitHub
   Releases.

## Roadmap (milestones)

| M | Name | Outcome |
|---|---|---|
| M0 | Repo & foundations | workspace, `core` (Money/Currency), `primary-data`, CI skeleton, governors. **DONE 2026-11-24 — see "M0 completed" below.** |
| M0.5 | Repo-grade foundation | contracts+bus, diagnostics+panic hook, deny/vet/Renovate, branch protection+templates, `xtask new-crate`, build-info, logging, i18n pipeline, profile registry, GPL embed, updater wire spec, fixtures structure, gitattributes+pre-commit, criterion harness. **DONE 2026-11-24 — see "M0.5 completed" below.** |
| M1 | Storage skeleton | `storage`: connection, migrations, WAL, backup/restore/verify, single-instance lock, crash-recovery self-check. |
| M2 | Ledger engine | double-entry postings, derived balances, reversals, hash chain, audit log; property tests. |
| M3 | Accounts & transactions UI | gpui-kit shell, **component gallery** (all styled components + versioned variants), first screens, ui-state, onboarding/empty states, EN/RO, keyboard map (`data/shortcuts.toml`), **timeline spine (F19)**. |
| M3b | Profiles | profile registry + switcher, isolated data roots, scoped export → import between profiles (F18, ADR-0018). |
| M4 | Import/export | CSV/OFX/QIF pipeline, dedup, reconciliation breaks; export package format v1. |
| M5 | Search | tantivy engine + query DSL, write-through + rebuild. |
| M6 | Budgets, reports & managers | envelopes, reports, golden tests, PDF/print; centralized **managers** (categories, tags, payees, debts & loans, documents, subscriptions) + context menu; recurrence schedules (ADR-0020). |
| M7 | Multi-currency, FX & live rates | rates mirror automation (BNR first) + `engine_rates` + rates UI (doc 11). |
| M8 | Documents, backups UI, reminders | attachments, snapshots, retention reminders, local notifications. |
| M9 | Scale & polish | materialized aggregations, a11y/keyboard pass, multi-display window mgmt, perf budgets green. |
| M10 | Updater & packaging | updater, installers/uninstallers (INV-APP-001), first `v0.1.0` release. |
| M11 | Future security | app lock / at-rest encryption (parked, see below). |

## M0 completed (2026-11-24) — what was actually built + findings the docs didn't cover

Built (backend only; UI comes from M3 via the frontend agent, RULE-PROC-002):

- `rust-toolchain.toml` (1.100.0 + rustfmt/clippy), workspace `Cargo.toml` (edition 2024,
  workspace lints: `unsafe_code=forbid`, deny unwrap/expect/dbg/todo, `float_arithmetic`
  denied crate-level in `core`), `.cargo/config.toml` alias.
- `crates/core` (`bookkeepy-core`): `Money` (i64 minor units + `CurrencyCode`, checked
  add/sub/neg, conservation property-tested with proptest, serde as integer struct — never
  a bare JSON number), `CurrencyCode` (shape-checked; ISO-set validation lives in
  primary-data), `ErrorCode` (`BK-<AREA>-<NNNN>`), `EntityId` (UUIDv7), `time` (jiff 1),
  `codes` + `all_error_codes()` (governor-checked against `data/errors.toml`).
- `crates/primary-data` (`bookkeepy-primary-data`): one module per data scope, each
  `include_str!` + `OnceLock` parse + structural validation; `validate_all()` runs at
  startup and in `cargo xtask govern data`. 27 tests green.
- `xtask` governor tree: `data`, `errors`, `adr`, `docs`, `deps`, `net` implemented and
  green; `lints`/`perf`/`ui`/`meta` are explicit stubs that activate as code lands
  (never silently skipped). CI (`.github/workflows/ci.yml`): fmt + clippy -D warnings +
  tests + governors on ubuntu/windows/macos + cargo audit.

Findings discovered while coding (now part of the architecture, recorded here per the
overlay rule):

1. `cargo xtask` does not auto-expose a workspace member binary → added
   `.cargo/config.toml` `[alias] xtask = "run -p xtask --"`.
2. Cargo 1.100 warns on *unused* `[workspace.dependencies]` entries → dependency entries
   are declared on first use; the full policy set stays in doc 07 (noted in root
   Cargo.toml). Keeps builds warning-free under `-D warnings`.
3. `[lints] workspace = true` is a separate table (not `package.lints`).
4. `Money` is `Clone` but not `Copy` (`CurrencyCode` owns a String) — ops take `self` by
   value; callers clone. Fine for a money type (no accidental copies).
5. `core::Display` for `Money` is the canonical *machine* form (`1234 RON`); user-facing
   formatting with locale separators + always the ISO code (RULE-FMT-001) belongs to the
   shared `format` helper, which reads minor units from primary data (dependency direction
   forbids core from reading `currencies.toml`). Noted in doc 05 §5.1.
6. Data generation bugs the governors/tests caught: unescaped quotes in country names
   (SUCRE), empty-name funds codes (XAD/BOV/… — omitted, fiat-only scope, fintecheng:
   "currency codes identify fiat only"; 168 fiat currencies), error-registry `area` must
   equal the code prefix (`store`, `fx`, …), `defaults.toml` table structure.
7. `clippy::should_implement_trait` → `Money::neg` renamed `checked_neg` (avoids confusion
   with `std::ops::Neg`).
8. Rust 2024 `let`-chains used where clippy demands collapsing.

## M0.5 completed (2026-11-24) — repo-grade foundation, backend only

Approved brainstorm items implemented (all but item 10, which is docs+primary-data handed
to the frontend agent per RULE-PROC-002 — theme tokens `data/theme/*.toml` + shortcut map
`data/shortcuts.toml`):

1. **Contracts + bus** — `crates/contracts` (`EventEnvelope` + `Actor`, ADR-0009 envelope),
   `crates/bus` (in-process publish/subscribe; the durable outbox lands with storage M1 —
   noted in code so the transient/durable split is explicit).
2. **Diagnostics + panic hook** — `crates/diagnostics`: `bookkeepy-diagnostic@1` bundles
   (JSON for issues + copyable text for the clipboard, RULE-ERR-003) and `install_panic_hook()`
   mapping panics to `BK-APP-0001` fatal bundles.
3. **Supply chain** — `deny.toml` (license allow-list incl. GPL-3.0-or-later, AGPL/SSPL
   banned, duplicates + wildcards denied, advisories/yanked denied, crates.io-only sources)
   enforced by a CI `deny` job; `cargo vet` job present but advisory until M2 (first
   crate-by-crate `cargo vet certify` pass); `.github/renovate.json` (cargo weekly, major
   monthly; enabling the Renovate app on the repo is a one-click repo setting).
4. **Repo protection** — CODEOWNERS, PR template (doc 04 §4.15 checklist), bug/feature
   issue templates (diagnostic-bundle-first), branch protection on `main` (required
   contexts = CI job names, 1 review, no force-push) set via `gh api`.
5. **`cargo xtask new-crate <name>`** — scaffolds crates in the doc-04 shape (api/ surface,
   forbid-unsafe, README, workspace registration) and reminds to extend the `govern deps`
   edge map. Dogfooded: all seven new crates were created with it. CRLF-safe members edit.
6. **Build info** — `crates/build-info`: build.rs embeds version/git-sha/rustc/build-time
   (SOURCE_DATE_EPOCH aware); root re-exports; `full()` one-liner for logs/diagnostics.
7. **Logging** — `crates/logging`: `init(dir)` → daily-rotated local files (14 kept),
   `RUST_LOG` override, default `info,bookkeepy=debug`; never any network sink
   (RULE-PRIV-001). `BK-LOG-0001` registered.
8. **i18n pipeline** — `data/locales/en.toml` + `ro.toml` (flat quoted dotted keys,
   `[meta]` tag must match file stem), `primary-data::locales` (OnceLock parse, key-parity
   validation), cross-check that every `errors.toml` `i18n_key` exists in **every** locale
   (in `validate_all` → `govern errors` enforces it), `crates/i18n` lookup with EN fallback
   then key-passthrough (never panics on a missing string).
9. **Profile registry** — `crates/profile`: `app_data_dir()` (directories crate),
   `profiles.toml` registry (atomic write temp+rename), `ProfileRoots` layout matching
   doc 05 §5.2 (`app.db`, `index/`, `rates/`, `exports/`, `logs/`, `drafts/`), invariants
   validated (unique ids/names, `#rrggbb`, no path escapes — INV-PROF-001/002).
   `BK-PROFILE-0001` registered.
11. **GPL embed** — `build-info::LICENSE_TEXT = include_str!("../../../LICENSE")`; About
    screen shows it offline; test asserts GPL v3 text.
12. **Updater wire format** — doc 08 "Release wire format v1" (`bookkeepy-release-manifest@1`
    + SHA256SUMS + per-target assets; verify-then-install only).
13. **Fixture corpus** — `tests/fixtures/` structure + rules (synthetic data only,
    generated by the app, upgrade-path CI activates at M1; INV-DATA-001).
14. **Hygiene** — `.gitattributes` (LF everywhere; db artifacts binary),
    `.githooks/pre-commit` + `cargo xtask hook install|pre-commit` (fmt + fast governors +
    clippy), xtask stubs `dev|gallery|bench`.
15. **Bench harness** — criterion wired (`crates/core/benches/money_ops.rs`), CI `bench`
    job runs `cargo bench --workspace -- --test` (compile+run sanity); measured comparison
    against `data/perf-budgets.toml` lands with the perf governor.

Findings discovered while coding (recorded per the overlay rule):

1. `directories::ProjectDirs::from` returns `Option` — on an OS without a data dir we fall
   back to a temp path instead of panicking (RULE-ERR-003: environment quirks never crash).
2. serde internally-tagged enums can't hold newtype string variants → `Actor` uses
   adjacently tagged (`{type, value}`).
3. `tracing_appender` builder returns `InitError`, not `io::Error` → mapped via
   `io::Error::other`; `max_log_files` takes `usize`.
4. `criterion::black_box` is deprecated in favor of `std::hint::black_box` (used).
5. Benches are separate targets: the `cfg_attr(test)` unwrap-allow doesn't cover them →
   bench files carry their own `#![allow(clippy::unwrap_used, expect_used)]` (same as
   integration tests).
6. xtask is a dev tool (failures exit non-zero) → it carries a crate-local
   `#![allow(clippy::expect_used, unwrap_used)]`; shipped crates keep the deny.
7. `govern net` allows the canonical repo URL string anywhere (it's a label in About/report
   links, never a request); all other URLs outside `bookkeepy-updater`/`bookkeepy-rates`
   still fail.
8. Workspace path-deps for not-yet-consumed crates trigger cargo's unused-dependency
   warning → they are declared on first use like any other dep (comment in root Cargo.toml).

## Round-2 brainstorming decisions (approved by owner)

1. **Single instance (RULE-APP-001):** enforced. A second launch does not open a new window —
   it sends an IPC "focus" message and focuses the existing window. Lock = per-book instance
   lock with PID + heartbeat, so a crash leaves no stale lock (take-over with a notice).
2. **Crash recovery & self-check on open:** WAL + single-write transactions guarantee no
   torn writes. On open the app starts immediately (never blocks, RULE-UX-001) and runs a
   background self-check job: `quick_check`, hash-chain verify, invariant guards. Drafts
   from `ui_state` are restored with a "Restore draft?" affordance.
3. **Uninstall never deletes user data (INV-APP-001):** uninstall removes only the app. The
   uninstaller shows an **unchecked-by-default** checkbox "also delete my data"; before it,
   an export screen offers a full export package and explains: *the data stays on this
   device; reinstalling Bookkeepy finds it unaltered; the export lets you use it in other
   apps.* Upgrades/reinstalls likewise never alter data (INV-DATA-001).
4. **Scale:** materialized aggregations (per-account-period balances, category totals…)
   refreshed by event-bus subscriptions as background jobs, with on-demand recompute
   fallback; virtualized paginated lists; budgets for 100k+ transaction books in
   `data/perf-budgets.toml`.
5. **Local reminders engine:** `reminders` crate — rules (subscription due, budget threshold,
   backup due, rates stale) → local notifications (in-app + OS notification), dismissible
   and snoozable, never blocking (RULE-UX-001).
6. **Window & multi-display management:** window geometry persisted per display id (from
   `system_context`); restore on the display where it closed; display disconnect → primary
   display, non-blocking notice; live DPI/scale changes handled via gpui-kit platform events.
7. **Security threat model** (local attacker, stolen device, tampering): documented in
   doc 08. **App lock / password and at-rest encryption are parked** — future feature, not in
   current scope. For now: data file in the user-profile data dir with OS permissions, no
   secrets stored, hash-chain tamper evidence.
8. **PDF/print pipeline:** reports render to PDF (maintained GPL-compatible crate, chosen
   at implementation with rationale — doc 07) + OS print dialog; PDFs are golden-tested.
9. **Export formats are versioned specs** (doc 05 §5.11) — portability forever.
   **Search query DSL** (doc 05 §5.7). **Fiscal periods & custom date ranges** in reports
   (doc 05 §5.12). **Locale-aware number/date input parsing** in the validation engine
   (doc 04 §4.3).
10. **First-run onboarding & empty states:** welcome → create/open book → first account →
    optional import; optional sample book; privacy disclosure dialog at setup (doc 08).

## Parked feature ideas (owner will specify later; logic patterns only for now)

Auto-categorization rules on import · command palette + global quick-add · user-defined
report templates · **app lock / at-rest encryption** · **voice control via AI** (when
un-parked it must run on-device — a cloud model would need an ADR amending ADR-0011; input
modalities now: mouse + trackpad + keyboard, RULE-INPUT-001). (Multi-book was specified by
the owner as **Profiles** — F18, ADR-0018 — no longer parked.)

## Edge-case & variant register (classes to enumerate per domain)

Concrete edge cases live in `edge_*.rs` adjacent to their engine (ADR-0013); this register
is the SSOT checklist of *classes* every domain must consider (point 9):

- **Empty states:** no book / no accounts / no transactions / no rates / empty search /
  empty report period — first-run onboarding covers the user-facing side.
- **Money:** zero-amount transaction (reject with BK-code) · negative balance (allowed,
  warned — never clamped) · i64 overflow on huge amounts (reject) · 0-minor-unit (JPY) and
  3-minor-unit (BHD) currencies · rounding residuals → rounding account · cross-currency
  two-hop conversion rounding.
- **Time:** DST transitions, leap days, value date far past/future, backdating across a
  fiscal period, system clock changes, time-zone change mid-session.
- **Import:** empty/malformed file, wrong encoding, unknown currency code, duplicate rows,
  partial failure (resume), file locked by another app.
- **Rates:** no rate for a date (BK-FX-0002 + defaults chain), source switch (history kept),
  deleted manual rate (irrecoverable → default), multiplier quotes (HUF per 100).
- **Jobs:** disk full, network lost mid-download (resume), cancel mid-stage, app killed
  mid-job, retry storms (backoff), two jobs on the same resource (per-resource mutex).
- **Storage:** corrupted DB (restore-from-backup UX), read-only data folder, disk full,
  interrupted migration (tx rollback), file written by a newer app version (refuse with a
  clear message), older file (migrate — INV-DATA-001).
- **UI:** window below min size, display disconnected, DPI change, theme switch, very long
  input, many accounts (virtualization), draft vs posted conflict.
- **Security/lifecycle:** data folder moved/copied, permissions changed by OS, audit tamper
  detected (report + continue), stale single-instance lock after crash.

## Docs hygiene (RULE-DOC-001)

Modular files, one canonical place per topic (index: `docs/README.md`), cross-links instead
of duplication. ADRs only for real decisions; routine changes → CHANGELOG.

## Definition of done (per milestone)

CI green on all platforms · property/golden tests for money · job retry/resume tests ·
docs updated in the canonical place · CHANGELOG entry · EN/RO strings · no HTTP outside
`updater`/`rates` (governor) · docs index accurate.
