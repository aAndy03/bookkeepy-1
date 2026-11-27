// SPDX-License-Identifier: GPL-3.0-or-later
//! ORGANISMS · 01 sheet (part 1) — title bar + sidebar
//! (references/designlanguage ORGANISMS · 01, blocks O1–O2).
//!
//! FLAT on the page background. The gallery's own top chrome IS the live
//! `TitleBar`; here the bar's focused/unfocused/close-hover states are painted
//! from `language::organisms`. The sidebar below is a LIVE `Sidebar`
//! (expanded + collapsed via its footer toggle); the resize affordance is painted.

use bookkeepy_ui::api::language::organisms::{self, RzIx, SbIx};
use bookkeepy_ui::api::language::{prim_hsla, primitives, role_hsla, shell};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Side;
use gpui_kit::component::button::{Button, ButtonVariants as _, Toggle, ToggleGroup};
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenuItem, SidebarToggleButton,
};
use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::toolbar::Toolbar;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{
    AnyView, AppContext as _, Context, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Styled as _, Window, div, px,
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
            .h(px(24.))
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
        let tb1_group = kit::group(
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
        let sb_side_group = kit::group(
            cx,
            "gallery.o01.sb.title",
            "gallery.o01.sb.hint",
            v_flex().gap_6().child(sb_live_cell).child(kit::cell(
                cx,
                "gallery.o01.sb.resize",
                organisms::sidebar_resize(296, mode),
            )),
        );

        // ── O9 · toolbar ─────────────────────────────────────────────────────
        let view_toggle = ToggleGroup::new("o2-view")
            .segmented()
            .on_click(|_states: &Vec<bool>, _window, _cx| {})
            .child(Toggle::new("o2-v-list").icon(IconName::List).checked(true))
            .child(Toggle::new("o2-v-chart").icon(IconName::ChartPie))
            .child(Toggle::new("o2-v-cal").icon(IconName::Calendar));
        let tb_live = Toolbar::new("o2-toolbar")
            .with_size(Size::Large)
            .w(px(760.))
            .content(
                h_flex()
                    .px(px(8.))
                    .gap(px(8.))
                    .items_center()
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                            .child(tr(cx, "gallery.o02.tb.title")),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(prim_hsla("neutral-400").unwrap_or_default())
                            .child(tr(cx, "gallery.o02.tb.count")),
                    ),
            )
            .content(div().flex_1())
            .child(view_toggle)
            .content(Separator::vertical().h(px(16.)))
            .child(
                Button::new("o2-filter")
                    .with_size(Size::Size(px(32.)))
                    .icon(IconName::Funnel)
                    .tooltip(tr(cx, "gallery.o02.tb.filter")),
            )
            .child(
                Button::new("o2-search")
                    .with_size(Size::Size(px(32.)))
                    .icon(IconName::Search)
                    .tooltip(tr(cx, "gallery.o02.tb.search")),
            )
            .child(primitives::medium(
                Button::new("o2-import")
                    .icon(IconName::Upload)
                    .label(tr(cx, "gallery.o02.tb.import"))
                    .dropdown_caret(true),
            ))
            // Primary actions go through `.content` — `.child` would force
            // ghost+compact via `prepare_for_toolbar` (gpui-kit toolbar.rs).
            .content(primitives::medium(
                Button::new("o2-add")
                    .primary()
                    .icon(IconName::Plus)
                    .label(tr(cx, "gallery.o02.tb.add")),
            ));
        let tb9_group = kit::group(
            cx,
            "gallery.o02.tb.title_g",
            "gallery.o02.tb.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.tb.live", tb_live))
                .child(kit::row([
                    kit::cell(
                        cx,
                        "gallery.o02.tb.selection",
                        organisms::toolbar_selection(
                            tr(cx, "gallery.o02.tb.sel3"),
                            tr(cx, "gallery.o02.tb.seltotal"),
                            mode,
                        ),
                    ),
                    kit::cell(
                        cx,
                        "gallery.atoms.state.disabled",
                        div().opacity(0.4).child(organisms::toolbar_selection(
                            tr(cx, "gallery.o02.tb.sel3"),
                            tr(cx, "gallery.o02.tb.seltotal"),
                            mode,
                        )),
                    ),
                ])),
        );

        // ── O10 · status bar ─────────────────────────────────────────────────
        let sb_live = StatusBar::new()
            .w(px(960.))
            .left(organisms::sync_line(
                IconName::CloudOff,
                prim_hsla("neutral-400").unwrap_or_default(),
                tr(cx, "gallery.o02.sb.offline"),
                prim_hsla("neutral-500").unwrap_or_default(),
                mode,
            ))
            .left(
                div()
                    .text_size(px(11.))
                    .text_color(prim_hsla("neutral-400").unwrap_or_default())
                    .child(tr(cx, "gallery.o02.sb.counts")),
            )
            .right(
                div()
                    .text_size(px(11.))
                    .text_color(prim_hsla("neutral-300").unwrap_or_default())
                    .child(tr(cx, "gallery.o02.sb.sum")),
            )
            .right(
                h_flex()
                    .gap(px(4.))
                    .items_center()
                    .child(organisms::kbd_chip("Ctrl+K", mode))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(prim_hsla("neutral-400").unwrap_or_default())
                            .child(tr(cx, "gallery.o02.sb.command")),
                    ),
            );
        let sb_states = kit::row([
            kit::cell(
                cx,
                "gallery.o02.sb.saved",
                organisms::sync_line(
                    IconName::CircleCheck,
                    prim_hsla("neutral-400").unwrap_or_default(),
                    tr(cx, "gallery.o02.sb.saved_t"),
                    prim_hsla("neutral-500").unwrap_or_default(),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.o02.sb.syncing",
                organisms::sync_line(
                    IconName::RefreshCw,
                    prim_hsla("neutral-200").unwrap_or_default(),
                    tr(cx, "gallery.o02.sb.syncing_t"),
                    prim_hsla("neutral-300").unwrap_or_default(),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.o02.sb.offline_s",
                organisms::sync_line(
                    IconName::CloudOff,
                    prim_hsla("neutral-400").unwrap_or_default(),
                    tr(cx, "gallery.o02.sb.offline"),
                    prim_hsla("neutral-500").unwrap_or_default(),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.o02.sb.paused",
                organisms::sync_line(
                    IconName::CircleAlert,
                    prim_hsla("amber-500").unwrap_or_default(),
                    tr(cx, "gallery.o02.sb.paused_t"),
                    prim_hsla("amber-300").unwrap_or_default(),
                    mode,
                ),
            ),
        ]);
        let sb10_group = kit::group(
            cx,
            "gallery.o02.sb.title",
            "gallery.o02.sb.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.sb.live", sb_live))
                .child(sb_states),
        );

        // ── O17 · resizable ──────────────────────────────────────────────────
        let rz_live = div().w(px(900.)).h(px(240.)).child(
            h_resizable("o2-split")
                .child(
                    resizable_panel()
                        .size(px(240.))
                        .size_range(px(200.)..px(360.))
                        .child(
                            v_flex()
                                .w(px(240.))
                                .h_full()
                                .bg(role_hsla(mode, "background").unwrap_or_default())
                                .p(px(8.))
                                .gap(px(4.))
                                .children((0..8).map(|i| {
                                    div()
                                        .h(px(20.))
                                        .w_full()
                                        .rounded(px(4.))
                                        .bg(role_hsla(mode, "surface").unwrap_or_default())
                                        .child(
                                            div()
                                                .pl(px(8.))
                                                .text_size(px(11.))
                                                .text_color(
                                                    prim_hsla("neutral-400").unwrap_or_default(),
                                                )
                                                .child(format!("Item {i}")),
                                        )
                                })),
                        ),
                )
                .child(
                    resizable_panel().child(
                        div()
                            .size_full()
                            .bg(role_hsla(mode, "surface").unwrap_or_default()),
                    ),
                ),
        );
        let rz_handles = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                organisms::resize_handle(RzIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                organisms::resize_handle(RzIx::Hover),
            ),
            kit::cell(
                cx,
                "gallery.o02.rz.dragging",
                organisms::resize_handle(RzIx::Drag),
            ),
        ]);
        let rz_group = kit::group(
            cx,
            "gallery.o02.rz.title",
            "gallery.o02.rz.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.rz.live", rz_live))
                .child(rz_handles),
        );

        // ── O18 · scrollable ─────────────────────────────────────────────────
        let scroll_rows = v_flex().children((0..20).map(|i| {
            organisms::list_item(
                div()
                    .size(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        bookkeepy_ui::api::language::atoms::icon(IconName::Landmark, 18.)
                            .text_color(prim_hsla("neutral-400").unwrap_or_default()),
                    ),
                format!("Account {i}").into(),
                "current · …4021".into(),
                organisms::amount_trailing(
                    "1.240,00".into(),
                    "RON",
                    role_hsla(mode, "foreground").unwrap_or_default(),
                )
                .into_any_element(),
                organisms::LiIx::Rest,
                mode,
            )
        }));
        let sc_live = div()
            .w(px(360.))
            .h(px(260.))
            .bg(role_hsla(mode, "surface").unwrap_or_default())
            .rounded(px(bookkeepy_ui::api::language::radius_tier("2xl")))
            .overflow_hidden()
            .child(
                div()
                    .id("o2-scroll-inner")
                    .size_full()
                    .p(px(8.))
                    .overflow_y_scrollbar()
                    .child(scroll_rows),
            );
        let sc_group = kit::group(
            cx,
            "gallery.o02.sc.title",
            "gallery.o02.sc.hint",
            v_flex().gap_6().child(kit::row([
                kit::cell(cx, "gallery.o02.sc.live", sc_live),
                kit::cell(
                    cx,
                    "gallery.o02.sc.scrolling",
                    organisms::scrollbar(SbIx::Scrolling),
                ),
                kit::cell(
                    cx,
                    "gallery.atoms.state.hover",
                    organisms::scrollbar(SbIx::Hover),
                ),
            ])),
        );

        // ── O16 · settings ──────────────────────────────────────────────────
        use gpui_kit::component::setting::{
            SettingField, SettingGroup, SettingItem, SettingPage, Settings,
        };
        let st_live = div().w(px(1000.)).h(px(420.)).overflow_hidden().child(
            Settings::new("o2-settings")
                .pages([SettingPage::new(tr(cx, "gallery.o02.st.general"))
                    .default_open(true)
                    .group(
                        SettingGroup::new()
                            .title(tr(cx, "gallery.o02.st.g_sync"))
                            .item(SettingItem::new(
                                tr(cx, "gallery.o02.st.sync"),
                                SettingField::switch(|_cx| true, |_v, _cx| {}),
                            ))
                            .item(SettingItem::new(
                                tr(cx, "gallery.o02.st.offline"),
                                SettingField::switch(|_cx| false, |_v, _cx| {}),
                            )),
                    )])
                .pages([SettingPage::new(tr(cx, "gallery.o02.st.accounts")).group(
                    SettingGroup::new()
                        .title(tr(cx, "gallery.o02.st.g_acc"))
                        .item(SettingItem::new(
                            tr(cx, "gallery.o02.st.default_acc"),
                            SettingField::input(
                                |_cx| gpui_kit::SharedString::from("BT Pay"),
                                |_v, _cx| {},
                            ),
                        )),
                )]),
        );
        let st_group = kit::group(
            cx,
            "gallery.o02.st.title",
            "gallery.o02.st.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.o02.st.live", st_live)),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.o01.sh.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.o01.sh.title")))
            .child(kit::caption(cx, tr(cx, "gallery.o01.sh.subtitle")))
            .child(tb1_group)
            .child(sb_side_group)
            .child(tb9_group)
            .child(sb10_group)
            .child(rz_group)
            .child(sc_group)
            .child(st_group)
    }
}
