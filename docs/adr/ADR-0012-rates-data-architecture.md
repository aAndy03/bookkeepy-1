# ADR-0012 — Rates data architecture: mirror sub-repo, additive sources, manual overrides, defaults

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** user requirements (points 2, 2.1–2.4): live rates via a sub-repo the app
  downloads packages from; manual rates and manual adjustments of live rates; deletable
  manual data is irrecoverable → explicit defaults; rates linked to `currencies.toml`;
  many sources, user chooses primary; switching source keeps history; sources accumulate,
  user picks which rate to use.
- **Decision:**
  - Sub-repo `aAndy03/bookkeepy-1-rates` publishes immutable packages + `manifest.json`
    (spec SSOT: its `docs/package-format.md`, `bookkeepy-rates-package@1`; SHA-256;
    incremental per source/date; addendum for corrections; BNR first source).
  - App stores packages read-only and imports rows into `fx_rates` with
    `origin = live | manual`, `source`, `package_id`, `date`, rate (string), hash (doc 05 §5.3).
  - **2.1** every currency code validated against `data/currencies.toml` (RULE-DATA-002).
  - **2.3** switching primary source affects only future lookups/defaults; historical rows
    keep their source's rate; only manual update changes them.
  - **2.4** packages from all sources accumulate; UI lets the user choose the rate per
    conversion/report.
  - Manual rows are deletable and **irrecoverable**; fallback chain from `data/defaults.toml`
    (primary-source latest ≤ value date → any-source ≤ value date → nearest earlier →
    BK-FX-0002 error).
- **Consequences:** + auditable, offline-capable, user-controlled rates; − more moving parts
  (mirror automation), mitigated by checksums + spec pinning.
- **Alternatives:** direct source fetching in-app (rejected, ADR-0011); storing only manual
  rates (rejected by user).
