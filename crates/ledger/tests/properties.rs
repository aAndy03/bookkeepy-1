// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Property tests for the ledger invariants (doc 10 M2):
//! - INV-LEDGER-001: the trial balance holds (debits == credits per currency) after EVERY
//!   operation, i.e. for any prefix of the journal.
//! - Reversal conservation: reversing an entry restores the exact balances from before it.
//! - INV-CHAIN-001: both hash chains verify after every operation.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::Actor;
use bookkeepy_core::time::now;
use bookkeepy_core::{CurrencyCode, EntityId, Money};
use bookkeepy_ledger::api::accounts::{self, AccountType};
use bookkeepy_ledger::api::balances::{self, account_balance};
use bookkeepy_ledger::api::catalog;
use bookkeepy_ledger::api::entries::{self, NewEntry, Posting, Side};
use bookkeepy_ledger::api::lifecycle::TxStatus;
use bookkeepy_ledger::api::transactions::{self, NewTransaction, TxKind};
use bookkeepy_storage::api::conn::Db;
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Transfer { a: usize, b: usize, amt: i64 },
    Expense { a: usize, c: usize, amt: i64 },
    ReverseLast,
}

fn op_strategy(n_accounts: usize, n_cats: usize) -> impl Strategy<Value = Op> {
    let na = n_accounts.max(1);
    let nc = n_cats.max(1);
    prop_oneof![
        3 => (0..na, 0..na, 1i64..10_000).prop_flat_map(move |(a, b, amt)| {
            let na = na;
            Just(Op::Transfer { a, b: if b == a { (b + 1) % na } else { b }, amt })
        }),
        3 => (0..na, 0..nc, 1i64..10_000).prop_map(move |(a, c, amt)| Op::Expense { a, c, amt }),
        1 => Just(Op::ReverseLast),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn ledger_invariants_hold(ops in proptest::collection::vec(op_strategy(3, 2), 1..25)) {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let ron = CurrencyCode::new("RON").unwrap();

        let mut accounts = Vec::new();
        for i in 0..3 {
            accounts.push(
                accounts::create(&db, &bus, &Actor::User, &format!("Acc{i}"), AccountType::Asset, "RON", "seed").unwrap(),
            );
        }
        let mut cats = Vec::new();
        for i in 0..2 {
            cats.push(
                catalog::create_category(&db, &bus, &Actor::User, &format!("Cat{i}"), None, AccountType::Expense, "seed").unwrap(),
            );
        }

        let mut entries_posted: Vec<EntityId> = Vec::new();
        // account balances right before each posted entry, to check reversal conservation
        let mut before_entry: Vec<Vec<i64>> = Vec::new();

        let snapshot = |db: &Db, accounts: &Vec<EntityId>| -> Vec<i64> {
            accounts
                .iter()
                .map(|a| account_balance(db, *a, None).unwrap().amount_minor())
                .collect()
        };

        for op in ops {
            match op {
                Op::Transfer { a, b, amt } => {
                    before_entry.push(snapshot(&db, &accounts));
                    let e = entries::post(
                        &db,
                        &bus,
                        &Actor::User,
                        NewEntry {
                            tx_id: None,
                            value_ts: now(),
                            note: None,
                            postings: vec![
                                Posting { account_id: accounts[b], side: Side::Debit, amount: Money::new(amt, ron.clone()) },
                                Posting { account_id: accounts[a], side: Side::Credit, amount: Money::new(amt, ron.clone()) },
                            ],
                        },
                        "prop transfer",
                    )
                    .unwrap();
                    entries_posted.push(e);
                }
                Op::Expense { a, c, amt } => {
                    before_entry.push(snapshot(&db, &accounts));
                    let tx = transactions::create(
                        &db,
                        &bus,
                        &Actor::User,
                        NewTransaction {
                            kind: TxKind::Expense,
                            amount: Money::new(amt, ron.clone()),
                            from: Some(accounts[a]),
                            to: None,
                            category_id: Some(cats[c]),
                            payee_id: None,
                            tag_ids: vec![],
                            note: None,
                            value_ts: now(),
                            status: TxStatus::Cleared,
                        },
                        "prop expense",
                    )
                    .unwrap();
                    let row = transactions::get(&db, tx).unwrap().unwrap();
                    entries_posted.push(row.entry_id.unwrap());
                }
                Op::ReverseLast => {
                    if let Some(e) = entries_posted.last().copied() {
                        // balances before the last entry was posted
                        let before = before_entry.last().unwrap().clone();
                        entries::reverse(&db, &bus, &Actor::User, e, "prop reverse").unwrap();
                        entries_posted.pop();
                        before_entry.pop();
                        prop_assert_eq!(snapshot(&db, &accounts), before, "reversal must restore prior balances");
                    }
                }
            }
            // INV-LEDGER-001: trial balance holds after every prefix
            balances::assert_trial_balance(&db, None).unwrap();
            // INV-CHAIN-001: chains verify
            entries::verify_chains(&db).unwrap();
        }
    }
}
