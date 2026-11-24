// SPDX-License-Identifier: GPL-3.0-or-later
//! Journal entries + postings (ADR-0009/0010, INV-LEDGER-001 / INV-CHAIN-001).
//!
//! fintecheng (doc 06 §6.4): "money is only moved, never created or destroyed" — every
//! entry balances per currency (debits == credits), enforced at post time. Tables are
//! append-only with DB triggers; each row carries prev_hash/hash (SHA-256 chains per table,
//! ordered by rowid). Corrections are linked reversals — fix-forward, never edits.
//! `value_ts` is user-editable (reports use it); `booked_ts` is system (traceability).

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_core::time::{Timestamp, now};
use bookkeepy_core::{EntityId, Money};
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::tx;
use rusqlite::{Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};

use crate::api::error::LedgerError;

/// Posting side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Debit.
    Debit,
    /// Credit.
    Credit,
}

impl Side {
    fn as_str(self) -> &'static str {
        match self {
            Side::Debit => "debit",
            Side::Credit => "credit",
        }
    }
    pub(crate) fn flipped(self) -> Side {
        match self {
            Side::Debit => Side::Credit,
            Side::Credit => Side::Debit,
        }
    }
}

/// One movement of money.
#[derive(Debug, Clone)]
pub struct Posting {
    /// Account.
    pub account_id: EntityId,
    /// Side.
    pub side: Side,
    /// Amount (positive minor units; currency must equal the account's).
    pub amount: Money,
}

