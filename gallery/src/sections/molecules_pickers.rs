// SPDX-License-Identifier: GPL-3.0-or-later
//! Molecules — pickers: Select, Combobox, DatePicker, Calendar, ColorPicker,
//! TimeField (design sheet: "MOLECULES · pickers"). State is retained in this
//! view (`SelectState`/`ComboboxState`/`DatePickerState`/`CalendarState`/
//! `ColorPickerState`/`TimeFieldState` are entities); every picker trigger and
//! every search field gets a localized placeholder (RULE-UI-005).

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::calendar::{Calendar, CalendarState};
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerState};
use gpui_kit::component::combobox::{Combobox, ComboboxState};
use gpui_kit::component::date_picker::{DatePicker, DatePickerState};
use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::component::select::{Select, SelectState};
use gpui_kit::component::time_field::{TimeField, TimeFieldState};
use gpui_kit::{AnyView, AppContext as _, Context, Entity, IntoElement, SharedString, Window};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// State of the entry-kind select (localizable items need `SharedString`).
type KindState = SelectState<Vec<SharedString>>;
/// State of the tag combobox (grouped-item delegate over localized strings).
type TagState = ComboboxState<SearchableVec<SharedString>>;

/// The picker-molecule section.
pub struct Section {
    kind: Entity<KindState>,
    tags: Entity<TagState>,
    issue_date: Entity<DatePickerState>,
    month: Entity<CalendarState>,
    time: Entity<TimeFieldState>,
    category: Entity<ColorPickerState>,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let kinds = vec![
            tr(cx, "gallery.molecules.pickers.select.option_income"),
            tr(cx, "gallery.molecules.pickers.select.option_expense"),
            tr(cx, "gallery.molecules.pickers.select.option_transfer"),
        ];
        let kind = cx.new(|cx| SelectState::new(kinds, None, window, cx).searchable(true));
        let tags = vec![
            tr(cx, "gallery.molecules.pickers.combobox.option_invoice"),
            tr(cx, "gallery.molecules.pickers.combobox.option_payroll"),
            tr(cx, "gallery.molecules.pickers.combobox.option_subscription"),
            tr(cx, "gallery.molecules.pickers.combobox.option_utilities"),
        ];
        let tags = cx.new(|cx| {
            ComboboxState::new(SearchableVec::new(tags), Vec::new(), window, cx)
                .multiple(true)
                .searchable(true)
        });
        let issue_date = cx.new(|cx| DatePickerState::new(window, cx));
        let month = cx.new(|cx| CalendarState::new(window, cx));
        let time = cx.new(|cx| TimeFieldState::new(window, cx));
        let category =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cx.theme().primary));
        Section {
            kind,
            tags,
            issue_date,
            month,
            time,
            category,
        }
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        section_page(
            cx,
            "gallery.section.molecules-pickers",
            [
                // List-style pickers.
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.select.caption",
                        Select::new(&self.kind)
                            .placeholder(tr(cx, "gallery.molecules.pickers.select.placeholder"))
                            .search_placeholder(tr(
                                cx,
                                "gallery.molecules.pickers.select.search_placeholder",
                            ))
                            .cleanable(true)
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.combobox.caption",
                        Combobox::new(&self.tags)
                            .placeholder(tr(cx, "gallery.molecules.pickers.combobox.placeholder"))
                            .search_placeholder(tr(
                                cx,
                                "gallery.molecules.pickers.combobox.search_placeholder",
                            ))
                            .cleanable(true)
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.color_picker.caption",
                        ColorPicker::new(&self.category)
                            .label(tr(cx, "gallery.molecules.pickers.color_picker.label"))
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                // Date-family pickers.
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.date_picker.caption",
                        DatePicker::new(&self.issue_date)
                            .placeholder(tr(
                                cx,
                                "gallery.molecules.pickers.date_picker.placeholder",
                            ))
                            .cleanable(true)
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.calendar.caption",
                        Calendar::new(&self.month).into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.time_field.caption",
                        TimeField::new(&self.time).into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
            ],
        )
    }
}
