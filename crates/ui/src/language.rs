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
//! Token names are the TOKENS · Bookkeepy v1 sheet's (`surface-raised`, `control`,
//! `white-a8`, `space-8`, radius role `card` …). No value originates in code: every
//! color/size resolves through the active theme mode's `data/theme/*.toml` tokens; a
//! corrupted token degrades to the component's own themed fallback instead of panicking
//! (RULE-DATA-001/003).

use bookkeepy_primary_data::theme::{Mode, colors, palette, radii, spacing};
use gpui_kit::component::button::{Button, ButtonRounded, ButtonVariant, ButtonVariants as _};
use gpui_kit::component::color_picker::ColorPicker;
use gpui_kit::component::theme::try_parse_color;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{Hsla, Styled as _, px};

/// Resolve a raw token value into a [`Hsla`]; `None` degrades to the component's own
/// themed fallback (RULE-DATA-003: corrupted data never panics the UI).
/// Public so gallery call-sites can pin sheet-exact token colors where a component has
/// no theme seat for them.
pub fn token_hsla(value: &str) -> Option<Hsla> {
    try_parse_color(value).ok()
}

/// Resolve a semantic color role of `mode` (e.g. `"surface-raised"`) into a [`Hsla`].
pub fn role_hsla(mode: Mode, role: &str) -> Option<Hsla> {
    colors(mode).get(role).and_then(token_hsla)
}

/// Resolve a raw palette alias (e.g. `"white-a8"`, `"cyan-900"`) into a [`Hsla`].
pub fn prim_hsla(alias: &str) -> Option<Hsla> {
    palette().get(alias).and_then(|s| token_hsla(s))
}

/// A radius role (`control`, `card`, `popover`, `dialog` …) as px (TOKENS §05 roles).
pub fn radius_role(name: &str) -> f32 {
    radii().roles.get(name).copied().unwrap_or(6) as f32
}

/// A radius tier (`sm`, `md`, `lg`, `viewport` …) as px (TOKENS §05 tiers).
pub fn radius_tier(name: &str) -> f32 {
    radii().tiers.get(name).copied().unwrap_or(6) as f32
}

/// A spacing scale token (`"space-8"` = 32px) as u32 (TOKENS §04 scale).
pub fn space(step: &str) -> u32 {
    spacing().scale.get(step).copied().unwrap_or(8)
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

/// Surface presets — the design's panel surfaces (TOKENS §02 surfaces: `surface`
/// #171717 spec panels, `surface-raised` #1c1c1c cards, `popover` #222222 floating
/// panels). All borderless — the design language avoids hard separators.
pub mod surface {
    use super::{Mode, radius_role, role_hsla};
    use gpui_kit::{Div, Styled as _, px};

    /// A card: `surface-raised` fill + radius-card (rx16), NO border (TEMPLATES demo
    /// cards are plain #1c1c1c rx16 fills). `mode` = the active theme mode.
    pub fn card(div: Div, mode: Mode) -> Div {
        let mut div = div.border_0();
        if let Some(bg) = role_hsla(mode, "surface-raised") {
            div = div.bg(bg);
        }
        div.rounded(px(radius_role("card")))
    }

    /// A dialog panel on the design surface: `surface-raised` rx20 borderless over the
    /// scrim (ORGANISMS · 01 dialog: 440x327 rx20 #1c1c1c on scrim black@60%).
    pub fn dialog<D>(d: D, mode: Mode) -> D
    where
        D: gpui_kit::Styled,
    {
        let mut d = d.border_0().rounded(px(radius_role("dialog")));
        if let Some(bg) = role_hsla(mode, "surface-raised") {
            d = d.bg(bg);
        }
        d
    }

    /// A group box: `surface` fill + radius-card, borderless (TOKENS §02 `surface`).
    pub fn group_box<G>(g: G, mode: Mode) -> G
    where
        G: gpui_kit::Styled,
    {
        let mut g = g.border_0().rounded(px(radius_role("card")));
        if let Some(bg) = role_hsla(mode, "surface") {
            g = g.bg(bg);
        }
        g
    }

    /// A spec/demo panel exactly as the design sheets draw their caption cards:
    /// `surface` #171717 rx16, borderless (ATOMS/ORGANISMS/CHARTS sheets).
    pub fn spec_panel(div: Div, mode: Mode) -> Div {
        let mut div = div.border_0();
        if let Some(bg) = role_hsla(mode, "surface") {
            div = div.bg(bg);
        }
        div.rounded(px(radius_role("card")))
    }

    /// A floating panel exactly as the design draws popovers/menus/hover cards:
    /// `popover` #222222 rx12, borderless (MOLECULES · 01 panels).
    pub fn popup_panel(div: Div, mode: Mode) -> Div {
        let mut div = div.border_0();
        if let Some(bg) = role_hsla(mode, "popover") {
            div = div.bg(bg);
        }
        div.rounded(px(radius_role("popover")))
    }
}

/// Display presets — badges/tags/avatars per ATOMS · 02 + TOKENS §02: tags are pills on
/// the hue wells (900) with the light tint (300) as ink; the neutral tag sits on
/// `control`; avatars are borderless circles.
pub mod display {
    use super::{Mode, prim_hsla, radius_tier, role_hsla};
    use bookkeepy_primary_data::theme::style;
    use gpui_kit::assets::IconName;
    use gpui_kit::base::h_flex;
    use gpui_kit::component::avatar::Avatar;
    use gpui_kit::component::tag::Tag;
    use gpui_kit::{
        DefiniteLength, Div, FontWeight, IntoElement, ParentElement as _, SharedString,
        Styled as _, div, px,
    };

    /// Apply a named type style (`data/theme/typography.toml`) to any styled element.
    fn type_style<S: gpui_kit::Styled>(el: S, id: &str) -> S {
        let mut el = el;
        if let Some(s) = style(id) {
            el = el
                .text_size(px(s.size_px as f32))
                .line_height(DefiniteLength::Fraction(
                    s.line_height_px as f32 / s.size_px as f32,
                ))
                .font_weight(FontWeight::from(s.weight as f32));
            if s.tabular {
                el = el.font_features(gpui_kit::FontFeatures(std::sync::Arc::new(vec![(
                    "tnum".into(),
                    1,
                )])));
            }
        }
        el
    }

    /// The tag tone, selecting the matching well + tint token pair (ATOMS · 02).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TagTone {
        /// The neutral tag: `control` fill + `secondary-foreground` ink.
        Neutral,
        /// `accent-subtle` well + `accent-foreground` tint.
        Accent,
        /// `success-subtle` + `success-foreground`.
        Success,
        /// `warning-subtle` well + `warning-foreground`.
        Warning,
        /// `danger-subtle` + `danger-foreground`.
        Danger,
        /// `info-subtle` well + `info-foreground`.
        Info,
    }

    impl TagTone {
        /// (well fill, tint ink) for the subtle style; the outline style reuses the
        /// hue-500 @ 40% border + hue-300 ink.
        fn well_tint(self, mode: Mode) -> (Option<gpui_kit::Hsla>, Option<gpui_kit::Hsla>) {
            match self {
                TagTone::Neutral => (
                    role_hsla(mode, "control"),
                    role_hsla(mode, "secondary-foreground"),
                ),
                TagTone::Accent => (
                    role_hsla(mode, "accent-subtle"),
                    role_hsla(mode, "accent-foreground"),
                ),
                TagTone::Success => (
                    role_hsla(mode, "success-subtle"),
                    role_hsla(mode, "success-foreground"),
                ),
                TagTone::Warning => (
                    role_hsla(mode, "warning-subtle"),
                    role_hsla(mode, "warning-foreground"),
                ),
                TagTone::Danger => (
                    role_hsla(mode, "danger-subtle"),
                    role_hsla(mode, "danger-foreground"),
                ),
                TagTone::Info => (
                    role_hsla(mode, "info-subtle"),
                    role_hsla(mode, "info-foreground"),
                ),
            }
        }

        /// hue-500 @ 40% — the outline border color (e.g. `#f5823a66` for accent).
        fn border_40(self) -> Option<gpui_kit::Hsla> {
            let alias = match self {
                TagTone::Neutral => "neutral-700",
                TagTone::Accent => "orange-500",
                TagTone::Success => "green-500",
                TagTone::Warning => "amber-500",
                TagTone::Danger => "red-500",
                TagTone::Info => "cyan-500",
            };
            prim_hsla(alias).map(|c| c.opacity(0.4))
        }
    }

    /// The sheet's tag geometry: h20, padding 0 6px, radius-sm (4), `tag` type style
    /// (11/500 lh15). The 24px size variant is [`tag_tall`]; filter pills use
    /// [`Tag::rounded_full`]. Instance refinement lands last in Tag's render, so these
    /// win over the component's own padding/size mapping.
    fn tag_geometry(tag: Tag) -> Tag {
        type_style(tag.h(px(20.)).px(px(6.)), "tag")
            .rounded(px(radius_tier("sm")))
            .gap(px(4.))
    }

    /// A subtle tag: tone well fill + tint ink, no border (ATOMS · 02 tone × style:
    /// subtle = the default state).
    pub fn tag_soft(tag: Tag, tone: TagTone, mode: Mode) -> Tag {
        let (well, tint) = tone.well_tint(mode);
        let mut tag = tag_geometry(tag).border_0();
        if let Some(bg) = well {
            tag = tag.bg(bg);
        }
        if let Some(fg) = tint {
            tag = tag.text_color(fg);
        }
        tag
    }

    /// An outline tag: transparent fill, 1px hue-500 @ 40% border, tint ink.
    pub fn tag_outline(tag: Tag, tone: TagTone, mode: Mode) -> Tag {
        let (_, tint) = tone.well_tint(mode);
        let mut tag = tag_geometry(tag)
            .bg(gpui_kit::transparent_white())
            .border_1();
        if let Some(b) = tone.border_40() {
            tag = tag.border_color(b);
        }
        if let Some(fg) = tint {
            tag = tag.text_color(fg);
        }
        tag
    }

    /// The 24px tag size (padding 0 8px, same type style).
    pub fn tag_tall(tag: Tag) -> Tag {
        type_style(tag.h(px(24.)).px(px(8.)), "tag")
    }

    /// The badge tone — one semantic color family per tone (ATOMS · 02: neutral ·
    /// accent · success · warning · danger · info).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BadgeTone {
        /// neutral-200 fill, dark ink.
        Neutral,
        /// orange-500 fill, dark ink.
        Accent,
        /// green-500 fill, dark ink.
        Success,
        /// amber-500 fill, dark ink.
        Warning,
        /// red-500 fill, white ink.
        Danger,
        /// cyan-500 fill, dark ink.
        Info,
    }

    impl BadgeTone {
        /// (fill, ink): hue-500 fill with dark ink, except danger (red-500 + white)
        /// and neutral (neutral-200 + neutral-925).
        fn fill_ink(self) -> (Option<gpui_kit::Hsla>, Option<gpui_kit::Hsla>) {
            let (fill, dark) = match self {
                BadgeTone::Neutral => ("neutral-200", true),
                BadgeTone::Accent => ("orange-500", true),
                BadgeTone::Success => ("green-500", true),
                BadgeTone::Warning => ("amber-500", true),
                BadgeTone::Danger => ("red-500", false),
                BadgeTone::Info => ("cyan-500", true),
            };
            (
                prim_hsla(fill),
                prim_hsla(if dark { "neutral-925" } else { "neutral-0" }),
            )
        }
    }

    /// A count badge: h16 pill, padding 0 5px, `badge` type style (10/600 lh12,
    /// tabular), tone fill + ink. `count == 0` renders nothing (the sheet hides it).
    pub fn badge_count(count: usize, tone: BadgeTone) -> Option<Div> {
        if count == 0 {
            return None;
        }
        let (fill, ink) = tone.fill_ink();
        let mut el = type_style(
            h_flex()
                .h(px(16.))
                .px(px(5.))
                .items_center()
                .justify_center()
                .rounded_full()
                .child(count.to_string()),
            "badge",
        );
        if let Some(bg) = fill {
            el = el.bg(bg);
        }
        if let Some(fg) = ink {
            el = el.text_color(fg);
        }
        Some(el)
    }

    /// Count badge as an element; renders an empty placeholder when `count == 0`.
    pub fn badge_count_el(count: usize, tone: BadgeTone) -> Div {
        badge_count(count, tone).unwrap_or_else(div)
    }

    /// A status dot: 8px circle with a 2px ring of the surface it sits on (gpui 0.3
    /// has no outline property, so the ring is a 2px border inside a 12px box — the
    /// same 8+2 geometry the sheet draws as outline).
    pub fn badge_dot(tone: BadgeTone, ring: gpui_kit::Hsla) -> Div {
        let (fill, _) = tone.fill_ink();
        let mut el = div()
            .size(px(12.))
            .rounded_full()
            .border_2()
            .border_color(ring);
        if let Some(bg) = fill {
            el = el.bg(bg);
        }
        el
    }

    /// An icon badge: 16px disc, 10px glyph in the tone ink, bottom-right anchored.
    pub fn badge_icon(icon: IconName, tone: BadgeTone) -> Div {
        let (fill, ink) = tone.fill_ink();
        let mut el = h_flex()
            .size(px(16.))
            .rounded_full()
            .items_center()
            .justify_center()
            .child(super::atoms::icon(icon, 10.));
        if let Some(bg) = fill {
            el = el.bg(bg);
        }
        if let Some(fg) = ink {
            el = el.text_color(fg);
        }
        el
    }

    /// Anchor a badge over a child element (bell icon, avatar…): the count sits at the
    /// top-right corner, the icon badge at the bottom-right (ATOMS · 02 anchored row).
    pub fn badge_anchored(anchor: impl IntoElement, badge: Div, top_right: bool) -> Div {
        // self_start so the wrapper hugs the anchor in a column-flex cell — a stretched block would
        // stretch to the cell width and fling the absolute badge to the cell corner.
        // Offsets overlap the anchor's corner like the sheet (pill over the icon's
        // top-right, dot over the avatar's bottom-right).
        let badge = badge.absolute();
        let badge = if top_right {
            badge.top(px(-6.)).right(px(-10.))
        } else {
            badge.bottom(px(-6.)).right(px(-6.))
        };
        div().relative().self_start().child(anchor).child(badge)
    }

    /// The avatar hue ramp — initials pick a well/tint pair deterministically from the
    /// name (ATOMS · 02: hue-900 disc, hue-300 letters, chosen from the name).
    const AVATAR_HUES: &[&str] = &[
        "cyan", "lime", "pink", "violet", "orange", "green", "amber", "red", "sand", "steel",
    ];

    fn name_hue(name: &str) -> &'static str {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        name.hash(&mut h);
        AVATAR_HUES[(h.finish() as usize) % AVATAR_HUES.len()]
    }

    fn initials_of(name: &str) -> SharedString {
        // Two words → first letter of each ("Andy Motor" → AM); one word → first two
        // letters ("IO" → IO).
        let mut words = name.split_whitespace();
        let first = words.next().unwrap_or("?");
        let second = words.next();
        let initials = match second {
            Some(w) => format!(
                "{}{}",
                first.chars().next().unwrap_or('?').to_uppercase(),
                w.chars().next().unwrap_or('?').to_uppercase()
            ),
            None => first.chars().take(2).collect::<String>().to_uppercase(),
        };
        SharedString::from(initials)
    }

    /// Initials avatar at an exact size (16/20/24/32/48): hue-900 disc + hue-300
    /// letters, weight 600, text scaled per the sheet (6/8/10/13/19px).
    pub fn avatar_initials(name: &str, size_px: u32) -> Div {
        let hue = name_hue(name);
        let well = prim_hsla(&format!("{hue}-900"));
        let tint = prim_hsla(&format!("{hue}-300"));
        let (fs, lh) = match size_px {
            16 => (6., 7.),
            20 => (8., 10.),
            24 => (10., 12.),
            32 => (13., 16.),
            48 => (19., 23.),
            s => (s as f32 * 0.4, s as f32 * 0.48),
        };
        let mut el = div()
            .size(px(size_px as f32))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(fs))
            .line_height(px(lh))
            .font_weight(FontWeight::SEMIBOLD)
            .child(initials_of(name));
        if let Some(bg) = well {
            el = el.bg(bg);
        }
        if let Some(fg) = tint {
            el = el.text_color(fg);
        }
        el
    }

    /// Placeholder avatar: `neutral-850` disc + user glyph in `neutral-400` at 60% of
    /// the size (ATOMS · 02 placeholder kind).
    pub fn avatar_placeholder(size_px: u32, mode: Mode) -> Div {
        let mut el = div()
            .size(px(size_px as f32))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .child(super::atoms::icon(IconName::User, size_px as f32 * 0.6));
        if let Some(bg) = role_hsla(mode, "surface-overlay") {
            el = el.bg(bg);
        }
        if let Some(fg) = role_hsla(mode, "icon-muted") {
            el = el.text_color(fg);
        }
        el
    }

    /// The "current profile" ring: 2px accent @ 40% drawn OUTSIDE the disc (gpui 0.3
    /// has no outline property, so the ring is a padded wrapper — the sheet's 32px
    /// disc + 2px outline = 36px overall).
    pub fn avatar_current(el: Div) -> Div {
        match prim_hsla("orange-500") {
            Some(c) => div().p(px(2.)).rounded_full().bg(c.opacity(0.4)).child(el),
            None => el,
        }
    }

    /// An overlap-stack item: 2px ring of the surface it sits on + −6px overlap
    /// (ATOMS · 02 group: −6 overlap, 2px surface ring, +N disc).
    pub fn avatar_stack_item(el: Div, ring: gpui_kit::Hsla, first: bool) -> Div {
        let ring_wrap = div().p(px(2.)).rounded_full().bg(ring).child(el);
        if first {
            ring_wrap
        } else {
            ring_wrap.ml(px(-6.))
        }
    }

    /// The +N overflow disc of an avatar stack: `neutral-800` fill, tint "+N" 10/600.
    pub fn avatar_overflow(n: usize, size_px: u32, mode: Mode) -> Div {
        let mut el = div()
            .size(px(size_px as f32))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(10.))
            .line_height(px(12.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(format!("+{n}"));
        if let Some(bg) = prim_hsla("neutral-800") {
            el = el.bg(bg);
        }
        if let Some(fg) = role_hsla(mode, "secondary-foreground") {
            el = el.text_color(fg);
        }
        el
    }

    /// An avatar: borderless circle (ATOMS · 02 — the ring appears only in overlap
    /// stacks and on the current profile, both composed above).
    pub fn avatar(avatar: Avatar) -> Avatar {
        avatar.border_0()
    }
}

