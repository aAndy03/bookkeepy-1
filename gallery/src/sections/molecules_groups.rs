// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 02 sheet — toggle group, radio group, input group, dropdown
//! button, hover card, collapsible, accordion, alert, description list,
//! clipboard, pagination, stepper (references/designlanguage).
//!
//! FLAT on the page background like the design frames. Rest cells are LIVE
//! stock components; hover/pressed/focus/invalid/open/loading/disabled ladders
//! are painted from `language::groups` — same tokens, same geometry.

use bookkeepy_ui::api::language::groups::{
    self, AlertTone, CbIx, ClIx, DdIx, DdVariant, IgIx, PgIx, RgIx, StIx, TgIx,
};
use bookkeepy_ui::api::language::popups::{self, MenuIx};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::accordion::Accordion;
use gpui_kit::component::alert::Alert;
use gpui_kit::component::button::{Button, DropdownButton, Toggle, ToggleGroup};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::collapsible::Collapsible;
use gpui_kit::component::description_list::DescriptionList;
use gpui_kit::component::hover_card::HoverCard;
use gpui_kit::component::input::{
    InputGroup, InputGroupAddon, InputGroupAddonAlignment, InputGroupButton, InputGroupInput,
    InputGroupText, InputState,
};
use gpui_kit::component::pagination::Pagination;
use gpui_kit::component::radio::{Radio, RadioGroup};
use gpui_kit::component::stepper::{Stepper, StepperItem};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, InteractiveElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let amount = cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(tr(cx, "gallery.m02.ig.label"))
            .default_value("1.250,00")
    });
    cx.new(|_cx| MoleculesGroups {
        tg: vec![false, true, false],
        rg: 1,
        coll_open: true,
        acc: 0,
        page: 5,
        step: 1,
        amount,
    })
    .into()
}

pub struct MoleculesGroups {
    tg: Vec<bool>,
    rg: usize,
    coll_open: bool,
    acc: usize,
    page: usize,
    step: usize,
    amount: Entity<InputState>,
}

/// The live ToggleGroup cell: controlled checked vector, segmented track.
fn live_toggle_group(view: &MoleculesGroups, cx: &mut Context<'_, MoleculesGroups>) -> ToggleGroup {
    ToggleGroup::new("m2-tg")
        .segmented()
        .on_click(cx.listener(|v, states: &Vec<bool>, _window, cx| {
            v.tg = states.clone();
            cx.notify();
        }))
        .child(
            Toggle::new("m2-tg-day")
                .label(tr(cx, "gallery.m02.tg.day"))
                .checked(view.tg[0]),
        )
        .child(
            Toggle::new("m2-tg-week")
                .label(tr(cx, "gallery.m02.tg.week"))
                .checked(view.tg[1]),
        )
        .child(
            Toggle::new("m2-tg-month")
                .label(tr(cx, "gallery.m02.tg.month"))
                .checked(view.tg[2]),
        )
}

/// Painted toggle group with items forced into states.
fn painted_tg(cx: &Context<'_, MoleculesGroups>, ix: [TgIx; 3], icons: bool) -> gpui_kit::Div {
    let item = |label: Option<&str>, glyph: Option<IconName>, ix: TgIx| {
        groups::toggle_item(label.map(|l| tr(cx, l)), glyph, ix)
    };
    groups::toggle_track(h_flex().gap(px(2.)).children([
        item(
            if icons {
                None
            } else {
                Some("gallery.m02.tg.day")
            },
            if icons { Some(IconName::List) } else { None },
            ix[0],
        ),
        item(
            if icons {
                None
            } else {
                Some("gallery.m02.tg.week")
            },
            if icons {
                Some(IconName::ChartPie)
            } else {
                None
            },
            ix[1],
        ),
        item(
            if icons {
                None
            } else {
                Some("gallery.m02.tg.month")
            },
            if icons {
                Some(IconName::Calendar)
            } else {
                None
            },
            ix[2],
        ),
    ]))
}

/// One collapsible body row: colored dot + label + meta, amount at the end.
fn cl_row(
    cx: &Context<'_, MoleculesGroups>,
    dot: &str,
    name_key: &str,
    meta_key: &str,
    amount_key: &str,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let amber = bookkeepy_ui::api::language::prim_hsla("amber-500").unwrap_or_default();
    let fg = bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default();
    let muted =
        bookkeepy_ui::api::language::role_hsla(mode, "muted-foreground").unwrap_or_default();
    let ron = tr(cx, "gallery.m02.dl.ron");
    h_flex()
        .h(px(24.))
        .gap(px(8.))
        .items_center()
        .w_full()
        .child(
            div()
                .size(px(6.))
                .rounded_full()
                .bg(bookkeepy_ui::api::language::prim_hsla(dot).unwrap_or_default()),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(fg)
                .child(tr(cx, name_key)),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(muted)
                .child(tr(cx, meta_key)),
        )
        .child(div().flex_1())
        .child(groups::dl_amount(
            tr(cx, "gallery.m02.dl.minus"),
            tr(cx, amount_key),
            ron,
            amber,
            mode,
        ))
}

