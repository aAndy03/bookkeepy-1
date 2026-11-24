// SPDX-License-Identifier: GPL-3.0-or-later
//! User-level transactions (F2) compiled into balanced double-entry entries (ADR-0009):
//! expense = credit source account + debit category account; income = debit destination +
//! credit category account; transfer = credit from + debit to (same currency in M2 — FX
//! conversions land in M4). Splits/fees are extra postings to their own categories.
//!
//! Statuses (ADR-0024): `pending/cleared/reconciled` are metadata (never affect balances);
//! `voided` = "deleted" — the linked reversal entry nets it out of every computed metric.
//! Editing a posted transaction posts a reversal + a replacement transaction (`replaces`
//! chain), all in one audited transaction.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_core::time::{Timestamp, now};
use bookkeepy_core::{EntityId, Money};
use bookkeepy_shared::{Actionable, change_status_in_tx};
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::tx;
use rusqlite::{OptionalExtension, Transaction};

use crate::api::catalog;
use crate::api::entries::{self, NewEntry, Posting, Side};
use crate::api::error::LedgerError;
use crate::api::lifecycle::TxStatus;

/// Transaction kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxKind {
    /// Money in.
    Income,
    /// Money out.
    Expense,
    /// Move between own accounts.
    Transfer,
}

impl TxKind {
    fn as_str(self) -> &'static str {
        match self {
            TxKind::Income => "income",
            TxKind::Expense => "expense",
            TxKind::Transfer => "transfer",
        }
    }
}

/// A new user transaction.
#[derive(Debug, Clone)]
pub struct NewTransaction {
    /// Kind.
    pub kind: TxKind,
    /// Amount (in the involved accounts' currency).
    pub amount: Money,
    /// Source account (expense/transfer).
    pub from: Option<EntityId>,
    /// Destination account (income/transfer).
    pub to: Option<EntityId>,
    /// Category (income/expense; its backing account balances the entry).
    pub category_id: Option<EntityId>,
    /// Payee.
    pub payee_id: Option<EntityId>,
    /// Tags.
    pub tag_ids: Vec<EntityId>,
    /// Note.
    pub note: Option<String>,
    /// Value time.
    pub value_ts: Timestamp,
    /// Initial status (default cleared).
    pub status: TxStatus,
    /// Owning schedule, if materialized from one.
    pub schedule_id: Option<EntityId>,
    /// Occurrence date, if materialized from a schedule.
    pub occurrence_date: Option<jiff::civil::Date>,
}

/// A transaction row (the user-facing record; `entry_id` points at its posted entry).
#[derive(Debug, Clone)]
pub struct TransactionRow {
    /// Id.
    pub id: EntityId,
    /// Kind.
    pub kind: TxKind,
    /// Status.
    pub status: TxStatus,
    /// Value time.
    pub value_ts: Timestamp,
    /// Posted journal entry.
    pub entry_id: Option<EntityId>,
    /// Replaces (edit chain).
    pub replaces: Option<EntityId>,
}

impl Actionable for TransactionRow {
    fn table(&self) -> &'static str {
        "transactions"
    }
    fn id(&self) -> EntityId {
        self.id
    }
}

/// Compile a user transaction into balanced postings.
pub fn compile(new: &NewTransaction, db: &Db) -> Result<Vec<Posting>, LedgerError> {
    let amt = new.amount.clone();
    match new.kind {
        TxKind::Expense => {
            let from = new.from.ok_or_else(|| {
                LedgerError::InvalidTransaction("expense needs a source account".into())
            })?;
            let cat = new.category_id.ok_or_else(|| {
                LedgerError::InvalidTransaction("expense needs a category".into())
            })?;
            let cat_acc = catalog::get_category(db, cat)?
                .ok_or_else(|| LedgerError::UnknownCatalog(format!("category {cat}")))?
                .account_id;
            Ok(vec![
                Posting {
                    account_id: from,
                    side: Side::Credit,
                    amount: amt.clone(),
                },
                Posting {
                    account_id: cat_acc,
                    side: Side::Debit,
                    amount: amt,
                },
            ])
        }
        TxKind::Income => {
            let to = new.to.ok_or_else(|| {
                LedgerError::InvalidTransaction("income needs a destination account".into())
            })?;
            let cat = new
                .category_id
                .ok_or_else(|| LedgerError::InvalidTransaction("income needs a category".into()))?;
            let cat_acc = catalog::get_category(db, cat)?
                .ok_or_else(|| LedgerError::UnknownCatalog(format!("category {cat}")))?
                .account_id;
            Ok(vec![
                Posting {
                    account_id: to,
                    side: Side::Debit,
                    amount: amt.clone(),
                },
                Posting {
                    account_id: cat_acc,
                    side: Side::Credit,
                    amount: amt,
                },
            ])
        }
        TxKind::Transfer => {
            let from = new
                .from
                .ok_or_else(|| LedgerError::InvalidTransaction("transfer needs a source".into()))?;
            let to = new.to.ok_or_else(|| {
                LedgerError::InvalidTransaction("transfer needs a destination".into())
            })?;
            Ok(vec![
                Posting {
                    account_id: to,
                    side: Side::Debit,
                    amount: amt.clone(),
                },
                Posting {
                    account_id: from,
                    side: Side::Credit,
                    amount: amt,
                },
            ])
        }
    }
}

