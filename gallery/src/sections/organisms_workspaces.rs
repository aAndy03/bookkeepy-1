// SPDX-License-Identifier: GPL-3.0-or-later
//! Organisms — workspaces: `DockArea` (via `DockSkin::dock_area`, styled dock panels
//! with draggable tab groups and splits — one minimal `Panel` implemented here) and
//! `GridLayout` (`grid_layout`, `GridLayoutState`, `GridArea`, `GridItem` — a grid
//! the user rearranges by dragging and resizing). Both keep their state entities
//! (`Entity<DockArea>`, `Entity<GridLayoutState>`) as retained fields.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::dock::{
    BasePanel, DockArea, DockLayout, DockSkin, Panel, PanelEvent, panel_handle,
};
use gpui_kit::component::grid_layout::{GridArea, GridItem, GridLayoutState, grid_layout};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// One dockable panel: a localized title and body (the minimum a styled dock panel
/// must implement for [`gpui_kit::component::dock`]).
struct DockPanel {
    focus_handle: FocusHandle,
    title_key: &'static str,
    body_key: &'static str,
}

impl DockPanel {
    fn new(title_key: &'static str, body_key: &'static str, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            title_key,
            body_key,
        }
    }
}

impl EventEmitter<PanelEvent> for DockPanel {}

impl Focusable for DockPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl BasePanel for DockPanel {
    /// Identifies the panel in persisted dock layouts.
    fn panel_name(&self) -> &'static str {
        "GalleryDockPanel"
    }
}

impl Panel for DockPanel {
    fn title(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tr(cx, self.title_key)
    }
}

impl gpui_kit::Render for DockPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p_4().child(tr(cx, self.body_key))
    }
}

/// The workspaces organism section.
pub struct Section {
    dock_area: Entity<DockArea>,
    grid: Entity<GridLayoutState>,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        // The styled dock area (the skin handle is unused here: defaults are kept).
        let (dock_area, _dock_skin) = DockSkin::dock_area("gallery-dock", Some(1), window, cx);

        let accounts = cx.new(|cx| {
            DockPanel::new(
                "gallery.organisms.workspaces.dock.panel_accounts",
                "gallery.organisms.workspaces.dock.body_accounts",
                cx,
            )
        });
        let ledger = cx.new(|cx| {
            DockPanel::new(
                "gallery.organisms.workspaces.dock.panel_ledger",
                "gallery.organisms.workspaces.dock.body_ledger",
                cx,
            )
        });
        let overview = cx.new(|cx| {
            DockPanel::new(
                "gallery.organisms.workspaces.dock.panel_overview",
                "gallery.organisms.workspaces.dock.body_overview",
                cx,
            )
        });

        // One split: the accounts panel beside a tab group of the other two.
        let center = DockLayout::h_split()
            .child(
                DockLayout::tabs().panel_view(panel_handle(accounts), cx),
                Some(px(240.)),
            )
            .child(
                DockLayout::tabs()
                    .panel_view(panel_handle(ledger), cx)
                    .panel_view(panel_handle(overview), cx),
                None,
            );
        dock_area.update(cx, |area, cx| area.set_center(center, window, cx));

        // The editable dashboard grid (12 columns by default).
        let grid = cx.new(|cx| {
            GridLayoutState::new(cx).cols(12).default_areas([
                GridArea::new("cashflow").col_span(8).row_span(4),
                GridArea::new("invoices").col(8).col_span(4).row_span(2),
                GridArea::new("goals").col(8).row(2).col_span(4).row_span(2),
            ])
        });

        Section { dock_area, grid }
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
            "gallery.section.organisms-workspaces",
            [demo_row([
                demo_card(
                    cx,
                    "gallery.organisms.workspaces.grid.caption",
                    grid_layout("gallery-grid-layout", &self.grid, cx)
                        .item(
                            GridItem::new("cashflow")
                                .rounded_2xl()
                                .accessibility_label(tr(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_cashflow",
                                ))
                                .child(grid_tile(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_cashflow",
                                )),
                        )
                        .item(
                            GridItem::new("invoices")
                                .rounded_2xl()
                                .accessibility_label(tr(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_invoices",
                                ))
                                .child(grid_tile(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_invoices",
                                )),
                        )
                        .item(
                            GridItem::new("goals")
                                .rounded_2xl()
                                .accessibility_label(tr(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_goals",
                                ))
                                .child(grid_tile(
                                    cx,
                                    "gallery.organisms.workspaces.grid.item_goals",
                                )),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.organisms.workspaces.dock.caption",
                    div()
                        .w_full()
                        .h(px(320.))
                        .child(self.dock_area.clone())
                        .into_any_element(),
                )
                .into_any_element(),
            ])
            .into_any_element()],
        )
    }
}

/// One dashboard tile: a small themed card with a localized title.
fn grid_tile(cx: &App, title_key: &'static str) -> gpui_kit::AnyElement {
    let theme = cx.theme();
    div()
        .size_full()
        .flex()
        .flex_col()
        .justify_center()
        .p_4()
        .bg(theme.muted)
        .child(div().text_sm().child(tr(cx, title_key)))
        .into_any_element()
}
