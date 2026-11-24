# ADR-0003 — Offline-first: the only network use is downloading app updates

- **Status:** superseded by ADR-0011 (network scope widened: updates + rates mirror)
- **Date:** 2026-11-24
- **Context:** "Bookkeepy is mainly offline. The only internet connection is used to
  download app updates and other things that we will define as we go."
- **Decision:** no network code anywhere except the `updater` crate with one HTTP client.
  Updates are fetched only from this repo's GitHub Releases, only on user action (or the
  documented default-off weekly check), and verified by SHA-256 before install. No
  telemetry, no bank APIs, no live FX, no cloud.
- **Rules:** RULE-NET-001 (enforced by `cargo xtask check-deps` + CI offline smoke test).
- **Consequences:** + trust and privacy; − no live features. Any future online feature
  needs a new ADR amending this one and a settings toggle defaulting to off.
- **Alternatives:** "optional sync" — deferred, would need its own ADR.
