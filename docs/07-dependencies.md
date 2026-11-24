# 07 — Dependency policy

## Policy (RULE-DEP-001)

- **Latest versions** of every dependency, always. Upgrades are routine, small and frequent
  (`cargo update` weekly cadence in CI), never big-bang.
- All versions pinned centrally in the workspace `Cargo.toml` (`[workspace.dependencies]`);
  member crates use `dep.workspace = true`. The binary's `Cargo.lock` is committed.
- Rust toolchain pinned by `rust-toolchain.toml` (`channel = "1.100.0"`, edition 2024).
- Minimalism: a dependency is added only with a written rationale (which doc/ADR needs it)
  in the PR description. Prefer the standard ecosystem crate over hand-rolled code —
  **except** where correctness demands our own code (money math, ledger), and except where
  the user ruled it out: *we do not build an indexing/searching engine — we use 100% of the
  tantivy API* (ADR-0006).
- Licenses must be compatible with GPL-3.0-or-later (checked by `cargo deny` in CI);
  `cargo audit` for vulnerabilities, `cargo vet` (or supply-chain review) for trust.
- No floating-point types in money paths (INV-MONEY-001); no `unsafe` (workspace lint).

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
| `jiff` | latest, serde | civil dates/times (gpui-kit 0.11 itself moved off chrono to jiff); IANA time-zone database ships as a dependency = primary data as a library (point 11) |
| `iana-time-zone` | latest | OS time-zone detection (local system context, doc 08) |
| `sysinfo` | latest | CPU/RAM/OS/process info for local diagnostics + perf budgets — local-only, never transmitted (doc 08) |
| `sha2` / `hex` | latest | hash chain tamper evidence (doc 05 §5.5) |
| `argon2` + `chacha20poly1305` (or `aes-gcm`) + `zeroize` | latest | optional app lock / at-rest encryption (doc 05 §5.8) |
| `csv` + `encoding_rs` | latest | bank CSV imports incl. Windows-1250 encodings |
| `roxmltree` | latest | OFX/QFX XML parsing (no full XML stack) |
| `directories` | latest | OS data-dir resolution |
| `tracing` + `tracing-subscriber` + `tracing-appender` | latest | local logs only (never shipped anywhere) |
| `ureq` (rustls) | latest | the one HTTP client, used **only** by the `updater` crate (RULE-NET-001) |
| `semver` | latest | update version comparison (updater) |
| dev: `proptest`, `insta`, `tempfile`, `criterion` | latest | doc 09 testing menu |

Everything else is added later via PR with rationale, per RULE-DEP-001.
