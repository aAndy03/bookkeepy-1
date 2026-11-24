# ADR-0025 — M2 hardening round: materialization uniqueness, Modify patches, event-kind registry, job backoff, self-check extension point

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** the M2/M2b brainstorm audit (owner prompt: "find bugs, find edge cases,
  QoL; is everything interconnected with M0/M1?") found six concrete defects/gaps in the
  freshly landed code: (1) `Exception::Modify` patches were silently ignored at
  materialization; (2) double materialization of one schedule occurrence was possible
  (no unique constraint; the `is_materialized` check raced outside the posting
  transaction); (3) `replace()` dropped the `schedule_id`/`occurrence_date` link when the
  caller didn't re-pass it; (4) self-transfers posted net-zero entries; (5) corrupt
  `kind`/`status` values were silently defaulted in `get`; (6) a failing job re-queued
  with no delay (poison-job hot loop). Plus three interconnection/SSOT gaps: engines
  emitted no trace events, event kinds were free-string literals, and the boot self-check
  could not include higher-layer (ledger) checks without breaking the dependency
  direction.
- **Decision:**
  1. **RULE-RECUR-004** — a `Modify` exception is a shallow JSON-object patch merged over
     the schedule template for that occurrence only (`apply_patch` in
     `ledger::schedules`); unknown keys fail loudly (BK-RECUR-0001), never ignored.
  2. Schema **v4** (`v4_hardening`): partial unique index
     `transactions_schedule_occ ON transactions(schedule_id, occurrence_date) WHERE
     schedule_id IS NOT NULL AND status <> 'voided'` — the DB-level backstop for
     BK-RECUR-0003; voided rows leave the index so an edited occurrence's replacement can
     carry the link. The engine also checks inside the posting transaction (race-free),
     and `LedgerError` codes now survive the storage closure boundary
     (`from_storage` downcasts both `LedgerError` and `SharedError`).
  3. `transactions::replace` voids the old row BEFORE inserting the replacement and
     INHERITS `schedule_id`/`occurrence_date` from the old row unless explicitly
     overridden; `expand_window` joins the live (non-voided) materialization.
  4. **BK-LEDGER-0009** — self-transfer (from == to) and self-categorized transactions
     (account == its own category's backing account) are rejected at `compile`.
  5. `transactions::get` parses `kind`/`status` strictly — corruption is BK-LEDGER-0007,
     never a silent default.
  6. **RULE-JOB-002** — `jobs.not_before` (schema v4): `fail` re-queues with
     `backoff_delay(attempts, id)` = exponential 2s→…→cap 300s + deterministic id-hash
     jitter (no RNG dep); `claim_next` only picks due jobs. Poison jobs cannot spin.
  7. **RULE-EVT-001 (mechanics)** — event kinds are defined ONLY as consts in
     `bookkeepy_contracts::kinds` and registered in `data/events.toml` (SSOT, loaded by
     `primary-data::events`); every engine emits via consts. `cargo xtask govern events`
     fails on any stray kind literal, and on const/registry mismatch.
  8. Engines (ledger/shared/storage-jobs) emit `tracing` events at their boundaries
     (post / replace / materialize / action / job claim+fail+backoff); local-only file
     logs (RULE-PRIV-001) pick them up.
  9. `session::open_session_with(data_dir, id, &[ExtraCheck])` — higher layers inject
     boot checks as plain `fn(&Db, &Path) -> Result<CheckItem, _>` pointers, recorded in
     `SelfCheckReport.extra`. Dependency direction stays clean (storage never depends on
     ledger — RULE-ARCH-001); the M3 app shell registers `entries::verify_chains` and the
     schedules due-driver there.
  10. QoL (non-blocking, RULE-UX-001): `archive_account` returns `ArchiveOutcome {
      balance }` so the UI can warn when archiving a non-zero account (history keeps it).
- **Consequences:** materialization is exactly-once even under concurrency; edited
  occurrences stay linked in projections; job failures back off instead of spinning;
  event kinds are rename-safe (compile error, not runtime drift); boot self-check covers
  the ledger without storage→ledger coupling. All covered by unit tests + proptest;
  `cargo xtask check` green (19 gates incl. `govern events`).
- **Rules/invariants introduced:** RULE-RECUR-004, RULE-JOB-002, RULE-EVT-001 mechanics.
  Registered in docs/04-conventions.md / doc 05 §5.13.
- **Alternatives considered:** unique index without the `status <> 'voided'` exclusion
  (rejected: `replace` would violate it); RNG-based jitter (rejected: non-deterministic
  tests, extra dep); storage depending on ledger for self-check (rejected: breaks
  RULE-ARCH-001 — injection chosen instead).
