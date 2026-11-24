// SPDX-License-Identifier: GPL-3.0-or-later
//! Ledger errors — registered BK-codes (RULE-ERR-002): BK-LEDGER-0001..0008, plus
//! pass-through of storage/shared codes.

use bookkeepy_core::EntityId;

/// Errors the ledger engine produces.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    /// Debits and credits differ for a currency (INV-LEDGER-001 violation) —
    /// BK-LEDGER-0001.
    #[error("entry unbalanced for {currency}: debit-credit diff {diff}")]
    Unbalanced {
        /// Currency that does not balance.
        currency: String,
        /// Debits minus credits.
        diff: i64,
    },
    /// Posting references a missing account — BK-LEDGER-0002.
    #[error("unknown account {0}")]
    UnknownAccount(EntityId),
    /// Posting targets an archived account — BK-LEDGER-0003.
    #[error("account {0} is archived")]
    AccountArchived(EntityId),
    /// Posting currency ≠ account currency — BK-LEDGER-0004.
    #[error("currency mismatch: account holds {expected}, posting is {got}")]
    CurrencyMismatch {
        /// Account currency.
        expected: String,
        /// Posting currency.
        got: String,
    },
    /// Reversal target missing or already reversed — BK-LEDGER-0005.
    #[error("invalid reversal: {0}")]
    InvalidReversal(String),
    /// Category/tag/payee reference missing — BK-LEDGER-0006.
    #[error("unknown catalog entity: {0}")]
    UnknownCatalog(String),
    /// Transaction invalid (missing account/category, bad transition) — BK-LEDGER-0007.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),
    /// Amount must be positive — BK-LEDGER-0008.
    #[error("amount must be positive")]
    AmountNotPositive,
    /// Underlying storage failure (code passed through).
    #[error(transparent)]
    Store(#[from] bookkeepy_storage::StorageError),
    /// Underlying shared-action failure (code passed through).
    #[error(transparent)]
    Shared(#[from] bookkeepy_shared::SharedError),
    /// Underlying SQL failure (mapped to BK-STORE-0001 like storage does).
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    /// Invalid recurrence schedule (BK-RECUR-0001).
    #[error("invalid schedule: {0}")]
    InvalidSchedule(String),
    /// Expansion hit its cap (BK-RECUR-0002, warning severity).
    #[error("expansion capped at {0} occurrences")]
    ExpansionCapped(usize),
    /// Occurrence already materialized (BK-RECUR-0003).
    #[error("occurrence {0} already materialized")]
    AlreadyMaterialized(String),
}

/// Map into the storage error type (for running ledger `_in_tx` steps inside
/// `storage::tx::post_with`, whose closure is storage-typed). Semantic errors travel as
/// `StorageError::Foreign` so their BK-code survives the boundary — recover with
/// [`from_storage`].
pub fn into_storage(e: LedgerError) -> bookkeepy_storage::StorageError {
    match e {
        LedgerError::Store(s) => s,
        LedgerError::Sql(s) => s.into(),
        LedgerError::Shared(s) => bookkeepy_shared::into_storage(s),
        other => bookkeepy_storage::StorageError::Foreign(Box::new(other)),
    }
}

/// Recover a ledger error from a storage error that crossed a closure boundary.
pub fn from_storage(e: bookkeepy_storage::StorageError) -> LedgerError {
    match e {
        bookkeepy_storage::StorageError::Foreign(b) => b
            .downcast::<LedgerError>()
            .map(|b| *b)
            .unwrap_or_else(|b| LedgerError::InvalidTransaction(b.to_string())),
        other => LedgerError::Store(other),
    }
}

impl LedgerError {
    /// The registered BK-code (RULE-ERR-002).
    pub fn code(&self) -> &'static str {
        match self {
            LedgerError::Unbalanced { .. } => "BK-LEDGER-0001",
            LedgerError::UnknownAccount(_) => "BK-LEDGER-0002",
            LedgerError::AccountArchived(_) => "BK-LEDGER-0003",
            LedgerError::CurrencyMismatch { .. } => "BK-LEDGER-0004",
            LedgerError::InvalidReversal(_) => "BK-LEDGER-0005",
            LedgerError::UnknownCatalog(_) => "BK-LEDGER-0006",
            LedgerError::InvalidTransaction(_) => "BK-LEDGER-0007",
            LedgerError::AmountNotPositive => "BK-LEDGER-0008",
            LedgerError::Store(e) => e.code(),
            LedgerError::Shared(e) => e.code(),
            LedgerError::Sql(_) => "BK-STORE-0001",
            LedgerError::InvalidSchedule(_) => "BK-RECUR-0001",
            LedgerError::ExpansionCapped(_) => "BK-RECUR-0002",
            LedgerError::AlreadyMaterialized(_) => "BK-RECUR-0003",
        }
    }
}
