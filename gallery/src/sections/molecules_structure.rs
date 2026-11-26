// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 01 sheet (part 3) — tabs, empty (references/designlanguage).
//!
//! FLAT on the page background. Live cells: interactive `TabBar` (underline with
//! icons + count badge, and segmented) and `Empty` blocks; the per-tab interaction
//! ladder is painted from `language::structure`.

use bookkeepy_ui::api::language::{
    atoms, controls::ControlSize, prim_hsla, structure, structure::TabIx,
};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::component::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::{
    AnyView, AppContext as _, Context, ParentElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| MoleculesStructure { tab: 0 }).into()
}

pub struct MoleculesStructure {
    tab: usize,
}

/// The count badge used as a tab suffix (h16, pad 0 5, rx999, neutral-200 fill).
fn count_badge(n: usize) -> gpui_kit::Div {
    div()
        .h(px(16.))
        .px(px(5.))
        .flex()
        .items_center()
        .rounded_full()
        .bg(prim_hsla("neutral-200").unwrap_or_default())
        .text_size(px(10.))
        .line_height(px(12.))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .text_color(prim_hsla("neutral-925").unwrap_or_default())
        .child(n.to_string())
}

impl gpui_kit::Render for MoleculesStructure {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── tabs ─────────────────────────────────────────────────────────────
        let tab_masters = kit::row([
            kit::cell(
                cx,
                "gallery.m01.tab.underline",
                structure::tab_underline(
                    tr(cx, "gallery.m01.tab.transactions"),
                    None,
                    None,
                    TabIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.tab.segmented",
                structure::tab_segment(tr(cx, "gallery.m01.tab.day"), TabIx::Rest, mode),
            ),
        ]);
        let tab_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                structure::tab_underline(
                    tr(cx, "gallery.m01.tab.tab"),
                    None,
                    None,
                    TabIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                structure::tab_underline(
                    tr(cx, "gallery.m01.tab.tab"),
                    None,
                    None,
                    TabIx::Hover,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.selected",
                structure::tab_underline(
                    tr(cx, "gallery.m01.tab.tab"),
                    None,
                    None,
                    TabIx::Selected,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                structure::tab_underline(
                    tr(cx, "gallery.m01.tab.tab"),
                    None,
                    None,
                    TabIx::Disabled,
                    mode,
                ),
            ),
        ]);
        // Live underline bar with icons + count, click to switch.
        let live_bar = TabBar::new("tabs-underline")
            .underline()
            .selected_index(self.tab)
            .on_click(cx.listener(|view, ix: &usize, _window, cx| {
                view.tab = *ix;
                cx.notify();
            }))
            .child(
                Tab::new()
                    .label(tr(cx, "gallery.m01.tab.transactions"))
                    .icon(IconName::Receipt),
            )
            .child(
                Tab::new()
                    .label(tr(cx, "gallery.m01.tab.uncategorized"))
                    .icon(IconName::Tag)
                    .suffix(count_badge(12)),
            )
            .child(
                Tab::new()
                    .label(tr(cx, "gallery.m01.tab.scheduled"))
                    .icon(IconName::Clock),
            )
            .child(
                Tab::new()
                    .label(tr(cx, "gallery.m01.tab.archived"))
                    .icon(IconName::Archive)
                    .disabled(true),
            );
        // Live segmented bar.
        let seg_bar = TabBar::new("tabs-segmented")
            .segmented()
            .selected_index(self.tab.min(3))
            .on_click(cx.listener(|view, ix: &usize, _window, cx| {
                view.tab = *ix;
                cx.notify();
            }))
            .child(Tab::new().label(tr(cx, "gallery.m01.tab.day")))
            .child(Tab::new().label(tr(cx, "gallery.m01.tab.week")))
            .child(Tab::new().label(tr(cx, "gallery.m01.tab.month")))
            .child(Tab::new().label(tr(cx, "gallery.m01.tab.year")));
        let painted_bars = kit::row([
            kit::cell(
                cx,
                "gallery.m01.tab.bar_underline",
                structure::underline_bar(
                    h_flex()
                        .gap(px(24.))
                        .items_end()
                        .child(structure::tab_underline(
                            tr(cx, "gallery.m01.tab.transactions"),
                            Some(IconName::Receipt),
                            None,
                            TabIx::Selected,
                            mode,
                        ))
                        .child(structure::tab_underline(
                            tr(cx, "gallery.m01.tab.uncategorized"),
                            Some(IconName::Tag),
                            Some(12),
                            TabIx::Rest,
                            mode,
                        ))
                        .child(structure::tab_underline(
                            tr(cx, "gallery.m01.tab.scheduled"),
                            Some(IconName::Clock),
                            None,
                            TabIx::Hover,
                            mode,
                        ))
                        .child(structure::tab_underline(
                            tr(cx, "gallery.m01.tab.archived"),
                            Some(IconName::Archive),
                            None,
                            TabIx::Disabled,
                            mode,
                        )),
                    560.,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.tab.bar_segmented",
                structure::segmented_bar(
                    h_flex()
                        .gap(px(2.))
                        .child(structure::tab_segment(
                            tr(cx, "gallery.m01.tab.day"),
                            TabIx::Rest,
                            mode,
                        ))
                        .child(structure::tab_segment(
                            tr(cx, "gallery.m01.tab.week"),
                            TabIx::Hover,
                            mode,
                        ))
                        .child(structure::tab_segment(
                            tr(cx, "gallery.m01.tab.month"),
                            TabIx::Selected,
                            mode,
                        ))
                        .child(structure::tab_segment(
                            tr(cx, "gallery.m01.tab.year"),
                            TabIx::Rest,
                            mode,
                        )),
                ),
            ),
        ]);
        let tabs_group = kit::group(
            cx,
            "gallery.m01.tabs.title",
            "gallery.m01.tabs.hint",
            v_flex()
                .gap_6()
                .child(tab_masters)
                .child(tab_states)
                .child(painted_bars)
                .child(kit::row([
                    kit::cell(cx, "gallery.m01.tabs.live_underline", live_bar),
                    kit::cell(cx, "gallery.m01.tabs.live_segmented", seg_bar),
                ])),
        );

        // ── empty ────────────────────────────────────────────────────────────
        let live_empty = Empty::new()
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(atoms::icon(IconName::Receipt, 24.)),
                    )
                    .title(EmptyTitle::new().child(tr(cx, "gallery.m01.empty.title")))
                    .description(EmptyDescription::new().child(tr(cx, "gallery.m01.empty.desc"))),
            )
            .content(
                EmptyContent::new()
                    .child(atoms::button(
                        "empty-add",
                        ButtonVariant::Primary,
                        Some(atoms::ButtonGlyph::Icon(IconName::Plus)),
                        tr(cx, "gallery.m01.empty.add"),
                        ControlSize::Medium,
                    ))
                    .child(atoms::button(
                        "empty-import",
                        ButtonVariant::Ghost,
                        None,
                        tr(cx, "gallery.m01.empty.import"),
                        ControlSize::Medium,
                    )),
            );
        // Painted variants on their own #171717 rx16 frames (part of the design).
        let variant_frame = |inner: gpui_kit::Div| {
            div()
                .rounded(px(bookkeepy_ui::api::language::radius_tier("2xl")))
                .bg(prim_hsla("neutral-900").unwrap_or_default())
                .child(inner)
        };
        let empty_variants = kit::row([
            kit::cell(
                cx,
                "gallery.m01.empty.first_run",
                variant_frame(structure::empty_block(
                    300.,
                    IconName::Receipt,
                    tr(cx, "gallery.m01.empty.first_run_t"),
                    tr(cx, "gallery.m01.empty.first_run_d"),
                    h_flex()
                        .child(atoms::button(
                            "e1a",
                            ButtonVariant::Primary,
                            Some(atoms::ButtonGlyph::Icon(IconName::Plus)),
                            tr(cx, "gallery.m01.empty.add"),
                            ControlSize::Medium,
                        ))
                        .child(atoms::button(
                            "e1b",
                            ButtonVariant::Ghost,
                            None,
                            tr(cx, "gallery.m01.empty.import"),
                            ControlSize::Medium,
                        )),
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.m01.empty.no_matches",
                variant_frame(structure::empty_block(
                    300.,
                    IconName::Search,
                    tr(cx, "gallery.m01.empty.no_matches_t"),
                    tr(cx, "gallery.m01.empty.no_matches_d"),
                    h_flex().child(atoms::button(
                        "e2a",
                        ButtonVariant::Ghost,
                        None,
                        tr(cx, "gallery.m01.empty.clear_filters"),
                        ControlSize::Medium,
                    )),
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.m01.empty.all_caught_up",
                variant_frame(structure::empty_block(
                    300.,
                    IconName::CircleCheck,
                    tr(cx, "gallery.m01.empty.caught_t"),
                    tr(cx, "gallery.m01.empty.caught_d"),
                    h_flex(),
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.m01.empty.offline",
                variant_frame(structure::empty_block(
                    300.,
                    IconName::CloudOff,
                    tr(cx, "gallery.m01.empty.offline_t"),
                    tr(cx, "gallery.m01.empty.offline_d"),
                    h_flex(),
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.m01.empty.error",
                variant_frame(structure::empty_block(
                    300.,
                    IconName::CircleAlert,
                    tr(cx, "gallery.m01.empty.error_t"),
                    tr(cx, "gallery.m01.empty.error_d"),
                    h_flex().child(atoms::button(
                        "e5a",
                        ButtonVariant::Secondary,
                        None,
                        tr(cx, "gallery.m01.empty.retry"),
                        ControlSize::Medium,
                    )),
                    mode,
                )),
            ),
        ]);
        let empty_group = kit::group(
            cx,
            "gallery.m01.empty.title_group",
            "gallery.m01.empty.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m01.empty.live", live_empty))
                .child(empty_variants),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.m01.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.m01.structure_title")))
            .child(kit::caption(cx, tr(cx, "gallery.m01.subtitle")))
            .child(tabs_group)
            .child(empty_group)
    }
}
