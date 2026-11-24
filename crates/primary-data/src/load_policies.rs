//! `data/load-policies.toml` — what loads when; hot/warm/cold tiers; prefetch (point 12,
//! doc 03). Consumed by the `loadpolicy` crate (from M3+).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::PrimaryDataError;

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/load-policies.toml"
));

/// The parsed load policies.
#[derive(Debug, Clone, Deserialize)]
pub struct LoadPolicies {
    /// What loads on app open.
    pub on_app_open: Section,
    /// What loads on navigation.
    pub on_navigate: toml::Value,
    /// What prefetches on idle.
    pub on_idle: Section,
    /// Usage-tracking scope declaration.
    pub usage_tracking: UsageTracking,
}

/// A named policy section.
#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    /// Tier name.
    pub tier: Option<String>,
    /// Items loaded.
    pub loads: Option<Vec<String>>,
    /// Items never auto-loaded.
    pub never: Option<Vec<String>>,
    /// Prefetch items.
    pub prefetch: Option<Vec<String>>,
}

/// Usage-tracking declaration (local-only, visible+clearable in Settings → Privacy).
#[derive(Debug, Clone, Deserialize)]
pub struct UsageTracking {
    /// Whether tracking is enabled by default.
    pub enabled: bool,
    /// Scope note.
    pub scope: String,
    /// Where it is stored.
    pub stored_in: String,
    /// Where the user sees it.
    pub visible_in: String,
    /// Whether the user can clear it.
    pub clearable: bool,
}

fn file() -> &'static LoadPolicies {
    static FILE: OnceLock<LoadPolicies> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(RAW)
            .unwrap_or_else(|e| panic!("primary data load-policies.toml failed to parse: {e}"))
    })
}

/// The parsed load policies.
pub fn get() -> &'static LoadPolicies {
    file()
}

pub(crate) fn validate_structure() -> Result<(), PrimaryDataError> {
    let p = get();
    if p.on_app_open.never.is_none() {
        return Err(PrimaryDataError::Invalid(
            "load-policies on_app_open.never must list what is NOT auto-loaded".into(),
        ));
    }
    if !p.usage_tracking.clearable {
        return Err(PrimaryDataError::Invalid(
            "usage_tracking must stay clearable (privacy rule, doc 08)".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policies_parse_and_validate() {
        validate_structure().unwrap();
        assert!(get().on_app_open.loads.as_ref().unwrap().len() >= 3);
    }
}
