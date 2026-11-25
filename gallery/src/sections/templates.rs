// SPDX-License-Identifier: GPL-3.0-or-later
//! TEMPLATES sheet — whole-app arrangements: shell, board, places. The shell chrome
//! itself is demoed in `organisms-shell` (title bar/sidebar/status bar); this section
//! shows the three template-level compositions from "TEMPLATES · shell, board, places".

use gpui_kit::component::grid_layout::{GridArea, GridItem, GridLayoutState, grid_layout};
use gpui_kit::component::label::Label;
use gpui_kit::component::sidebar::{Sidebar, SidebarGroup, SidebarMenuItem};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Side, Sizable as _, h_flex};
use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Styled as _, Window, div,
    px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The templates section.
pub struct Section {
    board: gpui_kit::Entity<GridLayoutState>,
}

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let board = cx.new(|cx| {
            GridLayoutState::new(cx).cols(12).default_areas([
                GridArea::new("tpl-cashflow").col_span(8).row_span(2),
                GridArea::new("tpl-invoices").col(8).col_span(4).row_span(2),
            ])
        });
        Section { board }
    })
    .into()
}

/// One board tile: the design's board cards are #1c1c1c rx16, borderless.
fn board_tile(cx: &gpui_kit::App, title_key: &'static str) -> gpui_kit::AnyElement {
    let mode = cx.global::<crate::app::Prefs>().mode;
    bookkeepy_ui::api::language::surface::card(
        div()
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .p_4()
            .child(div().text_sm().child(tr(cx, title_key))),
        mode,
    )
    .into_any_element()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let mode = cx.global::<crate::app::Prefs>().mode;
        let divider = bookkeepy_ui::api::language::token_hsla(
            &bookkeepy_primary_data::theme::colors(mode).border_heavy,
        )
        .unwrap_or(theme.border);
        section_page(
            cx,
            "gallery.section.templates",
            [
                demo_card(
                    cx,
                    "gallery.templates.shell.caption",
                    // TEMPLATES · shell: the app shell = title bar + 284px sidebar with
                    // the 2px divider + content slot + status bar (all demoed live in
                    // `organisms-shell`); here the mini composition.
                    demo_row([bookkeepy_ui::api::language::shell::sidebar(
                        Sidebar::new("tpl-sidebar")
                            .side(Side::Left)
                            .header(
                                div()
                                    .p_2()
                                    .child(Label::new(tr(cx, "gallery.templates.shell.workspace"))),
                            )
                            .child(
                                SidebarGroup::new(tr(cx, "gallery.templates.shell.nav")).child(
                                    SidebarMenuItem::new(tr(
                                        cx,
                                        "gallery.templates.shell.overview",
                                    ))
                                    .icon(Icon::new(IconName::Inbox).small())
                                    .active(true),
                                ),
                            ),
                    )
                    .h(px(140.))
                    .overflow_hidden()
                    .border_r_2()
                    .border_color(divider)
                    .into_any_element()])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.templates.board.caption",
                    // TEMPLATES · board: the dashboard grid (full interactive demo in
                    // `organisms-workspaces`); tiles are the #1c1c1c rx16 board cards.
                    div()
                        .h(px(160.))
                        .w_full()
                        .child(
                            grid_layout("gallery-templates-board", &self.board, cx)
                                .item(GridItem::new("tpl-cashflow").child(board_tile(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_cashflow",
                                )))
                                .item(GridItem::new("tpl-invoices").child(board_tile(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_invoices",
                                )))
                                .into_any_element(),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.templates.places.caption",
                    // TEMPLATES · places: saved places render as a list of labeled rows.
                    h_flex()
                        .gap_3()
                        .child(Label::new(tr(cx, "gallery.templates.places.home")))
                        .child(
                            Label::new(tr(cx, "gallery.templates.places.overview"))
                                .secondary(tr(cx, "gallery.templates.places.here")),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
