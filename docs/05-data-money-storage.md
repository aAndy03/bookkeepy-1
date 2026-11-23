# 05 — Data, money and storage

Everything here follows the quoted principles in [06-fintech-principles.md](06-fintech-principles.md)
(source: `references/fintecheng.txt`).

## 5.1 Money model (ADR-0004)

```text
Money = { amount_minor: i64, currency: CurrencyCode }   // CurrencyCode validated against data/currencies.toml
```

- Minor units per currency come from ISO 4217 primary data — *"don't assume it's always 2,
  it's not!"* (fintecheng, Precision handling). `data/currencies.toml` carries `minor_units`
  per currency (e.g. JPY = 0, BHD = 3).
- No cross-currency arithmetic; conversion only via an explicit stored rate (5.3).
- Rounding (fintecheng, Rounding strategies): explicit, a product decision, done as seldom
  as possible and at boundaries (persist/display); **residuals are tracked, never dropped** —
  conversion rounding differences post to an explicit *rounding account* in the ledger.
- Serialization: amounts cross any boundary as string or integer minor units, never a bare
  JSON number.

## 5.2 Storage (ADR-0005)

- One SQLite file per user book, via `rusqlite` with `bundled`. WAL mode, `foreign_keys=ON`,
  `busy_timeout`, `journal_mode=WAL`. The app never holds the file exclusively; the user
  can copy it anytime.
- Location: OS app-data dir by default (e.g. `%APPDATA%/Bookkeepy 1/` on Windows), user
  changeable; "Open data folder" in Help shows it.
- Migrations: numbered SQL files in `crates/storage/migrations/`, applied inside a
  transaction, recorded in `PRAGMA user_version` + a `schema_migrations` table. Every
  migration is tested up *and* down against fixture DBs (doc 09).
- Backups: `VACUUM INTO 'backup-<ts>.db'` snapshots + restore with verification
  (`PRAGMA integrity_check` + hash-chain verify, 5.5). Export/import of the whole book as
  JSON/CSV is a pure projection (F3).

### Schema overview (tables; owned by the crate named)

| Table | Owner | Notes |
|---|---|---|
| `accounts` | ledger | type asset/liability, currency, archived flag; balance **derived**, never stored |
| `journal_entries` + `postings` | ledger | double-entry, append-only (5.4); hash chain |
| `transactions` | transactions | user-level view/compilation of entries; status, value date, schedule |
| `categories`, `tags`, `tx_tags`, `parties` | catalog | user-editable; presets from primary data |
| `attachments` | documents | file path + hash + linked entry |
| `budgets`, `budget_lines` | budget | envelopes per category/period |
| `fx_rates` | fx | rate, from/to, date, source note (5.3) |
| `import_batches`, `import_ids` | import | idempotency keys for dedup (5.6) |
| `audit_log` | audit | what/when/who/why for every change |
| `settings`, `schema_migrations` | settings/storage | app config + migration bookkeeping |

## 5.3 FX (offline adaptation)

fintecheng FX rates, adapted: a rate is always directional; its time is critical; there are
transactional rates (fall out of original+result amounts) and reference rates (valuation).
**Bookkeepy never fetches live rates.** The user enters rates manually (or imports them in a
CSV) with date + source; every conversion stores which rate was used. Valuation in the base
currency uses the latest stored rate on or before the value date. Quotes and adaptation in
doc 06 §FX.

## 5.4 The ledger (ADR-0009, ADR-0010)

- Double-entry (fintecheng): every entry moves the same amount out of one account into
  another — "money is only moved, never created or destroyed". External world gets dedicated
  accounts (e.g. "Income:Salary", "Expense:Groceries"), so money entering/leaving is tracked.
- "Balance is never stored. It's derived from the movements of money." Balances, reports,
  budgets are projections over postings; they may be cached/snapshotted for speed.
- Posting = commit. Posted entries are **immutable**: append-only tables, UPDATE/DELETE
  revoked at the DB permission level where possible, no mutating API in the app layer, plus
  tamper evidence — a hash chain (SHA-256) over journal rows, verifiable from Help
  (fintecheng, Immutability toolbox: "by construction / runtime checks / post-factum").
- Corrections fix forward: a **reversal** negates the original and links both ways; a
  **correction** books the difference. Value date vs booking date are both recorded
  (fintecheng, Value/booking/settlement time): value date = when it happened, booking =
  when recorded; reports use value date.
- Overdrafts: negative balances are *representable* (never clamped to zero — "clamping a
  negative balance to zero mints money"); policy warnings at entry time, detection post-factum.
- Audit trail (fintecheng): for every change record **what** happened, **when**, **who/what
  triggered it** (user / import batch / scheduler / system), **why** (reference to the
  instruction/batch). Configuration changes are audited too.

## 5.5 Integrity & tamper evidence

- `postings` carry `prev_hash`/`hash` forming a chain; `engine_audit::verify_chain()` checks
  it (post-factum invariant check, run on open optionally and from Help).
- `PRAGMA integrity_check` on open (cheap `quick_check`).

## 5.6 Imports & idempotency

- Each import run gets a batch id; every imported row stores an explicit idempotency key
  scoped to (batch, source row) — fintecheng: "prefer explicit keys… scoped to a particular
  operation and client". Re-importing the same file posts nothing twice.
- Reconciliation (fintecheng, adapted offline): imported bank statements can be matched
  against ledger entries by the persisted external id (or amount+date heuristics);
  discrepancies surface as first-class "reconciliation breaks" — never silently overwritten.

## 5.7 Search index (ADR-0006)

- Tantivy index directory lives next to the DB (`search.index/`). Documents = transactions,
  payees, notes, categories, tags, attachments names. Written through on ledger events;
  `rebuild()` recreates it from the DB at any time. The index is a cache: if deleted, it is
  rebuilt; it is never authoritative.

## 5.8 Encryption at rest (optional, phase 2)

- Phase 1: plain SQLite file owned by the user (they can encrypt their disk/folder
  themselves). Phase 2: optional app lock — passphrase → Argon2id → AEAD key; either
  whole-file encryption of the DB or field-level encryption of PII with per-record keys so
  erasure = key deletion ("crypto-shredding", fintecheng GDPR section). Attachments (PII)
  live outside the immutable ledger, in a mutable store that *can* be redacted/erased.
