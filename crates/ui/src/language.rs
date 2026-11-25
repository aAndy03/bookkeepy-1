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
    use gpui_kit::component::avatar::Avatar;
    use gpui_kit::component::tag::Tag;
    use gpui_kit::{Styled as _, px};

    /// The tag tone, selecting the matching well + tint token pair (ATOMS · 02).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TagTone {
        /// The neutral tag: `control` fill + `icon` ink.
        Neutral,
        /// `accent-subtle` well + `accent-foreground` tint.
        Accent,
        /// `success-subtle` + `success-foreground`.
        Success,
        /// `amber-900` well + `warning-foreground`.
        Warning,
        /// `danger-subtle` + `danger-foreground`.
        Danger,
        /// `cyan-900` well + `cyan-300`.
        Info,
    }

    /// A soft tag: well fill + tint ink + radius-full pill, no border (ATOMS · 02 tag
    /// variants — the design's tags are filled pills, not outlined chips).
    pub fn tag_soft(tag: Tag, tone: TagTone, mode: Mode) -> Tag {
        let (well, tint) = match tone {
            TagTone::Neutral => (role_hsla(mode, "control"), role_hsla(mode, "icon")),
            TagTone::Accent => (
                role_hsla(mode, "accent-subtle"),
                role_hsla(mode, "accent-foreground"),
            ),
            TagTone::Success => (
                role_hsla(mode, "success-subtle"),
                role_hsla(mode, "success-foreground"),
            ),
            TagTone::Warning => (
                prim_hsla("amber-900"),
                role_hsla(mode, "warning-foreground"),
            ),
            TagTone::Danger => (
                role_hsla(mode, "danger-subtle"),
                role_hsla(mode, "danger-foreground"),
            ),
            TagTone::Info => (prim_hsla("cyan-900"), prim_hsla("cyan-300")),
        };
        let mut tag = tag.rounded(px(radius_tier("full"))).border_0();
        if let Some(bg) = well {
            tag = tag.bg(bg);
        }
        if let Some(fg) = tint {
            tag = tag.text_color(fg);
        }
        tag
    }

    /// An avatar: borderless circle (ATOMS · 02 — the ring appears only in overlap
    /// stacks, which AvatarGroup draws itself).
    pub fn avatar(avatar: Avatar) -> Avatar {
        avatar.border_0()
    }
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
