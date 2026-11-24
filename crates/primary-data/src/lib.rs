#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-primary-data` — primary data loader (RULE-DATA-001/002/003, ADR-0008).
//!
//! Every `data/*.toml` file is compiled into the binary with `include_str!` and parsed +
//! validated **once** at first access (`OnceLock`). Invalid primary data fails loudly at
//! startup/`validate_all` — and `cargo xtask govern data` checks it in CI on every commit.
//! Scopes are modular (one module per scope); growing scopes become directories with an
//! `index.toml` (RULE-DATA-003).

pub mod currencies;
pub mod defaults;
pub mod errors;
pub mod load_policies;
pub mod perf_budgets;
mod validate;

pub use validate::validate_all;

/// Error type for primary-data loading/validation failures (BK-DATA-0001).
#[derive(Debug, thiserror::Error)]
pub enum PrimaryDataError {
    /// A data file failed validation; the message says which file and why.
    #[error("invalid primary data: {0}")]
    Invalid(String),
}

impl PrimaryDataError {
    /// The registered BK-code for this error.
    pub fn code(&self) -> bookkeepy_core::ErrorCode {
        bookkeepy_core::ErrorCode::from_static(errors::DATA_INVALID)
    }
}
