// SPDX-License-Identifier: GPL-3.0-or-later
//! The shared action engines (RULE-SHARED-001). Each action exists in two forms:
//!
//! - `*_in_tx(&Transaction, …)` — the raw column change, for composing engines (e.g. the
//!   ledger) that must run several actions + their audit/outbox rows in ONE transaction;
//! - `*(db, bus, actor, …)` — the public wrapper: validates, then runs the `_in_tx` variant
//!   inside `storage::tx::post_with` (change + outbox + audit atomically, then publish).
//!
//! Every action is the same shape: `UPDATE <table> SET <column> = <value> WHERE id = ?`.
//! Money-affecting "delete" of posted things is NOT here — it posts a linked reversal in
//! the ledger engine (ADR-0010); `change_status` to `voided` is orchestrated there.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_core::EntityId;
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::tx;
use rusqlite::Transaction;

use super::error::SharedError;

/// Tables the shared actions may touch (allowlist — table names come from `Actionable`
/// impls, this list is the guard against SQL injection through `table()`).
pub const ACTION_TABLES: &[&str] = &["accounts", "categories", "tags", "parties", "transactions"];

/// Tables with an `archived` column.
const ARCHIVABLE: &[&str] = &["accounts", "categories", "tags", "parties"];

/// Tables with a `status` column.
const STATUSED: &[&str] = &["transactions"];

/// Tables with a `name` column.
const RENAMABLE: &[&str] = &["accounts", "categories", "tags", "parties"];

/// An entity the shared actions can operate on.
pub trait Actionable {
    /// Table name — must be in [`ACTION_TABLES`].
    fn table(&self) -> &'static str;
    /// Entity id.
    fn id(&self) -> EntityId;
}

fn check(table: &'static str, allowed: &[&str]) -> Result<(), SharedError> {
    if !ACTION_TABLES.contains(&table) || !allowed.contains(&table) {
        return Err(SharedError::NotApplicable(table));
    }
    Ok(())
}

/// One shared action, fully described: set `column` = `value` on the entity's row, audited
/// as event `kind` with `payload`.
struct Act<'a> {
    kind: &'a str,
    entity: &'a dyn Actionable,
    reason: &'a str,
    payload: serde_json::Value,
    allowed_tables: &'static [&'static str],
    column: &'static str,
    value: &'a str,
}

fn run_in_tx(tx: &Transaction, act: &Act) -> Result<(), SharedError> {
    let table = act.entity.table();
    check(table, act.allowed_tables)?;
    let sql = format!("UPDATE {table} SET {} = ?2 WHERE id = ?1", act.column);
    let changed = tx.execute(
        &sql,
        rusqlite::params![act.entity.id().to_string(), act.value.to_string()],
    )?;
    if changed == 0 {
        return Err(SharedError::UnknownEntity {
            table,
            id: act.entity.id(),
        });
    }
    Ok(())
}

fn public(db: &Db, bus: &Bus, actor: &Actor, act: &Act) -> Result<(), SharedError> {
    let table = act.entity.table();
    // Semantic checks BEFORE the posting transaction so BK-ACTION codes survive.
    check(table, act.allowed_tables)?;
    // Existence pre-check (the connection is mutex-guarded; the UPDATE itself runs in the
    // same posting transaction as the audit row).
    let exists: Option<i64> = db.with_conn(|c| {
        Ok(c.query_row(
            &format!("SELECT 1 FROM {table} WHERE id=?1"),
            [&act.entity.id().to_string()],
            |r| r.get(0),
        )
        .ok())
    })?;
    if exists.is_none() {
        return Err(SharedError::UnknownEntity {
            table,
            id: act.entity.id(),
        });
    }
    let ev = EventEnvelope::new(act.kind, actor.clone(), act.reason, act.payload.clone());
    tx::post_with(db, bus, &ev, |tx| {
        run_in_tx(tx, act).map_err(super::error::into_storage)?;
        Ok(())
    })
    .map_err(super::error::from_storage)?;
    tracing::info!(
        kind = act.kind,
        table = act.entity.table(),
        id = %act.entity.id(),
        "shared action applied (audited)"
    );
    Ok(())
}

/// Rename, inside an explicit transaction.
pub fn rename_in_tx(
    tx: &Transaction,
    entity: &impl Actionable,
    new_name: &str,
) -> Result<(), SharedError> {
    run_in_tx(
        tx,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_RENAMED,
            entity,
            reason: "",
            payload: serde_json::Value::Null,
            allowed_tables: RENAMABLE,
            column: "name",
            value: new_name,
        },
    )
}

/// Rename an entity (audited event `entity.renamed`).
pub fn rename(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    entity: &impl Actionable,
    new_name: &str,
    reason: &str,
) -> Result<(), SharedError> {
    let table = entity.table();
    let old: Option<String> = db.with_conn(|c| {
        Ok(c.query_row(
            &format!("SELECT name FROM {table} WHERE id=?1"),
            [&entity.id().to_string()],
            |r| r.get(0),
        )
        .ok())
    })?;
    let Some(old) = old else {
        return Err(SharedError::UnknownEntity {
            table,
            id: entity.id(),
        });
    };
    public(
        db,
        bus,
        actor,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_RENAMED,
            entity,
            reason,
            payload: serde_json::json!({"table": table, "id": entity.id().to_string(), "old": old, "new": new_name}),
            allowed_tables: RENAMABLE,
            column: "name",
            value: new_name,
        },
    )
}

