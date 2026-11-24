// SPDX-License-Identifier: GPL-3.0-or-later
//! Error-code constants for this crate. Every constant must be registered in
//! `data/errors.toml` (SSOT registry, RULE-ERR-002) — enforced by `cargo xtask govern errors`.

/// Adding two `Money` values of different currencies (fintecheng: no cross-currency arithmetic).
pub const CURRENCY_MISMATCH: &str = "BK-CORE-0001";
/// i64 overflow in minor units.
pub const AMOUNT_OVERFLOW: &str = "BK-CORE-0002";
/// Malformed ISO 4217 alpha-3 code.
pub const INVALID_CURRENCY_CODE: &str = "BK-CORE-0003";
/// Malformed `BK-<AREA>-<NNNN>` error code.
pub const INVALID_ERROR_CODE: &str = "BK-CORE-0004";

/// All codes this crate can emit (checked against the registry by the governor).
pub const ALL: &[&str] = &[
    CURRENCY_MISMATCH,
    AMOUNT_OVERFLOW,
    INVALID_CURRENCY_CODE,
    INVALID_ERROR_CODE,
];
