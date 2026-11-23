# 06 — Fintech principles for Bookkeepy 1 (quoted from `references/fintecheng.txt`)

Source: *Fintech Engineering Handbook — Patterns for building software that handles money*,
Voytek Pitula (vendored verbatim at `references/fintecheng.txt`). **This guide describes more
than Bookkeepy needs** — Bookkeepy never does live transactions or anything over the internet.
Below, each relevant passage is **quoted** and then **adapted** to our offline, single-user,
desktop reality. When a principle is integrated into an engine/API/pipeline, the quote is the
authority and the adaptation here is the policy.

## 6.1 The three principles (verbatim)

> Everything you will read below is a way to adhere to the three principles:
> 1. No invented data. Money can’t be created out of nowhere, so we can’t tolerate duplicates
> or arbitrary balance updates. We enforce this with idempotency, deduplication, and recon-
> ciliation.
> 2. No lost data. Everything that happens to money has to be tracked and persisted. We
> protect this with full precision, at-least-once deliveries, event sourcing, audit trails, and
> immutability.
> 3. No trust. Trust neither external providers, internal components, nor the world. We uphold
> this by verifying webhooks, cross-checking data across sources, and failing loudly on
> broken assumptions.

**Adaptation (offline):** "external providers" becomes *imported files and the user
themselves*; "webhooks" becomes *imported statements*; "at-least-once deliveries" becomes
*resumable, idempotent import/schedule pipelines*; "failing loudly" becomes loud local
errors + audit log. All three principles apply in full.

## 6.2 Representing money

> In practice this means storing the amount as an integer in its smallest unit - €12.34
> becomes 1234. … don’t assume it’s always 2, it’s not!
> … Send money either as a string ("12.34") or as an integer in its smallest unit.

**Adaptation:** `Money { amount_minor: i64, currency }` (INV-MONEY-001, ADR-0004);
minor units from `data/currencies.toml` (ISO 4217); no floats anywhere in money code; money
serializes as string/minor-units integer only (doc 05 §5.1).

> Rounding is inevitable. It should be done explicitly … It’s a business decision … Round as
> seldom as possible … Rounding breaks sums … e.g. an explicit rounding account.

**Adaptation:** all division/conversion/fee math keeps full precision internally and rounds
once at the boundary (persist/display); rounding mode is a per-book setting (half-even
default); residuals post to an explicit rounding account — never dropped, never minted
(doc 05 §5.1).

> Pack amount and currency together … No cross-currency arithmetic … Use a controlled
> currency set … Never accept arbitrary currency codes; validate at the boundaries of the
> system. … Currencies carry metadata. Symbol, precision, name, etc.

**Adaptation:** `Money` newtype pairs amount+currency; the controlled set is
`data/currencies.toml` validated at every boundary (RULE-VAL-001); metadata (symbol,
precision, name) drives display only (doc 05 §5.1, RULE-DATA-001).

## 6.3 FX rates

> A rate is always directional … The time of the rate is critical … Transactional rate …
> Reference rate (mid-market or central bank) … There is no canonical rate … the source
> should be part of the data.

**Adaptation (offline):** Bookkeepy never fetches rates. The user records reference rates
manually (rate, date, source note) in `fx_rates`; transactional rates fall out of
recorded conversions. Valuation uses the latest stored rate ≤ value date; every stored
conversion keeps the rate id + source. No live FX, ever (doc 05 §5.3, ADR-0003).

## 6.4 Recording money: the ledger

> Double-entry bookkeeping … every entry moves the same amount out of one account and into
> another, the books always balance - money is only moved, never created or destroyed.
> • Money always has a source and a destination. External providers get dedicated accounts
> too … • Balance is never stored. It’s derived from the movements of money.
> • Accounts have a type … • One transaction, many movements … • Posted entries are
> immutable. By convention, corrections are made by adding new compensating entries that
> offset the original.

**Adaptation:** the `ledger` engine implements exactly this (ADR-0009). "External world" =
dedicated income/expense/transfer accounts. Splits and fees are extra postings. Balances,
budgets, reports are derived projections (doc 03 data flow).

> Value time vs booking time vs settlement time … Value time. When the transaction occurred.
> Booking time. When the transaction was recorded in the system. … Business and
> business-consumed reports usually care about value or settlement time, while booking time
> is useful for traceability.

**Adaptation:** every entry stores `value_ts` (user-editable, reports use it) and
`booked_ts` (system, traceability). No settlement time (offline, no rails) — pending/cleared
status covers the user's mental model (doc 05 §5.4).

> A useful audit trail captures, for every change: What happened. When it happened … Who or
> what triggered it - a user, an operator, an automated job. Why it happened - a reference
> to the order, instruction, or incident that caused it. … configuration changes … need
> trails too.

**Adaptation:** `audit_log` records what/when/actor (user, import batch id, scheduler job,
app upgrade)/why-reference for money *and* config changes (doc 05 §5.4, RULE-EVT-001).