/// Choice-control specimens — ATOMS · 02 checkbox / radio / switch / toggle / slider.
///
/// gpui-kit 0.11 draws these controls' inner track/indicator as CHILD elements fed by
/// theme seats (so the live rest/on cells are pixel-right through the theme bridge),
/// but hover/pressed/focus are live-only pseudo-states with no state flag. The gallery
/// matrix therefore paints those columns with these composed specimens — same tokens,
/// same geometry as the sheet — while the rest/on/mixed cells stay LIVE components you
/// can click. All colors resolve through roles/aliases, never literals.
pub mod choice {
    use super::{Mode, prim_hsla, radius_tier, role_hsla};
    use gpui_kit::assets::IconName;
    use gpui_kit::base::h_flex;
    use gpui_kit::prelude::FluentBuilder as _;
    use gpui_kit::{Div, FontWeight, Hsla, ParentElement as _, SharedString, Styled as _, div, px};

    /// The checked axis: off / on / mixed (checkbox + radio).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Tri {
        /// unchecked.
        Off,
        /// checked.
        On,
        /// indeterminate (some rows selected).
        Mixed,
    }

    /// The interaction axis. Rest is a live state on the real components; hover /
    /// pressed / focus / disabled are painted here (gpui has no flag for them).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Ix {
        /// live rest state.
        Rest,
        /// painted hover.
        Hover,
        /// painted pressed.
        Pressed,
        /// painted focus ring.
        Focus,
        /// painted disabled (40%).
        Disabled,
    }

    fn disabled(ix: Ix) -> bool {
        ix == Ix::Disabled
    }

    /// The 20px focus ring wrapper drawn around checkbox/radio/switch (rx6, padding
    /// 1px, 1.5px border — transparent at rest, `ring` color when focused).
    fn ring_wrap(box_el: Div, ix: Ix, mode: Mode, full: bool) -> Div {
        // hugs the control (checkbox/radio 16px box, switch 20px track) + 1px padding
        // + 1.5px border; gpui borders sit inside, so the wrapper grows instead of
        // clipping the control the way a fixed 20px box did for the switch track.
        let mut w = div()
            .p(px(1.))
            .border(px(1.5))
            .border_color(gpui_kit::transparent_white())
            .rounded(if full {
                px(radius_tier("full"))
            } else {
                px(radius_tier("md"))
            })
            .child(box_el);
        if ix == Ix::Focus
            && let Some(c) = role_hsla(mode, "ring")
        {
            w = w.border_color(c);
        }
        if disabled(ix) {
            w = w.opacity(0.4);
        }
        w
    }

    /// Checkbox box (16px rx4) at a checked × interaction cell, with the check/minus
    /// glyph in `primary-foreground` when filled.
    pub fn checkbox_box(tri: Tri, ix: Ix, mode: Mode) -> Div {
        let filled = tri != Tri::Off;
        let mut b = h_flex()
            .size(px(16.))
            .rounded(px(radius_tier("sm")))
            .border_1()
            .items_center()
            .justify_center();
        // fill + border per interaction (off: transparent → white-a4 → white-a8;
        // on: primary → foreground → ring; focus brightens the box border)
        let (bg, border) = match (filled, ix) {
            (false, Ix::Rest) | (false, Ix::Disabled) => (None, role_hsla(mode, "input")),
            (false, Ix::Hover) => (prim_hsla("white-a4"), role_hsla(mode, "input-hover")),
            (false, Ix::Pressed) => (prim_hsla("white-a8"), role_hsla(mode, "input-hover")),
            (false, Ix::Focus) => (None, role_hsla(mode, "input-focus")),
            (true, Ix::Rest) | (true, Ix::Disabled) | (true, Ix::Focus) => {
                (role_hsla(mode, "primary"), role_hsla(mode, "primary"))
            }
            (true, Ix::Hover) => (role_hsla(mode, "foreground"), role_hsla(mode, "foreground")),
            (true, Ix::Pressed) => (role_hsla(mode, "ring"), role_hsla(mode, "ring")),
        };
        if let Some(bg) = bg {
            b = b.bg(bg);
        }
        if let Some(bd) = border {
            b = b.border_color(bd);
        }
        if filled {
            let mark = if tri == Tri::Mixed {
                IconName::Minus
            } else {
                IconName::Check
            };
            let mut icon = super::atoms::icon(mark, 12.);
            if let Some(fg) = role_hsla(mode, "primary-foreground") {
                icon = icon.text_color(fg);
            }
            b = b.child(icon);
        }
        ring_wrap(b, ix, mode, false)
    }

    /// Radio circle (16px, full round) at a checked × interaction cell, with the 6px
    /// `primary-foreground` dot when on.
    pub fn radio_circle(on: bool, ix: Ix, mode: Mode) -> Div {
        let mut c = h_flex()
            .size(px(16.))
            .rounded_full()
            .border_1()
            .items_center()
            .justify_center();
        let (bg, border) = match (on, ix) {
            (false, Ix::Rest) | (false, Ix::Disabled) => (None, role_hsla(mode, "input")),
            (false, Ix::Hover) => (prim_hsla("white-a4"), role_hsla(mode, "input-hover")),
            (false, Ix::Pressed) => (prim_hsla("white-a8"), role_hsla(mode, "input-hover")),
            (false, Ix::Focus) => (None, role_hsla(mode, "input-focus")),
            (true, Ix::Rest) | (true, Ix::Disabled) | (true, Ix::Focus) => {
                (role_hsla(mode, "primary"), role_hsla(mode, "primary"))
            }
            (true, Ix::Hover) => (role_hsla(mode, "foreground"), role_hsla(mode, "foreground")),
            (true, Ix::Pressed) => (role_hsla(mode, "ring"), role_hsla(mode, "ring")),
        };
        if let Some(bg) = bg {
            c = c.bg(bg);
        }
        if let Some(bd) = border {
            c = c.border_color(bd);
        }
        if on {
            let mut dot = div().size(px(6.)).rounded_full();
            if let Some(fg) = role_hsla(mode, "primary-foreground") {
                dot = dot.bg(fg);
            }
            c = c.child(dot);
        }
        ring_wrap(c, ix, mode, true)
    }

    /// Switch track + thumb at an on × interaction cell. Medium = 36x20 (thumb 16),
    /// small = 28x16 (thumb 12).
    pub fn switch_track(on: bool, small: bool, ix: Ix, mode: Mode) -> Div {
        let (tw, th, thumb) = if small {
            (28., 16., 12.)
        } else {
            (36., 20., 16.)
        };
        let (bg, thumb_bg) = match (on, ix) {
            (false, Ix::Rest) | (false, Ix::Disabled) => (
                role_hsla(mode, "input"),
                role_hsla(mode, "secondary-foreground"),
            ),
            (false, Ix::Hover) => (role_hsla(mode, "border-strong"), role_hsla(mode, "ring")),
            (false, Ix::Pressed) => (role_hsla(mode, "input-hover"), role_hsla(mode, "ring")),
            (false, Ix::Focus) => (
                role_hsla(mode, "input"),
                role_hsla(mode, "secondary-foreground"),
            ),
            (true, Ix::Rest) | (true, Ix::Disabled) | (true, Ix::Focus) => (
                role_hsla(mode, "primary"),
                role_hsla(mode, "primary-foreground"),
            ),
            (true, Ix::Hover) => (
                role_hsla(mode, "foreground"),
                role_hsla(mode, "primary-foreground"),
            ),
            (true, Ix::Pressed) => (
                role_hsla(mode, "ring"),
                role_hsla(mode, "primary-foreground"),
            ),
        };
        let mut t = h_flex()
            .w(px(tw))
            .h(px(th))
            .p(px(2.))
            .rounded_full()
            .items_center()
            .map(|this| {
                if on {
                    this.justify_end()
                } else {
                    this.justify_start()
                }
            });
        if let Some(bg) = bg {
            t = t.bg(bg);
        }
        let mut knob = div().size(px(thumb)).rounded_full().shadow_sm().border_0();
        if let Some(bg) = thumb_bg {
            knob = knob.bg(bg);
        }
        t = t.child(knob);
        ring_wrap(t, ix, mode, true)
    }

    /// Toggle chip at an on × interaction cell (ghost or outline), at a control size:
    /// xs 20/6 · sm 24/8 · md 32/10 · lg 40/14, gap 6, 14px glyph, 12/500 label, rx6.
    /// Icon is duotone whenever the state is "engaged" (hover/pressed/on), regular at
    /// rest-off — the sheet's regular ↔ duotone swap.
    #[allow(clippy::too_many_arguments)]
    pub fn toggle_chip(
        on: bool,
        outline: bool,
        ix: Ix,
        icon: IconName,
        label: impl Into<SharedString>,
        h_px: f32,
        height_px: f32,
        mode: Mode,
    ) -> Div {
        let engaged = on || matches!(ix, Ix::Hover | Ix::Pressed);
        let (bg, fg) = match (on, ix) {
            (false, Ix::Rest) | (false, Ix::Disabled) => {
                (None, role_hsla(mode, "secondary-foreground"))
            }
            (false, Ix::Hover) => (prim_hsla("white-a4"), role_hsla(mode, "foreground")),
            (false, Ix::Pressed) => (prim_hsla("white-a8"), role_hsla(mode, "foreground")),
            (false, Ix::Focus) => (None, role_hsla(mode, "secondary-foreground")),
            (true, Ix::Rest) | (true, Ix::Disabled) => (
                role_hsla(mode, "control-selected"),
                role_hsla(mode, "foreground"),
            ),
            (true, Ix::Hover) => (prim_hsla("white-a16"), role_hsla(mode, "foreground")),
            (true, Ix::Pressed) => (prim_hsla("white-a8"), role_hsla(mode, "foreground")),
            (true, Ix::Focus) => (
                role_hsla(mode, "control-selected"),
                role_hsla(mode, "foreground"),
            ),
        };
        let glyph = if engaged {
            super::atoms::duotone_icon(icon, 14.)
        } else {
            super::atoms::icon(icon, 14.)
        };
        let mut el = h_flex()
            .h(px(height_px))
            .px(px(h_px))
            .gap(px(6.))
            .items_center()
            .rounded(px(radius_tier("md")))
            .border(px(2.))
            .border_color(gpui_kit::transparent_white())
            .child(glyph)
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(label.into()),
            );
        if outline {
            el = el.border_1().map(|this| {
                this.border_color(
                    role_hsla(mode, "border-strong").unwrap_or(gpui_kit::transparent_white()),
                )
            });
        }
        if ix == Ix::Focus
            && let Some(c) = role_hsla(mode, "ring")
        {
            el = el.border_2().border_color(c);
        }
        if let Some(bg) = bg {
            el = el.bg(bg);
        }
        if let Some(fg) = fg {
            el = el.text_color(fg);
        }
        if disabled(ix) {
            el = el.opacity(0.4);
        }
        el
    }

    /// Slider specimen: 4px track (white-a8) + `primary` fill + 14px thumb (2px
    /// `primary-foreground` ring inside) + 20px halo (transparent / white-a12 hover /
    /// white-a24 pressed / `ring` on focus). gpui's stock Slider keeps fixed internal
    /// geometry, so the sheet-exact track is composed here; the live draggable Slider
    /// rides the same theme seats.
    pub fn slider_track(pct: f32, ix: Ix, width: f32, mode: Mode) -> Div {
        slider_track_bubble(pct, ix, width, None, mode)
    }

    /// [`slider_track`] with a value bubble floating above the thumb (pressed state).
    pub fn slider_track_bubble(
        pct: f32,
        ix: Ix,
        width: f32,
        bubble: Option<SharedString>,
        mode: Mode,
    ) -> Div {
        let pct = pct.clamp(0., 100.);
        let fill_w = width * pct / 100.;
        let halo = match ix {
            Ix::Hover => prim_hsla("white-a12"),
            Ix::Pressed => prim_hsla("white-a24"),
            Ix::Focus => role_hsla(mode, "ring"),
            _ => Some(gpui_kit::transparent_white()),
        };
        let mut track = div()
            .w(px(width))
            .h(px(4.))
            .rounded_full()
            .bg(prim_hsla("white-a8").unwrap_or(gpui_kit::transparent_white()));
        let mut fill = div()
            .w(px(fill_w))
            .h(px(4.))
            .rounded_full()
            .bg(role_hsla(mode, "primary").unwrap_or(gpui_kit::transparent_white()));
        if disabled(ix) {
            track = track.opacity(0.4);
            fill = fill.opacity(0.4);
        }
        let mut halo_el = div()
            .absolute()
            .size(px(20.))
            .rounded_full()
            .left(px(fill_w - 10.))
            .top(px(-5.));
        if let Some(h) = halo {
            halo_el = halo_el.bg(h);
        }
        let mut thumb = div()
            .absolute()
            .size(px(14.))
            .rounded_full()
            .border_2()
            .left(px(fill_w - 7.))
            .top(px(3.))
            .bg(role_hsla(mode, "primary").unwrap_or(gpui_kit::transparent_white()))
            .border_color(
                role_hsla(mode, "primary-foreground").unwrap_or(gpui_kit::transparent_white()),
            );
        if disabled(ix) {
            thumb = thumb.opacity(0.4);
        }
        let mut root = div()
            .relative()
            .w(px(width))
            .h(px(20.))
            .child(track)
            .child(fill)
            .child(halo_el)
            .child(thumb);
        if let Some(text) = bubble {
            // Bubble floats above the thumb, centered on the thumb's midpoint. A
            // zero-width flex container at left=fill_w with justify_center centers the
            // bubble on that x without needing to know the bubble's measured width.
            root = root.child(
                div()
                    .absolute()
                    .left(px(fill_w))
                    .top(px(-26.))
                    .w(px(0.))
                    .h(px(20.))
                    .flex()
                    .justify_center()
                    .child(slider_bubble(text, mode)),
            );
        }
        root
    }

    /// Range slider specimen: fill BETWEEN the two thumbs (p1..p2).
    pub fn slider_range(p1: f32, p2: f32, width: f32, mode: Mode) -> Div {
        let (a, b) = (
            p1.clamp(0., 100.).min(p2.clamp(0., 100.)),
            p2.clamp(0., 100.).max(p1.clamp(0., 100.)),
        );
        let fill = div()
            .absolute()
            .h(px(4.))
            .rounded_full()
            .left(px(width * a / 100.))
            .w(px(width * (b - a) / 100.))
            .top(px(8.))
            .bg(role_hsla(mode, "primary").unwrap_or(gpui_kit::transparent_white()));
        let thumb = |p: f32| {
            div()
                .absolute()
                .size(px(14.))
                .rounded_full()
                .border_2()
                .left(px(width * p / 100. - 7.))
                .top(px(3.))
                .bg(role_hsla(mode, "primary").unwrap_or(gpui_kit::transparent_white()))
                .border_color(
                    role_hsla(mode, "primary-foreground").unwrap_or(gpui_kit::transparent_white()),
                )
        };
        div()
            .relative()
            .w(px(width))
            .h(px(20.))
            .child(
                div()
                    .absolute()
                    .w(px(width))
                    .h(px(4.))
                    .rounded_full()
                    .top(px(8.))
                    .bg(prim_hsla("white-a8").unwrap_or(gpui_kit::transparent_white())),
            )
            .child(fill)
            .child(thumb(a))
            .child(thumb(b))
    }

    /// The pressed-state value bubble above the thumb: `primary` pill, rx4, padding
    /// 2px 6px, 11/500 ink (ATOMS · 02 slider pressed: "620 RON").
    pub fn slider_bubble(text: impl Into<SharedString>, mode: Mode) -> Div {
        let mut el = h_flex()
            .p(px(2.))
            .px(px(6.))
            .rounded(px(radius_tier("sm")))
            .text_size(px(11.))
            .line_height(px(15.))
            .font_weight(FontWeight::MEDIUM)
            .child(text.into());
        if let Some(bg) = role_hsla(mode, "primary") {
            el = el.bg(bg);
        }
        if let Some(fg) = role_hsla(mode, "primary-foreground") {
            el = el.text_color(fg);
        }
        el
    }

    /// One mark tick under the slider track + its caption (11/400 `icon-muted`).
    pub fn slider_mark(pct: f32, width: f32, label: impl Into<SharedString>, mode: Mode) -> Div {
        let mut tick = div()
            .absolute()
            .w(px(1.))
            .h(px(4.))
            .left(px(width * pct / 100.))
            .bg(role_hsla(mode, "icon-muted").unwrap_or(gpui_kit::transparent_white()));
        let mut cap = div()
            .absolute()
            .left(px(width * pct / 100.))
            .top(px(8.))
            .text_size(px(11.))
            .line_height(px(15.))
            .child(label.into());
        if let Some(fg) = role_hsla(mode, "icon-muted") {
            tick = tick.bg(fg);
            cap = cap.text_color(fg);
        }
        div()
            .relative()
            .w(px(width))
            .h(px(20.))
            .child(tick)
            .child(cap)
    }

    /// Re-export for call sites that build the live controls' labels.
    pub fn label_text(text: impl Into<SharedString>) -> SharedString {
        text.into()
    }

    /// Type alias so gallery tables can name the specimen kinds.
    pub type HslaOpt = Option<Hsla>;
}

