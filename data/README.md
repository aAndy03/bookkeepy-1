# Primary data

Files here are **primary data** (RULE-DATA-001/002, ADR-0008): grouped constants, rules and
variables the app is built on, imported by engines/APIs/pipelines — never hardcoded in
source. The `primary-data` crate loads them (`include_str!` + startup parse + validation).
Primary data also includes **data-as-dependencies** (IANA tz database via `jiff`, point 11).

| File | Contents | Source |
|---|---|---|
| `currencies.toml` | ISO 4217 currencies: code, name, numeric, minor units, countries | Official ISO 4217 via SIX Group, published 2026-09-17 (`[source]` header) |
| `errors.toml` | Registry of every user-facing error/warning BK-code: severity + i18n key | Project-maintained; governor enforces code ↔ registry ↔ locales |
| `defaults.toml` | Fallback/default policies (rates fallback chain, deleted-manual-rates behavior, system-context defaults) | Project-maintained |
| `load-policies.toml` | What loads when; hot/warm/cold tiers; prefetch rules; usage-tracking scope | Project-maintained |
| `perf-budgets.toml` | Performance budgets asserted in CI by criterion benches + `xtask govern perf` | Project-maintained |

Planned (add when the feature lands, each with a `[source]` header): `shortcuts.toml`
(keyboard map — SSOT, remappable, RULE-INPUT-001), `theme/*.toml` (UI variables/tokens),
`countries.toml` (ISO 3166), `languages.toml` (ISO 639), `category-presets.toml`,
`default-accounts.toml`.

Rules: regenerate external data only from official sources; never hand-edit codes/minor
units; cross-references between primary data use ISO codes (RULE-DATA-002 — e.g. rates
packages must use codes present in `currencies.toml`); schema changes require an ADR;
content changes require a CHANGELOG entry.
