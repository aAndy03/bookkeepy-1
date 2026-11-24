# 05 — Data, money and storage

Quoted fintech principles behind this doc: [06-fintech-principles.md](06-fintech-principles.md)
(source: `references/fintecheng.txt`).

## 5.1 Money model (ADR-0004)

`Money = { amount_minor: i64, currency: CurrencyCode }` — minor units per currency from ISO
4217 primary data ("don't assume it's always 2, it's not!"); no floats; serialize as string
or minor-unit integer, never a bare JSON number; rounding explicit, once, at boundaries, with
an explicit **rounding account** for residuals ("Rounding breaks sums… e.g. an explicit
rounding account").

## 5.2 Storage (ADR-0005)

- One SQLite file per user book via `rusqlite` (`bundled`), WAL, foreign keys ON, background
  thread (RULE-ASYNC-001). User-chosen folder; never locked exclusively.
- Migrations: numbered SQL files, applied in a transaction, recorded in `user_version` +
  `schema_migrations`. Every migration tested up *and* down on fixtures.
- Backups: `VACUUM INTO` snapshots + restore verification (`quick_check` + hash chain).

### Seamless, long-lived data (points 14–15, INV-DATA-001)

- **Seamless is a principle**: a user on any version can install any later version and the
  data works — 1, 5, 10, 20 years of updates. Enforced by:
  - migrations are **forward-only, additive, never destructive** (no dropped columns/data
    without a superseding ADR + export path);
  - the data-file format is versioned; every release ships an **upgrade-path CI test** that
    opens fixtures produced by every previous tagged release and verifies them (doc 09);
  - backward-compat corpus in `tests/fixtures/` (fintecheng: "today's code must still read
    what old code wrote");
  - primary data upgrades (e.g. new currencies) merge, never remove codes in use;
  - rates packages are additive by design (doc 11);
  - settings/config migrate with defaults; UI state never blocks data access.
- INV-DATA-001: *for any release N and any later release M, a data file written by N opens
  unaltered in M.* Property/CI-enforced via the fixture corpus.

### Schema overview

| Table | Owner | Notes |
|---|---|---|
| `accounts` | ledger | type asset/liability, currency; balance **derived**, never stored |
| `journal_entries` + `postings` | ledger | double-entry, append-only, hash chain (5.4/5.5) |
| `transactions` | transactions | user-level compilation; status, value date, schedule |
| `categories`, `tags`, `tx_tags`, `parties` | catalog | user-editable; presets from primary data |
| `attachments` | documents | path + hash; PII store, separately erasable |
| `budgets`, `budget_lines` | budget | |
| `fx_rates` | rates | see 5.3: origin live/manual/adjusted, source, package id, date, rate (string), hash |
| `rates_packages` | rates | installed packages (id, source, sha256, kind, file path) — immutable once installed |
| `import_batches`, `import_ids` | import | idempotency keys |
| `audit_log` | audit | what/when/who/why — **append-only, hash-chained, never editable/deletable via the app** (RULE-AUD-001, ADR-0014) |
| `app_history` | ui | UI-level action tracking; retention user-configured (keep / N days / on exit / off); clearable; never money truth |
| `jobs` | jobs | durable job state machines (RULE-JOB-001): state, progress, attempts, payload, result, BK error code |
| `ui_state` | ui | session/interface state (namespace.key → JSON), SSOT schema in the ui-state module (RULE-STATE-001) |
| `usage_stats` | loadpolicy | local-only prefetch tuning; visible+deletable in Settings→Privacy |
| `system_context` | diagnostics | local machine context snapshot (doc 08); local-only |
| `reminder_rules`, `reminder_state` | reminders | local reminders (dismissed/snoozed state) |
| `agg_*` (materialized) | reporting/storage | aggregations refreshed via events; recompute fallback |
| `settings`, `schema_migrations` | settings/storage | |

## 5.3 FX rates — live via the mirror sub-repo + manual (points 2, 2.1–2.4; ADR-0011/0012)

Architecture and package format: [11-rates-mirror-and-live-data.md](11-rates-mirror-and-live-data.md)
+ `bookkeepy-1-rates/docs/package-format.md`. Semantics here:

- **The app never contacts rate sources.** It downloads immutable, checksummed packages
  **only from `aAndy03/bookkeepy-1-rates`** (ADR-0011). Live rates = the latest publication
  each source pushes to the mirror. The app always works offline with downloaded data.
- **2.1 linking:** every rate's currency code must exist in `data/currencies.toml`
  (RULE-DATA-002); packages are validated against it on import.
- **2.3 source switching:** each `fx_rates` row stores its source + package id + rate as
  published. Changing the user's *primary source* never rewrites historical rows — they were
  already computed with their source's rate; only a manual update changes them.
- **2.4 accumulation:** packages from different sources add together; the UI shows all
  sources' rates and lets the user choose which rate a given conversion/report uses.
- **Manual rates & adjustments:** the user can enter a rate or adjust a live one
  (`origin = "manual"`). Manual rows are user data: **deleting them is irreversible** — the
  UI warns, and the fallback from `data/defaults.toml` applies (default chain: latest live
  rate of the primary source ≤ value date → latest live rate of any source ≤ value date →
  nearest earlier live rate → error `BK-FX-0002` "no rate available").
- fintecheng FX (quoted, doc 06 §6.3): rates are directional and time-stamped; transactional
  vs reference rate; "the source should be part of the data" — we store source + package id
  + fetched_at with every row.
- Rates are stored as decimal **strings** in packages and as integer minor-unit ratios
  internally where possible; conversion keeps full precision, rounds once at the boundary,
  residual → rounding account.

## 5.4 The ledger (ADR-0009/0010)

- Double-entry: "money is only moved, never created or destroyed"; external world gets
  dedicated accounts; balances derived, never stored; one transaction → many postings.
- Posted entries **immutable**: append-only tables, no mutating API, SHA-256 hash chain
  (verify via `engine_audit::verify_chain()`); corrections = linked reversals/corrections,
  fix-forward; value date + booking date both stored; reports use value date.
- Overdrafts representable, never clamped ("clamping… mints money"); warn at entry, detect
  post-factum.
- Audit trail: what / when / who-or-what (user, import batch, scheduler, updater, rates
  sync) / why (reference). The audit is distinct from `app_history` (UI tracking with
  user-set retention) — RULE-AUD-001 / ADR-0014: audit events are immutable; undo/redo never
  touches anything posted (RULE-UNDO-001); editing a posted transaction posts a linked
  correction event.

## 5.5 Integrity & tamper evidence

`postings.prev_hash`/`hash` chain (SHA-256); `quick_check` on open; both verifiable from
Help → Diagnostics.

## 5.6 Imports & idempotency

Explicit idempotency keys scoped to (import batch, source row); re-import posts nothing
twice; reconciliation against imported statements by persisted external id; discrepancies
are first-class "breaks", fixed by correction entries, never by overwriting.

## 5.7 Search index & query DSL (ADR-0006)

Tantivy, 100% of its API; index dir next to the DB; write-through from events; rebuildable
from the ledger — a cache, never a source of truth. The user-facing **query DSL** (small,
documented, versioned): field filters (`payee:…`, `category:…`, `tag:…`), amount ranges
(`amount:100..500`), date ranges (`date:2026-01-01..2026-01-31`), boolean `AND/OR/NOT`,
plain full-text otherwise — parsed by a dedicated parser module into a tantivy query; invalid
queries produce a BK-coded, localized error, never a panic.

## 5.8 Encryption at rest — PARKED (future feature)

The owner parked app lock / passphrase and at-rest encryption (doc 10). Current posture:
plain user-owned SQLite file in the user-profile data dir with OS file permissions; no
secrets stored; hash-chain tamper evidence (5.5). When un-parked: optional app lock
(Argon2id → AEAD), whole-file or field-level with per-record keys for PII → erasure = key
deletion ("crypto-shredding"); attachments/PII live in the mutable store, separate from the
immutable journal.

## 5.11 Export package format (versioned spec — portability forever)

Full export = one folder/zip `bookkeepy-export-v1/`: `manifest.json` (book id, exported_at,
app version, file list + SHA-256 each), `book.db` (SQLite copy), `attachments/`,
`settings.json`, `rates/` (installed packages). CSV/JSON per-report exports carry their own
versioned schema docs. The spec is the SSOT here; the app imports its own exports back
(round-trip tested, doc 09). Offered in-app (backup/export UI) and in the uninstaller with the
"data stays on this device; reinstalling finds it unaltered; this export works in other apps"
explanation (INV-APP-001).

## 5.12 Periods & fiscal calendar

Settings define fiscal-year start and week start; reports/filters accept calendar or fiscal
periods and custom date ranges; all period math is timezone-aware (jiff) and DST-safe
(edge register, doc 10).

## 5.9 Loading & caching

Demand-driven loading per `data/load-policies.toml` + `engine_loadpolicy`; hot (memory) /
warm (SQLite+tantivy) / cold (files, packages); prefetch on idle; INV-LOAD-001 cold-start
budget in `data/perf-budgets.toml`. Details: doc 03 §"Loading, caching, prefetch".
