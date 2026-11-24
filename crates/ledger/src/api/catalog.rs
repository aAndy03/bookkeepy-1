// SPDX-License-Identifier: GPL-3.0-or-later
//! Catalog entities (F21 managers): categories, tags, parties. Each category is backed by
//! a hidden SYSTEM account (expense/income type) so that user transactions compile into
//! balanced double-entry postings (ADR-0009: "external world = dedicated accounts") — the
//! category account's balance IS the spending/income per category. Archive = soft delete
//! (ADR-0024) via the shared action engines.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_core::{EntityId, time::now};
use bookkeepy_shared::Actionable;
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::tx;
use rusqlite::{OptionalExtension, Transaction};

use crate::api::accounts::{self, AccountType};
use crate::api::error::LedgerError;

/// A category (with its backing system account id).
#[derive(Debug, Clone)]
pub struct Category {
    /// Id.
    pub id: EntityId,
    /// Name.
    pub name: String,
    /// Parent category, if any.
    pub parent_id: Option<EntityId>,
    /// Backing system account (expense/income) used as the balancing posting.
    pub account_id: EntityId,
    /// Soft-deleted (ADR-0024).
    pub archived: bool,
}

impl Actionable for Category {
    fn table(&self) -> &'static str {
        "categories"
    }
    fn id(&self) -> EntityId {
        self.id
    }
}

/// A tag.
#[derive(Debug, Clone)]
pub struct Tag {
    /// Id.
    pub id: EntityId,
    /// Name.
    pub name: String,
    /// Soft-deleted.
    pub archived: bool,
}

impl Actionable for Tag {
    fn table(&self) -> &'static str {
        "tags"
    }
    fn id(&self) -> EntityId {
        self.id
    }
}

/// A payee.
#[derive(Debug, Clone)]
pub struct Party {
    /// Id.
    pub id: EntityId,
    /// Name.
    pub name: String,
    /// Soft-deleted.
    pub archived: bool,
}

impl Actionable for Party {
    fn table(&self) -> &'static str {
        "parties"
    }
    fn id(&self) -> EntityId {
        self.id
    }
}

fn parse_id(s: String) -> EntityId {
    s.parse().unwrap_or_else(|_| EntityId::new())
}

/// Create a category + its backing system account, atomically with the audit event.
/// `account_type` is Expense (default) or Income.
pub fn create_category(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    name: &str,
    parent_id: Option<EntityId>,
    account_type: AccountType,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    if !matches!(account_type, AccountType::Expense | AccountType::Income) {
        return Err(LedgerError::InvalidTransaction(
            "category account must be expense or income".into(),
        ));
    }
    // category accounts use the profile's default currency (rates/FX land in M4)
    let cur = bookkeepy_primary_data::settings::get()
        .default_currency
        .clone();
    let cur = bookkeepy_core::CurrencyCode::new(&cur)
        .map_err(|_| LedgerError::InvalidTransaction("bad default currency".into()))?;
    if let Some(p) = parent_id {
        let exists: Option<i64> = db.with_conn(|c| {
            Ok(c.query_row(
                "SELECT 1 FROM categories WHERE id=?1",
                [p.to_string()],
                |r| r.get(0),
            )
            .optional()?)
        })?;
        if exists.is_none() {
            return Err(LedgerError::UnknownCatalog(format!("parent category {p}")));
        }
    }
    let id = EntityId::new();
    let account_id = EntityId::new();
    let ev = EventEnvelope::new(
        bookkeepy_contracts::kinds::CATEGORY_CREATED,
        actor.clone(),
        reason,
        serde_json::json!({"id": id.to_string(), "name": name, "account_id": account_id.to_string()}),
    );
    tx::post_with(db, bus, &ev, |tx| {
        accounts::insert_in_tx(
            tx,
            account_id,
            &format!("#{name}"),
            account_type,
            &cur,
            false,
            true,
        )
        .map_err(crate::api::error::into_storage)?;
        tx.execute(
            "INSERT INTO categories (id, name, parent_id, account_id, archived, color, created_at)
             VALUES (?1, ?2, ?3, ?4, 0, NULL, ?5)",
            rusqlite::params![
                id.to_string(),
                name,
                parent_id.map(|p| p.to_string()),
                account_id.to_string(),
                now().to_string()
            ],
        )?;
        Ok(())
    })
    .map_err(crate::api::error::from_storage)?;
    Ok(id)
}

/// Get a category (with its backing account id).
pub fn get_category(db: &Db, id: EntityId) -> Result<Option<Category>, LedgerError> {
    Ok(db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT id, name, parent_id, account_id, archived FROM categories WHERE id=?1",
            [id.to_string()],
            |r| {
                Ok(Category {
                    id: parse_id(r.get(0)?),
                    name: r.get(1)?,
                    parent_id: r.get::<_, Option<String>>(2)?.map(parse_id),
                    account_id: parse_id(r.get(3)?),
                    archived: r.get::<_, i64>(4)? != 0,
                })
            },
        )
        .optional()?)
    })?)
}

