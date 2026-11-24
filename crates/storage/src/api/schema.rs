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
pub const MAX_KNOWN: i64 = 3;

/// One migration. Append-only — never edit existing entries; "reverts" are compensating
/// forward migrations (INV-DATA-001), proven by tests/migrations_flow.rs.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Schema version this migration produces.
    pub version: i64,
    /// Short name (recorded in `schema_migrations`).
    pub name: &'static str,
    /// SQL batch, run inside the migration transaction.
    pub sql: &'static str,
}

/// v1 — core tables (meta, migrations, settings, ui_state, audit_log, outbox, jobs).
pub const MIGRATION_V1: Migration = Migration {
    version: 1,
    name: "v1_core_tables",
    sql: r#"
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
};

/// M2 — the ledger engine's tables (ADR-0009/0010, doc 05 §5.4). Append-only, hash-chained
/// `journal_entries`/`postings` (UPDATE/DELETE rejected by triggers, like `audit_log`);
/// `transactions` allows ONLY status updates (money-affecting edits post linked reversals
/// instead — ADR-0010). Catalog tables (accounts/categories/tags/parties) are mutable but
/// every change goes through the shared action engines → audited (RULE-SHARED-001).
pub const MIGRATION_V2_LEDGER: Migration = Migration {
    version: 2,
    name: "v2_ledger_tables",
    sql: r#"
    CREATE TABLE IF NOT EXISTS accounts (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        type TEXT NOT NULL CHECK(type IN ('asset','liability','equity','income','expense')),
        currency TEXT NOT NULL,
        color TEXT,
        icon TEXT,
        archived INTEGER NOT NULL DEFAULT 0,
        is_system INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS journal_entries (
        id TEXT PRIMARY KEY,
        tx_id TEXT,
        kind TEXT NOT NULL CHECK(kind IN ('entry','reversal')),
        value_ts TEXT NOT NULL,
        booked_ts TEXT NOT NULL,
        note TEXT,
        reversal_of TEXT,
        prev_hash TEXT NOT NULL,
        hash TEXT NOT NULL,
        created_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS journal_entries_tx ON journal_entries(tx_id);
    CREATE INDEX IF NOT EXISTS journal_entries_reversal ON journal_entries(reversal_of);
    CREATE TABLE IF NOT EXISTS postings (
        id TEXT PRIMARY KEY,
        entry_id TEXT NOT NULL REFERENCES journal_entries(id),
        account_id TEXT NOT NULL REFERENCES accounts(id),
        side TEXT NOT NULL CHECK(side IN ('debit','credit')),
        amount_minor INTEGER NOT NULL CHECK(amount_minor > 0),
        currency TEXT NOT NULL,
        prev_hash TEXT NOT NULL,
        hash TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS postings_entry ON postings(entry_id);
    CREATE INDEX IF NOT EXISTS postings_account ON postings(account_id);
    CREATE TABLE IF NOT EXISTS transactions (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL CHECK(kind IN ('income','expense','transfer')),
        status TEXT NOT NULL DEFAULT 'cleared'
            CHECK(status IN ('pending','cleared','reconciled','voided')),
        value_ts TEXT NOT NULL,
        entry_ts TEXT NOT NULL,
        payee_id TEXT,
        category_id TEXT,
        note TEXT,
        entry_id TEXT REFERENCES journal_entries(id),
        replaces TEXT,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS categories (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        parent_id TEXT REFERENCES categories(id),
        account_id TEXT NOT NULL REFERENCES accounts(id),
        archived INTEGER NOT NULL DEFAULT 0,
        color TEXT,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS tags (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        archived INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS parties (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        archived INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS tx_tags (
        tx_id TEXT NOT NULL REFERENCES transactions(id),
        tag_id TEXT NOT NULL REFERENCES tags(id),
        PRIMARY KEY (tx_id, tag_id)
    );
    "#,
};

/// M2b — recurrence schedules (ADR-0020). `schedules` holds the RULE (template JSON +
/// frequency/interval/start/end/exceptions), never occurrence rows; occurrences are derived
/// projections until materialized. Schedules are not posted money, so the table is mutable
/// (every change audited via the shared action engines / scheduler actor). `transactions`
/// gains `schedule_id`/`occurrence_date` linking a materialized occurrence back; the
/// status-only trigger is recreated to cover the new columns.
pub const MIGRATION_V3_SCHEDULES: Migration = Migration {
    version: 3,
    name: "v3_schedules",
    sql: r#"
    CREATE TABLE IF NOT EXISTS schedules (
        id TEXT PRIMARY KEY,
        template TEXT NOT NULL,
        frequency TEXT NOT NULL CHECK(frequency IN ('daily','weekly','monthly','yearly')),
        interval INTEGER NOT NULL DEFAULT 1 CHECK(interval > 0),
        start_date TEXT NOT NULL,
        end_kind TEXT NOT NULL CHECK(end_kind IN ('never','on_date','after_count')),
        end_value TEXT,
        exceptions TEXT NOT NULL DEFAULT '[]',
        created_at TEXT NOT NULL
    );
    ALTER TABLE transactions ADD COLUMN schedule_id TEXT;
    ALTER TABLE transactions ADD COLUMN occurrence_date TEXT;
    DROP TRIGGER IF EXISTS tx_only_status;
    CREATE TRIGGER tx_only_status BEFORE UPDATE ON transactions
    WHEN NEW.kind IS NOT OLD.kind OR NEW.value_ts IS NOT OLD.value_ts
        OR NEW.entry_ts IS NOT OLD.entry_ts OR NEW.payee_id IS NOT OLD.payee_id
        OR NEW.category_id IS NOT OLD.category_id OR NEW.note IS NOT OLD.note
        OR NEW.entry_id IS NOT OLD.entry_id OR NEW.replaces IS NOT OLD.replaces
        OR NEW.created_at IS NOT OLD.created_at
        OR NEW.schedule_id IS NOT OLD.schedule_id
        OR NEW.occurrence_date IS NOT OLD.occurrence_date
    BEGIN SELECT RAISE(ABORT, 'transactions: only status may change; edit via reversal (ADR-0010)'); END;
    "#,
};

/// Ordered migrations. Append-only — never edit existing entries.
pub const MIGRATIONS: &[Migration] = &[MIGRATION_V1, MIGRATION_V2_LEDGER, MIGRATION_V3_SCHEDULES];

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
    db.with_conn(|conn| migrate_with(conn, MIGRATIONS, MAX_KNOWN))
}

/// Apply `migrations` newer than the profile's current version, in one transaction.
/// Public so tests can prove compensating (revert) migrations preserve data
/// (tests/migrations_flow.rs). Production code passes [`MIGRATIONS`]/[`MAX_KNOWN`].
pub fn migrate_with(
    conn: &mut Connection,
    migrations: &[Migration],
    max_known: i64,
) -> Result<(), StorageError> {
    let found = current_version(conn)?;
    if found > max_known {
        return Err(StorageError::SchemaTooNew {
            found,
            max: max_known,
        });
    }
    let tx = conn.transaction()?;
    let applied = current_version_tx(&tx)?;
    for m in migrations {
        if m.version <= applied {
            continue;
        }
        tx.execute_batch(m.sql)
            .map_err(|source| StorageError::Migrate {
                name: m.name.to_string(),
                source,
            })?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![m.version, m.name, jiff_now()],
        )?;
        tx.execute("INSERT INTO meta (key, value) VALUES ('schema_version', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", rusqlite::params![m.version.to_string()])?;
    }
    // audit immutability guards ship with the schema (RULE-AUD-001)
    crate::api::audit::install_guard_triggers(&tx)?;
    install_ledger_guard_triggers(&tx)?;
    tx.commit()?;
    Ok(())
}

/// Ledger immutability guards (ADR-0010, INV-CHAIN-001): `journal_entries` and `postings`
/// reject UPDATE/DELETE outright; `transactions` rejects everything except `status`
/// (money-affecting edits post linked reversals instead; status is metadata, audited via
/// the shared action engines). Idempotent.
pub fn install_ledger_guard_triggers(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS entries_no_update BEFORE UPDATE ON journal_entries
        BEGIN SELECT RAISE(ABORT, 'journal_entries is immutable (ADR-0010)'); END;
        CREATE TRIGGER IF NOT EXISTS entries_no_delete BEFORE DELETE ON journal_entries
        BEGIN SELECT RAISE(ABORT, 'journal_entries is immutable (ADR-0010)'); END;
        CREATE TRIGGER IF NOT EXISTS postings_no_update BEFORE UPDATE ON postings
        BEGIN SELECT RAISE(ABORT, 'postings is immutable (ADR-0010)'); END;
        CREATE TRIGGER IF NOT EXISTS postings_no_delete BEFORE DELETE ON postings
        BEGIN SELECT RAISE(ABORT, 'postings is immutable (ADR-0010)'); END;
        CREATE TRIGGER IF NOT EXISTS tx_only_status BEFORE UPDATE ON transactions
        WHEN NEW.kind IS NOT OLD.kind OR NEW.value_ts IS NOT OLD.value_ts
            OR NEW.entry_ts IS NOT OLD.entry_ts OR NEW.payee_id IS NOT OLD.payee_id
            OR NEW.category_id IS NOT OLD.category_id OR NEW.note IS NOT OLD.note
            OR NEW.entry_id IS NOT OLD.entry_id OR NEW.replaces IS NOT OLD.replaces
            OR NEW.created_at IS NOT OLD.created_at
        BEGIN SELECT RAISE(ABORT, 'transactions: only status may change; edit via reversal (ADR-0010)'); END;
        CREATE TRIGGER IF NOT EXISTS tx_no_delete BEFORE DELETE ON transactions
        BEGIN SELECT RAISE(ABORT, 'transactions cannot be deleted; void via reversal (ADR-0010)'); END;
        "#,
    )?;
    Ok(())
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
    fn fresh_db_migrates_to_max_known() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        assert_eq!(db.schema_version().unwrap(), MAX_KNOWN);
        // reopening is idempotent
        let db2 = Db::open(&tmp.path().join("app.db")).unwrap();
        assert_eq!(db2.schema_version().unwrap(), MAX_KNOWN);
    }

    #[test]
    fn ledger_tables_are_immutable_at_db_level() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        db.with_conn(|c| {
            c.execute(
                "INSERT INTO accounts (id, name, type, currency, archived, is_system, created_at) VALUES ('a1','Cash','asset','RON',0,0,'2026-11-24T10:00:00Z')",
                [],
            )?;
            c.execute(
                "INSERT INTO journal_entries (id, tx_id, kind, value_ts, booked_ts, note, reversal_of, prev_hash, hash, created_at) VALUES ('e1', NULL, 'entry', '2026-11-24T10:00:00Z','2026-11-24T10:00:00Z', NULL, NULL, 'GENESIS', 'h1', '2026-11-24T10:00:00Z')",
                [],
            )?;
            c.execute(
                "INSERT INTO postings (id, entry_id, account_id, side, amount_minor, currency, prev_hash, hash) VALUES ('p1','e1','a1','debit',100,'RON','GENESIS','h1')",
                [],
            )?;
            // entries/postings reject UPDATE and DELETE
            assert!(c.execute("UPDATE journal_entries SET note='x' WHERE id='e1'", []).is_err());
            assert!(c.execute("DELETE FROM postings WHERE id='p1'", []).is_err());
            // transactions allow status-only updates
            c.execute(
                "INSERT INTO transactions (id, kind, status, value_ts, entry_ts, payee_id, category_id, note, entry_id, replaces, created_at) VALUES ('t1','expense','cleared','2026-11-24T10:00:00Z','2026-11-24T10:00:00Z',NULL,NULL,NULL,'e1',NULL,'2026-11-24T10:00:00Z')",
                [],
            )?;
            c.execute("UPDATE transactions SET status='reconciled' WHERE id='t1'", [])?;
            assert!(c.execute("UPDATE transactions SET note='x' WHERE id='t1'", []).is_err());
            assert!(c.execute("DELETE FROM transactions WHERE id='t1'", []).is_err());
            Ok(())
        })
        .unwrap();
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
