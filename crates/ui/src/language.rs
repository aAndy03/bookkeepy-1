// SPDX-License-Identifier: GPL-3.0-or-later
//! The design-language **toolkit** — how any component (existing gpui-kit or new
//! Bookkeepy widget) gets the exact design look without re-deriving it (RULE-UI-005/007).
//!
//! Two layers, in order:
//!
//! 1. **The theme bridge** (`crate::theme`) is the primary method: it maps every
//!    `ThemeConfig` seat to a `data/theme/` token, so the stock components' defaults
//!    already ARE the design language. A new component that only uses themed seats
//!    needs nothing from here.
//! 2. **These preset modules** cover what the config cannot reach — per-family
//!    builders that take the component's own public API and pin the sheet-exact
//!    geometry/fills from the tokens. Prefer adding a preset here over styling at
//!    a call site: one place, token-driven, testable.
//!
//! No value originates in code: every color/size resolves through the active theme
//! mode's `data/theme/*.toml` tokens; a corrupted token degrades to the component's
//! own themed fallback instead of panicking (RULE-DATA-001/003).

use bookkeepy_primary_data::theme::{Mode, colors, radii, spacing};
use gpui_kit::component::button::{Button, ButtonRounded, ButtonVariant, ButtonVariants as _};
use gpui_kit::component::color_picker::ColorPicker;
use gpui_kit::component::theme::try_parse_color;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{Hsla, Styled as _, px};

/// Resolve a mode-specific design-token value into a [`Hsla`]; `None` degrades to the
/// component's own themed fallback (RULE-DATA-003: corrupted data never panics the UI).
fn token_hsla(value: &str) -> Option<Hsla> {
    try_parse_color(value).ok()
}

/// Shell presets — TEMPLATES · shell + ORGANISMS · 01, so the app shell and any new
/// window build identically from the tokens: one call per shell region.
pub mod shell {
    use bookkeepy_primary_data::theme::layout;
    use gpui_kit::component::sidebar::{Sidebar, SidebarItem};
    use gpui_kit::{Styled as _, px};

    /// The app sidebar at the design's expanded width (ORGANISMS · 01: 284px; the
    /// collapsed 72px state is the same component with `.collapsed(true)`) with the
    /// 2px trailing divider line (same sheet: "2px border-strong on the trailing edge").
    pub fn sidebar<E: SidebarItem>(sidebar: Sidebar<E>) -> Sidebar<E> {
        sidebar
            .w(px(layout().sidebar_expanded_px as f32))
            .border_r(px(layout().sidebar_divider_px as f32))
    }
}

/// Surface presets — the design's panel surfaces (sheets draw caption/spec cards as
/// `#171717` rx16; popovers/menus ride `#222222` via the bridge's `popover` seat).
pub mod surface {
    use super::{Mode, token_hsla};
    use bookkeepy_primary_data::theme::{colors, radii};
    use gpui_kit::{Div, Styled as _, px};

    /// A spec/demo card: sheet panel fill + overlay radius, NO border — the design
    /// language avoids hard separators on cards (ATOMS/ORGANISMS sheets: caption
    /// panels are plain `#171717` rx16 fills). `mode` = the active theme mode.
    pub fn card(div: Div, mode: Mode) -> Div {
        let t = colors(mode);
        let mut div = div;
        if let Some(bg) = token_hsla(&t.bg_sheet) {
            div = div.bg(bg);
        }
        div.rounded(px(radii().overlay as f32))
    }

    /// A dialog panel on the design surface: `#1c1c1c` rx20 borderless over the scrim
    /// (ORGANISMS · 01 dialog: 440x327 rx20 #1c1c1c on #000@60%). gpui's dialog bg reads
    /// the base background; this pins the drawn panel fill + geometry.
    pub fn dialog<D>(d: D, mode: Mode) -> D
    where
        D: gpui_kit::Styled,
    {
        let t = colors(mode);
        let mut d = d.border_0().rounded(px(radii().banner as f32));
        if let Some(bg) = token_hsla(&t.bg_sheet) {
            d = d.bg(bg);
        }
        d
    }

