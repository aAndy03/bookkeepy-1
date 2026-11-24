// SPDX-License-Identifier: GPL-3.0-or-later
//! Design-language styling builders: the expansion of the ATOMS · 01 control-size scale
//! and the solid functional fills to the exact gpui-kit component APIs (RULE-UI-005/007).
//!
//! The theme bridge (`crate::theme`) carries everything the `ThemeConfig` seats reach.
//! These builders cover the two things it cannot:
//!
//! 1. The **control-size scale** — ATOMS · 01 draws controls at 20/24/32/40px with radii
//!    `mini`/`item`/`control`/`control_lg`. gpui sizes halt at 32 (its `large` is a padded
//!    32), so the scale is applied per instance through the components' own
//!    [`gpui_kit::component::Sizable`] and `rounded` APIs. Heights come from the
//!    `data/theme/spacing.toml` grid (x5/x6/x8/x10 — the same chips), radii from
//!    `radii.toml`: never a raw px in code.
//! 2. **The danger pressed fill** — gpui's functional variants render the rest states
//!    (the calm well + tint ink, mapped by the bridge); the design's "solid red when
//!    pressed" has no variant seat, so it rides the public `Styled` instance refinement,
//!    which gpui-kit applies after its variant paints.
//!
//! No value here originates in code: every color resolves through the active theme mode's
//! `data/theme/colors.toml` tokens; a corrupted token degrades to the component's own
//! themed fallback instead of panicking (RULE-DATA-001).

use bookkeepy_primary_data::theme::{Mode, colors, radii, spacing};
use gpui_kit::component::button::{Button, ButtonRounded, ButtonVariant, ButtonVariants as _};
use gpui_kit::component::color_picker::ColorPicker;
use gpui_kit::component::theme::try_parse_color;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{Hsla, Styled as _, px};

/// The design language's control-size scale (ATOMS · 01 size chips):
/// 20/24/32/40px tall, radii `mini`/`item`/`control`/`control_lg`.
///
/// The old sheets drew the scale on buttons only; gpui-kit 0.11 carries more
/// size-aware components, and every one of them is sized through this scale, so
/// the whole gallery stays on the same ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSize {
    /// 20px — chips and micro controls (radius token `mini`).
    XSmall,
    /// 24px — dense controls (radius token `item`).
    Small,
    /// 32px — the standard control (radius token `control`).
    Medium,
    /// 40px — large controls (radius token `control_lg`).
    Large,
}

impl ControlSize {
    /// Height on the scale: the `data/theme/spacing.toml` grid values (x5/x6/x8/x10).
    pub fn height(self) -> u32 {
        let s = spacing();
        match self {
            Self::XSmall => s.x5,
            Self::Small => s.x6,
            Self::Medium => s.x8,
            Self::Large => s.x10,
        }
    }

    /// Corner radius on the scale: the `data/theme/radii.toml` control tokens.
    pub fn radius(self) -> u32 {
        let r = radii();
        match self {
            Self::XSmall => r.mini,
            Self::Small => r.item,
            Self::Medium => r.control,
            Self::Large => r.control_lg,
        }
    }
}

/// Size a [`Button`] on the design control-size scale — exact height and radius stay
/// on the scale even where gpui's own sizes stop at 32px (its `large`).
///
/// Any further refinement (padding, width) is the caller's via the normal `Styled` API.
pub fn button_size(button: Button, size: ControlSize) -> Button {
    button
        .with_size(Size::Size(px(size.height() as f32)))
        .rounded(ButtonRounded::Size(px(size.radius() as f32)))
}

/// Size a [`ColorPicker`] on the design control-size scale — the only size-aware picker;
/// its swatch rides the same height/radius ladder as buttons (ATOMS · 01/03 grip chips).
pub fn color_picker_size(picker: ColorPicker, size: ControlSize) -> ColorPicker {
    picker.with_size(Size::Size(px(size.height() as f32)))
}

/// Resolve a mode-specific design-token value into a [`Hsla`]; `None` degrades to the
/// component's own themed fallback (RULE-DATA-003: corrupted data never panics the UI).
fn token_hsla(value: &str) -> Option<Hsla> {
    try_parse_color(value).ok()
}

/// Paint `button` as a solid fill of `variant`: `bg` at rest, `fg` as ink.
///
/// The variant itself stays applied — gpui-kit's `InteractiveElement::hover` pseudo-state
/// is a single slot the component claims inside its render, so an instance override would
/// panic ("hover style already set"); hover/press keep the variant's own overlays (the
/// tint @ ~20-40%, drawn over the refined fill), which is the design's brightening press
/// feedback on a solid surface.
fn solid_fill(
    button: Button,
    variant: ButtonVariant,
    bg: Option<Hsla>,
    fg: Option<Hsla>,
) -> Button {
    let mut button = button.with_variant(variant);
    if let Some(bg) = bg {
        button = button.bg(bg);
    }
    if let Some(fg) = fg {
        button = button.text_color(fg);
    }
    button
}

/// The design's pressed danger — solid red with the white glyph (ATOMS · 01: danger is
/// "calm until touched; subtle at rest, solid only when pressed"). gpui's `danger()`
/// renders the calm rest state; this paints the pressed fill for pressed previews and
/// confirm-destruct flows that must start loud.
pub fn solid_danger(button: Button, mode: Mode) -> Button {
    let t = colors(mode);
    solid_fill(
        button,
        ButtonVariant::Danger,
        token_hsla(&t.danger),
        token_hsla(&t.on_solid),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_primary_data::theme::Mode;

    #[test]
    fn control_scale_matches_the_design_sheets() {
        // ATOMS · 01 size chips: 20/24/32/40 with rx 4/6/6/8.
        let ladder = [
            (ControlSize::XSmall, 20, 4),
            (ControlSize::Small, 24, 6),
            (ControlSize::Medium, 32, 6),
            (ControlSize::Large, 40, 8),
        ];
        for (size, height, radius) in ladder {
            assert_eq!(size.height(), height, "{size:?} height");
            assert_eq!(size.radius(), radius, "{size:?} radius");
        }
    }

    #[test]
    fn tokens_resolve_for_both_modes() {
        for mode in [Mode::Dark, Mode::Light] {
            assert!(token_hsla(&colors(mode).accent).is_some(), "{mode:?}");
            assert!(token_hsla("not-a-color").is_none(), "{mode:?}");
        }
    }
}
