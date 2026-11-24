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
| M1 | Storage skeleton | ✅ DONE 2026-11-24 — `storage` crate: WAL conn, forward-only migrations + downgrade refusal, durable outbox, audit hash chain, backup/verify/restore, single-instance lock, self-check, KV. See "M1 completed" below. |
| M2 | Ledger engine | ✅ DONE 2026-11-24 — `ledger` + `shared` crates + storage v2: double-entry postings (hash-chained, trigger-guarded), derived balances, linked reversals, transactions→balanced postings, shared audited actions, lifecycle/soft-delete (ADR-0024), proptest invariants. See "M2 completed" below. |
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

## Round-4 decisions (owner approved all, 2026-11-24 — brainstorm A–E + channels + code registry)

Approved and implemented:

1. **OSS surface:** `CHANGELOG.md` (Keep a Changelog), `SECURITY.md` (private reporting
   via Security Advisories), `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Covenant 3.0),
   `.editorconfig`; repo settings via `gh api` (secret scanning + push protection +
   vulnerability alerts on, Dependabot off — Renovate only, Wiki off, labels, `main`
   protection without required checks).
2. **License hardening:** `SPDX-License-Identifier: GPL-3.0-or-later` on every `.rs`
   (retrofitted; `xtask new-crate` stamps it; `govern lints` enforces); release assets
   include `THIRD-PARTY-LICENSES.md` (cargo-about) + `sbom.cdx.json` (cargo-cyclonedx).
3. **Release engineering:** reproducible release builds (`SOURCE_DATE_EPOCH` =
   release `published_at`), `codegen-units = 1` in `[profile.release]`; `release.yml`
   hardened (channel stamp, licenses, SBOM). Downgrade policy: ADR-0022 / RULE-DATA-004
   / `BK-DATA-0002`.
4. **Dev/release channels (owner request):** ADR-0021 / RULE-CHANNEL-001 — same code,
   channel fixed at build time (`BOOKKEEPY_CHANNEL`), dev-only tooling behind the
   `dev-channel` cargo feature (never compiled into release); About + diagnostic bundles
   show the channel; updater never crosses channels.
5. **Code registry (owner request — track everything we code):** ADR-0021-adjacent rule
   RULE-META-001 — `docs/registry/code-map.toml` is the SSOT of what exists; statuses
   `active|deprecated|removed` with mandatory `successor`/`removal_note` so deprecate/
   replace/refactor/remove is a one-place auditable edit; `cargo xtask new-crate`
   auto-registers; `govern meta` enforces.
6. **New conventions:** RULE-LOG-001 (structured log fields), RULE-TIME-001 (store UTC,
   render local), RULE-ERR-005 (severity→UI: toast/dialog/fatal+copy), INV-MONEY-002
   (rounding half-even at boundaries + explicit rounding account, fintecheng quote in
   doc 05 §5.1).
7. **Testing/tooling:** `cargo nextest` in `xtask check` when installed (fallback
   `cargo test`); cargo-fuzz targets for import parsers at M4; `cargo xtask doctor`,
   `cargo xtask new-adr`, `cargo xtask licenses`.
8. **Rates mirror (owner amended: may run on Actions automatically):** `bookkeepy-1-rates`
   got a scheduled (`0 6 * * *`) + `workflow_dispatch` workflow that fetches BNR, builds
   spec-conformant packages, sha256-checksums and **ed25519-signs** them (ADR-0023;
   secret `RATES_SIGNING_KEY`, pubkey pinned at `keys/rates-ed25519.pub.pem`; app refuses
   bad signatures with `BK-RATES-0002`). TODO for owner: set the `RATES_SIGNING_KEY`
   secret in the mirror repo settings.

## Round-3 decisions (owner, 2026-11-24 — CI minutes + failing action fix)

1. **Failing action fixed**: the M0.5 push's `deny (cargo-deny)` job failed — cargo-deny
   0.22 changed schema (`unmaintained` is now a scope `all|workspace|transitive|none`,
   `severity-threshold` and `licenses.deny` removed, `wildcards` needs
   `allow-wildcard-paths` + `publish = false` for `workspace = true`). Fixed in
   `deny.toml`, verified locally with cargo-deny 0.22.1 (all checks green) — details in
   doc 09.
2. **Actions minutes budget**: pushes and PRs no longer trigger any workflow (zero
   minutes). `ci.yml` runs only on `workflow_dispatch` (manual) and via `workflow_call`
   from `release.yml` (published/prereleased releases — the important stuff). The same
   gate runs locally: `cargo xtask check` (full) + pre-commit hook (fast subset).
   Branch protection updated accordingly: no required status-check contexts; PRs are
   gated by the PR-template checkbox "ran `cargo xtask check` locally" + 1 review.

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

### Round-4 addendum (owner clarifications, 2026-11-24 — rates mirror)

- The mirror repo now carries the **full BNR history 2005→2026** as `kind: "archive"`
  packages (`packages/bnr/archive-YYYY.json`, additive kind in spec @1) + daily packages;
  23 packages, manifest + SHA256SUMS, all JSON validated. `.gitattributes` marks packages
  binary so bytes stay stable for the checksums.
- The **scheduled** action (`update-rates.yml`, banking days ~after the 13:00 update)
  fetches ONLY `https://curs.bnr.ro/nbrfxrates.xml`; yearly archives are (re)built by the
  manual `backfill.yml`. Both ed25519-sign packages once `RATES_SIGNING_KEY` is set.

