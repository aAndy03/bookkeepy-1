// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 01 sheet (part 1) — popover, menu (references/designlanguage).
//!
//! FLAT on the page background. Live cells: a forced-open `Popover` anchored to its
//! trigger, and a live `PopupMenu` inside a forced-open popover; the row interaction
//! ladder (rest/hover/pressed/disabled) and the static panels are painted from
//! `bookkeepy_ui::api::language::popups` — same tokens.

use bookkeepy_ui::api::language::{atoms, controls::ControlSize, popups, popups::MenuIx};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::component::popover::Popover;

use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Styled as _, Window, div,
    px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let menu = PopupMenu::build(window, cx, |menu, window, cx| {
        menu.label("Transaction")
            .item(
                PopupMenuItem::new("Edit")
                    .icon(IconName::Pencil)
                    .on_click(|_, _, _| {}),
            )
            .item(
                PopupMenuItem::new("Duplicate")
                    .icon(IconName::Copy)
                    .checked(true)
                    .on_click(|_, _, _| {}),
            )
            .item(
                PopupMenuItem::new("Delete")
                    .icon(IconName::Trash)
                    .disabled(true)
                    .on_click(|_, _, _| {}),
            )
            .separator()
            .submenu("Share", window, cx, |sub, _window, _cx| {
                sub.item(PopupMenuItem::new("Copy link").on_click(|_, _, _| {}))
                    .item(PopupMenuItem::new("Export CSV").on_click(|_, _, _| {}))
            })
            .link("Open in ledger", "https://bookkeepy.app")
    });
    cx.new(|_cx| MoleculesPopups { menu }).into()
}

pub struct MoleculesPopups {
    menu: gpui_kit::Entity<PopupMenu>,
}

impl gpui_kit::Render for MoleculesPopups {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── popover ──────────────────────────────────────────────────────────
        let masters = kit::row([
            kit::cell(
                cx,
                "gallery.m01.popup.surface",
                popups::popup_surface(240., 4., mode).child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(
                            bookkeepy_ui::api::language::role_hsla(mode, "muted-foreground")
                                .unwrap_or_default(),
                        )
                        .child(tr(cx, "gallery.m01.popup.content")),
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.popover.card",
                popups::popover_card(
                    popups::popover_head(
                        tr(cx, "gallery.m01.popover.title"),
                        tr(cx, "gallery.m01.popover.desc"),
                        mode,
                    ),
                    h_flex()
                        .gap(px(8.))
                        .child(atoms::button(
                            "pop-dismiss",
                            ButtonVariant::Ghost,
                            None,
                            tr(cx, "gallery.m01.popover.dismiss"),
                            ControlSize::Small,
                        ))
                        .child(atoms::button(
                            "pop-open",
                            ButtonVariant::Secondary,
                            None,
                            tr(cx, "gallery.m01.popover.open_sub"),
                            ControlSize::Small,
                        )),
                    mode,
                ),
            ),
        ]);
        // Live popover, forced open and anchored to its trigger.
        let title = tr(cx, "gallery.m01.popover.title");
        let desc = tr(cx, "gallery.m01.popover.desc");
        let dismiss = tr(cx, "gallery.m01.popover.dismiss");
        let open_sub = tr(cx, "gallery.m01.popover.open_sub");
        let live_popover = Popover::new("pop-live")
            .open(true)
            .appearance(false) // the painted card below is the surface — no double card
            .on_open_change(|_, _, _| {})
            .trigger(atoms::button(
                "pop-trigger",
                ButtonVariant::Secondary,
                Some(bookkeepy_ui::api::language::atoms::ButtonGlyph::Icon(
                    IconName::Bell,
                )),
                tr(cx, "gallery.m01.popover.trigger"),
                ControlSize::Medium,
            ))
            .content(move |_, _, _cx| {
                popups::popover_card(
                    popups::popover_head(title.clone(), desc.clone(), mode),
                    h_flex()
                        .gap(px(8.))
                        .child(atoms::button(
                            "pop-live-dismiss",
                            ButtonVariant::Ghost,
                            None,
                            dismiss.clone(),
                            ControlSize::Small,
                        ))
                        .child(atoms::button(
                            "pop-live-open",
                            ButtonVariant::Secondary,
                            None,
                            open_sub.clone(),
                            ControlSize::Small,
                        )),
                    mode,
                )
                .into_any_element()
            });
        let popover_group = kit::group(
            cx,
            "gallery.m01.popover.title_group",
            "gallery.m01.popover.hint",
            v_flex().gap_6().child(masters),
        );

