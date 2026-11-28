// SPDX-License-Identifier: GPL-3.0-or-later
//! ORGANISMS · 02 sheet (part 2) — dock + nav stack
//! (references/designlanguage ORGANISMS · 02, blocks O19–O20).
//!
//! FLAT on the page background. Both blocks are painted from
//! `language::organisms` / local helpers — the sheet pins their geometry; the
//! app's real dock gets the same tokens when it adopts `DockArea`.

use bookkeepy_ui::api::language::organisms;
use bookkeepy_ui::api::language::{prim_hsla, role_hsla};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::StyledExt as _;
use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement as _, ParentElement as _, Styled as _, Window,
    div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| OrganismsWorkspaces).into()
}

pub struct OrganismsWorkspaces;

/// One dock panel: 300×360 surface card with tab header (O19).
fn dock_panel(
    tabs: &[&'static str],
    active: usize,
    locked: bool,
    _cx: &gpui_kit::App,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let fg = role_hsla(mode, "foreground").unwrap_or_default();
    let muted = prim_hsla("neutral-400").unwrap_or_default();
    let tab = |label: &'static str, active: bool| {
        let mut t = h_flex()
            .h(px(31.))
            .px(px(4.))
            .items_center()
            .border_b(px(2.))
            .border_color(if active {
                c0("neutral-0")
            } else {
                gpui_kit::transparent_white()
            });
        t = t.child(
            div()
                .refine_style(&kit::label_refinement())
                .line_height(px(14.))
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_color(if active { fg } else { muted })
                .child(label),
        );
        t
    };
    v_flex()
        .w(px(300.))
        .h(px(360.))
        .rounded(px(bookkeepy_ui::api::language::radius_tier("2xl")))
        .border_1()
        .border_color(role_hsla(mode, "border-subtle").unwrap_or_default())
        .bg(role_hsla(mode, "surface").unwrap_or_default())
        .child(
            h_flex()
                .h(px(34.))
                .pl(px(10.))
                .pr(px(4.))
                .gap(px(12.))
                .items_center()
                .border_b(px(1.))
                .border_color(prim_hsla("white-a8").unwrap_or_default())
                .children(tabs.iter().enumerate().map(|(i, t)| tab(t, i == active)))
                .child(div().flex_1())
                .child(if locked {
                    bookkeepy_ui::api::language::atoms::icon(IconName::Lock, 14.)
                        .text_color(muted)
                        .into_any_element()
                } else {
                    organisms::icon_btn(
                        IconName::EllipsisVertical,
                        14.,
                        role_hsla(mode, "icon").unwrap_or_default(),
                        mode,
                    )
                    .into_any_element()
                }),
        )
        .child(v_flex().p(px(12.)).gap(px(8.)).children((0..6).map(|_| {
            div()
                .h(px(22.))
                .w_full()
                .rounded(px(4.))
                .bg(role_hsla(mode, "surface-raised").unwrap_or_default())
        })))
}

fn c0(alias: &str) -> gpui_kit::Hsla {
    prim_hsla(alias).unwrap_or_default()
}

/// The painted O19 dock area: left dock + centre board with accent drop target +
/// bottom collapsed strip + right locked dock.
fn dock_area(cx: &gpui_kit::App, mode: bookkeepy_primary_data::theme::Mode) -> gpui_kit::Div {
    h_flex()
        .w(px(1100.))
        .h(px(420.))
        .p(px(6.))
        .gap(px(6.))
        .rounded(px(bookkeepy_ui::api::language::radius_tier("2xl")))
        .border_1()
        .border_color(role_hsla(mode, "border-subtle").unwrap_or_default())
        .bg(role_hsla(mode, "background").unwrap_or_default())
        .child(dock_panel(&["Files", "Notes"], 0, false, cx, mode))
        .child(
            v_flex()
                .flex_1()
                .gap(px(6.))
                .child(
                    div()
                        .flex_1()
                        .rounded(px(8.))
                        .border_1()
                        .border_color(
                            prim_hsla("orange-500")
                                .map(|mut h| {
                                    h.a = 0.4;
                                    h
                                })
                                .unwrap_or_default(),
                        )
                        .bg(role_hsla(mode, "accent-subtle").unwrap_or_default())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(prim_hsla("orange-300").unwrap_or_default())
                                .child(tr(cx, "gallery.o02.dk.drop")),
                        ),
                )
                .child(
                    h_flex()
                        .h(px(30.))
                        .px(px(8.))
                        .gap(px(6.))
                        .items_center()
                        .child(
                            bookkeepy_ui::api::language::atoms::icon(IconName::Clock, 14.)
                                .text_color(prim_hsla("neutral-400").unwrap_or_default()),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(prim_hsla("neutral-400").unwrap_or_default())
                                .child(tr(cx, "gallery.o02.dk.timeline")),
                        )
                        .child(
                            bookkeepy_ui::api::language::atoms::icon(IconName::ChevronUp, 12.)
                                .text_color(prim_hsla("neutral-400").unwrap_or_default()),
                        ),
                ),
        )
        .child(dock_panel(&["Locked"], 0, true, cx, mode))
}

impl gpui_kit::Render for OrganismsWorkspaces {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── O19 · dock ───────────────────────────────────────────────────────
        let dk_group = kit::group(
            cx,
            "gallery.o02.dk.title",
            "gallery.o02.dk.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.dk.master", dock_area(cx, mode))),
        );

        // ── O20 · nav stack ──────────────────────────────────────────────────
        let ns_page = |caption: &'static str, title: &'static str, back: bool| {
            v_flex()
                .w(px(340.))
                .rounded(px(bookkeepy_ui::api::language::radius_tier("2xl")))
                .bg(role_hsla(mode, "surface").unwrap_or_default())
                .overflow_hidden()
                .child(organisms::navstack_header(
                    back,
                    tr(cx, caption),
                    tr(cx, title),
                    mode,
                ))
                .child(v_flex().gap_2().p(px(8.)).children((0..4).map(|i| {
                    h_flex()
                        .h(px(48.))
                        .px(px(12.))
                        .gap(px(12.))
                        .items_center()
                        .rounded(px(bookkeepy_ui::api::language::radius_tier("md")))
                        .child(
                            div()
                                .size(px(28.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    bookkeepy_ui::api::language::atoms::icon(
                                        IconName::Landmark,
                                        18.,
                                    )
                                    .text_color(prim_hsla("neutral-400").unwrap_or_default()),
                                ),
                        )
                        .child(
                            v_flex().flex_1().gap(px(2.)).child(
                                div()
                                    .refine_style(&kit::label_refinement())
                                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                                    .child(format!("Row {i}")),
                            ),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(gpui_kit::FontWeight::MEDIUM)
                                .text_color(prim_hsla("green-500").unwrap_or_default())
                                .child("+ 6.800,00"),
                        )
                })))
        };
        let ns_group = kit::group(
            cx,
            "gallery.o02.ns.title",
            "gallery.o02.ns.hint",
            v_flex().gap_6().child(kit::row([
                kit::cell(
                    cx,
                    "gallery.o02.ns.root",
                    ns_page("gallery.o02.ns.root_c", "gallery.o02.ns.accounts", false),
                ),
                kit::cell(
                    cx,
                    "gallery.o02.ns.page",
                    ns_page("gallery.o02.ns.back_c", "gallery.o02.ns.btd", true),
                ),
            ])),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::eyebrow(cx, tr(cx, "gallery.o02.ws.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.o02.ws.title")))
            .child(kit::caption(cx, tr(cx, "gallery.o02.ws.subtitle")))
            .child(dk_group)
            .child(ns_group)
    }
}
