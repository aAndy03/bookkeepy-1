// SPDX-License-Identifier: GPL-3.0-or-later
//! Organisms — shell chrome: TitleBar, Sidebar, Toolbar, StatusBar, Resizable,
//! Scrollable, NativeMenu, Settings (design sheet: "ORGANISMS · the window
//! shell"). Every demo is a mini standalone surface, never a nested full
//! window. `Root` — the app bootstrap container — cannot nest inside a section;
//! it is exercised by `gallery/src/main.rs` itself (the gallery window's own
//! root).

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::h_flex;
use gpui_kit::component::label::Label;
use gpui_kit::component::native_menu::NativeMenu;
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::setting::{
    SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_kit::component::sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenuItem};
use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::toolbar::{Toolbar, ToolbarGroup};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Side, Sizable as _, TitleBar};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, Window, div, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

gpui_kit::actions!(
    gallery,
    [ShellMenuReview, ShellMenuExport, ShellMenuAutosave]
);

/// i18n key shown once the native menu item was picked.
const PICK_REVIEWED: &str = "gallery.organisms.shell.native_menu.picked_reviewed";
/// i18n key shown once the native menu item was picked.
const PICK_EXPORT: &str = "gallery.organisms.shell.native_menu.picked_export";
/// i18n key shown once the native menu item was picked.
const PICK_AUTOSAVE: &str = "gallery.organisms.shell.native_menu.picked_autosave";

/// The shell-organism section.
pub struct Section {
    /// Whether the sidebar's first item is the active destination.
    invoices_active: bool,
    /// i18n key of the last picked native-menu item (feedback line below the button).
    menu_pick: Option<&'static str>,
    /// Whether the Settings demo's autosave switch is on.
    settings_autosave: bool,
    /// The Settings demo's reporting-currency dropdown value.
    settings_currency: SharedString,
}

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_| Section {
        invoices_active: true,
        menu_pick: None,
        settings_autosave: true,
        settings_currency: "RON".into(),
    })
    .into()
}

impl Section {
    fn pick_menu_item(&mut self, key: &'static str, cx: &mut gpui_kit::Context<'_, Self>) {
        self.menu_pick = Some(key);
        cx.notify();
    }

    fn on_review(
        &mut self,
        _: &ShellMenuReview,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        self.pick_menu_item(PICK_REVIEWED, cx);
    }

    fn on_export(
        &mut self,
        _: &ShellMenuExport,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        self.pick_menu_item(PICK_EXPORT, cx);
    }

    fn on_autosave(
        &mut self,
        _: &ShellMenuAutosave,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        self.pick_menu_item(PICK_AUTOSAVE, cx);
    }
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let picked = tr(
            cx,
            self.menu_pick
                .unwrap_or("gallery.organisms.shell.native_menu.hint"),
        );
        // Retained demo state lives on this Section entity, so the Settings
        // field closures read/write through this handle (they are 'static and
        // cannot borrow `self`). Each closure gets its own clone.
        let autosave_get_entity = cx.entity();
        let autosave_set_entity = cx.entity();
        let currency_get_entity = cx.entity();
        let currency_set_entity = cx.entity();

