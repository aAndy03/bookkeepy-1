# bookkeepy-gallery

The component gallery binary (RULE-UI-006/008, ADR-0017): every interface component is
built and approved HERE first, then used in the app. It is a harness/viewer over the
same `bookkeepy-ui` widgets crate — ONE component set only, never a second copy.

Run it with `cargo run -p bookkeepy-gallery --bin gallery`.

## What it shows

- **ALL published gpui-kit styled components** (gpui-kit.com/component/), arranged by
  atomic layers — the sidebar groups: atoms, molecules, organisms, viz, ours. The
  section registry is `src/sections/mod.rs`; the component inventory it must cover is
  `docs/registry/gpui-kit-components.toml` (cross-checked by `cargo xtask govern ui`;
  only documented exceptions are allowed: root, speech, theme).
- **Ours:** Bookkeepy's versioned widgets (`crates/ui`) with their capability-trait
  variants, edge cases, and the version history rendered from each widget's
  `meta.toml` (RULE-UI-008). Seed: the `amount` atom and its `signed` variant.
- **Shell:** title bar with EN/RO locale + dark/light theme switches (the design
  language's tokens via `bookkeepy-ui::theme`), sections lazily created and retained.

## Invariants

- No hardcoded strings: every label comes from `data/locales` (`bookkeepy-i18n`,
  EN/RO at exact parity — enforced); every input demo carries a localized placeholder
  (RULE-UI-005).
- No hardcoded shortcut keys: the quit binding is read from `data/shortcuts.toml`
  (RULE-INPUT-001).
- No hardcoded colors: gpui-kit `ActiveTheme` seeded from `data/theme` tokens.
- Demo-only: the gallery talks to `bookkeepy-ui` + the primary-data layers, never to
  app-services/storage/engines.

## ADR / docs

ADR-0017 (UI system: Atomic Design + gallery + capability traits), ADR-0007 (gpui-kit
stack); doc 04 §4.9; doc 10 "M3 UI".
