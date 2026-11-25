// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — choice controls: Checkbox, Radio (+RadioGroup), Switch, Toggle, Slider
//! (design sheet: "ATOMS · 02"; the Switch is the settings-takes-effect control, the
//! design language's pill toggle; Rating lives in the ATOMS · 04 section `atoms-media`).

use gpui_kit::component::button::Toggle;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::radio::RadioGroup;
use gpui_kit::component::slider::{Slider, SliderState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Sizable as _};
use gpui_kit::{AnyView, AppContext as _, Context, IntoElement, Styled as _, Window};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The choice-controls section.
pub struct Section {
    checkbox: bool,
    delivery: usize,
    sync: bool,
    toggle: bool,
    slider: gpui_kit::Entity<SliderState>,
    slider_accent: gpui_kit::Entity<SliderState>,
}

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .default_value(50.)
                .step(1.)
        });
        let slider_accent = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .default_value(72.)
                .step(1.)
        });
        Section {
            checkbox: false,
            delivery: 0,
            sync: true,
            toggle: true,
            slider,
            slider_accent,
        }
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let mode = cx.global::<crate::app::Prefs>().mode;
        section_page(
            cx,
            "gallery.section.atoms-choice",
            [
                demo_card(
                    cx,
                    "gallery.atoms.choice.checkbox",
                    demo_row([
                        bookkeepy_ui::api::language::controls::checkbox(
                            Checkbox::new("agree"),
                            mode,
                        )
                        .label(tr(cx, "gallery.atoms.choice.checkbox_label"))
                        .checked(self.checkbox)
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.checkbox = *checked;
                            cx.notify();
                        }))
                        .into_any_element(),
                        bookkeepy_ui::api::language::controls::checkbox(
                            Checkbox::new("agree-on"),
                            mode,
                        )
                        .label(tr(cx, "gallery.atoms.choice.checkbox_on"))
                        .checked(true)
                        .into_any_element(),
                        bookkeepy_ui::api::language::controls::checkbox(
                            Checkbox::new("agree-off"),
                            mode,
                        )
                        .label(tr(cx, "gallery.atoms.choice.checkbox_disabled"))
                        .disabled(true)
                        .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.choice.radio",
                    RadioGroup::new("delivery")
                        .children([
                            tr(cx, "gallery.atoms.choice.radio_immediate"),
                            tr(cx, "gallery.atoms.choice.radio_daily"),
                        ])
                        .selected_index(Some(self.delivery))
                        .on_change(cx.listener(|this, value: &usize, _, cx| {
                            this.delivery = *value;
                            cx.notify();
                        }))
                        .into_any_element(),
                )
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.atoms.choice.switch",
                        demo_row([
                            Switch::new("sync")
                                .label(tr(cx, "gallery.atoms.choice.switch_label"))
                                .checked(self.sync)
                                .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                    this.sync = *checked;
                                    cx.notify();
                                }))
                                .into_any_element(),
                            // ATOMS · 02 small switch: 28x16 rx8.
                            Switch::new("sync-sm")
                                .small()
                                .checked(self.sync)
                                .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                    this.sync = *checked;
                                    cx.notify();
                                }))
                                .into_any_element(),
                        ])
                        .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.atoms.choice.toggle",
                        bookkeepy_ui::api::language::controls::toggle(
                            Toggle::new("b-toggle"),
                            mode,
                        )
                        .label(tr(cx, "gallery.atoms.choice.toggle_label"))
                        .checked(self.toggle)
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.toggle = *checked;
                            cx.notify();
                        }))
                        .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.choice.slider",
                    // ATOMS · 02 slider: neutral fill + accent fill variant (the fill
                    // rides the instance bg as the bar color).
                    demo_row([
                        Slider::new(&self.slider).into_any_element(),
                        Slider::new(&self.slider_accent)
                            .bg(cx.theme().ring)
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
