// SPDX-License-Identifier: GPL-3.0-or-later
//! ORGANISMS · 01 sheet (part 3) — data table, list, virtual list
//! (references/designlanguage ORGANISMS · 01, blocks O6–O8).
//!
//! FLAT on the page background. O6/O7 have LIVE `DataTable` / `List` cells
//! (select rows, sort columns, scroll) next to painted masters that pin the
//! sheet's state axis (hover/selected/focus bar, day groups, footer, skeleton,
//! empty). O8 is a live `v_virtual_list` with mixed row heights.

use std::rc::Rc;

use bookkeepy_ui::api::language::choice::{self, Ix as ChkIx, Tri};
use bookkeepy_ui::api::language::organisms::{self, LiIx, SortIx, TrIx};
use bookkeepy_ui::api::language::{prim_hsla, role_hsla};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::list::{List, ListDelegate, ListItem, ListState};
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::component::{IndexPath, v_virtual_list};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, Hsla, IntoElement, ParentElement as _, Size,
    Styled as _, Subscription, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let table_state = cx.new(|cx| {
        TableState::new(TxDelegate::new(), window, cx)
            .row_selectable(true)
            .sortable(true)
    });
    let list_state = cx.new(|cx| ListState::new(AcctDelegate, window, cx).selectable(true));
    let vlist_sizes: Rc<Vec<Size<gpui_kit::Pixels>>> = Rc::new(
        (0..12_480)
            .map(|i| {
                let h = match i % 5 {
                    0 => 28., // day group header
                    3 => 56., // note row
                    _ => 36., // plain row
                };
                Size {
                    width: px(0.),
                    height: px(h),
                }
            })
            .collect(),
    );
    cx.new(|_cx| OrganismsTable {
        table_state,
        list_state,
        vlist_sizes,
        _subs: Vec::new(),
    })
    .into()
}

pub struct OrganismsTable {
    table_state: Entity<TableState<TxDelegate>>,
    list_state: Entity<ListState<AcctDelegate>>,
    vlist_sizes: Rc<Vec<Size<gpui_kit::Pixels>>>,
    _subs: Vec<Subscription>,
}

// ── O6 · data table rows ─────────────────────────────────────────────────────

struct TxRow {
    time: &'static str,
    payee: &'static str,
    cat: &'static str,
    cat_dot: &'static str,
    cat_label: &'static str,
    account: &'static str,
    dir: IconName,
    amount: &'static str,
    amt_color: Hsla,
}

struct TxDelegate {
    rows: Vec<TxRow>,
}

impl TxDelegate {
    fn new() -> Self {
        let w = prim_hsla("neutral-0").unwrap_or_default();
        let g = prim_hsla("green-500").unwrap_or_default();
        let m = prim_hsla("neutral-400").unwrap_or_default();
        let a = prim_hsla("amber-500").unwrap_or_default();
        Self {
            rows: vec![
                TxRow {
                    time: "14:32",
                    payee: "Mega Image",
                    cat: "groceries",
                    cat_dot: "cyan-500",
                    cat_label: "neutral-300",
                    account: "BT Pay",
                    dir: IconName::ArrowUpRight,
                    amount: "−142,50",
                    amt_color: w,
                },
                TxRow {
                    time: "09:10",
                    payee: "Uber",
                    cat: "transport",
                    cat_dot: "lime-500",
                    cat_label: "neutral-300",
                    account: "Revolut",
                    dir: IconName::ArrowUpRight,
                    amount: "−23,00",
                    amt_color: w,
                },
                TxRow {
                    time: "08:00",
                    payee: "Salary · Acme SRL",
                    cat: "income",
                    cat_dot: "green-500",
                    cat_label: "neutral-300",
                    account: "BT Pay",
                    dir: IconName::ArrowDownLeft,
                    amount: "+8.400,00",
                    amt_color: g,
                },
                TxRow {
                    time: "07:45",
                    payee: "Lidl",
                    cat: "groceries",
                    cat_dot: "cyan-500",
                    cat_label: "neutral-300",
                    account: "Cash",
                    dir: IconName::ArrowUpRight,
                    amount: "−61,20",
                    amt_color: w,
                },
                TxRow {
                    time: "16:20",
                    payee: "Starbucks",
                    cat: "uncategorized",
                    cat_dot: "orange-500",
                    cat_label: "orange-300",
                    account: "Revolut",
                    dir: IconName::ArrowUpRight,
                    amount: "−18,00",
                    amt_color: w,
                },
                TxRow {
                    time: "15:11",
                    payee: "Transfer to Savings",
                    cat: "transfer",
                    cat_dot: "neutral-400",
                    cat_label: "neutral-400",
                    account: "BT Pay",
                    dir: IconName::ArrowLeftRight,
                    amount: "1.000,00",
                    amt_color: m,
                },
                TxRow {
                    time: "12:41",
                    payee: "Netflix",
                    cat: "subscriptions",
                    cat_dot: "violet-500",
                    cat_label: "neutral-300",
                    account: "Revolut",
                    dir: IconName::Clock,
                    amount: "−55,00",
                    amt_color: a,
                },
            ],
        }
    }
}