## Round-5 decisions (owner approved all, 2026-11-24 — M0 quality-of-life)

**Dev QoL:** `cargo xtask pr "<title>"` (branch+push+PR via gh — main is protected, PRs are
the path), `cargo xtask changelog "<line>"` (appends under Unreleased), `.vscode/settings.json`,
README badges (license + toolchain + CI-minutes note).

**App foundation QoL (specs + primary data now; UI at M3 via the frontend agent):**
- `data/settings-defaults.toml` — settings schema + defaults as SSOT primary data (theme,
  language, date/number format, first day of week, timezone, default currency, rates source
  priority, show_currency_code); `primary-data::settings` validates cross-refs
  (language↔locales, default_currency↔currencies, IANA tz via jiff).
- `data/load-policies.toml [boot_sequence]` — the canonical cold-start order
  (single-instance → panic hook/logging → registry → profile open+migrations → settings →
  theme → locale → window restore → accounts summary → last view → rates cache → job resume);
  boot never blocks on optional steps (RULE-UX-001), INV-LOAD-001 budget applies.
- doc 02 **F22 app chrome**: About screen (build-info + channel + GPL + third-party
  licenses + Open data folder + Copy diagnostic), shortcuts legend generated from
  `data/shortcuts.toml`, Settings bound to settings-defaults keys + Privacy section
  (doc 08), onboarding (F17 expanded: create profile → optional import → optional rates
  download; crash-recovery drafts offered after a crash).
- doc 05 **§5.14 window & session state**: `ui_state` table (window rect/display/last
  view/timeline scope/selection/drafts), single-instance IPC per ADR-0016 (named mutex on
  Windows, lockfile+socket elsewhere; second process forwards argv and exits).
- Locale seeds for the above screens added to en+ro (parity enforced by `govern errors`).

## M1 completed (2026-11-24) — storage skeleton

`crates/storage` (`bookkeepy-storage`, ADR-0005), scaffolded with `cargo xtask new-crate`:

- `conn`: one SQLite file per profile, WAL + foreign_keys + busy_timeout, `Mutex<Connection>`
  behind `Db::with_conn` (transactions explicit).
- `schema`: append-only migration list in code, applied in one transaction; `meta.schema_version`;
  too-new profiles refused without writing (RULE-DATA-004 / ADR-0022 → BK-DATA-0002); v1 tables:
  meta, schema_migrations, settings, ui_state, audit_log, outbox, jobs + audit immutability triggers.
- `outbox`: durable envelope store; `publish_pending` feeds the in-process `Bus` exactly-once
  after commit (ADR-0009 split is now real).
- `audit`: sha256 hash-chained append-only log; UPDATE/DELETE rejected by triggers; `verify`
  detects tampering (BK-STORE-0006).
- `backup`: local live backup (rusqlite backup API; source stays usable, fully offline) → `backups/`, verify (integrity_check + audit chain), restore.
- `lock`: single-instance `app.lock` (fs2), second launch → BK-STORE-0004 (app shell focuses window).
- `selfcheck`: startup report (integrity, audit chain, pending outbox, drafts) — never blocks.
- `kv`: `ui_state` + `settings` tables; settings resolve = user override over primary-data defaults.
- New BK codes BK-STORE-0003..0006 (+locales en/ro). Deps pinned (doc 07): rusqlite 0.43.0
  (bundled/blob/backup), sha2 0.11.1, hex 0.4.3, fs2 0.4.3.
- 8 new tests (migrations idempotent + downgrade refusal, outbox exactly-once, audit tamper,
  backup/restore roundtrip, lock, selfcheck). Full `cargo xtask check` green.

