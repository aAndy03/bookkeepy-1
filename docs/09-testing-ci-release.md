# 09 — Testing, CI, release

## Testing menu (from fintecheng "Testing", adopted per doc 06 §6.8)

| Technique | Where in Bookkeepy 1 |
|---|---|
| Property-based (`proptest`) | "for any sequence of postings, the books balance" (INV-LEDGER-001); money rounding/conversion properties; FX conversion round-trips |
| Invariants after every step | ledger test harness applies ops one by one and asserts invariants (debits=credits, hash chain, balances derived = recomputed) after each |
| Generative idempotency | replay whole import batches / API calls twice; assert zero effect the second time |
| Crash & resume injection | kill pipelines (import, schedule materialization, backup) at each stage; resume; assert exactly-once outcome |
| Round-trip | serialize→deserialize Money/DTOs/exports; assert equality (no precision loss at boundaries) |
| Golden (`insta`) | reports, statements, export files pinned to reviewed snapshots |
| Backward compatibility | corpus of old DB files / old-format payloads in `tests/fixtures/`; current code must read them; every migration tested up+down on fixtures |
| UI | gpui-kit test utilities (see `skills/gpui-kit/references/gpui/test*.md`) for view logic; presenters unit-tested without a window |
| Jobs (RULE-JOB-001) | crash/resume injection at every stage; auto-retry up to max attempts then manual-retry surfacing; pause/resume/cancel; startup driver resumes interrupted jobs |
| Audit (RULE-AUD-001) | tamper-evidence: flipping any audit/journal row breaks `verify_chain()`; attempts to UPDATE/DELETE audit tables fail; app_history retention settings honored |

Levels: unit (per crate) → integration (`tests/`) → golden → property. Money-touching code
requires a property test before merge.

## Lints (workspace `Cargo.toml`)

- `unsafe_code = "forbid"`; `missing_docs = "warn"` (CI escalates to deny).
- clippy: `unwrap_used/expect_used = "deny"` (outside tests), `dbg_macro`, `todo!` denied;
  `float_arithmetic` denied in `core` and money crates (INV-MONEY-001).

## CI (GitHub Actions, `.github/workflows/` — added in M0, extended in M0.5)

- Matrix: Windows / macOS / Linux × stable Rust 1.100 (pinned by `rust-toolchain.toml`).
- Jobs (stable `name:`s — these are the required status-check contexts on `main`):
  `check (ubuntu-latest)` / `check (windows-latest)` / `check (macos-latest)` =
  `cargo fmt --check` · `cargo clippy --workspace --all-targets -- -D warnings` ·
  `cargo test --workspace` · `cargo xtask govern` (whole governor tree: deps + net
  allowlist, lints, docs, adr, errors registry ↔ code ↔ **locales**, primary-data
  validation + ISO referential integrity) · `audit (rustsec)` · `deny (cargo-deny)` —
  licenses allow-list (GPL-3.0-or-later compatible), AGPL/SSPL banned, duplicates and
  wildcards denied, advisories/yanked denied, crates.io-only sources (`deny.toml`);
  `cargo vet` advisory until M2 · `bench (sanity)` = `cargo bench --workspace -- --test`
  (benches compile + run once; measured budget comparison lands with the perf governor).
  Later milestones add: offline smoke test (app fully functional with networking
  disabled) · **upgrade-path test**: fixtures in `tests/fixtures/versions/` written by
  every previous tagged release are opened, migrated and verified by the new build
  (INV-DATA-001, points 14–15) · build artifacts.
- Branch protection on `main` (set via `gh api`, M0.5): PRs only, required reviews (1)
  + required status checks = the CI job names above, no force-push, no deletions
  (fintecheng SDLC, doc 06 §6.7). CODEOWNERS + PR/issue templates in `.github/`.
- Dependency updates: Renovate (`.github/renovate.json`, cargo weekly / major monthly);
  every dep PR must pass the full CI incl. `deny`.
- Pre-commit locally: `cargo xtask hook install` once, then `.githooks/pre-commit` runs
  fmt + fast governors + clippy on every commit.

## Release & packaging

- Semver; `CHANGELOG.md` (Keep a Changelog) updated in the same PR as the change.
- Tag `vX.Y.Z` → release workflow builds installers (Windows `.exe` per-user installer,
  macOS `.dmg`, Linux `.deb`/`.tar.gz`) + `SHA256SUMS`; published as a GitHub Release —
  the only distribution channel and the updater's only source (doc 08).
- What version is running, who released it and when is reconstructable from tags +
  changelog (fintecheng: "Deployments are traceable").
- Installers are unsigned until we add signing; this is documented in the README.
- **Uninstall (INV-APP-001, ADR-0016):** uninstallers remove only app files, never the data
  folder; the "also delete my data" option is an unchecked-by-default checkbox, preceded by
  a full-export offer (export package format: doc 05 §5.11). CI tests the uninstaller against
  a fixture data folder and asserts the data survives.
- PDF/print: golden tests for generated PDFs; print via OS dialog.
- Accessibility: keyboard-only pass + focus-order and screen-reader-label checks per screen
  (doc 10 item 8/9); CI runs an automated a11y tree check where the platform allows.
