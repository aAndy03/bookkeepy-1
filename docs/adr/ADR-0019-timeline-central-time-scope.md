# ADR-0019 — The timeline is the central time scope: one SSOT engine drives all bound components

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** owner: time is core to managing money; past/present/future entries; the
  timeline is the main component tying all screens together; views by day/week/month/
  trimester/year; focus a day or period; centralized filters (per-component filters would
  drift — "one stuck at 60 days, another at 30"); compare mode (day vs period, period vs
  period) with components aware via traits/variants.
- **Decision (RULE-TL-001, F19):**
  - `engine_timeline` (crate `timeline`) owns the single `TimeScope` state: view granularity,
    focus range, optional compare scope, filters (accounts/categories/tags/parties/text),
    include-future toggle. Persisted in `ui_state`; broadcast as `TimeScopeChanged` on the bus.
  - Components opt in via the `timeline` capability trait; presenters subscribe and
    parameterize every query by the scope. **No component owns its own period state.**
  - Compare mode sets a second scope; components with the `compare` trait render comparison
    variants and compute diffs (queries run per scope).
  - Future scope shows scheduled/recurring entries as forecast projections (never posted until
    materialized); past scope uses value dates (fintecheng value vs booking time).
  - `govern ui`/`govern meta` fail on views keeping local period state outside the timeline
    engine.
- **Consequences:** + one place to change the period for the whole app; consistent compare;
  − components must be disciplined (enforced by governors).
- **Alternatives:** per-component filters — rejected (drift, exactly as the owner said).
