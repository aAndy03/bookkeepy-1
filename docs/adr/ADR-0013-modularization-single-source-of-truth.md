# ADR-0013 — Code organization: modular files/folders + single source of truth + adjacent variants

- **Status:** accepted (amends doc 04 §4.2)
- **Date:** 2026-11-24
- **Context:** the first convention draft crammed a crate's concerns into a few files. The
  owner requires modularization and orchestration as principles (points 3, 3.1–3.4, 9).
- **Decision (RULE-MOD-001, RULE-SSOT-001):**
  - Folders group and nest; file per responsibility; no artificial line cap, but files stay
    small and reusable — split proactively as they grow (3.1).
  - Variants and edge cases are coded in close proximity to the thing they vary, as small
    extending files (`variant_*.rs`, `edge_*.rs`) (3.2) — including UI components (3.2.1).
  - Every concept has exactly one defining file (single source of truth); everything else
    imports and links to it (3.4). This applies to backend and frontend alike; the Figma model
    (components, variables, properties, variants linked to source) is the mind model for the
    UI (9).
  - Engines are specialized by role: domain `engine_*`, helpers `helper_*`, guards
    `guard_*`, brokers `broker_*` (3.3).
- **Consequences:** + easy import/reuse of specific logic; refactors touch one small file;
  − more files to navigate (mitigated by `mod.rs` re-exports and the docs index).
- **Alternatives:** few large files per crate — rejected by owner.
