# Bookkeepy 1

**Bookkeepy 1** is an open-source, offline-first desktop application for managing your
personal finances. One app for cash, bank accounts, cards, wallets, loans, budgets,
subscriptions, documents and multiple currencies — with a double-entry ledger underneath
so the books always balance.

- **Offline first.** The app works fully offline. Its only internet connections are
  downloading app updates from this repository's GitHub Releases and FX-rates packages from
  the mirror sub-repo [aAndy03/bookkeepy-1-rates](https://github.com/aAndy03/bookkeepy-1-rates)
  (the app never contacts rate sources directly) — see
  [docs/08-network-updates-privacy.md](docs/08-network-updates-privacy.md) and
  [docs/11-rates-mirror-and-live-data.md](docs/11-rates-mirror-and-live-data.md).
- **Your data is yours.** Bookkeepy never closes your data in. Everything you create lives
  in a single SQLite file on your device that you own, can copy, move, back up and delete
  (see [docs/05-data-money-storage.md](docs/05-data-money-storage.md)).
- **Correct by design.** Money is stored as integer minor units, never floating point;
  the ledger is double-entry and append-only; corrections are reversals, not edits
  (see [docs/06-fintech-principles.md](docs/06-fintech-principles.md)).

## Stack

| Layer | Choice |
|---|---|
| Language | Rust 1.100.0+ (edition 2024, pinned via `rust-toolchain.toml`) |
| UI | [gpui-kit](https://gpui-kit.com/docs/) `0.11.0` with the `gpui-fast` feature — all styled components from the [component catalog](https://gpui-kit.com/component/) |
| Database | `rusqlite` (bundled SQLite), one file owned by the user |
| Search | [tantivy](https://crates.io/crates/tantivy) — 100% of its API, no custom indexing engine |
| License | GNU GPL v3 or later (`GPL-3.0-or-later`) — see [LICENSE](LICENSE) |

## Status

**M0 done, M0.5 done (2026-11-24):** Cargo workspace scaffold + `bookkeepy-core`
(Money/CurrencyCode/ErrorCode/EntityId/time) + `bookkeepy-primary-data` (loads and
validates `data/*.toml` incl. locales) + M0.5 foundation crates (`build-info`,
`contracts`, `bus`, `diagnostics`, `logging`, `profile`, `i18n`) + `xtask` governor tree
(+ `new-crate` scaffolder, git hooks) + CI (fmt/clippy/test/govern/deny/audit/bench) +
branch protection + supply-chain config. Brainstorming docs are complete under
[docs/](docs/README.md); development follows
[docs/10-process-roadmap.md](docs/10-process-roadmap.md) (next: M1 storage, M2 ledger).

```bash
cargo xtask check        # the FULL local gate (what CI runs on releases only — see docs/09):
                         # fmt, clippy -D warnings, tests, governors, bench sanity, cargo deny
cargo xtask hook install # once per clone: fast pre-commit gate (fmt + governors + clippy)
cargo xtask govern       # the governor tree alone (data/errors/adr/docs/deps/net/…)
```

Pushes and PRs run **no** GitHub Actions (minutes budget); releases/prereleases run the
full CI + build release assets. Branch protection on `main`: 1 review, no force-push.

## Repository layout

```
bookkeepy-1/
├── docs/            product + engineering decisions, conventions, ADRs, roadmap
├── docs/adr/        architecture decision records (ADR-0001 …)
├── data/            primary data files (currencies.toml, …) imported by engines/APIs
├── references/      fintecheng.txt — the fintech engineering handbook we quote and follow
├── skills/          gpui-kit + gpui-kit-design-guides skill files (vendored from longbridge/gpui-kit)
├── LICENSE          GPL-3.0-or-later
└── (app code will live here as a Cargo workspace once implementation starts)

Companion repo: [aAndy03/bookkeepy-1-rates](https://github.com/aAndy03/bookkeepy-1-rates) —
the FX-rates mirror the app downloads rates packages from (never the sources directly).
```

## Getting help

Open an issue on this repository. For the design and engineering rules that govern every
change, start at [docs/README.md](docs/README.md).

## License

Licensed under the **GNU General Public License, version 3 or (at your option) any later
version** (`GPL-3.0-or-later`). See [LICENSE](LICENSE).
