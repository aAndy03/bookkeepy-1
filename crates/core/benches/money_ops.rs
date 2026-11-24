// SPDX-License-Identifier: GPL-3.0-or-later
//! Criterion harness (item 15, M0.5): benches live next to the crate they measure
//! (RULE-MOD-002) and assert against `data/perf-budgets.toml` in CI (`govern perf` +
//! the `bench` CI job). `-- --test` mode runs each bench once without measuring (CI sanity).
#![allow(clippy::unwrap_used, clippy::expect_used)] // bench target, like integration tests

use bookkeepy_core::{CurrencyCode, Money};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_money_add(c: &mut Criterion) {
    let ron = CurrencyCode::new("RON").unwrap();
    let a = Money::new(123_456_789, ron.clone());
    let b = Money::new(987_654_321, ron);
    c.bench_function("money_checked_add", |bencher| {
        bencher.iter(|| black_box(a.clone()).checked_add(black_box(b.clone())))
    });
}

fn bench_currency_code_parse(c: &mut Criterion) {
    c.bench_function("currency_code_new", |bencher| {
        bencher.iter(|| CurrencyCode::new(black_box("RON")))
    });
}

criterion_group!(benches, bench_money_add, bench_currency_code_parse);
criterion_main!(benches);
