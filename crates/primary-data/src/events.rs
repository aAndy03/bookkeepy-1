// SPDX-License-Identifier: GPL-3.0-or-later
//! `data/events.toml` — domain event-kind registry (RULE-EVT-001, ADR-0009). SSOT that
//! `cargo xtask govern events` checks against the `bookkeepy_contracts::kinds` consts:
//! both sets must be identical, and no other code may contain a kind string literal.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::PrimaryDataError;

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/events.toml"
));

/// One registered event kind.
#[derive(Debug, Clone, Deserialize)]
pub struct EventDef {
    /// Stable dotted kind, e.g. `transaction.posted`. Never renamed (audit immutability).
    pub kind: String,
    /// Emitting crate::module.
    pub owner: String,
    /// JSON payload shape (informational).
    pub payload: String,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    #[serde(default)]
    event: Vec<EventDef>,
}

static EVENTS: OnceLock<Result<Vec<EventDef>, String>> = OnceLock::new();

/// All registered event kinds (parsed + validated once).
pub fn get() -> Result<&'static [EventDef], PrimaryDataError> {
    match EVENTS.get_or_init(|| {
        let f: File = toml::from_str(RAW).map_err(|e| format!("events.toml: {e}"))?;
        validate(&f.event).map_err(|e| e.to_string())?;
        Ok(f.event)
    }) {
        Ok(v) => Ok(v.as_slice()),
        Err(msg) => Err(PrimaryDataError::Invalid(msg.clone())),
    }
}

/// Structure validation: dotted kind format, uniqueness (RULE-DATA-002).
pub fn validate_structure() -> Result<(), PrimaryDataError> {
    get().map(|_| ())
}

fn validate(events: &[EventDef]) -> Result<(), PrimaryDataError> {
    let mut seen = std::collections::HashSet::new();
    for e in events {
        let parts: Vec<&str> = e.kind.split('.').collect();
        if parts.len() < 2
            || parts.iter().any(|p| {
                p.is_empty()
                    || !p
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
            })
        {
            return Err(PrimaryDataError::Invalid(format!(
                "events.toml: bad kind {:?} (dotted lowercase snake expected)",
                e.kind
            )));
        }
        if !seen.insert(e.kind.clone()) {
            return Err(PrimaryDataError::Invalid(format!(
                "events.toml: duplicate kind {:?}",
                e.kind
            )));
        }
        if e.owner.is_empty() {
            return Err(PrimaryDataError::Invalid(format!(
                "events.toml: kind {:?} has no owner",
                e.kind
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn registry_loads() {
        let evs = super::get().unwrap();
        assert!(evs.iter().any(|e| e.kind == "transaction.posted"));
        assert!(super::validate_structure().is_ok());
    }
}