impl TableDelegate for TxDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        5
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        match col_ix {
            0 => Column::new("time", "Time").width(px(56.)),
            1 => Column::new("payee", "Payee").width(px(240.)),
            2 => Column::new("category", "Category").width(px(170.)),
            3 => Column::new("account", "Account").width(px(110.)),
            _ => Column::new("amount", "Amount")
                .width(px(140.))
                .text_right()
                .sortable()
                .descending(),
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        if col_ix == 4 {
            let num = |s: &str| -> f64 {
                s.chars()
                    .filter(|c| *c != '+' && *c != '.')
                    .map(|c| {
                        if c == '−' {
                            '-'
                        } else if c == ',' {
                            '.'
                        } else {
                            c
                        }
                    })
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0.)
            };
            match sort {
                ColumnSort::Ascending => self.rows.sort_by(|a, b| {
                    num(a.amount)
                        .partial_cmp(&num(b.amount))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                ColumnSort::Descending => self.rows.sort_by(|a, b| {
                    num(b.amount)
                        .partial_cmp(&num(a.amount))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                _ => {}
            }
        }
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.rows[row_ix];
        let mode = cx.global::<Prefs>().mode;
        match col_ix {
            0 => div()
                .w(px(56.))
                .text_size(px(11.))
                .text_color(prim_hsla("neutral-500").unwrap_or_default())
                .child(row.time),
            1 => h_flex()
                .gap(px(8.))
                .items_center()
                .child(
                    bookkeepy_ui::api::language::atoms::icon(row.dir, 14.)
                        .text_color(row.amt_color),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                        .child(row.payee),
                ),
            2 => h_flex()
                .w(px(170.))
                .gap(px(8.))
                .items_center()
                .child(
                    div()
                        .size(px(8.))
                        .rounded_full()
                        .bg(prim_hsla(row.cat_dot).unwrap_or_default()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(prim_hsla(row.cat_label).unwrap_or_default())
                        .child(row.cat),
                ),
            3 => div()
                .w(px(110.))
                .text_size(px(11.))
                .text_color(prim_hsla("neutral-400").unwrap_or_default())
                .child(row.account),
            _ => organisms::amount_trailing(row.amount.into(), "RON", row.amt_color),
        }
    }
}

// ── O7 · list ────────────────────────────────────────────────────────────────

struct AcctDelegate;

impl ListDelegate for AcctDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        5
    }

    fn set_selected_index(
        &mut self,
        _ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let mode = cx.global::<Prefs>().mode;
        let fg = role_hsla(mode, "foreground").unwrap_or_default();
        let muted = prim_hsla("neutral-400").unwrap_or_default();
        fn leading(inner: impl IntoElement) -> gpui_kit::Div {
            div()
                .size(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .child(inner)
        }
        let (lead, title, sub, trail) = match ix.row {
            0 => (
                leading(
                    bookkeepy_ui::api::language::atoms::icon(IconName::Landmark, 18.)
                        .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                ),
                "BT Pay",
                "current · …4021",
                organisms::amount_trailing("300,00".into(), "RON", fg),
            ),
            1 => (
                leading(
                    bookkeepy_ui::api::language::atoms::icon(IconName::Wallet, 18.)
                        .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                ),
                "Revolut",
                "current · EUR + RON",
                organisms::amount_trailing("20.967,00".into(), "RON", fg),
            ),
            2 => (
                leading(
                    bookkeepy_ui::api::language::atoms::icon(IconName::Wallet, 18.)
                        .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                ),
                "Cash",
                "wallet",
                organisms::amount_trailing("1.240,00".into(), "RON", fg),
            ),
            3 => (
                leading(
                    bookkeepy_ui::api::language::atoms::icon(IconName::PiggyBank, 18.)
                        .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                ),
                "Savings",
                "deposit · 4.1%",
                organisms::amount_trailing("25.813,00".into(), "RON", fg),
            ),
            _ => (
                leading(
                    bookkeepy_ui::api::language::atoms::icon(IconName::Landmark, 18.)
                        .text_color(muted),
                ),
                "Old ING",
                "closed in 2024",
                organisms::amount_trailing("0,00".into(), "RON", fg),
            ),
        };
        let _ = muted;
        Some(
            ListItem::new(ix.row).child(
                h_flex()
                    .w_full()
                    .h(px(48.))
                    .px(px(12.))
                    .gap(px(12.))
                    .items_center()
                    .child(lead)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                                    .text_color(fg)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(prim_hsla("neutral-400").unwrap_or_default())
                                    .child(sub),
                            ),
                    )
                    .child(trail),
            ),
        )
    }
}

