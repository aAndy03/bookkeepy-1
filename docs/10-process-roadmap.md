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
| M0 | Repo & foundations | workspace, `core` (Money/Currency), `primary-data`, CI skeleton, governors. **Current.** |
| M1 | Storage skeleton | `storage`: connection, migrations, WAL, backup/restore/verify, single-instance lock, crash-recovery self-check. |
| M2 | Ledger engine | double-entry postings, derived balances, reversals, hash chain, audit log; property tests. |
| M3 | Accounts & transactions UI | gpui-kit shell, first screens, ui-state, onboarding/empty states, EN/RO. |
| M4 | Import/export | CSV/OFX/QIF pipeline, dedup, reconciliation breaks; export package format v1. |
| M5 | Search | tantivy engine + query DSL, write-through + rebuild. |
| M6 | Budgets & reports | envelopes, reports, golden tests, PDF/print. |
| M7 | Multi-currency, FX & live rates | rates mirror automation (BNR first) + `engine_rates` + rates UI (doc 11). |
| M8 | Documents, backups UI, reminders | attachments, snapshots, retention reminders, local notifications. |
| M9 | Scale & polish | materialized aggregations, a11y/keyboard pass, multi-display window mgmt, perf budgets green. |
| M10 | Updater & packaging | updater, installers/uninstallers (INV-APP-001), first `v0.1.0` release. |
| M11 | Future security | app lock / at-rest encryption (parked, see below). |

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

Auto-categorization rules on import · command palette + global quick-add · multi-book
switcher · user-defined report templates · **app lock / at-rest encryption**.

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
