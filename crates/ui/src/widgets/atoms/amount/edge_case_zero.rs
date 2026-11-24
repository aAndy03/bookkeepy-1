// SPDX-License-Identifier: GPL-3.0-or-later
//! The `zero` edge case of [`Amount`] (RULE-UI-008): a money value that exists but is
//! nothing. Zero renders in the muted foreground (not the danger hue — a zero balance
//! is information, not an error) and never gains a sign, under any variant.

use bookkeepy_core::Money;

use super::source::Amount;

/// Whether `money` is the zero edge case.
pub(crate) fn is_zero(money: &Money) -> bool {
    money.is_zero()
}

impl Amount {
    /// Build the zero edge case for `currency` — the demo/showcase form
    /// (`Amount::new(Money::zero(...))` is the app-facing equivalent).
    pub fn zero(currency: bookkeepy_core::CurrencyCode) -> Self {
        Self::new(Money::zero(currency))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_core::CurrencyCode;

    #[test]
    fn zero_is_the_edge_case() {
        assert!(is_zero(&Money::zero(CurrencyCode::new("EUR").unwrap())));
        assert!(!is_zero(&Money::new(1, CurrencyCode::new("EUR").unwrap())));
    }
}
