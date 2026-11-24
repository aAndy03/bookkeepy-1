# ADR-0024 — Lifecycle & soft-delete semantics (there is only soft delete, in two forms)

- **Status:** accepted
- **Date:** 2026-11-24
- **Deciders:** Andy (product) + Vibe (engineering)
- **Context:** the product rule is "only soft delete exists — as archive or as status; a
  deleted status means the thing is excluded from computed metrics". That is right, but it
  needs one nuance to stay provable: posted money must never be excluded by a flag filter,
  because the double-entry conservation proof (INV-LEDGER-001) only holds if every posted
  row counts. So "deleted" has two different mechanics depending on what the thing is.
- **Decision:** (1) **Catalog entities** (accounts, categories, tags, parties) have
  `Lifecycle::Archived` — excluded from pickers/new entries and from FORWARD-looking
  projections (budgets, forecasts, recurring templates), but kept in history (old postings
  still reference them; historical reports include them). Archiving never touches money.
  (2) **Transactions** have `TxStatus` — `pending/cleared/reconciled` are metadata
  (reconciled affects reconciliation reports only, never balances); `voided` = "deleted":
  the ledger posts a LINKED REVERSAL entry in the same audited transaction, so the voided
  transaction is excluded from every computed money metric **because the journal itself
  nets it out** — never by filtering rows out of a report. The row and entry stay forever
  (audit, ADR-0010). (3) Non-posted items (scheduled/planned transactions, budgets,
  reminders) are excluded from projections directly by their status — they have no
  postings yet, so no conservation concern.
  SSOT: `bookkeepy-ledger`'s `api::lifecycle` module (mirrored in doc 05 §5.4).
- **Consequences:** reports never need "hide deleted" filters for posted money — balances
  are computed over the raw journal and reversals do the work; the trial balance holds for
  any prefix (property-tested). UI pickers filter `archived=0`. Hard delete exists only for
  non-audited data (drafts, ui_state), never for catalog/ledger rows.
- **Rules/invariants introduced:** RULE-LIFE-001 (soft-delete-only), INV-LEDGER-001 already
  covers the conservation side. Registered in docs/04-conventions.md.
- **Alternatives considered:** filtering posted rows by a `deleted` flag in every query —
  rejected: it breaks the conservation proof and the hash chain's meaning; also `DELETE`
  rows — rejected by ADR-0010 (immutability).
