//! `data/locales/*.toml` — locale catalogs (item 8, M0.5; RULE-ERR-002/003, RULE-DATA-003).
//! One file per locale, `[meta]` + flat `[messages]` (quoted dotted keys). English (`en`)
//! is the reference locale and the fallback: every key must exist in every shipped locale
//! (parity is enforced here and by `xtask govern errors`), and every `i18n_key` in
//! `data/errors.toml` must have a message in every locale.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::PrimaryDataError;

const EN_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/locales/en.toml"
));
const RO_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/locales/ro.toml"
));

/// Locale file metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct LocaleMeta {
    /// ISO 639-1 tag, must equal the file stem (`en`, `ro`).
    pub locale: String,
    /// Human-readable language name.
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocaleFile {
    meta: LocaleMeta,
    messages: HashMap<String, String>,
}

/// A shipped locale with parsed messages.
#[derive(Debug)]
pub struct Locale {
    /// ISO 639-1 tag.
    pub tag: &'static str,
    /// Human-readable language name.
    pub name: &'static str,
    /// Flat message map (dotted keys).
    pub messages: &'static HashMap<String, String>,
}

fn parse(raw: &str, expect_tag: &str) -> Result<LocaleFile, PrimaryDataError> {
    let file: LocaleFile = toml::from_str(raw).map_err(|e| {
        PrimaryDataError::Invalid(format!(
            "locale file {expect_tag}.toml failed to parse: {e}"
        ))
    })?;
    if file.meta.locale != expect_tag {
        return Err(PrimaryDataError::Invalid(format!(
            "locale file {expect_tag}.toml declares locale {:?}",
            file.meta.locale
        )));
    }
    if file.messages.is_empty() {
        return Err(PrimaryDataError::Invalid(format!(
            "locale {expect_tag} has no messages"
        )));
    }
    Ok(file)
}

fn en_file() -> &'static LocaleFile {
    static FILE: OnceLock<LocaleFile> = OnceLock::new();
    FILE.get_or_init(|| parse(EN_RAW, "en").unwrap_or_else(|e| panic!("{e}")))
}

fn ro_file() -> &'static LocaleFile {
    static FILE: OnceLock<LocaleFile> = OnceLock::new();
    FILE.get_or_init(|| parse(RO_RAW, "ro").unwrap_or_else(|e| panic!("{e}")))
}

/// All shipped locales (reference locale first).
pub fn all() -> &'static [Locale] {
    static LOCALES: OnceLock<Vec<Locale>> = OnceLock::new();
    LOCALES.get_or_init(|| {
        vec![
            Locale {
                tag: "en",
                name: &en_file().meta.name,
                messages: &en_file().messages,
            },
            Locale {
                tag: "ro",
                name: &ro_file().meta.name,
                messages: &ro_file().messages,
            },
        ]
    })
}

/// Reference locale tag (`en`).
pub const REFERENCE: &str = "en";

/// Messages of a locale by tag, if shipped.
pub fn messages(tag: &str) -> Option<&'static HashMap<String, String>> {
    all().iter().find(|l| l.tag == tag).map(|l| l.messages)
}

/// Structural validation: files parse, meta tags match file stems, key parity across locales.
pub fn validate_structure() -> Result<(), PrimaryDataError> {
    // force parse (panics are converted to validation errors via validate entry below)
    let en = parse(EN_RAW, "en")?;
    let ro = parse(RO_RAW, "ro")?;
    let en_keys: std::collections::HashSet<_> = en.messages.keys().collect();
    let ro_keys: std::collections::HashSet<_> = ro.messages.keys().collect();
    let missing_in_ro: Vec<_> = en_keys.difference(&ro_keys).collect();
    let missing_in_en: Vec<_> = ro_keys.difference(&en_keys).collect();
    if !missing_in_ro.is_empty() || !missing_in_en.is_empty() {
        return Err(PrimaryDataError::Invalid(format!(
            "locale key parity violated; missing in ro: {missing_in_ro:?}; missing in en: {missing_in_en:?}"
        )));
    }
    Ok(())
}

/// Every `i18n_key` in the error registry must have a message in every shipped locale.
pub fn validate_covers_errors(errors: &[crate::errors::ErrorDef]) -> Result<(), PrimaryDataError> {
    for e in errors {
        for locale in all() {
            if !locale.messages.contains_key(&e.i18n_key) {
                return Err(PrimaryDataError::Invalid(format!(
                    "error {} i18n_key {:?} has no message in locale {}",
                    e.code, e.i18n_key, locale.tag
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_parse_and_have_parity() {
        validate_structure().unwrap();
    }

    #[test]
    fn reference_locale_is_en() {
        assert_eq!(REFERENCE, "en");
        assert!(messages("en").is_some());
        assert!(messages("ro").is_some());
        assert!(messages("fr").is_none());
    }
}
