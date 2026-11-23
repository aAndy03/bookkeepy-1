# ADR-0006 — Search: 100% tantivy API, no custom indexing engine

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** user: "We shall not build an indexing and searching engine. Rather we shall
  use 100% of tantivy search engine API."
- **Decision:** the `search` crate wraps tantivy only. Index = derived cache next to the
  SQLite file; write-through from storage events; fully rebuildable from the ledger
  (`engine_search::rebuild()`). Tantivy is never authoritative — the DB is.
- **Consequences:** + battle-tested full-text/relevance; − index can drift (mitigated: rebuild
  + verify in diagnostics). Any schema change of the index is versioned and rebuilt, not
  migrated.
- **Alternatives:** hand-rolled inverted index, SQLite FTS5 — rejected by user decision;
  FTS5 may be a *fallback* if tantivy proves unsuitable, via a superseding ADR.