/// Archive, inside an explicit transaction (soft delete — ADR-0024).
pub fn archive_in_tx(tx: &Transaction, entity: &impl Actionable) -> Result<(), SharedError> {
    run_in_tx(
        tx,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_ARCHIVED,
            entity,
            reason: "",
            payload: serde_json::Value::Null,
            allowed_tables: ARCHIVABLE,
            column: "archived",
            value: "1",
        },
    )
}

/// Archive an entity — soft delete (ADR-0024): excluded from pickers/new entries and
/// forward projections, kept in history. Audited event `entity.archived`.
pub fn archive(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    entity: &impl Actionable,
    reason: &str,
) -> Result<(), SharedError> {
    public(
        db,
        bus,
        actor,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_ARCHIVED,
            entity,
            reason,
            payload: serde_json::json!({"table": entity.table(), "id": entity.id().to_string()}),
            allowed_tables: ARCHIVABLE,
            column: "archived",
            value: "1",
        },
    )
}

/// Restore, inside an explicit transaction.
pub fn restore_in_tx(tx: &Transaction, entity: &impl Actionable) -> Result<(), SharedError> {
    run_in_tx(
        tx,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_RESTORED,
            entity,
            reason: "",
            payload: serde_json::Value::Null,
            allowed_tables: ARCHIVABLE,
            column: "archived",
            value: "0",
        },
    )
}

/// Restore an archived entity. Audited event `entity.restored`.
pub fn restore(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    entity: &impl Actionable,
    reason: &str,
) -> Result<(), SharedError> {
    public(
        db,
        bus,
        actor,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_RESTORED,
            entity,
            reason,
            payload: serde_json::json!({"table": entity.table(), "id": entity.id().to_string()}),
            allowed_tables: ARCHIVABLE,
            column: "archived",
            value: "0",
        },
    )
}

/// Change `status`, inside an explicit transaction.
pub fn change_status_in_tx(
    tx: &Transaction,
    entity: &impl Actionable,
    new_status: &str,
) -> Result<(), SharedError> {
    run_in_tx(
        tx,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_STATUS_CHANGED,
            entity,
            reason: "",
            payload: serde_json::Value::Null,
            allowed_tables: STATUSED,
            column: "status",
            value: new_status,
        },
    )
}

/// Change the `status` column (transactions). Audited event `entity.status_changed`.
/// Money-affecting transitions (e.g. to `voided`) are orchestrated by the ledger engine,
/// which posts the linked reversal in the same transaction.
pub fn change_status(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    entity: &impl Actionable,
    new_status: &str,
    reason: &str,
) -> Result<(), SharedError> {
    public(
        db,
        bus,
        actor,
        &Act {
            kind: bookkeepy_contracts::kinds::ENTITY_STATUS_CHANGED,
            entity,
            reason,
            payload: serde_json::json!({"table": entity.table(), "id": entity.id().to_string(), "status": new_status}),
            allowed_tables: STATUSED,
            column: "status",
            value: new_status,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_storage::api::conn::Db;

    struct Acct(EntityId);
    impl Actionable for Acct {
        fn table(&self) -> &'static str {
            "accounts"
        }
        fn id(&self) -> EntityId {
            self.0
        }
    }

    fn setup() -> (tempfile::TempDir, Db, Bus, Acct) {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let id = EntityId::new();
        db.with_conn(|c| {
            c.execute(
                "INSERT INTO accounts (id, name, type, currency, archived, is_system, created_at) VALUES (?1,'Cash','asset','RON',0,0,'2026-11-24T10:00:00Z')",
                [id.to_string()],
            )?;
            Ok(())
        })
        .unwrap();
        (tmp, db, Bus::new(), Acct(id))
    }

    #[test]
    fn rename_archive_restore_are_audited_and_atomic() {
        let (_t, db, bus, a) = setup();
        rename(&db, &bus, &Actor::User, &a, "Wallet", "user rename").unwrap();
        archive(&db, &bus, &Actor::User, &a, "done using").unwrap();
        restore(&db, &bus, &Actor::User, &a, "needed again").unwrap();
        let (name, archived): (String, i64) = db
            .with_conn(|c| {
                Ok(c.query_row(
                    "SELECT name, archived FROM accounts WHERE id=?1",
                    [a.0.to_string()],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!(name, "Wallet");
        assert_eq!(archived, 0);
        let ghost = Acct(EntityId::new());
        let err = rename(&db, &bus, &Actor::User, &ghost, "x", "y").unwrap_err();
        assert_eq!(err.code(), "BK-ACTION-0001");
        assert_eq!(
            bookkeepy_storage::api::outbox::pending_count(&db).unwrap(),
            0
        );
    }

    #[test]
    fn action_on_wrong_table_is_refused() {
        let (_t, db, bus, a) = setup();
        let err = change_status(&db, &bus, &Actor::User, &a, "voided", "x").unwrap_err();
        assert_eq!(err.code(), "BK-ACTION-0001");
    }
}
