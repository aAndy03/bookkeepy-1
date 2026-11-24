// SPDX-License-Identifier: GPL-3.0-or-later
//! Accounts (F1, ADR-0009): typed (asset/liability/equity/income/expense), one currency
//! each (validated against `data/currencies.toml`), balance ALWAYS derived (never stored).
//! Normal-balance semantics: asset/expense are debit-normal; liability/equity/income are
//! credit-normal (fintecheng 6.4: "accounts have a type").
//!
//! Archive = soft delete (ADR-0024): archived accounts can't receive NEW postings
//! (BK-LEDGER-0003) but stay in history. Rename/archive go through the shared action
//! engines (RULE-SHARED-001) — localized here.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::Actor;
use bookkeepy_core::time::now;
use bookkeepy_core::{CurrencyCode, EntityId, Money};
use bookkeepy_shared::{Actionable, archive, rename, restore};
use bookkeepy_storage::api::conn::Db;
use rusqlite::Transaction;

use crate::api::error::LedgerError;

/// Account type drives the normal balance side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    /// Debit-normal (cash, bank, card…).
    Asset,
    /// Credit-normal (loans, credit cards owed…).
    Liability,
    /// Credit-normal (opening balances).
    Equity,
    /// Credit-normal — system accounts backing income categories.
    Income,
    /// Debit-normal — system accounts backing expense categories.
    Expense,
}

impl AccountType {
    /// DB string.
    pub fn as_str(self) -> &'static str {
        match self {
            AccountType::Asset => "asset",
            AccountType::Liability => "liability",
            AccountType::Equity => "equity",
            AccountType::Income => "income",
            AccountType::Expense => "expense",
        }
    }

    /// Parse a DB string.
    pub fn from_db(s: &str) -> Option<AccountType> {
        Some(match s {
            "asset" => AccountType::Asset,
            "liability" => AccountType::Liability,
            "equity" => AccountType::Equity,
            "income" => AccountType::Income,
            "expense" => AccountType::Expense,
            _ => return None,
        })
    }

    /// True when debits increase the balance (asset/expense).
    pub fn normal_debit(self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }
}

/// An account row.
#[derive(Debug, Clone)]
pub struct Account {
    /// Id.
    pub id: EntityId,
    /// Display name.
    pub name: String,
    /// Type.
    pub ty: AccountType,
    /// ISO 4217 currency.
    pub currency: CurrencyCode,
    /// Soft-deleted (ADR-0024).
    pub archived: bool,
    /// System account (backs a category / opening balance) — not user-deletable.
    pub is_system: bool,
}

impl Actionable for Account {
    fn table(&self) -> &'static str {
        "accounts"
    }
    fn id(&self) -> EntityId {
        self.id
    }
}

fn row_to_account(r: &rusqlite::Row) -> rusqlite::Result<Account> {
    let ty: String = r.get(2)?;
    let cur: String = r.get(3)?;
    Ok(Account {
        id: r.get::<_, String>(0)?.parse().map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other("bad id")),
            )
        })?,
        name: r.get(1)?,
        ty: AccountType::from_db(&ty).unwrap_or(AccountType::Asset),
        currency: CurrencyCode::new(&cur).map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other("bad currency")),
            )
        })?,
        archived: r.get::<_, i64>(4)? != 0,
        is_system: r.get::<_, i64>(5)? != 0,
    })
}

/// Insert an account inside an explicit transaction (no event of its own — the caller's
/// action posts the audit event). Used for user accounts AND system accounts.
pub fn insert_in_tx(
    tx: &Transaction,
    id: EntityId,
    name: &str,
    ty: AccountType,
    currency: &CurrencyCode,
    archived: bool,
    is_system: bool,
) -> Result<(), LedgerError> {
    tx.execute(
        "INSERT INTO accounts (id, name, type, currency, color, icon, archived, is_system, created_at)
         VALUES (?1, ?2, ?3, ?4, NULL, NULL, ?5, ?6, ?7)",
        rusqlite::params![
            id.to_string(),
            name,
            ty.as_str(),
            currency.as_str(),
            if archived { 1 } else { 0 },
            if is_system { 1 } else { 0 },
            now().to_string()
        ],
    )?;
    Ok(())
}

