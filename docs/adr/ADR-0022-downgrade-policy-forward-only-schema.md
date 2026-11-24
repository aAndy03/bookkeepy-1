# ADR-0022 — Downgrade policy: data schema is forward-only; older apps refuse newer profiles safely

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** INV-DATA-001 says data must survive any sequence of app versions (points
  14–15). A user may install an older Bookkeepy over a profile written by a newer one
  (downgrade). Migrations are one-way by design (they may add columns/indexes a new
  engine needs).
- **Decision (RULE-DATA-004):**
  1. Every profile DB stores `schema_version` (monotonic integer, bumped by migrations).
  2. On open, if `schema_version > app's max known schema` → the app **refuses to open
     that profile** and shows a dialog (never blocks other profiles, never writes):
     "This profile was written by a newer Bookkeepy (schema N). Install a newer version,
     or export this profile from the newer app and import it here." Error
     `BK-DATA-0002` (severity error) + export offer (F17). The data file is never
     modified by the older app.
  3. If `schema_version <= app's max` → open normally and run migrations forward.
  4. Downgrading the app itself is allowed; only the too-new profile is refused.
- **Consequences:** easier — no silent corruption, no reverse migrations to maintain;
  the user always has a path (export from newer app). Harder — a downgraded app can't
  read newer profiles until re-upgraded; acceptable and explicit.
- **Rules/invariants introduced:** RULE-DATA-004; error code BK-DATA-0002.
- **Alternatives considered:** reverse migrations (rejected — doubles migration surface,
  error-prone over 20 years); opening read-only (rejected — user must be able to fix
  things, and read-only ledger is a lie for a bookkeeping app).
