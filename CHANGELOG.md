# Changelog — Bookkeepy 1

Keep a Changelog format. Every PR adds an entry under `Unreleased` (doc 09); releases
move it under the version tag. The release workflow fails if a release section is missing.

## [Unreleased]

### Added
- M0.5 foundation crates: `build-info` (build facts + channel + embedded GPL text),
  `contracts` (EventEnvelope/Actor), `bus`, `diagnostics` (bookkeepy-diagnostic@1 +
  panic hook), `logging` (local rotating logs), `profile` (registry + data-dir layout),
  `i18n` (EN/RO catalogs with enforced parity).
- Primary data: `locales/en.toml`, `locales/ro.toml`, `theme/*.toml` (colors, spacing,
  radii, typography, elevation + index), `shortcuts.toml` (default keymap SSOT).
- Error registry: BK-APP-0001, BK-LOG-0001, BK-PROFILE-0001, BK-DATA-0002, BK-RATES-0002.
- Supply chain: `deny.toml` (cargo-deny 0.22 schema), Renovate config, SPDX headers on
  all Rust sources (`govern lints` enforces).
- Tooling: `cargo xtask new-crate` / `new-adr` / `doctor` / `licenses` / `check` /
  `hook`; pre-commit hook; `.gitattributes` (LF); `.editorconfig`.
- Code registry SSOT: `docs/registry/code-map.toml` (`govern meta`, RULE-META-001).
- Channels: dev/release via `BOOKKEEPY_CHANNEL` + `dev-channel` cargo feature
  (ADR-0021, RULE-CHANNEL-001); downgrade policy forward-only schema (ADR-0022,
  RULE-DATA-004, BK-DATA-0002); rates packages ed25519-signed (ADR-0023, BK-RATES-0002).
- CI: minutes-budget policy — Actions run only on releases/prereleases
  (`release.yml`) and manual `workflow_dispatch`; local gate `cargo xtask check`.
- Community files: SECURITY.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md, PR/issue templates,
  CODEOWNERS; tag protection for `v*`.

## [0.1.0] — not released yet
