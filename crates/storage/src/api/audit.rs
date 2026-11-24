// SPDX-License-Identifier: GPL-3.0-or-later
//! The audit log (RULE-AUD-001, doc 05 §5.5): append-only, hash-chained (sha256).
//! Rows are immutable — UPDATE/DELETE are rejected by triggers; corrections are new rows.
//! `verify` recomputes every hash; any tampering breaks it (BK-STORE-0006).

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::api::conn::Db;
use crate::api::error::StorageError;

/// The fields hashed into one audit row (order fixed).
struct ChainRow<'a> {
    seq: i64,
    id: &'a str,
    kind: &'a str,
    occurred_at: &'a str,
    actor: &'a str,
    reason: &'a str,
    payload: &'a str,
    prev_hash: &'a str,
}

/// Canonical string that gets hashed for one audit row (order fixed).
fn chain_input(r: &ChainRow<'_>) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        r.seq, r.id, r.kind, r.occurred_at, r.actor, r.reason, r.payload, r.prev_hash
    )
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

/// Append an audit row; returns its seq. Call inside the posting transaction.
pub fn append(
    conn: &Connection,
    id: &str,
    kind: &str,
    occurred_at: &str,
    actor: &str,
    reason: &str,
    payload: &str,
) -> Result<i64, StorageError> {
    let prev: String = conn
        .query_row(
            "SELECT hash FROM audit_log ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "GENESIS".to_string());
    let seq: i64 = conn.query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM audit_log", [], |r| {
        r.get(0)
    })?;
    let hash = sha256_hex(&chain_input(&ChainRow {
        seq,
        id,
        kind,
        occurred_at,
        actor,
        reason,
        payload,
        prev_hash: &prev,
    }));
    conn.execute(
        "INSERT INTO audit_log (seq, id, kind, occurred_at, actor, reason, payload, prev_hash, hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![seq, id, kind, occurred_at, actor, reason, payload, prev, hash],
    )?;
    Ok(seq)
}

/// Verify the whole chain; Err(BK-STORE-0006) at the first broken row.
pub fn verify(db: &Db) -> Result<(), StorageError> {
    db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT seq, id, kind, occurred_at, actor, reason, payload, prev_hash, hash FROM audit_log ORDER BY seq",
        )?;
        let mut prev = "GENESIS".to_string();
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
            ))
        })?;
        for row in rows {
            let (seq, id, kind, occurred_at, actor, reason, payload, prev_hash, hash) = row?;
            if prev_hash != prev {
                return Err(StorageError::AuditChainBroken { seq });
            }
            let expect = sha256_hex(&chain_input(&ChainRow {
                seq,
                id: &id,
                kind: &kind,
                occurred_at: &occurred_at,
                actor: &actor,
                reason: &reason,
                payload: &payload,
                prev_hash: &prev_hash,
            }));
            if expect != hash {
                return Err(StorageError::AuditChainBroken { seq });
            }
            prev = hash;
        }
        Ok(())
    })
}

/// Install the immutability triggers (idempotent). Called by `migrate` after the schema.
pub fn install_guard_triggers(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS audit_no_update BEFORE UPDATE ON audit_log
        BEGIN SELECT RAISE(ABORT, 'audit_log is immutable (RULE-AUD-001)'); END;
        CREATE TRIGGER IF NOT EXISTS audit_no_delete BEFORE DELETE ON audit_log
        BEGIN SELECT RAISE(ABORT, 'audit_log is immutable (RULE-AUD-001)'); END;
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_verifies_and_tamper_breaks() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("app.db");
        let db = Db::open(&path).unwrap();
        db.with_conn(|c| {
            append(
                c,
                "id-1",
                "ledger.posted",
                "2026-11-24T10:00:00Z",
                "user",
                "t",
                "{}",
            )?;
            append(
                c,
                "id-2",
                "ledger.posted",
                "2026-11-24T10:01:00Z",
                "user",
                "t",
                "{}",
            )?;
            c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
            Ok(())
        })
        .unwrap();
        verify(&db).unwrap();
        // triggers reject direct tampering on a live connection
        let conn = Connection::open(&path).unwrap();
        conn.execute("UPDATE audit_log SET reason='tampered' WHERE seq=1", [])
            .expect_err("trigger must reject UPDATE");
        drop(conn);
        drop(db);
        // simulate on-disk tampering on a copy (drop the trigger, flip a row)
        let path2 = tmp.path().join("evil.db");
        std::fs::copy(&path, &path2).unwrap();
        let conn = Connection::open(&path2).unwrap();
        conn.execute("DROP TRIGGER audit_no_update", []).unwrap();
        conn.execute("UPDATE audit_log SET reason='tampered' WHERE seq=1", [])
            .unwrap();
        drop(conn);
        let evil = Db::open(&path2).unwrap();
        let err = verify(&evil).unwrap_err();
        assert_eq!(err.code(), "BK-STORE-0006");
    }
}
