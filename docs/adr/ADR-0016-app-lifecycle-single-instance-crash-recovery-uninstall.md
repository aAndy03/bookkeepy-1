# ADR-0016 — App lifecycle: single instance, crash recovery, uninstall never deletes data

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** owner decisions (brainstorming round 2, items 1–3).
- **Decision:**
  - **Single instance (RULE-APP-001, INV-APP-002):** one app process per book. A second
    launch sends an IPC "focus" message and focuses the existing window. The instance lock
    carries PID + heartbeat; a stale lock after a crash is taken over with a notice.
  - **Crash recovery:** WAL + single-write transactions mean no torn writes. On open the
    app starts immediately and runs a background self-check job (never blocking,
    RULE-UX-001): `quick_check`, hash-chain verify, invariant guards; results surfaced in
    Diagnostics; drafts from `ui_state` restored with a "Restore draft?" affordance.
  - **Uninstall (INV-APP-001):** uninstall removes only the app binary/installer files —
    never the data folder. The uninstaller offers an **unchecked-by-default** checkbox
    "also delete my data", preceded by a full-export offer whose text explains: the data
    stays on this device; reinstalling finds it unaltered; the export allows use in other
    apps. Reinstall/upgrade likewise never alters data (INV-DATA-001, doc 05).
- **Consequences:** + predictable, safe behavior; − IPC/lock code per OS (small, isolated in
  the `app` crate / platform layer).
- **Alternatives:** multi-instance with read-only second windows — rejected (complex locking
  for no user benefit); silent data deletion — rejected.
