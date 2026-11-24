# 07 — Dependency policy

## Policy (RULE-DEP-001)

- **Latest versions** of every dependency, always. Upgrades are routine, small and frequent
  (`cargo update` weekly cadence in CI), never big-bang. (Verified 2026-11-24: serde 1.0.229,
  toml 1.2.2, jiff 1.0.3, uuid 1.29.3, thiserror 2.0.21, rusqlite 0.43.0, tantivy 0.27.0,
  gpui-kit 0.11.0, sha2 0.11.1.)
- All versions pinned centrally in the workspace `Cargo.toml` (`[workspace.dependencies]`);
  member crates use `dep.workspace = true`. The binary's `Cargo.lock` is committed.
- **Declared on first use (M0 finding):** cargo 1.100 warns on unused
  `[workspace.dependencies]` entries, so an entry is added to the manifest when the first
  crate uses it; this doc remains the policy SSOT with versions + rationale.
- Rust toolchain pinned by `rust-toolchain.toml` (`channel = "1.100.0"`, edition 2024).
- Minimalism: a dependency is added only with a written rationale (which doc/ADR needs it)
  in the PR description. Prefer the standard ecosystem crate over hand-rolled code —
  **except** where correctness demands our own code (money math, ledger), and except where
  the user ruled it out: *we do not build an indexing/searching engine — we use 100% of the
  tantivy API* (ADR-0006).
- Licenses must be compatible with GPL-3.0-or-later (checked by `cargo deny` in CI);
  `cargo audit` for vulnerabilities, `cargo vet` (or supply-chain review) for trust.
- No floating-point types in money paths (INV-MONEY-001); no `unsafe` (workspace lint).
- **System-context collection uses official, actively maintained dependencies only
  (RULE-DEP-002):** `sysinfo`, `os_info`, `iana-time-zone`, `jiff` (IANA tzdb), and
  display/GPU info via gpui-kit's own platform APIs (the UI stack's official source).
  A dependency for this job is chosen only if it is official for the platform concern or
  demonstrably maintained; the choice + rationale is recorded here and in the PR.

## Initial dependency set (to be declared in the workspace manifest)

| Crate | Version policy | Why |
|---|---|---|
| `gpui-kit` | `= "0.11.0"`, features `["gpui-fast"]` | the one UI dependency (ADR-0007); GPUI Fast backend via the `gpui-fast` feature |
| `rusqlite` | latest, features `bundled`, `blob`, `backup` | all database access (ADR-0005); bundled SQLite so users need no system SQLite |
| `tantivy` | latest | 100% of its API for search (ADR-0006) |
| `serde` / `serde_json` | latest, derive | DTOs, primary data, exports |
| `toml` | latest | parsing `data/*.toml` primary data |
| `thiserror` / `anyhow` | latest | error rules RULE-ERR-001/002 |
| `uuid` | latest, v7 + serde | entity ids |
| `jiff` | `"1"`, serde | civil dates/times; IANA time-zone database ships as a dependency = primary data as a library (point 11). (gpui-kit 0.11 uses jiff 0.2 internally; our workspace uses jiff 1 independently — no shared types needed.) |
| `iana-time-zone` | latest | OS time-zone detection (local system context, doc 08) |
| `sysinfo` | latest | CPU/RAM/process info for local diagnostics + perf budgets — local-only, never transmitted (doc 08) |
| `os_info` | latest | OS name/version for diagnostics + fatal-error reports (doc 08) |
| `sha2` / `hex` | latest | hash chain tamper evidence (doc 05 §5.5) |
| `argon2` + `chacha20poly1305` (or `aes-gcm`) + `zeroize` | latest | optional app lock / at-rest encryption (doc 05 §5.8) |
| `csv` + `encoding_rs` | latest | bank CSV imports incl. Windows-1250 encodings |
| `roxmltree` | latest | OFX/QFX XML parsing (no full XML stack) |
| `directories` | `"6"` (6.0.0, M0.5) | OS data-dir resolution (profile registry) |
| `tracing` 0.1.44 + `tracing-subscriber` 0.3.23 (env-filter) + `tracing-appender` 0.2.5 (M0.5) | local rotating logs only — never shipped anywhere (RULE-PRIV-001) |
| `ureq` (rustls) | latest | the one HTTP client, used **only** by the `updater`/`rates` crates (RULE-NET-001) |
| `semver` | latest | update version comparison (updater) |
| `printpdf` (or successor) | latest | PDF rendering for reports — must be actively maintained + GPL-compatible; final choice recorded here at implementation with rationale (RULE-DEP-002 spirit applies: best maintained option) |
| `opener` | latest | "open data folder" via the OS (actively maintained) |
| dev: `proptest` 1.12, `tempfile` 3.27 (M0.5), `criterion` 0.8.2 (M0.5 benches), `insta` (later) | doc 09 testing menu |

Everything else is added later via PR with rationale, per RULE-DEP-001.
