# Primary data

Files in this directory are **primary data** (RULE-DATA-001, ADR-0008): grouped sets of
constants the app is built on, imported by engines/APIs/pipelines — never hardcoded in
source code. They are compiled into the binary by the `primary-data` crate
(`include_str!` + startup parse + validation).

| File | Contents | Source |
|---|---|---|
| `currencies.toml` | All ISO 4217 currencies: code, name, numeric code, minor units, countries | Official ISO 4217 list via SIX Group, published 2026-09-17 (see `[source]` header in the file) |

Planned (add when the feature lands, each with a `[source]` header):
`countries.toml`, `languages.toml`, `category-presets.toml`, `date-formats.toml`,
`default-accounts.toml`.

Rules: regenerate only from official sources; never hand-edit codes/minor units; schema
changes require an ADR; content changes require a CHANGELOG entry.
