// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-ledger` — the ledger engine (M2, ADR-0009/0010, doc 05 §5.4).
//!
//! fintecheng (doc 06 §6.4, quoted): "every entry moves the same amount out of one account
//! and into another, the books always balance — money is only moved, never created or
//! destroyed"; "balance is never stored, it's derived"; "posted entries are immutable;
//! corrections are made by adding new compensating entries". This crate implements exactly
//! that: `journal_entries` + `postings` are append-only, SHA-256 hash-chained, DB-trigger
//! guarded; user-level `transactions` compile into balanced postings; balances are
//! read-only projections; edits/voids post linked reversals (fix-forward).
//!
//! Lifecycle / soft-delete semantics (ADR-0024): see `api::lifecycle`.

pub mod api;

pub use api::accounts::{Account, AccountType};
pub use api::balances;
pub use api::catalog::{self, Category, Party, Tag};
pub use api::entries::{self, NewEntry, Posting, Side};
pub use api::error::LedgerError;
pub use api::lifecycle::{Lifecycle, TxStatus};
pub use api::transactions::{self, NewTransaction, TxKind};
