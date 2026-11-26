// SPDX-License-Identifier: GPL-3.0-or-later
//! ORGANISMS · 01 sheet (part 2) — dialog, alert dialog, notification
//! (references/designlanguage ORGANISMS · 01, blocks O3–O5).
//!
//! FLAT on the page background. Panels and toasts are painted from
//! `language::organisms` (sheet-exact geometry); the "open live …" buttons open
//! the REAL gpui-kit dialog / push a REAL notification through the window's
//! Root layers, so the interactive behavior is exercised too.

use bookkeepy_ui::api::language::organisms::{self, NoteTone};
use bookkeepy_ui::api::language::{prim_hsla, role_hsla};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariant, ButtonVariants as _};
use gpui_kit::component::dialog::{DialogFooter, DialogHeader, DialogTitle};
use gpui_kit::component::notification::{Notification, NotificationType};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, ParentElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| OrganismsOverlay).into()
}

pub struct OrganismsOverlay;

/// The painted O3 dialog panel: header + amount/account/payee body + footer.
fn painted_dialog(
    saving: bool,
    cx: &App,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let fg = role_hsla(mode, "foreground").unwrap_or_default();
    let muted = prim_hsla("neutral-400").unwrap_or_default();
    let input = role_hsla(mode, "input").unwrap_or_default();
    let input_focus = role_hsla(mode, "input-focus").unwrap_or_default();
    let white_a16 = prim_hsla("white-a16").unwrap_or_default();
    let transparent = gpui_kit::transparent_white();

    let amount_field = h_flex()
        .px(px(10.))
        .gap(px(8.))
        .items_center()
        .child(
            div()
                .text_size(px(13.))
                .line_height(px(16.))
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_color(fg)
                .child("−142,50"),
        )
        .child(
            div()
                .text_size(px(11.))
                .line_height(px(15.))
                .text_color(muted)
                .child("RON"),
        )
        .child(div().flex_1())
        .child(
            h_flex()
                .h(px(20.))
                .pl(px(4.))
                .ml(px(4.))
                .border_l(px(1.))
                .border_color(prim_hsla("neutral-800").unwrap_or_default())
                .items_center()
                .gap(px(3.))
                .child(organisms::icon_btn(IconName::Minus, 12., fg, mode))
                .child(organisms::icon_btn(IconName::Plus, 12., fg, mode)),
        );
    let account_field = h_flex()
        .px(px(10.))
        .gap(px(8.))
        .items_center()
        .child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .text_color(fg)
                .child(tr(cx, "gallery.o01.dlg.revolut")),
        )
        .child(div().flex_1())
        .child(
            bookkeepy_ui::api::language::atoms::icon(IconName::ChevronsUpDown, 12.)
                .text_color(muted),
        );
    let payee_field = h_flex()
        .px(px(10.))
        .items_center()
        .child(
            div()
                .text_size(px(12.))
                .line_height(px(14.))
                .text_color(fg)
                .child(tr(cx, "gallery.o01.dlg.mega")),
        )
        .child(div().w(px(1.)).h(px(14.)).bg(fg).ml(px(1.)));

    let body = v_flex()
        .gap(px(16.))
        .child(
            h_flex()
                .gap(px(16.))
                .items_start()
                .child(div().w(px(180.)).child(organisms::dlg_field(
                    tr(cx, "gallery.o01.dlg.amount"),
                    transparent,
                    input,
                    amount_field,
                    mode,
                ))),
        )
        .child(div().flex_1().min_w_0().child(organisms::dlg_field(
            tr(cx, "gallery.o01.dlg.account"),
            transparent,
            input,
            account_field,
            mode,
        )))
        .child(
            v_flex()
                .gap(px(6.))
                .child(organisms::dlg_field(
                    tr(cx, "gallery.o01.dlg.payee"),
                    white_a16,
                    input_focus,
                    payee_field,
                    mode,
                ))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(muted)
                        .child(tr(cx, "gallery.o01.dlg.help")),
                ),
        );

    let (confirm_label, confirm_icon) = if saving {
        (
            tr(cx, "gallery.o01.dlg.saving"),
            Some(IconName::LoaderCircle),
        )
    } else {
        (tr(cx, "gallery.o01.dlg.save"), None)
    };
    let footer = h_flex()
        .justify_end()
        .gap(px(8.))
        .child(organisms::dlg_btn(
            tr(cx, "gallery.o01.dlg.cancel"),
            gpui_kit::transparent_white(),
            muted,
            None,
            muted,
        ))
        .child(organisms::dlg_btn(
            confirm_label,
            prim_hsla("neutral-100").unwrap_or_default(),
            prim_hsla("neutral-925").unwrap_or_default(),
            confirm_icon,
            muted,
        ));

    organisms::dlg_frame(440., mode)
        .child(organisms::dlg_header(
            tr(cx, "gallery.o01.dlg.new_tx"),
            Some(tr(cx, "gallery.o01.dlg.new_tx_desc")),
            mode,
        ))
        .child(body)
        .child(footer)
}

