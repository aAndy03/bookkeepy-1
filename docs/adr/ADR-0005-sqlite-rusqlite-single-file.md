# ADR-0005 — Storage: one user-owned SQLite file via rusqlite (bundled)

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** user: "we can safely use rusqlite for all things database related"; "the data
  is of the user".
- **Decision:** all persistence goes through the `storage` crate on `rusqlite` (feature
  `bundled`). One file per book, WAL, foreign keys ON, numbered migrations, `VACUUM INTO`
  backups, hash-chained append-only journal tables (doc 05). The file lives in a
  user-chosen folder; the app never locks it exclusively.
- **Consequences:** + portable, inspectable, user-owned data; no server. − concurrency limited
  to one writer (fine: single-user desktop app; serialized via a single connection mutex).
- **Alternatives:** embedded key-value store, custom format — rejected: SQL gives us
  constraints, migrations, reconciliation queries and user trust ("it's just a file I can
  open").
