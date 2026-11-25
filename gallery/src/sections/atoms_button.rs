// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — ATOMS · 01 sheet: Button family (Button, ButtonGroup, DropdownButton, Link,
//! Clipboard) plus the Icon, Label and Kbd primitives (moved here from molecules-nav so
//! the section mirrors the design sheet one-to-one).

use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::link::Link;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, IconName, Selectable as _, Sizable as _, h_flex,
};
use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement, Keystroke, Modifiers, ParentElement as _,
    Styled as _, Window, div,
};

use bookkeepy_ui::api::language::{ControlSize, button_size, controls, solid_danger};

use crate::app::{Gallery, tr};
use crate::sections::Layer;
use crate::sections::layout::{demo_card, demo_row, section_page};

/// `Ctrl+<key>` keystroke for the kbd demos.
fn ctrl_stroke(key: &str) -> Keystroke {
    Keystroke {
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
        key: key.to_string(),
        key_char: None,
    }
}

/// The Button-family section (ATOMS · 01).
pub struct Section {
    loading_btn: bool,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let _ = window;
    cx.new(|_| Section { loading_btn: false }).into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let _ = Layer::Atoms;
        let mode = cx.global::<crate::app::Prefs>().mode;
        let empty: Vec<gpui_kit::AnyElement> = Vec::new();
        section_page(
            cx,
            "gallery.section.atoms-button",
            [
                demo_card(
                    cx,
                    "gallery.atoms.button.variants",
                    // The design's button set (ATOMS · 01): primary inverts (light
                    // fill, dark ink — mapped in the theme bridge), secondary is a step
                    // lighter than its surface, outline is the bordered variant for busy
                    // surfaces, ghost is quiet text, and danger is calm at rest.
                    // warning/success/info are STATUS hues (alerts, badges), not buttons.
                    demo_row([
                        Button::new("b-primary")
                            .primary()
                            .label(tr(cx, "gallery.common.save"))
                            .into_any_element(),
                        Button::new("b-default")
                            .secondary()
                            .label(tr(cx, "gallery.common.cancel"))
                            .into_any_element(),
                        Button::new("b-outline")
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
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.danger_states",
                    // Danger's two faces (ATOMS · 01 "calm until touched"): the calm
                    // rest state — well fill, tint ink — and the solid red pressed fill,
                    // which gpui's variant API does not carry and the ui crate's language
                    // builder paints from the tokens of the active mode (RULE-UI-005).
                    demo_row([
                        Button::new("b-danger-rest")
                            .danger()
                            .label(tr(cx, "gallery.common.delete"))
                            .into_any_element(),
                        solid_danger(Button::new("b-danger-pressed"), mode)
                            .label(tr(cx, "gallery.common.delete"))
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.sizes",
                    // The design's control-size scale (ATOMS · 01: 20/24/32/40,
                    // rx 4/6/6/8) applied through the components' own size + rounded
                    // APIs — gpui's built-in sizes stop at 32.
                    demo_row([
                        button_size(Button::new("b-xs"), ControlSize::XSmall)
                            .label("XS")
                            .into_any_element(),
                        button_size(Button::new("b-sm"), ControlSize::Small)
                            .label("S")
                            .into_any_element(),
                        button_size(Button::new("b-md"), ControlSize::Medium)
                            .label("M")
                            .into_any_element(),
                        button_size(Button::new("b-lg"), ControlSize::Large)
                            .label("L")
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.button.states",
                    // Disabled/loading ride the primary per ATOMS · 01's state grid
                    // (loading = the inverted chip with a spinner); the icon button is
                    // a ghost — no plate at rest, the press fills the glyph.
                    demo_row([
                        Button::new("b-disabled")
                            .primary()
                            .disabled(true)
                            .label(tr(cx, "gallery.atoms.button.disabled"))
                            .into_any_element(),
                        Button::new("b-loading")
                            .primary()
                            .loading(self.loading_btn)
                            .label(tr(cx, "gallery.common.save"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.loading_btn = !this.loading_btn;
                                cx.notify();
                            }))
                            .into_any_element(),
                        Button::new("b-icon")
                            .ghost()
                            .icon(IconName::Search)
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
                demo_card(
                    cx,
                    "gallery.molecules.nav.icon",
                    // Icon ramp + color roles (ATOMS · 01): the hues ride the
                    // functional seats set in the theme bridge (red/yellow/green).
                    h_flex()
                        .items_center()
                        .gap_4()
                        .child(Icon::new(IconName::Search).small())
                        .child(Icon::new(IconName::Heart).text_color(cx.theme().red))
                        .child(
                            Icon::new(IconName::Star)
                                .large()
                                .text_color(cx.theme().yellow),
                        )
                        .child(
                            Icon::new(IconName::CircleCheck)
                                .large()
                                .text_color(cx.theme().green),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.molecules.nav.label",
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            Label::new(tr(cx, "gallery.molecules.nav.label.text"))
                                .secondary(tr(cx, "gallery.molecules.nav.label.secondary")),
                        )
                        .child(
                            Label::new(tr(cx, "gallery.molecules.nav.label.masked"))
                                .masked(true)
                                .text_xl(),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.molecules.nav.kbd",
                    // Kbd keys as raised #1c1c1c rx4 chips (ATOMS · 01 kbd atom).
                    h_flex()
                        .items_center()
                        .gap_3()
                        .child(tr(cx, "gallery.molecules.nav.kbd.save"))
                        .child(controls::kbd(Kbd::new(ctrl_stroke("s")), mode))
                        .child(tr(cx, "gallery.molecules.nav.kbd.search"))
                        .child(controls::kbd(Kbd::new(ctrl_stroke("f")), mode))
                        .into_any_element(),
                )
                .into_any_element(),
            ]
            .into_iter()
            .chain(empty),
        )
    }
}
