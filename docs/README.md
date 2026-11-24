# Bookkeepy 1 — documentation index

Single source of truth for *how Bookkeepy 1 is built and why*. One canonical file per topic
(RULE-DOC-001): cross-link, never duplicate. If a file outgrows its topic, split by subtopic
and link here.

## Reading order

| # | Document | What it answers |
|---|---|---|
| 01 | [01-vision-and-scope.md](01-vision-and-scope.md) | What Bookkeepy 1 is, who for, in/out of scope |
| 02 | [02-features.md](02-features.md) | User-facing feature inventory by domain |
| 03 | [03-architecture.md](03-architecture.md) | Repos, crate slices, layers, dependency rules, async, loading/caching, perf |
| 04 | [04-conventions.md](04-conventions.md) | Coding rules: modularization + SSOT, primary data, validation, errors, events, async, UI Figma model, governors, docs hygiene |
| 05 | [05-data-money-storage.md](05-data-money-storage.md) | Money, currencies, SQLite schema, ledger, FX rates storage, seamless migrations/longevity, backups, encryption |
| 06 | [06-fintech-principles.md](06-fintech-principles.md) | fintecheng.txt principles, quoted + adapted offline |
| 07 | [07-dependencies.md](07-dependencies.md) | Dependency policy + initial set |
| 08 | [08-network-updates-privacy.md](08-network-updates-privacy.md) | Network allowlist (updates + rates mirror), updater, local system context, privacy |
| 09 | [09-testing-ci-release.md](09-testing-ci-release.md) | Testing menu, lints, CI incl. upgrade-path, releases |
| 10 | [10-process-roadmap.md](10-process-roadmap.md) | How we work; milestones |
| 11 | [11-rates-mirror-and-live-data.md](11-rates-mirror-and-live-data.md) | Live rates via the `bookkeepy-1-rates` mirror sub-repo; manual rates; defaults |

## Companion repo

- [aAndy03/bookkeepy-1-rates](https://github.com/aAndy03/bookkeepy-1-rates) — rates mirror.
  Its `docs/package-format.md` is the SSOT for the rates package format the app consumes.

## Decisions

[adr/](adr/) — numbered, immutable once accepted; supersede, never edit (except status).
Start at [adr/ADR-0001-monorepo-cargo-workspace.md](adr/ADR-0001-monorepo-cargo-workspace.md).

## Primary data

[../data/](../data/) — imported by engines/APIs/pipelines, never hardcoded
(RULE-DATA-001/002). Ships: `currencies.toml`, `errors.toml`, `defaults.toml`,
`load-policies.toml`, `perf-budgets.toml`.

## References and skills

- [../references/fintecheng.txt](../references/fintecheng.txt) — *Fintech Engineering
  Handbook* (Voytek Pitula), quoted in doc 06.
- [../skills/](../skills/) — vendored gpui-kit skill files ([README](../skills/README.md)).
