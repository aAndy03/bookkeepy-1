# ADR-0014 — Audit log vs app history vs undo/redo scope

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** owner: any bookkeeping needs an audit log; if it can be tampered with it has
  no value. Undo/redo must only affect non-essential, non-audited things (input boxes etc.).
  The audit differs from the app's usage history (a form of tracking), which the user may
  configure (keep / auto-delete at a frequency / off).
- **Decision (RULE-AUD-001, RULE-UNDO-001, RULE-UX-001):**
  - `audit_log`: append-only, hash-chained (SHA-256), tamper-evident; records what/when/
    who-what/why for every posted change. **Never editable or deletable through the app.**
    User erasure happens outside the app (delete the data file); PII lives in separate
    mutable stores (crypto-shredding option, doc 05 §5.8).
  - `app_history`: UI-level action tracking, separate table, user-configured retention
    (keep / every N days / on exit / off), clearable anytime; never money truth.
  - Undo/redo operates only on non-audited state (drafts, inputs, filters, UI state). Posted
    changes are changed only via linked corrections/reversals, which post new audit events.
    Example: record transaction → audit event A; edit it → correction posts audit event B.
  - Never-block principle: reminders (e.g. backups due) are dismissible; the app never
    obstructs user actions because a background task hasn't completed.
- **Consequences:** + trustworthy books, clear user control over mere tracking; − two stores to
  keep distinct (guard engine enforces the boundary; `govern errors`/tests check it).
- **Alternatives:** undo across posted entries — rejected (would falsify the audit).
