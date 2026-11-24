// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-shared` — the shared action engines (RULE-SHARED-001, doc 04 §4.2
//! `actions/`): `rename` / `archive` / `restore` / `change_status`, ONE generic audited
//! implementation used by every entity kind (accounts, categories, tags, parties,
//! transactions). No feature may grow its own rename/archive — localization happens at the
//! call site, semantics here (single source of truth). Every action runs in the same
//! transaction as its audit + outbox event (`storage::tx::post_with`).

pub mod api;

pub use api::actions::{
    Actionable, archive, archive_in_tx, change_status, change_status_in_tx, rename, rename_in_tx,
    restore, restore_in_tx,
};
pub use api::error::{SharedError, into_storage};
