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

/// A collision-free timestamped backup name for `<root>/backups/` (auto-<ts>.db,
/// auto-<ts>-2.db, … — two opens in the same second must not overwrite each other).
pub fn auto_name(root: &Path) -> String {
    let base = format!(
        "auto-{}",
        bookkeepy_core::time::now().strftime("%Y%m%d-%H%M%S")
    );
    for n in 0..1000 {
        let candidate = if n == 0 {
            format!("{base}.db")
        } else {
            format!("{base}-{n}.db")
        };
        if !root.join(BACKUPS_DIR).join(&candidate).exists() {
            return candidate;
        }
    }
    format!("auto-{}.db", uuid_v4ish())
}

fn uuid_v4ish() -> String {
    format!("{}", bookkeepy_core::EntityId::new())
}

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
    harden_file(&dst_path);
    Ok(dst_path)
}

/// Best-effort privacy hardening: on unix, restrict the file to the owner (0600) — the
/// data dir may sit on a shared/multi-user machine (RULE-PRIV-001). No-op elsewhere.
#[cfg(unix)]
fn harden_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(m) = std::fs::metadata(path) {
        let mut p = m.permissions();
        p.set_mode(0o600);
        let _ = std::fs::set_permissions(path, p);
    }
}

#[cfg(not(unix))]
fn harden_file(_path: &Path) {}

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

/// Verify a backup file: opens it READ-ONLY (verification must never create `-wal`/`-shm`
/// next to the backup or mutate it), runs integrity_check and full audit-chain verification.
pub fn verify_backup(path: &Path) -> Result<(), StorageError> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| StorageError::Backup {
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

/// Restore: replace the DB file at `db_path` with the backup file (caller holds the
/// single-instance lock; no open handle exists).
///
/// Crash-safe order: copy to a temp sibling → delete the stale `app.db-wal`/`app.db-shm`
/// (otherwise the next open would replay OLD WAL frames onto the restored DB — corruption /
/// resurrected rows) → atomically rename temp over `db_path`. A crash at any point leaves
/// either the old DB+old WAL (consistent) or the new DB (consistent).
pub fn restore(db_path: &Path, backup: &Path) -> Result<(), StorageError> {
    verify_backup(backup)?;
    let tmp = db_path.with_extension("db.restore-tmp");
    std::fs::copy(backup, &tmp).map_err(|e| StorageError::Backup {
        path: tmp.clone(),
        message: e.to_string(),
    })?;
    for sidecar in [
        db_path.with_extension("db-wal"),
        db_path.with_extension("db-shm"),
    ] {
        if sidecar.exists() {
            std::fs::remove_file(&sidecar).map_err(|e| StorageError::Backup {
                path: sidecar.clone(),
                message: e.to_string(),
            })?;
        }
    }
    std::fs::rename(&tmp, db_path).map_err(|e| StorageError::Backup {
        path: db_path.to_path_buf(),
        message: e.to_string(),
    })?;
    harden_file(db_path);
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
        // simulate a crash leftover: a stale WAL sidecar must be removed by restore
        std::fs::write(root.join("app.db-wal"), b"stale wal frames").unwrap();
        std::fs::write(root.join("app.db-shm"), b"stale shm").unwrap();
        restore(&db_path, &bak).unwrap();
        assert!(!root.join("app.db-wal").exists());
        assert!(!root.join("app.db-shm").exists());
        let db2 = Db::open(&db_path).unwrap();
        assert_eq!(ui_get(&db2, "k").unwrap().as_deref(), Some("v1"));
    }

    #[test]
    fn auto_name_never_collides() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(BACKUPS_DIR)).unwrap();
        let n1 = auto_name(root);
        std::fs::write(root.join(BACKUPS_DIR).join(&n1), b"x").unwrap();
        let n2 = auto_name(root);
        assert_ne!(n1, n2);
        assert!(n2.ends_with(".db"));
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