/// Control presets — the ATOMS · 01 control-size scale (20/24/32/40px with radii
/// sm/md/md/lg = 4/6/6/8). The old sheets drew the scale on buttons only; gpui-kit 0.11
/// carries more size-aware components, and every one of them is sized through this
/// scale, so the whole gallery stays on the same ladder.
pub mod controls {
    use super::*;

    /// The design language's control-size scale (ATOMS · 01 size chips:
    /// xsmall 20 · small 24 · medium 32 · large 40, radius 6/6/6/8).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ControlSize {
        /// 20px — chips and micro controls (radius `md` tier = 6).
        XSmall,
        /// 24px — dense controls (radius `md` tier = 6).
        Small,
        /// 32px — the standard control (radius `md` tier = 6).
        Medium,
        /// 40px — large controls (radius `lg` tier = 8).
        Large,
    }

    impl ControlSize {
        /// Height on the scale: the `space-*` ladder (space-5/6/8/10 = 20/24/32/40).
        pub fn height(self) -> u32 {
            match self {
                Self::XSmall => space("space-5"),
                Self::Small => space("space-6"),
                Self::Medium => space("space-8"),
                Self::Large => space("space-10"),
            }
        }

        /// Corner radius on the scale as drawn: 20px → **4** (tier sm), 24/32 → 6 (md),
        /// 40 → 8 (lg).
        pub fn radius(self) -> u32 {
            match self {
                Self::XSmall => radius_tier("sm") as u32,
                Self::Small => radius_tier("md") as u32,
                Self::Medium => radius_tier("md") as u32,
                Self::Large => radius_tier("lg") as u32,
            }
        }

        /// Horizontal padding on the scale as drawn: 20 → 6, 24 → 8, 32 → 12, 40 → 16
        /// (TOKENS §04 `inset-control` ladder; gpui-kit's own h_*/px_* steps differ).
        pub fn h_padding(self) -> u32 {
            match self {
                Self::XSmall => space("space-1.5"),
                Self::Small => space("space-2"),
                Self::Medium => space("space-3"),
                Self::Large => space("space-4"),
            }
        }

        /// Glyph size INSIDE a control of this size — ATOMS · 01 sizes row as drawn:
        /// 20px control → 12px icon, 24 → 14, 32 → 14, 40 → 16 (icons stay ~44% of the
        /// control height; gpui-kit's default icon size is too large for the small
        /// controls, which is why the preset pins it per size).
        pub fn icon_px(self) -> u32 {
            match self {
                Self::XSmall => 12,
                Self::Small => 14,
                Self::Medium => 14,
                Self::Large => 16,
            }
        }
    }

    /// Size a [`Button`] on the design control-size scale — height, radius AND horizontal
    /// padding stay on the sheet's ladder even where gpui's own sizes differ (its
    /// `large` is 32px, the sheet's large is 40px). The label weight pins to 500 —
    /// every button label in ATOMS · 01 is drawn medium (gpui leaves it at the theme
    /// default 400; the component never sets a weight on the label, so the instance
    /// refinement cascades into it).
    pub fn button_size(button: Button, size: ControlSize) -> Button {
        button
            .with_size(Size::Size(px(size.height() as f32)))
            // gpui-kit maps Size::Size on a LABELED button to horizontal padding only
            // (px(size*0.2)) and never sets the height — pin it explicitly so the
            // control keeps its 20/24/32/40 ladder instead of collapsing to text height
            .h(px(size.height() as f32))
            .rounded(ButtonRounded::Size(px(size.radius() as f32)))
            .px(px(size.h_padding() as f32))
            .font_weight(gpui_kit::FontWeight::MEDIUM)
    }

    /// Size an icon-ONLY [`Button`] on the scale — a square (height = width), no
    /// horizontal padding (the padded branch of gpui's layout only applies to labeled
    /// buttons, and an instance `px()` would break the square).
    pub fn icon_button_size(button: Button, size: ControlSize) -> Button {
        button
            .with_size(Size::Size(px(size.height() as f32)))
            .rounded(ButtonRounded::Size(px(size.radius() as f32)))
    }

    /// Size a [`ColorPicker`] on the design control-size scale — the only size-aware
    /// picker; its swatch rides the same height ladder as buttons (ATOMS · 01/03).
    pub fn color_picker_size(picker: ColorPicker, size: ControlSize) -> ColorPicker {
        picker.with_size(Size::Size(px(size.height() as f32)))
    }

    /// A checkbox on the design geometry — 16x16 rx4, rest fill white @ 8% with NO
    /// border (ATOMS · 02: rest = `sidebar-selected` overlay, not gpui's dark input
    /// well). Checked fill/ink ride the `primary` seat (inverted neutral).
    pub fn checkbox(
        c: gpui_kit::component::checkbox::Checkbox,
        mode: Mode,
    ) -> gpui_kit::component::checkbox::Checkbox {
        let mut c = c.rounded(px(radius_tier("sm"))).border_0();
        if let Some(bg) = role_hsla(mode, "sidebar-selected") {
            c = c.bg(bg);
        }
        c
    }

    /// A radio on the design geometry — full-round outer, rest fill light (ATOMS · 02:
    /// rest outer = the inverted neutral, no dark well), checked = `primary` + dot.
    pub fn radio(
        r: gpui_kit::component::radio::Radio,
        mode: Mode,
    ) -> gpui_kit::component::radio::Radio {
        let mut r = r.rounded(px(radius_tier("full"))).border_0();
        if let Some(bg) = role_hsla(mode, "primary") {
            r = r.bg(bg);
        }
        r
    }

    /// A kbd keycap on the design surface: `control` fill (#1c1c1c) rx4 with a 2px
    /// `border-strong` base (the "raised key with a 2px base", ATOMS · 01 kbd:
    /// 44x16.5, padding 0 5px, ink `secondary-foreground` 11px/500).
    pub fn kbd(k: gpui_kit::component::kbd::Kbd, mode: Mode) -> gpui_kit::component::kbd::Kbd {
        let mut k = k.rounded(px(radius_tier("sm"))).border_b(px(2.)).px(px(5.));
        if let Some(bg) = role_hsla(mode, "control") {
            k = k.bg(bg);
        }
        if let Some(border) = role_hsla(mode, "border-strong") {
            k = k.border_color(border);
        }
        if let Some(fg) = role_hsla(mode, "secondary-foreground") {
            k = k.text_color(fg);
        }
        k
    }

    /// A toggle button on the design surface: rest fill `ghost-hover` (white @ 4%),
    /// borderless, the 120x32 drawn size (ATOMS · 02 toggle: rest → active overlay
    /// ladder; the checked overlay rides the component's own active state).
    pub fn toggle(
        t: gpui_kit::component::button::Toggle,
        mode: Mode,
    ) -> gpui_kit::component::button::Toggle {
        let mut t = t.border_0().min_w(px(120.));
        if let Some(bg) = role_hsla(mode, "ghost-hover") {
            t = t.bg(bg);
        }
        t
    }

    /// A segmented toggle group on the design surface: `control` rx6 container, segments
    /// `control-selected` (white @ 12%) rx4 when selected (MOLECULES · 02 toggle group:
    /// 198x28 rx6 container, 47/55x24 rx4 segments).
    pub fn toggle_group<G>(g: G, mode: Mode) -> G
    where
        G: gpui_kit::Styled,
    {
        let mut g = g.border_0().rounded(px(radius_tier("md")));
        if let Some(bg) = role_hsla(mode, "control") {
            g = g.bg(bg);
        }
        g
    }
}

