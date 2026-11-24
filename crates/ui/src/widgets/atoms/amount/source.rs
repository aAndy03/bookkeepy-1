// SPDX-License-Identifier: GPL-3.0-or-later
//! The SSOT `Amount` component: a money value rendered with the `amount` text style
//! from `data/theme/typography.toml` (RULE-UI-005 — no hardcoded fonts/sizes/colors) and
//! the integer-minor-units formatting of `bookkeepy-core` (ADR-0004).
//!
//! Variants extend [`Amount`] by toggling `signed` only — see `variant_signed.rs`;
//! the zero edge case is factored in `edge_case_zero.rs`. Do NOT fork this file: a new
//! variant is a capability-trait toggle plus a `meta.toml` entry (ADR-0017).

use bookkeepy_core::Money;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{
    App, DefiniteLength, FontFeatures, FontWeight, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled as _, Window, div, px,
};

use super::edge_case_zero;
use super::variant_signed;

/// The nominal minor-unit fallback when a currency is missing from
/// `data/currencies.toml` (ISO 4217's overwhelmingly common case: 2).
const DEFAULT_MINOR_UNITS: u8 = 2;

/// The money amount atom: renders one `Money` per the design language's amount
/// typography, semantic coloring for negatives (danger hue via the installed theme),
/// and the muted zero edge case.
#[derive(IntoElement)]
pub struct Amount {
    money: Money,
    /// The `signed` capability trait's toggle — set ONLY via [`Amount::signed`]
    /// (defined in `variant_signed.rs`, registered in `capabilities::REGISTERED`).
    signed: bool,
}

impl Amount {
    /// Render `money` as the default (unsigned) amount display.
    pub fn new(money: Money) -> Self {
        Self {
            money,
            signed: false,
        }
    }

    /// Flip the `signed` toggle — called only from the `variant_signed` variant,
    /// so the trait's code path stays in its own file.
    pub(crate) fn set_signed(&mut self) {
        self.signed = true;
    }
}

/// The formatted decimal value (e.g. `12.34`), using the currency's minor units
/// from primary data (`data/currencies.toml` — `Money::to_decimal_string` doc).
pub(crate) fn decimal_value(money: &Money) -> String {
    let minor = bookkeepy_primary_data::currencies::get(money.currency().as_str())
        .and_then(|c| c.minor_units.as_u8())
        .unwrap_or(DEFAULT_MINOR_UNITS);
    money.to_decimal_string(minor)
}

impl RenderOnce for Amount {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // The `amount` text style from data/theme/typography.toml — its absence (a
        // corrupted token file) degrades to sane defaults instead of panicking.
        let style = bookkeepy_primary_data::theme::style("amount");
        let fonts = bookkeepy_primary_data::theme::typography();
        let family = style
            .map(|s| {
                if s.font == "mono" {
                    &fonts.fonts().mono
                } else {
                    &fonts.fonts().ui
                }
            })
            .unwrap_or(&fonts.fonts().ui);

        let theme = cx.theme();
        let color = if edge_case_zero::is_zero(&self.money) {
            theme.muted_foreground
        } else if self.money.is_negative() {
            theme.danger
        } else {
            theme.foreground
        };

        let mut el = div()
            .font_family(family.clone())
            .text_color(color)
            .child(variant_signed::display_text(&self.money, self.signed));
        if let Some(s) = style {
            el = el
                .text_size(px(s.size_px as f32))
                .font_weight(FontWeight::from(s.weight as f32))
                .line_height(DefiniteLength::Fraction(s.line_height));
            // `letter_spacing_px` has no gpui seam yet (the `amount` token is 0).
            if s.tabular {
                el = el.font_features(FontFeatures(std::sync::Arc::new(vec![("tnum".into(), 1)])));
            }
        }
        el
    }
}

/// `display_text`'s currency separator: `"12.34 RON"`.
pub(crate) fn with_currency(decimal: String, money: &Money) -> SharedString {
    SharedString::from(format!("{decimal} {}", money.currency().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_core::CurrencyCode;

    fn money(amount_minor: i64) -> Money {
        Money::new(amount_minor, CurrencyCode::new("RON").unwrap())
    }

    #[test]
    fn formats_decimal_with_currency() {
        let m = money(1234);
        assert_eq!(with_currency(decimal_value(&m), &m).as_ref(), "12.34 RON");
    }

    #[test]
    fn default_is_unsigned_even_for_positives() {
        let m = money(500);
        assert!(!variant_signed::display_text(&m, false).starts_with('+'));
        assert!(variant_signed::display_text(&m, true).starts_with('+'));
    }
}