    /// A group box on the design surface: borderless #171717 rx16 card (ORGANISMS · 02
    /// group box: 440x106 rx16 `#171717`, no border — the hard outline is a gpui default).
    pub fn group_box<G>(g: G, mode: Mode) -> G
    where
        G: gpui_kit::Styled,
    {
        let t = colors(mode);
        let mut g = g.border_0().rounded(px(radii().overlay as f32));
        if let Some(bg) = token_hsla(&t.bg_raised) {
            g = g.bg(bg);
        }
        g
    }
}

/// Display presets — badges/tags/avatars per ATOMS · 02: tags are 20px rx4 pills on
/// `#1c1c1c` with a colored dot; the colored variants sit on the tint WELLS with the
/// tint as ink (not solid hues); avatars are borderless circles.
pub mod display {
    use super::{Mode, token_hsla};
    use bookkeepy_primary_data::theme::{colors, radii};
    use gpui_kit::component::avatar::Avatar;
    use gpui_kit::component::tag::Tag;
    use gpui_kit::{Styled as _, px};

    /// The tag tone, selecting the matching `*_well` + `*_tint` token pair (ATOMS · 02).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TagTone {
        /// `accent_well` + `accent_hover`.
        Accent,
        /// `success_well` + `success_tint`.
        Success,
        /// `warning_well` + `warning_tint`.
        Warning,
        /// `danger_well` + `danger_tint`.
        Danger,
        /// `info_well` + `info_tint`.
        Info,
    }

    /// A soft tag: tint-well fill + tint ink + rx4 (ATOMS · 02 tinted tag variants).
    pub fn tag_soft(tag: Tag, tone: TagTone, mode: Mode) -> Tag {
        let t = colors(mode);
        let (well, tint) = match tone {
            TagTone::Accent => (&t.accent_well, &t.accent_hover),
            TagTone::Success => (&t.success_well, &t.success_tint),
            TagTone::Warning => (&t.warning_well, &t.warning_tint),
            TagTone::Danger => (&t.danger_well, &t.danger_tint),
            TagTone::Info => (&t.info_well, &t.info_tint),
        };
        let mut tag = tag.rounded(px(radii().mini as f32));
        if let Some(bg) = token_hsla(well) {
            tag = tag.bg(bg);
        }
        if let Some(fg) = token_hsla(tint) {
            tag = tag.text_color(fg);
        }
        tag
    }

    /// An avatar: borderless circle (ATOMS · 02 — the #2a2a2a ring appears only in
    /// overlap stacks, which AvatarGroup draws itself).
    pub fn avatar(avatar: Avatar) -> Avatar {
        avatar.border_0()
    }
}

/// Control presets — the ATOMS · 01 control-size scale (20/24/32/40px with radii
/// `mini`/`item`/`control`/`control_lg`). The old sheets drew the scale on buttons
/// only; gpui-kit 0.11 carries more size-aware components, and every one of them is
/// sized through this scale, so the whole gallery stays on the same ladder.
pub mod controls {
    use super::*;

    /// The design language's control-size scale (ATOMS · 01 size chips).
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

    /// Size a [`Button`] on the design control-size scale — exact height and radius
    /// stay on the scale even where gpui's own sizes stop at 32px (its `large`).
    pub fn button_size(button: Button, size: ControlSize) -> Button {
        button
            .with_size(Size::Size(px(size.height() as f32)))
            .rounded(ButtonRounded::Size(px(size.radius() as f32)))
    }

    /// Size a [`ColorPicker`] on the design control-size scale — the only size-aware
    /// picker; its swatch rides the same height ladder as buttons (ATOMS · 01/03).
    pub fn color_picker_size(picker: ColorPicker, size: ControlSize) -> ColorPicker {
        picker.with_size(Size::Size(px(size.height() as f32)))
    }

