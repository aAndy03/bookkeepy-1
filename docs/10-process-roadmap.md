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

## Docs hygiene (point 16)

Docs follow the same rules as code: modular files, one canonical place per topic
(RULE-DOC-001), cross-links instead of duplication. ADRs only for real decisions — routine
changes go to the CHANGELOG, not to new ADRs.

## Definition of done (per milestone)

CI green on all platforms · property/golden tests for money behavior · docs updated
(features, conventions, ADRs if needed) · CHANGELOG entry · user-facing strings localized
(EN/RO) · no network code outside `updater` (governor) · README/docs index still accurate.
