// SPDX-License-Identifier: GPL-3.0-or-later
//! Crash-recovery self-check (round-2 item 2, approved): run at startup after open —
//! `PRAGMA integrity_check`, audit-chain verification, pending-outbox count, leftover
//! draft files. Never blocks (RULE-UX-001): it returns a report; the app shell decides
//! what to offer (draft recovery, "repair via restore from backup").

use std::path::Path;

use serde::Serialize;

use crate::api::audit;
use crate::api::conn::Db;
use crate::api::error::StorageError;
use crate::api::outbox;

/// Self-check report (serializable into a diagnostic bundle entry).
#[derive(Debug, Clone, Serialize)]
pub struct SelfCheckReport {
    /// `PRAGMA integrity_check` result ("ok" expected).
    pub integrity: String,
    /// Audit chain verified (true) or broken (false + seq in `audit_broken_at`).
    pub audit_ok: bool,
    /// Sequence where the audit chain broke, if any.
    pub audit_broken_at: Option<i64>,
    /// Unpublished outbox rows (republished by the startup driver).
    pub outbox_pending: i64,
    /// Draft files found in `<root>/drafts/` (crash recovery candidates).
    pub drafts: Vec<String>,
}

impl SelfCheckReport {
    /// Whether the DB is healthy (integrity ok + audit chain ok).
    pub fn healthy(&self) -> bool {
        self.integrity == "ok" && self.audit_ok
    }
}

/// Run the self-check on an open DB inside profile `root`.
pub fn run(db: &Db, root: &Path) -> Result<SelfCheckReport, StorageError> {
    let integrity = db
        .with_conn(|c| Ok(c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))?))?;
    let (audit_ok, audit_broken_at) = match audit::verify(db) {
        Ok(()) => (true, None),
        Err(StorageError::AuditChainBroken { seq }) => (false, Some(seq)),
        Err(e) => return Err(e),
    };
    let outbox_pending = outbox::pending_count(db)?;
    let drafts = std::fs::read_dir(root.join("drafts"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    Ok(SelfCheckReport {
        integrity,
        audit_ok,
        audit_broken_at,
        outbox_pending,
        drafts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_profile::ProfileRoots;

    #[test]
    fn healthy_db_reports_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = ProfileRoots::new(tmp.path().join("profiles/p1"));
        let db = Db::open_profile(&roots).unwrap();
        let report = run(&db, roots.root.as_path()).unwrap();
        assert!(report.healthy());
        assert_eq!(report.outbox_pending, 0);
    }
}