/// Composed atoms — the design-language atoms the gallery showcases AND the app reuses
/// (doc 04 §4.9 / ADR-0017: one component set, the gallery is only a harness). These are
/// the ATOMS · 01 building blocks where gpui-kit's stock component cannot express the
/// drawn design (suffix at caption size, highlight well, external-link glyph, duotone
/// weight) — each is a thin composition over the styled component + tokens, so there is
/// exactly one place to change the look.
pub mod atoms {
    use super::controls::ControlSize;
    use super::{Mode, role_hsla};
    use bookkeepy_primary_data::theme::style;
    use gpui_kit::assets::IconName;
    use gpui_kit::base::h_flex;
    use gpui_kit::component::button::{Button, ButtonRounded, ButtonVariant, ButtonVariants as _};
    use gpui_kit::component::label::Label;
    use gpui_kit::component::spinner::Spinner;
    use gpui_kit::component::{Icon, Sizable as _, Size};
    use gpui_kit::prelude::FluentBuilder as _;
    use gpui_kit::{
        Div, ElementId, FontWeight, HighlightStyle, ParentElement as _, SharedString, Styled as _,
        StyledText, div, px,
    };
    use std::ops::Range;

    /// The sheet's `Label` master: `label` type style (12px) drawn at weight **500**
    /// with the foreground ink (ATOMS · 01 label masters are semibold; the `label`
    /// style itself stays 400 for nav/buttons).
    pub fn label(text: impl Into<SharedString>) -> Label {
        let mut l = Label::new(text);
        if let Some(s) = style("label") {
            // (gpui 0.3 has no letter-spacing axis; the style's tracking is sub-pixel
            // at these sizes — size/line-height/weight carry the look.)
            l = l
                .text_size(px(s.size_px as f32))
                .line_height(px(s.line_height_px as f32))
                .font_weight(FontWeight::MEDIUM);
        }
        l
    }

    /// A caption-styled suffix span (the `caption` type style, muted ink) — the
    /// "quiet suffix" of ATOMS · 01 (`(optional)`, `*`, `RON`, `locked`, `required`).
    pub fn suffix(text: impl Into<SharedString>, mode: Mode) -> Div {
        let mut el = div().child(text.into());
        if let Some(s) = style("caption") {
            el = el
                .text_size(px(s.size_px as f32))
                .line_height(px(s.line_height_px as f32))
                .font_weight(FontWeight::from(s.weight as f32));
        }
        if let Some(c) = role_hsla(mode, "muted-foreground") {
            el = el.text_color(c);
        }
        el
    }

    /// A field label: label text + quiet caption suffix on the same baseline, 4px apart
    /// (ATOMS · 01: "secondary suffix: caption, muted" — NOT the same level as the label).
    pub fn field_label(
        text: impl Into<SharedString>,
        suffix_text: impl Into<SharedString>,
        mode: Mode,
    ) -> Div {
        h_flex()
            .gap(px(super::space("space-1") as f32))
            .items_baseline()
            .child(label(text))
            .child(suffix(suffix_text, mode))
    }

    /// A masked amount label: bullets for the amount + the currency suffix beside it
    /// (ATOMS · 01 masked: "•••• with the eye toggle beside it" — the toggle is the
    /// caller's button; this is the static part).
    pub fn masked_label(
        text: impl Into<SharedString>,
        suffix_text: impl Into<SharedString>,
        mode: Mode,
    ) -> Div {
        h_flex()
            .gap(px(super::space("space-1") as f32))
            .items_baseline()
            .child(label(text).masked(true))
            .child(suffix(suffix_text, mode))
    }