/// The painted O4 alert dialog: glyph + title + description + footer.
fn painted_alert(
    destructive: bool,
    cx: &App,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    let (glyph, glyph_c, title_key, desc_key, ok_key) = if destructive {
        (
            IconName::CircleAlert,
            prim_hsla("amber-500").unwrap_or_default(),
            "gallery.o01.ad.del_cat",
            "gallery.o01.ad.del_cat_desc",
            "gallery.o01.ad.delete",
        )
    } else {
        (
            IconName::Info,
            role_hsla(mode, "info").unwrap_or_default(),
            "gallery.o01.ad.discard",
            "gallery.o01.ad.discard_desc",
            "gallery.o01.ad.discard_ok",
        )
    };
    let muted = prim_hsla("neutral-400").unwrap_or_default();
    let footer = h_flex().justify_end().gap(px(8.)).child(
        // focus starts on Cancel for the destructive variant (2px ring ring)
        div()
            .border_2()
            .border_color(if destructive {
                prim_hsla("neutral-200").unwrap_or_default()
            } else {
                gpui_kit::transparent_white()
            })
            .rounded(px(bookkeepy_ui::api::language::radius_tier("md")))
            .child(organisms::dlg_btn(
                tr(cx, "gallery.o01.dlg.cancel"),
                gpui_kit::transparent_white(),
                muted,
                None,
                muted,
            )),
    );
    let footer = if destructive {
        footer.child(organisms::dlg_btn(
            tr(cx, ok_key),
            role_hsla(mode, "danger-subtle").unwrap_or_default(),
            role_hsla(mode, "danger-foreground").unwrap_or_default(),
            Some(IconName::Trash),
            role_hsla(mode, "danger-foreground").unwrap_or_default(),
        ))
    } else {
        footer.child(organisms::dlg_btn(
            tr(cx, ok_key),
            prim_hsla("neutral-100").unwrap_or_default(),
            prim_hsla("neutral-925").unwrap_or_default(),
            None,
            prim_hsla("neutral-925").unwrap_or_default(),
        ))
    };
    organisms::dlg_frame(380., mode)
        .child(
            h_flex()
                .gap(px(16.))
                .items_start()
                .child(bookkeepy_ui::api::language::atoms::icon(glyph, 24.).text_color(glyph_c))
                .child(
                    v_flex().flex_1().gap(px(8.)).child(
                        div()
                            .text_size(px(20.))
                            .line_height(px(24.))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                            .child(tr(cx, title_key)),
                    ),
                ),
        )
        .child(
            div()
                .text_size(px(12.))
                .line_height(px(18.))
                .text_color(muted)
                .child(tr(cx, desc_key)),
        )
        .child(footer)
}

