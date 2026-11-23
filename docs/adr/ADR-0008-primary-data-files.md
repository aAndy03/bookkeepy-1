# ADR-0008 — Primary data lives in `data/*.toml` and is imported, never hardcoded

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** user: "There will always be sets of primary data (constants or variables).
  These have to be grouped and maintained as json/toml files that the engines/api/pipeline
  of the app will import. One example: the currencies this app supports will be a list stored
  as primary data."
- **Decision:** primary data files live in `data/` (TOML), each with a `[source]` header;
  the `primary-data` crate loads them via `include_str!`, parses once at startup, validates,
  and exposes typed accessors. Example shipped: `data/currencies.toml` (official ISO 4217,
  SIX Group list, published 2026-09-17, 178 currencies with minor units).
- **Rules:** RULE-DATA-001. Changes to primary data = changelog entry; schema changes =
  new ADR.
- **Alternatives:** hardcoded Rust constants, runtime-downloaded lists — rejected (drift,
  and offline-first forbids downloads).
