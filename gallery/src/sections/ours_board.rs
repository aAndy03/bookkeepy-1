// SPDX-License-Identifier: GPL-3.0-or-later
//! ORGANISMS · bookkeepy custom — board cards: account card, net worth card, to-do list
//! (references/designlanguage). Only D1–D3 of that sheet.
//!
//! FLAT on the page background; the cards ARE the components (surface-raised frame),
//! painted through the reusable `language::custom` painters.

use bookkeepy_ui::api::language::{custom, prim_hsla, role_hsla};
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
    cx.new(|_cx| OursBoard).into()
}

pub struct OursBoard;

fn cap(cx: &gpui_kit::App, key: &str) -> gpui_kit::Div {
    kit::overline(cx, tr(cx, key))
}

/// 16 diverging in/out bar pairs (D1).
fn acct_pairs() -> [(f32, f32); 16] {
    [
        (4., 15.791),
        (18.747, 12.128),
        (4.915, 6.),
        (12.932, 7.853),
        (9.98, 4.808),
        (11.256, 10.063),
        (14.017, 15.741),
        (6.649, 14.888),
        (14.734, 9.416),
        (2., 2.),
        (2., 2.),
        (2., 2.),
        (6.861, 2.92),
        (17.135, 9.657),
        (4.077, 12.984),
        (6.944, 6.274),
    ]
}

/// 36 trend bar heights ascending to the current bar (D2).
fn trend() -> [f32; 36] {
    [
        20., 22., 21., 24., 23., 26., 25., 28., 27., 30., 29., 32., 31., 34., 33., 36., 35., 38.,
        37., 40., 39., 42., 41., 44., 43., 46., 45., 48., 47., 50., 52., 55., 58., 62., 66.,
        71.215,
    ]
}

impl gpui_kit::Render for OursBoard {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let muted = role_hsla(mode, "muted-foreground").unwrap_or_default();
        let white_a4 = prim_hsla("white-a4").unwrap_or_default();

        // ── D1 · account card ───────────────────────────────────────────────
        let acct = |key: &str, name: &str, digits: &str, ix: custom::AcctIx| {
            v_flex()
                .gap(px(4.))
                .child(cap(cx, key))
                .child(custom::account_card(
                    custom::AcctCard {
                        name: SharedString::from(name),
                        period: tr(cx, "gallery.ours.d1.period"),
                        sign: String::new(),
                        digits: digits.to_string(),
                        currency: "RON".to_string(),
                        pairs: acct_pairs().to_vec(),
                        ix,
                    },
                    mode,
                ))
        };
        let d1_states = h_flex().gap(px(32.)).children([
            acct(
                "gallery.ours.d1.rest",
                "BT Pay",
                "300,00",
                custom::AcctIx::Rest,
            ),
            acct(
                "gallery.ours.d1.hover",
                "Revolut",
                "20.967,00",
                custom::AcctIx::Hover,
            ),
            acct(
                "gallery.ours.d1.selected",
                "Savings",
                "25.813,00",
                custom::AcctIx::Selected,
            ),
            acct(
                "gallery.ours.d1.hidden",
                "Cash",
                "1.240,00",
                custom::AcctIx::Hidden,
            ),
            acct(
                "gallery.ours.d1.loading",
                "BT Pay",
                "300,00",
                custom::AcctIx::Loading,
            ),
        ]);
        let d1_group = kit::group(
            cx,
            "gallery.ours.d1.title",
            "gallery.ours.d1.hint",
            v_flex()
                .gap_2()
                .child(cap(cx, "gallery.ours.d1.states"))
                .child(d1_states),
        );

        // ── D2 · net worth card ─────────────────────────────────────────────
        let nw = |key: &str,
                  sign: &str,
                  digits: &str,
                  dsign: &str,
                  ddigits: &str,
                  cap_key: &str,
                  ix: custom::NwIx| {
            v_flex()
                .gap(px(4.))
                .child(cap(cx, key))
                .child(custom::net_worth_card(
                    custom::NwCard {
                        sign: sign.to_string(),
                        digits: digits.to_string(),
                        currency: "RON".to_string(),
                        delta_sign: dsign.to_string(),
                        delta_digits: ddigits.to_string(),
                        delta_cap: tr(cx, cap_key),
                        trend: trend().to_vec(),
                        assets: "52.100".to_string(),
                        debts: "3.780".to_string(),
                        ix,
                    },
                    mode,
                ))
        };
        let d2_states = h_flex().gap(px(32.)).children([
            nw(
                "gallery.ours.d2.now",
                "",
                "54.320,00",
                "+",
                "2.140",
                "gallery.ours.d2.month",
                custom::NwIx::Now,
            ),
            nw(
                "gallery.ours.d2.hidden",
                "",
                "54.320,00",
                "+",
                "2.140",
                "gallery.ours.d2.month",
                custom::NwIx::Hidden,
            ),
            nw(
                "gallery.ours.d2.forecast",
                "≈",
                "56.720,00",
                "≈",
                "2.400",
                "gallery.ours.d2.fcm",
                custom::NwIx::Forecast,
            ),
        ]);
        let d2_group = kit::group(
            cx,
            "gallery.ours.d2.title",
            "gallery.ours.d2.hint",
            v_flex()
                .gap_2()
                .child(cap(cx, "gallery.ours.d2.states"))
                .child(d2_states),
        );

