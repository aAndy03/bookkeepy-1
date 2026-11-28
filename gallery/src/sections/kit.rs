// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared specimen scaffolding for gallery sections (RULE-UI-006, ADR-0017).
//!
//! Every section is a declarative list of groups; each group is a caption + rows of
//! cells built with these helpers, so a new sheet = a new data table, not new layout
//! code. Pages render FLAT on the background exactly like the exported design frames
//! (no cards around elements — a card only when it is part of a component's own design).

use bookkeepy_primary_data::theme::style;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{
    App, DefiniteLength, Div, FontWeight, ParentElement as _, SharedString, Styled as _, div, px,
};

use crate::app::tr;

/// Apply a named type style (`data/theme/typography.toml`) to a div — text always
/// renders through a named style, never a raw size/weight in code (TOKENS §03).
pub fn with_style(el: Div, id: &str) -> Div {
    let mut el = el;
    if let Some(s) = style(id) {
        el = el
            .text_size(px(s.size_px as f32))
            .line_height(DefiniteLength::Fraction(
                s.line_height_px as f32 / s.size_px as f32,
            ))
            .font_weight(FontWeight::from(s.weight as f32));
    }
    el
}

/// Overline caption (cell state captions) in muted ink.
pub fn overline(cx: &App, text: impl Into<SharedString>) -> Div {
    with_style(
        div()
            .text_color(cx.theme().muted_foreground)
            .child(text.into()),
        "overline",
    )
}

/// Sheet eyebrow — the "bookkeepy · atoms 01" line above each section title. The design
/// paints this one overline in `accent-foreground` (#f5a76a), unlike cell captions.
pub fn eyebrow(cx: &App, text: impl Into<SharedString>) -> Div {
    let mode = cx.global::<crate::app::Prefs>().mode;
    let ink = bookkeepy_ui::api::language::role_hsla(mode, "accent-foreground")
        .unwrap_or(cx.theme().accent_foreground);
    with_style(div().text_color(ink).child(text.into()), "overline")
}

/// A `StyleRefinement` carrying the named `label` type style (12/400/14) — for pinning
/// stock components' text to the label size without a raw value in code (TOKENS §03).
pub fn label_refinement() -> gpui_kit::StyleRefinement {
    let mut st = gpui_kit::StyleRefinement::default();
    if let Some(s) = style("label") {
        st = st
            .text_size(px(s.size_px as f32))
            .line_height(px(s.line_height_px as f32))
            .font_weight(FontWeight::from(s.weight as f32));
    }
    st
}

/// Body caption (hints, demo text) in muted ink.
pub fn caption(cx: &App, text: impl Into<SharedString>) -> Div {
    with_style(
        div()
            .text_color(cx.theme().muted_foreground)
            .child(text.into()),
        "caption",
    )
}

/// Group title (the sheet's headline style, foreground ink).
pub fn group_title(cx: &App, text: impl Into<SharedString>) -> Div {
    with_style(
        div().text_color(cx.theme().foreground).child(text.into()),
        "headline",
    )
}

/// One specimen cell: state/variant caption above the live component.
/// `items_start` keeps the component hugging its intrinsic width at the leading
/// edge (design frames left-align; without it the cell stretches full-width).
pub fn cell(cx: &App, state_key: &str, el: impl gpui_kit::IntoElement) -> Div {
    v_flex()
        .items_start()
        .gap_2()
        .child(overline(cx, tr(cx, state_key)))
        .child(el)
}

/// A labeled group: overline group key + hint caption + content rows.
pub fn group(cx: &App, title_key: &str, hint_key: &str, content: Div) -> Div {
    v_flex()
        .gap_4()
        .child(overline(cx, tr(cx, title_key)))
        .child(caption(cx, tr(cx, hint_key)))
        .child(content)
}

/// A horizontal row of cells.
pub fn row(cells: impl IntoIterator<Item = Div>) -> Div {
    h_flex().gap_6().items_start().children(cells)
}

/// The interaction states every matrix column shows (ATOMS · 01: rest/hover/pressed/
/// focus are live; selected/open/loading/disabled are forced via the component flags).
pub const STATE_KEYS: &[&str] = &[
    "gallery.atoms.state.rest",
    "gallery.atoms.state.hover",
    "gallery.atoms.state.pressed",
    "gallery.atoms.state.focus",
    "gallery.atoms.state.selected",
    "gallery.atoms.state.open",
    "gallery.atoms.state.loading",
    "gallery.atoms.state.disabled",
];