/// The collapsible body (shared by the live and the painted open cell).
fn cl_body(
    cx: &Context<'_, MoleculesGroups>,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    v_flex()
        .gap(px(4.))
        .p(px(8.))
        .pl(px(24.))
        .w_full()
        .child(cl_row(
            cx,
            "cyan-500",
            "gallery.m02.cl.rent",
            "gallery.m02.cl.in2",
            "gallery.m02.cl.a_rent",
            mode,
        ))
        .child(cl_row(
            cx,
            "pink-500",
            "gallery.m02.cl.electricity",
            "gallery.m02.cl.in4",
            "gallery.m02.cl.a_elec",
            mode,
        ))
        .child(cl_row(
            cx,
            "orange-500",
            "gallery.m02.cl.gym",
            "gallery.m02.cl.in6",
            "gallery.m02.cl.a_gym",
            mode,
        ))
}

/// Localized hover-card copy, captured so the live card's `content` closure can
/// be `'static`.
#[derive(Clone)]
struct HcCopy {
    bank: gpui_kit::SharedString,
    account: gpui_kit::SharedString,
    balance: gpui_kit::SharedString,
    amt_bal: gpui_kit::SharedString,
    in_month: gpui_kit::SharedString,
    in_amt: gpui_kit::SharedString,
    out_month: gpui_kit::SharedString,
    out_amt: gpui_kit::SharedString,
    synced: gpui_kit::SharedString,
    ron: gpui_kit::SharedString,
}

/// The hover-card panel content (shared by live content and painted open cell).
fn hc_panel(c: &HcCopy, mode: bookkeepy_primary_data::theme::Mode) -> gpui_kit::Div {
    let fg = bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default();
    let green = bookkeepy_ui::api::language::prim_hsla("green-500").unwrap_or_default();
    let stat =
        |label: gpui_kit::SharedString, digits: gpui_kit::SharedString, ink: gpui_kit::Hsla| {
            groups::hc_stat(label, digits, c.ron.clone(), ink, mode)
        };
    groups::hover_card(
        v_flex()
            .gap(px(12.))
            .child(groups::hc_head(
                IconName::Landmark,
                c.bank.clone(),
                c.account.clone(),
                mode,
            ))
            .child(stat(c.balance.clone(), c.amt_bal.clone(), fg))
            .child(h_flex().gap(px(24.)).children([
                stat(c.in_month.clone(), c.in_amt.clone(), green),
                stat(c.out_month.clone(), c.out_amt.clone(), fg),
            ]))
            .child(groups::hc_foot(c.synced.clone())),
        mode,
    )
}

/// Hover-card trigger text with optional underline (hover/open states).
fn hc_trigger(
    cx: &Context<'_, MoleculesGroups>,
    underline: bool,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let fg = bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default();
    let mut t = div()
        .text_size(px(12.))
        .font_weight(gpui_kit::FontWeight::MEDIUM)
        .text_color(fg)
        .child(tr(cx, "gallery.m02.hc.bank"));
    if underline {
        t = t.border_b(px(1.)).border_color(fg);
    }
    t
}

/// One painted description-list row at fixed width.
fn dl_row_w(
    cx: &Context<'_, MoleculesGroups>,
    label_key: &str,
    value: gpui_kit::Div,
    w: f32,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let muted =
        bookkeepy_ui::api::language::role_hsla(mode, "muted-foreground").unwrap_or_default();
    h_flex()
        .w(px(w))
        .py(px(8.))
        .gap(px(16.))
        .border_b(px(1.))
        .border_color(bookkeepy_ui::api::language::prim_hsla("white-a8").unwrap_or_default())
        .child(
            div()
                .w(px(120.))
                .text_size(px(12.))
                .text_color(muted)
                .child(tr(cx, label_key)),
        )
        .child(value)
}

/// A plain white 12/400 value cell.
fn dl_value(
    text: gpui_kit::SharedString,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    div()
        .text_size(px(12.))
        .text_color(bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default())
        .child(text)
}

/// The full 6-row horizontal master (design M16).
fn dl_master_rows(
    cx: &Context<'_, MoleculesGroups>,
    w: f32,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let fg = bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default();
    v_flex().children([
        dl_row_w(
            cx,
            "gallery.m02.dl.payee",
            dl_value(tr(cx, "gallery.m02.dl.lidl"), mode),
            w,
            mode,
        ),
        dl_row_w(
            cx,
            "gallery.m02.dl.account",
            dl_value(tr(cx, "gallery.m02.dl.btpay"), mode),
            w,
            mode,
        ),
        dl_row_w(
            cx,
            "gallery.m02.dl.category",
            groups::dl_tag(tr(cx, "gallery.m02.dl.tag"), mode),
            w,
            mode,
        ),
        dl_row_w(
            cx,
            "gallery.m02.dl.amount",
            groups::dl_amount(
                tr(cx, "gallery.m02.dl.minus"),
                tr(cx, "gallery.m02.dl.amount_v"),
                tr(cx, "gallery.m02.dl.ron"),
                fg,
                mode,
            ),
            w,
            mode,
        ),
        dl_row_w(
            cx,
            "gallery.m02.dl.date",
            dl_value(tr(cx, "gallery.m02.dl.date_v"), mode),
            w,
            mode,
        ),
        dl_row_w(
            cx,
            "gallery.m02.dl.note",
            dl_value(tr(cx, "gallery.m02.dl.note_v"), mode),
            w,
            mode,
        ),
    ])
}