        // ── menu ─────────────────────────────────────────────────────────────
        let menu_masters = kit::row([
            kit::cell(
                cx,
                "gallery.m01.menu.item",
                popups::menu_item(
                    tr(cx, "gallery.m01.menu.edit"),
                    Some(IconName::Pencil),
                    false,
                    Some("Ctrl E"),
                    MenuIx::Rest,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m01.menu.separator",
                div().w(px(232.)).child(popups::menu_separator()),
            ),
            kit::cell(
                cx,
                "gallery.m01.menu.label",
                div()
                    .w(px(232.))
                    .child(popups::menu_label(tr(cx, "gallery.m01.menu.group"), mode)),
            ),
        ]);
        let menu_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                div().w(px(232.)).child(popups::menu_item(
                    tr(cx, "gallery.m01.menu.edit"),
                    Some(IconName::Pencil),
                    false,
                    Some("Ctrl E"),
                    MenuIx::Rest,
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                div().w(px(232.)).child(popups::menu_item(
                    tr(cx, "gallery.m01.menu.edit"),
                    Some(IconName::Pencil),
                    false,
                    Some("Ctrl E"),
                    MenuIx::Hover,
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.pressed",
                div().w(px(232.)).child(popups::menu_item(
                    tr(cx, "gallery.m01.menu.edit"),
                    Some(IconName::Pencil),
                    false,
                    Some("Ctrl E"),
                    MenuIx::Pressed,
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.m01.menu.checked",
                div().w(px(232.)).child(popups::menu_item(
                    tr(cx, "gallery.m01.menu.duplicate"),
                    Some(IconName::Copy),
                    true,
                    None,
                    MenuIx::Rest,
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                div().w(px(232.)).child(popups::menu_item(
                    tr(cx, "gallery.m01.menu.delete"),
                    Some(IconName::Trash),
                    false,
                    Some("Del"),
                    MenuIx::Disabled,
                    mode,
                )),
            ),
        ]);
        // Painted open context-menu panel (exact sheet look).
        let painted_menu = popups::popup_surface(240., 4., mode)
            .flex()
            .flex_col()
            .child(popups::menu_label(tr(cx, "gallery.m01.menu.group"), mode))
            .child(popups::menu_item(
                tr(cx, "gallery.m01.menu.edit"),
                Some(IconName::Pencil),
                false,
                Some("Ctrl E"),
                MenuIx::Rest,
                mode,
            ))
            .child(popups::menu_item(
                tr(cx, "gallery.m01.menu.duplicate"),
                Some(IconName::Copy),
                true,
                None,
                MenuIx::Rest,
                mode,
            ))
            .child(popups::menu_separator())
            .child(popups::menu_item(
                tr(cx, "gallery.m01.menu.share"),
                Some(IconName::ArrowUpRight),
                false,
                None,
                MenuIx::Hover,
                mode,
            ))
            .child(popups::menu_item(
                tr(cx, "gallery.m01.menu.delete"),
                Some(IconName::Trash),
                false,
                Some("Del"),
                MenuIx::Disabled,
                mode,
            ));
        // Live menu inside a forced-open popover.
        let menu_trigger = tr(cx, "gallery.m01.menu.live_trigger");
        let live_menu = Popover::new("menu-pop")
            .open(true)
            .appearance(false) // the PopupMenu panel is the surface — no double card
            .on_open_change(|_, _, _| {})
            .trigger(atoms::button(
                "menu-trigger",
                ButtonVariant::Secondary,
                None,
                menu_trigger,
                ControlSize::Medium,
            ))
            .content({
                let menu = self.menu.clone();
                move |_, _, _| menu.clone().into_any_element()
            });
        let menu_group = kit::group(
            cx,
            "gallery.m01.menu.title",
            "gallery.m01.menu.hint",
            v_flex()
                .gap_6()
                .child(menu_masters)
                .child(menu_states)
                .child(kit::cell(cx, "gallery.m01.menu.open", painted_menu)),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.m01.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.m01.popups_title")))
            .child(kit::caption(cx, tr(cx, "gallery.m01.subtitle")))
            .child(popover_group)
            .child(menu_group)
            // Live popovers float over the overlay layer — keep them in the last
            // row with reserved space beneath so they never cover other specimens.
            .child(kit::row([
                kit::cell(cx, "gallery.m01.popover.live", live_popover),
                kit::cell(cx, "gallery.m01.menu.live", live_menu),
            ]))
            .child(div().h(px(230.)))
    }
}
