// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Cross-layer seam contract tests (RULE-SEAM-001, ADR-0025). These pin the behavior of
//! the sanctioned boundaries so the bug classes from the M2-hardening audit cannot
//! regress: engine BK-codes must survive the `StorageError::Foreign` closure boundary,
//! posting must be atomic across audit+outbox+bus, and shared actions must report their
//! own code. `cargo xtask govern seams` enforces the same rules statically.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope, kinds};
use bookkeepy_core::time::now;
use bookkeepy_core::{CurrencyCode, EntityId, Money};
use bookkeepy_ledger::api::accounts::{self, AccountType};
use bookkeepy_ledger::api::catalog;
use bookkeepy_ledger::api::error::LedgerError;
use bookkeepy_ledger::api::lifecycle::TxStatus;
use bookkeepy_ledger::api::transactions::{self, NewTransaction, TxKind};
use bookkeepy_shared::{Actionable, archive};
use bookkeepy_storage::api::audit;
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::outbox;
use bookkeepy_storage::api::tx;
use std::sync::{Arc, Mutex};

fn setup() -> (tempfile::TempDir, Db, Bus, EntityId, EntityId) {
    let tmp = tempfile::tempdir().unwrap();
    let db = Db::open(&tmp.path().join("app.db")).unwrap();
    let bus = Bus::new();
    let cash = accounts::create(
        &db,
        &bus,
        &Actor::User,
        "Cash",
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
    (tmp, db, bus, cash, food)
}

/// An engine error raised INSIDE the posting closure must keep its BK-code when it
/// crosses the storage boundary (S2). Unknown tag -> BK-LEDGER-0006, not BK-STORE-0001.
#[test]
fn engine_error_code_survives_boundary() {
    let (_t, db, bus, cash, food) = setup();
    let ron = CurrencyCode::new("RON").unwrap();
    let ghost_tag = EntityId::new();
    let audit_before: i64 = db
        .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))?))
        .unwrap();
    let err = transactions::create(
        &db,
        &bus,
        &Actor::User,
        NewTransaction {
            kind: TxKind::Expense,
            amount: Money::new(100, ron),
            from: Some(cash),
            to: None,
            category_id: Some(food),
            payee_id: None,
            tag_ids: vec![ghost_tag],
            note: None,
            value_ts: now(),
            status: TxStatus::Cleared,
            schedule_id: None,
            occurrence_date: None,
        },
        "bad tag",
    )
    .unwrap_err();
    assert_eq!(err.code(), "BK-LEDGER-0006", "got: {err}");
    // and nothing was persisted (atomic rollback)
    let n: i64 = db
        .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(n, audit_before); // rollback left setup events only
}

/// Shared action on a missing entity reports BK-ACTION-0001 (its own code, not a storage one).
#[test]
fn shared_action_code_survives_boundary() {
    struct Ghost(EntityId);
    impl Actionable for Ghost {
        fn table(&self) -> &'static str {
            "accounts"
        }
        fn id(&self) -> EntityId {
            self.0
        }
    }
    let (_t, db, bus, _c, _f) = setup();
    let err = archive(&db, &bus, &Actor::User, &Ghost(EntityId::new()), "ghost").unwrap_err();
    assert_eq!(err.code(), "BK-ACTION-0001", "got: {err}");
}

/// Posting one event is atomic across audit + outbox + bus (the M1 seam).
#[test]
fn post_is_atomic_across_audit_outbox_bus() {
    let (_t, db, bus, _c, _f) = setup();
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    bus.subscribe(move |ev| s2.lock().unwrap().push(ev.kind.clone()));
    let ev = EventEnvelope::new(
        kinds::TRANSACTION_POSTED,
        Actor::User,
        "t",
        serde_json::json!({"x":1}),
    );
    let seq = tx::post(&db, &bus, &ev).unwrap();
    assert!(seq > 0); // setup already posted account.created + category.created
    audit::verify(&db).unwrap(); // chain intact
    assert_eq!(outbox::pending_count(&db).unwrap(), 0); // published
    assert_eq!(seen.lock().unwrap().as_slice(), [kinds::TRANSACTION_POSTED]);
}

/// A ledger-level error code still round-trips through StorageError::Foreign.
#[test]
fn foreign_roundtrip_preserves_code() {
    let le = LedgerError::AlreadyMaterialized("2026-01-01".into());
    let se = bookkeepy_ledger::api::error::into_storage(le);
    let back = bookkeepy_ledger::api::error::from_storage(se);
    assert_eq!(back.code(), "BK-RECUR-0003");
}
