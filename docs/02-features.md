# 02 — Feature inventory (user-facing)

Features are grouped by domain. Each domain maps to one crate and one "engine" under the
conventions of doc 04. "User sees" describes the surface; "Engine does" describes the
hidden machinery the prompt must software-engineer.

## F1 — Accounts
- **User sees:** a list of accounts (cash, bank, card, wallet, loan, investment…), each
  with a name, type (asset/liability), currency, opening balance, current balance, and a
  color/icon. Create, rename, archive (never hard-delete an account with history).
- **Engine does:** account types drive debit/credit normal balance; balances are
  *derived* from ledger postings, never stored (doc 06); archiving keeps history intact.

## F2 — Transactions
- **User sees:** add/edit income, expense, transfer between accounts; split
  transactions; categories and tags; payee (party); note; attachments; value date vs
  entry date; status (pending / cleared / reconciled); recurring & scheduled
  transactions.
- **Engine does:** every user transaction compiles into balanced double-entry postings
  (fees and splits become extra movements — fintecheng: "one transaction, many
  movements"); posted entries are immutable, edits post linked reversals/corrections;
  scheduled transactions materialize through a durable, resumable pipeline (doc 06).

## F3 — Import / export
- **User sees:** import bank files (CSV, OFX/QFX, QIF) with a column-mapping step and a
  preview; duplicates are detected and skipped; export the ledger and any report to
  CSV/JSON; full-file backup/restore.
- **Engine does:** import pipeline = parse → map → validate → dedup by import idempotency
  keys → post to ledger → index; re-running an import never double-posts (fintecheng
  idempotency); export is a pure projection of the ledger.

## F4 — Budgets & planning
- **User sees:** envelope budgets per category per period (month/quarter/year/custom);
  planned vs actual bars; rollover rules; simple forecasts.
- **Engine does:** budgets are projections over the ledger, not separate money; actuals
  computed at value date with the account's currency, converted at stored rates.

## F5 — Reports
- **User sees:** cashflow, spending by category/tag/party, net worth over time,
  income vs expense, per-account statements, period comparison; print/PDF export.
- **Engine does:** every report is a deterministic projection of the journal, golden-tested
  (doc 09); reports group by value date and convert currencies at the rate *of that date*
  with its source recorded (fintecheng FX).

## F6 — Multi-currency & FX
- **User sees:** each account has a currency from the ISO 4217 primary data
  (`data/currencies.toml`); a base currency for totals; a rate table the user enters
  manually (rate, date, source note); net worth in base currency.
- **Engine does:** no cross-currency arithmetic without an explicit stored rate;
  rates are directional and time-stamped; **no live rate fetching** — offline adaptation
  of fintecheng FX (doc 06). Conversion keeps full precision and rounds once, at the
  boundary, with the residual booked to an explicit rounding account.

## F7 — Search
- **User sees:** instant full-text search over payees, notes, categories, amounts and
  dates; saved searches.
- **Engine does:** 100% tantivy (ADR-0006). The index is derived state: built
  write-through from ledger events and fully rebuildable from the SQLite file, so it can
  never become a second source of truth. No custom indexing engine is ever written.

## F8 — Documents & attachments
- **User sees:** attach receipts/invoices (PDF, images) to transactions; view them inline.
- **Engine does:** files stored in the user's data folder, referenced by the DB; optional
  encryption at rest; PII separation so erasure can delete attachments without rewriting
  the ledger (fintecheng GDPR section, doc 06).

## F9 — Security & privacy
- **User sees:** optional app lock with a passphrase; privacy statement; "open data
  folder"; integrity check.
- **Engine does:** no telemetry, no network except updater (doc 08); optional whole-file
  encryption (passphrase-derived key, AEAD) or field-level encryption with
  crypto-shredding for erasure; audit log records what/when/who/why for every change.

## F10 — Backups & data management
- **User sees:** one-click snapshot backup (timestamped copy), restore, verify, and a
  "data folder" location display; the file can be moved/copied freely.
- **Engine does:** SQLite `VACUUM INTO` snapshots + verification on restore; the data
  file is plain and portable — Bookkeepy never locks it exclusively; hash-chain tamper
  evidence over the journal (doc 05).

## F11 — Settings & customization
- **User sees:** theme (gpui-kit theming), locale (EN/RO first), date & number formats,
  first day of week, base currency, custom categories/tags, primary-data overrides.
- **Engine does:** settings stored in the DB + a local config file; primary data
  (currencies etc.) comes from `data/*.toml` compiled into the app (ADR-0008).

## F12 — Updates
- **User sees:** "Check for updates"; release notes; download & install from GitHub
  Releases only after explicit approval.
- **Engine does:** the *only* crate allowed to do HTTP (RULE-NET-001); verifies SHA-256
  checksums of downloaded artifacts; never auto-installs (doc 08, ADR-0003).

## F13 — Help & diagnostics
- **User sees:** about dialog (version, build, license), log viewer, DB integrity check,
  open data folder, reset app settings (never user data).
- **Engine does:** diagnostics are read-only over the data file; logs never contain
  amounts/payees unless the user explicitly enables verbose diagnostics locally.
