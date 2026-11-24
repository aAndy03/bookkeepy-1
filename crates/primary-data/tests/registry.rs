//! Cross-crate registry tests: everything the code emits is registered in primary data.
// Tests may unwrap/expect (RULE-ERR-002 applies outside tests only).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use bookkeepy_core::all_error_codes;
use bookkeepy_primary_data::{currencies, errors, perf_budgets, validate_all};

#[test]
fn all_primary_data_validates() {
    validate_all().unwrap();
}

#[test]
fn every_core_error_code_is_registered() {
    for code in all_error_codes() {
        let def =
            errors::get(&code).unwrap_or_else(|| panic!("unregistered code {}", code.as_str()));
        assert_eq!(def.area, "core");
    }
}

#[test]
fn currency_set_is_iso_4217() {
    assert_eq!(currencies::count(), 168);
    assert_eq!(currencies::source().published, "2026-09-17");
}

#[test]
fn budgets_cover_cold_start() {
    assert!(perf_budgets::get("cold_start_to_add_transaction").is_some());
}
