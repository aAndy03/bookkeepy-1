//! `data/perf-budgets.toml` — performance budgets (RULE-PERF-001, point 7): the SSOT
//! asserted by criterion benches + `cargo xtask govern perf` so no backend part drifts.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::PrimaryDataError;

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/perf-budgets.toml"
));

/// One performance budget.
#[derive(Debug, Clone, Deserialize)]
pub struct Budget {
    /// Stable budget id.
    pub id: String,
    /// Unit (`ms`, `s`, …).
    pub unit: String,
    /// Target value.
    pub target: u64,
    /// Hard max (CI fails above this).
    pub max: u64,
    /// What it measures.
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    #[serde(default)]
    source: Option<toml::Value>,
    budget: Vec<Budget>,
}

fn file() -> &'static File {
    static FILE: OnceLock<File> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(RAW)
            .unwrap_or_else(|e| panic!("primary data perf-budgets.toml failed to parse: {e}"))
    })
}

/// All budgets.
pub fn all() -> &'static [Budget] {
    &file().budget
}

/// Look up a budget by id.
pub fn get(id: &str) -> Option<&'static Budget> {
    all().iter().find(|b| b.id == id)
}

pub(crate) fn validate_structure() -> Result<(), PrimaryDataError> {
    if file().source.is_none() {
        return Err(PrimaryDataError::Invalid(
            "perf-budgets.toml is missing its [source] header".into(),
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for b in all() {
        if b.max < b.target {
            return Err(PrimaryDataError::Invalid(format!(
                "budget {} has max < target",
                b.id
            )));
        }
        if !seen.insert(b.id.clone()) {
            return Err(PrimaryDataError::Invalid(format!(
                "duplicate budget id {}",
                b.id
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budgets_valid_and_include_cold_start() {
        validate_structure().unwrap();
        let b = get("cold_start_to_add_transaction").unwrap();
        assert_eq!(b.unit, "ms");
        assert!(b.max >= b.target);
    }
}
