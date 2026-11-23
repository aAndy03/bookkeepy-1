# ADR-0009 — The ledger is double-entry; balances are derived, never stored

- **Status:** accepted
- **Date:** 2026-11-24
- **Context:** fintecheng Double-entry bookkeeping (quoted, doc 06 §6.4): books always
  balance; money always has a source and destination; balance is derived; accounts have a
  type; one transaction, many movements.
- **Decision:** `ledger` engine stores `journal_entries` + `postings` (debit/credit account,
  amount, currency). User-level "transactions" (income/expense/transfer, splits, fees) are
  compiled into balanced postings. Accounts typed asset/liability with normal-balance
  semantics; equity/income/expense accounts represent the external world. All balances,
  budgets and reports are read-only projections over postings.
- **Invariants:** INV-LEDGER-001 (every entry balances; trial balance holds for any prefix
  of the journal — property-tested).
- **Alternatives:** single-entry "transactions with from/to" — rejected: cannot represent
  fees/splits correctly or prove conservation.