/// Create a user account (audited event `account.created`; currency validated against the
/// currencies SSOT).
pub fn create(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    name: &str,
    ty: AccountType,
    currency: &str,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    let cur = CurrencyCode::new(currency)
        .map_err(|_| LedgerError::InvalidTransaction(format!("bad currency code {currency:?}")))?;
    if bookkeepy_primary_data::currencies::get(cur.as_str()).is_none() {
        return Err(LedgerError::InvalidTransaction(format!(
            "currency {currency} not in currencies.toml"
        )));
    }
    let id = EntityId::new();
    let ev = bookkeepy_contracts::EventEnvelope::new(
        bookkeepy_contracts::kinds::ACCOUNT_CREATED,
        actor.clone(),
        reason,
        serde_json::json!({"id": id.to_string(), "name": name, "type": ty.as_str(), "currency": cur.as_str()}),
    );
    bookkeepy_storage::api::tx::post_with(db, bus, &ev, |tx| {
        insert_in_tx(tx, id, name, ty, &cur, false, false)
            .map_err(crate::api::error::into_storage)?;
        Ok(())
    })
    .map_err(crate::api::error::from_storage)?;
    Ok(id)
}

/// Get one account.
pub fn get(db: &Db, id: EntityId) -> Result<Option<Account>, LedgerError> {
    Ok(db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT id, name, type, currency, archived, is_system FROM accounts WHERE id=?1",
            [id.to_string()],
            row_to_account,
        )
        .optional()?)
    })?)
}

/// List accounts. `include_archived=false` is the picker/forward-projection view
/// (ADR-0024); history always reads the rows directly.
pub fn list(db: &Db, include_archived: bool) -> Result<Vec<Account>, LedgerError> {
    Ok(db.with_conn(|c| {
        let sql = if include_archived {
            "SELECT id, name, type, currency, archived, is_system FROM accounts ORDER BY name"
        } else {
            "SELECT id, name, type, currency, archived, is_system FROM accounts WHERE archived=0 ORDER BY name"
        };
        let mut stmt = c.prepare(sql)?;
        Ok(stmt
            .query_map([], row_to_account)?
            .collect::<Result<Vec<_>, _>>()?)
    })?)
}

/// Rename an account via the shared action engine (RULE-SHARED-001).
pub fn rename_account(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    account: &Account,
    new_name: &str,
    reason: &str,
) -> Result<(), LedgerError> {
    Ok(rename(db, bus, actor, account, new_name, reason)?)
}

/// Outcome of archiving an account: carries the balance at archive time so the UI can
/// WARN (never block — RULE-UX-001) when an account with money is archived.
#[derive(Debug, Clone)]
pub struct ArchiveOutcome {
    /// Balance of the account at archive time (all time, in the account's currency).
    pub balance: Money,
}

/// Archive (soft delete) an account — never hard-delete one with history (F1, ADR-0024).
/// Returns the balance at archive time as a non-blocking warning signal.
pub fn archive_account(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    account: &Account,
    reason: &str,
) -> Result<ArchiveOutcome, LedgerError> {
    archive(db, bus, actor, account, reason)?;
    let balance = crate::api::balances::account_balance(db, account.id, None)?;
    if balance.amount_minor() != 0 {
        tracing::warn!(
            account = %account.id,
            balance_minor = balance.amount_minor(),
            currency = %balance.currency().as_str(),
            "archiving account with non-zero balance (kept in history; hidden from pickers)"
        );
    }
    Ok(ArchiveOutcome { balance })
}

/// Restore an archived account.
pub fn restore_account(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    account: &Account,
    reason: &str,
) -> Result<(), LedgerError> {
    Ok(restore(db, bus, actor, account, reason)?)
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_rename_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let id = create(
            &db,
            &bus,
            &Actor::User,
            "Cash",
            AccountType::Asset,
            "RON",
            "seed",
        )
        .unwrap();
        // bad currency rejected
        assert!(
            create(
                &db,
                &bus,
                &Actor::User,
                "X",
                AccountType::Asset,
                "NOPE",
                "x"
            )
            .is_err()
        );
        let a = get(&db, id).unwrap().unwrap();
        assert_eq!(a.name, "Cash");
        assert!(AccountType::Asset.normal_debit());
        assert!(!AccountType::Liability.normal_debit());
        rename_account(&db, &bus, &Actor::User, &a, "Wallet", "rename").unwrap();
        archive_account(&db, &bus, &Actor::User, &a, "unused").unwrap();
        assert!(list(&db, false).unwrap().is_empty());
        assert_eq!(list(&db, true).unwrap().len(), 1);
        let a = get(&db, id).unwrap().unwrap();
        restore_account(&db, &bus, &Actor::User, &a, "back").unwrap();
        assert_eq!(list(&db, false).unwrap().len(), 1);
    }
}
