// SPDX-License-Identifier: GPL-3.0-or-later
//! `data/defaults.toml` — default/fallback policies (what happens when user data is missing
//! or deleted; job retry defaults). Point 2 of the owner: "we have to provide what happens, a
//! default".

use std::sync::OnceLock;

use serde::Deserialize;

use crate::PrimaryDataError;

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/defaults.toml"
));

/// FX fallback chain (tried in order when no rate matches a conversion).
#[derive(Debug, Clone, Deserialize)]
pub struct FxRates {
    /// Ordered fallback steps.
    pub fallback_chain: Vec<String>,
    /// Behavior when manual rates are deleted.
    pub manual_deleted: ManualDeleted,
}

/// Deletion of manual rates is irrecoverable; the fallback chain applies.
#[derive(Debug, Clone, Deserialize)]
pub struct ManualDeleted {
    /// Always false (documents irreversibility).
    pub recoverable: bool,
    /// What applies instead.
    pub behavior: String,
}

/// System-context defaults when local context is missing/cleared.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemContextDefaults {
    /// Assumed DPI scale.
    pub dpi_scale: f64,
    /// Assume one display.
    pub assume_single_display: bool,
    /// Assumed perf tier.
    pub perf_tier: String,
}

/// System-context section of the defaults file.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemContext {
    /// Defaults applied when local system context is missing/cleared.
    pub defaults: SystemContextDefaults,
}

/// Job orchestration defaults (RULE-JOB-001, ADR-0015).
#[derive(Debug, Clone, Deserialize)]
pub struct Jobs {
    /// Max automatic attempts before the job is failed and manual retry is surfaced.
    pub max_attempts: u32,
    /// Backoff base in milliseconds.
    pub backoff_base_ms: u64,
    /// Backoff multiplier.
    pub backoff_multiplier: f64,
    /// Add jitter to backoff.
    pub jitter: bool,
    /// Surface manual retry to the user after max attempts.
    pub surface_manual_retry: bool,
}

/// The parsed defaults file.
#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    /// FX rates defaults.
    pub fx_rates: FxRates,
    /// System context defaults.
    pub system_context: SystemContext,
    /// Jobs defaults.
    pub jobs: Jobs,
}

fn file() -> &'static Defaults {
    static FILE: OnceLock<Defaults> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(RAW)
            .unwrap_or_else(|e| panic!("primary data defaults.toml failed to parse: {e}"))
    })
}

/// The parsed defaults.
pub fn get() -> &'static Defaults {
    file()
}

pub(crate) fn validate_structure() -> Result<(), PrimaryDataError> {
    if get().jobs.max_attempts == 0 {
        return Err(PrimaryDataError::Invalid(
            "jobs.max_attempts must be >= 1".into(),
        ));
    }
    if get().fx_rates.fallback_chain.is_empty() {
        return Err(PrimaryDataError::Invalid(
            "fx_rates.fallback_chain must not be empty".into(),
        ));
    }
    if get().fx_rates.manual_deleted.recoverable {
        return Err(PrimaryDataError::Invalid(
            "fx_rates.manual_deleted.recoverable must stay false (deleted manual rates are irrecoverable)".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_parse_and_validate() {
        validate_structure().unwrap();
        assert!(get().jobs.max_attempts >= 1);
        assert!(get().jobs.surface_manual_retry);
        assert!(!get().fx_rates.fallback_chain.is_empty());
    }
}
