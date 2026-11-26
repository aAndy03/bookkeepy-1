// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 01 sheet (part 2) — select, combobox, date picker
//! (references/designlanguage).
//!
//! FLAT on the page background. Live cells: `Select`, `Combobox` (type to filter,
//! click to open), `DatePicker` trigger + a live `Calendar`. Open-dropdown and
//! day-cell pseudo-states are painted from `language::popups` / `language::pickers`.

use bookkeepy_ui::api::language::{
    atoms, pickers, pickers::DayIx, popups, popups::TriggerIx, prim_hsla, role_hsla,
};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::IndexPath;
use gpui_kit::component::StyledExt as _;
use gpui_kit::component::calendar::{Calendar, CalendarState};
use gpui_kit::component::combobox::{Combobox, ComboboxState};
use gpui_kit::component::date_picker::{DatePicker, DatePickerState};
use gpui_kit::component::searchable_list::SearchableListItem;
use gpui_kit::component::select::{SearchableVec, Select, SelectState};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, Hsla, IntoElement, ParentElement as _, SharedString,
    Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// A select/combobox row model with a glyph and an optional category dot.
#[derive(Clone)]
struct Account {
    name: &'static str,
    icon: IconName,
    dot: Option<Hsla>,
    icon_color: Hsla,
}

impl SearchableListItem for Account {
    type Value = &'static str;
    fn title(&self) -> SharedString {
        self.name.into()
    }
    fn value(&self) -> &Self::Value {
        &self.name
    }
    fn render(&self, _window: &mut gpui_kit::Window, _cx: &mut gpui_kit::App) -> impl IntoElement {
        let mut row = h_flex().gap(px(8.)).items_center();
        if let Some(d) = self.dot {
            row = row.child(div().size(px(8.)).rounded_full().bg(d));
        } else {
            row = row.child(atoms::icon(self.icon, 14.).text_color(self.icon_color));
        }
        row.child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .child(self.name),
        )
    }
}

fn accounts(mode: bookkeepy_primary_data::theme::Mode) -> Vec<Account> {
    let icon_color = role_hsla(mode, "icon").unwrap_or_default();
    let mk = |name: &'static str, icon: IconName, dot: Option<Hsla>| Account {
        name,
        icon,
        dot,
        icon_color,
    };
    vec![
        mk("BT Pay", IconName::Landmark, None),
        mk("Revolut", IconName::Wallet, None),
        mk("Cash", IconName::Wallet, None),
        mk("Savings", IconName::PiggyBank, None),
        mk("groceries", IconName::Tag, prim_hsla("cyan-500")),
        mk("transport", IconName::Tag, prim_hsla("lime-500")),
        mk("fun", IconName::Tag, prim_hsla("pink-500")),
        mk("grants", IconName::Tag, prim_hsla("green-500")),
    ]
}

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let mode = cx.global::<Prefs>().mode;
    let select_state = cx.new(|cx| {
        SelectState::new(
            SearchableVec::new(accounts(mode).into_iter().take(4).collect::<Vec<_>>()),
            Some(IndexPath::default()),
            window,
            cx,
        )
        .searchable(true)
    });
    let combo_state = cx.new(|cx| {
        ComboboxState::new(
            SearchableVec::new(accounts(mode).into_iter().skip(4).collect::<Vec<_>>()),
            vec![IndexPath::new(0), IndexPath::new(2)],
            window,
            cx,
        )
        .multiple(true)
        .searchable(true)
    });
    let date_state = cx.new(|cx| DatePickerState::new(window, cx));
    date_state.update(cx, |s, cx| {
        s.set_date(jiff::civil::date(2026, 10, 21), window, cx);
    });
    let cal_single = cx.new(|cx| CalendarState::new(window, cx));
    cal_single.update(cx, |s, cx| {
        s.set_date(jiff::civil::date(2026, 10, 21), window, cx);
    });
    let cal_range = cx.new(|cx| CalendarState::new(window, cx));
    cal_range.update(cx, |s, cx| {
        s.set_date(
            (
                jiff::civil::date(2026, 10, 10),
                jiff::civil::date(2026, 10, 20),
            ),
            window,
            cx,
        );
    });
    cx.new(|_cx| MoleculesPickers {
        select_state,
        combo_state,
        date_state,
        cal_single,
        cal_range,
    })
    .into()
}