    /// A search-highlighted label: the match sits on the `accent-subtle` well with the
    /// `accent-foreground` ink (ATOMS · 01 highlight: match on accent-subtle, rest
    /// muted) — gpui-kit's `Label::highlights` hardcodes `theme.blue`, so the composed
    /// atom renders the StyledText itself.
    pub fn highlighted(
        text: impl Into<SharedString>,
        matched: &str,
        prefix: bool,
        mode: Mode,
    ) -> Div {
        let text: SharedString = text.into();
        let lower = text.to_lowercase();
        let needle = matched.to_lowercase();
        let mut ranges: Vec<Range<usize>> = Vec::new();
        if !needle.is_empty() {
            if prefix {
                if lower.starts_with(&needle) {
                    ranges.push(0..matched.len());
                }
            } else {
                let mut start = 0;
                while let Some(pos) = lower[start..].find(&needle) {
                    let s = start + pos;
                    let e = s + matched.len();
                    if e <= text.len() {
                        ranges.push(s..e);
                    }
                    start = e.max(s + 1);
                    while start < text.len() && !text.is_char_boundary(start) {
                        start += 1;
                    }
                }
            }
        }
        let hl: Vec<(Range<usize>, HighlightStyle)> = ranges
            .into_iter()
            .map(|r| {
                (
                    r,
                    HighlightStyle {
                        color: role_hsla(mode, "accent-foreground"),
                        background_color: role_hsla(mode, "accent-subtle"),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let mut el = div().child(StyledText::new(&text).with_highlights(hl));
        if let Some(s) = style("label") {
            el = el
                .text_size(px(s.size_px as f32))
                .line_height(px(s.line_height_px as f32));
        }
        if let Some(c) = role_hsla(mode, "muted-foreground") {
            el = el.text_color(c);
        }
        el
    }

    /// An external link: `base::Link` behaviour via `Button::link()`, underlined, with
    /// the 12px arrow-up-right glyph after the text (ATOMS · 01 link: "links are only
    /// for leaving the app"). Ink rides the theme's link seats (rest secondary, hover
    /// foreground, pressed muted, disabled no-underline).
    pub fn external_link(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Button {
        Button::new(id).link().underline().child(
            h_flex()
                .gap_1()
                .items_center()
                .child(text.into())
                .child(Icon::new(IconName::ArrowUpRight).with_size(Size::Size(px(12.)))),
        )
    }

    /// A Phosphor regular icon at an exact size (the size ladder xs 12 … xl 24,
    /// ATOMS · 01 — icons never sit on a plate).
    pub fn icon(name: IconName, size_px: f32) -> Icon {
        Icon::new(name).with_size(Size::Size(px(size_px)))
    }

    /// A button glyph sized for its control (ATOMS · 01: icon ≈ 44% of control height —
    /// 20→12, 24→14, 32→14, 40→16). Pass to `Button::icon(...)`.
    pub fn button_icon(name: IconName, size: ControlSize) -> Icon {
        icon(name, size.icon_px() as f32)
    }

    /// The leading glyph of a [`button`]: a Phosphor icon, or the loading spinner.
    #[derive(Debug, Clone, Copy)]
    pub enum ButtonGlyph {
        /// A Phosphor glyph at the control's icon size.
        Icon(IconName),
        /// The rotating spinner (the `loading` state's glyph).
        Spinner,
    }

    /// Label text metrics per control size, as drawn in ATOMS · 01's sizes row:
    /// 20px → 11/15 (caption), 24 & 32 → 12/14 (label), 40 → 14/22 (body-lg); all 500.
    fn label_metrics(size: ControlSize) -> (f32, f32) {
        match size {
            ControlSize::XSmall => (11., 15.),
            ControlSize::Small | ControlSize::Medium => (12., 14.),
            ControlSize::Large => (14., 22.),
        }
    }

    /// A design-language button with EXACT sheet metrics: height/radius/h-padding on
    /// the control scale, 6px icon↔label gap, glyph at the control's icon size, label
    /// at the control's type metrics in weight 500.
    ///
    /// The icon+label are rendered as the content child (not `.icon()`/`.label()`)
    /// because gpui-kit 0.11 forces `.icon()` to `0.75 × control height` when the
    /// button is sized via `Size::Size` — too large vs the drawn 12/14/14/16.
    pub fn button(
        id: impl Into<ElementId>,
        variant: ButtonVariant,
        glyph: Option<ButtonGlyph>,
        label: impl Into<SharedString>,
        size: ControlSize,
    ) -> Button {
        let label = label.into();
        let (lsize, llh) = label_metrics(size);
        let content = h_flex()
            .gap(px(super::space("space-1.5") as f32))
            .items_center()
            .when_some(glyph, |c, g| match g {
                ButtonGlyph::Icon(name) => c.child(icon(name, size.icon_px() as f32)),
                ButtonGlyph::Spinner => {
                    c.child(Spinner::new().with_size(Size::Size(px(size.icon_px() as f32))))
                }
            })
            .child(
                div()
                    .text_size(px(lsize))
                    .line_height(px(llh))
                    .font_weight(FontWeight::MEDIUM)
                    .whitespace_nowrap()
                    .child(label.clone()),
            );
        Button::new(id)
            .with_variant(variant)
            .with_size(Size::Size(px(size.height() as f32)))
            // labeled Size::Size buttons get no fixed height from gpui-kit — pin it
            .h(px(size.height() as f32))
            .rounded(ButtonRounded::Size(px(size.radius() as f32)))
            .px(px(size.h_padding() as f32))
            .accessibility_label(label)
            .child(content)
    }

    /// An icon-ONLY button on the scale: a square (height = width), glyph centered at
    /// the control's icon size (ATOMS · 01 icon-button: 16px glyph in the 32px square).
    pub fn icon_button(id: impl Into<ElementId>, glyph: IconName, size: ControlSize) -> Button {
        Button::new(id)
            .ghost()
            .with_size(Size::Size(px(size.height() as f32)))
            .h(px(size.height() as f32))
            .rounded(ButtonRounded::Size(px(size.radius() as f32)))
            .w(px(size.height() as f32))
            .px_0()
            .child(icon(glyph, size.icon_px() as f32))
    }

    /// The same glyph in the **duotone** weight (interaction state) — served by
    /// [`crate::assets::BookkeepyAssets`] under `icons/duotone/…` and swapped in via
    /// `Icon::path`, gpui-kit 0.11's icon-style seam. Falls back to regular when no
    /// duotone master is embedded for the glyph.
    pub fn duotone_icon(name: IconName, size_px: f32) -> Icon {
        let icon = Icon::new(name);
        let icon = match crate::assets::duotone_path(&name.path()) {
            Some(duo) => icon.path(duo),
            None => icon,
        };
        icon.with_size(Size::Size(px(size_px)))
    }

    /// A semantic icon role (ATOMS · 01 color roles: `icon`, `icon-muted`, …,
    /// `money-in`) — glyph in the role's color, no plate.
    pub fn role_icon(name: IconName, size_px: f32, role: &str, mode: Mode) -> Icon {
        let mut ic = icon(name, size_px);
        if let Some(c) = role_hsla(mode, role) {
            ic = ic.text_color(c);
        }
        ic
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
        solid_fill(
            button,
            ButtonVariant::Danger,
            role_hsla(mode, "danger"),
            role_hsla(mode, "on-solid"),
        )
    }

    /// Paint a button as a FORCED interaction specimen (the gallery matrix's hover /
    /// pressed columns, which are otherwise live-only and read as "rest" in a static
    /// view). bg/fg come from the same roles the theme seats map to, so the forced
    /// cell is pixel-identical to the real state.
    pub fn forced_interaction(
        button: Button,
        variant: ButtonVariant,
        pressed: bool,
        mode: Mode,
    ) -> Button {
        let (bg, fg) = match variant {
            ButtonVariant::Primary => {
                if pressed {
                    ("primary-active", "primary-foreground")
                } else {
                    ("primary-hover", "primary-foreground")
                }
            }
            ButtonVariant::Danger => ("danger", "on-solid"),
            // outline is a ghost plate with a border (ATOMS · 01: hover/pressed are
            // white-alpha washes, not the solid control fill)
            ButtonVariant::Default => {
                if pressed {
                    ("ghost-active", "icon-hover")
                } else {
                    ("ghost-hover", "icon-hover")
                }
            }
            ButtonVariant::Ghost => {
                if pressed {
                    ("ghost-active", "icon-hover")
                } else {
                    ("ghost-hover", "icon-hover")
                }
            }
            _ => {
                if pressed {
                    ("control-active", "icon")
                } else {
                    ("control-hover", "icon-hover")
                }
            }
        };
        solid_fill(button, variant, role_hsla(mode, bg), role_hsla(mode, fg))
    }
}

/// Feedback presets — alert banners are the design's hue WELL fills (900) with NO border
/// and the card radius (MOLECULES · 02 alert: rx16 in the semantic well, light glyph).
/// gpui's alert bg is a 4% hue wash; this pins the drawn well fill per tone.
pub mod feedback {
    use super::{Mode, prim_hsla, radius_role, role_hsla};
    use gpui_kit::component::alert::Alert;
    use gpui_kit::{Styled as _, px};

    /// The alert tone, selecting the matching well token (MOLECULES · 02 alert wells).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AlertTone {
        /// `accent-subtle` well + `accent-foreground` glyph.
        Accent,
        /// `cyan-900` well.
        Info,
        /// `success-subtle`.
        Success,
        /// `amber-900` well.
        Warning,
        /// `danger-subtle`.
        Danger,
    }

    /// An alert banner on the design surface: well fill, borderless, radius-card, with
    /// the title in `icon` ink and the glyph in the light tint (MOLECULES · 02 alert:
    /// title #d1d1d1/#ededed, light icon variant — not the raw hue).
    pub fn alert(alert: Alert, tone: AlertTone, mode: Mode) -> Alert {
        let well = match tone {
            AlertTone::Accent => role_hsla(mode, "accent-subtle"),
            AlertTone::Info => prim_hsla("cyan-900"),
            AlertTone::Success => role_hsla(mode, "success-subtle"),
            AlertTone::Warning => prim_hsla("amber-900"),
            AlertTone::Danger => role_hsla(mode, "danger-subtle"),
        };
        let mut alert = alert
            .border_0()
            .rounded(px(radius_role("card")))
            .text_color(role_hsla(mode, "icon").unwrap_or_default());
        if let Some(bg) = well {
            alert = alert.bg(bg);
        }
        alert
    }

    /// A toast notification on the design surface: borderless `surface-raised` rx20 card
    /// (ORGANISMS · 01 notification: 380x238 rx20 #1c1c1c, no hard border — gpui draws
    /// a 1px `border` outline by default).
    pub fn notification<N>(n: N, mode: Mode) -> N
    where
        N: gpui_kit::Styled,
    {
        let mut n = n.border_0().rounded(px(radius_role("dialog")));
        if let Some(bg) = role_hsla(mode, "surface-raised") {
            n = n.bg(bg);
        }
        n
    }
}

pub use actions::solid_danger;
pub use controls::{ControlSize, button_size, color_picker_size, icon_button_size};

/// Field specimens — ATOMS · 03 input / number input / textarea. The LIVE text input
/// (`Input::new(&state)`) renders rest/focus through the theme seats; hover, loading,
/// disabled, invalid and readonly have no live flag in gpui 0.11, so the gallery paints
/// those columns with these composed cells — same tokens, same geometry as the sheet.
pub mod fields {
    use super::{Mode, prim_hsla, radius_tier, role_hsla};
    use gpui_kit::assets::IconName;
    use gpui_kit::base::{h_flex, v_flex};
    use gpui_kit::component::Sizable as _;
    use gpui_kit::prelude::FluentBuilder as _;
    use gpui_kit::{Div, FontWeight, Hsla, ParentElement as _, SharedString, Styled as _, div, px};

    /// Field interaction state (ATOMS · 03 input matrix columns).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FieldIx {
        /// rest.
        Rest,
        /// hover (border `input-hover`).
        Hover,
        /// focus (border `input-focus` + 2px white-a16 ring on the wrapper).
        Focus,
        /// loading (spinner in the suffix slot).
        Loading,
        /// disabled (40%).
        Disabled,
        /// invalid (danger border, danger label + help).
        Invalid,
        /// readonly (no fill, subtle border, lock glyph).
        Readonly,
    }

    /// The 32px (or sized) field: sunken fill `neutral-950`, rx6, hairline outline,
    /// padding 0 10px, gap 8 — inside the rx8 2px ring wrapper (ring = white-a16 on
    /// focus, transparent otherwise; gpui borders sit inside, so the wrapper is
    /// padding 2px + border 2px around the field).
    pub fn input_field(inner: Div, height: f32, ix: FieldIx, mode: Mode) -> Div {
        let (bg, border) = match ix {
            FieldIx::Rest | FieldIx::Loading | FieldIx::Disabled => {
                (prim_hsla("neutral-950"), role_hsla(mode, "input"))
            }
            FieldIx::Hover => (prim_hsla("neutral-950"), role_hsla(mode, "input-hover")),
            FieldIx::Focus => (prim_hsla("neutral-950"), role_hsla(mode, "input-focus")),
            FieldIx::Invalid => (prim_hsla("neutral-950"), role_hsla(mode, "danger")),
            FieldIx::Readonly => (
                Some(gpui_kit::transparent_white()),
                prim_hsla("neutral-825"),
            ),
        };
        let mut field = h_flex()
            .h(px(height))
            .px(px(10.))
            .gap(px(8.))
            .items_center()
            .rounded(px(radius_tier("md")))
            .border_1()
            .child(inner);
        if let Some(bg) = bg {
            field = field.bg(bg);
        }
        if let Some(b) = border {
            field = field.border_color(b);
        }
        let mut ring = div()
            .p(px(2.))
            .rounded(px(radius_tier("lg")))
            .border_2()
            .border_color(gpui_kit::transparent_white())
            .child(field);
        if ix == FieldIx::Focus
            && let Some(c) = prim_hsla("white-a16")
        {
            ring = ring.border_color(c);
        }
        if ix == FieldIx::Disabled {
            ring = ring.opacity(0.4);
        }
        ring
    }

    /// One full input cell: label (12/500), field, help text (11/400) — the sheet's
    /// column. `prefix`/`suffix` are glyph slots inside the field; `text` is the value
    /// or placeholder ink (`placeholder` = neutral-500).
    #[allow(clippy::too_many_arguments)]
    pub fn input_cell(
        label: impl Into<SharedString>,
        ix: FieldIx,
        prefix: Option<IconName>,
        text: impl Into<SharedString>,
        placeholder: bool,
        suffix: Option<IconName>,
        help: Option<SharedString>,
        height: f32,
        mode: Mode,
    ) -> Div {
        let label_color = match ix {
            FieldIx::Invalid => role_hsla(mode, "danger-foreground"),
            FieldIx::Disabled => prim_hsla("neutral-600"),
            _ => role_hsla(mode, "foreground"),
        };
        let mut row = h_flex()
            .items_center()
            .gap(px(8.))
            .child(super::atoms::icon(prefix.unwrap_or(IconName::Search), 14.));
        let text_color = if placeholder {
            prim_hsla("neutral-500")
        } else {
            role_hsla(mode, "foreground")
        };
        row = row.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .when_some(text_color, |d, c| d.text_color(c))
                .child(text.into()),
        );
        if ix == FieldIx::Loading {
            row = row.child(
                gpui_kit::component::spinner::Spinner::new()
                    .with_size(gpui_kit::component::Size::Size(px(14.)))
                    .color(role_hsla(mode, "muted-foreground").unwrap_or_default()),
            );
        } else if let Some(s) = suffix {
            let mut ic = super::atoms::icon(s, 14.);
            let fg = match ix {
                FieldIx::Invalid => role_hsla(mode, "danger-foreground"),
                FieldIx::Disabled => prim_hsla("neutral-600"),
                _ => role_hsla(mode, "icon-muted"),
            };
            if let Some(fg) = fg {
                ic = ic.text_color(fg);
            }
            row = row.child(ic);
        }
        let field = input_field(row, height, ix, mode);
        let mut col = v_flex().gap(px(6.)).child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .font_weight(FontWeight::MEDIUM)
                .when_some(label_color, |d, c| d.text_color(c))
                .child(label.into()),
        );
        col = col.child(field);
        if let Some(help) = help {
            let help_color = match ix {
                FieldIx::Invalid => role_hsla(mode, "danger-foreground"),
                _ => prim_hsla("neutral-500"),
            };
            col = col.child(
                div()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .when_some(help_color, |d, c| d.text_color(c))
                    .child(help),
            );
        }
        col
    }

    /// Textarea specimen: same field surface, h96, padding 8px 10px, text 12/400 lh18,
    /// counter `n / max` bottom-right (warning tone near max).
    pub fn textarea_cell(
        label: impl Into<SharedString>,
        ix: FieldIx,
        text: impl Into<SharedString>,
        counter: Option<SharedString>,
        mode: Mode,
    ) -> Div {
        let (bg, border) = match ix {
            FieldIx::Readonly => (
                Some(gpui_kit::transparent_white()),
                prim_hsla("neutral-825"),
            ),
            FieldIx::Hover => (prim_hsla("neutral-950"), role_hsla(mode, "input-hover")),
            FieldIx::Focus => (prim_hsla("neutral-950"), role_hsla(mode, "input-focus")),
            FieldIx::Invalid => (prim_hsla("neutral-950"), role_hsla(mode, "danger")),
            _ => (prim_hsla("neutral-950"), role_hsla(mode, "input")),
        };
        let body = v_flex()
            .w_full()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(18.))
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .child(text.into()),
            )
            .child(
                h_flex().w_full().justify_end().child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(prim_hsla("neutral-400").unwrap_or(gpui_kit::white()))
                        .child(counter.unwrap_or_default()),
                ),
            );
        let mut field = v_flex()
            .w_full()
            .h(px(96.))
            .p(px(8.))
            .px(px(10.))
            .rounded(px(radius_tier("md")))
            .border_1()
            .child(body);
        if let Some(bg) = bg {
            field = field.bg(bg);
        }
        if let Some(b) = border {
            field = field.border_color(b);
        }
        let mut ring = div()
            .p(px(2.))
            .rounded(px(radius_tier("lg")))
            .border_2()
            .border_color(gpui_kit::transparent_white())
            .child(field);
        if ix == FieldIx::Focus
            && let Some(c) = prim_hsla("white-a16")
        {
            ring = ring.border_color(c);
        }
        if ix == FieldIx::Disabled {
            ring = ring.opacity(0.4);
        }
        v_flex()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(
                        role_hsla(mode, "danger-foreground")
                            .filter(|_| ix == FieldIx::Invalid)
                            .or_else(|| role_hsla(mode, "foreground"))
                            .unwrap_or_default(),
                    )
                    .child(label.into()),
            )
            .child(ring)
    }

    /// Re-export so call sites can name the icon type.
    pub type Glyph = IconName;

    /// Type alias kept for table readability.
    pub type ColorOpt = Option<Hsla>;
}

