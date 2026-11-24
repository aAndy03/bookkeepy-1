// SPDX-License-Identifier: GPL-3.0-or-later
//! `Money` — amount in integer minor units + currency (ADR-0004, INV-MONEY-001).
//!
//! fintecheng (quoted, doc 06 §6.2): "storing the amount as an integer in its smallest unit
//! (€12.34 becomes 1234 … don't assume it's always 2, it's not!)" and "Send money either as
//! a string ("12.34") or as an integer in its smallest unit" — never a bare JSON number.
//! Serde therefore serializes the struct with an *integer* `amount_minor`.
//!
//! No cross-currency arithmetic (fintecheng: "No cross-currency arithmetic… conversion
//! should happen very explicitly with a strictly controlled rate") — conversion lives in the
//! `fx`/`rates` crates with a stored rate. `Display` here is the canonical *machine* form
//! (`1234 RON`); user-facing formatting (locale separators, symbol, always the ISO code —
//! RULE-FMT-001) is the shared `format` helper, which looks up minor units from primary data.

use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::codes;
use crate::currency::CurrencyCode;
use crate::error_code::ErrorCode;

/// An amount of money: integer minor units paired with its currency.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    amount_minor: i64,
    currency: CurrencyCode,
}

/// Errors from money operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MoneyError {
    /// Adding/subtracting across currencies (fintecheng: prohibited without a controlled rate).
    #[error("currency mismatch: {left} vs {right}")]
    CurrencyMismatch {
        /// Left currency.
        left: CurrencyCode,
        /// Right currency.
        right: CurrencyCode,
    },
    /// i64 overflow in minor units.
    #[error("amount overflow in minor units")]
    Overflow,
}

impl MoneyError {
    /// The registered BK-code for this error.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::CurrencyMismatch { .. } => ErrorCode::from_static(codes::CURRENCY_MISMATCH),
            Self::Overflow => ErrorCode::from_static(codes::AMOUNT_OVERFLOW),
        }
    }
}

impl Money {
    /// Create from minor units + currency.
    pub fn new(amount_minor: i64, currency: CurrencyCode) -> Self {
        Self {
            amount_minor,
            currency,
        }
    }

    /// Zero in the given currency.
    pub fn zero(currency: CurrencyCode) -> Self {
        Self::new(0, currency)
    }

    /// The amount in minor units (e.g. cents): `1234` for `12.34`.
    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    /// The currency.
    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    /// Whether the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount_minor == 0
    }

    /// Whether the amount is negative (overdrafts are representable — never clamped, doc 06).
    pub fn is_negative(&self) -> bool {
        self.amount_minor < 0
    }

    /// Checked addition; fails on currency mismatch or overflow. Money is conserved:
    /// `(a + b) - b == a` (property-tested below).
    pub fn checked_add(self, rhs: Money) -> Result<Money, MoneyError> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: rhs.currency,
            });
        }
        let amount = self
            .amount_minor
            .checked_add(rhs.amount_minor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency))
    }

    /// Checked subtraction (see [`Money::checked_add`]).
    pub fn checked_sub(self, rhs: Money) -> Result<Money, MoneyError> {
        rhs.checked_neg()?.checked_add(self)
    }

    /// Checked negation.
    pub fn checked_neg(self) -> Result<Money, MoneyError> {
        let amount = self
            .amount_minor
            .checked_neg()
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency))
    }

    /// Canonical decimal string with a `.` separator, e.g. `12.34` (integer math only).
    /// `minor_units` comes from primary data (`data/currencies.toml`).
    pub fn to_decimal_string(&self, minor_units: u8) -> String {
        let abs = self.amount_minor.unsigned_abs();
        if minor_units == 0 {
            return if self.amount_minor < 0 {
                format!("-{abs}")
            } else {
                abs.to_string()
            };
        }
        let div = 10u64
            .checked_pow(u32::from(minor_units))
            .unwrap_or(u64::MAX);
        let whole = abs / div;
        let frac = abs % div;
        format!(
            "{}{}.{:0width$}",
            if self.amount_minor < 0 { "-" } else { "" },
            whole,
            frac,
            width = usize::from(minor_units)
        )
    }
}

impl PartialOrd for Money {
    /// Orders only within the same currency; `None` across currencies (meaningless comparison).
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency != other.currency {
            None
        } else {
            Some(self.amount_minor.cmp(&other.amount_minor))
        }
    }
}

impl fmt::Display for Money {
    /// Canonical machine form `1234 RON` — not user-facing (see module docs / RULE-FMT-001).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount_minor, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn ron(x: i64) -> Money {
        Money::new(x, CurrencyCode::new("RON").unwrap())
    }

    #[test]
    fn add_sub_conserve() {
        let a = ron(1234);
        let b = ron(56);
        assert_eq!(
            a.clone()
                .checked_add(b.clone())
                .unwrap()
                .checked_sub(b)
                .unwrap(),
            a
        );
    }

    #[test]
    fn cross_currency_is_an_error() {
        let eur = Money::new(100, CurrencyCode::new("EUR").unwrap());
        let err = ron(1).checked_add(eur).unwrap_err();
        assert_eq!(
            err,
            MoneyError::CurrencyMismatch {
                left: CurrencyCode::new("RON").unwrap(),
                right: CurrencyCode::new("EUR").unwrap(),
            }
        );
        assert_eq!(err.code().as_str(), "BK-CORE-0001");
    }

    #[test]
    fn overflow_is_an_error() {
        assert_eq!(
            ron(i64::MAX)
                .checked_add(ron(1))
                .unwrap_err()
                .code()
                .as_str(),
            "BK-CORE-0002"
        );
        assert_eq!(
            ron(i64::MIN).checked_neg().unwrap_err().code().as_str(),
            "BK-CORE-0002"
        );
    }

    #[test]
    fn decimal_string_uses_minor_units() {
        assert_eq!(ron(1234).to_decimal_string(2), "12.34");
        assert_eq!(ron(-5).to_decimal_string(2), "-0.05");
        assert_eq!(ron(392).to_decimal_string(0), "392"); // JPY-style
        assert_eq!(ron(1234567).to_decimal_string(3), "1234.567"); // BHD-style
    }

    #[test]
    fn serde_is_integer_not_float() {
        let j = serde_json::to_string(&ron(1234)).unwrap();
        assert_eq!(j, r#"{"amount_minor":1234,"currency":"RON"}"#);
        assert!(!j.contains("1234.0"));
        assert_eq!(serde_json::from_str::<Money>(&j).unwrap(), ron(1234));
    }

    proptest! {
        /// fintecheng: "for any sequence of postings, the books balance" — here: money is conserved.
        #[test]
        fn conservation(a in -1_000_000_000i64..1_000_000_000, b in -1_000_000_000i64..1_000_000_000) {
            let (a, b) = (ron(a), ron(b));
            if let Ok(sum) = a.clone().checked_add(b.clone()) {
                prop_assert_eq!(sum.clone().checked_sub(b.clone()).unwrap(), a.clone());
                prop_assert_eq!(sum.checked_sub(a.clone()).unwrap(), b.clone());
            }
        }

        #[test]
        fn neg_is_involution(x in -1_000_000_000i64..1_000_000_000) {
            let m = ron(x);
            prop_assert_eq!(m.clone().checked_neg().unwrap().checked_neg().unwrap(), m);
        }
    }
}
