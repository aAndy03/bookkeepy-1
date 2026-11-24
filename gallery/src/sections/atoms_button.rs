// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — Button family (Button, ButtonGroup, Toggle, DropdownButton, Link, Clipboard):
//! our wrappers' gpui-kit sources, showcased with every variant the design language uses
//! (design sheet: "ATOMS · 01 button, link, icon, label, kbd").

use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _, Toggle};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::link::Link;
use gpui_kit::component::{Disableable as _, IconName, Selectable as _, Sizable as _};
use gpui_kit::{AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Window};

use crate::app::{Gallery, tr};
use crate::sections::Layer;
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The Button-family section.
pub struct Section {
    loading_btn: bool,
    toggled: bool,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let _ = window;
    cx.new(|_| Section {
        loading_btn: false,
        toggled: false,
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let _ = Layer::Atoms;
        let empty: Vec<gpui_kit::AnyElement> = Vec::new();
        section_page(
            cx,
            "gallery.section.atoms-button",
            [
                demo_card(
                    cx,
                    "gallery.atoms.button.variants",
                    demo_row([
                        Button::new("b-primary")
                            .primary()
                            .label(tr(cx, "gallery.common.save"))
                            .into_any_element(),
                        Button::new("b-default")
                            .label(tr(cx, "gallery.common.cancel"))
                            .into_any_element(),
                        Button::new("b-outline")
                            .primary()
                            .outline()
                            .label(tr(cx, "gallery.atoms.button.outline"))
                            .into_any_element(),
                        Button::new("b-ghost")
                            .ghost()
                            .label(tr(cx, "gallery.atoms.button.ghost"))
                            .into_any_element(),
                        Button::new("b-danger")
                            .danger()
                            .label(tr(cx, "gallery.common.delete"))
                            .into_any_element(),
                        Button::new("b-warning")
                            .warning()
                            .label(tr(cx, "gallery.atoms.button.warning"))
                            .into_any_element(),
                        Button::new("b-success")
                            .success()
                            .label(tr(cx, "gallery.atoms.button.success"))
                            .into_any_element(),
                        Button::new("b-info")
                            .info()
                            .label(tr(cx, "gallery.atoms.button.info"))
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.sizes",
                    demo_row([
                        Button::new("b-xs").xsmall().label("XS").into_any_element(),
                        Button::new("b-sm").small().label("S").into_any_element(),
                        Button::new("b-md").label("M").into_any_element(),
                        Button::new("b-lg").large().label("L").into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.states",
                    demo_row([
                        Button::new("b-disabled")
                            .disabled(true)
                            .label(tr(cx, "gallery.atoms.button.disabled"))
                            .into_any_element(),
                        Button::new("b-loading")
                            .loading(self.loading_btn)
                            .label(tr(cx, "gallery.common.save"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.loading_btn = !this.loading_btn;
                                cx.notify();
                            }))
                            .into_any_element(),
                        Button::new("b-icon")
                            .icon(IconName::Search)
                            .into_any_element(),
                        Toggle::new("b-toggle")
                            .label(tr(cx, "gallery.atoms.button.toggle"))
                            .checked(self.toggled)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.toggled = *checked;
                                cx.notify();
                            }))
                            .into_any_element(),
                        Clipboard::new("b-clipboard")
                            .value(tr(cx, "gallery.app.title").to_string())
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.group",
                    ButtonGroup::new("b-group")
                        .child(Button::new("b-day").label(tr(cx, "gallery.atoms.button.day")))
                        .child(
                            Button::new("b-week")
                                .selected(true)
                                .label(tr(cx, "gallery.atoms.button.week")),
                        )
                        .child(Button::new("b-month").label(tr(cx, "gallery.atoms.button.month")))
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.link",
                    demo_row([Link::new("b-link")
                        .child(tr(cx, "gallery.atoms.button.link_external"))
                        .href("https://github.com/aAndy03/bookkeepy-1")
                        .into_any_element()])
                    .into_any_element(),
                )
                .into_any_element(),
            ]
            .into_iter()
            .chain(empty),
        )
    }
}
