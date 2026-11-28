// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 02 sheet (part 2) — checkbox, radio, switch, toggle, slider
//! (references/designlanguage).
//!
//! FLAT on the page background like the design frames. The rest/on/mixed cells are
//! LIVE stock gpui-kit components (click them); hover/pressed/focus/disabled columns
//! are painted specimens from `bookkeepy_ui::api::language::choice` — same tokens,
//! same geometry as the sheet (gpui 0.11 has no state flag for those pseudo-states).

use bookkeepy_ui::api::language::{atoms, choice, choice::Ix, choice::Tri};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::StyledExt as _;
use gpui_kit::component::button::{Toggle, ToggleGroup, ToggleVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::radio::{Radio, RadioGroup};
use gpui_kit::component::slider::{Slider, SliderState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, SharedString,
    Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| AtomsChoice {
        cb_off: cx.new(|_| false),
        cb_on: cx.new(|_| true),
        sw: cx.new(|_| true),
        slider: cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(1000.)
                .step(10.)
                .default_value(620.)
        }),
    })
    .into()
}

pub struct AtomsChoice {
    cb_off: Entity<bool>,
    cb_on: Entity<bool>,
    sw: Entity<bool>,
    slider: Entity<SliderState>,
}

const IX_KEYS: [&str; 5] = [
    "gallery.atoms.state.rest",
    "gallery.atoms.state.hover",
    "gallery.atoms.state.pressed",
    "gallery.atoms.state.focus",
    "gallery.atoms.state.disabled",
];

const IX: [Ix; 5] = [Ix::Rest, Ix::Hover, Ix::Pressed, Ix::Focus, Ix::Disabled];

impl gpui_kit::Render for AtomsChoice {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let label = tr(cx, "gallery.atoms02.choice.select_all");

