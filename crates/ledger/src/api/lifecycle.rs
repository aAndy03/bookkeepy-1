// SPDX-License-Identifier: GPL-3.0-or-later
//! Lifecycle & soft-delete semantics — the single source of truth (ADR-0024, RULE-SSOT-001).
//!
//! There is **only soft delete**, in two forms, and they mean different things:
//!
//! 1. **`Lifecycle::Archived` on catalog entities** (accounts, categories, tags, parties).
//!    "Deleted" = archived. Archived entities are excluded from pickers/new entries and from
//!    FORWARD-looking projections (budgets, forecasts, recurring templates, defaults), but
//!    they stay in HISTORY: old postings keep referencing them and historical reports still
//!    include them. Archiving never touches money.
//! 2. **`TxStatus` on transactions.** `pending` = not yet cleared (metadata only),
//!    `cleared`/`reconciled` = cleared / matched against an external statement (reconciled
//!    affects reconciliation reports only — never balances). `voided` = "deleted": the
//!    ledger posts a LINKED REVERSAL entry (ADR-0010), so the voided transaction is excluded
//!    from every computed money metric **because the journal itself nets it out** — never by
//!    filtering rows out of reports. The transaction row and its entry stay forever (audit).
//!
//! The rule that keeps the books provable: **computed metrics never filter posted rows by a
//! status flag** — exclusion of posted money happens exclusively through reversal entries, so
//! the trial balance holds for any prefix of the journal (INV-LEDGER-001). Status/archive
//! flags only steer what is *offered* (pickers, defaults) and which *non-posted* items
//! (scheduled/planned) are projected.

use bookkeepy_core::EntityId;
use bookkeepy_storage::api::conn::Db;

use crate::api::error::LedgerError;

/// Soft-delete state of a catalog entity (ADR-0024).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// Normal, usable for new entries.
    Active,
    /// Soft-deleted: hidden from pickers/forward projections, kept in history.
    Archived,
}

impl Lifecycle {
    /// DB flag value.
    pub fn flag(self) -> i64 {
        match self {
            Lifecycle::Active => 0,
            Lifecycle::Archived => 1,
        }
    }
}

/// User-level transaction status (ADR-0024 §2). Stored on `transactions.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxStatus {
    /// Recorded, not yet cleared.
    Pending,
    /// Cleared (money moved).
    Cleared,
    /// Matched against an external statement (reconciliation reports only).
    Reconciled,
    /// "Deleted": a linked reversal was posted; excluded from all money metrics via the
    /// journal, never by filtering.
    Voided,
}

impl TxStatus {
    /// DB string.
    pub fn as_str(self) -> &'static str {
        match self {
            TxStatus::Pending => "pending",
            TxStatus::Cleared => "cleared",
            TxStatus::Reconciled => "reconciled",
            TxStatus::Voided => "voided",
        }
    }

    /// Parse a DB string.
    pub fn from_db(s: &str) -> Option<TxStatus> {
        Some(match s {
            "pending" => TxStatus::Pending,
            "cleared" => TxStatus::Cleared,
            "reconciled" => TxStatus::Reconciled,
            "voided" => TxStatus::Voided,
            _ => return None,
        })
    }

    /// Allowed transitions (ADR-0024): pending→cleared→reconciled; anything→voided (terminal).
    pub fn can_transition(from: TxStatus, to: TxStatus) -> bool {
        if to == TxStatus::Voided {
            return from != TxStatus::Voided;
        }
        if from == TxStatus::Voided {
            return false;
        }
        let rank = |s: TxStatus| match s {
            TxStatus::Pending => 0,
            TxStatus::Cleared => 1,
            TxStatus::Reconciled => 2,
            TxStatus::Voided => 3,
        };
        rank(to) >= rank(from)
    }
}

/// Is this journal entry reversed by a linked reversal entry?
pub fn entry_is_reversed(db: &Db, entry_id: EntityId) -> Result<bool, LedgerError> {
    Ok(db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE reversal_of=?1",
            [entry_id.to_string()],
            |r| r.get::<_, i64>(0),
        )? > 0)
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_transitions() {
        assert!(TxStatus::can_transition(
            TxStatus::Pending,
            TxStatus::Cleared
        ));
        assert!(TxStatus::can_transition(
            TxStatus::Cleared,
            TxStatus::Reconciled
        ));
        assert!(TxStatus::can_transition(
            TxStatus::Pending,
            TxStatus::Voided
        ));
        assert!(!TxStatus::can_transition(
            TxStatus::Voided,
            TxStatus::Cleared
        ));
        assert!(!TxStatus::can_transition(
            TxStatus::Reconciled,
            TxStatus::Pending
        ));
        assert_eq!(TxStatus::from_db("voided"), Some(TxStatus::Voided));
        assert_eq!(TxStatus::from_db("nope"), None);
    }
}
