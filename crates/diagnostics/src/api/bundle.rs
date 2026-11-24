//! Diagnostic bundle, wire spec `bookkeepy-diagnostic@1` (item 2, M0.5; RULE-ERR-003:
//! every fatal error surfaces as a copyable bundle the user can paste into a bug report).
//! A bundle is plain JSON so it can be attached to GitHub issues; `to_copyable_text`
//! renders the same content as human-readable text for the clipboard.

use bookkeepy_core::time::{Timestamp, now};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bundle format identifier (bump on breaking change; consumers must reject unknown).
pub const SPEC: &str = "bookkeepy-diagnostic@1";

/// How bad the entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational.
    Info,
    /// Something failed but the app continues (RULE-ERR-004).
    Warning,
    /// Fatal — surfaced to the user for reporting (INV-ERR-001).
    Fatal,
}

/// One diagnosed problem: a registered error code + readable message + context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEntry {
    /// `BK-…` code from `data/errors.toml` (RULE-ERR-002).
    pub code: String,
    /// Severity.
    pub severity: Severity,
    /// Readable message (localized at surface time via the error's `i18n_key`).
    pub message: String,
    /// Extra key=value context (profile id, file, step…), sorted for stable output.
    pub context: BTreeMap<String, String>,
}

impl DiagnosticEntry {
    /// New entry with empty context.
    pub fn new(code: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    /// Attach a context key=value (builder style).
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// A full diagnostic snapshot: build info + OS + entries + when it was taken.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticBundle {
    /// Wire spec id, always `bookkeepy-diagnostic@1`.
    pub spec: String,
    /// `build_info::full()` one-liner.
    pub app: String,
    /// Operating system (`std::env::consts::OS`); richer info via `os_info` later (doc 07).
    pub os: String,
    /// When the bundle was created (UTC).
    pub created_at: Timestamp,
    /// Entries, in insertion order.
    pub entries: Vec<DiagnosticEntry>,
}

impl DiagnosticBundle {
    /// New bundle stamped with the current build and OS.
    pub fn new() -> Self {
        Self {
            spec: SPEC.to_string(),
            app: bookkeepy_build_info::full(),
            os: std::env::consts::OS.to_string(),
            created_at: now(),
            entries: Vec::new(),
        }
    }

    /// Add an entry (builder style).
    pub fn push(mut self, entry: DiagnosticEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Serialize to canonical JSON (what gets attached to a bug report).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Human-readable, clipboard-friendly rendering (RULE-ERR-003).
    pub fn to_copyable_text(&self) -> String {
        let mut out = format!(
            "{}\n{}\nOS: {}\nAt: {}\n",
            self.spec, self.app, self.os, self.created_at
        );
        for e in &self.entries {
            out.push_str(&format!("[{:?}] {}: {}\n", e.severity, e.code, e.message));
            for (k, v) in &e.context {
                out.push_str(&format!("    {k}: {v}\n"));
            }
        }
        out
    }
}

impl Default for DiagnosticBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_json_roundtrip_and_copyable() {
        let b = DiagnosticBundle::new().push(
            DiagnosticEntry::new("BK-APP-0001", Severity::Fatal, "boom").with("profile", "demo"),
        );
        let json = b.to_json().unwrap();
        assert!(json.contains("bookkeepy-diagnostic@1"));
        let back: DiagnosticBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.entries[0].code, "BK-APP-0001");
        let text = b.to_copyable_text();
        assert!(text.contains("[Fatal] BK-APP-0001: boom"));
        assert!(text.contains("profile: demo"));
    }
}
