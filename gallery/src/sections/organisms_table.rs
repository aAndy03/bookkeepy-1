// SPDX-License-Identifier: GPL-3.0-or-later
//! Organisms — tables and trees: the stateless `Table` family (Table, TableHeader,
//! TableBody, TableFooter, TableRow, TableHead, TableCell, TableCaption), `DataTable`
//! (via `TableDelegate`/`TableState`/`Column` — virtual scrolling, stripes, sorting),
//! the hierarchical `Tree` (`tree`, `TreeState`, `TreeItem`) and `VirtualList`
//! (`v_virtual_list` — a large dataset with only the visible rows rendered).
//! All demo text goes through `tr`; the `DataTable` keeps its `TableState` and the
//! tree keeps its `TreeState` as retained entities.

use std::rc::Rc;

use gpui_kit::component::list::ListItem;
use gpui_kit::component::table::{
    Column, DataTable, Table, TableBody, TableCaption, TableCell, TableDelegate, TableFooter,
    TableHead, TableHeader, TableRow, TableState,
};
use gpui_kit::component::tree::{TreeItem, TreeState, tree};
use gpui_kit::component::{VirtualListScrollHandle, v_virtual_list};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Pixels, Size,
    Styled as _, Window, div, px, size,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// A sample invoice row, as (document, client, amount, status) i18n keys.
type RowKeys = (&'static str, &'static str, &'static str, &'static str);

/// The sample invoices every table in this section shows (labels live in i18n keys).
const ROWS: &[RowKeys] = &[
    (
        "gallery.organisms.table.data.doc_1",
        "gallery.organisms.table.data.client_1",
        "gallery.organisms.table.data.amount_1",
        "gallery.organisms.table.data.status_paid",
    ),
    (
        "gallery.organisms.table.data.doc_2",
        "gallery.organisms.table.data.client_2",
        "gallery.organisms.table.data.amount_2",
        "gallery.organisms.table.data.status_pending",
    ),
    (
        "gallery.organisms.table.data.doc_3",
        "gallery.organisms.table.data.client_3",
        "gallery.organisms.table.data.amount_3",
        "gallery.organisms.table.data.status_overdue",
    ),
];

/// `DataTable` delegate over the localized sample invoices.
struct InvoicesDelegate {
    columns: Vec<Column>,
}

impl TableDelegate for InvoicesDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        ROWS.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = ROWS[row_ix];
        match col_ix {
            0 => tr(cx, row.0),
            1 => tr(cx, row.1),
            2 => tr(cx, row.2),
            _ => tr(cx, row.3),
        }
    }
}

/// The tables/trees organism section.
pub struct Section {
    invoices: Entity<TableState<InvoicesDelegate>>,
    accounts: Entity<TreeState>,
    /// One `Size` per `VirtualList` row (kept out of `render` — pre-calculated).
    entries: Rc<Vec<Size<Pixels>>>,
    entries_scroll: VirtualListScrollHandle,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let columns = vec![
            Column::new("doc", tr(cx, "gallery.organisms.table.col_doc")),
            Column::new("client", tr(cx, "gallery.organisms.table.col_client")),
            Column::new("amount", tr(cx, "gallery.organisms.table.col_amount"))
                .text_right()
                .sortable(),
            Column::new("status", tr(cx, "gallery.organisms.table.col_status")),
        ];
        let invoices = cx.new(|cx| {
            TableState::new(InvoicesDelegate { columns }, window, cx).row_selectable(true)
        });
        let accounts = cx.new(|cx| {
            TreeState::new(cx).items(vec![
                TreeItem::new(
                    "accounts-assets",
                    tr(cx, "gallery.organisms.table.tree.assets"),
                )
                .expanded(true)
                .child(TreeItem::new(
                    "accounts-bank",
                    tr(cx, "gallery.organisms.table.tree.bank"),
                ))
                .child(TreeItem::new(
                    "accounts-receivables",
                    tr(cx, "gallery.organisms.table.tree.receivables"),
                )),
                TreeItem::new(
                    "accounts-liabilities",
                    tr(cx, "gallery.organisms.table.tree.liabilities"),
                )
                .expanded(true)
                .child(TreeItem::new(
                    "accounts-suppliers",
                    tr(cx, "gallery.organisms.table.tree.suppliers"),
                )),
            ])
        });
        let entries = Rc::new(
            (0..2000)
                .map(|_| size(px(600.), px(28.)))
                .collect::<Vec<_>>(),
        );
        let entries_scroll = VirtualListScrollHandle::new();
        Section {
            invoices,
            accounts,
            entries,
            entries_scroll,
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
            "gallery.section.organisms-table",
            [
                demo_row([
                    demo_card(
                        cx,
                        "gallery.organisms.table.table.caption",
                        static_invoices_table(cx).into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.organisms.table.tree.caption",
                        div()
                            .h(px(220.))
                            .child(tree(&self.accounts, |ix, entry, selected, _window, _cx| {
                                ListItem::new(ix)
                                    .selected(selected)
                                    .pl(px(16.) * entry.depth() + px(12.))
                                    .child(entry.item().label.clone())
                            }))
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.organisms.table.data_table.caption",
                    div()
                        .h(px(240.))
                        .child(
                            DataTable::new(&self.invoices)
                                .stripe(true)
                                .accessibility_label(tr(
                                    cx,
                                    "gallery.organisms.table.data_table.a11y",
                                )),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.organisms.table.virtual_list.caption",
                    div()
                        .h(px(240.))
                        .child(
                            v_virtual_list(
                                cx.entity().clone(),
                                "virtual-list-demo",
                                self.entries.clone(),
                                |_view, visible_range, _window, cx| {
                                    let label = tr(cx, "gallery.organisms.table.virtual_list.item");
                                    visible_range
                                        .map(|ix| {
                                            div()
                                                .h(px(28.))
                                                .w_full()
                                                .flex()
                                                .items_center()
                                                .child(format!("{} {}", label, ix + 1))
                                        })
                                        .collect::<Vec<_>>()
                                },
                            )
                            .track_scroll(&self.entries_scroll),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}

/// The stateless `Table` demo: the same invoices, composed part by part.
fn static_invoices_table(cx: &App) -> Table {
    let mut body = TableBody::new();
    for row in ROWS {
        let data_row = TableRow::new()
            .child(TableCell::new().child(tr(cx, row.0)))
            .child(TableCell::new().child(tr(cx, row.1)))
            .child(TableCell::new().text_right().child(tr(cx, row.2)))
            .child(TableCell::new().child(tr(cx, row.3)));
        body = body.child(data_row);
    }
    Table::new()
        .child(
            TableHeader::new().child(
                TableRow::new()
                    .child(TableHead::new().child(tr(cx, "gallery.organisms.table.col_doc")))
                    .child(TableHead::new().child(tr(cx, "gallery.organisms.table.col_client")))
                    .child(
                        TableHead::new()
                            .text_right()
                            .child(tr(cx, "gallery.organisms.table.col_amount")),
                    )
                    .child(TableHead::new().child(tr(cx, "gallery.organisms.table.col_status"))),
            ),
        )
        .child(body)
        .child(
            TableFooter::new().child(
                TableRow::new()
                    .child(TableCell::new().child(tr(cx, "gallery.organisms.table.total")))
                    .child(TableCell::new())
                    .child(
                        TableCell::new()
                            .text_right()
                            .child(tr(cx, "gallery.organisms.table.total_amount")),
                    )
                    .child(TableCell::new()),
            ),
        )
        .child(TableCaption::new().child(tr(cx, "gallery.organisms.table.table.caption_text")))
}
