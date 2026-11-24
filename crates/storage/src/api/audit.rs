// SPDX-License-Identifier: GPL-3.0-or-later
//! The audit log (RULE-AUD-001, doc 05 §5.5): append-only, hash-chained (sha256).
//! Rows are immutable — UPDATE/DELETE are rejected by triggers; corrections are new rows.
//! `verify` recomputes every hash (full chain — backups, on-demand Diagnostics); any
//! tampering breaks it (BK-STORE-0006). `verify_incremental` is the boot-time path: it
//! trusts a checkpoint in `meta` and verifies only newer rows, forcing a full verify when
//! the checkpoint is missing/stale or > FULL_VERIFY_EVERY_ROWS accumulated — so a 20-year
//! profile still boots fast (load-on-demand principle).

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

/// Outcome of an incremental verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyOutcome {
    /// Last seq that was verified.
    pub verified_up_to: i64,
    /// Whether a full-chain verification ran (vs only rows after the checkpoint).
    pub full: bool,
}

/// `meta` keys holding the verification checkpoint (engine state, NOT part of the audit).
const CKPT_SEQ: &str = "audit_verified_up_to";
const CKPT_HASH: &str = "audit_verified_hash";

/// Rows may accumulate past the checkpoint before a full re-verify is forced at boot.
pub const FULL_VERIFY_EVERY_ROWS: i64 = 50_000;

/// Verify the whole chain from GENESIS. Err(BK-STORE-0006) at the first broken row.
/// Used for backups and on-demand Diagnostics.
pub fn verify(db: &Db) -> Result<(), StorageError> {
    db.with_conn(|c| {
        verify_range(c, 0, "GENESIS")?;
        Ok(())
    })
}

/// Incremental boot-time verification: if the `meta` checkpoint is present and the row at
/// that seq still hashes to the checkpoint hash, verify only rows after it; otherwise (or
/// when more than [`FULL_VERIFY_EVERY_ROWS`] accumulated) verify the full chain. On
/// success the checkpoint advances to the last verified row. Sound because the chain is
/// hash-linked; tampering with `meta` itself is caught by the periodic full verify.
pub fn verify_incremental(db: &Db) -> Result<VerifyOutcome, StorageError> {
    db.with_conn(|c| {
        let total: i64 = c.query_row("SELECT COALESCE(MAX(seq),0) FROM audit_log", [], |r| {
            r.get(0)
        })?;
        let ck = read_checkpoint(c);
        let (start, prev) = match ck {
            Some((seq, hash)) if seq <= total => {
                let row_hash: Option<String> = c
                    .query_row("SELECT hash FROM audit_log WHERE seq=?1", [seq], |r| {
                        r.get(0)
                    })
                    .ok();
                match row_hash {
                    Some(h) if h == hash && seq > 0 => (seq, hash),
                    _ => (0, "GENESIS".to_string()),
                }
            }
            _ => (0, "GENESIS".to_string()),
        };
        let do_full = start == 0 || (total - start) > FULL_VERIFY_EVERY_ROWS;
        let (from, prev) = if do_full {
            (0, "GENESIS".to_string())
        } else {
            (start, prev)
        };
        let (last, last_hash) = verify_range(c, from, &prev)?;
        write_checkpoint(c, last, &last_hash)?;
        Ok(VerifyOutcome {
            verified_up_to: last,
            full: do_full,
        })
    })
}

fn read_checkpoint(c: &Connection) -> Option<(i64, String)> {
    let seq: i64 = c
        .query_row("SELECT value FROM meta WHERE key=?1", [CKPT_SEQ], |r| {
            r.get::<_, String>(0)
        })
        .ok()?
        .parse()
        .ok()?;
    let hash: String = c
        .query_row("SELECT value FROM meta WHERE key=?1", [CKPT_HASH], |r| {
            r.get(0)
        })
        .ok()?;
    Some((seq, hash))
}

fn write_checkpoint(c: &Connection, seq: i64, hash: &str) -> Result<(), StorageError> {
    for (k, v) in [(CKPT_SEQ, seq.to_string()), (CKPT_HASH, hash.to_string())] {
        c.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![k, v],
        )?;
    }
    Ok(())
}

/// Verify rows with seq > `after_seq`, chaining from `prev_hash`. Returns (last seq, last
/// hash) — or `(after_seq, prev_hash)` when there is nothing new.
fn verify_range(
    conn: &Connection,
    after_seq: i64,
    prev_hash: &str,
) -> Result<(i64, String), StorageError> {
    let mut stmt = conn.prepare(
        "SELECT seq, id, kind, occurred_at, actor, reason, payload, prev_hash, hash FROM audit_log WHERE seq > ?1 ORDER BY seq",
    )?;
    let mut prev = prev_hash.to_string();
    let mut last = after_seq;
    let rows = stmt.query_map([after_seq], |r| {
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
        let (seq, id, kind, occurred_at, actor, reason, payload, prev_h, hash) = row?;
        if prev_h != prev {
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
            prev_hash: &prev_h,
        }));
        if expect != hash {
            return Err(StorageError::AuditChainBroken { seq });
        }
        prev = hash.clone();
        last = seq;
    }
    Ok((last, prev))
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
                bookkeepy_contracts::kinds::LEDGER_ENTRY_POSTED,
                "2026-11-24T10:00:00Z",
                "user",
                "t",
                "{}",
            )?;
            append(
                c,
                "id-2",
                bookkeepy_contracts::kinds::LEDGER_ENTRY_POSTED,
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

    #[test]
    fn incremental_verify_uses_and_advances_checkpoint() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        db.with_conn(|c| {
            for i in 1..=3 {
                append(
                    c,
                    &format!("id-{i}"),
                    "k",
                    "2026-11-24T10:00:00Z",
                    "u",
                    "t",
                    "{}",
                )?;
            }
            Ok(())
        })
        .unwrap();
        // first run: no checkpoint -> full verify
        let o1 = verify_incremental(&db).unwrap();
        assert!(o1.full);
        assert_eq!(o1.verified_up_to, 3);
        // second run right away: checkpoint valid -> incremental, nothing new
        let o2 = verify_incremental(&db).unwrap();
        assert!(!o2.full);
        assert_eq!(o2.verified_up_to, 3);
        // append two more rows -> incremental verifies only the new ones
        db.with_conn(|c| {
            for i in 4..=5 {
                append(
                    c,
                    &format!("id-{i}"),
                    "k",
                    "2026-11-24T10:00:00Z",
                    "u",
                    "t",
                    "{}",
                )?;
            }
            Ok(())
        })
        .unwrap();
        let o3 = verify_incremental(&db).unwrap();
        assert!(!o3.full);
        assert_eq!(o3.verified_up_to, 5);
        // full verify still works and agrees
        verify(&db).unwrap();
    }
}