        section_page(
            cx,
            "gallery.section.organisms-shell",
            [
                // Window chrome: TitleBar, Sidebar, Toolbar, StatusBar.
                demo_row([
                    demo_card(
                        cx,
                        "gallery.organisms.shell.title_bar.caption",
                        TitleBar::new()
                            .max_w(px(360.))
                            .child(Label::new(tr(cx, "gallery.organisms.shell.title_bar.name")))
                            .child(
                                Button::new("shell-title-bar-bell")
                                    .icon(IconName::Bell)
                                    .ghost()
                                    .small()
                                    .tooltip(tr(cx, "gallery.organisms.shell.title_bar.bell")),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.organisms.shell.sidebar.caption",
                        div()
                            .id("shell-sidebar-frame")
                            .h(px(220.))
                            .max_w(px(240.))
                            .child(
                                Sidebar::new("shell-sidebar")
                                    .side(Side::Left)
                                    .header(
                                        SidebarHeader::new()
                                            .child(tr(cx, "gallery.organisms.shell.sidebar.header")),
                                    )
                                    .child(
                                        SidebarGroup::new(tr(cx, "gallery.organisms.shell.sidebar.group"))
                                            .child(
                                                SidebarMenuItem::new(tr(
                                                    cx,
                                                    "gallery.organisms.shell.sidebar.invoices",
                                                ))
                                                .icon(IconName::LayoutDashboard)
                                                .active(self.invoices_active)
                                                .on_click(cx.listener(
                                                    |this: &mut Self,
                                                     _: &gpui_kit::ClickEvent,
                                                     _window: &mut Window,
                                                     cx: &mut gpui_kit::Context<'_, Self>| {
                                                        this.invoices_active = !this.invoices_active;
                                                        cx.notify();
                                                    },
                                                )),
                                            )
                                            .child(
                                                SidebarMenuItem::new(tr(
                                                    cx,
                                                    "gallery.organisms.shell.sidebar.reports",
                                                ))
                                                .icon(IconName::FileText),
                                            ),
                                    ),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.organisms.shell.toolbar.caption",
                        Toolbar::new("shell-toolbar")
                            .max_w(px(360.))
                            .child(
                                ToolbarGroup::new("shell-toolbar-group")
                                    .label(tr(cx, "gallery.organisms.shell.toolbar.group"))
                                    .gap_2()
                                    .child(
                                        Button::new("shell-toolbar-undo")
                                            .icon(IconName::Undo2)
                                            .label(tr(cx, "gallery.organisms.shell.toolbar.undo")),
                                    )
                                    .child(
                                        Button::new("shell-toolbar-redo")
                                            .icon(IconName::Redo2)
                                            .label(tr(cx, "gallery.organisms.shell.toolbar.redo")),
                                    ),
                            )
                            .content(Separator::vertical().h_5())
                            .content(div().flex_1())
                            .child(
                                Button::new("shell-toolbar-more")
                                    .icon(IconName::Ellipsis)
                                    .tooltip(tr(cx, "gallery.organisms.shell.toolbar.more")),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.organisms.shell.status_bar.caption",
                        StatusBar::new()
                            .max_w(px(420.))
                            .left(tr(cx, "gallery.organisms.shell.status_bar.state"))
                            .left(Separator::vertical())
                            .child(tr(cx, "gallery.organisms.shell.status_bar.file"))
                            .right(tr(cx, "gallery.organisms.shell.status_bar.encoding"))
                            .right(
                                h_flex()
                                    .items_center()
                                    .gap_1()
                                    .child(Icon::new(IconName::CircleCheck).xsmall())
                                    .child(tr(cx, "gallery.organisms.shell.status_bar.saved")),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                // Content surfaces: Resizable, Scrollable, NativeMenu.
                demo_row([
                    demo_card(
                        cx,
                        "gallery.organisms.shell.resizable.caption",
                        div()
                            .id("shell-resizable-frame")
                            .max_w(px(480.))
                            .child(
                                h_resizable("shell-resizable")
                                    .size(px(140.))
                                    .child(
                                        resizable_panel()
                                            .size(px(140.))
                                            .size_range(px(110.)..px(260.))
                                            .flex_none()
                                            .child(
                                                div()
                                                    .h_full()
                                                    .p_3()
                                                    .bg(theme.secondary)
                                                    .child(tr(
                                                        cx,
                                                        "gallery.organisms.shell.resizable.left",
                                                    )),
                                            ),
                                    )
                                    .child(
                                        resizable_panel().child(
                                            div().h_full().p_3().child(tr(
                                                cx,
                                                "gallery.organisms.shell.resizable.main",
                                            )),
                                        ),
                                    ),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.organisms.shell.scrollable.caption",
                        div()
                            .id("shell-scrollable")
                            .h(px(180.))
                            .max_w(px(300.))
                            .flex()
                            .flex_col()
                            .gap_2()
                            .overflow_y_scrollbar()
                            .children((1..=12).map(|i| {
                                div()
                                    .h_8()
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .px_3()
                                    .rounded_md()
                                    .bg(theme.secondary)
                                    .child(format!(
                                        "{} {}",
                                        tr(cx, "gallery.organisms.shell.scrollable.row"),
                                        i
                                    ))
                            }))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.organisms.shell.native_menu.caption",
                        div()
                            .id("shell-native-menu")
                            .flex()
                            .flex_col()
                            .gap_3()
                            .max_w(px(320.))
                            .on_action(cx.listener(Self::on_review))
                            .on_action(cx.listener(Self::on_export))
                            .on_action(cx.listener(Self::on_autosave))
                            .child(
                                Button::new("shell-native-menu-open")
                                    .label(tr(cx, "gallery.organisms.shell.native_menu.open"))
                                    .on_click(|ev, window, cx| {
                                        NativeMenu::new()
                                            .menu(
                                                tr(cx, "gallery.organisms.shell.native_menu.review"),
                                                Box::new(ShellMenuReview),
                                            )
                                            .menu(
                                                tr(cx, "gallery.organisms.shell.native_menu.export"),
                                                Box::new(ShellMenuExport),
                                            )
                                            .separator()
                                            .menu_with_check(
                                                tr(cx, "gallery.organisms.shell.native_menu.autosave"),
                                                true,
                                                Box::new(ShellMenuAutosave),
                                            )
                                            .show(ev.position(), window, cx);
                                    }),
                            )
                            .child(Label::new(picked))
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                // Preferences surface: Settings.
                demo_row([demo_card(
                    cx,
                    "gallery.organisms.shell.settings.caption",
                    div()
                        .id("shell-settings-frame")
                        .h(px(280.))
                        .max_w(px(560.))
                        .child(
                            Settings::new("shell-settings")
                                .sidebar_width(px(150.))
                                .page(
                                    SettingPage::new(tr(cx, "gallery.organisms.shell.settings.page"))
                                        .group(
                                            SettingGroup::new()
                                                .title(tr(cx, "gallery.organisms.shell.settings.group"))
                                                .item(
                                                    SettingItem::new(
                                                        tr(cx, "gallery.organisms.shell.settings.autosave"),
                                                        SettingField::switch(
                                                            move |cx: &App| {
                                                                autosave_get_entity
                                                                    .read(cx)
                                                                    .settings_autosave
                                                            },
                                                            move |val: bool, cx: &mut App| {
                                                                autosave_set_entity.update(
                                                                    cx,
                                                                    |this, cx| {
                                                                        this.settings_autosave =
                                                                            val;
                                                                        cx.notify();
                                                                    },
                                                                );
                                                            },
                                                        ),
                                                    )
                                                    .description(tr(
                                                        cx,
                                                        "gallery.organisms.shell.settings.autosave_hint",
                                                    )),
                                                )
                                                .item(
                                                    SettingItem::new(
                                                        tr(cx, "gallery.organisms.shell.settings.currency"),
                                                        SettingField::dropdown(
                                                            vec![
                                                                (
                                                                    "RON".into(),
                                                                    tr(cx, "gallery.organisms.shell.settings.currency_ron"),
                                                                ),
                                                                (
                                                                    "EUR".into(),
                                                                    tr(cx, "gallery.organisms.shell.settings.currency_eur"),
                                                                ),
                                                            ],
                                                            move |cx: &App| {
                                                                currency_get_entity
                                                                    .read(cx)
                                                                    .settings_currency
                                                                    .clone()
                                                            },
                                                            move |val: SharedString, cx: &mut App| {
                                                                currency_set_entity
                                                                    .update(cx, |this, cx| {
                                                                        this.settings_currency =
                                                                            val;
                                                                        cx.notify();
                                                                    });
                                                            },
                                                        ),
                                                    )
                                                    .description(tr(
                                                        cx,
                                                        "gallery.organisms.shell.settings.currency_hint",
                                                    )),
                                                ),
                                        ),
                                ),
                        )
                        .into_any_element(),
                )
                .into_any_element()])
                .into_any_element(),
            ],
        )
    }
}
