// SPDX-License-Identifier: GPL-3.0-or-later
//! Forward-only migrations (doc 05 §5.4, RULE-DATA-004 / ADR-0022).
//!
//! - `schema_version` lives in the `meta` table; migrations are applied in order inside a
//!   transaction, each recorded in `schema_migrations`.
//! - If the profile's version is NEWER than `MAX_KNOWN`, opening fails with
//!   `SchemaTooNew` (BK-DATA-0002) and the file is never written.
//! - Migrations are additive, never destructive (INV-DATA-001). Down-migrations are not
//!   supported; every migration is tested up AND down on fixtures (doc 09).

use rusqlite::{Connection, Transaction};

use crate::api::conn::Db;
use crate::api::error::StorageError;

/// Highest schema version this build knows.
pub const MAX_KNOWN: i64 = 1;

/// Ordered migrations: (version, name, SQL). Append-only — never edit existing entries.
const MIGRATIONS: &[(i64, &str, &str)] = &[(
    1,
    "v1_core_tables",
    r#"
    CREATE TABLE IF NOT EXISTS meta (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS schema_migrations (
        version INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        applied_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS ui_state (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS audit_log (
        seq INTEGER PRIMARY KEY AUTOINCREMENT,
        id TEXT NOT NULL UNIQUE,
        kind TEXT NOT NULL,
        occurred_at TEXT NOT NULL,
        actor TEXT NOT NULL,
        reason TEXT NOT NULL,
        payload TEXT NOT NULL,
        prev_hash TEXT NOT NULL,
        hash TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS audit_log_hash ON audit_log(hash);
    CREATE TABLE IF NOT EXISTS outbox (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        occurred_at TEXT NOT NULL,
        actor TEXT NOT NULL,
        reason TEXT NOT NULL,
        payload TEXT NOT NULL,
        published INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS jobs (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        state TEXT NOT NULL,
        payload TEXT NOT NULL,
        attempts INTEGER NOT NULL DEFAULT 0,
        updated_at TEXT NOT NULL
    );
    "#,
)];

/// Read the current schema version (0 if the meta table doesn't exist yet).
pub fn current_version(conn: &Connection) -> Result<i64, StorageError> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='meta'",
        [],
        |r| r.get(0),
    )?;
    if exists == 0 {
        return Ok(0);
    }
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .ok();
    Ok(v.and_then(|s| s.parse().ok()).unwrap_or(0))
}

/// Apply pending migrations. Refuses too-new profiles first (RULE-DATA-004).
pub fn migrate(db: &Db) -> Result<(), StorageError> {
    db.with_conn(|conn| {
        let found = current_version(conn)?;
        if found > MAX_KNOWN {
            return Err(StorageError::SchemaTooNew {
                found,
                max: MAX_KNOWN,
            });
        }
        let tx = conn.transaction()?;
        let applied = current_version_tx(&tx)?;
        for (version, name, sql) in MIGRATIONS {
            if *version <= applied {
                continue;
            }
            tx.execute_batch(sql).map_err(|source| StorageError::Migrate {
                name: name.to_string(),
                source,
            })?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![version, name, jiff_now()],
            )?;
            tx.execute("INSERT INTO meta (key, value) VALUES ('schema_version', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", rusqlite::params![version.to_string()])?;
        }
        // audit immutability guards ship with the schema (RULE-AUD-001)
        crate::api::audit::install_guard_triggers(&tx)?;
        tx.commit()?;
        Ok(())
    })
}

fn current_version_tx(tx: &Transaction) -> Result<i64, StorageError> {
    let exists: i64 = tx.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='meta'",
        [],
        |r| r.get(0),
    )?;
    if exists == 0 {
        return Ok(0);
    }
    let v: Option<String> = tx
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .ok();
    Ok(v.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn jiff_now() -> String {
    bookkeepy_core::time::now().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_db_migrates_to_v1() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        assert_eq!(db.schema_version().unwrap(), 1);
        // reopening is idempotent
        let db2 = Db::open(&tmp.path().join("app.db")).unwrap();
        assert_eq!(db2.schema_version().unwrap(), 1);
    }

    #[test]
    fn too_new_profile_is_refused_without_writing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("app.db");
        {
            let db = Db::open(&path).unwrap();
            db.with_conn(|c| {
                c.execute(
                    "UPDATE meta SET value=?1 WHERE key='schema_version'",
                    rusqlite::params![(MAX_KNOWN + 5).to_string()],
                )
                .unwrap();
                Ok(())
            })
            .unwrap();
        }
        let err = Db::open(&path).unwrap_err();
        assert_eq!(err.code(), "BK-DATA-0002");
        assert!(matches!(err, StorageError::SchemaTooNew { .. }));
    }
}
