# ADR-0010 — Posted entries are immutable; corrections are linked reversals; hash chain

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** fintecheng Immutability + Reversals and corrections (quoted, doc 06 §6.4):
  append-only, no mutating API, tamper evidence via hash chains; fix forward with linked
  compensating entries.
- **Decision:** `postings` are append-only (DB + API level); each row carries prev/hash
  (SHA-256 chain); `engine_audit::verify_chain()` checks it. UI "edit/delete" posts a
  reversal or correction linked both ways; value date vs booking date both stored; reports
  use value date. PII (payees, notes, attachments) lives in separate mutable stores so
  erasure doesn't rewrite history (GDPR adaptation, doc 06).
- **Invariants:** INV-CHAIN-001; rules RULE-ERR/EVT per doc 04.
- **Alternatives:** soft-delete flags on entries — rejected: an editable history proves
  nothing.