// ── painted O6 master table ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct RowSpec {
    ix: TrIx,
    checked: bool,
    time: &'static str,
    dir: IconName,
    payee: &'static str,
    cat_dot: &'static str,
    cat: &'static str,
    cat_label: &'static str,
    account: &'static str,
    amount: &'static str,
    amt_color: Hsla,
}

fn painted_row(mode: bookkeepy_primary_data::theme::Mode, row: &RowSpec) -> gpui_kit::Div {
    let RowSpec {
        ix,
        checked,
        time,
        dir,
        payee,
        cat_dot,
        cat,
        cat_label,
        account,
        amount,
        amt_color,
    } = *row;
    let fg = role_hsla(mode, "foreground").unwrap_or_default();
    let muted = prim_hsla("neutral-400").unwrap_or_default();
    let muted2 = prim_hsla("neutral-500").unwrap_or_default();
    organisms::tr_base(ix, mode)
        .child(choice::checkbox_box(
            if checked { Tri::On } else { Tri::Off },
            ChkIx::Rest,
            mode,
        ))
        .child(
            div()
                .w(px(56.))
                .text_size(px(11.))
                .text_color(muted2)
                .child(time.to_string()),
        )
        .child(
            h_flex()
                .flex_1()
                .gap(px(8.))
                .items_center()
                .child(bookkeepy_ui::api::language::atoms::icon(dir, 14.).text_color(amt_color))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(fg)
                        .child(payee.to_string()),
                ),
        )
        .child(
            h_flex()
                .w(px(170.))
                .gap(px(8.))
                .items_center()
                .child(
                    div()
                        .size(px(8.))
                        .rounded_full()
                        .bg(prim_hsla(cat_dot).unwrap_or_default()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(prim_hsla(cat_label).unwrap_or_default())
                        .child(cat.to_string()),
                ),
        )
        .child(
            div()
                .w(px(110.))
                .text_size(px(11.))
                .text_color(muted)
                .child(account.to_string()),
        )
        .child(
            div()
                .w(px(140.))
                .flex()
                .justify_end()
                .child(organisms::amount_trailing(
                    amount.to_string().into(),
                    "RON",
                    amt_color,
                )),
        )
        .child(organisms::tr_actions(!matches!(ix, TrIx::Rest), mode))
}