> Event sourcing … the double-entry ledger is an example of this pattern applied to money -
> balance is never stored, it is calculated from the stored entries … Plan for schema
> evolution. Events live for years, so today’s code must still read events written long ago.

**Adaptation:** the journal is our event store for money; other domains use a conventional
model + the audit log (fintecheng: "You don’t need it everywhere"). A backward-compatibility
corpus of old DB formats/payloads is kept and tested (doc 09).

> Immutability is an invariant … 1. By construction. Append-only tables, revoking
> UPDATE/DELETE at the database-permission level. 2. Runtime checks. The application layer
> exposes no mutating operations on posted records. 3. Post-factum. Tamper evidence:
> checksums or hash chains over the records, periodically verified …

**Adaptation:** append-only `postings` (DB-level + API-level) + SHA-256 hash chain verified
by `engine_audit::verify_chain()` (doc 05 §5.5).

> Reversals and corrections … 1. Reversal. Negates the original in full … 2. Correction
> (adjustment). Books the difference … link it to the record it corrects, in both directions.

**Adaptation:** the UI exposes "edit"/"delete" only as posting a linked reversal/correction;
the original stays visible in history (ADR-0010).

> Immutability vs GDPR … 2. Separate PII from financial data … PII (names, addresses,
> documents) lives in a separate, mutable store that can be redacted or erased independently.
> 3. Crypto-shredding for embedded PII … erase by deleting the key.

**Adaptation:** the user owns the whole file, so erasure = deleting the file; inside the
app, payee names/notes/attachments (PII) live in mutable tables separate from the immutable
journal, and optional encryption uses per-record keys so erasure is key deletion
(doc 05 §5.8).

## 6.5 Executing money flows (offline: imports & schedules)

> Invariants … 1. By construction … 2. Runtime checks … property-based testing shines here
> (e.g. “for any sequence of postings, the books balance”). 3. Post-factum …

**Adaptation:** invariants INV-LEDGER-001 (debits = credits per entry and in total),
INV-MONEY-001, INV-CHAIN-001 (hash chain verifies) are checked by construction + property
tests + post-factum verify (doc 09).

> Handling overdrafts … "balance >= 0" is just an invariant … don’t force it by construction.
> When an overdraft is detected, it’s a signal to investigate …

**Adaptation:** negative balances are representable; the app warns but never clamps
(doc 05 §5.4).

> Idempotency … 1. Prefer explicit keys … scoped to a particular operation and client …
> 6. Test for retries … automatically repeats every call.

**Adaptation:** import pipeline dedups on explicit (batch, source-row) keys; tests replay
whole imports twice and assert zero double-postings (doc 05 §5.6, doc 09).

> Full resumability … Persist progress, don’t keep it in memory … Something must resume
> stalled flows … Every step must be safe to re-run …

**Adaptation:** multi-step flows (import, scheduled-transaction materialization, backup)
are durable state machines in the DB; on startup a driver resumes stalled pipelines
(RULE: `pipeline_*` staged + persisted progress, doc 04 §4.2).

## 6.6 The external world → reconciliation (offline adaptation)

> Reconciliation … Usually you want to persist the external provider id within your system
> so that matching is straightforward … Each discrepancy found should be understood and fixed
> through first-class support, e.g. a correction record …

**Adaptation:** "external provider" = the bank's statement file. Imported statement rows
persisted with their external id enable exact matching; unmatched rows surface as
reconciliation breaks fixed via correction entries, never by overwriting (doc 05 §5.6).
Not adopted (online-only): consuming APIs, webhooks, outbox/CDC, payment rails, KYC/AML,
custody, trading.

## 6.7 Controls and access

> Segregation of duties and four-eyes … It applies to engineering too. Merging code,
> deploying to production … usually require review and approval. … The approval is part of
> the trail.
> Access control … Least privilege … Authorization changes need a trail.
> The change trail (SDLC) … Source control is the record … Reviews and pipelines must be
> enforced. Required reviews, status checks, and “no direct pushes to main” …

**Adaptation:** Bookkeepy is single-user, so four-eyes on money actions is N/A; the control
moves to *engineering*: protected `main`, required PR reviews, required CI, signed-off
releases (doc 09/10). Local access control = the optional app lock + OS file permissions;
the audit log records settings/security changes too (doc 05 §5.4).

## 6.8 Testing money (menu, adopted subset)

> 1. Property-based testing … “for any sequence of postings, the books balance”. 2. Invariant
> checks between steps … assert them after every single step. 3. Generative idempotency
> testing … repeat all the declared operations and assert the lack of impact … 4. Crash and
> resume injection … 5. Round-trip testing … 6. Golden testing … 7. Backward-compatibility
> testing … today’s code must still read what old code wrote.

**Adaptation:** all of 1–7 are adopted (doc 09). "Testing in production" is N/A offline —
replaced by golden reports over fixture books and the backward-compat corpus.
