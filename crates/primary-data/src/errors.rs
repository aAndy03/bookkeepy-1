//! `data/errors.toml` — the error-code registry (SSOT, RULE-ERR-002, doc 04 §4.4).
//! `cargo xtask govern errors` checks: format, uniqueness, area↔code match, and that every
//! code used in code (e.g. `bookkeepy_core::all_error_codes()`) is registered here.

use std::sync::OnceLock;

use bookkeepy_core::ErrorCode;
use serde::Deserialize;

use crate::PrimaryDataError;

/// Registry-level failure code.
pub const DATA_INVALID: &str = "BK-DATA-0001";

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/errors.toml"
));

/// Severity of a user-facing diagnostic (doc 04 §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Toast; the app continues.
    Warning,
    /// Dialog; the app continues.
    Error,
    /// Surfaced with "Copy diagnostic"; the affected flow halts, the app keeps running.
    Fatal,
}

/// One registered error/warning code.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDef {
    /// Stable code, `BK-<AREA>-<NNNN>`.
    pub code: String,
    /// Area token matching the code prefix (`STORE`, `FX`, …).
    pub area: String,
    /// Severity.
    pub severity: Severity,
    /// Locale message key (i18n crate maps this to text in the chosen language).
    pub i18n_key: String,
    /// English summary (fallback text).
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    #[serde(default)]
    source: Option<toml::Value>,
    #[serde(default)]
    error: Vec<ErrorDef>,
}

fn file() -> &'static File {
    static FILE: OnceLock<File> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(RAW)
            .unwrap_or_else(|e| panic!("primary data errors.toml failed to parse: {e}"))
    })
}

/// All registered error definitions.
pub fn all() -> &'static [ErrorDef] {
    &file().error
}

/// Look up a definition by code.
pub fn get(code: &ErrorCode) -> Option<&'static ErrorDef> {
    all().iter().find(|e| e.code == code.as_str())
}

/// Registry validation: code format, uniqueness, area↔prefix match.
pub fn validate_registry() -> Result<(), PrimaryDataError> {
    if file().source.is_none() {
        return Err(PrimaryDataError::Invalid(
            "errors.toml is missing its [source] header".into(),
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for e in all() {
        let code = ErrorCode::new(&e.code).map_err(|_| {
            PrimaryDataError::Invalid(format!("bad error code format {:?}", e.code))
        })?;
        let prefix = format!("BK-{}-", e.area.to_ascii_uppercase());
        if !e.code.starts_with(&prefix) {
            return Err(PrimaryDataError::Invalid(format!(
                "error code {} does not match its area {}",
                e.code, e.area
            )));
        }
        if e.i18n_key.is_empty() || e.summary.is_empty() {
            return Err(PrimaryDataError::Invalid(format!(
                "error {} is missing i18n_key or summary",
                code.as_str()
            )));
        }
        if !seen.insert(e.code.clone()) {
            return Err(PrimaryDataError::Invalid(format!(
                "duplicate error code {}",
                e.code
            )));
        }
    }
    Ok(())
}

/// Check that every code produced by `core` is registered here.
pub(crate) fn validate_core_codes_registered() -> Result<(), PrimaryDataError> {
    for code in bookkeepy_core::all_error_codes() {
        if get(&code).is_none() {
            return Err(PrimaryDataError::Invalid(format!(
                "core error code {} is not registered in data/errors.toml",
                code.as_str()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_valid() {
        validate_registry().unwrap();
        validate_core_codes_registered().unwrap();
    }

    #[test]
    fn severity_parses() {
        assert_eq!(
            all()
                .iter()
                .find(|e| e.code == "BK-STORE-0001")
                .unwrap()
                .severity,
            Severity::Fatal
        );
        assert_eq!(
            all()
                .iter()
                .find(|e| e.code == "BK-RATES-0001")
                .unwrap()
                .severity,
            Severity::Warning
        );
    }
}
