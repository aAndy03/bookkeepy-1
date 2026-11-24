// SPDX-License-Identifier: GPL-3.0-or-later
//! Derived balances (ADR-0009: "balance is never stored, it's derived from the movements
//! of money"). Reversal entries net their originals out inside the SUM itself — metrics
//! never filter posted rows by a status flag (ADR-0024), so the trial balance holds for any
//! value-date prefix (INV-LEDGER-001). `as_of` filters on `value_ts` (reports use value time).

use bookkeepy_core::time::Timestamp;
use bookkeepy_core::{CurrencyCode, EntityId, Money};
use bookkeepy_storage::api::conn::Db;

use crate::api::accounts::{self, AccountType};
use crate::api::error::LedgerError;

/// Balance of one account at `as_of` (None = all time). Signed minor units in the
/// account's currency; sign follows the account's normal balance (positive = more of it).
pub fn account_balance(
    db: &Db,
    account_id: EntityId,
    as_of: Option<Timestamp>,
) -> Result<Money, LedgerError> {
    let acc = accounts::get(db, account_id)?.ok_or(LedgerError::UnknownAccount(account_id))?;
    let signed: i64 = db.with_conn(|c| {
        let base = "SELECT COALESCE(SUM(CASE WHEN p.side='debit' THEN p.amount_minor ELSE -p.amount_minor END),0)
                    FROM postings p JOIN journal_entries e ON e.id = p.entry_id
                    WHERE p.account_id=?1";
        let (sql, params): (String, Vec<String>) = match &as_of {
            Some(ts) => (
                format!("{base} AND e.value_ts <= ?2"),
                vec![account_id.to_string(), ts.to_string()],
            ),
            None => (base.to_string(), vec![account_id.to_string()]),
        };
        let mut stmt = c.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        Ok(stmt.query_row(rusqlite::params_from_iter(refs), |r| r.get(0))?)
    })?;
    let balance = if acc.ty.normal_debit() {
        signed
    } else {
        -signed
    };
    Ok(Money::new(balance, acc.currency))
}

/// All account balances at `as_of`.
pub fn account_balances(
    db: &Db,
    as_of: Option<Timestamp>,
) -> Result<Vec<(EntityId, Money)>, LedgerError> {
    let mut out = Vec::new();
    for a in accounts::list(db, true)? {
        out.push((a.id, account_balance(db, a.id, as_of)?));
    }
    Ok(out)
}

/// Trial balance at `as_of`: per currency, debits minus credits across ALL postings
/// (reversals included — they net out). INV-LEDGER-001: every diff must be 0 for any
/// prefix of the journal.
pub fn trial_balance(
    db: &Db,
    as_of: Option<Timestamp>,
) -> Result<Vec<(CurrencyCode, i64)>, LedgerError> {
    Ok(db.with_conn(|c| {
        let base = "SELECT p.currency, SUM(CASE WHEN p.side='debit' THEN p.amount_minor ELSE -p.amount_minor END)
                    FROM postings p JOIN journal_entries e ON e.id = p.entry_id";
        let (sql, params): (String, Vec<String>) = match &as_of {
            Some(ts) => (
                format!("{base} WHERE e.value_ts <= ?1 GROUP BY p.currency"),
                vec![ts.to_string()],
            ),
            None => (format!("{base} GROUP BY p.currency"), vec![]),
        };
        let mut stmt = c.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = stmt.query_map(rusqlite::params_from_iter(refs), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (cur, diff) = row?;
            let cur = CurrencyCode::new(&cur).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::other("bad currency")),
                )
            })?;
            out.push((cur, diff));
        }
        Ok(out)
    })?)
}

/// Assert the trial balance holds (every per-currency diff is 0) — used by property tests
/// and diagnostics.
pub fn assert_trial_balance(db: &Db, as_of: Option<Timestamp>) -> Result<(), LedgerError> {
    for (cur, diff) in trial_balance(db, as_of)? {
        if diff != 0 {
            return Err(LedgerError::Unbalanced {
                currency: cur.as_str().to_string(),
                diff,
            });
        }
    }
    Ok(())
}

/// Account type helper re-export for report code.
pub fn normal_debit(ty: AccountType) -> bool {
    ty.normal_debit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::accounts::{AccountType, create as create_account};
    use crate::api::entries::{NewEntry, Posting, Side, post};
    use crate::api::transactions::{self, NewTransaction, TxKind};
    use bookkeepy_bus::Bus;
    use bookkeepy_contracts::Actor;
    use bookkeepy_core::time::now;

    #[test]
    fn balances_derive_and_trial_balance_holds() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let a =
            create_account(&db, &bus, &Actor::User, "A", AccountType::Asset, "RON", "s").unwrap();
        let b =
            create_account(&db, &bus, &Actor::User, "B", AccountType::Asset, "RON", "s").unwrap();
        let ron = CurrencyCode::new("RON").unwrap();
        post(
            &db,
            &bus,
            &Actor::User,
            NewEntry {
                tx_id: None,
                value_ts: now(),
                note: None,
                postings: vec![
                    Posting {
                        account_id: b,
                        side: Side::Debit,
                        amount: Money::new(300, ron.clone()),
                    },
                    Posting {
                        account_id: a,
                        side: Side::Credit,
                        amount: Money::new(300, ron),
                    },
                ],
            },
            "t",
        )
        .unwrap();
        assert_eq!(account_balance(&db, b, None).unwrap().amount_minor(), 300);
        assert_eq!(account_balance(&db, a, None).unwrap().amount_minor(), -300);
        assert_trial_balance(&db, None).unwrap();
        // transaction-level expense also keeps trial balance
        let food = crate::api::catalog::create_category(
            &db,
            &bus,
            &Actor::User,
            "Food",
            None,
            AccountType::Expense,
            "s",
        )
        .unwrap();
        transactions::create(
            &db,
            &bus,
            &Actor::User,
            NewTransaction {
                kind: TxKind::Expense,
                amount: Money::new(50, CurrencyCode::new("RON").unwrap()),
                from: Some(b),
                to: None,
                category_id: Some(food),
                payee_id: None,
                tag_ids: vec![],
                note: None,
                value_ts: now(),
                status: crate::api::lifecycle::TxStatus::Cleared,
                schedule_id: None,
                occurrence_date: None,
            },
            "x",
        )
        .unwrap();
        assert_trial_balance(&db, None).unwrap();
        assert_eq!(account_balance(&db, b, None).unwrap().amount_minor(), 250);
    }
}
