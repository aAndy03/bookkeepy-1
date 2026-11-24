// SPDX-License-Identifier: GPL-3.0-or-later
//! Storage errors — every variant carries a registered BK-code (RULE-ERR-002) with a
//! message in every locale. Codes: BK-STORE-0001..0006, BK-DATA-0002 (downgrade),
//! BK-DATA-0003 (unknown settings key), BK-PROFILE-0001 / BK-LOG-0001 (session composition).

use std::path::PathBuf;

/// Storage-layer errors.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// The data file cannot be opened (BK-STORE-0001).
    #[error("cannot open data file {path}: {source}")]
    Open {
        /// Path of the DB file.
        path: PathBuf,
        /// Underlying rusqlite error.
        source: rusqlite::Error,
    },
    /// A migration failed; previous data is untouched (transaction rolled back)
    /// (BK-STORE-0002).
    #[error("migration {name} failed: {source}")]
    Migrate {
        /// Migration name.
        name: String,
        /// Underlying rusqlite error.
        source: rusqlite::Error,
    },
    /// The profile was written by a newer Bookkeepy (RULE-DATA-004, ADR-0022) —
    /// BK-DATA-0002. The file is never modified in this case.
    #[error(
        "profile schema_version {found} is newer than this app supports ({max}); install a newer Bookkeepy or export from it (BK-DATA-0002)"
    )]
    SchemaTooNew {
        /// Schema version found in the profile.
        found: i64,
        /// Max schema version this build knows.
        max: i64,
    },
    /// `PRAGMA integrity_check` failed or the audit chain is broken (BK-STORE-0003 /
    /// BK-STORE-0006).
    #[error("integrity check failed: {0} (BK-STORE-0003)")]
    Integrity(String),
    /// Another instance holds the profile lock (BK-STORE-0004, ADR-0016).
    #[error("another Bookkeepy instance has this profile open (BK-STORE-0004)")]
    LockHeld,
    /// Backup/restore failed (BK-STORE-0005).
    #[error("backup operation failed at {path}: {message} (BK-STORE-0005)")]
    Backup {
        /// Path involved.
        path: PathBuf,
        /// Detail message.
        message: String,
    },
    /// The audit hash chain does not verify (BK-STORE-0006).
    #[error("audit hash chain broken at seq {seq} (BK-STORE-0006)")]
    AuditChainBroken {
        /// Sequence where verification failed.
        seq: i64,
    },
    /// Profile registry failure during session composition (BK-PROFILE-0001).
    #[error("profile error: {0}")]
    Profile(#[from] bookkeepy_profile::ProfileError),
    /// Logging init failure during session composition (BK-LOG-0001, warning severity —
    /// the session continues without file logs, never blocks, RULE-UX-001).
    #[error("logging init failed: {0}")]
    Log(#[from] bookkeepy_logging::LogError),
    /// A settings key not present in `data/settings-defaults.toml` (BK-DATA-0003).
    #[error("unknown settings key {key:?} (BK-DATA-0003)")]
    UnknownSetting {
        /// The offending key.
        key: String,
    },
    /// A foreign engine error carried through a `with_conn`/`post_with` closure
    /// (generic code BK-STORE-0001; the real code travels inside — downcast to recover it,
    /// see `bookkeepy-ledger`'s `from_storage`). Used so semantic errors keep their
    /// BK-code across the transaction boundary.
    #[error(transparent)]
    Foreign(Box<dyn std::error::Error + Send + Sync>),
}

impl StorageError {
    /// The registered BK-code for this error (RULE-ERR-002).
    pub fn code(&self) -> &'static str {
        match self {
            StorageError::Open { .. } => "BK-STORE-0001",
            StorageError::Migrate { .. } => "BK-STORE-0002",
            StorageError::SchemaTooNew { .. } => "BK-DATA-0002",
            StorageError::Integrity(_) => "BK-STORE-0003",
            StorageError::LockHeld => "BK-STORE-0004",
            StorageError::Backup { .. } => "BK-STORE-0005",
            StorageError::AuditChainBroken { .. } => "BK-STORE-0006",
            StorageError::Profile(_) => "BK-PROFILE-0001",
            StorageError::Log(_) => "BK-LOG-0001",
            StorageError::UnknownSetting { .. } => "BK-DATA-0003",
            StorageError::Foreign(_) => "BK-STORE-0001",
        }
    }
}
