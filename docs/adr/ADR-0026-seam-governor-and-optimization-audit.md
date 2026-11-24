# ADR-0026 — Seam governor (`govern seams`) + M0/M1/M2 optimization audit & batch fixes

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** the owner asked for a long-term mechanism against seams/bugs ("like the
  governors"), a second findings batch across M0/M1/M2, and an optimization audit ("did you
  consider optimizations? are any applied? can you find them easily? is everything
  consistent?"). The M2-hardening audit had already shown one systemic seam bug: engine
  BK-codes were lost at the `tx::post_with` boundary (`StorageError::Foreign` mapped to
  BK-STORE-0001) at 8 of 11 call sites, and `shared::into_storage` mapped action errors to
  BK-STORE-0003.
- **Decision:**
  1. **RULE-SEAM-001 + `cargo xtask govern seams`** — four static seam rules, all
     file-based and test-module-aware:
     - **S1 risky deps**: `rusqlite` only in storage/ledger/shared; `fs2` only in storage;
       `sha2`/`hex` only in storage/ledger. No other crate may list them (any Cargo.toml
       section).
     - **S2 boundary mapping**: in ledger/shared, every `tx::post`/`tx::post_with`
       statement must route its error through `from_storage` (BK-code survival).
     - **S3 table ownership**: literal SQL may only touch the crate's own tables (storage:
       meta/schema_migrations/settings/ui_state/audit_log/outbox/jobs + CREATE
       TRIGGER/INDEX guard lines; ledger: its 9 tables; shared: none — dynamic via the
       `Actionable` allowlist only).
     - **S4 connection construction**: only storage may call `Db::open`/`Connection::open`.
  2. **Boundary contract tests** (`crates/ledger/tests/seams.rs`): engine error code
     survives the boundary (BK-LEDGER-0006 through a failed post, with rollback proven),
     shared action code (BK-ACTION-0001), atomicity of post across audit+outbox+bus,
     `Foreign` round-trip (BK-RECUR-0003).
  3. **Seam bug fixes**: all 11 `tx::post_with` sites in ledger/shared now map
     `from_storage`; `shared::into_storage` uses `Foreign` (was `Integrity`) and gained
     `from_storage`.
  4. **Optimization audit results (applied):**
     - schema **v5** covering indexes: `journal_entries(value_ts)`, `outbox(published)`,
       `jobs(state, not_before)`, `tx_tags(tag_id)`, `transactions(status)`.
     - N+1 eliminated: `account_balances` is ONE grouped query (was one query per
       account); `expand_window` joins materialized txs in ONE window query (was one per
       occurrence); `materialize_due` fetches done-dates per schedule in ONE query.
     - hot paths use `prepare_cached` (balances, trial balance, expand_window,
       materialize_due, jobs claim, outbox replay).
     - already present from M1 and kept consistent: WAL + `synchronous=NORMAL` +
       `busy_timeout` + `wal_autocheckpoint=1000` + `foreign_keys=ON` + 0600 file perms.
     - measurability: new criterion bench `ledger/benches/post_and_balances.rs`
       (post_100_expenses, read_all_balances_grouped, trial_balance) + three new budgets
       in `data/perf-budgets.toml` (RULE-PERF-001) so regressions are caught in CI.
- **Consequences:** seam bug classes are now compile/CI-time failures instead of runtime
  surprises; hot read paths are single-query; every optimization is findable via
  `govern perf` + `perf-budgets.toml` + benches next to the crate (RULE-MOD-002).
- **Rules/invariants introduced:** RULE-SEAM-001 (S1–S4). Registered in doc 04.
- **Alternatives considered:** runtime boundary wrappers (rejected: can't catch missing
  mappings); regex-based SQL parsing (rejected: manual token scan is enough and dep-free).
