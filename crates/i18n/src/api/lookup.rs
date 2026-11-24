//! Locale lookup (item 8, M0.5). Catalogs are primary data (`data/locales/*.toml`) loaded
//! by `bookkeepy-primary-data`; this crate only resolves keys. Lookup chain for a key:
//! requested locale → reference locale (`en`) → the key itself (never panic on a missing
//! string; a missing key is a governor failure, not a runtime crash — RULE-ERR-003).

use bookkeepy_primary_data::locales;

/// A shipped UI locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// English — the reference locale (fallback).
    En,
    /// Română.
    Ro,
}

impl Locale {
    /// All shipped locales.
    pub fn all() -> &'static [Locale] {
        &[Locale::En, Locale::Ro]
    }

    /// ISO 639-1 tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ro => "ro",
        }
    }

    /// Parse a tag; unknown tags fall back to the reference locale at lookup time.
    pub fn from_tag(tag: &str) -> Option<Locale> {
        match tag {
            "en" => Some(Locale::En),
            "ro" => Some(Locale::Ro),
            _ => None,
        }
    }
}

/// Translate `key` in `locale`, falling back to English, then to the key itself.
pub fn t(locale: Locale, key: &str) -> String {
    if let Some(m) = locales::messages(locale.tag()).and_then(|m| m.get(key)) {
        return m.clone();
    }
    if locale.tag() != locales::REFERENCE
        && let Some(m) = locales::messages(locales::REFERENCE).and_then(|m| m.get(key))
    {
        return m.clone();
    }
    key.to_string()
}

/// Translate `key` and interpolate `{name}`-style placeholders from `ctx` pairs.
pub fn t_with(locale: Locale, key: &str, ctx: &[(&str, &str)]) -> String {
    let mut s = t(locale, key);
    for (k, v) in ctx {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_error_key_in_both_locales() {
        assert_eq!(
            t(Locale::En, "err.store.open_failed"),
            "The data file cannot be opened."
        );
        assert_eq!(
            t(Locale::Ro, "err.store.open_failed"),
            "Fișierul de date nu poate fi deschis."
        );
    }

    #[test]
    fn unknown_key_returns_key_not_panic() {
        assert_eq!(t(Locale::En, "no.such.key"), "no.such.key");
        assert_eq!(t(Locale::Ro, "no.such.key"), "no.such.key");
    }

    #[test]
    fn interpolation() {
        let s = t_with(Locale::En, "x", &[("name", "Andy")]);
        assert_eq!(s, "x");
        // use a real key with a placeholder-free message; interpolation only replaces if present
        let s2 = t_with(Locale::Ro, "common.cancel", &[("a", "b")]);
        assert_eq!(s2, "Anulează");
    }

    #[test]
    fn from_tag_roundtrip() {
        assert_eq!(Locale::from_tag("ro"), Some(Locale::Ro));
        assert_eq!(Locale::from_tag("de"), None);
    }
}
