# ADR-0021 — Dev channel vs release channel (same code, different build stamp)

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** owner wants dev builds to carry extra tooling (performance monitoring
  overlays, debug screens, extra logging) that must never appear in the production app
  users install from releases. Both channels run the same code and produce the same app.
- **Decision (RULE-CHANNEL-001):**
  1. The channel is fixed at **build time** via `BOOKKEEPY_CHANNEL=dev|release` (default
     `release`), embedded by `bookkeepy-build-info` as `CHANNEL` / `is_dev()`. The About
     screen and every diagnostic bundle show the channel.
  2. Dev-only functionality must be **compile-time gated**: crates that need it declare a
     `dev-channel` cargo feature (aggregated by the app crate); `cargo xtask dev` builds
     with `BOOKKEEPY_CHANNEL=dev` and the feature enabled. Release builds (`release.yml`)
     build without it — dev-only code is not even present in shipped binaries.
  3. The updater never crosses channels: release builds update only from published
     releases of the main repo; dev builds never auto-update (they are built from source).
     A dev build opened on release data works normally (same schema), and vice versa.
- **Consequences:** easier — one codebase, honest About/diagnostics, no accidental leakage
  of debug tooling into production. Harder — every dev-only surface needs the feature
  gate; `govern lints` checks that `cfg(feature = "dev-channel")` blocks exist only where
  the registry (docs/registry/code-map.toml) marks a module as dev-only.
- **Rules/invariants introduced:** RULE-CHANNEL-001.
- **Alternatives considered:** runtime flag in settings (rejected — users could toggle dev
  tooling into "production" builds and support would see phantom reports); separate
  dev repo (rejected — violates single-source-of-truth).