Findings: rusqlite `Backup::new` needs `&mut` dest and `run_to_completion` takes a progress
callback; `Connection::transaction` needs `&mut` (so `with_conn` hands out `&mut`);
file copies of a WAL DB must be preceded by `wal_checkpoint(TRUNCATE)`; restore must happen
with no open handle (caller holds the lock).

### M1 addendum (2026-11-24) — interconnection of M0/M1, terminology, new surfaces

Prompt: *"why 'online backup'? what else could M1 account for? make M0+M1 interconnected."*

- **Terminology (fintecheng/doc-05 aligned):** "online backup" was ambiguous — SQLite's
  "online backup API" only means the source DB stays usable during the copy. Bookkeepy's
  backups are **local live backups**: purely local, fully offline, no network (RULE-NET-001:
  the only network uses remain app updates + rates packages). All docs/code now say
  "local live backup".
- **`session::open_session` (composition root):** one call wires the whole M0+M1 stack —
  `Registry::load` → profile lookup → `ProfileRoots` → single-instance lock →
  `Db::open_profile` (pragmas incl. `wal_autocheckpoint=1000` + migrations) → logging +
  panic hook (once per process) → `jobs::resume_pending` → self-check → outbox replay →
  auto local live backup + prune (`KEEP_BACKUPS=10`) → `registry.set_active`. Returns
  `Session { data_dir, roots, profile, db, bus, report, _lock }`. Lock contention →
  BK-STORE-0004, app shell focuses the existing window (ADR-0016). Nothing recoverable ever
  blocks (RULE-UX-001): logging and auto-backup are best-effort, reported in
  `SelfCheckReport.backup_note`.
- **Atomic posting (`tx::post`):** outbox row + audit row in ONE transaction, commit, then
  publish to the bus; `outbox::enqueue` now takes `&Connection` (a `&Transaction` derefs to
  it) so it runs inside the posting transaction. Crash between commit and publish is healed
  by `publish_pending` at next startup — exactly-once at app level.
- **Durable jobs (`jobs`, RULE-JOB-001):** state machine
  queued→running→done|failed(re-queue until `max_attempts`)|paused|cancelled, with
  `resume_pending` crash-recovery driver; user-facing pause/resume/cancel + manual retry of
  `failed` jobs (surfaced, never blocking).
- **Settings SSOT enforcement:** `kv::settings_set` validates the key against
  `primary_data::settings::keys()` (from `data/settings-defaults.toml`); unknown key →
  new code **BK-DATA-0003** (+ en/ro locales).
- **i18n error text:** `bookkeepy_i18n::error_text(locale, code)` maps any BK-code through
  `data/errors.toml`'s `i18n_key` to localized text (unknown code returned as-is, copyable).
- New edges (govern deps): storage → primary-data, logging, diagnostics. New workspace path
  deps declared on first use: bookkeepy-logging, bookkeepy-diagnostics.
- Tests: 12 storage unit tests + `tests/session_flow.rs` end-to-end (open → post →
  crash-sim drop → reopen → audit verifies, outbox empty, ui_state persisted, backups
  pruned). Full `cargo xtask check` green (fmt, clippy -D warnings, tests, govern ×10,
  deny).
- Findings: `Result::unwrap_err` needs `T: Debug` (manual `Debug` for `Session`); global
  tracing subscriber can be set once per process (session guards logging with a
  `Mutex<Option<LogGuard>>` static); clippy `manual_range_contains` denies `a >= 1 && a <= n`.

### M1 hardening round (2026-11-24) — edge cases + M0↔M1 interconnection gaps closed

Prompt: *"what if we create a migration and want to revert it? what else could M1 improve —
edge cases? is M0↔M1 integration complete?"* — all items below approved and implemented.

**Q1 — reverting a migration/database.** Shipped data: NO down-migrations, ever
(INV-DATA-001/ADR-0022). Revert = a **compensating forward migration** appended to
`schema::MIGRATIONS` (append-only, one transaction each); proven by
`tests/migrations_flow.rs` (v2 add column → v3 drop column, user data intact, and the
older build still refuses the newer profile read-only with BK-DATA-0002). Dev-phase:
revert = delete the disposable dev profile via **`BOOKKEEPY_CHANNEL=dev cargo xtask
dev-reset-profile <id> --data-dir <path>`** — dev-channel-gated, path-escape-checked,
refuses while the instance lock is held; never touches shipped profiles (doc 09).

**Q2 — edge cases found by reading M1 code, now fixed/covered:**

1. `restore` left stale `app.db-wal`/`-shm` → old WAL frames would replay onto the restored
   DB. Now crash-safe: copy→temp, delete sidecars, atomic rename; `verify_backup` opens
   backups READ-ONLY; auto-backup names are collision-free (`auto-<ts>[-n].db`).
