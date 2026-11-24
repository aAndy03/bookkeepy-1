// SPDX-License-Identifier: GPL-3.0-or-later
//! Criterion harness for the ledger write/read hot paths (ADR-0025 optimization audit):
//! posting N transactions, then reading all balances (one grouped query — the batched
//! `account_balances`) and the trial balance. Budgets live in `data/perf-budgets.toml`;
//! CI runs this with `-- --test` (sanity) and the `bench` job measures for real.
#![allow(clippy::unwrap_used, clippy::expect_used)] // bench target, like integration tests

use bookkeepy_bus::Bus;
use bookkeepy_contracts::Actor;
use bookkeepy_core::time::now;
use bookkeepy_core::{CurrencyCode, Money};
use bookkeepy_ledger::api::accounts::{self, AccountType};
use bookkeepy_ledger::api::balances;
use bookkeepy_ledger::api::catalog;
use bookkeepy_ledger::api::lifecycle::TxStatus;
use bookkeepy_ledger::api::transactions::{self, NewTransaction, TxKind};
use bookkeepy_storage::api::conn::Db;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_post_and_read(c: &mut Criterion) {
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
    let ron = CurrencyCode::new("RON").unwrap();
    let mut group = c.benchmark_group("ledger_hot_paths");
    group.sample_size(20);
    group.bench_function("post_100_expenses", |b| {
        b.iter(|| {
            for i in 0..100 {
                transactions::create(
                    black_box(&db),
                    black_box(&bus),
                    &Actor::User,
                    NewTransaction {
                        kind: TxKind::Expense,
                        amount: Money::new(100 + i, ron.clone()),
                        from: Some(cash),
                        to: None,
                        category_id: Some(food),
                        payee_id: None,
                        tag_ids: vec![],
                        note: Some("bench".into()),
                        value_ts: now(),
                        status: TxStatus::Cleared,
                        schedule_id: None,
                        occurrence_date: None,
                    },
                    "bench",
                )
                .unwrap();
            }
        })
    });
    group.bench_function("read_all_balances_grouped", |b| {
        b.iter(|| black_box(balances::account_balances(black_box(&db), None).unwrap()))
    });
    group.bench_function("trial_balance", |b| {
        b.iter(|| black_box(balances::trial_balance(black_box(&db), None).unwrap()))
    });
    group.finish();
}

criterion_group!(benches, bench_post_and_read);
criterion_main!(benches);
