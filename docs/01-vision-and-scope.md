# 01 — Vision and scope

## Vision

Bookkeepy 1 lets any person manage their complete financial situation **locally and
offline**, with the same correctness guarantees a financial system is held to: money is
never invented, never lost, and never trusted blindly. The user sees simple transactions;
underneath, a double-entry, append-only ledger keeps the books balanced and auditable.

## Core promises

1. **Offline first.** Bookkeepy 1 works with no internet connection at all. The only
   network use in the entire application is downloading app updates (doc 08,
   ADR-0003). There are no accounts, no cloud sync, no bank APIs, no live rates.
2. **Data ownership.** "Bookkeepy will never close the data. The data is of the user."
   All user data lives in one portable SQLite file on the user's device. Bookkeepy only
   *manages* the data the user creates inside the app: it validates it, stores it
   correctly, keeps it auditable, and never sends it anywhere.
3. **Correct by design.** Integer minor units for money, a double-entry ledger, derived
   balances, reversals instead of edits, audit trail, tamper evidence (doc 06).
4. **Open source, distributable.** GNU GPL v3 or later; distributed through this
   repository's GitHub Releases (ADR-0002).
5. **Long-term maintainable.** Every addition, refactor, subtraction or deprecation
   follows the conventions in doc 04 so the codebase stays easy to change (doc 03, 10).

## Users

- Individuals managing personal finances (cash, banks, cards, loans, subscriptions).
- People in the EU / Romania first: multi-currency (RON, EUR, …), EN + RO locales from
  day one, date/number formats configurable.
- Users who value privacy: everything stays on their machine.

## Platforms

Windows first (primary development platform), then macOS and Linux. Native desktop app
built with gpui-kit — no web wrapper.

## In scope

See the full inventory in [02-features.md](02-features.md): accounts, transactions,
imports/exports, budgets, reports, multi-currency with user-provided FX rates, full-text
search (tantivy), attachments, backups, app lock, updates via GitHub Releases.

## Out of scope (explicitly)

- Live transactions, payments, bank connections, PSD2/AIS APIs, trading, crypto custody.
- Cloud sync, multi-device, accounts/login.
- Telemetry and analytics (doc 08).
- Any network call except the updater (RULE-NET-001).
- The fintecheng.txt topics that presuppose an online system (webhooks, payment rails,
  KYC/AML, custody) are **not** adopted; their offline equivalents are defined in
  doc 06 (e.g. reconciliation against imported bank statements instead of webhooks).

## Success criteria

- A user can record their whole financial life offline and, years later, reproduce any
  balance or report from the stored history alone (auditability, doc 06).
- The books always balance: sum of debits = sum of credits for any sequence of
  operations (property-tested, doc 09).
- The app never phones home; the only egress is the updater, and only when the user asks.
- A new contributor can add a feature by following doc 04 without tribal knowledge.