2. `audit::verify` was full-chain O(n) on every boot → `verify_incremental` with a `meta`
   checkpoint (trusted only if the checkpoint row still hashes correctly; full verify
   forced every 50 000 rows / on demand / for backups).
3. Self-check blind spots → report now also carries `migrations_match` (DB vs code list),
   `orphan_settings` (newer app's keys — ignored, never fatal), `wal_bytes` + warn
   threshold (hand-copied DB hazard), `boot_ms` vs `perf-budgets.toml` cold-start budget.
4. M0↔M1 drift → `load-policies.toml` boot steps now have machine `id`s;
   `session::BOOT_STEPS` is unit-tested to be a subsequence of them; session also resolves
   the full settings snapshot at boot (`kv::settings_snapshot` over the SSOT
   `settings::default_value`s).
5. `Bus::publish` now isolates panicking subscribers (`catch_unwind` + tracing) — one bad
   subscriber can't break `tx::post`.
6. Privacy on shared machines: `app.db`, backups, `profiles.toml` chmod 0600 on unix.

M0↔M1 integration is now complete: registry, lock, logging, panic hook, settings SSOT,
i18n `error_text`, bus, load-policies boot order, perf-budgets boot timing, build-info
channel (dev tool) — all wired into `session::open_session`. Remaining M0 item by design:
diagnostics bundle attachment on fatal errors (M2, app shell).

Tests: 15 storage unit + 2 integration (`session_flow`, `migrations_flow`), bus panic
isolation, boot-subsequence invariant, primary-data keys↔defaults sync. Full
`cargo xtask check` green (fmt, clippy -D warnings, tests, govern ×10, deny).

## M2 completed (2026-11-24) — ledger engine + shared action engines

`crates/shared` (`bookkeepy-shared`, RULE-SHARED-001) and `crates/ledger`
(`bookkeepy-ledger`, ADR-0009/0010), plus storage migration **v2** (`MAX_KNOWN=2`):

- storage v2 tables: `accounts`, `journal_entries`, `postings`, `transactions`,
  `categories`, `tags`, `parties`, `tx_tags` + DB triggers: entries/postings reject
  UPDATE/DELETE; `transactions` allows ONLY `status` updates. `tx::post_with` runs a side
  effect + outbox + audit in ONE transaction, then publishes. `StorageError::Foreign`
  carries engine errors across closure boundaries without losing their BK-code.
- `shared::actions`: `rename` / `archive` / `restore` / `change_status`, generic over
  `Actionable` with a table allowlist; each has an `_in_tx` variant for composing engines.
  Every action = column change + audit + outbox atomically, then bus event.
- `ledger::accounts`: typed accounts (asset/liability/equity/income/expense, normal-balance
  semantics), currency validated against `currencies.toml`; rename/archive via shared.
- `ledger::catalog`: categories (each backed by a hidden system account so transactions
  balance), tags, parties; archive = soft delete.
- `ledger::entries`: `post` (validates positive amounts, known/non-archived accounts,
  currency match, per-currency balance — INV-LEDGER-001), `reverse` (linked reversal,
  keeps original value_ts), per-table SHA-256 chains + `verify_chains`.
- `ledger::transactions`: income/expense/transfer compile to balanced postings; statuses
  pending/cleared/reconciled/voided with validated transitions; `voided` posts the linked
  reversal in the same transaction; `replace` = reversal + replacement (`replaces` chain).
- `ledger::balances`: derived balances (signed per normal balance, `as_of` on value_ts),
  trial balance per currency, `assert_trial_balance`.
- `ledger::lifecycle` (ADR-0024, RULE-LIFE-001 — the soft-delete SSOT): archived catalog
  entities are excluded from pickers/forward projections but kept in history; `voided`
  transactions are excluded from computed money metrics VIA THE JOURNAL (linked reversal),
  never by filtering posted rows; non-posted items are excluded by status directly.
- New BK codes BK-LEDGER-0001..0008 + BK-ACTION-0001 (+ en/ro locales). New edges:
  storage ← shared ← ledger (govern deps PASS).
- Tests: unit (accounts/catalog/entries/transactions/balances/shared actions) +
  `tests/properties.rs` proptest: trial balance holds after EVERY op (any prefix),
  reversal restores exact prior balances, both chains verify after every op.
- Docs: ADR-0024, doc 04 §4.10 (RULE-LIFE-001), doc 05 §5.4 rewritten to the implemented
  reality, doc 03 crate tree/direction, code-map. Full `cargo xtask check` green.