    /// A checkbox on the design geometry — 16x16 rx4 (ATOMS · 02: box 16x16 rx `mini`;
    /// gpui's default rounds it at the control radius 6). Checked fill/ink ride the
    /// `primary` seat (inverted neutral).
    pub fn checkbox(
        c: gpui_kit::component::checkbox::Checkbox,
    ) -> gpui_kit::component::checkbox::Checkbox {
        c.rounded(px(radii().mini as f32))
    }

    /// A radio on the design geometry — r8 outer (ATOMS · 02: r8 + r3 dot, rx = half).
    pub fn radio(r: gpui_kit::component::radio::Radio) -> gpui_kit::component::radio::Radio {
        r.rounded(px(radii().circle as f32))
    }
}

/// Action presets — states the gpui variant API does not carry.
pub mod actions {
    use super::*;

    /// Paint `button` as a solid fill of `variant`: `bg` at rest, `fg` as ink.
    ///
    /// The variant itself stays applied — gpui-kit's `InteractiveElement::hover`
    /// pseudo-state is a single slot the component claims inside its render, so an
    /// instance override would panic ("hover style already set"); hover/press keep the
    /// variant's own overlays (the tint @ ~20-40%, drawn over the refined fill), which
    /// is the design's brightening press feedback on a solid surface.
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

    /// The design's pressed danger — solid red with the white glyph (ATOMS · 01: danger
    /// is "calm until touched; subtle at rest, solid only when pressed"). gpui's
    /// `danger()` renders the calm rest state; this paints the pressed fill for pressed
    /// previews and confirm-destruct flows that must start loud.
    pub fn solid_danger(button: Button, mode: Mode) -> Button {
        let t = colors(mode);
        solid_fill(
            button,
            ButtonVariant::Danger,
            token_hsla(&t.danger),
            token_hsla(&t.on_solid),
        )
    }
}

/// Feedback presets — alert banners are the design's tint WELL fills with NO border and
/// the overlay radius (MOLECULES · 02 alert: 460x55 rx16 in the semantic well, light
/// glyph). gpui's alert bg is a 4% hue wash; this pins the drawn well fill per tone.
pub mod feedback {
    use super::{Mode, token_hsla};
    use bookkeepy_primary_data::theme::{colors, radii};
    use gpui_kit::component::alert::Alert;
    use gpui_kit::{Styled as _, px};

    /// The alert tone, selecting the matching `*_well` token (MOLECULES · 02 alert wells).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AlertTone {
        /// `info_well`.
        Info,
        /// `success_well`.
        Success,
        /// `warning_well`.
        Warning,
        /// `danger_well`.
        Danger,
    }

    /// An alert banner on the design surface: well fill, borderless, rx16. The light
    /// glyph + title ink ride the variant's own theme seats.
    pub fn alert(alert: Alert, tone: AlertTone, mode: Mode) -> Alert {
        let t = colors(mode);
        let well = match tone {
            AlertTone::Info => &t.info_well,
            AlertTone::Success => &t.success_well,
            AlertTone::Warning => &t.warning_well,
            AlertTone::Danger => &t.danger_well,
        };
        let mut alert = alert.border_0().rounded(px(radii().overlay as f32));
        if let Some(bg) = token_hsla(well) {
            alert = alert.bg(bg);
        }
        alert
    }
}

pub use actions::solid_danger;
pub use controls::{ControlSize, button_size, color_picker_size};

#[cfg(test)]
mod tests {
    use super::controls::ControlSize;
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
            assert!(token_hsla(&colors(mode).bg_invert).is_some(), "{mode:?}");
        }
        // A corrupted token degrades to `None` — the themed fallback — never a panic.
        assert!(token_hsla("not-a-color").is_none());
    }
}