/// List categories (`include_archived=false` = picker view, ADR-0024).
pub fn list_categories(db: &Db, include_archived: bool) -> Result<Vec<Category>, LedgerError> {
    Ok(db.with_conn(|c| {
        let sql = if include_archived {
            "SELECT id, name, parent_id, account_id, archived FROM categories ORDER BY name"
        } else {
            "SELECT id, name, parent_id, account_id, archived FROM categories WHERE archived=0 ORDER BY name"
        };
        let mut stmt = c.prepare(sql)?;
        Ok(stmt
            .query_map([], |r| {
                Ok(Category {
                    id: parse_id(r.get(0)?),
                    name: r.get(1)?,
                    parent_id: r.get::<_, Option<String>>(2)?.map(parse_id),
                    account_id: parse_id(r.get(3)?),
                    archived: r.get::<_, i64>(4)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?)
    })?)
}

/// Create a tag (audited `tag.created`).
pub fn create_tag(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    name: &str,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    let id = EntityId::new();
    let ev = EventEnvelope::new(
        bookkeepy_contracts::kinds::TAG_CREATED,
        actor.clone(),
        reason,
        serde_json::json!({"id": id.to_string(), "name": name}),
    );
    tx::post_with(db, bus, &ev, |tx| {
        tx.execute(
            "INSERT INTO tags (id, name, archived, created_at) VALUES (?1, ?2, 0, ?3)",
            rusqlite::params![id.to_string(), name, now().to_string()],
        )?;
        Ok(())
    })
    .map_err(crate::api::error::from_storage)?;
    Ok(id)
}

/// Create a payee (audited `party.created`).
pub fn create_party(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    name: &str,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    let id = EntityId::new();
    let ev = EventEnvelope::new(
        bookkeepy_contracts::kinds::PARTY_CREATED,
        actor.clone(),
        reason,
        serde_json::json!({"id": id.to_string(), "name": name}),
    );
    tx::post_with(db, bus, &ev, |tx| {
        tx.execute(
            "INSERT INTO parties (id, name, archived, created_at) VALUES (?1, ?2, 0, ?3)",
            rusqlite::params![id.to_string(), name, now().to_string()],
        )?;
        Ok(())
    })
    .map_err(crate::api::error::from_storage)?;
    Ok(id)
}

/// List tags / parties (archived excluded unless asked).
pub fn list_simple(
    db: &Db,
    table: &str,
    include_archived: bool,
) -> Result<Vec<(EntityId, String, bool)>, LedgerError> {
    let t = match table {
        "tags" | "parties" => table,
        _ => return Err(LedgerError::UnknownCatalog(table.to_string())),
    };
    Ok(db.with_conn(|c| {
        let sql = format!(
            "SELECT id, name, archived FROM {t} {} ORDER BY name",
            if include_archived {
                ""
            } else {
                "WHERE archived=0"
            }
        );
        let mut stmt = c.prepare(&sql)?;
        Ok(stmt
            .query_map([], |r| {
                Ok((parse_id(r.get(0)?), r.get(1)?, r.get::<_, i64>(2)? != 0))
            })?
            .collect::<Result<Vec<_>, _>>()?)
    })?)
}

/// Attach tags to a transaction (idempotent).
pub fn attach_tags_in_tx(
    tx: &Transaction,
    tx_id: EntityId,
    tag_ids: &[EntityId],
) -> Result<(), LedgerError> {
    for t in tag_ids {
        let exists: Option<i64> = tx
            .query_row("SELECT 1 FROM tags WHERE id=?1", [t.to_string()], |r| {
                r.get(0)
            })
            .optional()?;
        if exists.is_none() {
            return Err(LedgerError::UnknownCatalog(format!("tag {t}")));
        }
        tx.execute(
            "INSERT OR IGNORE INTO tx_tags (tx_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![tx_id.to_string(), t.to_string()],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_storage::api::conn::Db;

    #[test]
    fn category_creates_backing_account() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let c = create_category(
            &db,
            &bus,
            &Actor::User,
            "Food",
            None,
            AccountType::Expense,
            "seed",
        )
        .unwrap();
        let cat = get_category(&db, c).unwrap().unwrap();
        let acc = accounts::get(&db, cat.account_id).unwrap().unwrap();
        assert!(acc.is_system);
        assert_eq!(acc.ty, AccountType::Expense);
        assert_eq!(list_categories(&db, false).unwrap().len(), 1);
        // unknown parent rejected
        assert!(
            create_category(
                &db,
                &bus,
                &Actor::User,
                "X",
                Some(EntityId::new()),
                AccountType::Expense,
                "x"
            )
            .is_err()
        );
        let t = create_tag(&db, &bus, &Actor::User, "work", "seed").unwrap();
        let p = create_party(&db, &bus, &Actor::User, "Mega", "seed").unwrap();
        assert_eq!(list_simple(&db, "tags", false).unwrap().len(), 1);
        assert_eq!(list_simple(&db, "parties", false).unwrap().len(), 1);
        let _ = (t, p);
    }
}
