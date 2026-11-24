# Changelog — Bookkeepy 1

Keep a Changelog format. Every PR adds an entry under `Unreleased` (doc 09); releases
move it under the version tag. The release workflow fails if a release section is missing.

## [Unreleased]

### Added
- M2 hardening round (2026-11-24, ADR-0025): fix Modify-exception patches at materialization (RULE-RECUR-004); schema v4 unique partial index transactions_schedule_occ + in-tx guard (exactly-once materialization, BK-RECUR-0003 survives closure boundary); replace() inherits schedule link; self-transfer rejected (BK-LEDGER-0009); strict row parsing; jobs persisted backoff not_before (RULE-JOB-002); contracts::kinds + data/events.toml event registry with new 'govern events' check (RULE-EVT-001); open_session_with extra boot checks; tracing at engine boundaries; archive_account returns balance warning; 19/19 xtask check PASS
- M2b (2026-11-24): recurring/scheduled transactions — storage v3 schedules table + transactions.schedule_id/occurrence_date; shared::api::recurrence SSOT (anchor-based bounded expansion, exceptions, cap); ledger::schedules (create/expand_window/materialize_one/materialize_due no-backfill boot driver + durable range jobs via storage::jobs); BK-RECUR-0001..0003 + en/ro locales; docs 02/03/04/05/10 + code-map updated; xtask check 18/18 PASS
- M2 ledger engine: bookkeepy-ledger + bookkeepy-shared crates, storage v2 (accounts/journal_entries/postings/transactions/catalog + immutability triggers), tx::post_with side-effect-in-tx, double-entry post with per-currency balance validation, linked reversals, hash chains + verify_chains, transactions compiled to balanced postings with status lifecycle (voided=reversal), derived balances + trial balance, shared audited actions (rename/archive/restore/change_status), ADR-0024 soft-delete semantics, BK-LEDGER-0001..0008 + BK-ACTION-0001, proptest invariants (trial balance any prefix, reversal conservation)
- M1 hardening: crash-safe restore (stale WAL sidecars removed, read-only backup verify, collision-free auto names), incremental checkpointed audit verify at boot, self-check gains migrations_match/orphan_settings/wal_bytes/boot_ms, boot order synced to load-policies.toml step ids (BOOT_STEPS subsequence test), settings snapshot + SSOT default_value, bus panic isolation, unix 0600 perms, dev-channel 'cargo xtask dev-reset-profile' (no down-migrations; compensating-migration fixture test proves data-preserving reverts)
- M1 addendum: session::open_session composition root (registry->lock->db->logging->selfcheck->outbox->auto backup->set_active), durable jobs state machine (RULE-JOB-001), atomic tx::post (outbox+audit one tx), settings keys validated against SSOT (BK-DATA-0003), i18n error_text, backup prune KEEP_BACKUPS=10, wal_autocheckpoint, 'local live backup' terminology (fully offline)
- M1 storage skeleton: crates/storage (WAL conn, forward-only migrations + downgrade refusal BK-DATA-0002, durable outbox->bus, audit hash chain, backup/verify/restore, single-instance lock, self-check, ui_state/settings KV); +BK-STORE-0003..0006; deps rusqlite 0.43.0, sha2 0.11.1, hex 0.4.3, fs2 0.4.3
- round-5 QoL: xtask pr/changelog commands, .vscode settings, README badges; data/settings-defaults.toml (SSOT settings schema, governor-validated); load-policies boot_sequence; doc 02 F22 app chrome; doc 05 §5.14 window/session state + single-instance spec
- Rates mirror (aAndy03/bookkeepy-1-rates): BNR full history 2005–2026 committed as kind="archive" packages (23 packages incl. daily 2026-11-24) + manifest/SHA256SUMS; spec gains the additive archive kind; scheduled update-rates.yml fetches only the latest quote (curs.bnr.ro/nbrfxrates.xml, after 13:00 banking days); manual backfill.yml rebuilds yearly archives; ed25519 signing (ADR-0023) activates once the RATES_SIGNING_KEY secret is set.
- M0.5 foundation crates: `build-info` (build facts + channel + embedded GPL text),
  `contracts` (EventEnvelope/Actor), `bus`, `diagnostics` (bookkeepy-diagnostic@1 +
  panic hook), `logging` (local rotating logs), `profile` (registry + data-dir layout),
  `i18n` (EN/RO catalogs with enforced parity).
- Primary data: `locales/en.toml`, `locales/ro.toml`, `theme/*.toml` (colors, spacing,
  radii, typography, elevation + index), `shortcuts.toml` (default keymap SSOT).
- Error registry: BK-APP-0001, BK-LOG-0001, BK-PROFILE-0001, BK-DATA-0002, BK-RATES-0002.
- Supply chain: `deny.toml` (cargo-deny 0.22 schema), Renovate config, SPDX headers on
  all Rust sources (`govern lints` enforces).
- Tooling: `cargo xtask new-crate` / `new-adr` / `doctor` / `licenses` / `check` /
  `hook`; pre-commit hook; `.gitattributes` (LF); `.editorconfig`.
- Code registry SSOT: `docs/registry/code-map.toml` (`govern meta`, RULE-META-001).
- Channels: dev/release via `BOOKKEEPY_CHANNEL` + `dev-channel` cargo feature
  (ADR-0021, RULE-CHANNEL-001); downgrade policy forward-only schema (ADR-0022,
  RULE-DATA-004, BK-DATA-0002); rates packages ed25519-signed (ADR-0023, BK-RATES-0002).
- CI: minutes-budget policy — Actions run only on releases/prereleases
  (`release.yml`) and manual `workflow_dispatch`; local gate `cargo xtask check`.
- Community files: SECURITY.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md, PR/issue templates,
  CODEOWNERS; tag protection for `v*`.

## [0.1.0] — not released yet