/// The vertical 3-column grid (design M16): two rows of three cells, row gap 32.
fn dl_vertical_grid(
    cx: &Context<'_, MoleculesGroups>,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let cell = |l: &str, v: &str| groups::dl_cell(tr(cx, l), tr(cx, v), mode);
    v_flex()
        .w(px(520.))
        .gap(px(32.))
        .child(h_flex().gap(px(16.)).children([
            cell("gallery.m02.dl.opened", "gallery.m02.dl.opened_v"),
            cell("gallery.m02.dl.type", "gallery.m02.dl.type_v"),
            cell("gallery.m02.dl.currency", "gallery.m02.dl.ron"),
        ]))
        .child(h_flex().gap(px(16.)).children([
            cell("gallery.m02.dl.iban", "gallery.m02.dl.iban_v"),
            cell("gallery.m02.dl.transactions", "gallery.m02.dl.count"),
            cell("gallery.m02.dl.last_import", "gallery.m02.dl.last_import_v"),
        ]))
}

/// Painted "open" dropdown button with its menu popup beneath (design M11).
const CTRL_I: &str = "Ctrl+I";

fn dd_open_menu(
    cx: &Context<'_, MoleculesGroups>,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    v_flex()
        .gap(px(8.))
        .items_start()
        .child(groups::dropdown_button(
            tr(cx, "gallery.m02.dd.import"),
            IconName::Upload,
            DdVariant::Secondary,
            DdIx::Open,
            mode,
        ))
        .child(
            popups::popup_surface(232., 4., mode).child(v_flex().children([
                popups::menu_item(
                    tr(cx, "gallery.m02.dd.bank"),
                    None,
                    false,
                    Some(CTRL_I),
                    MenuIx::Rest,
                    mode,
                ),
                popups::menu_item(
                    tr(cx, "gallery.m02.dd.ofx"),
                    None,
                    false,
                    None,
                    MenuIx::Rest,
                    mode,
                ),
                popups::menu_item(
                    tr(cx, "gallery.m02.dd.bk"),
                    None,
                    false,
                    None,
                    MenuIx::Rest,
                    mode,
                ),
            ])),
        )
}