fn painted_table(mode: bookkeepy_primary_data::theme::Mode) -> gpui_kit::Div {
    let w = prim_hsla("neutral-0").unwrap_or_default();
    let g = prim_hsla("green-500").unwrap_or_default();
    let m = prim_hsla("neutral-400").unwrap_or_default();
    let a = prim_hsla("amber-500").unwrap_or_default();
    organisms::table_frame(960., mode)
        // header row: indeterminate select-all + columns
        .child(
            h_flex()
                .h(px(32.))
                .px(px(12.))
                .gap(px(12.))
                .items_center()
                .border_b(px(1.))
                .border_color(prim_hsla("white-a8").unwrap_or_default())
                .child(choice::checkbox_box(Tri::Mixed, ChkIx::Rest, mode))
                .child(organisms::th("Time".into(), SortIx::None, mode))
                .child(
                    div()
                        .flex_1()
                        .child(organisms::th("Payee".into(), SortIx::None, mode)),
                )
                .child(div().w(px(170.)).child(organisms::th(
                    "Category".into(),
                    SortIx::None,
                    mode,
                )))
                .child(
                    div()
                        .w(px(110.))
                        .child(organisms::th("Account".into(), SortIx::None, mode)),
                )
                .child(h_flex().w(px(140.)).justify_end().child(organisms::th(
                    "Amount".into(),
                    SortIx::Desc,
                    mode,
                )))
                .child(div().w(px(24.))),
        )
        .child(organisms::day_header(
            "18 OCT · SUNDAY".into(),
            "−226,70 · +8.400,00".into(),
            mode,
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Rest,
                checked: false,
                time: "14:32",
                dir: IconName::ArrowUpRight,
                payee: "Mega Image",
                cat_dot: "cyan-500",
                cat: "groceries",
                cat_label: "neutral-300",
                account: "BT Pay",
                amount: "−142,50",
                amt_color: w,
            },
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Hover,
                checked: false,
                time: "09:10",
                dir: IconName::ArrowUpRight,
                payee: "Uber",
                cat_dot: "lime-500",
                cat: "transport",
                cat_label: "neutral-300",
                account: "Revolut",
                amount: "−23,00",
                amt_color: w,
            },
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Selected,
                checked: true,
                time: "08:00",
                dir: IconName::ArrowDownLeft,
                payee: "Salary · Acme SRL",
                cat_dot: "green-500",
                cat: "income",
                cat_label: "neutral-300",
                account: "BT Pay",
                amount: "+8.400,00",
                amt_color: g,
            },
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Focus,
                checked: false,
                time: "07:45",
                dir: IconName::ArrowUpRight,
                payee: "Lidl",
                cat_dot: "cyan-500",
                cat: "groceries",
                cat_label: "neutral-300",
                account: "Cash",
                amount: "−61,20",
                amt_color: w,
            },
        ))
        .child(organisms::day_header(
            "17 OCT · SATURDAY".into(),
            "−440,99".into(),
            mode,
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Rest,
                checked: false,
                time: "16:20",
                dir: IconName::ArrowUpRight,
                payee: "Starbucks",
                cat_dot: "orange-500",
                cat: "uncategorized",
                cat_label: "orange-300",
                account: "Revolut",
                amount: "−18,00",
                amt_color: w,
            },
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Rest,
                checked: false,
                time: "15:11",
                dir: IconName::ArrowLeftRight,
                payee: "Transfer to Savings",
                cat_dot: "neutral-400",
                cat: "transfer",
                cat_label: "neutral-400",
                account: "BT Pay",
                amount: "1.000,00",
                amt_color: m,
            },
        ))
        .child(painted_row(
            mode,
            &RowSpec {
                ix: TrIx::Rest,
                checked: false,
                time: "12:41",
                dir: IconName::Clock,
                payee: "Netflix",
                cat_dot: "violet-500",
                cat: "subscriptions",
                cat_label: "neutral-300",
                account: "Revolut",
                amount: "−55,00",
                amt_color: a,
            },
        ))
        .child(organisms::load_more_row(
            "Loading older transactions…".into(),
            mode,
        ))
        .child(organisms::table_footer(
            "1 selected · 142 transactions in October".into(),
            "+2.140,00".into(),
            mode,
        ))
}

