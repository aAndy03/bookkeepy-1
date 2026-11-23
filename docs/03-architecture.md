# 03 — Architecture

## Repo topology: monorepo (ADR-0001)

One GitHub repository, one **Cargo workspace** (monorepo). Multirepo was considered and
rejected for now: Bookkeepy 1's crates share domain types (Money, Currency, ids, events)
intensively, and atomic cross-crate refactors are the norm, not the exception. A split into
multiple repos is possible *later* and is a deliberate, criteria-based decision (see
ADR-0001 "Revisit criteria"); until then, crate boundaries inside the workspace give us the
same ownership clarity a multirepo would.

```
bookkeepy-1/                     (this repo, workspace root)
├── Cargo.toml                   workspace manifest: pinned versions, workspace lints
├── rust-toolchain.toml          channel = "1.100.0"
├── crates/
│   ├── core/                    Money, Currency (from primary data), ids, time, errors, invariants. No I/O.
│   ├── primary-data/            loads data/*.toml (include_str!) and exposes typed accessors (ADR-0008)
│   ├── contracts/               DTOs + domain events shared across crates; UI- and storage-agnostic
│   ├── storage/                 rusqlite: connection, migrations, repositories, hash-chained journal (ADR-0005)
│   ├── ledger/                  engine_ledger: double-entry posting, balances derivation, reversals (ADR-0009/0010)
│   ├── transactions/            engine_transactions: user-level transactions → postings; splits, schedules
│   ├── catalog/                 categories, tags, parties (payees) — user-editable, presets from primary data
│   ├── fx/                      engine_fx: stored rates, valuation, conversion with rounding account
│   ├── import/                  pipeline_import: parse → map → validate → dedup → post → index
│   ├── export/                  pipeline_export: ledger/reports → CSV/JSON/PDF (pure projections)
│   ├── budget/                  engine_budget: envelopes, planned vs actual, rollover
│   ├── reporting/               engine_reporting: cashflow, net worth, statements… (golden-tested)
│   ├── search/                  engine_search: tantivy wrapper, write-through + rebuild (ADR-0006)
│   ├── documents/               attachments storage + reference
│   ├── backup/                  engine_backup: snapshot / restore / verify
│   ├── audit/                   append-only audit trail + hash-chain verification
│   ├── settings/                app config (DB + local file), locale, formats
│   ├── i18n/                    locale catalogs (EN, RO first)
│   ├── security/                app lock, key derivation, optional at-rest encryption
│   ├── updater/                 the ONLY crate allowed to perform HTTP (RULE-NET-001, ADR-0003)
│   ├── app-services/            composition root: wires engines to storage, exposes `api` to the UI
│   └── ui/                      gpui-kit views: shell + one module per feature; presenters; theme
├── app/                         the binary crate (thin): window setup, DI, run
├── xtask/                       repo governors: CI checks as cargo commands (dep rules, lints, ADR lint)
├── data/                        primary data (TOML) — imported, never hardcoded
├── tests/                       workspace-level integration tests + fixtures (old-format corpus)
└── packaging/                   installers per OS (later phase)
```

## Dependency direction (RULE-ARCH-001)

```
core ← primary-data ← contracts ← storage ← {ledger, transactions, catalog, fx, …}
                                                        ↓
                                              app-services ← ui ← app
updater → (HTTP only, isolated)          search ← storage events (write-through)
audit  ← storage events
```

- `core` depends on nothing in the workspace.
- Domain crates never depend on `ui` or `app` (RULE-UI-004).
- `ui` talks only to `app-services`' public `api` — never to storage directly.
- `updater` is a leaf with exactly one HTTP client; nothing else may open sockets
  (RULE-NET-001). Enforced by `cargo xtask check-deps`.
- Cycles are forbidden; enforced by `cargo xtask check-deps` (cargo metadata based).

## Data flow (runtime)

```
UI action → app-services api → validate (RULE-VAL-001) → engine_* (state machine)
    → storage transaction (rusqlite) → domain events emitted
        → audit (append-only log)  → search (tantivy write-through)  → UI notify (gpui cx.notify)
```

Every state change is: validate → mutate DB in one transaction → emit events → observers
react. The DB write is the source of truth; the tantivy index and all balances are derived.

## UI stack (ADR-0007)

- `gpui-kit = { version = "0.11.0", features = ["gpui-fast"] }` — the single UI dependency.
- **All** styled components from the [component catalog](https://gpui-kit.com/component/)
  are used for the interface; unstyled `base` primitives only where the catalog has no
  equivalent (and the gap is documented in an ADR).
- Conventions for components follow the vendored skills in `skills/gpui-kit/`
  (component-family conventions, recipes, coding guides) and the design guides in
  `skills/gpui-kit-design-guides/`.
- Screens are `Entity` + `Render`; domain → view-model conversion happens in `presenters`
  inside `ui`, so views never touch the DB.

## Search (ADR-0006)

Tantivy, 100% of its API — we do not build an indexing/searching engine. The index lives
in the app data directory next to (never inside) the SQLite file, is written through from
storage events, and is fully rebuildable from the ledger (`engine_search::rebuild()`), so
it is a cache, never a source of truth.

## Storage (ADR-0005)

One SQLite file per user "book", opened through `rusqlite` with the `bundled` feature,
WAL mode, foreign keys ON, `PRAGMA user_version` for migrations. Schema and rules in
doc 05.

## Why this shape

The user prompts describe surfaces ("manage my finances offline"); this architecture is
the hidden machinery that makes those surfaces trustworthy: a validated command layer, a
double-entry ledger as the single source of truth, derived caches (balances, index,
reports), append-only audit, and a hard network boundary. Additions/refactors/deprecations
land in exactly one crate each, which is what keeps a long-term project maintainable.
