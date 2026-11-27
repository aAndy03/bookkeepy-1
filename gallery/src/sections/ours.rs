// SPDX-License-Identifier: GPL-3.0-or-later
//! COMPONENTS · bookkeepy custom sheet — our own atoms, molecules, board parts
//! (references/designlanguage). C4 "Day tick" intentionally excluded.
//!
//! FLAT on the page background. Every specimen is painted through the reusable
//! `language::custom` painters (C1 amount, C2 dot, C3 direction, C5 switcher,
//! C6 location chip, C8 zoom, C9 tucked tag, C10 sticky note); C7 reuses
//! `organisms::sync_line`; C1 also demos the live SSOT `Amount` widget.

use bookkeepy_core::{CurrencyCode, Money};
use bookkeepy_ui::api::language::popups::{MenuIx, menu_item, menu_separator, popup_surface};
use bookkeepy_ui::api::language::{custom, organisms, prim_hsla, role_hsla};
use bookkeepy_ui::api::widgets::atoms::amount::Amount;
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::{
    AnyView, AppContext as _, Context, ParentElement as _, SharedString, Styled as _, Window, div,
    px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| Ours).into()
}

pub struct Ours;

/// Caption cell used across the sheet (specimen label under/over the part).
fn cap(cx: &gpui_kit::App, key: &str) -> gpui_kit::Div {
    kit::overline(cx, tr(cx, key))
}

impl gpui_kit::Render for Ours {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let fg = role_hsla(mode, "foreground").unwrap_or_default();
        let muted = role_hsla(mode, "muted-foreground").unwrap_or_default();
        let subtle = role_hsla(mode, "subtle-foreground").unwrap_or_default();
        let money_in = role_hsla(mode, "money-in").unwrap_or_default();
        let money_out = role_hsla(mode, "money-out").unwrap_or_default();
        let money_tr = role_hsla(mode, "money-transfer").unwrap_or_default();
        let money_pend = role_hsla(mode, "money-pending").unwrap_or_default();
        let money_fc = role_hsla(mode, "money-forecast").unwrap_or_default();
        let accent = role_hsla(mode, "accent").unwrap_or_default();
        let time_past = prim_hsla("sand-500").unwrap_or_default();

