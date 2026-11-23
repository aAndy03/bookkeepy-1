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

Levels: unit (per crate) → integration (`tests/`) → golden → property. Money-touching code
requires a property test before merge.

## Lints (workspace `Cargo.toml`)

- `unsafe_code = "forbid"`; `missing_docs = "warn"` (CI escalates to deny).
- clippy: `unwrap_used/expect_used = "deny"` (outside tests), `dbg_macro`, `todo!` denied;
  `float_arithmetic` denied in `core` and money crates (INV-MONEY-001).

## CI (GitHub Actions, `.github/workflows/` — added in M0)

- Matrix: Windows / macOS / Linux × stable Rust 1.100.
- Jobs: `cargo fmt --check` · `cargo clippy --workspace -- -D warnings` · `cargo test
  --workspace` · `cargo xtask check-deps` (dependency direction + RULE-NET-001) ·
  `cargo audit` · `cargo deny check` (licenses/bans) · offline smoke test (app starts with
  networking disabled) · build artifacts.
- Branch protection on `main`: PRs only, required reviews + required status checks, no
  force-push (fintecheng SDLC, doc 06 §6.7).

## Release

- Semver; `CHANGELOG.md` (Keep a Changelog) updated in the same PR as the change.
- Tag `vX.Y.Z` → release workflow builds installers (Windows `.exe` per-user installer,
  macOS `.dmg`, Linux `.deb`/`.tar.gz`) + `SHA256SUMS`; published as a GitHub Release —
  the only distribution channel and the updater's only source (doc 08).
- What version is running, who released it and when is reconstructable from tags +
  changelog (fintecheng: "Deployments are traceable").
- Installers are unsigned until we add signing; this is documented in the README.
