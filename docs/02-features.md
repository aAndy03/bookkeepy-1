# 02 — Feature inventory (user-facing)

Features are grouped by domain. Each feature = one **horizontal slice** (a crate) composed
of recursive engines (an engine is itself composed of parts that can be engines). Crucially,
**generic actions are shared engines, implemented once**: rename, archive/restore, duplicate,
delete-as-reversal, tag, attach, note, change-status, bulk-apply… live in the shared crate
and are *localized* per feature through thin variant adapters — never re-implemented per
feature (RULE-SHARED-001, doc 04 §4.2). "User sees" = surface; "Engine does" = hidden
machinery (naming the shared engine it uses).

## F1 — Accounts
- **User sees:** accounts (cash, bank, card, wallet, loan…): name, type (asset/liability),
  currency, opening balance, current balance, color/icon; create/rename/archive (never
  hard-delete an account with history).
- **Engine does:** account types drive debit/credit normal balance; balances derived from
  postings, never stored; rename/archive go through the **shared action engines**
  (`shared::actions::rename` / `archive`), localized here (RULE-SHARED-001).

## F2 — Transactions
- **User sees:** add/edit income, expense, transfer; splits; categories, tags, payee, note,
  attachments; value date vs entry date; status (pending/cleared/reconciled); recurring &
  scheduled transactions.
- **Engine does:** user transactions compile into balanced double-entry postings (splits/fees
  = extra movements); posted entries immutable — edits post linked reversals; scheduled
  transactions materialize via durable resumable pipelines.

## F3 — Import / export
- **User sees:** import bank files (CSV, OFX/QFX, QIF) with column mapping + preview; duplicates
  detected/skipped; export ledger & reports to CSV/JSON; full backup/restore.
- **Engine does:** pipeline parse → map → validate → dedup by explicit idempotency keys →
  post → index; re-import never double-posts; reconciliation breaks surfaced, never
  overwritten.

## F4 — Budgets & planning
- **User sees:** envelope budgets per category/period; planned vs actual; rollover; forecasts.
- **Engine does:** budgets are projections over the ledger, computed at value date.

## F5 — Reports
- **User sees:** cashflow, spending by category/tag/party, net worth over time, income vs
  expense, per-account statements, period comparison; print/PDF/CSV export.
- **Engine does:** deterministic projections of the journal, golden-tested; value-date rates
  with recorded source.

## F6 — Multi-currency & FX (live via mirror + manual)
- **User sees:** per-account currency from ISO 4217 primary data; base currency for totals;
  a **rates browser** showing all downloaded sources side by side (date + source badge),
  primary-source picker, manual rate entry, "adjust this live rate" (manual override),
  "Download rates" button, last-sync status; net worth in base currency; a clear note that
  deleted manual rates cannot be recovered (default chain applies).
- **Engine does:** rates come **only** from the `bookkeepy-1-rates` mirror repo as immutable
  checksummed packages (doc 11); app never contacts sources; live rows immutable; manual
  rows `origin=manual`, deletable-irreversible, defaults from `data/defaults.toml`; switching
  primary source never rewrites history; sources accumulate; conversion at the chosen rate,
  full precision, round once at the boundary, residual → rounding account.

## F7 — Search
- **User sees:** instant full-text search over payees, notes, categories, tags, amounts,
  dates; saved searches.
- **Engine does:** 100% tantivy; index = derived cache, write-through, rebuildable.

## F8 — Documents & attachments
- **User sees:** attach receipts/invoices to transactions; view inline.
- **Engine does:** files in the user's data folder; optional encryption; PII separable for
  erasure.

## F9 — Security & privacy
- **User sees (now):** privacy statement; Settings → Privacy → "Data we collect" (every
  locally collected item listed, viewable, clearable); first-run disclosure dialog; "open
  data folder"; integrity check (hash-chain verify); uninstall keeps data (INV-APP-001).
- **Parked (future feature, owner will specify):** app lock / passphrase, at-rest encryption.
  Threat model is documented (doc 08); for now the data file sits in the user-profile data
  dir with OS permissions, no secrets are stored, and the hash chain detects tampering.
- **Engine does:** no transmission of personal data ever (doc 08); audit log records
  what/when/who/why and is never app-editable (ADR-0014).

## F10 — Backups & data management
- **User sees:** one-click snapshot backup, restore, verify, data-folder location; retention
  policy; the file is portable.
- **Engine does:** `VACUUM INTO` + verify; hash-chain tamper evidence; seamless across
  versions (INV-DATA-001).