        // ── C1 · amount ─────────────────────────────────────────────────────
        let money_cell =
            |key: &str, sign: &str, digits: &str, ink: gpui_kit::Hsla, masked: bool| {
                v_flex()
                    .w(px(160.))
                    .gap(px(4.))
                    .child(custom::amount(
                        sign,
                        digits,
                        "RON",
                        ink,
                        custom::AmountSz::Sm,
                        masked,
                        mode,
                    ))
                    .child(
                        div()
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(subtle)
                            .child(tr(cx, key)),
                    )
            };
        let c1_money = h_flex().gap(px(32.)).children([
            money_cell("gallery.ours.c1.out", "−", "142,50", money_out, false),
            money_cell("gallery.ours.c1.in", "+", "8.400,00", money_in, false),
            money_cell("gallery.ours.c1.tr", "", "1.000,00", money_tr, false),
            money_cell("gallery.ours.c1.pend", "−", "55,00", money_pend, false),
            money_cell("gallery.ours.c1.fc", "≈", "2.190,00", money_fc, false),
            money_cell("gallery.ours.c1.hidden", "−", "142,50", money_out, true),
        ]);
        let c1_sizes = h_flex().items_end().gap(px(32.)).children([
            v_flex()
                .gap(px(4.))
                .child(custom::amount(
                    "−",
                    "142,50",
                    "RON",
                    money_out,
                    custom::AmountSz::Xl,
                    false,
                    mode,
                ))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c1.xl")),
                ),
            v_flex()
                .gap(px(4.))
                .child(custom::amount(
                    "−",
                    "142,50",
                    "RON",
                    money_out,
                    custom::AmountSz::Md,
                    false,
                    mode,
                ))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c1.md")),
                ),
            v_flex()
                .gap(px(4.))
                .child(custom::amount(
                    "−",
                    "142,50",
                    "RON",
                    money_out,
                    custom::AmountSz::Sm,
                    false,
                    mode,
                ))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c1.sm")),
                ),
        ]);
        // Live SSOT widget (ADR-0017): unsigned + signed variant + zero edge case.
        // ISO-shape literals cannot fail; the OnceLock pattern from the old section.
        static RON: std::sync::OnceLock<CurrencyCode> = std::sync::OnceLock::new();
        let ron = RON
            .get_or_init(|| match CurrencyCode::new("RON") {
                Ok(code) => code,
                Err(_) => unreachable!("RON is a valid ISO shape (CurrencyCode::new)"),
            })
            .clone();
        let c1_live = h_flex().items_end().gap(px(32.)).children([
            v_flex()
                .gap(px(4.))
                .child(Amount::new(Money::new(-14250, ron.clone())))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c1.live")),
                ),
            v_flex()
                .gap(px(4.))
                .child(Amount::new(Money::new(840000, ron.clone())).signed())
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c1.signed")),
                ),
            v_flex()
                .gap(px(4.))
                .child(Amount::new(Money::zero(ron)))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c1.zero")),
                ),
        ]);
        let c1_group = kit::group(
            cx,
            "gallery.ours.c1.title",
            "gallery.ours.c1.hint",
            v_flex().gap_8().children([
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c1.money"))
                    .child(c1_money),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c1.sizes"))
                    .child(c1_sizes),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c1.widget"))
                    .child(c1_live),
            ]),
        );

        // ── C2 · category dot ───────────────────────────────────────────────
        let palette: [(&str, &str, &str); 8] = [
            ("data-1", "groceries", "gallery.ours.c2.groceries"),
            ("data-2", "transport", "gallery.ours.c2.transport"),
            ("data-3", "fun", "gallery.ours.c2.fun"),
            ("data-4", "subscriptions", "gallery.ours.c2.subs"),
            ("data-5", "eating out", "gallery.ours.c2.eat"),
            ("data-6", "health", "gallery.ours.c2.health"),
            ("data-7", "gifts", "gallery.ours.c2.gifts"),
            ("data-8", "home", "gallery.ours.c2.home"),
        ];
        let c2_palette = h_flex()
            .gap(px(32.))
            .children(palette.iter().map(|(alias, _, key)| {
                v_flex()
                    .items_center()
                    .gap(px(4.))
                    .child(custom::cat_dot(
                        8.,
                        prim_hsla(alias).unwrap_or_default(),
                        false,
                    ))
                    .child(
                        div()
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(role_hsla(mode, "secondary-foreground").unwrap_or_default())
                            .child(tr(cx, key)),
                    )
            }));
        let pink = prim_hsla("pink-500").unwrap_or_default();
        let c2_sizes = h_flex().items_end().gap(px(32.)).children([
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::cat_dot(6., pink, false))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child("6"),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::cat_dot(8., pink, false))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child("8"),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::cat_dot(10., pink, false))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child("10"),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::cat_dot(8., pink, true))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c2.sel")),
                ),
        ]);
        let c2_group = kit::group(
            cx,
            "gallery.ours.c2.title",
            "gallery.ours.c2.hint",
            v_flex().gap_8().children([
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c2.palette"))
                    .child(c2_palette),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c2.sizes"))
                    .child(c2_sizes),
            ]),
        );

        // ── C3 · money direction mark ───────────────────────────────────────
        let c3_marks = h_flex().gap(px(32.)).children([
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::dir_mark(
                    IconName::ArrowUpRight,
                    role_hsla(mode, "money-out").unwrap_or_default(),
                ))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c3.out")),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::dir_mark(IconName::ArrowDownLeft, money_in))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c3.in")),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::dir_mark(IconName::ArrowLeftRight, money_tr))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c3.tr")),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::dir_mark(IconName::Clock, money_pend))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c3.pend")),
                ),
            v_flex()
                .items_center()
                .gap(px(4.))
                .child(custom::dir_mark(IconName::Clock, money_fc))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, "gallery.ours.c3.fc")),
                ),
        ]);
        let c3_rows = h_flex().gap(px(16.)).children([
            custom::dir_plate(
                IconName::ArrowUpRight,
                money_out,
                "−",
                "142,50",
                "RON",
                mode,
            ),
            custom::dir_plate(
                IconName::ArrowDownLeft,
                money_in,
                "+",
                "8.400,00",
                "RON",
                mode,
            ),
            custom::dir_plate(
                IconName::ArrowLeftRight,
                money_tr,
                "",
                "1.000,00",
                "RON",
                mode,
            ),
            custom::dir_plate(IconName::Clock, money_pend, "−", "55,00", "RON", mode),
        ]);
        let c3_group = kit::group(
            cx,
            "gallery.ours.c3.title",
            "gallery.ours.c3.hint",
            v_flex().gap_8().children([
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c3.marks"))
                    .child(c3_marks),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c3.inrow"))
                    .child(c3_rows),
            ]),
        );

        // ── molecules intro · nav item ──────────────────────────────────────
        let nav_group = kit::group(
            cx,
            "gallery.ours.nav.title",
            "gallery.ours.nav.hint",
            v_flex()
                .gap_2()
                .child(div().w(px(220.)).child(organisms::nav_item(
                    Some(IconName::List),
                    Some(tr(cx, "gallery.ours.nav.transactions")),
                    None,
                    organisms::NavIx::Rest,
                    mode,
                ))),
        );

        // ── C5 · switcher ───────────────────────────────────────────────────
        let c5_states = h_flex().gap(px(32.)).children([
            custom::switcher(
                tr(cx, "gallery.ours.c5.workspace"),
                tr(cx, "gallery.ours.c5.money"),
                None,
                custom::SwIx::Rest,
                mode,
            ),
            custom::switcher(
                tr(cx, "gallery.ours.c5.workspace"),
                tr(cx, "gallery.ours.c5.money"),
                None,
                custom::SwIx::Open,
                mode,
            ),
            custom::switcher(
                tr(cx, "gallery.ours.c5.workspace"),
                tr(cx, "gallery.ours.c5.money"),
                None,
                custom::SwIx::Focus,
                mode,
            ),
            custom::switcher(
                tr(cx, "gallery.ours.c5.workspace"),
                tr(cx, "gallery.ours.c5.money"),
                None,
                custom::SwIx::Disabled,
                mode,
            ),
        ]);
        let c5_profile = custom::switcher(
            tr(cx, "gallery.ours.c5.profile"),
            tr(cx, "gallery.ours.c5.andy"),
            Some(("AM", "cyan-900", "cyan-300")),
            custom::SwIx::Open,
            mode,
        );
        // Open menu popup (painted): popover surface, checked current item, New profile.
        let menu = v_flex().child(popup_surface(220., 4., mode).child(
            v_flex().gap(px(2.)).children([
                menu_item(
                    tr(cx, "gallery.ours.c5.andy"),
                    None,
                    true,
                    None,
                    MenuIx::Hover,
                    mode,
                ),
                menu_item(
                    tr(cx, "gallery.ours.c5.side"),
                    None,
                    false,
                    None,
                    MenuIx::Rest,
                    mode,
                ),
                menu_separator(),
                menu_item(
                    tr(cx, "gallery.ours.c5.new"),
                    Some(IconName::Plus),
                    false,
                    None,
                    MenuIx::Rest,
                    mode,
                ),
            ]),
        ));
        let c5_group = kit::group(
            cx,
            "gallery.ours.c5.title",
            "gallery.ours.c5.hint",
            v_flex().gap_8().children([
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c5.states"))
                    .child(c5_states),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c5.open"))
                    .child(
                        h_flex()
                            .items_start()
                            .gap(px(24.))
                            .child(c5_profile)
                            .child(menu),
                    ),
            ]),
        );

        // ── C6 · location chip ──────────────────────────────────────────────
        let c6_chips = v_flex()
            .p(px(16.))
            .gap(px(32.))
            .rounded(px(12.))
            .bg(role_hsla(mode, "surface").unwrap_or_default())
            .w(px(520.))
            .children([
                custom::loc_chip(
                    &[
                        (tr(cx, "gallery.ours.c6.money"), muted),
                        (tr(cx, "gallery.ours.c6.transactions"), fg),
                    ],
                    None,
                    mode,
                ),
                custom::loc_chip(
                    &[
                        (tr(cx, "gallery.ours.c6.money"), muted),
                        (tr(cx, "gallery.ours.c6.transactions"), fg),
                    ],
                    Some(1),
                    mode,
                ),
                custom::loc_chip(
                    &[
                        (SharedString::from("→"), accent),
                        (tr(cx, "gallery.ours.c6.transactions"), fg),
                    ],
                    None,
                    mode,
                ),
                custom::loc_chip(
                    &[
                        (tr(cx, "gallery.ours.c6.money"), muted),
                        (tr(cx, "gallery.ours.c6.transactions"), fg),
                        (SharedString::from("· Jul 2026"), time_past),
                    ],
                    None,
                    mode,
                ),
            ]);
        let c6_group = kit::group(
            cx,
            "gallery.ours.c6.title",
            "gallery.ours.c6.hint",
            v_flex()
                .gap_2()
                .child(cap(cx, "gallery.ours.c6.variants"))
                .child(c6_chips),
        );

        // ── C7 · sync line ──────────────────────────────────────────────────
        let plate = |key: &str, el: gpui_kit::Div| {
            v_flex()
                .gap(px(4.))
                .rounded(px(8.))
                .bg(role_hsla(mode, "background").unwrap_or_default())
                .p(px(8.))
                .child(
                    div()
                        .text_size(px(10.))
                        .line_height(px(12.))
                        .text_color(muted)
                        .child(tr(cx, key)),
                )
                .child(el)
        };
        let c7_states = h_flex().gap(px(16.)).children([
            plate(
                "gallery.ours.c7.saved",
                organisms::sync_line(
                    IconName::CircleCheck,
                    subtle,
                    tr(cx, "gallery.ours.c7.saved_t"),
                    subtle,
                    mode,
                ),
            ),
            plate(
                "gallery.ours.c7.syncing",
                organisms::sync_line(
                    IconName::RefreshCw,
                    role_hsla(mode, "secondary-foreground").unwrap_or_default(),
                    tr(cx, "gallery.ours.c7.syncing_t"),
                    role_hsla(mode, "secondary-foreground").unwrap_or_default(),
                    mode,
                ),
            ),
            plate(
                "gallery.ours.c7.offline",
                organisms::sync_line(
                    IconName::CloudOff,
                    subtle,
                    tr(cx, "gallery.ours.c7.offline_t"),
                    subtle,
                    mode,
                ),
            ),
            plate(
                "gallery.ours.c7.paused",
                organisms::sync_line(
                    IconName::CircleAlert,
                    prim_hsla("amber-500").unwrap_or_default(),
                    tr(cx, "gallery.ours.c7.paused_t"),
                    prim_hsla("amber-300").unwrap_or_default(),
                    mode,
                ),
            ),
        ]);
        let c7_group = kit::group(
            cx,
            "gallery.ours.c7.title",
            "gallery.ours.c7.hint",
            v_flex()
                .gap_2()
                .child(cap(cx, "gallery.ours.c7.states"))
                .child(c7_states),
        );

        // ── C8 · zoom control ───────────────────────────────────────────────
        let c8_states = h_flex().gap(px(32.)).children([
            custom::zoom_ctrl("64 %", false, false, false, None, mode),
            custom::zoom_ctrl("64 %", false, false, false, Some(2), mode),
            custom::zoom_ctrl("64 %", false, false, false, Some(1), mode),
            custom::zoom_ctrl("10 %", true, false, false, None, mode),
            custom::zoom_ctrl("400 %", false, true, false, None, mode),
            custom::zoom_ctrl("64 %", false, false, true, None, mode),
        ]);
        let c8_group = kit::group(
            cx,
            "gallery.ours.c8.title",
            "gallery.ours.c8.hint",
            v_flex()
                .gap_2()
                .child(cap(cx, "gallery.ours.c8.states"))
                .child(c8_states),
        );

        // ── C9 · tucked tag ─────────────────────────────────────────────────
        let place = |stroke: gpui_kit::Hsla, dimmed: bool, key: &str| {
            v_flex()
                .gap(px(0.))
                .child(custom::place_card(
                    tr(cx, "gallery.ours.c9.overview"),
                    stroke,
                    dimmed,
                    mode,
                ))
                .child(custom::tucked_tag(
                    tr(cx, "gallery.ours.c9.tag"),
                    dimmed,
                    mode,
                ))
                .child(
                    div()
                        .pt(px(4.))
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(subtle)
                        .child(tr(cx, key)),
                )
        };
        let c9_states = h_flex().gap(px(32.)).children([
            place(
                prim_hsla("white-a4").unwrap_or_default(),
                false,
                "gallery.ours.c9.rest",
            ),
            place(
                prim_hsla("white-a12").unwrap_or_default(),
                false,
                "gallery.ours.c9.hover",
            ),
            place(
                prim_hsla("orange-500").unwrap_or_default().opacity(0.4),
                false,
                "gallery.ours.c9.active",
            ),
            place(
                prim_hsla("white-a4").unwrap_or_default(),
                true,
                "gallery.ours.c9.dimmed",
            ),
        ]);
        let c9_group = kit::group(
            cx,
            "gallery.ours.c9.title",
            "gallery.ours.c9.hint",
            v_flex()
                .gap_2()
                .child(cap(cx, "gallery.ours.c9.states"))
                .child(c9_states),
        );

        // ── C10 · sticky note ───────────────────────────────────────────────
        let note = |bg: gpui_kit::Hsla, ink: gpui_kit::Hsla, editing: bool, dimmed: bool| {
            custom::sticky_note(
                bg,
                ink,
                tr(cx, "gallery.ours.c10.text"),
                tr(cx, "gallery.ours.c10.meta"),
                editing,
                dimmed,
            )
        };
        let note_bg = role_hsla(mode, "board-note").unwrap_or_default();
        let note_ink = prim_hsla("neutral-925").unwrap_or_default();
        let c10_tones = h_flex().gap(px(16.)).children([
            note(note_bg, note_ink, false, false),
            note(
                prim_hsla("violet-900").unwrap_or_default(),
                prim_hsla("violet-300").unwrap_or_default(),
                false,
                false,
            ),
            note(
                prim_hsla("green-900").unwrap_or_default(),
                prim_hsla("green-300").unwrap_or_default(),
                false,
                false,
            ),
            note(
                prim_hsla("amber-900").unwrap_or_default(),
                prim_hsla("amber-300").unwrap_or_default(),
                false,
                false,
            ),
            note(
                prim_hsla("red-900").unwrap_or_default(),
                prim_hsla("red-300").unwrap_or_default(),
                false,
                false,
            ),
            note(
                prim_hsla("cyan-900").unwrap_or_default(),
                prim_hsla("cyan-300").unwrap_or_default(),
                false,
                false,
            ),
        ]);
        let c10_states = h_flex().gap(px(16.)).children([
            note(note_bg, note_ink, true, false),
            note(note_bg, note_ink, false, true),
        ]);
        let c10_group = kit::group(
            cx,
            "gallery.ours.c10.title",
            "gallery.ours.c10.hint",
            v_flex().gap_8().children([
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c10.tones"))
                    .child(c10_tones),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.c10.states"))
                    .child(c10_states),
            ]),
        );

        v_flex()
            .size_full()
            .p(px(48.))
            .gap_12()
            .child(c1_group)
            .child(c2_group)
            .child(c3_group)
            .child(nav_group)
            .child(c5_group)
            .child(c6_group)
            .child(c7_group)
            .child(c8_group)
            .child(c9_group)
            .child(c10_group)
    }
}