        // ── D3 · to-do list ─────────────────────────────────────────────────
        let todo = custom::todo_row;
        let d3_list = v_flex()
            .w(px(420.))
            .p(px(12.))
            .gap(px(2.))
            .rounded(px(16.))
            .bg(role_hsla(mode, "surface-raised").unwrap_or_default())
            .border_1()
            .border_color(white_a4)
            .child(
                h_flex()
                    .items_center()
                    .gap(px(8.))
                    .p(px(4.))
                    .pl(px(12.))
                    .pb(px(8.))
                    .child(txt_overline(cx, tr(cx, "gallery.ours.d3.todo")))
                    .child(custom::count_pill("4")),
            )
            .child(todo(
                IconName::Receipt,
                custom::TodoTone::Accent,
                true,
                tr(cx, "gallery.ours.d3.t1"),
                tr(cx, "gallery.ours.d3.t1c"),
                prim_hsla("white-a4").unwrap_or_default(),
                mode,
            ))
            .child(todo(
                IconName::Receipt,
                custom::TodoTone::Warning,
                false,
                tr(cx, "gallery.ours.d3.t2"),
                tr(cx, "gallery.ours.d3.t2c"),
                gpui_kit::transparent_white(),
                mode,
            ))
            .child(todo(
                IconName::Repeat,
                custom::TodoTone::Neutral,
                false,
                tr(cx, "gallery.ours.d3.t3"),
                tr(cx, "gallery.ours.d3.t3c"),
                gpui_kit::transparent_white(),
                mode,
            ))
            .child(todo(
                IconName::FileText,
                custom::TodoTone::Neutral,
                false,
                tr(cx, "gallery.ours.d3.t4"),
                tr(cx, "gallery.ours.d3.t4c"),
                gpui_kit::transparent_white(),
                mode,
            ));
        // Empty state: success glyph + heading + body inside the same card frame.
        let d3_empty = v_flex()
            .w(px(420.))
            .p(px(12.))
            .rounded(px(16.))
            .bg(role_hsla(mode, "surface-raised").unwrap_or_default())
            .border_1()
            .border_color(white_a4)
            .child(
                v_flex()
                    .w(px(360.))
                    .p(px(32.))
                    .pl(px(24.))
                    .pr(px(24.))
                    .items_center()
                    .gap(px(12.))
                    .child(
                        bookkeepy_ui::api::language::atoms::duotone_icon(
                            IconName::CircleCheck,
                            24.,
                        )
                        .text_color(role_hsla(mode, "money-in").unwrap_or_default()),
                    )
                    .child(
                        v_flex()
                            .items_center()
                            .gap(px(6.))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .line_height(px(21.))
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                                    .child(tr(cx, "gallery.ours.d3.empty_t")),
                            )
                            .child(
                                div()
                                    .w(px(280.))
                                    .text_center()
                                    .text_size(px(12.))
                                    .line_height(px(18.))
                                    .text_color(muted)
                                    .child(tr(cx, "gallery.ours.d3.empty_d")),
                            ),
                    ),
            );
        let d3_group = kit::group(
            cx,
            "gallery.ours.d3.title",
            "gallery.ours.d3.hint",
            v_flex().gap_8().children([
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.d3.list"))
                    .child(d3_list),
                v_flex()
                    .gap_2()
                    .child(cap(cx, "gallery.ours.d3.empty"))
                    .child(d3_empty),
            ]),
        );

        v_flex()
            .size_full()
            .p(px(48.))
            .gap_12()
            .child(d1_group)
            .child(d2_group)
            .child(d3_group)
    }
}

/// Overline-styled text helper (10/500, muted).
fn txt_overline(cx: &gpui_kit::App, text: impl Into<gpui_kit::SharedString>) -> gpui_kit::Div {
    kit::overline(cx, text)
}
