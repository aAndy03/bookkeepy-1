// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 01 sheet (part 3) — tabs, empty (references/designlanguage).
//!
//! FLAT on the page background. Live cells: interactive `TabBar` (underline with
//! icons + count badge, and segmented) and `Empty` blocks; the per-tab interaction
//! ladder is painted from `language::structure`.

use bookkeepy_ui::api::language::{
    atoms, controls::ControlSize, prim_hsla, role_hsla, structure, structure::TabIx,
};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::form::{field, v_form};
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::radio::Radio;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, ParentElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let payee = cx.new(|cx| InputState::new(window, cx).placeholder("Mega Image"));
    let amount = cx.new(|cx| InputState::new(window, cx).placeholder("0,00"));
    let note = cx.new(|cx| InputState::new(window, cx).placeholder("Add a note…"));
    cx.new(|_cx| MoleculesStructure {
        tab: 0,
        payee,
        amount,
        note,
    })
    .into()
}

pub struct MoleculesStructure {
    tab: usize,
    payee: Entity<InputState>,
    amount: Entity<InputState>,
    note: Entity<InputState>,
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
            .w(px(360.))
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Default)
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
                    None,
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
                    None,
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
                    role_hsla(mode, "success"),
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
                    None,
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
                    role_hsla(mode, "danger"),
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

        // ── O13 · form ───────────────────────────────────────────────────────
        let form_live = v_form()
            .w(px(440.))
            .gap(px(16.))
            .child(
                field()
                    .label(tr(cx, "gallery.o02.fm.payee"))
                    .child(Input::new(&self.payee)),
            )
            .child(
                field()
                    .label(tr(cx, "gallery.o02.fm.amount"))
                    .child(Input::new(&self.amount)),
            )
            .child(
                field()
                    .label(tr(cx, "gallery.o02.fm.note"))
                    .description(tr(cx, "gallery.o02.fm.note_help"))
                    .child(Input::new(&self.note)),
            )
            .child(
                field().label(tr(cx, "gallery.o02.fm.category")).child(
                    gpui_kit::component::radio::RadioGroup::horizontal("o2-fm-cat")
                        .child(Radio::new("groceries").label(tr(cx, "gallery.o02.fm.groceries")))
                        .child(Radio::new("transport").label(tr(cx, "gallery.o02.fm.transport"))),
                ),
            )
            .footer(
                h_flex()
                    .justify_end()
                    .gap(px(8.))
                    .pt(px(8.))
                    .child(organisms_dlg_btn_ghost(tr(cx, "gallery.o01.dlg.cancel")))
                    .child(organisms_dlg_btn_primary(tr(cx, "gallery.o01.dlg.save"))),
            );
        let form_group = kit::group(
            cx,
            "gallery.o02.fm.title",
            "gallery.o02.fm.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.fm.live", form_live)),
        );

        // ── O14 · group box ──────────────────────────────────────────────────
        let gb_live = h_flex().gap_6().items_start().children([
            GroupBox::new()
                .fill()
                .title(tr(cx, "gallery.o02.gb.sync"))
                .w(px(440.))
                .p(px(16.))
                .child(v_flex().gap(px(12.)).child(gb_row(
                    tr(cx, "gallery.o02.gb.auto"),
                    tr(cx, "gallery.o02.gb.auto_d"),
                    true,
                    mode,
                )))
                .footer(tr(cx, "gallery.o02.gb.footer")),
            GroupBox::new()
                .outline()
                .title(tr(cx, "gallery.o02.gb.privacy"))
                .w(px(440.))
                .p(px(16.))
                .child(v_flex().gap(px(12.)).child(gb_row(
                    tr(cx, "gallery.o02.gb.private"),
                    tr(cx, "gallery.o02.gb.private_d"),
                    false,
                    mode,
                ))),
        ]);
        let gb_group = kit::group(
            cx,
            "gallery.o02.gb.title",
            "gallery.o02.gb.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.gb.live", gb_live)),
        );

        // ── O26 · questionnaire ─────────────────────────────────────────────
        let q_choice = |n: &str,
                        title: gpui_kit::SharedString,
                        desc: gpui_kit::SharedString,
                        sel: bool,
                        mode: bookkeepy_primary_data::theme::Mode| {
            let bc = if sel {
                prim_hsla("neutral-0")
            } else {
                prim_hsla("white-a0")
            };
            h_flex()
                .items_center()
                .gap(px(12.))
                .p(px(10.))
                .pl(px(12.))
                .w(px(440.))
                .rounded(px(16.))
                .bg(prim_hsla("neutral-900").unwrap_or_default())
                .border(px(1.5))
                .border_color(bc.unwrap_or_default())
                .child(bookkeepy_ui::api::language::organisms::kbd_chip(n, mode))
                .child(
                    v_flex()
                        .flex_1()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(12.))
                                .line_height(px(14.))
                                .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .line_height(px(15.))
                                .text_color(prim_hsla("neutral-400").unwrap_or_default())
                                .child(desc),
                        ),
                )
                .child(bookkeepy_ui::api::language::choice::radio_circle(
                    sel,
                    bookkeepy_ui::api::language::choice::Ix::Rest,
                    mode,
                ))
        };
        let qn_live = v_flex()
            .w(px(480.))
            .p(px(20.))
            .gap(px(16.))
            .rounded(px(20.))
            .bg(prim_hsla("neutral-850").unwrap_or_default())
            .border_1()
            .border_color(prim_hsla("border-subtle").unwrap_or_default())
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(prim_hsla("neutral-400").unwrap_or_default())
                    .child(tr(cx, "gallery.o02.qn.step")),
            )
            .child(
                v_flex()
                    .w_full()
                    .h(px(4.))
                    .rounded_full()
                    .bg(prim_hsla("white-a8").unwrap_or_default())
                    .child(
                        div()
                            .w(px(220.))
                            .h_full()
                            .rounded_full()
                            .bg(prim_hsla("neutral-100").unwrap_or_default()),
                    ),
            )
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .child(tr(cx, "gallery.o02.qn.question")),
            )
            .child(
                v_flex()
                    .gap(px(8.))
                    .child(q_choice(
                        "1",
                        tr(cx, "gallery.o02.qn.c1"),
                        tr(cx, "gallery.o02.qn.c1d"),
                        true,
                        mode,
                    ))
                    .child(q_choice(
                        "2",
                        tr(cx, "gallery.o02.qn.c2"),
                        tr(cx, "gallery.o02.qn.c2d"),
                        false,
                        mode,
                    )),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(organisms_dlg_btn_ghost(tr(cx, "gallery.o02.qn.back")))
                    .child(organisms_dlg_btn_primary(tr(cx, "gallery.o02.qn.cont"))),
            );
        let qn_group = kit::group(
            cx,
            "gallery.o02.qn.title",
            "gallery.o02.qn.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.qn.live", qn_live)),
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
            .child(form_group)
            .child(gb_group)
            .child(qn_group)
    }
}

fn organisms_dlg_btn_ghost(label: gpui_kit::SharedString) -> Button {
    Button::new("o2-fm-cancel").label(label).ghost()
}

fn organisms_dlg_btn_primary(label: gpui_kit::SharedString) -> Button {
    Button::new("o2-fm-save").label(label).primary()
}

/// One group-box setting row: title + subtitle left, switch right (O14).
fn gb_row(
    title: gpui_kit::SharedString,
    desc: gpui_kit::SharedString,
    on: bool,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    h_flex()
        .items_center()
        .justify_between()
        .child(
            v_flex()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(14.))
                        .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(prim_hsla("neutral-400").unwrap_or_default())
                        .child(desc),
                ),
        )
        .child(bookkeepy_ui::api::language::choice::switch_track(
            on,
            false,
            bookkeepy_ui::api::language::choice::Ix::Rest,
            mode,
        ))
}
