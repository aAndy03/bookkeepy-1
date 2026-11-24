# Pull request — Bookkeepy 1

> PRs run **zero** GitHub Actions (minutes-budget policy, doc 09): the gate is local.
> Run `cargo xtask check` (fmt/clippy/tests/governors/bench/deny) before opening — CI
> runs the identical suite only on releases and manual dispatch.

Checklist (doc 04 §4.15, doc 09):

- [ ] `cargo xtask check` run locally and green (fmt, clippy -D warnings, tests, governors, bench sanity, deny)
- [ ] Change traces to a rule/invariant/ADR (or links the doc section it updates) — RULE-SYS-001
- [ ] Docs updated where the change propagates (docs/, data/README.md, crate README) — RULE-DOC-001
- [ ] New/changed error codes are registered in `data/errors.toml` **and** every locale (`data/locales/*.toml`) — RULE-ERR-002 (`govern errors` checks)
- [ ] New primary data follows RULE-DATA-001/002/003 (`[source]` header, cross-refs by code)
- [ ] New dependencies: declared on first use in root `Cargo.toml`, rationale added to `docs/07-dependencies.md`, `cargo deny check` passes — RULE-DEP-001
- [ ] No network code outside `bookkeepy-updater`/`bookkeepy-rates` — RULE-NET-001 (`govern net` checks)
- [ ] No `unsafe`, no `unwrap/expect/dbg!/todo!` in shipped crates (tests/benches excepted)
- [ ] Workspace-internal dependency edges added to `xtask/src/govern.rs` allowed map if a new crate pair appears (`govern deps` checks)
- [ ] Tests: unit + proptest where money/math is involved; fixtures under `tests/fixtures/` for upgrade-path tests (INV-DATA-001)
- [ ] CHANGELOG.md entry under Unreleased

## What & why

<!-- one paragraph: user-facing effect + engineering rationale -->
