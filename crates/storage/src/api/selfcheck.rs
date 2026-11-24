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

/// WAL size (bytes) above which the self-check warns: a large `-wal` usually means the
/// user hand-copied `app.db` while the app was open — such a copy is stale (the WAL holds
/// recent commits). Auto-checkpoint (1000 pages) normally keeps this small.
pub const WAL_WARN_BYTES: u64 = 16 * 1024 * 1024;

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
    /// Note about the startup auto backup (None if it was not attempted/failed silently —
    /// set by `session::open_session`; backup failure never blocks startup, RULE-UX-001).
    pub backup_note: Option<String>,
    /// `schema_migrations` rows match the code's migration list (false = hand-copied or
    /// partially-written DB).
    pub migrations_match: bool,
    /// Settings overrides in the DB whose key is NOT in the SSOT list
    /// (`data/settings-defaults.toml`) — e.g. written by a NEWER app; ignored, never
    /// fatal (forward compatibility, INV-DATA-001).
    pub orphan_settings: Vec<String>,
    /// Size of `app.db-wal` in bytes (see [`WAL_WARN_BYTES`]).
    pub wal_bytes: u64,
    /// Boot duration in milliseconds, measured by `session::open_session` against the
    /// `cold_start_to_add_transaction` budget in `data/perf-budgets.toml` (INV-LOAD-001).
    pub boot_ms: Option<u64>,
}

impl SelfCheckReport {
    /// Whether the DB is healthy (integrity ok + audit chain ok).
    pub fn healthy(&self) -> bool {
        self.integrity == "ok" && self.audit_ok
    }

    /// True when the WAL sidecar is suspiciously large (possible hand-copied DB).
    pub fn wal_warn(&self) -> bool {
        self.wal_bytes > WAL_WARN_BYTES
    }
}

/// Run the self-check on an open DB inside profile `root`.
pub fn run(db: &Db, root: &Path) -> Result<SelfCheckReport, StorageError> {
    let integrity = db
        .with_conn(|c| Ok(c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))?))?;
    // Boot path uses incremental audit verification (checkpointed) — full verify stays
    // available on demand and for backups.
    let (audit_ok, audit_broken_at) = match audit::verify_incremental(db) {
        Ok(_) => (true, None),
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
    let migrations_match = db.with_conn(|c| {
        let mut stmt = c.prepare("SELECT version, name FROM schema_migrations ORDER BY version")?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows.len() == crate::api::schema::MIGRATIONS.len()
            && rows
                .iter()
                .zip(crate::api::schema::MIGRATIONS.iter())
                .all(|((v, n), m)| *v == m.version && n == m.name))
    })?;
    let orphan_settings = crate::api::kv::orphan_settings(db)?;
    let wal_bytes = std::fs::metadata(root.join("app.db").with_extension("db-wal"))
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(SelfCheckReport {
        integrity,
        audit_ok,
        audit_broken_at,
        outbox_pending,
        drafts,
        backup_note: None,
        migrations_match,
        orphan_settings,
        wal_bytes,
        boot_ms: None,
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
