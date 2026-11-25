# Primary data

Files here are **primary data** (RULE-DATA-001/002, ADR-0008): grouped constants, rules and
variables the app is built on, imported by engines/APIs/pipelines — never hardcoded in
source. The `primary-data` crate loads them (`include_str!` + startup parse + validation).
Primary data also includes **data-as-dependencies** (IANA tz database via `jiff`, point 11).

| File | Contents | Source |
|---|---|---|
| `currencies.toml` | ISO 4217 currencies: code, name, numeric, minor units, countries | Official ISO 4217 via SIX Group, published 2026-09-17 (`[source]` header) |
| `errors.toml` | Registry of every user-facing error/warning BK-code: severity + i18n key | Project-maintained; governor enforces code ↔ registry ↔ locales |
| `events.toml` | Registry of every domain event kind (RULE-EVT-001): owner crate + payload shape | Project-maintained; `xtask govern events` enforces registry ↔ `contracts::kinds` consts and no stray literals in code |
| `defaults.toml` | Fallback/default policies (rates fallback chain, deleted-manual-rates behavior, system-context defaults) | Project-maintained |
| `load-policies.toml` | What loads when; hot/warm/cold tiers; prefetch rules; usage-tracking scope | Project-maintained |
| `perf-budgets.toml` | Performance budgets asserted in CI by criterion benches + `xtask govern perf` | Project-maintained |
| `shortcuts.toml` | Default keyboard-shortcut map: flat entries — `id`, `keys`, `action` (dotted engine action id), `label_i18n_key`, `scope`, `when` guard (RULE-INPUT-001; SSOT, remappable — no hardcoded keys anywhere else) | Project-maintained; every `label_i18n_key` exists in every locale |
| `theme/index.toml` | Theme scope SSOT entry listing the scope's files (RULE-DATA-003) | Bookkeepy v1 design language (Penpot 'draft' page; exported under `references/designlanguage/`) |
| `theme/colors.toml` | Color tokens: raw `[palette]` ramp + semantic tokens (`bg`, `fg`, `accent`, `danger`, `success`, `warning`, `border`, …) for the light and dark themes (dark is the designed native theme) | Bookkeepy v1 design language (Penpot 'draft' page; exported under `references/designlanguage/`) |
| `theme/spacing.toml` | Spacing tokens — multiples of the 4px base grid | Bookkeepy v1 design language (same source) |
| `theme/radii.toml` | Corner-radius tokens (micro → circle/pill) | Bookkeepy v1 design language (same source) |
| `theme/typography.toml` | Font families + named text styles (size/weight/line-height/letter-spacing; amounts tabular) | Bookkeepy v1 design language (same source) |
| `theme/elevation.toml` | Elevation: shadow tokens (black @ 60%, blur/offset scale) + z-index stacking layers (base 0 … tooltip 60, SYSTEM · root sheet) | Bookkeepy v1 design language (same source) |
| `theme/layout.toml` | Fixed shell geometry: sidebar expanded/collapsed widths (ORGANISMS · 01 / TEMPLATES) | Bookkeepy v1 design language (same source) |
| `locales/en.toml`, `locales/ro.toml` | Locale catalogs (flat quoted dotted keys; `[meta].locale` must match the file stem). EN is the reference/fallback locale | Project-maintained; `xtask govern errors` enforces key parity + that every `errors.toml` i18n_key exists in every locale |

Theme tokens are the single source of truth gpui-kit styled components and gallery
variants reference (the "variables" of the Figma model, RULE-UI-005/007, doc 04 §4.9) —
semantic tokens in code, never raw values.

Planned (add when the feature lands, each with a `[source]` header):
`countries.toml` (ISO 3166), `languages.toml` (ISO 639), `category-presets.toml`,
`default-accounts.toml`.

Rules: regenerate external data only from official sources; never hand-edit codes/minor
units; cross-references between primary data use ISO codes (RULE-DATA-002 — e.g. rates
packages must use codes present in `currencies.toml`); schema changes require an ADR;
content changes require a CHANGELOG entry.
| `settings-defaults.toml` | Settings schema + defaults (theme, language, formats, tz, default currency, rates source priority); user overrides live in the profile DB, never in code. Governor validates cross-refs (language↔locales, default_currency↔currencies) | Project-maintained |