/// The O3 scrim demo: board cards under a black-60 scrim with the dialog on top.
fn scrim_demo(cx: &App, mode: bookkeepy_primary_data::theme::Mode) -> gpui_kit::Div {
    let card = |x: f32, y: f32, w: f32, h: f32| {
        div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(w))
            .h(px(h))
            .rounded(px(bookkeepy_ui::api::language::radius_tier("2xl")))
            .bg(role_hsla(mode, "surface-raised").unwrap_or_default())
    };
    div()
        .relative()
        .w(px(1100.))
        .h(px(520.))
        .rounded(px(bookkeepy_ui::api::language::radius_tier("viewport")))
        .overflow_hidden()
        .bg(role_hsla(mode, "surface").unwrap_or_default())
        .child(card(40., 40., 150., 220.))
        .child(card(220., 90., 150., 220.))
        .child(card(400., 30., 150., 220.))
        .child(card(580., 120., 150., 220.))
        .child(card(760., 50., 150., 220.))
        .child(card(930., 100., 150., 220.))
        .child(
            div()
                .absolute()
                .inset_0()
                .bg(prim_hsla("black-a60").unwrap_or_default()),
        )
        .child(
            div()
                .absolute()
                .left(px(330.))
                .top(px(40.))
                .child(painted_dialog(false, cx, mode)),
        )
}

