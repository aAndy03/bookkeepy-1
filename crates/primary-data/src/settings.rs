// SPDX-License-Identifier: GPL-3.0-or-later
//! `data/settings-defaults.toml` — settings schema + defaults (round-5 item 5).
//! SSOT for setting keys, defaults and allowed values; the profile DB stores user
//! overrides against these keys. Validation cross-references other primary data
//! (RULE-DATA-002): language ↔ locales, default_currency ↔ currencies.toml.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::{PrimaryDataError, currencies, locales};

const RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/settings-defaults.toml"
));

/// Parsed settings-defaults file.
#[derive(Debug, Clone, Deserialize)]
pub struct SettingsDefaults {
    /// The settings table.
    pub settings: Settings,
}

/// One setting key's default + allowed values are enforced at validation time.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// `light | dark | system`.
    pub theme: String,
    /// Locale tag, must match a shipped locale.
    pub language: String,
    /// Date display format id.
    pub date_format: String,
    /// `grouped | plain`.
    pub number_format: String,
    /// `monday | sunday`.
    pub first_day_of_week: String,
    /// IANA timezone or empty for system.
    pub timezone: String,
    /// ISO 4217 code, must exist in currencies.toml.
    pub default_currency: String,
    /// Ordered source ids (doc 11).
    pub rates_source_priority: Vec<String>,
    /// Always show ISO code next to amounts (RULE-FMT-001).
    pub show_currency_code: bool,
}

fn file() -> &'static SettingsDefaults {
    static FILE: OnceLock<SettingsDefaults> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(RAW)
            .unwrap_or_else(|e| panic!("primary data settings-defaults.toml failed to parse: {e}"))
    })
}

/// All settings defaults.
pub fn get() -> &'static Settings {
    &file().settings
}

/// All known setting keys — the SSOT list (RULE-SSOT-001). The profile `settings` table
/// may only store overrides for these keys; `bookkeepy-storage` validates against this
/// (BK-DATA-0003 on an unknown key).
pub fn keys() -> &'static [&'static str] {
    &[
        "theme",
        "language",
        "date_format",
        "number_format",
        "first_day_of_week",
        "timezone",
        "default_currency",
        "rates_source_priority",
        "show_currency_code",
    ]
}

/// The default of `key` as a string (for `settings_snapshot` in storage), or None if the
/// key is not in the SSOT list. Must stay in sync with [`keys`] (tested below).
pub fn default_value(key: &str) -> Option<String> {
    let s = get();
    Some(match key {
        "theme" => s.theme.clone(),
        "language" => s.language.clone(),
        "date_format" => s.date_format.clone(),
        "number_format" => s.number_format.clone(),
        "first_day_of_week" => s.first_day_of_week.clone(),
        "timezone" => s.timezone.clone(),
        "default_currency" => s.default_currency.clone(),
        "rates_source_priority" => s.rates_source_priority.join(","),
        "show_currency_code" => s.show_currency_code.to_string(),
        _ => return None,
    })
}

/// Structural + referential validation (RULE-DATA-002).
pub fn validate_structure() -> Result<(), PrimaryDataError> {
    let s = get();
    if !["light", "dark", "system"].contains(&s.theme.as_str()) {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.theme must be light|dark|system, got {:?}",
            s.theme
        )));
    }
    if !locales::all().iter().any(|l| l.tag == s.language) {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.language {:?} is not a shipped locale (data/locales)",
            s.language
        )));
    }
    if !["yyyy-MM-dd", "dd.MM.yyyy", "MM/dd/yyyy"].contains(&s.date_format.as_str()) {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.date_format unknown: {:?}",
            s.date_format
        )));
    }
    if !["grouped", "plain"].contains(&s.number_format.as_str()) {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.number_format must be grouped|plain, got {:?}",
            s.number_format
        )));
    }
    if !["monday", "sunday"].contains(&s.first_day_of_week.as_str()) {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.first_day_of_week must be monday|sunday, got {:?}",
            s.first_day_of_week
        )));
    }
    if !s.timezone.is_empty() && jiff_tz_ok(&s.timezone).is_none() {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.timezone {:?} is not a valid IANA name",
            s.timezone
        )));
    }
    if currencies::get(&s.default_currency).is_none() {
        return Err(PrimaryDataError::Invalid(format!(
            "settings.default_currency {:?} not in currencies.toml",
            s.default_currency
        )));
    }
    if s.rates_source_priority.is_empty() {
        return Err(PrimaryDataError::Invalid(
            "settings.rates_source_priority must list at least one source id".into(),
        ));
    }
    Ok(())
}

fn jiff_tz_ok(tz: &str) -> Option<()> {
    jiff::tz::TimeZone::get(tz).ok().map(|_| ())
}

#[cfg(test)]
mod tests {
    #[test]
    fn settings_validate() {
        super::validate_structure().unwrap();
        assert_eq!(super::get().language, "en");
        assert_eq!(super::get().default_currency, "RON");
    }

    #[test]
    fn keys_and_default_values_stay_in_sync() {
        for k in super::keys() {
            assert!(
                super::default_value(k).is_some(),
                "SSOT key {k} has no default_value mapping"
            );
        }
        assert!(super::default_value("no_such_key").is_none());
    }
}