/// A new journal entry to post.
#[derive(Debug, Clone)]
pub struct NewEntry {
    /// Owning user transaction, if any.
    pub tx_id: Option<EntityId>,
    /// Value time (when it happened — reports use this).
    pub value_ts: Timestamp,
    /// Note (PII-lite; edits go through reversal+replace).
    pub note: Option<String>,
    /// Movements; must balance per currency.
    pub postings: Vec<Posting>,
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

fn last_entry_hash(conn: &Connection) -> Result<String, LedgerError> {
    Ok(conn
        .query_row(
            "SELECT hash FROM journal_entries ORDER BY rowid DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or_else(|| "GENESIS".to_string()))
}

fn last_posting_hash(conn: &Connection) -> Result<String, LedgerError> {
    Ok(conn
        .query_row(
            "SELECT hash FROM postings ORDER BY rowid DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or_else(|| "GENESIS".to_string()))
}

/// Validate postings: positive amounts, known + non-archived accounts, account currency
/// match, and per-currency balance (INV-LEDGER-001).
fn validate(conn: &Connection, postings: &[Posting]) -> Result<(), LedgerError> {
    if postings.is_empty() {
        return Err(LedgerError::InvalidTransaction(
            "entry has no postings".into(),
        ));
    }
    for p in postings {
        if p.amount.amount_minor() <= 0 {
            return Err(LedgerError::AmountNotPositive);
        }
        let row: Option<(String, i64)> = conn
            .query_row(
                "SELECT currency, archived FROM accounts WHERE id=?1",
                [p.account_id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((acct_cur, archived)) = row else {
            return Err(LedgerError::UnknownAccount(p.account_id));
        };
        if archived != 0 {
            return Err(LedgerError::AccountArchived(p.account_id));
        }
        if acct_cur != p.amount.currency().as_str() {
            return Err(LedgerError::CurrencyMismatch {
                expected: acct_cur,
                got: p.amount.currency().as_str().to_string(),
            });
        }
    }
    // balance per currency
    let mut debits: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    let mut credits: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    for p in postings {
        let e = if p.side == Side::Debit {
            &mut debits
        } else {
            &mut credits
        };
        *e.entry(p.amount.currency().as_str()).or_insert(0) += p.amount.amount_minor();
    }
    for (cur, d) in &debits {
        let c = credits.get(cur).copied().unwrap_or(0);
        if *d != c {
            return Err(LedgerError::Unbalanced {
                currency: cur.to_string(),
                diff: d - c,
            });
        }
    }
    for (cur, c) in &credits {
        if !debits.contains_key(cur) {
            return Err(LedgerError::Unbalanced {
                currency: cur.to_string(),
                diff: -*c,
            });
        }
    }
    Ok(())
}

/// Validate postings against the DB (public so transaction compilation can pre-validate
/// before opening the posting transaction — keeps BK-LEDGER codes intact).
pub fn validate_postings(db: &Db, postings: &[Posting]) -> Result<(), LedgerError> {
    db.with_conn(|c| validate(c, postings).map_err(crate::api::error::into_storage))
        .map_err(crate::api::error::from_storage)
}

/// Insert entry + postings inside an explicit transaction, maintaining both hash chains.
/// `kind` is "entry" or "reversal"; `reversal_of` links a reversal to its original.
pub fn insert_in_tx(
    conn: &Transaction,
    id: EntityId,
    kind: &str,
    new: &NewEntry,
    reversal_of: Option<EntityId>,
) -> Result<(), LedgerError> {
    validate(conn, &new.postings)?;
    let prev = last_entry_hash(conn)?;
    let booked = now();
    let entry_input = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        id,
        new.tx_id.map(|t| t.to_string()).unwrap_or_default(),
        kind,
        new.value_ts,
        booked,
        new.note.clone().unwrap_or_default(),
        reversal_of.map(|r| r.to_string()).unwrap_or_default(),
        prev
    );
    let hash = sha256_hex(&entry_input);
    conn.execute(
        "INSERT INTO journal_entries (id, tx_id, kind, value_ts, booked_ts, note, reversal_of, prev_hash, hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            id.to_string(),
            new.tx_id.map(|t| t.to_string()),
            kind,
            new.value_ts.to_string(),
            booked.to_string(),
            new.note,
            reversal_of.map(|r| r.to_string()),
            prev,
            hash,
            booked.to_string()
        ],
    )?;
    let mut prev_p = last_posting_hash(conn)?;
    for (i, p) in new.postings.iter().enumerate() {
        let pid = EntityId::new();
        let input = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            pid,
            id,
            p.account_id,
            p.side.as_str(),
            p.amount.amount_minor(),
            p.amount.currency().as_str(),
            prev_p
        );
        let ph = sha256_hex(&input);
        conn.execute(
            "INSERT INTO postings (id, entry_id, account_id, side, amount_minor, currency, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                pid.to_string(),
                id.to_string(),
                p.account_id.to_string(),
                p.side.as_str(),
                p.amount.amount_minor(),
                p.amount.currency().as_str(),
                prev_p,
                ph
            ],
        )?;
        let _ = i;
        prev_p = ph;
    }
    Ok(())
}

/// Post a new entry (audited `ledger.entry.posted`, published on the bus after commit).
pub fn post(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    new: NewEntry,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    // Semantic validation BEFORE the posting transaction so BK-LEDGER codes survive
    // (inside the tx, errors map to storage codes as defense-in-depth only).
    validate_postings(db, &new.postings)?;
    let id = EntityId::new();
    let ev = EventEnvelope::new(
        bookkeepy_contracts::kinds::LEDGER_ENTRY_POSTED,
        actor.clone(),
        reason,
        serde_json::json!({"entry_id": id.to_string(), "tx_id": new.tx_id.map(|t| t.to_string()), "postings": new.postings.len()}),
    );
    let id2 = id;
    tx::post_with(db, bus, &ev, |tx| {
        insert_in_tx(tx, id2, "entry", &new, None).map_err(crate::api::error::into_storage)?;
        Ok(())
    })?;
    Ok(id)
}

/// Post a linked reversal of `entry_id` (fix-forward correction, ADR-0010). The reversal
/// keeps the original's `value_ts` so the trial balance holds for ANY value-date prefix.
/// Audited `ledger.entry.reversed`.
pub fn reverse(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    entry_id: EntityId,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    // read original inside the same transaction flow: pre-check here, full read in tx
    let orig: Option<(Option<String>, String, Option<String>, String)> = db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT tx_id, value_ts, note, kind FROM journal_entries WHERE id=?1",
            [entry_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?)
    })?;
    let Some((tx_id, value_ts, note, kind)) = orig else {
        return Err(LedgerError::InvalidReversal(format!(
            "entry {entry_id} not found"
        )));
    };
    if kind == "reversal" {
        // reversing a reversal is allowed (re-instates), but it must not be double-reversed
    }
    let already: i64 = db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE reversal_of=?1",
            [entry_id.to_string()],
            |r| r.get(0),
        )?)
    })?;
    if already > 0 {
        return Err(LedgerError::InvalidReversal(format!(
            "entry {entry_id} already reversed"
        )));
    }
    let postings: Vec<(String, String, i64, String)> = db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT account_id, side, amount_minor, currency FROM postings WHERE entry_id=?1 ORDER BY rowid",
        )?;
        Ok(stmt
            .query_map([entry_id.to_string()], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?)
    })?;
    let new = NewEntry {
        tx_id: tx_id.and_then(|t| t.parse().ok()),
        value_ts: value_ts
            .parse()
            .map_err(|_| LedgerError::InvalidReversal(format!("entry {entry_id} bad value_ts")))?,
        note,
        postings: postings
            .iter()
            .map(|(acc, side, amt, cur)| {
                let side = if side == "debit" {
                    Side::Debit
                } else {
                    Side::Credit
                };
                let cur = bookkeepy_core::CurrencyCode::new(cur)
                    .map_err(|_| LedgerError::InvalidReversal("bad currency".into()))?;
                Ok::<_, LedgerError>(Posting {
                    account_id: acc
                        .parse()
                        .map_err(|_| LedgerError::InvalidReversal("bad account id".into()))?,
                    side: side.flipped(),
                    amount: Money::new(*amt, cur),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    };
    let id = EntityId::new();
    let ev = EventEnvelope::new(
        bookkeepy_contracts::kinds::LEDGER_ENTRY_REVERSED,
        actor.clone(),
        reason,
        serde_json::json!({"entry_id": id.to_string(), "reversal_of": entry_id.to_string()}),
    );
    let id2 = id;
    tx::post_with(db, bus, &ev, |tx| {
        insert_in_tx(tx, id2, "reversal", &new, Some(entry_id))
            .map_err(crate::api::error::into_storage)?;
        Ok(())
    })?;
    Ok(id)
}

/// Verify both hash chains (entries, postings) — full recomputation, on demand /
/// diagnostics. Returns the first broken table+rowid.
pub fn verify_chains(db: &Db) -> Result<(), LedgerError> {
    db.with_conn(|c| {
        verify_table(c, "journal_entries").map_err(crate::api::error::into_storage)?;
        verify_table(c, "postings").map_err(crate::api::error::into_storage)?;
        Ok(())
    })
    .map_err(crate::api::error::from_storage)
}

fn verify_table(conn: &Connection, table: &str) -> Result<(), LedgerError> {
    let cols = if table == "journal_entries" {
        "id, tx_id, kind, value_ts, booked_ts, note, reversal_of, prev_hash, hash"
    } else {
        "id, entry_id, account_id, side, amount_minor, currency, prev_hash, hash"
    };
    let sql = format!("SELECT {cols} FROM {table} ORDER BY rowid");
    let mut stmt = conn.prepare(&sql)?;
    let mut prev = "GENESIS".to_string();
    let rows = stmt.query_map([], |r| {
        let n = if table == "journal_entries" { 9 } else { 8 };
        let mut fields: Vec<String> = Vec::with_capacity(n);
        for i in 0..n {
            let v: rusqlite::types::Value = r.get(i)?;
            fields.push(match v {
                rusqlite::types::Value::Null => String::new(),
                rusqlite::types::Value::Integer(x) => x.to_string(),
                rusqlite::types::Value::Real(x) => x.to_string(),
                rusqlite::types::Value::Text(s) => s,
                rusqlite::types::Value::Blob(b) => hex::encode(b),
            });
        }
        Ok(fields)
    })?;
    for row in rows {
        let f = row?;
        let (prev_hash, hash) = (f[f.len() - 2].clone(), f[f.len() - 1].clone());
        if prev_hash != prev {
            return Err(LedgerError::InvalidReversal(format!(
                "chain broken in {table} at {}: prev_hash mismatch",
                f[0]
            )));
        }
        let input = format!("{}|{}", f[..f.len() - 2].join("|"), prev_hash);
        if sha256_hex(&input) != hash {
            return Err(LedgerError::InvalidReversal(format!(
                "chain broken in {table} at {}: hash mismatch",
                f[0]
            )));
        }
        prev = hash;
    }
    Ok(())
}

/// Load the postings of an entry.
pub fn postings_of(db: &Db, entry_id: EntityId) -> Result<Vec<Posting>, LedgerError> {
    Ok(db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT account_id, side, amount_minor, currency FROM postings WHERE entry_id=?1 ORDER BY rowid",
        )?;
        Ok(stmt
            .query_map([entry_id.to_string()], |r| {
                let side: String = r.get(1)?;
                let cur = bookkeepy_core::CurrencyCode::new(&r.get::<_, String>(3)?).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::other("bad currency")),
                    )
                })?;
                Ok(Posting {
                    account_id: r.get::<_, String>(0)?.parse().map_err(|_| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::other("bad id")),
                        )
                    })?,
                    side: if side == "debit" { Side::Debit } else { Side::Credit },
                    amount: Money::new(r.get(2)?, cur),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?)
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::accounts::{AccountType, create as create_account};
    use bookkeepy_core::CurrencyCode;

    fn setup() -> (tempfile::TempDir, Db, Bus, EntityId, EntityId) {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let ron = CurrencyCode::new("RON").unwrap();
        let a = create_account(
            &db,
            &bus,
            &Actor::User,
            "Cash",
            AccountType::Asset,
            "RON",
            "s",
        )
        .unwrap();
        let b = create_account(
            &db,
            &bus,
            &Actor::User,
            "Bank",
            AccountType::Asset,
            "RON",
            "s",
        )
        .unwrap();
        let _ = ron;
        (tmp, db, bus, a, b)
    }

    fn transfer(a: EntityId, b: EntityId, amt: i64) -> NewEntry {
        let ron = CurrencyCode::new("RON").unwrap();
        NewEntry {
            tx_id: None,
            value_ts: now(),
            note: None,
            postings: vec![
                Posting {
                    account_id: b,
                    side: Side::Debit,
                    amount: Money::new(amt, ron.clone()),
                },
                Posting {
                    account_id: a,
                    side: Side::Credit,
                    amount: Money::new(amt, ron),
                },
            ],
        }
    }

    #[test]
    fn post_balances_and_reverses() {
        let (_t, db, bus, a, b) = setup();
        let e = post(&db, &bus, &Actor::User, transfer(a, b, 500), "move").unwrap();
        verify_chains(&db).unwrap();
        // unbalanced rejected
        let ron = CurrencyCode::new("RON").unwrap();
        let bad = NewEntry {
            tx_id: None,
            value_ts: now(),
            note: None,
            postings: vec![Posting {
                account_id: a,
                side: Side::Debit,
                amount: Money::new(1, ron),
            }],
        };
        let err = post(&db, &bus, &Actor::User, bad, "bad").unwrap_err();
        assert_eq!(err.code(), "BK-LEDGER-0001");
        // reverse
        let r = reverse(&db, &bus, &Actor::User, e, "undo").unwrap();
        verify_chains(&db).unwrap();
        assert!(crate::api::lifecycle::entry_is_reversed(&db, e).unwrap());
        // double reversal rejected
        let err = reverse(&db, &bus, &Actor::User, e, "again").unwrap_err();
        assert_eq!(err.code(), "BK-LEDGER-0005");
        let _ = r;
    }

    #[test]
    fn archived_account_rejects_posting() {
        let (_t, db, bus, a, b) = setup();
        let acc = crate::api::accounts::get(&db, a).unwrap().unwrap();
        bookkeepy_shared::archive(&db, &bus, &Actor::User, &acc, "arch").unwrap();
        let err = post(&db, &bus, &Actor::User, transfer(a, b, 10), "x").unwrap_err();
        assert_eq!(err.code(), "BK-LEDGER-0003");
    }
}
