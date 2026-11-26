// SPDX-License-Identifier: GPL-3.0-or-later
//! ORGANISMS · 01 sheet (part 1) — title bar + sidebar
//! (references/designlanguage ORGANISMS · 01, blocks O1–O2).
//!
//! FLAT on the page background. The gallery's own top chrome IS the live
//! `TitleBar`; here the bar's focused/unfocused/close-hover states are painted
//! from `language::organisms`. The sidebar below is a LIVE `Sidebar`
//! (expanded + collapsed via its footer toggle); the resize affordance is painted.

use bookkeepy_ui::api::language::organisms;
use bookkeepy_ui::api::language::{role_hsla, shell};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Side;
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenuItem, SidebarToggleButton,
};
use gpui_kit::{
    AnyView, AppContext as _, Context, ParentElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| OrganismsShell { collapsed: false }).into()
}

pub struct OrganismsShell {
    collapsed: bool,
}

/// The live sidebar specimen: design content (switchers, nav groups, badge,
/// footer sync line) at the design width via `language::shell::sidebar`.
fn live_sidebar(
    view: &OrganismsShell,
    cx: &mut Context<'_, OrganismsShell>,
) -> Sidebar<SidebarGroup<SidebarMenuItem>> {
    let mode = cx.global::<Prefs>().mode;
    fn item(
        cx: &mut Context<'_, OrganismsShell>,
        mode: bookkeepy_primary_data::theme::Mode,
        key: &'static str,
        icon: IconName,
        active: bool,
        badge: Option<&'static str>,
    ) -> SidebarMenuItem {
        let mut it = SidebarMenuItem::new(tr(cx, key))
            .icon(icon)
            .active(active)
            .on_click(cx.listener(|_v, _ev, _window, cx| cx.notify()));
        if let Some(b) = badge {
            let badge = tr(cx, b);
            it = it.suffix(move |_window, _cx| {
                div()
                    .h(px(16.))
                    .px(px(5.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .bg(role_hsla(mode, "accent").unwrap_or_default())
                    .text_size(px(10.))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .text_color(role_hsla(mode, "primary-foreground").unwrap_or_default())
                    .child(badge.clone())
            });
        }
        it
    }
    let header = SidebarHeader::new().child(
        v_flex()
            .gap(px(24.))
            .child(organisms::sb_switcher(
                tr(cx, "gallery.o01.sb.profile"),
                "AM",
                "cyan-900",
                "cyan-300",
                tr(cx, "gallery.o01.sb.andy"),
                mode,
            ))
            .child(organisms::sb_switcher(
                tr(cx, "gallery.o01.sb.workspace"),
                "M",
                "cyan-900",
                "cyan-300",
                tr(cx, "gallery.o01.sb.money"),
                mode,
            )),
    );
    let footer = SidebarFooter::new()
        .child(organisms::sb_sync_line(
            tr(cx, "gallery.o01.sb.offline"),
            view.collapsed,
        ))
        .child(
            SidebarToggleButton::new()
                .collapsed(view.collapsed)
                .on_click(cx.listener(|v, _ev, _window, cx| {
                    v.collapsed = !v.collapsed;
                    cx.notify();
                })),
        );
    shell::sidebar(Sidebar::new("o1-sidebar").side(Side::Left))
        .collapsed(view.collapsed)
        .header(header)
        .footer(footer)
        .child(
            SidebarGroup::new(tr(cx, "gallery.o01.sb.g_money")).children([
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.overview",
                    IconName::House,
                    true,
                    None,
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.transactions",
                    IconName::ArrowLeftRight,
                    false,
                    Some("gallery.o01.sb.badge12"),
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.accounts",
                    IconName::Landmark,
                    false,
                    None,
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.categories",
                    IconName::Tag,
                    false,
                    None,
                ),
            ]),
        )
        .child(
            SidebarGroup::new(tr(cx, "gallery.o01.sb.g_plan")).children([
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.budgets",
                    IconName::ChartPie,
                    false,
                    None,
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.subscriptions",
                    IconName::Repeat,
                    false,
                    None,
                ),
            ]),
        )
        .child(
            SidebarGroup::new(tr(cx, "gallery.o01.sb.g_review")).children([
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.reports",
                    IconName::FileText,
                    false,
                    None,
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.history",
                    IconName::Clock,
                    false,
                    None,
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.help",
                    IconName::CircleQuestionMark,
                    false,
                    None,
                ),
                item(
                    cx,
                    mode,
                    "gallery.o01.sb.settings",
                    IconName::Settings,
                    false,
                    None,
                ),
            ]),
        )
}

impl gpui_kit::Render for OrganismsShell {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── O1 · title bar ───────────────────────────────────────────────────
        // The gallery window's own top bar is the live TitleBar specimen; here
        // the sheet's states are painted: master, focused/unfocused in the
        // in-main-view chrome, and the close-hover danger glyph.
        let tb_group = kit::group(
            cx,
            "gallery.o01.tb.title",
            "gallery.o01.tb.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(
                    cx,
                    "gallery.o01.tb.master",
                    organisms::title_bar(
                        tr(cx, "gallery.o01.tb.kicker"),
                        tr(cx, "gallery.o01.tb.page"),
                        true,
                        mode,
                    ),
                ))
                .child(kit::cell(
                    cx,
                    "gallery.o01.tb.window_states",
                    organisms::window_chrome(mode)
                        .w(px(972.))
                        .child(organisms::title_bar(
                            tr(cx, "gallery.o01.tb.kicker"),
                            tr(cx, "gallery.o01.tb.page"),
                            true,
                            mode,
                        ))
                        .child(organisms::title_bar(
                            tr(cx, "gallery.o01.tb.kicker"),
                            tr(cx, "gallery.o01.tb.page"),
                            false,
                            mode,
                        )),
                ))
                .child(kit::cell(
                    cx,
                    "gallery.o01.tb.close_hover",
                    organisms::title_bar_right(false, true, mode),
                )),
        );

        // ── O2 · sidebar ─────────────────────────────────────────────────────
        let live_sb = live_sidebar(self, cx);
        let sb_live_cell = kit::cell(
            cx,
            "gallery.o01.sb.live",
            h_flex().gap_6().items_start().child(live_sb).child(
                div()
                    .border_l(px(2.))
                    .border_color(role_hsla(mode, "border-strong").unwrap_or_default())
                    .h(px(700.)),
            ),
        );
        let sb_group = kit::group(
            cx,
            "gallery.o01.sb.title",
            "gallery.o01.sb.hint",
            v_flex().gap_6().child(sb_live_cell).child(kit::cell(
                cx,
                "gallery.o01.sb.resize",
                organisms::sidebar_resize(296, mode),
            )),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.o01.sh.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.o01.sh.title")))
            .child(kit::caption(cx, tr(cx, "gallery.o01.sh.subtitle")))
            .child(tb_group)
            .child(sb_group)
    }
}
