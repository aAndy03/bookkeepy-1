# ADR-0018 — Profiles: fully isolated data roots, switched in-app; export/import is the only bridge

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** owner: the user can create profiles and switch at any time; profiles do not
  share any data and have no shared database — completely separated; the only way to share is
  export certain things → import into the other profile. All features exist per profile.
- **Decision (F18):**
  - A profile = an isolated data root: `<app data dir>/profiles/<id>/{book.db, attachments/,
    rates/packages/, settings, ui_state, jobs, backups/}`. No shared DB, no shared settings.
  - The app-level dir keeps only a tiny registry (id, name, icon, path, last-opened) — app
    metadata, never financial data (doc 08).
  - One process, one active profile (RULE-APP-001); switching closes the current profile
    (its jobs pause/resume inside its own DB) and opens the next. Profile switcher in the app
    shell; create/rename via shared action engines (RULE-SHARED-001) on profile metadata.
  - Bridge between profiles = export (whole book or scoped: accounts/date range/categories/
    explicit selection) → import into the target profile through the import pipeline with
    fresh ids, origin recorded, idempotent per target (doc 05 §5.11).
- **Consequences:** + hard isolation (privacy, mixing-up prevention); − profile switching must
  flush/pause jobs cleanly (handled by the jobs engine per profile DB).
- **Alternatives:** one DB with per-profile schemas — rejected (owner: no shared database).