impl gpui_kit::Render for OrganismsOverlay {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── O3 · dialog ──────────────────────────────────────────────────────
        let (dlg_title, dlg_desc, dlg_cancel, dlg_save, open_live) = (
            tr(cx, "gallery.o01.dlg.new_tx"),
            tr(cx, "gallery.o01.dlg.new_tx_desc"),
            tr(cx, "gallery.o01.dlg.cancel"),
            tr(cx, "gallery.o01.dlg.save"),
            tr(cx, "gallery.o01.dlg.open_live"),
        );
        let open_dialog_btn =
            Button::new("o1-open-dialog")
                .label(open_live)
                .on_click(move |_ev, window, cx| {
                    let (t, d, c, s) = (
                        dlg_title.clone(),
                        dlg_desc.clone(),
                        dlg_cancel.clone(),
                        dlg_save.clone(),
                    );
                    window.open_dialog(cx, move |dialog, _window, _cx| {
                        dialog
                            .title(t.clone())
                            .child(DialogHeader::new().child(DialogTitle::new().child(t.clone())))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(prim_hsla("neutral-400").unwrap_or_default())
                                    .child(d.clone()),
                            )
                            .footer(
                                DialogFooter::new()
                                    .child(Button::new("o1-dlg-cancel").label(c.clone()).ghost())
                                    .child(Button::new("o1-dlg-ok").label(s.clone()).primary()),
                            )
                    });
                });
        let dlg_group = kit::group(
            cx,
            "gallery.o01.dlg.title",
            "gallery.o01.dlg.hint",
            v_flex()
                .gap_6()
                .child(kit::row([
                    kit::cell(
                        cx,
                        "gallery.o01.dlg.master",
                        painted_dialog(false, cx, mode),
                    ),
                    kit::cell(cx, "gallery.o01.dlg.saving", painted_dialog(true, cx, mode)),
                ]))
                .child(kit::row([kit::cell(
                    cx,
                    "gallery.o01.dlg.live",
                    open_dialog_btn,
                )]))
                .child(kit::cell(cx, "gallery.o01.dlg.scrim", scrim_demo(cx, mode))),
        );

        // ── O4 · alert dialog ────────────────────────────────────────────────
        let (ad_title, ad_desc, ad_ok, ad_cancel, ad_open) = (
            tr(cx, "gallery.o01.ad.del_cat"),
            tr(cx, "gallery.o01.ad.del_cat_desc"),
            tr(cx, "gallery.o01.ad.delete"),
            tr(cx, "gallery.o01.dlg.cancel"),
            tr(cx, "gallery.o01.ad.open_live"),
        );
        let open_alert_btn =
            Button::new("o1-open-alert")
                .label(ad_open)
                .on_click(move |_ev, window, cx| {
                    let (t, d, ok, c) = (
                        ad_title.clone(),
                        ad_desc.clone(),
                        ad_ok.clone(),
                        ad_cancel.clone(),
                    );
                    window.open_alert_dialog(cx, move |alert, _window, _cx| {
                        alert
                            .confirm()
                            .title(t.clone())
                            .description(d.clone())
                            .ok_text(ok.clone())
                            .ok_variant(ButtonVariant::Danger)
                            .cancel_text(c.clone())
                    });
                });
        let ad_group = kit::group(
            cx,
            "gallery.o01.ad.title",
            "gallery.o01.ad.hint",
            v_flex()
                .gap_6()
                .child(kit::row([
                    kit::cell(
                        cx,
                        "gallery.o01.ad.destructive",
                        painted_alert(true, cx, mode),
                    ),
                    kit::cell(cx, "gallery.o01.ad.confirm", painted_alert(false, cx, mode)),
                ]))
                .child(kit::row([kit::cell(
                    cx,
                    "gallery.o01.ad.live",
                    open_alert_btn,
                )])),
        );

        // ── O5 · notification ────────────────────────────────────────────────
        let (nt_title, nt_msg, nt_push) = (
            tr(cx, "gallery.o01.nt.imported"),
            tr(cx, "gallery.o01.nt.imported_msg"),
            tr(cx, "gallery.o01.nt.push_live"),
        );
        let push_btn =
            Button::new("o1-push-note")
                .label(nt_push)
                .on_click(move |_ev, window, cx| {
                    let (t, m) = (nt_title.clone(), nt_msg.clone());
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t.clone())
                            .message(m.clone())
                            .autohide(true),
                        cx,
                    );
                });
        let nt_group = kit::group(
            cx,
            "gallery.o01.nt.title",
            "gallery.o01.nt.hint",
            v_flex()
                .gap_6()
                .child(kit::row([
                    kit::cell(
                        cx,
                        "gallery.o01.nt.info",
                        organisms::toast(
                            NoteTone::Info,
                            tr(cx, "gallery.o01.nt.imported"),
                            tr(cx, "gallery.o01.nt.imported_msg"),
                            Some(tr(cx, "gallery.o01.nt.review")),
                            220.,
                            mode,
                        ),
                    ),
                    kit::cell(
                        cx,
                        "gallery.o01.nt.success",
                        organisms::toast(
                            NoteTone::Success,
                            tr(cx, "gallery.o01.nt.rates"),
                            tr(cx, "gallery.o01.nt.rates_msg"),
                            None,
                            140.,
                            mode,
                        ),
                    ),
                ]))
                .child(kit::row([
                    kit::cell(
                        cx,
                        "gallery.o01.nt.warning",
                        organisms::toast(
                            NoteTone::Warning,
                            tr(cx, "gallery.o01.nt.budget"),
                            tr(cx, "gallery.o01.nt.budget_msg"),
                            None,
                            220.,
                            mode,
                        ),
                    ),
                    kit::cell(
                        cx,
                        "gallery.o01.nt.danger",
                        organisms::toast(
                            NoteTone::Danger,
                            tr(cx, "gallery.o01.nt.sync_fail"),
                            tr(cx, "gallery.o01.nt.sync_fail_msg"),
                            None,
                            360.,
                            mode,
                        ),
                    ),
                ]))
                .child(kit::row([
                    kit::cell(
                        cx,
                        "gallery.o01.nt.stack",
                        organisms::toast(
                            NoteTone::Success,
                            tr(cx, "gallery.o01.nt.deleted"),
                            tr(cx, "gallery.o01.nt.deleted_msg"),
                            Some(tr(cx, "gallery.o01.nt.undo")),
                            300.,
                            mode,
                        ),
                    ),
                    kit::cell(cx, "gallery.o01.nt.live", push_btn),
                ])),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.o01.ov.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.o01.ov.title")))
            .child(kit::caption(cx, tr(cx, "gallery.o01.ov.subtitle")))
            .child(dlg_group)
            .child(ad_group)
            .child(nt_group)
    }
}
