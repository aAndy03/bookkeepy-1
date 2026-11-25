// SPDX-License-Identifier: GPL-3.0-or-later
//! Molecules — the popup family: Popover (click-triggered floating panel),
//! Menu (dropdown + right-click context menus via the popup-menu builder API),
//! HoverCard (hover-triggered rich preview with open/close delays).

use std::time::Duration;

use gpui_kit::component::button::Button;
use gpui_kit::component::hover_card::HoverCard;
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::{ActiveTheme as _, Icon, IconName};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::{
    Anchor, AnyView, App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, Window, div, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The popups-molecule section.
pub struct Section {
    /// The menu action picked last (empty = none yet) — dropdown and context
    /// menu items record their label here, so the demo has visible state.
    menu_action: SharedString,
}

/// A clickable menu item with no bound `Action`: records its label on the section.
fn action_item(
    section: &Entity<Section>,
    key: &str,
    cx: &App,
    icon: Option<Icon>,
    checked: bool,
) -> PopupMenuItem {
    let label: SharedString = tr(cx, key);
    let section = section.clone();
    let action = label.clone();
    let mut item = PopupMenuItem::new(label).checked(checked);
    if let Some(icon) = icon {
        item = item.icon(icon);
    }
    item.on_click(move |_, _, cx| {
        section.update(cx, |this, cx| {
            this.menu_action = action.clone();
            cx.notify();
        });
    })
}

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_| Section {
        menu_action: SharedString::default(),
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let section = cx.entity();
        let theme = cx.theme();

        // Popover: basic click-triggered + anchored variant with arrow.
        let popover_demo =
            h_flex()
                .gap_3()
                .flex_wrap()
                .child(
                    Popover::new("popover-demo")
                        .trigger(
                            Button::new("popover-demo-trigger")
                                .label(tr(cx, "gallery.molecules.popups.popover.trigger"))
                                .outline(),
                        )
                        // MOL·01: the panel is #222222 rx12 borderless — owned here via
                        // appearance(false) + popup_panel (gpui's popover_style rounds at
                        // the shared control radius).
                        .appearance(false)
                        .child(bookkeepy_ui::api::language::surface::popup_panel(
                            v_flex()
                                .gap_1()
                                .p_3()
                                .child(tr(cx, "gallery.molecules.popups.popover.content_title"))
                                .child(div().text_sm().text_color(theme.muted_foreground).child(
                                    tr(cx, "gallery.molecules.popups.popover.content_body"),
                                )),
                            cx.global::<crate::app::Prefs>().mode,
                        )),
                )
                .child(
                    Popover::new("popover-arrow-demo")
                        .anchor(Anchor::TopCenter)
                        .offset(px(8.))
                        .arrow(true)
                        .trigger(
                            Button::new("popover-arrow-trigger")
                                .label(tr(cx, "gallery.molecules.popups.popover.trigger_arrow"))
                                .outline(),
                        )
                        .child(
                            div()
                                .text_sm()
                                .child(tr(cx, "gallery.molecules.popups.popover.content_arrow")),
                        ),
                )
                .into_any_element();

        // Menu: dropdown button + right-click context menu + the recorded action.
        let dropdown_section = section.clone();
        let dropdown = Button::new("menu-demo-trigger")
            .label(tr(cx, "gallery.molecules.popups.menu.open"))
            .outline()
            .dropdown_menu(move |menu, window, cx| {
                let submenu_section = dropdown_section.clone();
                menu.label(tr(cx, "gallery.molecules.popups.menu.group"))
                    .item(action_item(
                        &dropdown_section,
                        "gallery.molecules.popups.menu.item_new",
                        cx,
                        Some(Icon::new(IconName::Plus)),
                        false,
                    ))
                    .item(action_item(
                        &dropdown_section,
                        "gallery.molecules.popups.menu.item_duplicate",
                        cx,
                        Some(Icon::new(IconName::Copy)),
                        false,
                    ))
                    .separator()
                    .submenu(
                        tr(cx, "gallery.molecules.popups.menu.submenu_actions"),
                        window,
                        cx,
                        move |submenu, _window, cx| {
                            submenu
                                .item(action_item(
                                    &submenu_section,
                                    "gallery.molecules.popups.menu.item_archive",
                                    cx,
                                    Some(Icon::new(IconName::Inbox)),
                                    false,
                                ))
                                .item(action_item(
                                    &submenu_section,
                                    "gallery.molecules.popups.menu.item_delete",
                                    cx,
                                    Some(Icon::new(IconName::Delete)),
                                    false,
                                ))
                        },
                    )
                    .separator()
                    .item(action_item(
                        &dropdown_section,
                        "gallery.molecules.popups.menu.item_active_only",
                        cx,
                        None,
                        true,
                    ))
                    .item(
                        PopupMenuItem::new(tr(cx, "gallery.molecules.popups.menu.item_export"))
                            .icon(Icon::new(IconName::ExternalLink))
                            .disabled(true),
                    )
            });

        let context_section = section;
        let context = div()
            .id("menu-context-demo")
            .p_2()
            .rounded_lg()
            .text_sm()
            .child(tr(cx, "gallery.molecules.popups.menu.context_hint"))
            .context_menu(move |menu, _window, cx| {
                menu.item(action_item(
                    &context_section,
                    "gallery.molecules.popups.menu.item_new",
                    cx,
                    Some(Icon::new(IconName::Plus)),
                    false,
                ))
                .item(action_item(
                    &context_section,
                    "gallery.molecules.popups.menu.item_delete",
                    cx,
                    Some(Icon::new(IconName::Delete)),
                    false,
                ))
            });

        let menu_demo = v_flex()
            .gap_2()
            .child(h_flex().gap_3().flex_wrap().child(dropdown).child(context))
            .child(
                h_flex()
                    .gap_1()
                    .text_sm()
                    .child(tr(cx, "gallery.molecules.popups.menu.last_action"))
                    .child(div().text_color(theme.foreground).child(
                        if self.menu_action.is_empty() {
                            tr(cx, "gallery.molecules.popups.menu.none_yet")
                        } else {
                            self.menu_action.clone()
                        },
                    )),
            )
            .into_any_element();

        // HoverCard: default timing on a text trigger + fast timing above a button.
        let hover_card_demo = h_flex()
            .gap_4()
            .flex_wrap()
            .child(
                HoverCard::new("hover-card-demo")
                    .trigger(
                        div()
                            .text_sm()
                            .cursor_pointer()
                            .text_color(theme.link)
                            .child(tr(cx, "gallery.molecules.popups.hover_card.trigger")),
                    )
                    // MOL·02 hover card: 300x183 rx12 #222222 panel — owned via
                    // appearance(false) + popup_panel.
                    .appearance(false)
                    .child(bookkeepy_ui::api::language::surface::popup_panel(
                        v_flex()
                            .gap_1()
                            .p_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .child(tr(cx, "gallery.molecules.popups.hover_card.title")),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(tr(cx, "gallery.molecules.popups.hover_card.body")),
                            ),
                        cx.global::<crate::app::Prefs>().mode,
                    )),
            )
            .child(
                HoverCard::new("hover-card-fast-demo")
                    .open_delay(Duration::from_millis(200))
                    .close_delay(Duration::from_millis(100))
                    .anchor(Anchor::BottomCenter)
                    .trigger(
                        Button::new("hover-card-fast-trigger")
                            .label(tr(cx, "gallery.molecules.popups.hover_card.trigger_fast"))
                            .outline(),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(tr(cx, "gallery.molecules.popups.hover_card.body_fast")),
                    ),
            )
            .into_any_element();

        section_page(
            cx,
            "gallery.section.molecules-popups",
            [demo_row([
                demo_card(cx, "gallery.molecules.popups.popover.caption", popover_demo)
                    .into_any_element(),
                demo_card(cx, "gallery.molecules.popups.menu.caption", menu_demo)
                    .into_any_element(),
                demo_card(
                    cx,
                    "gallery.molecules.popups.hover_card.caption",
                    hover_card_demo,
                )
                .into_any_element(),
            ])
            .into_any_element()],
        )
    }
}
