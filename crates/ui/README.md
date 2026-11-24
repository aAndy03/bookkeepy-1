# bookkeepy-ui

The ONE component set (ADR-0017, doc 04 §4.9): gpui-kit 0.11.0 (`gpui-fast`) styled
components arranged by Atomic Design, with Bookkeepy variants = capability-trait
toggles linked to their single source of truth. The `gallery` workspace binary is the
harness/viewer over this crate — never a second copy (RULE-UI-006).

## What this crate owns

- **Theme bridge** (`src/theme.rs`): the `data/theme/` design tokens (loaded via
  `bookkeepy-primary-data`) installed into gpui-kit's `Theme` — the only place data-theme
  tokens meet the gpui-kit theme seam. Dark is the design language's native theme; light
  keeps the same accent + functional hues. `theme::apply(mode, cx)` switches in one step.
- **Capability-trait registry** (`src/capabilities.rs`, RULE-UI-007): the SSOT list
  (`algebra`, `signed`, `selectable`, `timeline`, `compare`, `menu`). Variants are built
  ONLY by enabling/disabling registered traits on the source component; widget
  `meta.toml` traits are validated against this list by `cargo xtask govern ui`.
- **Widget layers** (`src/widgets/{atoms,molecules,organisms}/`, RULE-UI-008): each
  widget folder is `source.rs` (the SSOT) + `variant_*.rs` (trait toggles extending it) +
  `edge_case_*.rs` + `meta.toml` (id, version, traits, variants, edge cases, changelog —
  the gallery renders this as the widget's version history). Seed widget:
  `atoms/amount/` — Money display per the `amount` typography token, `signed` variant,
  muted zero edge case.

## Invariants

- No hardcoded colors/fonts/sizes — tokens come from `data/theme/`
  (`bookkeepy-primary-data::theme`), values via gpui-kit `ActiveTheme`.
- Strings via `data/locales` (`bookkeepy-i18n`) — never literals; every input carries a
  localized placeholder (RULE-UI-005).
- The public surface is `api/` only (RULE-SSOT-001, RULE-UI-004): the gallery and the
  app compose through it.
- `#![forbid(unsafe_code)]`; no unwrap/expect outside tests (RULE-ERR-002).

## ADR / docs

ADR-0017 (UI system), ADR-0007 (gpui-kit stack); doc 04 §4.9 (UI conventions),
doc 03 (architecture: `… ← app-services ← ui ← app` — this crate may talk to
app-services' api and the data layers, never storage/engines directly).
