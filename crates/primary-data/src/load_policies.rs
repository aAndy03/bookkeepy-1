// SPDX-License-Identifier: GPL-3.0-or-later
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
    /// Canonical cold-start boot sequence (round-5 item 6).
    pub boot_sequence: BootSequence,
    /// What loads on app open.
    pub on_app_open: Section,
    /// What loads on navigation.
    pub on_navigate: toml::Value,
    /// What prefetches on idle.
    pub on_idle: Section,
    /// Usage-tracking scope declaration.
    pub usage_tracking: UsageTracking,
}

/// The boot sequence declaration.
#[derive(Debug, Clone, Deserialize)]
pub struct BootSequence {
    /// Ordered boot steps.
    pub steps: Vec<BootStep>,
}

/// One canonical boot step: machine-readable `id` + human label.
#[derive(Debug, Clone, Deserialize)]
pub struct BootStep {
    /// Machine id (e.g. `profile_open`); code keeps its executed order a subsequence of
    /// these ids.
    pub id: String,
    /// Human-readable description.
    pub label: String,
}

impl BootSequence {
    /// The ordered machine ids of all boot steps.
    pub fn step_ids(&self) -> Vec<&str> {
        self.steps.iter().map(|s| s.id.as_str()).collect()
    }
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
    if p.boot_sequence.steps.len() < 5 {
        return Err(PrimaryDataError::Invalid(
            "boot_sequence must list the canonical cold-start steps".into(),
        ));
    }
    let mut ids = std::collections::HashSet::new();
    for s in &p.boot_sequence.steps {
        if s.id.is_empty() || s.label.is_empty() || !ids.insert(s.id.clone()) {
            return Err(PrimaryDataError::Invalid(format!(
                "boot_sequence step id {:?} is empty, duplicated, or missing a label",
                s.id
            )));
        }
    }
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
