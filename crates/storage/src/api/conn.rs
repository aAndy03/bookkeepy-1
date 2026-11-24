// SPDX-License-Identifier: GPL-3.0-or-later
//! Connection management (ADR-0005): one SQLite file per profile, WAL mode, foreign
//! keys on, busy timeout. The connection lives behind a Mutex — rusqlite connections are
//! not Sync; all access goes through `Db::with_conn` so transactions stay explicit.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::api::error::StorageError;
use crate::api::schema;
use bookkeepy_profile::ProfileRoots;

/// A profile database handle.
pub struct Db {
    pub(crate) conn: Mutex<Connection>,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db").finish_non_exhaustive()
    }
}

impl Db {
    /// Open (or create) the DB at `path`, apply pragmas, run migrations (forward-only).
    /// Refuses profiles newer than this build (RULE-DATA-004 → BK-DATA-0002).
    pub fn open(path: &Path) -> Result<Db, StorageError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StorageError::Open {
                path: path.to_path_buf(),
                source: rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("cannot create dir {}: {e}", parent.display())),
                ),
            })?;
        }
        let conn = Connection::open(path).map_err(|source| StorageError::Open {
            path: path.to_path_buf(),
            source,
        })?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        schema::migrate(&db)?;
        Ok(db)
    }

    /// Open the DB inside a profile root (`<root>/app.db`, doc 05 §5.2).
    pub fn open_profile(roots: &ProfileRoots) -> Result<Db, StorageError> {
        roots.create().map_err(|e| StorageError::Open {
            path: roots.db(),
            source: rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(e.to_string()),
            ),
        })?;
        Db::open(&roots.db())
    }

    /// Run a closure with the connection (mutable, so transactions work; readers just
    /// use the `&mut` as `&`).
    pub fn with_conn<R>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<R, StorageError>,
    ) -> Result<R, StorageError> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut conn)
    }

    /// Current schema version (from `meta` table; 0 on a fresh DB before migrations).
    pub fn schema_version(&self) -> Result<i64, StorageError> {
        self.with_conn(|c| schema::current_version(c))
    }
}

// rusqlite::Error is not Clone; map pragma errors into StorageError at call sites via From.
impl From<rusqlite::Error> for StorageError {
    fn from(source: rusqlite::Error) -> Self {
        StorageError::Open {
            path: std::path::PathBuf::from("<conn>"),
            source,
        }
    }
}
