# ADR-0020 — Recurrence: bounded lazy expansion; past occurrences reconciled by default

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** owner: recurring income/expense/transfer — recurring must apply to most
  things, even with past or future dates. The UX/engine must have limits (never render until
  2099). Example: in 2026 the user adds a monthly recurrence that started in 2010 — the app
  must NOT make them mark each occurrence; by default all past occurrences are assumed
  reconciled. Only if the user explicitly selects not-completed ones must the engines/UI
  support that.
- **Decision (RULE-RECUR-001…003, doc 05 §5.13):**
  1. A schedule is a rule (frequency/interval/start/end/count/exceptions), not stored rows;
     occurrences are derived projections expanded lazily within the visible timeline scope +
     buffer, capped per expansion. UI never renders unbounded ranges.
  2. Past-start schedules: all past occurrences default to `reconciled`; the materialization
     driver does not backfill history — it posts only near-now/future occurrences (future =
     forecast). The user can explicitly materialize a bounded past range (job with progress),
     or flip selected occurrences to `pending` via a bounded selection UI.
  3. Occurrences become real, audited transactions only on materialization/confirmation;
     edits to them are linked corrections; exceptions are stored on the schedule, linked
     both ways.
- **Consequences:** + no 16-year backfill, no prompting storms, fast UI at any zoom; history is
  opt-in and bounded. − "assumed reconciled" is an assumption the user can override per
  occurrence (audit records the override).
- **Alternatives:** eager backfill of all past occurrences — rejected (16 years of postings,
  prompting fatigue); per-occurrence status prompts — rejected.
