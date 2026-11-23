# Bookkeepy 1 — documentation index

This directory is the single source of truth for *how Bookkeepy 1 is built and why*.
Every prompt-driven change to the app is interpreted against these documents first.

## Reading order

| # | Document | What it answers |
|---|---|---|
| 01 | [01-vision-and-scope.md](01-vision-and-scope.md) | What Bookkeepy 1 is, who it is for, what is in/out of scope |
| 02 | [02-features.md](02-features.md) | The user-facing feature inventory, grouped by domain |
| 03 | [03-architecture.md](03-architecture.md) | Monorepo layout, crate layering, dependency rules, UI stack, data flow |
| 04 | [04-conventions.md](04-conventions.md) | Coding rules: primary data, naming, pipelines/engines/APIs, validation, errors, events, rule IDs |
| 05 | [05-data-money-storage.md](05-data-money-storage.md) | Money model, currencies, SQLite storage, ledger schema, search index, backups, encryption |
| 06 | [06-fintech-principles.md](06-fintech-principles.md) | The fintecheng.txt principles, quoted and adapted to an offline app |
| 07 | [07-dependencies.md](07-dependencies.md) | Dependency policy and the initial dependency set |
| 08 | [08-network-updates-privacy.md](08-network-updates-privacy.md) | Offline-first network policy, updater, privacy guarantees |
| 09 | [09-testing-ci-release.md](09-testing-ci-release.md) | Testing strategy, CI, release/packaging pipeline |
| 10 | [10-process-roadmap.md](10-process-roadmap.md) | How we work: prompts → ADRs → issues → PRs; milestone roadmap |

## Decisions

Architecture Decision Records live in [adr/](adr/). They are numbered, immutable once
accepted, and superseded (never edited) by newer ADRs. Start at
[adr/ADR-0001-monorepo-cargo-workspace.md](adr/ADR-0001-monorepo-cargo-workspace.md).

## Primary data

Data files that engines/APIs/pipelines import (never hardcoded in source) live in
[../data/](../data/). First one: [../data/currencies.toml](../data/currencies.toml)
(ISO 4217, official SIX Group list).

## References and skills

- [../references/fintecheng.txt](../references/fintecheng.txt) — *Fintech Engineering
  Handbook* by Voytek Pitula. Quoted and adapted in doc 06.
- [../skills/](../skills/) — vendored gpui-kit skill files (see [../skills/README.md](../skills/README.md)).
