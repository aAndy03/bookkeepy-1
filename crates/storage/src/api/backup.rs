// SPDX-License-Identifier: GPL-3.0-or-later
//! Backup / restore / verify (doc 05 §5.4, M1): LOCAL LIVE backup via rusqlite's backup
//! API — "live" because the source DB stays open and usable during the copy; it is purely
//! local, fully offline, no network. Backups land in `backups/` under the profile root;
//! verify by opening the copy and running `PRAGMA integrity_check` + audit chain; restore
//! replaces the DB file (caller must hold the single-instance lock).

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;

use crate::api::audit;
use crate::api::conn::Db;
use crate::api::error::StorageError;

/// Directory for backups inside the profile root.
pub const BACKUPS_DIR: &str = "backups";

/// How many local live backups to keep; older ones are pruned (names are timestamped, so
/// lexicographic order = chronological order).
pub const KEEP_BACKUPS: usize = 10;

fn backup_path(root: &Path, name: &str) -> PathBuf {
    root.join(BACKUPS_DIR).join(name)
}

/// Local live backup of the open DB into `<root>/backups/<name>` (WAL-safe, offline).
pub fn backup(db: &Db, root: &Path, name: &str) -> Result<PathBuf, StorageError> {
    let dst_path = backup_path(root, name);
    let dir = dst_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root.to_path_buf());
    std::fs::create_dir_all(&dir).map_err(|e| StorageError::Backup {
        path: dst_path.clone(),
        message: e.to_string(),
    })?;
    db.with_conn(|src| {
        let mut dst = Connection::open(&dst_path).map_err(|e| StorageError::Backup {
            path: dst_path.clone(),
            message: e.to_string(),
        })?;
        let b = rusqlite::backup::Backup::new(src, &mut dst).map_err(|e| StorageError::Backup {
            path: dst_path.clone(),
            message: e.to_string(),
        })?;
        b.run_to_completion(5, Duration::from_millis(50), None)
            .map_err(|e| StorageError::Backup {
                path: dst_path.clone(),
                message: format!("{:?}", e),
            })?;
        Ok(())
    })?;
    Ok(dst_path)
}

/// Prune `backups/` keeping only the newest `keep` `.db` files. Returns how many were
/// deleted. Best-effort by design: the session calls it after each auto backup.
pub fn prune(root: &Path, keep: usize) -> Result<usize, StorageError> {
    let dir = root.join(BACKUPS_DIR);
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "db"))
            .collect(),
        Err(_) => return Ok(0),
    };
    entries.sort(); // timestamped names sort chronologically
    let excess = entries.len().saturating_sub(keep);
    for p in entries.into_iter().take(excess) {
        std::fs::remove_file(&p).map_err(|e| StorageError::Backup {
            path: p.clone(),
            message: e.to_string(),
        })?;
    }
    Ok(excess)
}

/// Verify a backup file: opens it, runs integrity_check and audit-chain verification.
pub fn verify_backup(path: &Path) -> Result<(), StorageError> {
    let conn = Connection::open(path).map_err(|e| StorageError::Backup {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let ic: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if ic != "ok" {
        return Err(StorageError::Integrity(format!(
            "integrity_check: {ic} in {}",
            path.display()
        )));
    }
    let db = Db {
        conn: std::sync::Mutex::new(conn),
    };
    audit::verify(&db)?;
    Ok(())
}

/// Restore: replace the DB file at `db_path` with the backup file (caller holds the lock).
pub fn restore(db_path: &Path, backup: &Path) -> Result<(), StorageError> {
    verify_backup(backup)?;
    std::fs::copy(backup, db_path).map_err(|e| StorageError::Backup {
        path: db_path.to_path_buf(),
        message: e.to_string(),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::kv::{ui_get, ui_set};

    #[test]
    fn backup_verify_restore_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let db_path = root.join("app.db");
        {
            let db = Db::open(&db_path).unwrap();
            ui_set(&db, "k", "v1").unwrap();
            let bak = backup(&db, root, "b1.db").unwrap();
            verify_backup(&bak).unwrap();
        } // close so the WAL is checkpointed
        {
            let db = Db::open(&db_path).unwrap();
            ui_set(&db, "k", "v2").unwrap();
        } // close again — restore must happen with no open handle (caller holds the lock)
        let bak = root.join("backups/b1.db");
        restore(&db_path, &bak).unwrap();
        let db2 = Db::open(&db_path).unwrap();
        assert_eq!(ui_get(&db2, "k").unwrap().as_deref(), Some("v1"));
    }

    #[test]
    fn prune_keeps_newest() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let db = Db::open(&root.join("app.db")).unwrap();
        for name in [
            "auto-20260101-000000.db",
            "auto-20260102-000000.db",
            "auto-20260103-000000.db",
        ] {
            backup(&db, root, name).unwrap();
        }
        assert_eq!(prune(root, 2).unwrap(), 1);
        let left: Vec<_> = std::fs::read_dir(root.join(BACKUPS_DIR))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            left,
            vec![
                "auto-20260102-000000.db".to_string(),
                "auto-20260103-000000.db".to_string()
            ]
        );
    }
}