/// Create + post a transaction in one audited transaction (`transaction.posted`).
/// Returns the transaction id.
pub fn create(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    new: NewTransaction,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    create_in_tx(db, bus, actor, new, None, reason).map(|(id, _)| id)
}

/// Create a transaction inside the posting flow; returns (tx_id, entry_id).
fn create_in_tx(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    new: NewTransaction,
    replaces: Option<EntityId>,
    reason: &str,
) -> Result<(EntityId, EntityId), LedgerError> {
    let postings = compile(&new, db)?;
    entries::validate_postings(db, &postings)?;
    let tx_id = EntityId::new();
    let entry_id = EntityId::new();
    let ev = EventEnvelope::new(
        "transaction.posted",
        actor.clone(),
        reason,
        serde_json::json!({
            "tx_id": tx_id.to_string(),
            "entry_id": entry_id.to_string(),
            "kind": new.kind.as_str(),
            "amount": new.amount.amount_minor(),
            "currency": new.amount.currency().as_str(),
            "status": new.status.as_str(),
            "replaces": replaces.map(|r| r.to_string()),
        }),
    );
    let entry = NewEntry {
        tx_id: Some(tx_id),
        value_ts: new.value_ts,
        note: new.note.clone(),
        postings,
    };
    let status = new.status.as_str().to_string();
    let kind = new.kind.as_str().to_string();
    let value_ts = new.value_ts.to_string();
    let payee = new.payee_id.map(|p| p.to_string());
    let cat = new.category_id.map(|c| c.to_string());
    let note = new.note.clone();
    let tags = new.tag_ids.clone();
    let sched = new.schedule_id.map(|s| s.to_string());
    let occ = new.occurrence_date.map(|d| d.to_string());
    tx::post_with(db, bus, &ev, |tx| {
        entries::insert_in_tx(tx, entry_id, "entry", &entry, None)
            .map_err(crate::api::error::into_storage)?;
        tx.execute(
            "INSERT INTO transactions (id, kind, status, value_ts, entry_ts, payee_id, category_id, note, entry_id, replaces, created_at, schedule_id, occurrence_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                tx_id.to_string(),
                kind,
                status,
                value_ts,
                now().to_string(),
                payee,
                cat,
                note,
                entry_id.to_string(),
                replaces.map(|r| r.to_string()),
                now().to_string(),
                sched,
                occ,
            ],
        )?;
        catalog::attach_tags_in_tx(tx, tx_id, &tags).map_err(crate::api::error::into_storage)?;
        Ok(())
    })?;
    Ok((tx_id, entry_id))
}

/// Get a transaction row.
pub fn get(db: &Db, id: EntityId) -> Result<Option<TransactionRow>, LedgerError> {
    Ok(db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT id, kind, status, value_ts, entry_id, replaces FROM transactions WHERE id=?1",
            [id.to_string()],
            |r| {
                let kind: String = r.get(1)?;
                let status: String = r.get(2)?;
                Ok(TransactionRow {
                    id: r.get::<_, String>(0)?.parse().map_err(|_| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::other("bad id")),
                        )
                    })?,
                    kind: match kind.as_str() {
                        "income" => TxKind::Income,
                        "transfer" => TxKind::Transfer,
                        _ => TxKind::Expense,
                    },
                    status: TxStatus::from_db(&status).unwrap_or(TxStatus::Cleared),
                    value_ts: r.get::<_, String>(3)?.parse().map_err(|_| {
                        rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::other("bad ts")),
                        )
                    })?,
                    entry_id: r.get::<_, Option<String>>(4)?.and_then(|s| s.parse().ok()),
                    replaces: r.get::<_, Option<String>>(5)?.and_then(|s| s.parse().ok()),
                })
            },
        )
        .optional()?)
    })?)
}

