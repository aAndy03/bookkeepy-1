// SPDX-License-Identifier: GPL-3.0-or-later
//! Key-value tables: `ui_state` (window/session state, doc 05 §5.14, RULE-STATE-001) and
//! `settings` (user overrides on top of `data/settings-defaults.toml` — the defaults are
//! the SSOT; this table only stores deltas).

use rusqlite::OptionalExtension;

use crate::api::conn::Db;
use crate::api::error::StorageError;

/// Read a `ui_state` value.
pub fn ui_get(db: &Db, key: &str) -> Result<Option<String>, StorageError> {
    db.with_conn(|c| {
        Ok(
            c.query_row("SELECT value FROM ui_state WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()?,
        )
    })
}

/// Write a `ui_state` value (upsert).
pub fn ui_set(db: &Db, key: &str, value: &str) -> Result<(), StorageError> {
    db.with_conn(|c| {
        c.execute(
            "INSERT INTO ui_state (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    })
}

/// Read a settings override (None if the default from primary data applies).
pub fn settings_get(db: &Db, key: &str) -> Result<Option<String>, StorageError> {
    db.with_conn(|c| {
        Ok(
            c.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()?,
        )
    })
}

/// Write a settings override (upsert). The key must exist in the SSOT list
/// `data/settings-defaults.toml` (`primary_data::settings::keys()`); an unknown key is
/// rejected with BK-DATA-0003 so the defaults file stays the single source of truth.
pub fn settings_set(db: &Db, key: &str, value: &str) -> Result<(), StorageError> {
    if !bookkeepy_primary_data::settings::keys().contains(&key) {
        return Err(StorageError::UnknownSetting {
            key: key.to_string(),
        });
    }
    db.with_conn(|c| {
        c.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    })
}

/// Resolve a setting: user override if present, else the primary-data default.
pub fn settings_resolve(db: &Db, key: &str, default: &str) -> Result<String, StorageError> {
    Ok(settings_get(db, key)?.unwrap_or_else(|| default.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kv_roundtrip_and_resolve() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        assert!(ui_get(&db, "window").unwrap().is_none());
        ui_set(&db, "window", "0,0,1200,800").unwrap();
        assert_eq!(
            ui_get(&db, "window").unwrap().as_deref(),
            Some("0,0,1200,800")
        );
        assert_eq!(settings_resolve(&db, "theme", "system").unwrap(), "system");
        settings_set(&db, "theme", "dark").unwrap();
        assert_eq!(settings_resolve(&db, "theme", "system").unwrap(), "dark");
        // unknown keys are rejected against the SSOT (BK-DATA-0003)
        let err = settings_set(&db, "no_such_key", "x").unwrap_err();
        assert_eq!(err.code(), "BK-DATA-0003");
    }
}