pub struct MoleculesPickers {
    select_state: Entity<SelectState<SearchableVec<Account>>>,
    combo_state: Entity<ComboboxState<SearchableVec<Account>>>,
    date_state: Entity<DatePickerState>,
    cal_single: Entity<CalendarState>,
    cal_range: Entity<CalendarState>,
}

impl gpui_kit::Render for MoleculesPickers {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── select ───────────────────────────────────────────────────────────
        let select_live = Select::new(&self.select_state)
            .placeholder(tr(cx, "gallery.m01.select.placeholder"))
            .w(px(240.));
        let select_triggers = kit::row([
            kit::cell(
                cx,
                "gallery.m01.select.placeholder_cell",
                popups::select_trigger(None, None, 200., TriggerIx::Rest, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                popups::select_trigger(
                    None,
                    Some(tr(cx, "gallery.m01.select.value")),
                    200.,
                    TriggerIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                popups::select_trigger(
                    None,
                    Some(tr(cx, "gallery.m01.select.value")),
                    200.,
                    TriggerIx::Hover,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.select.open",
                popups::select_trigger(
                    Some((IconName::Landmark, true)),
                    Some(tr(cx, "gallery.m01.select.value")),
                    240.,
                    TriggerIx::Open,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                popups::select_trigger(
                    None,
                    Some(tr(cx, "gallery.m01.select.value")),
                    200.,
                    TriggerIx::Disabled,
                    mode,
                ),
            ),
            kit::cell(cx, "gallery.m01.select.live", select_live),
        ]);
        // Painted open lists: plain, searchable with groups, empty + create.
        let plain_list = popups::popup_surface(240., 4., mode)
            .flex()
            .flex_col()
            .child(popups::menu_item(
                "BT Pay",
                Some(IconName::Landmark),
                false,
                None,
                popups::MenuIx::Rest,
                mode,
            ))
            .child(popups::menu_item(
                "Revolut",
                Some(IconName::Wallet),
                true,
                None,
                popups::MenuIx::Rest,
                mode,
            ))
            .child(popups::menu_item(
                "Cash",
                Some(IconName::Wallet),
                false,
                None,
                popups::MenuIx::Hover,
                mode,
            ))
            .child(popups::menu_item(
                "Savings",
                Some(IconName::PiggyBank),
                false,
                None,
                popups::MenuIx::Rest,
                mode,
            ));
        let grouped_list = popups::popup_surface(260., 4., mode)
            .flex()
            .flex_col()
            .child(popups::search_row(tr(cx, "gallery.m01.select.query"), mode))
            .child(popups::menu_label(
                tr(cx, "gallery.m01.select.group_spending"),
                mode,
            ))
            .child(popups::menu_item(
                "groceries",
                Some(IconName::Tag),
                true,
                None,
                popups::MenuIx::Rest,
                mode,
            ))
            .child(popups::menu_item(
                "gifts",
                Some(IconName::Tag),
                false,
                None,
                popups::MenuIx::Hover,
                mode,
            ))
            .child(popups::menu_label(
                tr(cx, "gallery.m01.select.group_income"),
                mode,
            ))
            .child(popups::menu_item(
                "grants",
                Some(IconName::Tag),
                false,
                None,
                popups::MenuIx::Rest,
                mode,
            ));
        let empty_list = popups::popup_surface(260., 4., mode)
            .flex()
            .flex_col()
            .child(popups::search_row("Glov", mode))
            .child(popups::empty_row(
                tr(cx, "gallery.m01.select.no_match"),
                mode,
            ))
            .child(popups::menu_item(
                tr(cx, "gallery.m01.select.create"),
                Some(IconName::Plus),
                false,
                None,
                popups::MenuIx::Hover,
                mode,
            ));
        let select_open = kit::row([
            kit::cell(cx, "gallery.m01.select.list_plain", plain_list),
            kit::cell(cx, "gallery.m01.select.list_search", grouped_list),
            kit::cell(cx, "gallery.m01.select.list_empty", empty_list),
        ]);
        let select_group = kit::group(
            cx,
            "gallery.m01.select.title",
            "gallery.m01.select.hint",
            v_flex().gap_6().child(select_triggers).child(select_open),
        );

        // ── combobox ─────────────────────────────────────────────────────────
        let chips = |with_query: bool| {
            let mut field = h_flex()
                .w_full()
                .h(px(32.))
                .px(px(10.))
                .gap(px(8.))
                .items_center()
                .rounded(px(bookkeepy_ui::api::language::radius_tier("md")))
                .bg(prim_hsla("neutral-950").unwrap_or_default());
            field = field.child(
                h_flex()
                    .gap(px(4.))
                    .child(popups::chip(
                        tr(cx, "gallery.m01.combo.chip_groceries"),
                        prim_hsla("cyan-500"),
                        mode,
                    ))
                    .child(popups::chip(
                        tr(cx, "gallery.m01.combo.chip_fun"),
                        prim_hsla("pink-500"),
                        mode,
                    ))
                    .child(popups::chip(
                        tr(cx, "gallery.m01.combo.chip_more"),
                        None,
                        mode,
                    )),
            );
            let (text, color) = if with_query {
                (SharedString::from("g"), prim_hsla("neutral-0"))
            } else {
                (
                    tr(cx, "gallery.m01.combo.placeholder"),
                    prim_hsla("neutral-500"),
                )
            };
            field = field
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(14.))
                        .text_color(color.unwrap_or_default())
                        .child(text),
                )
                .child(div().flex_1())
                .child(
                    atoms::icon(IconName::ChevronsUpDown, 12.)
                        .text_color(role_hsla(mode, "icon").unwrap_or_default()),
                );
            div()
                .w(px(320.))
                .p(px(2.))
                .rounded(px(bookkeepy_ui::api::language::radius_tier("lg")))
                .border_2()
                .border_color(if with_query {
                    prim_hsla("white-a16").unwrap_or_default()
                } else {
                    gpui_kit::transparent_white()
                })
                .child(field)
        };
        let combo_live = Combobox::new(&self.combo_state)
            .placeholder(tr(cx, "gallery.m01.combo.placeholder"))
            .search_placeholder(tr(cx, "gallery.m01.combo.search_ph"))
            .w(px(320.));
        let multi_list = popups::popup_surface(320., 4., mode)
            .flex()
            .flex_col()
            .child(popups::search_row(
                tr(cx, "gallery.m01.combo.search_ph"),
                mode,
            ))
            .child(combo_row(
                "groceries",
                true,
                prim_hsla("cyan-500"),
                false,
                mode,
            ))
            .child(combo_row(
                "transport",
                false,
                prim_hsla("lime-500"),
                true,
                mode,
            ))
            .child(combo_row("fun", true, prim_hsla("pink-500"), false, mode));
        let combo_triggers = kit::row([
            kit::cell(
                cx,
                "gallery.m01.combo.empty",
                popups::select_trigger(None, None, 320., TriggerIx::Rest, mode),
            ),
            kit::cell(cx, "gallery.m01.combo.chips", chips(false)),
            kit::cell(cx, "gallery.m01.combo.typing", chips(true)),
            kit::cell(cx, "gallery.m01.combo.live", combo_live),
        ]);
        let combo_group = kit::group(
            cx,
            "gallery.m01.combo.title",
            "gallery.m01.combo.hint",
            v_flex().gap_6().child(combo_triggers).child(kit::cell(
                cx,
                "gallery.m01.combo.open_multi",
                multi_list,
            )),
        );

        // ── date picker ──────────────────────────────────────────────────────
        let day_axis = kit::row(
            [
                ("gallery.m01.dp.default", 12, DayIx::Default),
                ("gallery.m01.dp.today", 18, DayIx::Today),
                ("gallery.m01.dp.selected", 21, DayIx::Selected),
                ("gallery.m01.dp.range_middle", 14, DayIx::RangeMiddle),
                ("gallery.m01.dp.outside", 29, DayIx::Outside),
                ("gallery.m01.dp.disabled", 25, DayIx::Disabled),
                ("gallery.m01.dp.hover", 13, DayIx::Hover),
            ]
            .into_iter()
            .map(|(key, n, ix)| {
                kit::cell(
                    cx,
                    key,
                    v_flex()
                        .gap_1()
                        .child(pickers::day_cell(n, ix, mode))
                        .child(kit::caption(cx, tr(cx, key))),
                )
            }),
        );
        let dp_triggers = kit::row([
            kit::cell(
                cx,
                "gallery.m01.dp.rest",
                pickers::date_trigger(
                    Some(tr(cx, "gallery.m01.dp.value")),
                    240.,
                    TriggerIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.dp.open",
                pickers::date_trigger(
                    Some(tr(cx, "gallery.m01.dp.value")),
                    240.,
                    TriggerIx::Open,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.dp.live",
                DatePicker::new(&self.date_state).w(px(240.)),
            ),
        ]);
        // Live calendars inside the painted popup frame; range gets a presets rail.
        let cal_single = pickers::calendar_popup(mode).child(Calendar::new(&self.cal_single));
        let presets_rail = v_flex()
            .h(px(148.))
            .p(px(4.))
            .border_r(px(1.))
            .border_color(prim_hsla("white-a8").unwrap_or_default())
            .child(pickers::preset_row(
                tr(cx, "gallery.m01.dp.preset_month"),
                false,
                mode,
            ))
            .child(pickers::preset_row(
                tr(cx, "gallery.m01.dp.preset_last_month"),
                false,
                mode,
            ))
            .child(pickers::preset_row(
                tr(cx, "gallery.m01.dp.preset_30"),
                true,
                mode,
            ))
            .child(pickers::preset_row(
                tr(cx, "gallery.m01.dp.preset_year"),
                false,
                mode,
            ))
            .child(pickers::preset_row(
                tr(cx, "gallery.m01.dp.preset_custom"),
                false,
                mode,
            ));
        let cal_range = h_flex()
            .rounded(px(bookkeepy_ui::api::language::radius_tier("xl")))
            .border_1()
            .border_color(role_hsla(mode, "border-subtle").unwrap_or_default())
            .bg(role_hsla(mode, "popover").unwrap_or_default())
            .refine_style(&gpui_kit::StyleRefinement {
                box_shadow: Some(popups::popup_shadow()),
                ..Default::default()
            })
            .child(presets_rail)
            .child(
                pickers::calendar_popup(mode)
                    .border_0()
                    .child(Calendar::new(&self.cal_range)),
            );
        let dp_group = kit::group(
            cx,
            "gallery.m01.dp.title",
            "gallery.m01.dp.hint",
            v_flex()
                .gap_6()
                .child(day_axis)
                .child(dp_triggers)
                .child(kit::row([
                    kit::cell(cx, "gallery.m01.dp.calendar_single", cal_single),
                    kit::cell(cx, "gallery.m01.dp.calendar_range", cal_range),
                ])),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.m01.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.m01.pickers_title")))
            .child(kit::caption(cx, tr(cx, "gallery.m01.subtitle")))
            .child(select_group)
            .child(combo_group)
            .child(dp_group)
    }
}

/// One combobox dropdown row: checkbox + optional dot + label.
fn combo_row(
    label: &'static str,
    checked: bool,
    dot: Option<Hsla>,
    hover: bool,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let box_on = bookkeepy_ui::api::language::choice::checkbox_box(
        if checked {
            bookkeepy_ui::api::language::choice::Tri::On
        } else {
            bookkeepy_ui::api::language::choice::Tri::Off
        },
        bookkeepy_ui::api::language::choice::Ix::Rest,
        mode,
    );
    let mut row = h_flex()
        .w_full()
        .h(px(28.))
        .px(px(8.))
        .gap(px(8.))
        .items_center()
        .rounded(px(bookkeepy_ui::api::language::radius_tier("md")));
    if hover {
        row = row.bg(prim_hsla("white-a4").unwrap_or_default());
    }
    row = row.child(box_on);
    if let Some(d) = dot {
        row = row.child(div().size(px(8.)).rounded_full().bg(d));
    }
    row.child(
        div()
            .text_size(px(12.))
            .line_height(px(14.))
            .text_color(prim_hsla("neutral-0").unwrap_or_default())
            .child(label),
    )
}
