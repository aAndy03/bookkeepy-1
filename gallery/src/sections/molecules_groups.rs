// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 02 sheet — toggle group, radio group, input group, dropdown
//! button, hover card, collapsible, accordion, alert, description list,
//! clipboard, pagination, stepper (references/designlanguage).
//!
//! FLAT on the page background like the design frames. Rest cells are LIVE
//! stock components; hover/pressed/focus/invalid/open/loading/disabled ladders
//! are painted from `language::groups` — same tokens, same geometry.

use bookkeepy_ui::api::language::groups::{
    self, AlertTone, CbIx, ClIx, DdIx, IgIx, PgIx, RgIx, StIx, TgIx,
};
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

/// Painted toggle group with one item forced into a state.
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

impl gpui_kit::Render for MoleculesGroups {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── M8 · toggle group ────────────────────────────────────────────────
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
        let live_tg = live_toggle_group(self, cx);
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
        let rg_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                groups::radio_group(
                    tr(cx, "gallery.m02.rg.label"),
                    Some(tr(cx, "gallery.m02.rg.help")),
                    None,
                    v_flex().gap_2().children([
                        groups::radio_option(tr(cx, "gallery.m02.rg.never"), RgIx::Rest),
                        groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), RgIx::Checked),
                        groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), RgIx::Rest),
                    ]),
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
                    v_flex().gap_2().children([
                        groups::radio_option(tr(cx, "gallery.m02.rg.never"), RgIx::Rest),
                        groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), RgIx::Checked),
                        groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), RgIx::Hover),
                    ]),
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
                    v_flex().gap_2().children([
                        groups::radio_option(tr(cx, "gallery.m02.rg.never"), RgIx::Rest),
                        groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), RgIx::Focus),
                        groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), RgIx::Rest),
                    ]),
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
                    v_flex().gap_2().children([
                        groups::radio_option(tr(cx, "gallery.m02.rg.never"), RgIx::Rest),
                        groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), RgIx::Invalid),
                        groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), RgIx::Rest),
                    ]),
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
                    v_flex().gap_2().children([
                        groups::radio_option(tr(cx, "gallery.m02.rg.never"), RgIx::Disabled),
                        groups::radio_option(tr(cx, "gallery.m02.rg.monthly"), RgIx::Disabled),
                        groups::radio_option(tr(cx, "gallery.m02.rg.yearly"), RgIx::Disabled),
                    ]),
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
            .addon(InputGroupAddon::new("m2-ig-pre").child(InputGroupText::new().child("RON")))
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
                                .child(groups::ig_prefix("RON", ix, mode))
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
        let dd_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.hover_main",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::HoverMain,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.hover_caret",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::HoverCaret,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.open",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::Open,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.loading",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::Loading,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::FocusMain,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    true,
                    DdIx::Disabled,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.dd.secondary",
                groups::dropdown_button(
                    tr(cx, "gallery.m02.dd.import"),
                    IconName::Upload,
                    false,
                    DdIx::Rest,
                    mode,
                ),
            ),
        ]);
        let dd_group = kit::group(
            cx,
            "gallery.m02.dd.title",
            "gallery.m02.dd.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.dd.live", live_dd))
                .child(dd_states),
        );

        // ── M12 · hover card ─────────────────────────────────────────────────
        let hc_card = groups::hover_card(
            v_flex()
                .gap(px(12.))
                .child(groups::hc_head(
                    IconName::Landmark,
                    tr(cx, "gallery.m02.hc.bank"),
                    tr(cx, "gallery.m02.hc.account"),
                    mode,
                ))
                .child(
                    h_flex()
                        .gap(px(12.))
                        .child(groups::hc_stat(
                            tr(cx, "gallery.m02.hc.balance"),
                            "12.480,55",
                            "RON",
                            bookkeepy_ui::api::language::role_hsla(mode, "foreground")
                                .unwrap_or_default(),
                            mode,
                        ))
                        .child(groups::hc_stat(
                            tr(cx, "gallery.m02.hc.in_month"),
                            "+4.210,00",
                            "RON",
                            bookkeepy_ui::api::language::prim_hsla("green-500").unwrap_or_default(),
                            mode,
                        ))
                        .child(groups::hc_stat(
                            tr(cx, "gallery.m02.hc.out_month"),
                            "-2.980,25",
                            "RON",
                            bookkeepy_ui::api::language::role_hsla(mode, "foreground")
                                .unwrap_or_default(),
                            mode,
                        )),
                )
                .child(groups::hc_foot(tr(cx, "gallery.m02.hc.synced"))),
            mode,
        );
        let live_hc = HoverCard::new("m2-hc")
            .open_delay(std::time::Duration::from_millis(500))
            .close_delay(std::time::Duration::from_millis(300))
            .trigger(
                div()
                    .text_size(px(12.))
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .text_color(
                        bookkeepy_ui::api::language::role_hsla(mode, "foreground")
                            .unwrap_or_default(),
                    )
                    .child(tr(cx, "gallery.m02.hc.bank")),
            )
            .content(move |_state, _window, _cx| {
                groups::hover_card(
                    v_flex()
                        .gap(px(12.))
                        .child(groups::hc_head(
                            IconName::Landmark,
                            "BT Pay",
                            "current account · …4021",
                            mode,
                        ))
                        .child(
                            h_flex().gap(px(12.)).child(groups::hc_stat(
                                "Balance",
                                "12.480,55",
                                "RON",
                                bookkeepy_ui::api::language::role_hsla(mode, "foreground")
                                    .unwrap_or_default(),
                                mode,
                            )),
                        ),
                    mode,
                )
            });
        let hc_group = kit::group(
            cx,
            "gallery.m02.hc.title",
            "gallery.m02.hc.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.hc.live", live_hc))
                .child(kit::cell(cx, "gallery.atoms.state.open", hc_card)),
        );

        // ── M13 · collapsible ────────────────────────────────────────────────
        let cl_body = v_flex()
            .gap(px(4.))
            .p(px(8.))
            .pl(px(24.))
            .w_full()
            .children(
                [
                    tr(cx, "gallery.m02.cl.rent"),
                    tr(cx, "gallery.m02.cl.electricity"),
                    tr(cx, "gallery.m02.cl.gym"),
                ]
                .into_iter()
                .map(|name| {
                    h_flex()
                        .h(px(24.))
                        .gap(px(8.))
                        .items_center()
                        .w_full()
                        .child(div().size(px(6.)).rounded_full().bg(
                            bookkeepy_ui::api::language::prim_hsla("amber-500").unwrap_or_default(),
                        ))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(
                                    bookkeepy_ui::api::language::role_hsla(mode, "foreground")
                                        .unwrap_or_default(),
                                )
                                .child(name),
                        )
                }),
            );
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
            .content(cl_body);
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
                groups::cl_trigger(tr(cx, "gallery.m02.cl.title"), Some(3), ClIx::Open, mode),
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
        let live_acc = Accordion::new("m2-acc")
            .w(px(360.))
            .on_toggle_click(cx.listener(|v, open: &[usize], _window, cx| {
                v.acc = open.first().copied().unwrap_or(usize::MAX);
                cx.notify();
            }))
            .item(|i| {
                i.title(tr(cx, "gallery.m02.acc.q1"))
                    .open(self.acc == 0)
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(
                                bookkeepy_ui::api::language::role_hsla(
                                    mode,
                                    "secondary-foreground",
                                )
                                .unwrap_or_default(),
                            )
                            .child(tr(cx, "gallery.m02.acc.a1")),
                    )
            })
            .item(|i| {
                i.title(tr(cx, "gallery.m02.acc.q2"))
                    .open(self.acc == 1)
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(
                                bookkeepy_ui::api::language::role_hsla(
                                    mode,
                                    "secondary-foreground",
                                )
                                .unwrap_or_default(),
                            )
                            .child(tr(cx, "gallery.m02.acc.a2")),
                    )
            })
            .item(|i| {
                i.title(tr(cx, "gallery.m02.acc.q3"))
                    .open(self.acc == 2)
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(
                                bookkeepy_ui::api::language::role_hsla(
                                    mode,
                                    "secondary-foreground",
                                )
                                .unwrap_or_default(),
                            )
                            .child(tr(cx, "gallery.m02.acc.a3")),
                    )
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
        let acc_group = kit::group(
            cx,
            "gallery.m02.acc.title",
            "gallery.m02.acc.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.acc.live", live_acc))
                .child(acc_states),
        );

        // ── M15 · alert ──────────────────────────────────────────────────────
        let live_alerts = h_flex()
            .gap(px(12.))
            .items_start()
            .child(
                Alert::info("m2-al-info", tr(cx, "gallery.m02.alert.msg"))
                    .title(tr(cx, "gallery.m02.alert.title"))
                    .on_close(|_, _, _| {}),
            )
            .child(
                Alert::error("m2-al-danger", tr(cx, "gallery.m02.alert.msg_err"))
                    .title(tr(cx, "gallery.m02.alert.title_err"))
                    .on_close(|_, _, _| {}),
            );
        let al_tones = kit::row([
            kit::cell(
                cx,
                "gallery.m02.alert.neutral",
                groups::alert_card(
                    AlertTone::Neutral,
                    IconName::CloudOff,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg"),
                    Some(tr(cx, "gallery.m02.alert.review")),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.info",
                groups::alert_card(
                    AlertTone::Info,
                    IconName::Info,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg"),
                    Some(tr(cx, "gallery.m02.alert.review")),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.success",
                groups::alert_card(
                    AlertTone::Success,
                    IconName::CircleCheck,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg"),
                    Some(tr(cx, "gallery.m02.alert.review")),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.warning",
                groups::alert_card(
                    AlertTone::Warning,
                    IconName::CircleAlert,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg"),
                    Some(tr(cx, "gallery.m02.alert.review")),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.danger",
                groups::alert_card(
                    AlertTone::Danger,
                    IconName::CircleX,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg_err"),
                    Some(tr(cx, "gallery.m02.alert.review")),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.accent",
                groups::alert_card(
                    AlertTone::Accent,
                    IconName::Info,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg"),
                    Some(tr(cx, "gallery.m02.alert.review")),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m02.alert.banner",
                groups::alert_card(
                    AlertTone::Warning,
                    IconName::CircleAlert,
                    tr(cx, "gallery.m02.alert.title"),
                    tr(cx, "gallery.m02.alert.msg"),
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
        fn dl_value(
            text: gpui_kit::SharedString,
            mode: bookkeepy_primary_data::theme::Mode,
        ) -> gpui_kit::Div {
            div()
                .text_size(px(12.))
                .text_color(
                    bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default(),
                )
                .child(text)
        }
        let live_dl = DescriptionList::new()
            .bordered(false)
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
            );
        let fg = bookkeepy_ui::api::language::role_hsla(mode, "foreground").unwrap_or_default();
        let dl_painted = kit::row([
            kit::cell(
                cx,
                "gallery.m02.dl.horizontal",
                v_flex().children([
                    groups::dl_row(
                        tr(cx, "gallery.m02.dl.payee"),
                        dl_value(tr(cx, "gallery.m02.dl.lidl"), mode),
                        mode,
                    ),
                    groups::dl_row(
                        tr(cx, "gallery.m02.dl.amount"),
                        groups::dl_amount(
                            tr(cx, "gallery.m02.dl.minus"),
                            tr(cx, "gallery.m02.dl.amount_v"),
                            tr(cx, "gallery.m02.dl.ron"),
                            fg,
                            mode,
                        ),
                        mode,
                    ),
                ]),
            ),
            kit::cell(
                cx,
                "gallery.m02.dl.vertical",
                h_flex().gap(px(12.)).children([
                    groups::dl_cell(
                        tr(cx, "gallery.m02.dl.opened"),
                        tr(cx, "gallery.m02.dl.opened_v"),
                        mode,
                    ),
                    groups::dl_cell(
                        tr(cx, "gallery.m02.dl.currency"),
                        tr(cx, "gallery.m02.dl.ron"),
                        mode,
                    ),
                ]),
            ),
        ]);
        let dl_group = kit::group(
            cx,
            "gallery.m02.dl.title",
            "gallery.m02.dl.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m02.dl.live_h", live_dl))
                .child(kit::cell(cx, "gallery.m02.dl.live_v", live_dlv))
                .child(dl_painted),
        );

        // ── M17 · clipboard ─────────────────────────────────────────────────
        let live_cb = Clipboard::new("m2-cb")
            .value("RO49 BTRL 0000 1234 5678 4021")
            .label(tr(cx, "gallery.m02.cb.iban"));
        let cb_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                groups::clipboard_chip("RO49 BTRL 0000 1234 5678 4021", CbIx::Idle, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                groups::clipboard_chip("RO49 BTRL 0000 1234 5678 4021", CbIx::Hover, mode),
            ),
            kit::cell(
                cx,
                "gallery.m02.cb.copied",
                v_flex()
                    .gap_2()
                    .items_end()
                    .child(groups::clipboard_chip(
                        "RO49 BTRL 0000 1234 5678 4021",
                        CbIx::Copied,
                        mode,
                    ))
                    .child(groups::clipboard_tooltip(tr(
                        cx,
                        "gallery.m02.cb.copied_tt",
                    ))),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::clipboard_chip("RO49 BTRL 0000 1234 5678 4021", CbIx::Focus, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                groups::clipboard_chip("RO49 BTRL 0000 1234 5678 4021", CbIx::Disabled, mode),
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
                "gallery.m02.pg.current",
                groups::page_item(5, PgIx::Current),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                groups::page_item(3, PgIx::Focus),
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
                    .child(groups::page_nav(IconName::ChevronLeft, PgIx::Rest))
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
                    tr(cx, "gallery.m02.st.s3d"),
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
