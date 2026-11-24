# 10 — Process and roadmap

## How we work (long-term collaboration)

1. **You prompt a surface-level wish** ("let me attach receipts to transactions").
2. **I interpret it as an engineer**: which feature domain (doc 02), which crate owns it
   (doc 03), which rules apply (doc 04), which fintech principles are touched (doc 06 —
   quoted), what tests prove it (doc 09).
3. If the change affects architecture, rules or invariants → an **ADR first**
   (`docs/adr/`), then an issue describing the change, then a PR implementing it with
   tests + docs + CHANGELOG.
4. Nothing merges without CI green and a review. Nothing ships to users except via a
   tagged GitHub Release.

This loop is what keeps additions, refactors, subtractions and deprecations easy over
years, not months.

## Roadmap (milestones)

| M | Name | Outcome |
|---|---|---|
| M0 | Repo & foundations | This repo: docs, ADRs, primary data, skills, CI skeleton, workspace manifest, `core` (Money/Currency) + `primary-data` crates with tests. **Current phase.** |
| M1 | Storage skeleton | `storage` crate: connection, migrations, WAL, backup/restore/verify. |
| M2 | Ledger engine | double-entry postings, derived balances, reversals, hash chain, audit log; property tests "books balance". |
| M3 | Accounts & transactions UI | first gpui-kit screens (all styled components), app shell, settings, EN/RO. |
| M4 | Import/export | CSV/OFX/QIF pipeline with mapping, dedup, reconciliation breaks; CSV/JSON export. |
| M5 | Search | tantivy engine, write-through + rebuild, search UI. |
| M6 | Budgets & reports | envelopes, cashflow/net-worth reports, golden tests, PDF/CSV export. |
| M7 | Multi-currency, FX & live rates | per-account currency, base-currency valuation, rounding account; **rates mirror sub-repo automation (BNR first) + app-side `engine_rates`** (download/verify/import packages, manual rates, defaults, rates UI F14) — docs/11. |
| M8 | Documents & backups UI | attachments, snapshots, verify, open data folder. |
| M9 | Security | app lock + optional at-rest encryption, privacy statement, integrity check UI. |
| M10 | Updater & packaging | GitHub Releases updater with checksum verification; installers; first `v0.1.0` release. |
| M11 | Polish | i18n completeness, theming, performance (gpui-fast), accessibility, docs site. |

## Parked feature ideas (not now — logic first)

Approved-in-principle but **deferred until the owner specifies the user-facing feature in a
future prompt**: auto-categorization rules on import (2), command palette + global quick-add
(6), multi-book switcher (8), user-defined report templates (9). Until then, only the
underlying *logic* patterns (rules as auditable artifacts, palette as a UI widget family,
books as independent files, templates as user-owned data) are in scope.

## Open brainstorming topics (what is still to be discussed)

1. Single-instance & file locking: two app windows on one book; behavior + message.
2. Crash recovery: WAL + tx guarantees, self-check on open, draft recovery UX.
3. Uninstall behavior: never touch the data folder; offer full export on uninstall.
4. First-run onboarding, empty states, optional sample book.
5. Window & multi-display management (geometry per monitor from `system_context`).
6. Local reminders/notifications engine (subscriptions, budget thresholds, backup due).
7. Scale: materialized aggregations for 100k+ transactions, refreshed via events.
8. PDF/print pipeline for reports.
9. Accessibility & keyboard-first: focus order, screen-reader labels, shortcuts map.
10. Fiscal periods & custom reporting date ranges; locale-aware number/date input parsing.
11. Security threat model (stolen device): app-lock timeout, lockout, encryption scope.
12. Export schema specs (versioned CSV/JSON formats as docs — SSOT).
13. Search query language and saved filters.

## Docs hygiene (point 16)

Docs follow the same rules as code: modular files, one canonical place per topic
(RULE-DOC-001), cross-links instead of duplication. ADRs only for real decisions — routine
changes go to the CHANGELOG, not to new ADRs.

## Definition of done (per milestone)

CI green on all platforms · property/golden tests for money behavior · docs updated
(features, conventions, ADRs if needed) · CHANGELOG entry · user-facing strings localized
(EN/RO) · no network code outside `updater` (governor) · README/docs index still accurate.
