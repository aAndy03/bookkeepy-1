// SPDX-License-Identifier: GPL-3.0-or-later
//! Localized error text (RULE-ERR-002): a BK-code maps through the error registry's
//! `i18n_key` to the message in the chosen locale. Every registered code has a message in
//! every shipped locale (enforced by `cargo xtask govern errors`), so lookup never fails;
//! an unparseable/unknown code is returned as-is so it stays copyable.

use bookkeepy_core::ErrorCode;
use bookkeepy_primary_data::errors;

use super::lookup::{Locale, t};

/// Localized, user-facing text for a BK error code (e.g. `BK-STORE-0001`).
pub fn error_text(locale: Locale, code: &str) -> String {
    if let Ok(c) = ErrorCode::new(code)
        && let Some(def) = errors::get(&c)
    {
        return t(locale, &def.i18n_key);
    }
    code.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_registered_code_in_both_locales() {
        assert_eq!(
            error_text(Locale::En, "BK-STORE-0004"),
            "This profile is already open in another Bookkeepy window."
        );
        assert_eq!(
            error_text(Locale::Ro, "BK-DATA-0003"),
            "Setare necunoscută — nu este definită în lista de setări ale aplicației."
        );
    }

    #[test]
    fn unknown_code_returns_code_itself() {
        assert_eq!(error_text(Locale::En, "BK-NOPE-9999"), "BK-NOPE-9999");
        assert_eq!(error_text(Locale::En, "not-a-code"), "not-a-code");
    }
}
