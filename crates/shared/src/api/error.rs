// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared-action errors — registered BK-codes (RULE-ERR-002): BK-ACTION-0001, plus
//! pass-through of storage codes.

use bookkeepy_core::EntityId;

/// Errors the shared action engines produce.
#[derive(Debug, thiserror::Error)]
pub enum SharedError {
    /// The entity id is not in the action's table (BK-ACTION-0001).
    #[error("unknown entity {table}/{id}")]
    UnknownEntity {
        /// Table the action targeted.
        table: &'static str,
        /// Entity id.
        id: EntityId,
    },
    /// The action is not applicable to this table (BK-ACTION-0001).
    #[error("action not applicable to table {0}")]
    NotApplicable(&'static str),
    /// Underlying storage failure (code passed through).
    #[error(transparent)]
    Store(#[from] bookkeepy_storage::StorageError),
    /// Underlying SQL failure (mapped to BK-STORE-0001 like storage does).
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
}

impl SharedError {
    /// The registered BK-code (RULE-ERR-002).
    pub fn code(&self) -> &'static str {
        match self {
            SharedError::UnknownEntity { .. } | SharedError::NotApplicable(_) => "BK-ACTION-0001",
            SharedError::Store(e) => e.code(),
            SharedError::Sql(_) => "BK-STORE-0001",
        }
    }
}

/// Map into the storage error type (for running `_in_tx` variants inside
/// `storage::tx::post_with`, whose closure is storage-typed). Semantic errors travel as
/// `StorageError::Foreign` so their BK-code survives the boundary — recover with
/// [`from_storage`]. (ADR-0025: this used to map to `Integrity`/BK-STORE-0003 and lost
/// BK-ACTION-0001 — fixed.)
pub fn into_storage(e: SharedError) -> bookkeepy_storage::StorageError {
    match e {
        SharedError::Store(s) => s,
        SharedError::Sql(s) => s.into(),
        other => bookkeepy_storage::StorageError::Foreign(Box::new(other)),
    }
}

/// Recover a shared error from a storage error that crossed a closure boundary
/// (downcasts `SharedError` so its BK-code survives; RULE-SSOT seam rule S2).
pub fn from_storage(e: bookkeepy_storage::StorageError) -> SharedError {
    match e {
        bookkeepy_storage::StorageError::Foreign(b) => {
            if b.is::<SharedError>() {
                *b.downcast::<SharedError>()
                    .unwrap_or_else(|_| unreachable!())
            } else {
                SharedError::Store(bookkeepy_storage::StorageError::Foreign(b))
            }
        }
        other => SharedError::Store(other),
    }
}
