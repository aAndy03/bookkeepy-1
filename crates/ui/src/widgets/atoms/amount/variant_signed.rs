// SPDX-License-Identifier: GPL-3.0-or-later
//! The `signed` variant of [`Amount`] (ADR-0017): a variant is NEVER a copy — it is the
//! source component with one registered capability trait toggled (see
//! `capabilities::REGISTERED`, id `signed`; `meta.toml` declares the link, `govern ui`
//! enforces it). Signed framing gives debits and credits the same visual weight:
//! positives gain an explicit `+` sign instead of reading as "unsigned leftovers".

use gpui_kit::SharedString;

use super::source::{Amount, decimal_value, with_currency};

impl Amount {
    /// Toggle the `signed` capability trait on: explicit `+`/`-` framing, so balanced
    /// pairs (`+59.00` / `-59.00`) scan as an algebraic pair (RULE-UI-007).
    pub fn signed(mut self) -> Self {
        self.set_signed();
        self
    }
}

/// The display text for `money` under the `signed` toggle: plain decimals when off;
/// explicit `+`/`-` prefixes (minus from `to_decimal_string`) when on. The zero edge
/// case never gains a sign (see `edge_case_zero.rs`).
pub(crate) fn display_text(money: &bookkeepy_core::Money, signed: bool) -> SharedString {
    let text = with_currency(decimal_value(money), money);
    if signed && !money.is_zero() && !money.is_negative() {
        SharedString::from(format!("+{text}"))
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_core::{CurrencyCode, Money};

    #[test]
    fn signed_prefixes_positive_and_keeps_negative() {
        let pos = Money::new(500, CurrencyCode::new("EUR").unwrap());
        let neg = Money::new(-500, CurrencyCode::new("EUR").unwrap());
        assert_eq!(display_text(&pos, true).as_ref(), "+5.00 EUR");
        assert_eq!(display_text(&neg, true).as_ref(), "-5.00 EUR");
    }

    #[test]
    fn unsigned_never_prefixes() {
        let pos = Money::new(500, CurrencyCode::new("EUR").unwrap());
        assert_eq!(display_text(&pos, false).as_ref(), "5.00 EUR");
    }
}
