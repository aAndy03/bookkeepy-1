// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
// INV-MONEY-001 (ADR-0004): no floating-point arithmetic anywhere in this crate.
#![deny(clippy::float_arithmetic)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-core` — the foundation crate (doc 03): money, currency codes, entity ids,
//! error codes, time. No I/O, no workspace-internal dependencies; every other crate builds
//! on this. Money is integer minor units paired with a currency (`Money`), never floats.

pub mod codes;
pub mod currency;
pub mod error_code;
pub mod id;
pub mod money;
pub mod time;

pub use currency::{CurrencyCode, CurrencyCodeError};
pub use error_code::{ErrorCode, ErrorCodeError};
pub use id::EntityId;
pub use money::{Money, MoneyError};

/// All error codes this crate can produce. `cargo xtask govern errors` asserts each one is
/// registered in `data/errors.toml` (RULE-ERR-002) with a message in every shipped locale.
pub fn all_error_codes() -> Vec<ErrorCode> {
    codes::ALL
        .iter()
        .map(|c| ErrorCode::from_static(c))
        .collect()
}
