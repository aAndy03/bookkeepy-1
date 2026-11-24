// SPDX-License-Identifier: GPL-3.0-or-later
//! Stable, registry-backed error codes: `BK-<AREA>-<NNNN>` (RULE-ERR-002, doc 04 §4.4).
//! Every user-facing diagnostic carries one; the text lives in `data/errors.toml` +
//! locale catalogs, so codes are copyable and localized.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::codes;

/// A stable error code, e.g. `BK-STORE-0001`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ErrorCode(String);

/// Error produced when parsing/constructing an [`ErrorCode`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorCodeError {
    /// The string does not match `BK-<AREA>-<NNNN>`.
    #[error("invalid error code {0:?} (expected BK-<AREA>-<NNNN>)")]
    Invalid(String),
}

impl ErrorCodeError {
    /// The BK-code of this error itself.
    pub fn code(&self) -> ErrorCode {
        ErrorCode::from_static(codes::INVALID_ERROR_CODE)
    }
}

impl ErrorCode {
    /// Parse and validate a code (`BK-<AREA>-<NNNN>`, area = uppercase letters).
    pub fn new(code: &str) -> Result<Self, ErrorCodeError> {
        let valid = match code.strip_prefix("BK-").and_then(|r| r.split_once('-')) {
            Some((area, num)) => {
                !area.is_empty()
                    && area.bytes().all(|b| b.is_ascii_uppercase())
                    && num.len() == 4
                    && num.bytes().all(|b| b.is_ascii_digit())
            }
            None => false,
        };
        if valid {
            Ok(Self(code.to_owned()))
        } else {
            Err(ErrorCodeError::Invalid(code.to_owned()))
        }
    }

    /// Wrap a compile-time constant known to be valid (debug-asserted).
    pub fn from_static(code: &'static str) -> Self {
        debug_assert!(Self::new(code).is_ok(), "invalid static error code {code}");
        Self(code.to_owned())
    }

    /// The code as a string, e.g. `BK-STORE-0001`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ErrorCode {
    type Err = ErrorCodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_codes() {
        assert_eq!(
            ErrorCode::new("BK-STORE-0001").unwrap().as_str(),
            "BK-STORE-0001"
        );
        assert!(ErrorCode::new("BK-FX-0002").is_ok());
    }

    #[test]
    fn rejects_invalid_codes() {
        assert!(ErrorCode::new("BK-store-0001").is_err());
        assert!(ErrorCode::new("XX-STORE-0001").is_err());
        assert!(ErrorCode::new("BK-STORE-001").is_err());
        assert!(ErrorCode::new("BK-STORE-00011").is_err());
        assert!(ErrorCode::new("").is_err());
    }

    #[test]
    fn round_trips() {
        let c = ErrorCode::new("BK-CORE-0001").unwrap();
        assert_eq!(c.to_string(), "BK-CORE-0001");
        assert_eq!(c.as_str().parse::<ErrorCode>().unwrap(), c);
    }
}
