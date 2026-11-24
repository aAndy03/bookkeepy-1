// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! M3 backend flow tests (RULE-TEST-001 guards): App::open runs the injected boot checks,
//! the due-schedule driver, and the job worker; ledger writes work through the app handle;
//! unknown job kinds fail fast instead of spinning.

use bookkeepy_app_services::App;
use bookkeepy_contracts::Actor;
use bookkeepy_core::time::now;
use bookkeepy_core::{CurrencyCode, Money};
use bookkeepy_ledger::api::accounts::{self, AccountType};
use bookkeepy_ledger::api::catalog;
use bookkeepy_ledger::api::lifecycle::TxStatus;
use bookkeepy_ledger::api::schedules::{self, ScheduleTemplate};
use bookkeepy_ledger::api::transactions::{self, NewTransaction, TxKind};
use bookkeepy_ledger::api::{self, schedules::TxKindSerde};
use bookkeepy_profile::Registry;
use bookkeepy_shared::api::recurrence::{EndRule, Frequency, ScheduleRule};
use bookkeepy_storage::api::jobs::{self, JobState};
use jiff::civil::Date;

fn open_app() -> (tempfile::TempDir, App) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let mut reg = Registry::load(dir).unwrap();
    let meta = reg.create_profile(dir, "Personal", "#3366ff").unwrap();
    let app = App::open(dir, meta.id).unwrap();
    (tmp, app)
}

#[test]
fn open_runs_injected_boot_checks_and_ledger_writes_work() {
    let (_t, app) = open_app();
    // the injected higher-layer checks landed in the boot report
    let chains = app
        .session
        .report
        .extra
        .iter()
        .find(|c| c.name == "ledger.chains")
        .unwrap();
    assert!(chains.ok, "{:?}", chains.detail);
    assert!(
        app.session
            .report
            .extra
            .iter()
            .any(|c| c.name == "schedules")
    );
    // a ledger write through the app handle
    let cash = accounts::create(
        app.db(),
        app.bus(),
        &Actor::User,
        "Cash",
        AccountType::Asset,
        "RON",
        "t",
    )
    .unwrap();
    let food = catalog::create_category(
        app.db(),
        app.bus(),
        &Actor::User,
        "Food",
        None,
        AccountType::Expense,
        "t",
    )
    .unwrap();
    let ron = CurrencyCode::new("RON").unwrap();
    let tx = transactions::create(
        app.db(),
        app.bus(),
        &Actor::User,
        NewTransaction {
            kind: TxKind::Expense,
            amount: Money::new(500, ron),
            from: Some(cash),
            to: None,
            category_id: Some(food),
            payee_id: None,
            tag_ids: vec![],
            note: None,
            value_ts: now(),
            status: TxStatus::Cleared,
            schedule_id: None,
            occurrence_date: None,
        },
        "test",
    )
    .unwrap();
    assert!(transactions::get(app.db(), tx).unwrap().is_some());
    api::entries::verify_chains(app.db()).unwrap();
}

#[test]
fn schedule_range_job_is_processed_by_worker_dispatch() {
    let (_t, app) = open_app();
    let cash = accounts::create(
        app.db(),
        app.bus(),
        &Actor::User,
        "Cash",
        AccountType::Asset,
        "RON",
        "t",
    )
    .unwrap();
    let food = catalog::create_category(
        app.db(),
        app.bus(),
        &Actor::User,
        "Food",
        None,
        AccountType::Expense,
        "t",
    )
    .unwrap();
    let tpl = ScheduleTemplate {
        kind: TxKindSerde::Expense,
        amount_minor: 100,
        currency: "RON".into(),
        from: Some(cash.to_string()),
        to: None,
        category_id: Some(food.to_string()),
        payee_id: None,
        tag_ids: vec![],
        note: Some("rent".into()),
    };
    let rule = ScheduleRule {
        frequency: Frequency::Monthly,
        interval: 1,
        start: Date::new(2026, 1, 31).unwrap(),
        end: EndRule::Never,
        exceptions: vec![],
    };
    let s = schedules::create(app.db(), app.bus(), &Actor::User, tpl, rule, "seed").unwrap();
    schedules::materialize_range_as_job(
        app.db(),
        s,
        Date::new(2026, 1, 1).unwrap(),
        Date::new(2026, 3, 31).unwrap(),
    )
    .unwrap();
    // synchronous worker iteration processes it
    assert!(app.drain_jobs_once().unwrap());
    assert_eq!(jobs::list(app.db(), JobState::Done).unwrap().len(), 1);
    // three monthly occurrences materialized (01-31, 02-28, 03-31)
    let proj = schedules::expand_window(
        app.db(),
        s,
        Date::new(2026, 1, 1).unwrap(),
        Date::new(2026, 3, 31).unwrap(),
        Date::new(2026, 12, 31).unwrap(),
    )
    .unwrap();
    assert_eq!(
        proj.occurrences
            .iter()
            .filter(|o| o.materialized_tx.is_some())
            .count(),
        3
    );
    api::balances::assert_trial_balance(app.db(), None).unwrap();
}

#[test]
fn unknown_job_kind_fails_fast_without_spinning() {
    let (_t, app) = open_app();
    jobs::enqueue(app.db(), "weird:job", "mystery.kind", "{}").unwrap();
    assert!(app.drain_jobs_once().unwrap());
    let failed = jobs::list(app.db(), JobState::Failed).unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].kind, "mystery.kind");
    // and the worker does not re-claim it (not_before/backoff + terminal state)
    assert!(!app.drain_jobs_once().unwrap());
}

#[test]
fn background_worker_processes_queued_job_and_stops_on_drop() {
    let (_t, app) = open_app();
    let cash = accounts::create(
        app.db(),
        app.bus(),
        &Actor::User,
        "Cash",
        AccountType::Asset,
        "RON",
        "t",
    )
    .unwrap();
    let food = catalog::create_category(
        app.db(),
        app.bus(),
        &Actor::User,
        "Food",
        None,
        AccountType::Expense,
        "t",
    )
    .unwrap();
    let tpl = ScheduleTemplate {
        kind: TxKindSerde::Expense,
        amount_minor: 42,
        currency: "RON".into(),
        from: Some(cash.to_string()),
        to: None,
        category_id: Some(food.to_string()),
        payee_id: None,
        tag_ids: vec![],
        note: None,
    };
    let rule = ScheduleRule {
        frequency: Frequency::Daily,
        interval: 1,
        start: Date::new(2026, 1, 1).unwrap(),
        end: EndRule::AfterCount(2),
        exceptions: vec![],
    };
    let s = schedules::create(app.db(), app.bus(), &Actor::User, tpl, rule, "seed").unwrap();
    schedules::materialize_range_as_job(
        app.db(),
        s,
        Date::new(2026, 1, 1).unwrap(),
        Date::new(2026, 1, 10).unwrap(),
    )
    .unwrap();
    // poll until the background worker finishes the job (bounded)
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if !jobs::list(app.db(), JobState::Done).unwrap().is_empty() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "worker did not process the job in time"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    drop(app); // stop + join worker
}

use std::time::Duration;