        // ── checkbox: off / on / mixed × interaction ────────────────────────
        fn cb_row(
            cx: &App,
            mode: bookkeepy_primary_data::theme::Mode,
            tri: Tri,
            live: Option<Entity<bool>>,
            label: &SharedString,
        ) -> gpui_kit::Div {
            kit::row(IX.iter().enumerate().map(|(i, ix)| {
                let el: gpui_kit::AnyElement = match (i, live.clone()) {
                    // rest cell = LIVE checkbox (click to toggle)
                    (0, Some(state)) => {
                        let id: &'static str = match tri {
                            Tri::Off => "cb-off-live",
                            Tri::On => "cb-on-live",
                            Tri::Mixed => "cb-mixed-live",
                        };
                        let mut cb = Checkbox::new(id).label(label.clone());
                        match tri {
                            Tri::On => cb = cb.checked(true),
                            Tri::Mixed => cb = cb.indeterminate(true),
                            Tri::Off => {}
                        }
                        cb.on_change(move |v, _, cx| state.update(cx, |b, _| *b = *v))
                            .into_any_element()
                    }
                    _ => h_flex()
                        .items_center()
                        .gap(px(8.))
                        .child(choice::checkbox_box(tri, *ix, mode))
                        .child(label_div(label.clone()))
                        .into_any_element(),
                };
                kit::cell(cx, IX_KEYS[i], el)
            }))
        }
        let cb_matrix = v_flex()
            .gap_4()
            .child(cb_row(
                cx,
                mode,
                Tri::Off,
                Some(self.cb_off.clone()),
                &label,
            ))
            .child(cb_row(cx, mode, Tri::On, Some(self.cb_on.clone()), &label))
            .child(cb_row(cx, mode, Tri::Mixed, None, &label));
        let checkbox_group = kit::group(
            cx,
            "gallery.atoms02.choice.checkbox",
            "gallery.atoms02.choice.hint",
            cb_matrix,
        );

        // ── radio: off / on × interaction + groups ──────────────────────────
        let radio_row = |on: bool| {
            kit::row(IX.iter().enumerate().map(|(i, ix)| {
                let el: gpui_kit::AnyElement = if i == 0 && on {
                    Radio::new("radio-on")
                        .label(label.clone())
                        .checked(true)
                        .into_any_element()
                } else if i == 0 {
                    Radio::new("radio-off")
                        .label(label.clone())
                        .into_any_element()
                } else {
                    h_flex()
                        .items_center()
                        .gap(px(8.))
                        .child(choice::radio_circle(on, *ix, mode))
                        .child(label_div(label.clone()))
                        .into_any_element()
                };
                kit::cell(cx, IX_KEYS[i], el)
            }))
        };
        let radio_groups = h_flex().gap_12().items_start().child(
            v_flex()
                .gap_3()
                .child(
                    RadioGroup::vertical("rg-v")
                        .selected_index(Some(0))
                        .child(
                            Radio::new("v0")
                                .label(tr(cx, "gallery.atoms02.demo.keep_local"))
                                .checked(true),
                        )
                        .child(
                            Radio::new("v1").label(tr(cx, "gallery.atoms02.demo.encrypted_sync")),
                        ),
                )
                .child(
                    RadioGroup::horizontal("rg-h")
                        .selected_index(Some(2))
                        .child(Radio::new("h0").label(tr(cx, "gallery.atoms02.demo.daily")))
                        .child(Radio::new("h1").label(tr(cx, "gallery.atoms02.demo.weekly")))
                        .child(Radio::new("h2").label(tr(cx, "gallery.atoms02.demo.monthly"))),
                ),
        );
        let radio_matrix = v_flex()
            .gap_4()
            .child(radio_row(false))
            .child(radio_row(true))
            .child(radio_groups);
        let radio_group_el = kit::group(
            cx,
            "gallery.atoms02.choice.radio",
            "gallery.atoms02.choice.hint",
            radio_matrix,
        );

        // ── switch: off / on × interaction + sizes ──────────────────────────
        fn switch_row(
            cx: &App,
            mode: bookkeepy_primary_data::theme::Mode,
            on: bool,
            small: bool,
            live: Option<Entity<bool>>,
            label: &SharedString,
        ) -> gpui_kit::Div {
            kit::row(IX.iter().enumerate().map(|(i, ix)| {
                let el: gpui_kit::AnyElement = if i == 0 {
                    let live = live.clone();
                    let id: &'static str = match (on, small) {
                        (false, false) => "sw-off-md",
                        (true, false) => "sw-on-md",
                        (true, true) => "sw-on-sm",
                        (false, true) => "sw-off-sm",
                    };
                    let mut sw = Switch::new(id).label(label.clone()).checked(on);
                    if small {
                        sw = sw.with_size(Size::Small);
                    }
                    if let Some(c) = bookkeepy_ui::api::language::role_hsla(mode, "primary") {
                        sw = sw.color(c);
                    }
                    if let Some(st) = live {
                        sw = sw.on_change(move |v, _, cx| st.update(cx, |b, _| *b = *v));
                    }
                    sw.into_any_element()
                } else {
                    h_flex()
                        .items_center()
                        .gap(px(8.))
                        .child(choice::switch_track(on, small, *ix, mode))
                        .child(label_div(label.clone()))
                        .into_any_element()
                };
                kit::cell(cx, IX_KEYS[i], el)
            }))
        }
        let switch_matrix = v_flex()
            .gap_4()
            .child(switch_row(cx, mode, false, false, None, &label))
            .child(switch_row(
                cx,
                mode,
                true,
                false,
                Some(self.sw.clone()),
                &label,
            ))
            .child(switch_row(cx, mode, true, true, None, &label));
        let settings_row = |title_key: &str, desc_key: &str, on: bool, id: &'static str| {
            h_flex()
                .w(px(420.))
                .items_center()
                .justify_between()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .refine_style(&kit::label_refinement())
                                .child(tr(cx, title_key)),
                        )
                        .child(kit::caption(cx, tr(cx, desc_key))),
                )
                .child({
                    let mut sw = Switch::new(id).checked(on);
                    if let Some(c) = bookkeepy_ui::api::language::role_hsla(mode, "primary") {
                        sw = sw.color(c);
                    }
                    sw
                })
        };
        let settings = v_flex()
            .gap_4()
            .child(settings_row(
                "gallery.atoms02.demo.hide_amounts",
                "gallery.atoms02.demo.hide_amounts_desc",
                true,
                "set-hide",
            ))
            .child(settings_row(
                "gallery.atoms02.demo.start_today",
                "gallery.atoms02.demo.start_today_desc",
                true,
                "set-today",
            ))
            .child(settings_row(
                "gallery.atoms02.demo.encrypted_sync",
                "gallery.atoms02.demo.encrypted_sync_desc",
                false,
                "set-sync",
            ));
        let switch_group = kit::group(
            cx,
            "gallery.atoms02.choice.switch",
            "gallery.atoms02.choice.hint",
            v_flex().gap_6().child(switch_matrix).child(kit::cell(
                cx,
                "gallery.atoms02.choice.settings",
                settings,
            )),
        );

        // ── toggle: off/on × ghost/outline × interaction + sizes + segmented ─
        let toggle_row = |on: bool, outline: bool, h: f32, px_: f32| {
            kit::row(IX.iter().enumerate().map(|(i, ix)| {
                let el: gpui_kit::AnyElement = if i == 0 {
                    let id: &'static str = match (on, outline) {
                        (false, false) => "tg-off-ghost",
                        (true, false) => "tg-on-ghost",
                        (false, true) => "tg-off-outline",
                        (true, true) => "tg-on-outline",
                    };
                    let mut t = Toggle::new(id)
                        .icon(atoms::duotone_icon(IconName::Eye, 14.))
                        .label(tr(cx, "gallery.atoms02.demo.hide_amounts"))
                        .checked(on);
                    if outline {
                        t = t.outline();
                    }
                    if let Some(bg) =
                        bookkeepy_ui::api::language::role_hsla(mode, "control-selected")
                    {
                        t = t.bg(bg);
                    }
                    t.into_any_element()
                } else {
                    choice::toggle_chip(
                        on,
                        outline,
                        *ix,
                        IconName::Eye,
                        tr(cx, "gallery.atoms02.demo.hide_amounts"),
                        px_,
                        h,
                        mode,
                    )
                    .into_any_element()
                };
                kit::cell(cx, IX_KEYS[i], el)
            }))
        };
        let toggle_sizes = kit::row(
            [
                (20., 6., "gallery.atoms02.tag.size20"),
                (24., 8., "gallery.atoms02.tag.size24"),
                (32., 10., "gallery.atoms02.choice.md"),
                (40., 14., "gallery.atoms02.choice.lg"),
            ]
            .into_iter()
            .map(|(h, px_, key)| {
                kit::cell(
                    cx,
                    key,
                    choice::toggle_chip(
                        true,
                        false,
                        Ix::Rest,
                        IconName::Eye,
                        tr(cx, "gallery.atoms02.demo.hide_amounts"),
                        px_,
                        h,
                        mode,
                    ),
                )
            }),
        );
        let segmented = ToggleGroup::new("seg")
            .segmented()
            .single()
            .with_size(Size::Small)
            .child(
                Toggle::new("seg-in")
                    .icon(atoms::duotone_icon(IconName::ArrowDown, 14.))
                    .label(tr(cx, "gallery.atoms02.demo.in"))
                    .checked(true),
            )
            .child(
                Toggle::new("seg-out")
                    .icon(atoms::duotone_icon(IconName::ArrowUp, 14.))
                    .label(tr(cx, "gallery.atoms02.demo.out")),
            )
            .child(
                Toggle::new("seg-tr")
                    .icon(atoms::icon(IconName::ArrowLeftRight, 14.))
                    .label(tr(cx, "gallery.atoms02.demo.transfers")),
            );
        let toggle_matrix = v_flex()
            .gap_4()
            .child(toggle_row(false, false, 32., 10.))
            .child(toggle_row(true, false, 32., 10.))
            .child(toggle_row(false, true, 32., 10.))
            .child(toggle_row(true, true, 32., 10.))
            .child(toggle_sizes)
            .child(kit::cell(cx, "gallery.atoms02.choice.segmented", segmented));
        let toggle_group = kit::group(
            cx,
            "gallery.atoms02.choice.toggle",
            "gallery.atoms02.choice.hint",
            toggle_matrix,
        );

        // ── slider: live + painted states + range + marks ────────────────────
        let live_slider = h_flex().w(px(240.)).child(
            Slider::new(&self.slider)
                .bg(
                    bookkeepy_ui::api::language::role_hsla(mode, "progress-fill")
                        .unwrap_or_default(),
                )
                .text_color(
                    bookkeepy_ui::api::language::role_hsla(mode, "progress-fill")
                        .unwrap_or_default(),
                ),
        );
        let slider_states = kit::row(IX.iter().enumerate().map(|(i, ix)| {
            let pct = [25., 37.5, 50., 62.5, 75.][i];
            let bubble = (*ix == Ix::Pressed).then(|| tr(cx, "gallery.atoms02.demo.ron620"));
            let cell = v_flex()
                .gap_2()
                .child(choice::slider_track_bubble(pct, *ix, 240., bubble, mode));
            kit::cell(cx, IX_KEYS[i], cell)
        }));
        let range = choice::slider_range(25., 70., 240., mode);
        let marks = v_flex()
            .gap_2()
            .child(choice::slider_track(66., Ix::Rest, 240., mode))
            .child(
                h_flex().w(px(240.)).justify_between().children(
                    ["0", "250", "500", "750", "1.000"]
                        .into_iter()
                        .map(|m| kit::caption(cx, SharedString::from(m))),
                ),
            );
        let slider_matrix = v_flex()
            .gap_4()
            .child(kit::cell(cx, "gallery.atoms02.slider.live", live_slider))
            .child(slider_states)
            .child(kit::cell(cx, "gallery.atoms02.slider.range", range))
            .child(kit::cell(cx, "gallery.atoms02.slider.marks", marks))
            .child(kit::cell(
                cx,
                "gallery.atoms02.slider.budget",
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .w(px(272.))
                            .justify_between()
                            .child(kit::caption(
                                cx,
                                tr(cx, "gallery.atoms02.demo.budget_title"),
                            ))
                            .child(kit::caption(cx, tr(cx, "gallery.atoms02.demo.ron420"))),
                    )
                    .child(choice::slider_track(47.5, Ix::Rest, 240., mode))
                    .child(kit::caption(cx, tr(cx, "gallery.atoms02.demo.budget_desc"))),
            ));
        let slider_group = kit::group(
            cx,
            "gallery.atoms02.choice.slider",
            "gallery.atoms02.choice.hint",
            slider_matrix,
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::eyebrow(cx, tr(cx, "gallery.atoms02.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.atoms02.choice.title")))
            .child(kit::caption(cx, tr(cx, "gallery.atoms02.choice.subtitle")))
            .child(checkbox_group)
            .child(radio_group_el)
            .child(switch_group)
            .child(toggle_group)
            .child(slider_group)
    }
}

/// The 12/400 label next to a choice control (sheet: white ink at rest).
fn label_div(text: SharedString) -> gpui_kit::Div {
    div()
        .refine_style(&kit::label_refinement())
        .line_height(px(14.))
        .whitespace_nowrap()
        .child(text)
}