/// Change a transaction's status. `voided` posts the linked reversal in the SAME
/// transaction (ADR-0024: exclusion from metrics happens via the journal, not a flag).
pub fn set_status(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    tx_id: EntityId,
    new_status: TxStatus,
    reason: &str,
) -> Result<(), LedgerError> {
    let row = get(db, tx_id)?
        .ok_or_else(|| LedgerError::InvalidTransaction(format!("transaction {tx_id} not found")))?;
    if !TxStatus::can_transition(row.status, new_status) {
        return Err(LedgerError::InvalidTransaction(format!(
            "invalid status transition {:?} -> {:?}",
            row.status, new_status
        )));
    }
    let entry_id = row.entry_id.ok_or_else(|| {
        LedgerError::InvalidTransaction(format!("transaction {tx_id} has no posted entry"))
    })?;
    let ev = EventEnvelope::new(
        "transaction.status_changed",
        actor.clone(),
        reason,
        serde_json::json!({"tx_id": tx_id.to_string(), "from": row.status.as_str(), "to": new_status.as_str()}),
    );
    let status_s = new_status.as_str().to_string();
    tx::post_with(db, bus, &ev, |tx| {
        if new_status == TxStatus::Voided {
            // linked reversal, same transaction (reversal reads via this conn)
            reverse_in_tx(tx, entry_id, reason).map_err(crate::api::error::into_storage)?;
        }
        change_status_in_tx(tx, &row, &status_s).map_err(bookkeepy_shared::into_storage)?;
        Ok(())
    })?;
    Ok(())
}

