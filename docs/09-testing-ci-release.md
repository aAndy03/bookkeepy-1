# 09 — Testing, CI, release

## Testing menu (from fintecheng "Testing", adopted per doc 06 §6.8)

| Technique | Where in Bookkeepy 1 |
|---|---|
| Property-based (`proptest`) | "for any sequence of postings, the books balance" (INV-LEDGER-001); money rounding/conversion properties; FX conversion round-trips |
| Invariants after every step | ledger test harness applies ops one by one and asserts invariants (debits=credits, hash chain, balances derived = recomputed) after each |
| Generative idempotency | replay whole import batches / API calls twice; assert zero effect the second time |
| Crash & resume injection | kill pipelines (import, schedule materialization, backup) at each stage; resume; assert exactly-once outcome |
| Round-trip | serialize→deserialize Money/DTOs/exports; assert equality (no precision loss at boundaries) |
| Fuzzing (M4+) | `cargo-fuzz` targets for the import parsers (CSV/OFX/QIF) — bank files are hostile input; fuzz before real user data hits them |
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

## CI / minutes-budget policy (round-3 decision, 2026-11-24)

**GitHub Actions minutes are spent only on releases/prereleases and manual important
runs — never on ordinary pushes or PRs.** All routine checks run locally on the dev PC:

- **Local full gate — `cargo xtask check`** (run before pushing / opening a PR):
  `cargo fmt --all --check` · `cargo clippy --workspace --all-targets -- -D warnings` ·
  `cargo nextest run --workspace` (falls back to `cargo test` when nextest isn't
  installed) · `cargo xtask govern` (whole governor tree: deps + net allowlist, **lints
  = SPDX headers on all .rs**, docs, adr, errors registry ↔ code ↔ **locales**, meta =
  code registry `docs/registry/code-map.toml`, primary-data validation + ISO referential
  integrity, perf-bench presence) · `cargo bench --workspace -- --test` (bench sanity) ·
  `cargo deny check` when cargo-deny is installed locally (SKIP note otherwise; CI always
  runs it on releases).
- **Pre-commit (fast subset)** — `cargo xtask hook install` once per clone, then every
  commit runs fmt + fast governors + clippy via `.githooks/pre-commit`.
- **`cargo xtask doctor`** — env self-check for contributors (toolchain, hooks, deny,
  nextest, git identity); see CONTRIBUTING.md.
- **`BOOKKEEPY_CHANNEL=dev cargo xtask dev-reset-profile <profile-id> --data-dir <path>`** —
  the dev-phase "revert" (M1 hardening round, doc 10): deletes ONE dev profile tree and
  removes it from the registry. There are no down-migrations by design (INV-DATA-001) —
  in dev, reverting a database means deleting the disposable dev profile. Guard rails:
  refuses on release-channel builds, on path escape, and while the profile is open
  (instance lock held). Shipped profiles are never touched by tooling.

**On GitHub** (`.github/workflows/`):

- `ci.yml` — triggers: `workflow_dispatch` (manual) + `workflow_call` only. **No `push`,
  no `pull_request` triggers** — those cost minutes on every commit. Jobs: `check` matrix
  (ubuntu/windows/macos: fmt, clippy -D warnings, tests, governors), `audit (rustsec)`,
  `deny (cargo-deny)` (licenses allow-list GPL-compatible only, AGPL/SSPL effectively
  banned by omission, duplicates + version wildcards denied, advisories/yanked denied,
  crates.io-only sources — `deny.toml`), `cargo vet` advisory until M2, `bench (sanity)`.
- `release.yml` — triggers: `release: [published, prereleased]`. Calls `ci.yml`, then
  builds `--release` with `BOOKKEEPY_CHANNEL=release` + `SOURCE_DATE_EPOCH` (reproducible,
  ADR-0021), generates `manifest.json` + `SHA256SUMS` (doc 08 wire format v1) plus
  **`THIRD-PARTY-LICENSES.md`** (`cargo about`) and **`sbom.cdx.json`** (`cargo
  cyclonedx`), attaches all to the release. Real installers land with M3/M10
  (`packaging/README.md`). Later milestones add to the release gate: offline smoke test
  · **upgrade-path test** over `tests/fixtures/versions/` (INV-DATA-001, points 14–15).
- Branch protection on `main` (via `gh api`): required PR review (1, CODEOWNERS), no
  force-push, no deletions, linear history, enforce admins — **no required status
  checks**, because PRs no longer run actions; the PR gate is the local `cargo xtask
  check` (PR template checkbox) plus review (fintecheng SDLC, doc 06 §6.7).
- Dependency updates: Renovate (`.github/renovate.json`, cargo weekly / major monthly) —
  Renovate PRs are reviewed by the owner and verified with the local gate before merge.
  **Dependabot is disabled** on this repo (Renovate is the single update bot).
- Repo settings (set via `gh api`, round-4): secret scanning + push protection enabled,
  vulnerability alerts enabled, Wiki disabled (docs are SSOT in-repo), labels
  `bug`/`enhancement`/`dependencies`, branch protection on `main` (1 review +
  CODEOWNERS, no force-push, linear history, no required status checks). Tag protection
  for `v*` attempted via API (endpoint 404 for this token — set it in Settings → Tags if
  desired; releases are also gated by the release workflow itself).

M0.5 CI findings (fixed after the first run failed): cargo-deny 0.22 schema —
`advisories.unmaintained` is a scope (`all`), `yanked` stays a lint level (`deny`),
`severity-threshold` and `licenses.deny` were removed (allow-list omission = deny);
`bans.wildcards` needs `allow-wildcard-paths = true` **and** `publish = false` on all
member crates (workspace.package) for `workspace = true` inheritance; one unavoidable
duplicate (`syn` 2/3) is skipped with a documented reason in `deny.toml`.

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
