//! ISO 4217 currency codes (ADR-0004, RULE-DATA-002).
//!
//! `CurrencyCode` checks the *shape* (3 ASCII letters, uppercased). Checking the code
//! against the controlled ISO 4217 set (`data/currencies.toml`) happens in the
//! `bookkeepy-primary-data` crate (`validate_iso`), because this crate must not depend on
//! primary data (dependency direction, doc 03). Boundaries validate with both.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::codes;
use crate::error_code::ErrorCode;

/// An ISO 4217 alpha-3 currency code (e.g. `RON`, `EUR`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencyCode(String);

/// Error produced when constructing a [`CurrencyCode`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CurrencyCodeError {
    /// Not 3 ASCII letters.
    #[error("invalid currency code {0:?} (expected 3 ASCII letters, e.g. RON)")]
    Invalid(String),
}

impl CurrencyCodeError {
    /// The BK-code of this error itself.
    pub fn code(&self) -> ErrorCode {
        ErrorCode::from_static(codes::INVALID_CURRENCY_CODE)
    }
}

impl CurrencyCode {
    /// Create from any case (`ron` → `RON`); validates shape only (see module docs).
    pub fn new(code: &str) -> Result<Self, CurrencyCodeError> {
        let upper = code.to_ascii_uppercase();
        if upper.len() == 3 && upper.bytes().all(|b| b.is_ascii_uppercase()) {
            Ok(Self(upper))
        } else {
            Err(CurrencyCodeError::Invalid(code.to_owned()))
        }
    }

    /// The code, e.g. `EUR`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for CurrencyCode {
    type Err = CurrencyCodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercases_and_validates() {
        assert_eq!(CurrencyCode::new("ron").unwrap().as_str(), "RON");
        assert_eq!(CurrencyCode::new("EUR").unwrap().as_str(), "EUR");
        assert!(CurrencyCode::new("EU").is_err());
        assert!(CurrencyCode::new("EURO").is_err());
        assert!(CurrencyCode::new("E1R").is_err());
    }

    #[test]
    fn serde_is_a_string() {
        let c = CurrencyCode::new("RON").unwrap();
        let j = serde_json::to_string(&c).unwrap();
        assert_eq!(j, "\"RON\"");
        assert_eq!(serde_json::from_str::<CurrencyCode>(&j).unwrap(), c);
    }
}