/// Feedback specimens — ATOMS · 03 progress / spinner / skeleton / tooltip / scrollbar
/// painted plates. Live variants (stock `Progress`/`Spinner`/`Skeleton`, real tooltips,
/// real scrollbars) ride the theme seats; these compositions pin the sheet-exact values
/// for the static gallery cells.
pub mod feedback2 {
    use super::{Mode, prim_hsla, role_hsla};
    use gpui_kit::base::h_flex;
    use gpui_kit::{Div, FontWeight, Hsla, ParentElement as _, SharedString, Styled as _, div, px};

    /// The progress tone (ATOMS · 03: neutral fill, or success/danger/warning/accent).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProgressTone {
        /// inverted neutral fill.
        Neutral,
        /// success (finish).
        Success,
        /// warning.
        Warning,
        /// danger (failure).
        Danger,
        /// accent.
        Accent,
    }

    impl ProgressTone {
        fn fill(self, mode: Mode) -> Option<Hsla> {
            match self {
                ProgressTone::Neutral => role_hsla(mode, "progress-fill"),
                ProgressTone::Success => role_hsla(mode, "success"),
                ProgressTone::Warning => role_hsla(mode, "warning"),
                ProgressTone::Danger => role_hsla(mode, "danger"),
                ProgressTone::Accent => role_hsla(mode, "accent"),
            }
        }
    }

    /// Determinate progress bar: white-a8 track + tone fill, pill radius, given height.
    pub fn progress_bar(pct: f32, tone: ProgressTone, height: f32, width: f32, mode: Mode) -> Div {
        let fill = tone.fill(mode).unwrap_or_default();
        div()
            .relative()
            .w(px(width))
            .h(px(height))
            .rounded_full()
            .bg(prim_hsla("white-a8").unwrap_or_default())
            .child(
                div()
                    .absolute()
                    .h(px(height))
                    .w(px(width * pct.clamp(0., 100.) / 100.))
                    .rounded_full()
                    .bg(fill),
            )
    }

    /// Progress label row: name left (11/400 icon-muted), `n / total` right (10/500).
    pub fn progress_label(
        name: impl Into<SharedString>,
        n: usize,
        total: usize,
        width: f32,
        mode: Mode,
    ) -> Div {
        h_flex()
            .w(px(width))
            .justify_between()
            .child(
                div()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .text_color(role_hsla(mode, "icon-muted").unwrap_or_default())
                    .child(name.into()),
            )
            .child(
                div()
                    .text_size(px(10.))
                    .line_height(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(prim_hsla("neutral-500").unwrap_or_default())
                    .child(format!("{n} / {total}")),
            )
    }

    /// Tooltip plate (the one inverted surface): `tooltip` bg, dark ink, rx6, shadow
    /// 0 4px 12px black@40%, padding 4px 8px, 11/400 text.
    pub fn tooltip_plate(content: Div, width: Option<f32>, mode: Mode) -> Div {
        let mut plate = div()
            .p(px(4.))
            .px(px(8.))
            .rounded(px(6.))
            .shadow_lg()
            .bg(role_hsla(mode, "tooltip").unwrap_or_default())
            .text_color(role_hsla(mode, "tooltip-foreground").unwrap_or_default())
            .child(content);
        if let Some(w) = width {
            plate = plate.w(px(w));
        }
        plate
    }

    /// The inverse kbd hint inside a tooltip: neutral-200 cap, neutral-300 border with a
    /// 2px base, rx4, h16.5, padding 0 5px, 11/500 neutral-750 ink.
    pub fn tooltip_kbd(key: impl Into<SharedString>, _mode: Mode) -> Div {
        div()
            .h(px(16.5))
            .px(px(5.))
            .rounded(px(4.))
            .bg(prim_hsla("neutral-200").unwrap_or_default())
            .border_1()
            .border_color(prim_hsla("neutral-300").unwrap_or_default())
            .border_b(px(2.))
            .text_size(px(11.))
            .line_height(px(15.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(prim_hsla("neutral-750").unwrap_or_default())
            .child(key.into())
    }

    /// Scrollbar interaction state (ATOMS · 03 scrollbar columns).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ScrollIx {
        /// not scrolling: track transparent, thumb 6px white-a24.
        Rest,
        /// scrolling: track white-a4, thumb 8px white-a40.
        Hover,
        /// dragging: track white-a14, thumb 8px neutral-300.
        Pressed,
    }

    /// Scrollbar specimen: 12px track + thumb per state — rest thumb 6px white-a24,
    /// hover track white-a4 + thumb 8px white-a40, pressed track white-a8 + thumb 8px
    /// neutral-300 (ATOMS · 03 scrollbar states).
    pub fn scrollbar(track_h: f32, thumb_top: f32, thumb_h: f32, ix: ScrollIx) -> Div {
        let (track_bg, thumb_w, thumb_bg) = match ix {
            ScrollIx::Rest => (
                Some(gpui_kit::transparent_white()),
                6.,
                prim_hsla("white-a24"),
            ),
            ScrollIx::Hover => (prim_hsla("white-a4"), 8., prim_hsla("white-a40")),
            ScrollIx::Pressed => (prim_hsla("white-a8"), 8., prim_hsla("neutral-300")),
        };
        let mut track = div().relative().w(px(12.)).h(px(track_h)).rounded_full();
        if let Some(bg) = track_bg {
            track = track.bg(bg);
        }
        let mut thumb = div()
            .absolute()
            .left(px(if thumb_w > 6. { 2. } else { 3. }))
            .top(px(thumb_top))
            .w(px(thumb_w))
            .h(px(thumb_h))
            .rounded_full();
        if let Some(bg) = thumb_bg {
            thumb = thumb.bg(bg);
        }
        track.child(thumb)
    }

    /// Type alias kept for table readability.
    pub type ColorOpt = Option<Hsla>;

    /// Re-export for call sites.
    pub type Shared = SharedString;
}

/// Media & misc atoms — ATOMS · 04 sheet: OTP cells, image frames, rating stars,
/// marker rows. Painted specimens pin the sheet-exact geometry from the tokens; the
/// gallery's rest cells use the live stock `OtpInput` / `img()` / `Rating` / `Marker`.
pub mod media {
    use super::atoms::{duotone_icon, icon};
    use super::{Mode, prim_hsla, radius_role, radius_tier, role_hsla};
    use gpui_kit::assets::IconName;
    use gpui_kit::base::h_flex;
    use gpui_kit::component::marker::{Marker, MarkerVariant};
    use gpui_kit::{
        Div, FontWeight, Hsla, ParentElement as _, SharedString, StyleRefinement, Styled as _, div,
        px,
    };

    /// OTP cell interaction state (ATOMS · 04 otp matrix).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OtpIx {
        /// rest: wrapper border transparent, box hairline `input`.
        Rest,
        /// active (focused) cell: wrapper border white-a16, box hairline `input-focus`.
        Active,
        /// invalid: box hairline danger on every cell.
        Invalid,
    }

    /// What the cell's box shows.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OtpContent {
        /// Nothing yet.
        Empty,
        /// A typed digit (16/600 white).
        Digit(char),
        /// Masked digit (dot).
        Masked,
        /// The caret (1.5×18 white) in the active cell.
        Caret,
    }

    /// One OTP cell: rx8 wrapper (2px border, 2px padding) around the 36×40 sunken box
    /// (rx6, hairline outline) — the field ring the sheet reuses for OTP (ATOMS · 04).
    pub fn otp_cell(content: OtpContent, ix: OtpIx, mode: Mode) -> Div {
        let (wrap_border, box_border) = match ix {
            OtpIx::Rest => (None, role_hsla(mode, "input")),
            OtpIx::Active => (prim_hsla("white-a16"), role_hsla(mode, "input-focus")),
            OtpIx::Invalid => (None, role_hsla(mode, "danger")),
        };
        let inner = match content {
            OtpContent::Empty => div(),
            OtpContent::Digit(c) => div()
                .text_size(px(16.))
                .line_height(px(21.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(prim_hsla("neutral-0").unwrap_or_default())
                .child(c.to_string()),
            OtpContent::Masked => div()
                .text_size(px(16.))
                .line_height(px(21.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(prim_hsla("neutral-0").unwrap_or_default())
                .child("•"),
            OtpContent::Caret => div()
                .w(px(1.5))
                .h(px(18.))
                .bg(prim_hsla("neutral-0").unwrap_or_default()),
        };
        let mut cell_box = h_flex()
            .w(px(36.))
            .h(px(40.))
            .items_center()
            .justify_center()
            .rounded(px(radius_tier("md")))
            .border_1()
            .bg(prim_hsla("neutral-950").unwrap_or_default())
            .child(inner);
        if let Some(b) = box_border {
            cell_box = cell_box.border_color(b);
        }
        let mut wrapper = div()
            .p(px(2.))
            .rounded(px(radius_tier("lg")))
            .border_2()
            .child(cell_box);
        if let Some(b) = wrap_border {
            wrapper = wrapper.border_color(b);
        } else {
            wrapper = wrapper.border_color(gpui_kit::transparent_white());
        }
        wrapper
    }

    /// The dash between OTP groups ("3 + 3 with a dash") — 16/600 muted.
    pub fn otp_dash() -> Div {
        div()
            .text_size(px(16.))
            .line_height(px(21.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(prim_hsla("neutral-400").unwrap_or_default())
            .child("–")
    }

    /// One OTP row: three cells, dash, three cells — gap 6px (ATOMS · 04).
    pub fn otp_row(a: [Div; 3], b: [Div; 3]) -> Div {
        h_flex()
            .items_center()
            .gap(px(6.))
            .children(a)
            .child(otp_dash())
            .children(b)
    }

    /// OTP help text under an invalid row — 11/400 danger (ATOMS · 04 invalid state).
    pub fn otp_help(text: impl Into<SharedString>, mode: Mode) -> Div {
        div()
            .text_size(px(11.))
            .line_height(px(15.))
            .text_color(role_hsla(mode, "danger-foreground").unwrap_or_default())
            .child(text.into())
    }

    /// The image frame: raised surface, card radius (ATOMS · 04 image, 160×120 master).
    pub fn image_frame(w: f32, h: f32, mode: Mode) -> Div {
        div()
            .relative()
            .w(px(w))
            .h(px(h))
            .rounded(px(radius_role("card")))
            .bg(role_hsla(mode, "surface-raised").unwrap_or_default())
    }

    /// Loading image: skeleton fill over the whole frame (ATOMS · 04 loading state).
    pub fn image_loading(w: f32, h: f32, mode: Mode) -> Div {
        image_frame(w, h, mode).child(
            div()
                .absolute()
                .size_full()
                .rounded(px(radius_role("card")))
                .bg(prim_hsla("neutral-850").unwrap_or_default()),
        )
    }

    /// Error image: glyph + "Can't load" caption centered on the raised frame
    /// (ATOMS · 04 error state).
    pub fn image_error(w: f32, h: f32, label: impl Into<SharedString>, mode: Mode) -> Div {
        image_frame(w, h, mode).child(
            div()
                .absolute()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(8.))
                .child(
                    icon(IconName::Image, 20.)
                        .text_color(role_hsla(mode, "icon-muted").unwrap_or_default()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
                        .child(label.into()),
                ),
        )
    }

    /// One rating star (20px): on = duotone star in `warning`, off = regular star in
    /// `icon-muted` (ATOMS · 04 rating states).
    pub fn rating_star(on: bool, mode: Mode) -> Div {
        let star = if on {
            duotone_icon(IconName::Star, 20.)
        } else {
            icon(IconName::Star, 20.)
        };
        let color = if on {
            role_hsla(mode, "warning")
        } else {
            role_hsla(mode, "icon-muted")
        };
        div()
            .size(px(20.))
            .child(star.text_color(color.unwrap_or_default()))
    }

    /// Rating interaction state (ATOMS · 04 value × interaction row).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RatingIx {
        /// rest: transparent ring.
        Rest,
        /// focus: 2px `ring` outline around the group (faked as a border, gpui has no outline).
        Focus,
        /// disabled: 40%.
        Disabled,
    }

    /// A painted rating group: padding 2px, rx4, 2px ring on focus, gap 2px stars,
    /// `on` duotone stars then off stars (ATOMS · 04).
    pub fn rating_group(on: usize, max: usize, ix: RatingIx, mode: Mode) -> Div {
        let stars = h_flex()
            .gap(px(2.))
            .children((1..=max).map(|i| rating_star(i <= on, mode)));
        let mut group = div()
            .p(px(2.))
            .rounded(px(4.))
            .border_2()
            .border_color(gpui_kit::transparent_white())
            .child(stars);
        if ix == RatingIx::Focus
            && let Some(c) = role_hsla(mode, "ring")
        {
            group = group.border_color(c);
        }
        if ix == RatingIx::Disabled {
            group = group.opacity(0.4);
        }
        group
    }

    /// Marker row base: fit-content pill, 2px vertical padding, 8px gap, 11/400/15
    /// muted caption (ATOMS · 04 marker plain).
    pub fn marker_plain(m: Marker, mode: Mode) -> Marker {
        let mut m = m
            .w_auto()
            .min_h(px(0.))
            .py(px(2.))
            .px(px(0.))
            .gap(px(8.))
            .rounded_full()
            .text_size(px(11.))
            .line_height(px(15.));
        if let Some(c) = role_hsla(mode, "muted-foreground") {
            m = m.text_color(c);
        }
        m
    }

    /// Marker border variant as the sheet draws it: pill outline 1px `border-subtle`,
    /// 10px horizontal padding (ATOMS · 04 border — stock Border is a bottom rule, so
    /// the pill look is a Plain marker with a refined frame).
    pub fn marker_pill(m: Marker, mode: Mode) -> Marker {
        let mut m = marker_plain(m, mode).px(px(10.)).border_1();
        if let Some(c) = role_hsla(mode, "border-subtle") {
            m = m.border_color(c);
        }
        m
    }

    /// Marker separator variant: hairlines either side, centered, 480px wide, rule in
    /// white-a8 (ATOMS · 04 separator = day breaks in the ledger).
    pub fn marker_separator(m: Marker, mode: Mode) -> Marker {
        let mut style = StyleRefinement::default();
        if let Some(c) = prim_hsla("white-a8") {
            style = style.bg(c);
        }
        marker_plain(m, mode)
            .with_variant(MarkerVariant::Separator)
            .w(px(480.))
            .separator_style(style)
    }

    /// Time/meta ink inside a marker — 11/400 `subtle-foreground` (ATOMS · 04).
    pub fn marker_meta(text: impl Into<SharedString>, mode: Mode) -> Div {
        div()
            .text_size(px(11.))
            .line_height(px(15.))
            .text_color(
                role_hsla(mode, "subtle-foreground")
                    .or_else(|| prim_hsla("neutral-500"))
                    .unwrap_or_default(),
            )
            .child(text.into())
    }

    /// Re-export so call sites can name the color type.
    pub type ColorOpt = Option<Hsla>;
}

// ── MOLECULES · 01 — popover / menu / select / combobox ──────────────────────
/// Popup surfaces + menu rows (MOLECULES · 01). The floating panel: `popover`
/// fill (#222 dark), hairline `border-subtle`, 0 8 24 black-a60 shadow, rx12,
/// padding 4 for lists / 12 for popover cards.
pub mod popups {
    use super::atoms::{duotone_icon, icon};
    use super::{Mode, prim_hsla, radius_tier, role_hsla};
    use gpui_kit::assets::IconName;
    use gpui_kit::base::{h_flex, v_flex};
    use gpui_kit::component::StyledExt as _;
    use gpui_kit::{
        BoxShadow, Div, FontWeight, ParentElement as _, SharedString, StyleRefinement, Styled as _,
        div, px,
    };

    /// The floating panel shadow (TOKENS §06: only popovers float).
    pub fn popup_shadow() -> Vec<BoxShadow> {
        vec![
            BoxShadow::new(px(0.), px(8.), prim_hsla("black-a60").unwrap_or_default())
                .blur_radius(px(12.)),
        ]
    }

    /// A floating panel: popover fill, hairline border-subtle, rx12, shadow, padding.
    pub fn popup_surface(w: f32, pad: f32, mode: Mode) -> Div {
        let mut el = div()
            .w(px(w))
            .p(px(pad))
            .rounded(px(radius_tier("xl")))
            .border_1()
            .bg(role_hsla(mode, "popover").unwrap_or_default())
            .refine_style(&StyleRefinement {
                box_shadow: Some(popup_shadow()),
                ..Default::default()
            });
        if let Some(b) = role_hsla(mode, "border-subtle") {
            el = el.border_color(b);
        }
        el
    }

    /// Menu/popover row interaction state (MOLECULES · 01 row × interaction).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MenuIx {
        /// rest: transparent, regular glyph.
        Rest,
        /// hover/keyboard focus: white-a4 fill, duotone glyph.
        Hover,
        /// pressed: white-a8 fill, duotone glyph.
        Pressed,
        /// disabled: 40%, neutral-600 label.
        Disabled,
    }

    /// One menu row: h28, rx6, padding 0 8, gap 8 — leading slot (icon / check),
    /// label 12/400, trailing kbd 11/500 (#6b6b6b).
    pub fn menu_item(
        label: impl Into<SharedString>,
        leading: Option<IconName>,
        checked: bool,
        kbd: Option<&'static str>,
        ix: MenuIx,
        mode: Mode,
    ) -> Div {
        let (bg, label_color, duotone) = match ix {
            MenuIx::Rest => (None, prim_hsla("neutral-0"), false),
            MenuIx::Hover => (prim_hsla("white-a4"), prim_hsla("neutral-0"), true),
            MenuIx::Pressed => (prim_hsla("white-a8"), prim_hsla("neutral-0"), true),
            MenuIx::Disabled => (None, prim_hsla("neutral-600"), false),
        };
        let glyph = if checked {
            Some(if duotone {
                duotone_icon(IconName::Check, 14.)
            } else {
                icon(IconName::Check, 14.)
            })
        } else {
            leading.map(|n| {
                if duotone {
                    duotone_icon(n, 14.)
                } else {
                    icon(n, 14.)
                }
            })
        };
        let mut row = h_flex()
            .w_full()
            .h(px(28.))
            .px(px(8.))
            .gap(px(8.))
            .items_center()
            .rounded(px(radius_tier("md")));
        if let Some(bg) = bg {
            row = row.bg(bg);
        }
        if let Some(g) = glyph {
            let c = role_hsla(mode, "icon").unwrap_or_default();
            row = row.child(g.text_color(c));
        } else {
            row = row.child(div().size(px(14.)));
        }
        row = row.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .text_color(label_color.unwrap_or_default())
                .child(label.into()),
        );
        if ix != MenuIx::Disabled {
            row = row.child(div().flex_1());
        }
        if let Some(k) = kbd {
            row = row.child(
                div()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(prim_hsla("neutral-500").unwrap_or_default())
                    .child(k),
            );
        }
        if ix == MenuIx::Disabled {
            row = row.opacity(0.4);
        }
        row
    }

    /// Menu separator: 1px white-a8 line with 4px vertical padding.
    pub fn menu_separator() -> Div {
        v_flex().w_full().py(px(4.)).child(
            div()
                .w_full()
                .h(px(1.))
                .bg(prim_hsla("white-a8").unwrap_or_default()),
        )
    }

    /// Menu group label: h24, padding 0 8, 10/500 muted overline.
    pub fn menu_label(text: impl Into<SharedString>, mode: Mode) -> Div {
        div()
            .w_full()
            .h(px(24.))
            .px(px(8.))
            .flex()
            .items_center()
            .text_size(px(10.))
            .line_height(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
            .child(text.into())
    }

    /// Select/Combobox/DatePicker trigger: rx8 2px ring wrapper (white-a16 when
    /// open/focused) around the sunken 32px field (rx6, padding 0 10, gap 8).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TriggerIx {
        /// rest.
        Rest,
        /// hover.
        Hover,
        /// open/focus: white-a16 ring, duotone caret, field hairline input-focus.
        Open,
        /// disabled: 40%.
        Disabled,
    }

    /// Select/Combobox trigger field (also reused by the date picker).
    pub fn select_trigger(
        leading: Option<(IconName, bool)>, // (glyph, duotone)
        value: Option<SharedString>,       // None = placeholder
        w: f32,
        ix: TriggerIx,
        mode: Mode,
    ) -> Div {
        let mut field = h_flex()
            .w_full()
            .h(px(32.))
            .px(px(10.))
            .gap(px(8.))
            .items_center()
            .rounded(px(radius_tier("md")))
            .bg(prim_hsla("neutral-950").unwrap_or_default());
        if ix == TriggerIx::Open
            && let Some(b) = role_hsla(mode, "input-focus")
        {
            field = field.border_1().border_color(b);
        }
        if let Some((name, duotone)) = leading {
            let ic = if duotone {
                duotone_icon(name, 14.)
            } else {
                icon(name, 14.)
            };
            field = field.child(ic.text_color(role_hsla(mode, "icon").unwrap_or_default()));
        }
        let text_color = if value.is_some() {
            prim_hsla("neutral-0").unwrap_or_default()
        } else {
            prim_hsla("neutral-500").unwrap_or_default()
        };
        field = field.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .text_color(text_color)
                .child(value.unwrap_or_default()),
        );
        field = field.child(div().flex_1());
        let caret_duotone = ix == TriggerIx::Open || ix == TriggerIx::Hover;
        let caret = if caret_duotone {
            duotone_icon(IconName::ChevronsUpDown, 12.)
        } else {
            icon(IconName::ChevronsUpDown, 12.)
        };
        field = field.child(caret.text_color(role_hsla(mode, "icon").unwrap_or_default()));
        let mut wrapper = div()
            .w(px(w))
            .p(px(2.))
            .rounded(px(radius_tier("lg")))
            .border_2()
            .border_color(gpui_kit::transparent_white())
            .child(field);
        if ix == TriggerIx::Open
            && let Some(c) = prim_hsla("white-a16")
        {
            wrapper = wrapper.border_color(c);
        }
        if ix == TriggerIx::Disabled {
            wrapper = wrapper.opacity(0.4);
        }
        wrapper
    }

    /// Search row at the top of an open list: bottom hairline white-a8, magnifier +
    /// query 12/400 + caret 1×14.
    pub fn search_row(query: impl Into<SharedString>, mode: Mode) -> Div {
        h_flex()
            .w_full()
            .h(px(32.))
            .px(px(8.))
            .gap(px(8.))
            .items_center()
            .border_b(px(1.))
            .border_color(prim_hsla("white-a8").unwrap_or_default())
            .child(
                icon(IconName::Search, 14.).text_color(role_hsla(mode, "icon").unwrap_or_default()),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(14.))
                    .text_color(prim_hsla("neutral-0").unwrap_or_default())
                    .child(query.into()),
            )
            .child(
                div()
                    .w(px(1.))
                    .h(px(14.))
                    .bg(prim_hsla("neutral-0").unwrap_or_default()),
            )
    }

    /// Empty-list row: padding 12 8, 11/400 muted.
    pub fn empty_row(text: impl Into<SharedString>, mode: Mode) -> Div {
        div()
            .w_full()
            .p(px(12.))
            .px(px(8.))
            .text_size(px(11.))
            .line_height(px(15.))
            .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
            .child(text.into())
    }

    /// Combobox chip inside the trigger: #1c1c1c rx4 h20 pad 0 6 gap 4, optional
    /// 6px category dot, label 11/500 #a8a8a8.
    pub fn chip(label: impl Into<SharedString>, dot: Option<gpui_kit::Hsla>, mode: Mode) -> Div {
        let mut c = h_flex()
            .h(px(20.))
            .px(px(6.))
            .gap(px(4.))
            .items_center()
            .rounded(px(4.))
            .bg(prim_hsla("neutral-875").unwrap_or_default());
        if let Some(d) = dot {
            c = c.child(div().size(px(6.)).rounded_full().bg(d));
        }
        c.child(
            div()
                .text_size(px(11.))
                .line_height(px(15.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(role_hsla(mode, "secondary-foreground").unwrap_or_default())
                .child(label.into()),
        )
    }

    /// Popover card (info popover): w280, padding 12, gap 12 — head (title 12/500
    /// white + desc 11/400 muted) and an actions row.
    pub fn popover_card(head: Div, actions: Div, mode: Mode) -> Div {
        popup_surface(280., 12., mode)
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(head)
            .child(actions)
    }

    /// Popover head: title 12/500 white, description 11/400 muted, gap 4.
    pub fn popover_head(
        title: impl Into<SharedString>,
        desc: impl Into<SharedString>,
        mode: Mode,
    ) -> Div {
        v_flex()
            .w_full()
            .gap(px(4.))
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .child(title.into()),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
                    .child(desc.into()),
            )
    }
}

// ── MOLECULES · 01 — date picker ──────────────────────────────────────────────
/// Date-picker calendar pieces (MOLECULES · 01).
pub mod pickers {
    use super::atoms::icon;
    use super::popups::{TriggerIx, popup_shadow};
    use super::{Mode, prim_hsla, radius_tier, role_hsla};
    use gpui_kit::assets::IconName;
    use gpui_kit::base::h_flex;
    use gpui_kit::component::StyledExt as _;
    use gpui_kit::{
        Div, FontWeight, ParentElement as _, SharedString, StyleRefinement, Styled as _, div, px,
    };

    /// Calendar day cell state (MOLECULES · 01 day-cell axis).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DayIx {
        /// default day.
        Default,
        /// today: accent ring, white number.
        Today,
        /// selected/range end: primary fill, dark number.
        Selected,
        /// in-range middle: white-a8 fill.
        RangeMiddle,
        /// other-month day: muted.
        Outside,
        /// disabled: struck, neutral-600.
        Disabled,
        /// hover: white-a4 fill.
        Hover,
    }

    /// One 32×32 day cell, rx6, 1px border, number 12/400.
    pub fn day_cell(n: u32, ix: DayIx, _mode: Mode) -> Div {
        let (bg, border, color) = match ix {
            DayIx::Default => (None, None, prim_hsla("neutral-300")),
            DayIx::Today => (None, prim_hsla("orange-500"), prim_hsla("neutral-0")),
            DayIx::Selected => (prim_hsla("neutral-100"), None, prim_hsla("neutral-925")),
            DayIx::RangeMiddle => (prim_hsla("white-a8"), None, prim_hsla("neutral-300")),
            DayIx::Outside => (None, None, prim_hsla("neutral-500")),
            DayIx::Disabled => (None, None, prim_hsla("neutral-600")),
            DayIx::Hover => (prim_hsla("white-a4"), None, prim_hsla("neutral-300")),
        };
        let mut cell = div()
            .size(px(32.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius_tier("md")))
            .border_1()
            .border_color(border.unwrap_or_else(gpui_kit::transparent_white));
        if let Some(bg) = bg {
            cell = cell.bg(bg);
        }
        cell.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .text_color(color.unwrap_or_default())
                .child(n.to_string()),
        )
    }

    /// Calendar popup frame: popover fill, hairline, shadow, rx12, padding 12,
    /// width 248 (single month).
    pub fn calendar_popup(mode: Mode) -> Div {
        let mut el = div()
            .w(px(248.))
            .p(px(12.))
            .rounded(px(radius_tier("xl")))
            .border_1()
            .bg(role_hsla(mode, "popover").unwrap_or_default())
            .refine_style(&StyleRefinement {
                box_shadow: Some(popup_shadow()),
                ..Default::default()
            })
            .flex()
            .flex_col()
            .gap(px(4.));
        if let Some(b) = role_hsla(mode, "border-subtle") {
            el = el.border_color(b);
        }
        el
    }

    /// Calendar head: prev/next month icon-buttons 24×24 rx6 + month title 12/500.
    pub fn calendar_head(month: impl Into<SharedString>, mode: Mode) -> Div {
        let btn = |name: IconName| {
            div()
                .size(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(radius_tier("md")))
                .child(icon(name, 14.).text_color(role_hsla(mode, "icon").unwrap_or_default()))
        };
        h_flex()
            .w_full()
            .pb(px(8.))
            .items_center()
            .justify_between()
            .child(btn(IconName::ChevronLeft))
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .child(month.into()),
            )
            .child(btn(IconName::ChevronRight))
    }

    /// Weekday header cell: 32×24, 10/500 muted.
    pub fn weekday(label: &'static str, mode: Mode) -> Div {
        div()
            .w(px(32.))
            .h(px(24.))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(10.))
            .line_height(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
            .child(label)
    }

    /// Date-picker trigger: like a select trigger with the calendar glyph
    /// (duotone when open) — reuses popups::select_trigger via leading icon.
    pub fn date_trigger(value: Option<SharedString>, w: f32, ix: TriggerIx, mode: Mode) -> Div {
        super::popups::select_trigger(
            Some((IconName::Calendar, ix == TriggerIx::Open)),
            value,
            w,
            ix,
            mode,
        )
    }

    /// Range-preset row in the left rail: h28 rx6 pad 0 8, label 12/400 white,
    /// hover white-a4 (rail separated by a right hairline white-a8).
    pub fn preset_row(label: impl Into<SharedString>, hover: bool, mode: Mode) -> Div {
        let mut r = h_flex()
            .w_full()
            .h(px(28.))
            .px(px(8.))
            .items_center()
            .rounded(px(radius_tier("md")));
        if hover {
            r = r.bg(prim_hsla("white-a4").unwrap_or_default());
        }
        r.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                .child(label.into()),
        )
    }
}