## F11 — Settings & customization
- **User sees:** theme (gpui-kit + theme tokens), locale (EN/RO), date & number formats, first
  day of week, base currency, primary rates source, custom categories/tags, update & rates
  sync toggles.
- **Engine does:** settings in DB + local config; primary data from `data/*.toml`.

## F12 — Updates
- **User sees:** "Check for updates", release notes, download & install from GitHub Releases
  after explicit approval, SHA-256 verified.
- **Engine does:** `updater` crate — one of only two HTTP-allowed crates (RULE-NET-001).

## F13 — Help & diagnostics
- **User sees:** about (version/build/license), log viewer, DB integrity + hash-chain verify,
  open data folder, reset app settings (never user data), cold-start performance info.
- **Engine does:** read-only diagnostics; fatal errors surface with one-click "Copy
  diagnostic" (BK-code + localized message + context + version) for GitHub issues.

## F14 — Rates manager (part of F6, called out)
- Browser of all downloaded rate packages per source/currency/date; choose rate per
  conversion/report; manual adjustments; re-download; delete-manual warning; offline note.
- **Engine does:** `engine_rates` + `rates_packages`/`fx_rates` tables (doc 05 §5.3).

## F15 — Performance & loading behavior (invisible but felt)
- **User sees:** the app opens fast and lets them add a transaction immediately; lists scroll
  smoothly; heavy things load when needed; stays fast with 100k+ transactions.
- **Engine does:** `engine_loadpolicy` + `data/load-policies.toml` (hot/warm/cold, prefetch
  on idle, INV-LOAD-001 cold-start budget); materialized aggregations refreshed by events;
  local-only usage stats to tune prefetch.

## F16 — Reminders & local notifications
- **User sees:** gentle, dismissible/snoozable reminders: subscription due, budget threshold
  reached, backup due, rates stale. Never blocks anything (RULE-UX-001).
- **Engine does:** `reminders` crate — rule set over ledger/budget/jobs data → in-app + OS
  notifications; state in `reminder_rules`/`reminder_state`.

## F17 — First-run onboarding & empty states
- **User sees:** welcome → create or open a book → first account → optional import; optional
  sample book; privacy disclosure at setup (doc 08); every empty view has a clear next step.
- **Engine does:** single-instance app lifecycle (ADR-0016), `ui_state` for continuity.

## F18 — Profiles (fully isolated)
- **User sees:** create/rename/delete profiles, switch at any time from the app shell; each
  profile has all features (F1–F20); profiles share **no data and no database** — the only
  bridge is export (whole book or a scoped selection) → import into the other profile.
- **Engine does:** a profile is an isolated data root (`profiles/<id>/{book.db, attachments/,
  rates/packages/, settings, ui_state, jobs, backups/}`); the app keeps only a tiny registry
  of profile id/name/path (app-level metadata, not user data); switching closes the current
  profile (jobs pause/resume per profile DB); import into another profile assigns fresh ids
  via the import pipeline (ADR-0018).

## F19 — Timeline & centralized filters (the temporal spine)
- **User sees:** the **timeline** is the main component tying every screen together: views by
  day / week / month / trimester / year (+ custom range); navigate past ↔ present ↔ future;
  focus a day or a period; timeline-level filters (accounts, categories, tags, parties,
  text); **compare mode** (day vs period, period vs period). Every timeline-bound component
  on enabled screens shows data for the focused scope. Past = backdated entries; future =
  scheduled/recurring forecast entries (toggleable).
- **Engine does:** `engine_timeline` is the **single source of truth for time scope**
  (RULE-TL-001, ADR-0019): one `TimeScope` state (view, focus range, compare scope, filters,
  include-future), persisted in `ui_state`, broadcast as an event; presenters subscribe and
  parameterize queries — no component owns its own period state (that's what prevents the
  "one stuck at 60 days, another at 30" drift). Compare mode exposes a `compare` capability
  trait: components render comparison variants and run queries twice (diff) when it's on.

## F20 — Bulk selection & bulk actions
- **User sees:** in tables (the default data presentation): checkbox column, select-all,
  shift-range selection; a bulk action bar appears with applicable shared actions (categorize,
  tag, move account, change status, archive, delete-as-reversal, export selection…).
- **Engine does:** selection state lives in `ui_state` (non-audited, clearable); bulk actions
  are batch API operations executed as **one job** with progress (pause/resume/cancel,
  RULE-JOB-001), each affected item audited individually; the actions themselves are the
  shared engines (RULE-SHARED-001).
