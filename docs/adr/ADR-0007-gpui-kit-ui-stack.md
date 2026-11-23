# ADR-0007 — UI: gpui-kit 0.11.0 with `gpui-fast`, all styled components

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** user: gpui-kit 0.11.0 with the `gpui-fast` feature; "we shall use ALL the
  library's styled components" from the component catalog.
- **Decision:** the single UI dependency is
  `gpui-kit = { version = "0.11.0", features = ["gpui-fast"] }`. The interface is built
  catalog-first from gpui-kit's styled components (https://gpui-kit.com/component/); the
  unstyled `base` layer only where the catalog lacks an equivalent (documented per use).
  Component usage follows the vendored skills (`skills/gpui-kit/`,
  `skills/gpui-kit-design-guides/`). UI talks only to `app-services::api` (RULE-UI-004).
- **Consequences:** + fast, consistent, themable UI; − tight coupling to gpui-kit releases
  (mitigated: pinned version, upgrade PRs with ADR notes).
- **Alternatives:** egui/iced/tauri — rejected by user decision.
