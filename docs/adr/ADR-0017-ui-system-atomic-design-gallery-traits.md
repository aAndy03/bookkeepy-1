# ADR-0017 — UI system: Atomic Design + component gallery + capability traits + placeholders

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** owner: build our own gallery featuring **all** gpui-kit styled components by
  default; arrange like the Figma model (variants linked to SSOT — mini-calendar example);
  guard interface design with Brad Frost's Atomic Design; every input needs a placeholder;
  input modalities = mouse + trackpad + keyboard in combination (voice/AI parked); variants
  must be trait toggles on the SSOT component (algebra-capable input vs plain text input),
  tracked by governors so they can't drift.
- **Decision (RULE-UI-005/006/007, RULE-INPUT-001, doc 04 §4.9):**
  - Atomic layers: atoms = wrappers over gpui-kit styled components; molecules, organisms,
    templates, pages composed from them (`ui/src/widgets/{atoms,molecules,organisms}/`,
    `ui/src/views/`).
  - `gallery` binary in the workspace showcases all gpui-kit styled components + our
    variants/edge cases, atomic-arranged.
  - Variants = `variant_*.rs` extending the SSOT source, built **only by enabling/disabling
    registered capability traits** (`ui/src/capabilities/` registry) — e.g. Input's
    `algebra` capability for `+ − × ÷` fields. Governors (`govern ui` + `govern meta`) track
    every trait/variant/placeholder/input-handler in use and fail on drift or missing
    placeholder.
  - Every input/element has a localized placeholder (+ label/helper/error).
  - Shortcuts are primary data (`data/shortcuts.toml`), remappable; mouse/trackpad/keyboard
    combine freely. Voice control parked; when un-parked it must run **on-device** (offline
    promise, ADR-0011) — a cloud model needs a superseding ADR.
- **Consequences:** + consistent, discoverable, accessible UI; variants can't silently diverge;
  − registry discipline required (enforced by governors).
- **Alternatives:** ad-hoc per-use variants — rejected (drift; untrackable).