impl gpui_kit::Render for MoleculesGroups {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── M8 · toggle group ────────────────────────────────────────────────
        let live_tg = live_toggle_group(self, cx);
        let tg_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                painted_tg(cx, [TgIx::Rest, TgIx::On, TgIx::Rest], false),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                painted_tg(cx, [TgIx::Rest, TgIx::Hover, TgIx::On], false),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.pressed",
                painted_tg(cx, [TgIx::Rest, TgIx::Pressed, TgIx::On], false),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                painted_tg(cx, [TgIx::Rest, TgIx::Rest, TgIx::Focus], false),
            ),
            kit::cell(
                cx,
                "gallery.m02.tg.multiple_on",
                painted_tg(cx, [TgIx::On, TgIx::Rest, TgIx::On], false),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                painted_tg(cx, [TgIx::Disabled, TgIx::Disabled, TgIx::Disabled], false),
            ),
            kit::cell(
                cx,
                "gallery.m02.tg.icons",
                painted_tg(cx, [TgIx::On, TgIx::Rest, TgIx::Rest], true),
            ),
        ]);
        let tg_group = kit::group(
            cx,
            "gallery.m02.tg.title",
            "gallery.m02.tg.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.tg.live", live_tg))
                .child(tg_states),
        );

        // ── M9 · radio group ─────────────────────────────────────────────────
        let live_rg = RadioGroup::vertical("m2-rg")
            .selected_index(Some(self.rg))
            .on_click(cx.listener(|v, ix: &usize, _window, cx| {
                v.rg = *ix;
                cx.notify();
            }))
            .child(Radio::new("m2-r-never").label(tr(cx, "gallery.m02.rg.never")))
            .child(Radio::new("m2-r-monthly").label(tr(cx, "gallery.m02.rg.monthly")))
            .child(Radio::new("m2-r-yearly").label(tr(cx, "gallery.m02.rg.yearly")));
        let rg_opts = |a: RgIx, b: RgIx, c: RgIx| {
            h_flex().gap(px(16.)).children([
                groups::radio_option(tr(cx, "gallery.m02.rg.never"), a),
                groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), b),
                groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), c),
            ])
        };
        let rg_col = |a: RgIx, b: RgIx, c: RgIx| {
            v_flex().gap_2().children([
                groups::radio_option(tr(cx, "gallery.m02.rg.never"), a),
                groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), b),
                groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), c),
            ])
        };
        let rg_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help")),
                    None,
                    rg_col(RgIx::Rest, RgIx::Checked, RgIx::Rest),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help")),
                    None,
                    rg_col(RgIx::Rest, RgIx::Checked, RgIx::Hover),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help")),
                    None,
                    rg_col(RgIx::Rest, RgIx::Focus, RgIx::Rest),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.invalid",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help_invalid")),
                    Some(bookkeepy_ui::api::language::prim_hsla("red-300").unwrap_or_default()),
                    rg_col(RgIx::Rest, RgIx::Invalid, RgIx::Rest),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.rg.horizontal",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help")),
                    None,
                    rg_opts(RgIx::Rest, RgIx::Checked, RgIx::Rest),
                    true,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help")),
                    None,
                    rg_col(RgIx::Disabled, RgIx::Disabled, RgIx::Disabled),
                    false,
                    mode,
                ),
            ),
        ]);
        let rg_group = kit::group(
            cx,
            "gallery.m02.rg.title",
            "gallery.m02.rg.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.rg.live", live_rg))
                .child(rg_states),
        );

        // ── M10 · input group ────────────────────────────────────────────────
        let live_ig = InputGroup::new("m2-ig")
            .w(px(280.))
            .input(InputGroupInput::new(&self.amount))
            .addon(
                InputGroupAddon::new("m2-ig-pre")
                    .child(InputGroupText::new().child(tr(cx, "gallery.m02.dl.ron"))),
            )
            .addon(
                InputGroupAddon::new("m2-ig-suf")
                    .align(InputGroupAddonAlignment::InlineEnd)
                    .child(
                        InputGroupButton::new("m2-ig-max")
                            .label(tr(cx, "gallery.m02.ig.max"))
                            .on_click(|_, _, _| {}),
                    ),
            );
        // Painted ladder cell: full group frame for one interaction state.
        let ig_cell = |ix: IgIx, help: Option<(&str, bool)>| {
            let help = help.map(|(k, danger)| {
                (
                    tr(cx, k),
                    if danger {
                        bookkeepy_ui::api::language::prim_hsla("red-300").unwrap_or_default()
                    } else {
                        bookkeepy_ui::api::language::role_hsla(mode, "muted-foreground")
                            .unwrap_or_default()
                    },
                )
            });
            v_flex()
                .w(px(280.))
                .gap_2()
                .child(groups::ig_label(tr(cx, "gallery.m02.ig.label"), help, mode))
                .child(groups::ig_frame(
                    groups::ig_field(
                        {
                            let row = h_flex()
                                .child(groups::ig_prefix(tr(cx, "gallery.m02.dl.ron"), ix, mode))
                                .child(groups::ig_text(tr(cx, "gallery.m02.ig.value"), ix, mode));
                            let mut suffix = h_flex();
                            if ix == IgIx::Invalid {
                                suffix = suffix.child(
                                    bookkeepy_ui::api::language::atoms::icon(
                                        IconName::CircleAlert,
                                        14.,
                                    )
                                    .text_color(
                                        bookkeepy_ui::api::language::prim_hsla("red-500")
                                            .unwrap_or_default(),
                                    ),
                                );
                            } else if ix == IgIx::Readonly {
                                suffix = suffix.child(
                                    bookkeepy_ui::api::language::atoms::icon(IconName::Lock, 14.)
                                        .text_color(
                                            bookkeepy_ui::api::language::role_hsla(
                                                mode,
                                                "muted-foreground",
                                            )
                                            .unwrap_or_default(),
                                        ),
                                );
                            }
                            row.child(groups::ig_suffix(
                                suffix.child(groups::ig_button(tr(cx, "gallery.m02.ig.max"), mode)),
                            ))
                        },
                        ix,
                    ),
                    ix,
                ))
        };
        let ig_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                ig_cell(IgIx::Rest, Some(("gallery.m02.ig.help", false))),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                ig_cell(IgIx::Hover, Some(("gallery.m02.ig.help", false))),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                ig_cell(IgIx::Focus, Some(("gallery.m02.ig.help", false))),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.invalid",
                ig_cell(IgIx::Invalid, Some(("gallery.m02.ig.help_invalid", true))),
            ),
            kit::cell(
                cx,
                "gallery.m02.ig.readonly",
                ig_cell(IgIx::Readonly, Some(("gallery.m02.ig.help", false))),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                ig_cell(IgIx::Disabled, Some(("gallery.m02.ig.help", false))),
            ),
        ]);
        let ig_group = kit::group(
            cx,
            "gallery.m02.ig.title",
            "gallery.m02.ig.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.ig.live", live_ig))
                .child(ig_states),
        );

        // ── M11 · dropdown button ────────────────────────────────────────────
        let live_dd = DropdownButton::new("m2-dd")
            .button(
                Button::new("m2-dd-main")
                    .label(tr(cx, "gallery.m02.dd.import"))
                    .icon(IconName::Upload),
            )
            .dropdown_menu(|menu, _window, _cx| {
                menu.item(
                    gpui_kit::component::menu::PopupMenuItem::new(tr_static("gallery.m02.dd.bank"))
                        .on_click(|_, _, _| {}),
                )
                .item(
                    gpui_kit::component::menu::PopupMenuItem::new(tr_static("gallery.m02.dd.ofx"))
                        .on_click(|_, _, _| {}),
                )
                .item(
                    gpui_kit::component::menu::PopupMenuItem::new(tr_static("gallery.m02.dd.bk"))
                        .on_click(|_, _, _| {}),
                )
            });
        let imp = |v: DdVariant, ix: DdIx| {
            groups::dropdown_button(
                tr(cx, "gallery.m02.dd.import"),
                IconName::Upload,
                v,
                ix,
                mode,
            )
        };
        let dd_primary = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                imp(DdVariant::Primary, DdIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.hover_main",
                imp(DdVariant::Primary, DdIx::HoverMain),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.hover_caret",
                imp(DdVariant::Primary, DdIx::HoverCaret),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.open",
                imp(DdVariant::Primary, DdIx::Open),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.loading",
                imp(DdVariant::Primary, DdIx::Loading),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                imp(DdVariant::Primary, DdIx::FocusMain),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                imp(DdVariant::Primary, DdIx::Disabled),
            ),
        ]);
        let dd_variants = kit::row([
            kit::cell(
                cx,
                "gallery.m02.dd.secondary",
                imp(DdVariant::Secondary, DdIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.outline",
                imp(DdVariant::Outline, DdIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.outline_hover",
                imp(DdVariant::Outline, DdIx::HoverMain),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.ghost",
                imp(DdVariant::Ghost, DdIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.ghost_hover",
                imp(DdVariant::Ghost, DdIx::HoverMain),
            ),
            kit::cell(cx, "gallery.m02.dd.open_menu", dd_open_menu(cx, mode)),
        ]);
        let dd_group = kit::group(
            cx,
            "gallery.m02.dd.title",
            "gallery.m02.dd.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.dd.live", live_dd))
                .child(dd_primary)
                .child(dd_variants),
        );

        // ── M12 · hover card ─────────────────────────────────────────────────
        let hc_copy = HcCopy {
            bank: tr(cx, "gallery.m02.hc.bank"),
            account: tr(cx, "gallery.m02.hc.account"),
            balance: tr(cx, "gallery.m02.hc.balance"),
            amt_bal: tr(cx, "gallery.m02.hc.amount_balance"),
            in_month: tr(cx, "gallery.m02.hc.in_month"),
            in_amt: tr(cx, "gallery.m02.hc.in_amount"),
            out_month: tr(cx, "gallery.m02.hc.out_month"),
            out_amt: tr(cx, "gallery.m02.hc.out_amount"),
            synced: tr(cx, "gallery.m02.hc.synced"),
            ron: tr(cx, "gallery.m02.dl.ron"),
        };
        let live_hc = HoverCard::new("m2-hc")
            .open_delay(std::time::Duration::from_millis(500))
            .close_delay(std::time::Duration::from_millis(300))
            .trigger(hc_trigger(cx, false, mode))
            .content({
                let hc_copy = hc_copy.clone();
                move |_state, _window, _cx| hc_panel(&hc_copy, mode)
            });
        let hc_flow = kit::row([
            kit::cell(cx, "gallery.m02.hc.closed", hc_trigger(cx, false, mode)),
            kit::cell(cx, "gallery.m02.hc.hovering", hc_trigger(cx, true, mode)),
            kit::cell(
                cx,
                "gallery.atoms.state.open",
                v_flex()
                    .gap_2()
                    .items_start()
                    .child(hc_trigger(cx, true, mode))
                    .child(hc_panel(&hc_copy, mode)),
            ),
        ]);
        let hc_group = kit::group(
            cx,
            "gallery.m02.hc.title",
            "gallery.m02.hc.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.hc.live", live_hc))
                .child(hc_flow),
        );

        // ── M13 · collapsible ────────────────────────────────────────────────
        let live_coll = Collapsible::new()
            .open(self.coll_open)
            .motion_id("m2-coll")
            .child(
                div()
                    .id("m2-coll-trigger")
                    .on_click(cx.listener(|v, _, _window, cx| {
                        v.coll_open = !v.coll_open;
                        cx.notify();
                    }))
                    .w_full()
                    .child(groups::cl_trigger(
                        tr(cx, "gallery.m02.cl.title"),
                        Some(3),
                        if self.coll_open {
                            ClIx::Open
                        } else {
                            ClIx::Closed
                        },
                        mode,
                    )),
            )
            .content(cl_body(cx, mode));
        let cl_states = kit::row([
            kit::cell(
                cx,
                "gallery.m02.cl.closed",
                groups::cl_trigger(tr(cx, "gallery.m02.cl.title"), Some(3), ClIx::Closed, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                groups::cl_trigger(tr(cx, "gallery.m02.cl.title"), Some(3), ClIx::Hover, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::cl_trigger(tr(cx, "gallery.m02.cl.title"), Some(3), ClIx::Focus, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.open",
                v_flex()
                    .w(px(300.))
                    .child(groups::cl_trigger(
                        tr(cx, "gallery.m02.cl.title"),
                        Some(3),
                        ClIx::Open,
                        mode,
                    ))
                    .child(cl_body(cx, mode)),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::cl_trigger(
                    tr(cx, "gallery.m02.cl.title"),
                    Some(3),
                    ClIx::Disabled,
                    mode,
                ),
            ),
        ]);
        let cl_group = kit::group(
            cx,
            "gallery.m02.cl.title_group",
            "gallery.m02.cl.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.cl.live", live_coll))
                .child(cl_states),
        );

        // ── M14 · accordion ──────────────────────────────────────────────────
        let acc_body = |key: &str| {
            div()
                .text_size(px(12.))
                .line_height(px(18.))
                .text_color(
                    bookkeepy_ui::api::language::role_hsla(mode, "secondary-foreground")
                        .unwrap_or_default(),
                )
                .child(tr(cx, key))
        };
        let live_acc = Accordion::new("m2-acc")
            .w(px(360.))
            .on_toggle_click(cx.listener(|v, open: &[usize], _window, cx| {
                v.acc = open.first().copied().unwrap_or(usize::MAX);
                cx.notify();
            }))
            .item(|i| {
                i.title(tr(cx, "gallery.m02.acc.q1"))
                    .open(self.acc == 0)
                    .child(acc_body("gallery.m02.acc.a1"))
            })
            .item(|i| {
                i.title(tr(cx, "gallery.m02.acc.q2"))
                    .open(self.acc == 1)
                    .child(acc_body("gallery.m02.acc.a2"))
            })
            .item(|i| {
                i.title(tr(cx, "gallery.m02.acc.q3"))
                    .open(self.acc == 2)
                    .child(acc_body("gallery.m02.acc.a3"))
            });
        let acc_states = kit::row([
            kit::cell(
                cx,
                "gallery.m02.acc.closed",
                groups::acc_item(
                    tr(cx, "gallery.m02.acc.q1"),
                    None::<&str>,
                    ClIx::Closed,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                groups::acc_item(
                    tr(cx, "gallery.m02.acc.q1"),
                    None::<&str>,
                    ClIx::Hover,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::acc_item(
                    tr(cx, "gallery.m02.acc.q1"),
                    None::<&str>,
                    ClIx::Focus,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.open",
                groups::acc_item(
                    tr(cx, "gallery.m02.acc.q1"),
                    Some(tr(cx, "gallery.m02.acc.a1")),
                    ClIx::Open,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::acc_item(
                    tr(cx, "gallery.m02.acc.q1"),
                    None::<&str>,
                    ClIx::Disabled,
                    mode,
                ),
            ),
        ]);
        // Bordered mode: card frame with hairline rows inside (design M14 modes).
        let acc_bordered = kit::cell(
            cx,
            "gallery.m02.acc.bordered",
            v_flex()
                .w(px(380.))
                .rounded(px(16.))
                .border(px(1.))
                .border_color(
                    bookkeepy_ui::api::language::prim_hsla("white-a8").unwrap_or_default(),
                )
                .p(px(12.))
                .children([
                    groups::acc_item(
                        tr(cx, "gallery.m02.acc.q1"),
                        Some(tr(cx, "gallery.m02.acc.a1")),
                        ClIx::Open,
                        mode,
                    ),
                    groups::acc_item(
                        tr(cx, "gallery.m02.acc.q2"),
                        None::<&str>,
                        ClIx::Closed,
                        mode,
                    ),
                    groups::acc_item(
                        tr(cx, "gallery.m02.acc.q3"),
                        None::<&str>,
                        ClIx::Closed,
                        mode,
                    ),
                ]),
        );
        let acc_group = kit::group(
            cx,
            "gallery.m02.acc.title",
            "gallery.m02.acc.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.acc.live", live_acc))
                .child(acc_states)
                .child(acc_bordered),
        );

        // ── M15 · alert ──────────────────────────────────────────────────────
        let live_alerts = h_flex()
            .gap(px(12.))
            .items_start()
            .child(
                Alert::info("m2-al-info", tr(cx, "gallery.m02.alert.m_info"))
                    .title(tr(cx, "gallery.m02.alert.t_info"))
                    .on_close(|_, _, _| {}),
            )
            .child(
                Alert::error("m2-al-danger", tr(cx, "gallery.m02.alert.m_danger"))
                    .title(tr(cx, "gallery.m02.alert.t_danger"))
                    .on_close(|_, _, _| {}),
            );
        let al = |tone: AlertTone, glyph: IconName, t: &str, m: &str| {
            groups::alert_card(
                tone,
                glyph,
                tr(cx, t),
                tr(cx, m),
                Some(tr(cx, "gallery.m02.alert.review")),
                false,
                mode,
            )
        };
        let al_tones = kit::row([
            kit::cell(
                cx,
                "gallery.m02.alert.neutral",
                al(
                    AlertTone::Neutral,
                    IconName::CloudOff,
                    "gallery.m02.alert.t_neutral",
                    "gallery.m02.alert.m_neutral",
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.info",
                al(
                    AlertTone::Info,
                    IconName::Info,
                    "gallery.m02.alert.t_info",
                    "gallery.m02.alert.m_info",
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.success",
                al(
                    AlertTone::Success,
                    IconName::CircleCheck,
                    "gallery.m02.alert.t_success",
                    "gallery.m02.alert.m_success",
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.warning",
                al(
                    AlertTone::Warning,
                    IconName::CircleAlert,
                    "gallery.m02.alert.t_warning",
                    "gallery.m02.alert.m_warning",
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.danger",
                al(
                    AlertTone::Danger,
                    IconName::CircleX,
                    "gallery.m02.alert.t_danger",
                    "gallery.m02.alert.m_danger",
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.accent",
                al(
                    AlertTone::Accent,
                    IconName::Info,
                    "gallery.m02.alert.t_accent",
                    "gallery.m02.alert.m_accent",
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.banner",
                groups::alert_card(
                    AlertTone::Warning,
                    IconName::CircleAlert,
                    tr(cx, "gallery.m02.alert.t_banner"),
                    tr(cx, "gallery.m02.alert.m_banner"),
                    None::<&str>,
                    true,
                    mode,
                ),
            ),
        ]);
        let al_group = kit::group(
            cx,
            "gallery.m02.alert.title_group",
            "gallery.m02.alert.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.alert.live", live_alerts))
                .child(al_tones),
        );

        // ── M16 · description list ───────────────────────────────────────────
        let live_dl = DescriptionList::new()
            .bordered(false)
            .columns(1)
            .item(
                tr(cx, "gallery.m02.dl.payee"),
                tr(cx, "gallery.m02.dl.lidl"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.account"),
                tr(cx, "gallery.m02.dl.btpay"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.date"),
                tr(cx, "gallery.m02.dl.date_v"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.note"),
                tr(cx, "gallery.m02.dl.note_v"),
                1,
            );
        let live_dlv = DescriptionList::vertical()
            .bordered(false)
            .columns(3)
            .item(
                tr(cx, "gallery.m02.dl.opened"),
                tr(cx, "gallery.m02.dl.opened_v"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.type"),
                tr(cx, "gallery.m02.dl.type_v"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.currency"),
                tr(cx, "gallery.m02.dl.ron"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.iban"),
                tr(cx, "gallery.m02.dl.iban_v"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.transactions"),
                tr(cx, "gallery.m02.dl.count"),
                1,
            )
            .item(
                tr(cx, "gallery.m02.dl.last_import"),
                tr(cx, "gallery.m02.dl.last_import_v"),
                1,
            );
        let dl_modes = kit::row([
            kit::cell(
                cx,
                "gallery.m02.dl.horizontal",
                dl_master_rows(cx, 440., mode),
            ),
            kit::cell(
                cx,
                "gallery.m02.dl.bordered",
                groups::dl_bordered_frame(dl_master_rows(cx, 442., mode)),
            ),
            kit::cell(cx, "gallery.m02.dl.vertical", dl_vertical_grid(cx, mode)),
        ]);
        let dl_group = kit::group(
            cx,
            "gallery.m02.dl.title",
            "gallery.m02.dl.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.dl.live_h", live_dl))
                .child(kit::cell(cx, "gallery.m02.dl.live_v", live_dlv))
                .child(dl_modes),
        );

        // ── M17 · clipboard ─────────────────────────────────────────────────
        let iban = "RO49 BTRL 0000 1234 5678 4021";
        let live_cb = Clipboard::new("m2-cb").value(iban).label(iban);
        let cb_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                groups::clipboard_chip(iban, CbIx::Idle, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                groups::clipboard_chip(iban, CbIx::Hover, mode),
            ),
            kit::cell(
                cx,
                "gallery.m02.cb.copied",
                v_flex()
                    .gap_2()
                    .items_end()
                    .child(groups::clipboard_chip(iban, CbIx::Copied, mode))
                    .child(groups::clipboard_tooltip(tr(
                        cx,
                        "gallery.m02.cb.copied_tt",
                    ))),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::clipboard_chip(iban, CbIx::Focus, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::clipboard_chip(iban, CbIx::Disabled, mode),
            ),
        ]);
        let cb_group = kit::group(
            cx,
            "gallery.m02.cb.title",
            "gallery.m02.cb.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.cb.live", live_cb))
                .child(cb_states),
        );

        // ── M18 · pagination ─────────────────────────────────────────────────
        let live_pg = Pagination::new("m2-pg")
            .current_page(self.page)
            .total_pages(12)
            .on_click(cx.listener(|v, page: &usize, _window, cx| {
                v.page = *page;
                cx.notify();
            }));
        let live_pg_compact = Pagination::new("m2-pgc")
            .current_page(self.page)
            .total_pages(12)
            .compact()
            .on_click(cx.listener(|v, page: &usize, _window, cx| {
                v.page = *page;
                cx.notify();
            }));
        let pg_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                groups::page_item(3, PgIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                groups::page_item(3, PgIx::Hover),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.pressed",
                groups::page_item(3, PgIx::Pressed),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::page_item(3, PgIx::Focus),
            ),
            kit::cell(
                cx,
                "gallery.m02.pg.current",
                groups::page_item(5, PgIx::Current),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::page_item(3, PgIx::Disabled),
            ),
            kit::cell(
                cx,
                "gallery.m02.pg.nav",
                h_flex()
                    .gap_1()
                    .child(groups::page_nav(IconName::ChevronLeft, PgIx::Disabled))
                    .child(groups::page_nav(IconName::ChevronRight, PgIx::Rest)),
            ),
            kit::cell(cx, "gallery.m02.pg.ellipsis", groups::page_ellipsis()),
        ]);
        let pg_group = kit::group(
            cx,
            "gallery.m02.pg.title",
            "gallery.m02.pg.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.pg.live", live_pg))
                .child(kit::cell(
                    cx,
                    "gallery.m02.pg.live_compact",
                    live_pg_compact,
                ))
                .child(pg_states),
        );

        // ── M19 · stepper ────────────────────────────────────────────────────
        let step_text = |t: &str, d: &str| {
            v_flex()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(gpui_kit::FontWeight::MEDIUM)
                        .text_color(
                            bookkeepy_ui::api::language::role_hsla(mode, "foreground")
                                .unwrap_or_default(),
                        )
                        .child(tr(cx, t)),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(
                            bookkeepy_ui::api::language::role_hsla(mode, "muted-foreground")
                                .unwrap_or_default(),
                        )
                        .child(tr(cx, d)),
                )
        };
        let live_stepper = Stepper::new("m2-st")
            .selected_index(self.step)
            .on_click(cx.listener(|v, ix: &usize, _window, cx| {
                v.step = *ix;
                cx.notify();
            }))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s1", "gallery.m02.st.s1d")))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s2", "gallery.m02.st.s2d")))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s3", "gallery.m02.st.s3d")))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s4", "gallery.m02.st.s4d")));
        let live_stepper_v = Stepper::new("m2-stv")
            .vertical()
            .selected_index(self.step)
            .on_click(cx.listener(|v, ix: &usize, _window, cx| {
                v.step = *ix;
                cx.notify();
            }))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s1", "gallery.m02.st.s1d")))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s2", "gallery.m02.st.s2d")))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s3", "gallery.m02.st.s3d")))
            .item(StepperItem::new().child(step_text("gallery.m02.st.s4", "gallery.m02.st.s4d")));
        let st_states = kit::row([
            kit::cell(
                cx,
                "gallery.m02.st.done",
                groups::step(
                    1,
                    tr(cx, "gallery.m02.st.s1"),
                    tr(cx, "gallery.m02.st.s1d"),
                    StIx::Done,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.st.current",
                groups::step(
                    2,
                    tr(cx, "gallery.m02.st.s2"),
                    tr(cx, "gallery.m02.st.s2d"),
                    StIx::Current,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.st.upcoming",
                groups::step(
                    3,
                    tr(cx, "gallery.m02.st.s3"),
                    tr(cx, "gallery.m02.st.s3d"),
                    StIx::Upcoming,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.invalid",
                groups::step(
                    3,
                    tr(cx, "gallery.m02.st.s3"),
                    tr(cx, "gallery.m02.st.s3err"),
                    StIx::Error,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::step(
                    4,
                    tr(cx, "gallery.m02.st.s4"),
                    tr(cx, "gallery.m02.st.s4d"),
                    StIx::Disabled,
                    mode,
                ),
            ),
        ]);
        let st_group = kit::group(
            cx,
            "gallery.m02.st.title",
            "gallery.m02.st.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(
                    cx,
                    "gallery.m02.st.live",
                    div().w(px(720.)).child(live_stepper),
                ))
                .child(kit::cell(cx, "gallery.m02.st.live_v", live_stepper_v))
                .child(st_states),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.m02.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.m02.title")))
            .child(kit::caption(cx, tr(cx, "gallery.m02.subtitle")))
            .child(tg_group)
            .child(rg_group)
            .child(ig_group)
            .child(dd_group)
            .child(hc_group)
            .child(cl_group)
            .child(acc_group)
            .child(al_group)
            .child(dl_group)
            .child(cb_group)
            .child(pg_group)
            .child(st_group)
    }
}

/// Static (non-localized) menu labels for the live dropdown — menu items are
/// built once per open and need plain strings.
fn tr_static(key: &str) -> gpui_kit::SharedString {
    match key {
        "gallery.m02.dd.bank" => "Bank statement (CSV)".into(),
        "gallery.m02.dd.ofx" => "OFX / QIF file".into(),
        "gallery.m02.dd.bk" => "Another Bookkeepy file".into(),
        _ => key.into(),
    }
}