// ── MOLECULES · 01 — tabs / empty ─────────────────────────────────────────────
/// Tabs + empty states (MOLECULES · 01).
pub mod structure {
    use super::atoms::icon;
    use super::{Mode, prim_hsla, radius_tier, role_hsla};
    use gpui_kit::assets::IconName;
    use gpui_kit::base::{h_flex, v_flex};
    use gpui_kit::{Div, FontWeight, ParentElement as _, SharedString, Styled as _, div, px};

    /// Underline-tab interaction state.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TabIx {
        /// rest.
        Rest,
        /// hover: white-a16 line preview, white label.
        Hover,
        /// selected: 2px white line.
        Selected,
        /// disabled: neutral-600 label.
        Disabled,
    }

    /// One underline tab: h31, padding 0 4, 2px bottom line (transparent / white-a16
    /// hover / white selected), gap 6, icon 14, label 12/500, optional count badge.
    pub fn tab_underline(
        label: impl Into<SharedString>,
        glyph: Option<IconName>,
        count: Option<usize>,
        ix: TabIx,
        _mode: Mode,
    ) -> Div {
        let (line, label_color) = match ix {
            TabIx::Rest => (prim_hsla("transparent"), prim_hsla("neutral-400")),
            TabIx::Hover => (prim_hsla("white-a16"), prim_hsla("neutral-0")),
            TabIx::Selected => (prim_hsla("neutral-0"), prim_hsla("neutral-0")),
            TabIx::Disabled => (prim_hsla("transparent"), prim_hsla("neutral-600")),
        };
        let mut content = h_flex()
            .h(px(31.))
            .px(px(4.))
            .gap(px(6.))
            .items_center()
            .border_b(px(2.))
            .border_color(line.unwrap_or_else(gpui_kit::transparent_white));
        if let Some(g) = glyph {
            content = content.child(icon(g, 14.).text_color(label_color.unwrap_or_default()));
        }
        content = content.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color.unwrap_or_default())
                .child(label.into()),
        );
        if let Some(n) = count {
            content = content.child(
                div()
                    .h(px(16.))
                    .px(px(5.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .bg(prim_hsla("neutral-200").unwrap_or_default())
                    .text_size(px(10.))
                    .line_height(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(prim_hsla("neutral-925").unwrap_or_default())
                    .child(n.to_string()),
            );
        }
        content
    }

    /// Underline bar: bottom hairline white-a8, height 31.5, gap 24.
    pub fn underline_bar(tabs: Div, w: f32) -> Div {
        h_flex()
            .w(px(w))
            .h(px(31.5))
            .gap(px(24.))
            .items_end()
            .border_b(px(1.))
            .border_color(prim_hsla("white-a8").unwrap_or_default())
            .child(tabs)
    }

    /// Segmented bar container: #1c1c1c rx6, padding 2, gap 2.
    pub fn segmented_bar(segs: Div) -> Div {
        h_flex()
            .p(px(2.))
            .gap(px(2.))
            .rounded(px(radius_tier("md")))
            .bg(prim_hsla("neutral-875").unwrap_or_default())
            .child(segs)
    }

    /// One segmented tab: h24, padding 0 10, rx4 — rest transparent / hover white-a4 /
    /// selected #2a2a2a; label 12/500.
    pub fn tab_segment(label: impl Into<SharedString>, ix: TabIx, _mode: Mode) -> Div {
        let (bg, label_color) = match ix {
            TabIx::Rest => (None, prim_hsla("neutral-400")),
            TabIx::Hover => (prim_hsla("white-a4"), prim_hsla("neutral-0")),
            TabIx::Selected => (prim_hsla("neutral-800"), prim_hsla("neutral-0")),
            TabIx::Disabled => (None, prim_hsla("neutral-600")),
        };
        let mut seg = h_flex()
            .h(px(24.))
            .px(px(10.))
            .items_center()
            .rounded(px(4.));
        if let Some(bg) = bg {
            seg = seg.bg(bg);
        }
        seg.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color.unwrap_or_default())
                .child(label.into()),
        )
    }

    /// Empty state block: gap 12, padding 32 24 — 24px muted glyph, title 16/600,
    /// description 12/400 muted, actions row (padding-top 8, gap 8).
    pub fn empty_block(
        w: f32,
        glyph: IconName,
        icon_color: Option<gpui_kit::Hsla>,
        title: impl Into<SharedString>,
        desc: impl Into<SharedString>,
        actions: Div,
        mode: Mode,
    ) -> Div {
        let icon_ink = icon_color
            .or_else(|| role_hsla(mode, "icon-muted"))
            .unwrap_or_default();
        v_flex()
            .w(px(w))
            .p(px(32.))
            .px(px(24.))
            .gap(px(12.))
            .child(icon(glyph, 24.).text_color(icon_ink))
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(16.))
                            .line_height(px(21.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                            .child(title.into()),
                    )
                    .child(
                        div()
                            .w(px(w - 80.))
                            .text_size(px(12.))
                            .line_height(px(18.))
                            .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
                            .child(desc.into()),
                    ),
            )
            .child(actions.pt(px(8.)).flex().gap(px(8.)))
    }
}