/// Reverse an entry inside an explicit transaction (shared with `entries::reverse`).
pub(crate) fn reverse_in_tx(
    tx: &Transaction,
    entry_id: EntityId,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    let _ = reason;
    let orig: Option<(Option<String>, String, Option<String>)> = tx
        .query_row(
            "SELECT tx_id, value_ts, note FROM journal_entries WHERE id=?1",
            [entry_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((tx_id, value_ts, note)) = orig else {
        return Err(LedgerError::InvalidReversal(format!(
            "entry {entry_id} not found"
        )));
    };
    let already: i64 = tx.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE reversal_of=?1",
        [entry_id.to_string()],
        |r| r.get(0),
    )?;
    if already > 0 {
        return Err(LedgerError::InvalidReversal(format!(
            "entry {entry_id} already reversed"
        )));
    }
    let postings: Vec<(String, String, i64, String)> = {
        let mut stmt = tx.prepare(
            "SELECT account_id, side, amount_minor, currency FROM postings WHERE entry_id=?1 ORDER BY rowid",
        )?;
        stmt.query_map([entry_id.to_string()], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    };
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
    entries::insert_in_tx(tx, id, "reversal", &new, Some(entry_id))?;
    Ok(id)
}

/// Edit a posted transaction: post a linked reversal of the old entry, create the
/// replacement transaction (`replaces` chain), void the old one — one audited transaction
/// (`transaction.replaced`).
pub fn replace(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    old_tx_id: EntityId,
    new: NewTransaction,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    let row = get(db, old_tx_id)?.ok_or_else(|| {
        LedgerError::InvalidTransaction(format!("transaction {old_tx_id} not found"))
    })?;
    if row.status == TxStatus::Voided {
        return Err(LedgerError::InvalidTransaction(
            "cannot edit a voided transaction".into(),
        ));
    }
    let old_entry = row.entry_id.ok_or_else(|| {
        LedgerError::InvalidTransaction(format!("transaction {old_tx_id} has no entry"))
    })?;
    let postings = compile(&new, db)?;
    entries::validate_postings(db, &postings)?;
    let new_tx = EntityId::new();
    let new_entry = EntityId::new();
    let ev = EventEnvelope::new(
        "transaction.replaced",
        actor.clone(),
        reason,
        serde_json::json!({"old_tx": old_tx_id.to_string(), "new_tx": new_tx.to_string()}),
    );
    let entry = NewEntry {
        tx_id: Some(new_tx),
        value_ts: new.value_ts,
        note: new.note.clone(),
        postings,
    };
    let status = new.status.as_str().to_string();
    let kind = new.kind.as_str().to_string();
    let value_ts = new.value_ts.to_string();
    let payee = new.payee_id.map(|p| p.to_string());
    let cat = new.category_id.map(|c| c.to_string());
    let note = new.note.clone();
    let tags = new.tag_ids.clone();
    let sched = new.schedule_id.map(|s| s.to_string());
    let occ = new.occurrence_date.map(|d| d.to_string());
    tx::post_with(db, bus, &ev, |tx| {
        reverse_in_tx(tx, old_entry, reason).map_err(crate::api::error::into_storage)?;
        entries::insert_in_tx(tx, new_entry, "entry", &entry, None)
            .map_err(crate::api::error::into_storage)?;
        tx.execute(
            "INSERT INTO transactions (id, kind, status, value_ts, entry_ts, payee_id, category_id, note, entry_id, replaces, created_at, schedule_id, occurrence_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                new_tx.to_string(),
                kind,
                status,
                value_ts,
                now().to_string(),
                payee,
                cat,
                note,
                new_entry.to_string(),
                Some(old_tx_id.to_string()),
                now().to_string(),
                sched,
                occ,
            ],
        )?;
        catalog::attach_tags_in_tx(tx, new_tx, &tags).map_err(crate::api::error::into_storage)?;
        change_status_in_tx(tx, &row, TxStatus::Voided.as_str())
            .map_err(bookkeepy_shared::into_storage)?;
        Ok(())
    })?;
    Ok(new_tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::accounts::{AccountType, create as create_account};
    use crate::api::balances;
    use bookkeepy_core::CurrencyCode;

    fn setup() -> (tempfile::TempDir, Db, Bus, EntityId, EntityId, EntityId) {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let cash = create_account(
            &db,
            &bus,
            &Actor::User,
            "Cash",
            AccountType::Asset,
            "RON",
            "s",
        )
        .unwrap();
        let bank = create_account(
            &db,
            &bus,
            &Actor::User,
            "Bank",
            AccountType::Asset,
            "RON",
            "s",
        )
        .unwrap();
        let food = catalog::create_category(
            &db,
            &bus,
            &Actor::User,
            "Food",
            None,
            AccountType::Expense,
            "s",
        )
        .unwrap();
        (tmp, db, bus, cash, bank, food)
    }

    #[test]
    fn expense_posts_balanced_and_moves_balance() {
        let (_t, db, bus, cash, _bank, food) = setup();
        let ron = CurrencyCode::new("RON").unwrap();
        let tx = create(
            &db,
            &bus,
            &Actor::User,
            NewTransaction {
                kind: TxKind::Expense,
                amount: Money::new(2500, ron),
                from: Some(cash),
                to: None,
                category_id: Some(food),
                payee_id: None,
                tag_ids: vec![],
                note: Some("groceries".into()),
                value_ts: now(),
                status: TxStatus::Cleared,
                schedule_id: None,
                occurrence_date: None,
            },
            "buy food",
        )
        .unwrap();
        let row = get(&db, tx).unwrap().unwrap();
        assert_eq!(row.status, TxStatus::Cleared);
        let bal = balances::account_balance(&db, cash, None).unwrap();
        assert_eq!(bal.amount_minor(), -2500); // credit-normal? asset is debit-normal; credit reduces
        entries::verify_chains(&db).unwrap();
        // void it -> reversal nets it out
        set_status(&db, &bus, &Actor::User, tx, TxStatus::Voided, "mistake").unwrap();
        let bal = balances::account_balance(&db, cash, None).unwrap();
        assert_eq!(bal.amount_minor(), 0);
        assert!(crate::api::lifecycle::entry_is_reversed(&db, row.entry_id.unwrap()).unwrap());
    }

    #[test]
    fn transfer_and_replace_chain() {
        let (_t, db, bus, cash, bank, _food) = setup();
        let ron = CurrencyCode::new("RON").unwrap();
        let tx = create(
            &db,
            &bus,
            &Actor::User,
            NewTransaction {
                kind: TxKind::Transfer,
                amount: Money::new(10000, ron.clone()),
                from: Some(cash),
                to: Some(bank),
                category_id: None,
                payee_id: None,
                tag_ids: vec![],
                note: None,
                value_ts: now(),
                status: TxStatus::Cleared,
                schedule_id: None,
                occurrence_date: None,
            },
            "move",
        )
        .unwrap();
        assert_eq!(
            balances::account_balance(&db, bank, None)
                .unwrap()
                .amount_minor(),
            10000
        );
        assert_eq!(
            balances::account_balance(&db, cash, None)
                .unwrap()
                .amount_minor(),
            -10000
        );
        // edit: replace with a different amount
        let tx2 = replace(
            &db,
            &bus,
            &Actor::User,
            tx,
            NewTransaction {
                kind: TxKind::Transfer,
                amount: Money::new(7000, ron),
                from: Some(cash),
                to: Some(bank),
                category_id: None,
                payee_id: None,
                tag_ids: vec![],
                note: None,
                value_ts: now(),
                status: TxStatus::Cleared,
                schedule_id: None,
                occurrence_date: None,
            },
            "fix amount",
        )
        .unwrap();
        let old = get(&db, tx).unwrap().unwrap();
        assert_eq!(old.status, TxStatus::Voided);
        let new = get(&db, tx2).unwrap().unwrap();
        assert_eq!(new.replaces, Some(tx));
        assert_eq!(
            balances::account_balance(&db, bank, None)
                .unwrap()
                .amount_minor(),
            7000
        );
        entries::verify_chains(&db).unwrap();
    }
}
