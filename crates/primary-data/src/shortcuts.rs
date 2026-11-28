// SPDX-License-Identifier: GPL-3.0-or-later
//! `data/shortcuts.toml` — the default keyboard-shortcut map (RULE-INPUT-001, doc 04
//! §4.9): flat entries (`id`, `keys`, `action`, `label_i18n_key`, `scope`, `when` guard).
//! SSOT for every keybinding — no key or action is hardcoded anywhere else; remapping
//! later = user overrides in `ui_state`, never a code edit. The UI binds the dotted
//! `action` ids via gpui actions/keybindings.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::{PrimaryDataError, locales};

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/shortcuts.toml"
));

/// One shortcut entry.
#[derive(Debug, Clone, Deserialize)]
pub struct Shortcut {
    /// Stable entry id (SSOT name; user overrides key on this id).
    pub id: String,
    /// Default binding, lowercase gpui grammar, e.g. "ctrl+n".
    pub keys: String,
    /// Dotted engine action id bound via gpui actions/keybindings.
    pub action: String,
    /// Label i18n key (`data/locales/*`) — must exist in every shipped locale.
    pub label_i18n_key: String,
    /// `global` (any screen) or `screen` (a specific screen must be active).
    pub scope: String,
    /// Context guard expression ("" = always active when the scope matches).
    pub when: String,
}

/// Parsed shortcuts file.
#[derive(Debug, Deserialize)]
struct ShortcutsFile {
    shortcut: Vec<Shortcut>,
}

fn file() -> &'static ShortcutsFile {
    static FILE: OnceLock<ShortcutsFile> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(RAW)
            .unwrap_or_else(|e| panic!("primary data shortcuts.toml failed to parse: {e}"))
    })
}

/// All default shortcuts, in file order.
pub fn all() -> &'static [Shortcut] {
    &file().shortcut
}

/// The entry with `id`, if present.
pub fn get(id: &str) -> Option<&'static Shortcut> {
    all().iter().find(|s| s.id == id)
}

/// Modifier set allowed in `keys` (the `primary` modifier resolves per platform in the UI).
const MODIFIERS: &[&str] = &["ctrl", "alt", "shift", "cmd", "secondary"];

fn keys_ok(keys: &str) -> bool {
    if keys.is_empty() {
        return false;
    }
    keys.split('+').all(|part| {
        part.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && (!MODIFIERS.contains(&part) || part.chars().all(|c| c.is_ascii_alphabetic()))
    }) && keys
        .split('+')
        .next_back()
        .is_some_and(|last| !MODIFIERS.contains(&last))
}

/// Structural + referential validation (RULE-DATA-002): ids unique, `keys` grammar sane,
/// `scope` known, and every `label_i18n_key` exists in every shipped locale (RULE-INPUT-001
/// promises localized shortcut labels).
pub fn validate_structure() -> Result<(), PrimaryDataError> {
    let entries = all();
    if entries.is_empty() {
        return Err(PrimaryDataError::Invalid(
            "shortcuts.toml must list at least the M0 seed entries".into(),
        ));
    }
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    for s in entries {
        if !ids.insert(s.id.as_str()) {
            return Err(PrimaryDataError::Invalid(format!(
                "shortcuts.toml duplicate id {:?}",
                s.id
            )));
        }
        if !keys_ok(&s.keys) {
            return Err(PrimaryDataError::Invalid(format!(
                "shortcuts.toml id {:?} keys {:?} must be lowercase \"modifier+key\" (modifiers: ctrl|alt|shift|cmd|secondary, last part a key)",
                s.id, s.keys
            )));
        }
        if !s.action.split('.').all(|part| {
            part.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                && !part.is_empty()
        }) {
            return Err(PrimaryDataError::Invalid(format!(
                "shortcuts.toml id {:?} action {:?} must be a dotted lowercase id",
                s.id, s.action
            )));
        }
        if !["global", "screen"].contains(&s.scope.as_str()) {
            return Err(PrimaryDataError::Invalid(format!(
                "shortcuts.toml id {:?} scope {:?} must be global|screen",
                s.id, s.scope
            )));
        }
        if s.scope == "screen" && s.when.is_empty() {
            return Err(PrimaryDataError::Invalid(format!(
                "shortcuts.toml id {:?} scope \"screen\" needs a `when` guard naming the screen",
                s.id
            )));
        }
        for locale in locales::all() {
            if !locale.messages.contains_key(&s.label_i18n_key) {
                return Err(PrimaryDataError::Invalid(format!(
                    "shortcuts.toml id {:?} label_i18n_key {:?} has no message in locale {:?}",
                    s.id, s.label_i18n_key, locale.tag
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn shortcuts_validates() {
        super::validate_structure().unwrap();
        assert_eq!(super::all().len(), 3);
        assert_eq!(super::get("app.new_draft").unwrap().keys, "ctrl+n");
        assert!(super::get("nope").is_none());
    }
}
