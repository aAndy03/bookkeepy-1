# ADR-0001 — Monorepo (single repo, Cargo workspace), not multirepo

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** "This codebase will become a multirepo; if this is not suited, then a
  monorepo." The crates share domain types (Money, Currency, ids, events) and need atomic
  cross-crate refactors constantly.
- **Decision:** one GitHub repository (`aAndy03/bookkeepy-1`), one Cargo workspace with
  clearly owned crates (doc 03 layout). Crate boundaries provide the ownership clarity a
  multirepo would; splitting repos later is possible once crates stabilize.
- **Revisit criteria:** split a crate into its own repo only when it (a) has a stable public
  API, (b) has a second consumer outside this workspace, (c) has its own release cadence.
- **Consequences:** + atomic refactors, single CI, single changelog; − bigger clone. Accepted.
- **Rules:** RULE-ARCH-001 (dependency direction), RULE-ARCH-002 (crate anatomy).
- **Alternatives:** multirepo now — rejected: versioning hell and cross-repo PR cascades for
  a young codebase.
