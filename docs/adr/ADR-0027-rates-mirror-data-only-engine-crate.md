# ADR-0027 — Rates mirror stays data-only; consumption engine is a crate in this repo; scale via archive rollups

- **Status:** accepted
- **Date:** 2026-11-27
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** `aAndy03/bookkeepy-1-rates` publishes one immutable daily package per banking day (~3.8 KB each, ~250/year). Long-term concern: file-count growth and discoverability ("chaos" of many daily files vs. few archives), plus a proposal to vendor a Rust engine/API into the rates repo so third parties can copy it into their own codebases.
- **Decision:**
  1. The rates repo stays **data-only**: `manifest.json` + immutable packages + sha256 + ed25519 signatures + producer tooling. No consumer engine code is vendored there.
  2. The consumption engine is a **Rust crate in this repo** (`crates/`), publishable to crates.io. Third parties use it via `cargo add` or by copying the crate folder. It verifies sha256 + ed25519, pins `spec_version`, and refuses unknown spec versions loudly (per `bookkeepy-rates-package@1`).
  3. File-count scaling is handled at the **data layer**: a periodic rollup job publishes new yearly `kind: "archive"` packages and/or addenda. Existing packages are never rewritten — the manifest stays the small, human-readable index consumers actually fetch.
  4. Optional future: the engine crate may define a single-file bundle format (one packed artifact with an index) for consumers who want to ship exactly one file.
  5. The rates repo license changes GPL-3.0 → MIT: the data is BNR's, not ours.
- **Consequences:** the rates repo remains a pure trust anchor; parse/verify logic has exactly one home (the engine crate), so no drift between vendored copies; consumers never see file count behind the manifest; GPL copyleft no longer attaches to people reusing the tooling.
- **Rules/invariants introduced:** packages immutable (existing spec rule, reaffirmed); engine crate is the single consumer implementation; rates repo contains no consumer code. (Proposed for registration in `docs/04-conventions.md`.)
- **Alternatives considered:** (a) daily files forever — fine size-wise (~1 MB/year) but rejected for long-term tidiness; (b) the scheduled action rewriting `archive-YYYY.json` in place — rejected, violates immutability and breaks sha256/signature verification for anyone who already fetched the package; (c) vendoring the engine in the rates repo — rejected, two homes for the verify logic, stale-until-used code, GPL inheritance for tinkerers.

ADRs are immutable once accepted. To change a decision, write a new ADR that supersedes.