impl gpui_kit::Render for OrganismsTable {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── O6 · data table ──────────────────────────────────────────────────
        let live_table = div()
            .w(px(960.))
            .h(px(8. * 36. + 32.))
            .child(DataTable::new(&self.table_state).bordered(false));
        let skel_row = |w1: f32, w2: f32, w3: f32| {
            h_flex()
                .h(px(36.))
                .px(px(16.))
                .gap(px(12.))
                .items_center()
                .border_b(px(1.))
                .border_color(prim_hsla("white-a8").unwrap_or_default())
                .child(
                    div()
                        .w(px(w1))
                        .h(px(10.))
                        .rounded(px(2.))
                        .bg(role_hsla(mode, "popover").unwrap_or_default()),
                )
                .child(
                    div()
                        .w(px(w2))
                        .h(px(10.))
                        .rounded(px(2.))
                        .bg(role_hsla(mode, "popover").unwrap_or_default()),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .w(px(w3))
                        .h(px(10.))
                        .rounded(px(2.))
                        .bg(role_hsla(mode, "popover").unwrap_or_default()),
                )
        };
        let skeleton = organisms::table_frame(460., mode)
            .child(skel_row(40., 140., 60.))
            .child(skel_row(40., 120., 60.))
            .child(skel_row(40., 150., 60.))
            .child(skel_row(40., 110., 60.))
            .child(skel_row(40., 130., 60.));
        let empty =
            organisms::table_frame(460., mode).child(div().w_full().flex().justify_center().child(
                organisms::table_empty(
                    IconName::Search,
                    tr(cx, "gallery.o01.dt.empty_title"),
                    tr(cx, "gallery.o01.dt.empty_desc"),
                    mode,
                ),
            ));
        let dt_group = kit::group(
            cx,
            "gallery.o01.dt.title",
            "gallery.o01.dt.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o01.dt.live", live_table))
                .child(kit::cell(cx, "gallery.o01.dt.master", painted_table(mode)))
                .child(kit::row([
                    kit::cell(cx, "gallery.o01.dt.skeleton", skeleton),
                    kit::cell(cx, "gallery.o01.dt.empty", empty),
                ])),
        );

        // ── O7 · list ────────────────────────────────────────────────────────
        let live_list = div()
            .w(px(420.))
            .h(px(5. * 48. + 36.))
            .child(List::new(&self.list_state));
        let fg = role_hsla(mode, "foreground").unwrap_or_default();
        let muted = prim_hsla("neutral-400").unwrap_or_default();
        let avatar = |bg: &str, ink: &str, initials: &str| {
            div()
                .size(px(28.))
                .rounded_full()
                .bg(prim_hsla(bg).unwrap_or_default())
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(11.))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .text_color(prim_hsla(ink).unwrap_or_default())
                        .child(initials.to_string()),
                )
        };
        let _ = (fg, muted);
        let item_ladder = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                organisms::list_item(
                    div()
                        .size(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            bookkeepy_ui::api::language::atoms::icon(IconName::Landmark, 18.)
                                .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                        ),
                    "BT Pay".into(),
                    "current · …4021".into(),
                    organisms::amount_trailing("300,00".into(), "RON", fg).into_any_element(),
                    LiIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                organisms::list_item(
                    div()
                        .size(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            bookkeepy_ui::api::language::atoms::icon(IconName::Wallet, 18.)
                                .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                        ),
                    "Revolut".into(),
                    "current · EUR + RON".into(),
                    organisms::amount_trailing("20.967,00".into(), "RON", fg).into_any_element(),
                    LiIx::Hover,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.selected",
                organisms::list_item(
                    div()
                        .size(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            bookkeepy_ui::api::language::atoms::icon(IconName::Wallet, 18.)
                                .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                        ),
                    "Cash".into(),
                    "wallet".into(),
                    organisms::amount_trailing("1.240,00".into(), "RON", fg).into_any_element(),
                    LiIx::Selected,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                organisms::list_item(
                    avatar("cyan-900", "cyan-300", "SV"),
                    "Savings".into(),
                    "deposit · 4.1%".into(),
                    organisms::amount_trailing("25.813,00".into(), "RON", fg).into_any_element(),
                    LiIx::Focus,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                organisms::list_item(
                    div()
                        .size(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            bookkeepy_ui::api::language::atoms::icon(IconName::Landmark, 18.)
                                .text_color(muted),
                        ),
                    "Old ING".into(),
                    "closed in 2024".into(),
                    organisms::amount_trailing("0,00".into(), "RON", fg).into_any_element(),
                    LiIx::Disabled,
                    mode,
                ),
            ),
        ]);
        let payee_list = organisms::list_frame(mode)
            .child(organisms::list_search(
                tr(cx, "gallery.o01.ls.search"),
                mode,
            ))
            .child(organisms::list_section_header(
                "payees · confirmed".into(),
                "3".into(),
            ))
            .child(organisms::list_item(
                avatar("cyan-900", "cyan-300", "AP"),
                "Andrei Popescu".into(),
                "owes you 120 RON".into(),
                bookkeepy_ui::api::language::atoms::icon(IconName::Check, 14.)
                    .text_color(prim_hsla("neutral-200").unwrap_or_default())
                    .into_any_element(),
                LiIx::Rest,
                mode,
            ))
            .child(organisms::list_item(
                avatar("pink-900", "pink-300", "IM"),
                "Ioana M.".into(),
                "settled".into(),
                div().into_any_element(),
                LiIx::Hover,
                mode,
            ))
            .child(organisms::list_item(
                div()
                    .size(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        bookkeepy_ui::api::language::atoms::icon(IconName::Receipt, 18.)
                            .text_color(prim_hsla("neutral-200").unwrap_or_default()),
                    ),
                "Mega Image".into(),
                "42 transactions".into(),
                div().into_any_element(),
                LiIx::Rest,
                mode,
            ));
        let ls_group = kit::group(
            cx,
            "gallery.o01.ls.title",
            "gallery.o01.ls.hint",
            v_flex()
                .gap_6()
                .child(kit::row([
                    kit::cell(cx, "gallery.o01.ls.live", live_list),
                    kit::cell(cx, "gallery.o01.ls.master", payee_list),
                ]))
                .child(item_ladder),
        );

        // ── O8 · virtual list ────────────────────────────────────────────────
        let vlist = v_virtual_list(
            cx.entity(),
            "o8-vlist",
            self.vlist_sizes.clone(),
            |_v, range, _window, _cx| {
                range
                    .map(|ix| {
                        let h = self_h(ix);
                        if ix % 5 == 0 {
                            h_flex()
                                .h(px(h))
                                .px(px(16.))
                                .items_center()
                                .justify_between()
                                .bg(prim_hsla("neutral-875").unwrap_or_default())
                                .border_b(px(1.))
                                .border_color(prim_hsla("white-a8").unwrap_or_default())
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .font_weight(gpui_kit::FontWeight::MEDIUM)
                                        .text_color(prim_hsla("neutral-400").unwrap_or_default())
                                        .child(format!("{} OCT", 18 - (ix / 5) % 5)),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(prim_hsla("neutral-500").unwrap_or_default())
                                        .child(format!("{} px", h as u32)),
                                )
                                .into_any_element()
                        } else {
                            h_flex()
                                .h(px(h))
                                .px(px(16.))
                                .items_center()
                                .bg(prim_hsla("neutral-900").unwrap_or_default())
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(prim_hsla("neutral-0").unwrap_or_default())
                                        .child(format!("Row {ix}")),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(prim_hsla("neutral-500").unwrap_or_default())
                                        .child(format!("{} px", h as u32)),
                                )
                                .into_any_element()
                        }
                    })
                    .collect::<Vec<_>>()
            },
        )
        .w(px(480.))
        .h(px(480.));
        let vl_group = kit::group(
            cx,
            "gallery.o01.vl.title",
            "gallery.o01.vl.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.o01.vl.live",
                h_flex().gap_6().items_start().child(vlist).child(
                    v_flex().w(px(300.)).gap_3().children([
                        div()
                            .text_size(px(12.))
                            .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                            .child(tr(cx, "gallery.o01.vl.viewport")),
                        div()
                            .text_size(px(12.))
                            .line_height(px(18.))
                            .text_color(prim_hsla("neutral-400").unwrap_or_default())
                            .child(tr(cx, "gallery.o01.vl.desc")),
                        div()
                            .text_size(px(11.))
                            .text_color(prim_hsla("orange-300").unwrap_or_default())
                            .child(tr(cx, "gallery.o01.vl.range")),
                    ]),
                ),
            )),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.o01.tb.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.o01.tb.title")))
            .child(kit::caption(cx, tr(cx, "gallery.o01.tb.subtitle")))
            .child(dt_group)
            .child(ls_group)
            .child(vl_group)
    }
}

fn self_h(ix: usize) -> f32 {
    match ix % 5 {
        0 => 28.,
        3 => 56.,
        _ => 36.,
    }
}
