# ADR-0011 — Network policy v2: updates + rates mirror only (supersedes ADR-0003)

- **Status:** accepted (supersedes ADR-0003)
- **Date:** 2026-11-24
- **Context:** the app stays mainly offline, but now also consumes **live FX rates** — yet
  must never connect to rate sources directly. Correction to the original policy: live rates
  are in scope, via a mirror.
- **Decision:** exactly two HTTP allowances, in two crates with one client each
  (`ureq`+rustls):
  1. `updater` → GitHub Releases of `aAndy03/bookkeepy-1` (app updates, SHA-256 verified,
     user-approved).
  2. `rates` → `aAndy03/bookkeepy-1-rates` only: `manifest.json` + package files
     (SHA-256 verified). **Never** BNR or any other source directly.
  RULE-NET-001 is enforced mechanically: `xtask govern deps` fails on any other HTTP
  dependency; `govern net` fails on hardcoded hosts outside the allowlist; CI runs an
  offline smoke test. No telemetry/analytics/crash-upload — ever.
- **Consequences:** + live rates without exposing users' machines to arbitrary providers;
  the mirror is auditable and pinnable (checksums, later signatures). − one more repo to
  operate.
- **Alternatives:** app fetches BNR directly — rejected (user requirement: app connects only
  to the sub-repo). No live rates at all — rejected (user requirement).
