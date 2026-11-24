# ADR-0015 — Central jobs orchestration engine for all async work

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** the app is primarily async (RULE-ASYNC-001). gpui-kit offers async primitives,
  but using them inline per call site would repeat retry/progress/cancel logic everywhere.
- **Decision (RULE-JOB-001, doc 04 §4.8):** all long/failing async operations are Jobs
  managed by one `jobs` crate (`engine_jobs` + `broker_jobs`). Jobs are durable state machines
  in the `jobs` table (queued/running/paused/done/failed/cancelled, progress, attempts,
  payload, result, BK error code). Auto-retry with exponential backoff up to `max_attempts`
  (policy in `data/defaults.toml [jobs]`), then user-surfaced manual retry. Progress events
  drive one shared UI indicator with pause/resume/cancel. A startup driver resumes
  interrupted jobs (full resumability). Job types are persisted pipelines (exact resume).
  UI: single non-modal Jobs drawer (RULE-UX-001).
- **Consequences:** + uniform, testable behavior (retry/resume/cancel injected in tests);
  − one more crate and table; mitigated by making every job a declarative pipeline.
- **Alternatives:** inline `cx.spawn` per feature — rejected (drift, repeated logic).