#[cfg(test)]
mod tests {
    use super::controls::ControlSize;
    use super::*;
    use bookkeepy_primary_data::theme::Mode;

    #[test]
    fn control_scale_matches_the_design_sheets() {
        // ATOMS · 01 sizes row as drawn: 20/24/32/40 height, rx 4/6/6/8,
        // h-padding 6/8/12/16, glyph 12/14/14/16.
        let ladder = [
            (ControlSize::XSmall, 20, 4, 6, 12),
            (ControlSize::Small, 24, 6, 8, 14),
            (ControlSize::Medium, 32, 6, 12, 14),
            (ControlSize::Large, 40, 8, 16, 16),
        ];
        for (size, height, radius, h_pad, icon) in ladder {
            assert_eq!(size.height(), height, "{size:?} height");
            assert_eq!(size.radius(), radius, "{size:?} radius");
            assert_eq!(size.h_padding(), h_pad, "{size:?} h-padding");
            assert_eq!(size.icon_px(), icon, "{size:?} icon");
        }
    }

    #[test]
    fn tokens_resolve_for_both_modes() {
        for mode in [Mode::Dark, Mode::Light] {
            assert!(role_hsla(mode, "accent").is_some(), "{mode:?}");
            assert!(role_hsla(mode, "primary").is_some(), "{mode:?}");
            assert!(role_hsla(mode, "surface-raised").is_some(), "{mode:?}");
        }
        assert!(prim_hsla("white-a8").is_some());
        assert!(prim_hsla("orange-500").is_some());
        // A corrupted token degrades to `None` — the themed fallback — never a panic.
        assert!(token_hsla("not-a-color").is_none());
        assert!(colors(Mode::Dark).get("no-such-role").is_none());
    }
}
