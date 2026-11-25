// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 01 sheet — button, link, icon, label, kbd (references/designlanguage).
//!
//! Rendered FLAT on the page background exactly like the exported design frames: no
//! cards behind elements (a card only appears when it is part of a component's own
//! design). Groups are separated by spacing only. Every control is LIVE — hover,
//! press or tab the cells to see the designed interaction states (rest/hover/pressed/
//! focus are runtime states; selected/open/loading/disabled are forced via the
//! component flags).

use bookkeepy_ui::api::language::controls::{ControlSize, button_size};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Selectable as _, Sizable as _,
};
use gpui_kit::{
    AnyView, App, AppContext as _, ClickEvent, Context, IntoElement, Keystroke, ParentElement as _,
    SharedString, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| AtomsButton::default()).into()
}

/// The ATOMS · 01 section view. Holds a click counter so the demo is visibly live.
#[derive(Default)]
pub struct AtomsButton {
    clicks: usize,
}

impl AtomsButton {
    fn on_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<'_, Self>) {
        self.clicks += 1;
        cx.notify();
    }

    /// Small overline caption (TOKENS §03 `overline`: 10px/500/+0.3px, muted).
    fn overline(cx: &App, text: impl Into<SharedString>) -> gpui_kit::Div {
        let t = bookkeepy_primary_data::theme::style("overline");
        let mut el = div()
            .text_color(cx.theme().muted_foreground)
            .child(text.into());
        if let Some(s) = t {
            el = el
                .text_size(px(s.size_px as f32))
                .font_weight(gpui_kit::FontWeight::from(s.weight as f32));
        }
        el
    }

    /// One matrix cell: state caption above the live component.
    fn cell(cx: &App, state_key: &str, el: impl IntoElement) -> gpui_kit::Div {
        v_flex()
            .gap_2()
            .child(Self::overline(cx, tr(cx, state_key)))
            .child(el)
    }

    /// One variant row of the matrix: row label + the 8 state cells.
    fn variant_row(
        &mut self,
        cx: &mut Context<'_, Self>,
        variant_key: &str,
        make: &dyn Fn(String) -> Button,
    ) -> gpui_kit::Div {
        let states = [
            "gallery.atoms.state.rest",
            "gallery.atoms.state.hover",
            "gallery.atoms.state.pressed",
            "gallery.atoms.state.focus",
            "gallery.atoms.state.selected",
            "gallery.atoms.state.open",
            "gallery.atoms.state.loading",
            "gallery.atoms.state.disabled",
        ];
        let mut cells = h_flex().gap_6();
        for (i, key) in states.iter().enumerate() {
            let id = format!("{variant_key}-cell-{i}");
            let mut b = make(id).on_click(cx.listener(Self::on_click));
            match i {
                4 => b = b.selected(true),                      // selected
                5 => b = b.selected(true).dropdown_caret(true), // open (menu showing)
                6 => b = b.loading(true),                       // loading
                7 => b = b.disabled(true),                      // disabled
                _ => {}                                         // rest/hover/pressed/focus: live
            }
            cells = cells.child(Self::cell(cx, key, b));
        }
        h_flex()
            .gap_6()
            .items_start()
            .child(
                div()
                    .w(px(120.))
                    .pt_1()
                    .child(Self::overline(cx, tr(cx, variant_key))),
            )
            .child(cells)
    }
}

impl gpui_kit::Render for AtomsButton {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let body = bookkeepy_primary_data::theme::style("body");
        let title = bookkeepy_primary_data::theme::style("title");
        let caption = |cx: &App, text: SharedString| {
            let mut el = div().text_color(cx.theme().muted_foreground).child(text);
            if let Some(s) = body {
                el = el.text_size(px(s.size_px as f32)).line_height(
                    gpui_kit::DefiniteLength::Fraction(s.line_height_px as f32 / s.size_px as f32),
                );
            }
            el
        };
        let mut title_el = div()
            .text_color(cx.theme().foreground)
            .child(tr(cx, "gallery.atoms.button.title"));
        if let Some(s) = title {
            title_el = title_el
                .text_size(px(s.size_px as f32))
                .font_weight(gpui_kit::FontWeight::from(s.weight as f32));
        }

        // ── variant × interaction matrix ─────────────────────────────────────
        let l_add = tr(cx, "gallery.atoms.demo.add_transaction");
        let l_filter = tr(cx, "gallery.atoms.demo.filter");
        let l_export = tr(cx, "gallery.atoms.demo.export");
        let l_cancel = tr(cx, "gallery.atoms.demo.cancel");
        let l_delete = tr(cx, "gallery.atoms.demo.delete");
        let matrix = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.matrix")))
            .child(caption(cx, tr(cx, "gallery.atoms.matrix.hint")))
            .child(
                v_flex()
                    .gap_6()
                    .child(
                        self.variant_row(cx, "gallery.atoms.variant.primary", &|id: String| {
                            Button::new(id)
                                .primary()
                                .icon(IconName::Plus)
                                .label(l_add.clone())
                        }),
                    )
                    .child(self.variant_row(
                        cx,
                        "gallery.atoms.variant.secondary",
                        &|id: String| {
                            Button::new(id)
                                .icon(IconName::Funnel)
                                .label(l_filter.clone())
                        },
                    ))
                    .child(
                        self.variant_row(cx, "gallery.atoms.variant.outline", &|id: String| {
                            Button::new(id)
                                .outline()
                                .icon(IconName::Download)
                                .label(l_export.clone())
                        }),
                    )
                    .child(
                        self.variant_row(cx, "gallery.atoms.variant.ghost", &|id: String| {
                            Button::new(id).ghost().label(l_cancel.clone())
                        }),
                    )
                    .child(
                        self.variant_row(cx, "gallery.atoms.variant.danger", &|id: String| {
                            Button::new(id)
                                .danger()
                                .icon(IconName::X)
                                .label(l_delete.clone())
                        }),
                    ),
            );

        // ── sizes ────────────────────────────────────────────────────────────
        let l_add = tr(cx, "gallery.atoms.demo.add_transaction");
        let l_filter = tr(cx, "gallery.atoms.demo.filter");
        let l_cancel = tr(cx, "gallery.atoms.demo.cancel");
        let l_compact = tr(cx, "gallery.atoms.demo.compact");
        let sizes_row = |cx: &mut Context<'_, Self>, make: &dyn Fn(String) -> Button| {
            h_flex().gap_4().children(
                [
                    ControlSize::XSmall,
                    ControlSize::Small,
                    ControlSize::Medium,
                    ControlSize::Large,
                ]
                .iter()
                .enumerate()
                .map(|(i, size)| {
                    let b = make(format!("size-{i}")).on_click(cx.listener(Self::on_click));
                    button_size(b, *size)
                }),
            )
        };
        let sizes = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.button.sizes")))
            .child(caption(cx, tr(cx, "gallery.atoms.sizes.hint")))
            .child(sizes_row(cx, &|id: String| {
                Button::new(id)
                    .primary()
                    .icon(IconName::Plus)
                    .label(l_add.clone())
            }))
            .child(sizes_row(cx, &|id: String| {
                Button::new(id)
                    .icon(IconName::Funnel)
                    .label(l_filter.clone())
            }))
            .child(sizes_row(cx, &|id: String| {
                Button::new(id).ghost().label(l_cancel.clone())
            }))
            .child(
                h_flex().gap_4().child(
                    Button::new("compact")
                        .label(l_compact)
                        .compact()
                        .on_click(cx.listener(Self::on_click)),
                ),
            );

        // ── anatomy ──────────────────────────────────────────────────────────
        let anatomy = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.anatomy")))
            .child(
                h_flex().gap_4().items_center().children([
                    Self::cell(
                        cx,
                        "gallery.atoms.anatomy.label_only",
                        Button::new("anat-label")
                            .label(tr(cx, "gallery.atoms.demo.save"))
                            .on_click(cx.listener(Self::on_click)),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.anatomy.icon_label",
                        Button::new("anat-icon-label")
                            .icon(IconName::Plus)
                            .label(tr(cx, "gallery.atoms.demo.add_transaction"))
                            .on_click(cx.listener(Self::on_click)),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.anatomy.dropdown",
                        Button::new("anat-dropdown")
                            .label(tr(cx, "gallery.atoms.anatomy.dropdown"))
                            .dropdown_caret(true)
                            .on_click(cx.listener(Self::on_click)),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.anatomy.all_three",
                        Button::new("anat-all")
                            .icon(IconName::Funnel)
                            .label(tr(cx, "gallery.atoms.anatomy.all_three"))
                            .dropdown_caret(true)
                            .on_click(cx.listener(Self::on_click)),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.anatomy.rounded_full",
                        button_size(
                            Button::new("anat-full")
                                .label(tr(cx, "gallery.atoms.demo.save"))
                                .rounded(gpui_kit::component::button::ButtonRounded::Size(px(999.)))
                                .on_click(cx.listener(Self::on_click)),
                            ControlSize::Small,
                        ),
                    ),
                ]),
            )
            .child(Self::cell(
                cx,
                "gallery.atoms.anatomy.segmented",
                ButtonGroup::new("anat-segmented")
                    .child(
                        Button::new("seg-day")
                            .label(tr(cx, "gallery.atoms.demo.day"))
                            .selected(true),
                    )
                    .child(Button::new("seg-week").label(tr(cx, "gallery.atoms.demo.week")))
                    .child(Button::new("seg-month").label(tr(cx, "gallery.atoms.demo.month")))
                    .child(Button::new("seg-year").label(tr(cx, "gallery.atoms.demo.year"))),
            ));

        // ── icon buttons ─────────────────────────────────────────────────────
        let icon_btn = |cx: &mut Context<'_, Self>, id: String, icon: IconName| {
            Button::new(id)
                .ghost()
                .icon(icon)
                .on_click(cx.listener(Self::on_click))
        };
        let icon_buttons = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.icon_button")))
            .child(
                h_flex().gap_6().children(
                    [
                        ("gallery.atoms.state.rest", false),
                        ("gallery.atoms.state.selected", true),
                        ("gallery.atoms.state.disabled", false),
                    ]
                    .iter()
                    .enumerate()
                    .map(|(i, (key, sel))| {
                        let mut b = icon_btn(cx, format!("icon-btn-{i}"), IconName::Search);
                        if *sel {
                            b = b.selected(true);
                        }
                        if i == 2 {
                            b = b.disabled(true);
                        }
                        Self::cell(cx, key, b)
                    }),
                ),
            )
            .child(
                h_flex()
                    .gap_4()
                    .child(icon_btn(cx, "icon-eye".to_string(), IconName::EyeOff))
                    .child(icon_btn(cx, "icon-x".to_string(), IconName::X)),
            );

        // ── link ─────────────────────────────────────────────────────────────
        // ATOMS·01 Link: external URLs only. rest = underlined secondary text + external
        // glyph (12px) · hover = foreground, glyph duotone · pressed = muted ·
        // focus = 2px ring · disabled = no underline. Built on Button.link() (base::Link).
        let l_bank = tr(cx, "gallery.atoms.demo.open_bank");
        let link_states = [
            "gallery.atoms.state.rest",
            "gallery.atoms.state.hover",
            "gallery.atoms.state.pressed",
            "gallery.atoms.state.focus",
            "gallery.atoms.state.disabled",
        ];
        let link_row = h_flex()
            .gap_6()
            .children(link_states.iter().enumerate().map(|(i, key)| {
                let mut b = Button::new(format!("link-{i}"))
                    .link()
                    .underline()
                    .child(
                        h_flex().gap_1().items_center().child(l_bank.clone()).child(
                            Icon::new(IconName::ArrowUpRight)
                                .with_size(gpui_kit::component::Size::Size(px(12.))),
                        ),
                    )
                    .on_click(cx.listener(Self::on_click));
                if i == 4 {
                    b = b.disabled(true);
                }
                Self::cell(cx, key, b)
            }));
        let l_lead = tr(cx, "gallery.atoms.link.inline_lead");
        let l_inline = tr(cx, "gallery.atoms.link.inline_link");
        let link = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.link.title")))
            .child(caption(cx, tr(cx, "gallery.atoms.link.hint")))
            .child(link_row)
            .child(
                h_flex()
                    .gap_1()
                    .items_baseline()
                    .child(caption(cx, l_lead))
                    .child(
                        Button::new("link-inline")
                            .link()
                            .underline()
                            .label(l_inline)
                            .on_click(cx.listener(Self::on_click)),
                    ),
            );

        // ── icon ─────────────────────────────────────────────────────────────
        // ATOMS·01 Icon: no plate — icons never sit on a background; regular at rest,
        // duotone on interaction; colors come from the semantic icon roles only.
        let icon_sizes_row = h_flex().gap_6().items_center().children(
            [
                ("xs", 12.),
                ("sm", 14.),
                ("md", 16.),
                ("lg", 20.),
                ("xl", 24.),
            ]
            .iter()
            .map(|(name, s)| {
                v_flex()
                    .gap_2()
                    .items_center()
                    .child(Self::overline(
                        cx,
                        SharedString::from(format!("{name} {s:.0}")),
                    ))
                    .child(
                        Icon::new(IconName::House)
                            .with_size(gpui_kit::component::Size::Size(px(*s))),
                    )
            }),
        );
        let mode = cx.global::<Prefs>().mode;
        let icon_roles = [
            "icon",
            "icon-muted",
            "icon-hover",
            "icon-selected",
            "icon-disabled",
            "icon-attention",
            "success",
            "danger",
            "warning",
            "money-in",
        ];
        let icon_roles_row = h_flex()
            .gap_6()
            .items_center()
            .children(icon_roles.iter().map(|role| {
                let color = bookkeepy_ui::api::language::role_hsla(mode, role).unwrap_or_default();
                v_flex()
                    .gap_2()
                    .items_center()
                    .child(Self::overline(cx, SharedString::from(*role)))
                    .child(Icon::new(IconName::CircleCheck).text_color(color))
            }));
        let icon = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.icon.title")))
            .child(caption(cx, tr(cx, "gallery.atoms.icon.hint")))
            .child(icon_sizes_row)
            .child(Self::overline(cx, tr(cx, "gallery.atoms.icon.roles")))
            .child(icon_roles_row);

        // ── label ────────────────────────────────────────────────────────────
        // ATOMS·01 Label: default = label style in foreground · secondary suffix = caption
        // muted · required * muted · masked •••• + eye · highlight = match on accent-subtle
        // (Full) / accent-foreground (Prefix) · disabled / invalid.
        let fg = cx.theme().foreground;
        let labels = v_flex()
            .gap_4()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.label.title")))
            .child(caption(cx, tr(cx, "gallery.atoms.label.hint")))
            .child(
                h_flex().gap_6().items_center().children([
                    Self::cell(
                        cx,
                        "gallery.atoms.label.default",
                        Label::new(tr(cx, "gallery.atoms.demo.payee"))
                            .text_size(px(12.))
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(fg),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.label.secondary",
                        Label::new(tr(cx, "gallery.atoms.demo.account"))
                            .secondary(tr(cx, "gallery.atoms.demo.optional")),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.label.required",
                        Label::new(tr(cx, "gallery.atoms.demo.amount")).secondary("*"),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.label.masked",
                        Label::new("\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}")
                            .secondary(tr(cx, "gallery.atoms.demo.ron")),
                    ),
                ]),
            )
            .child(
                h_flex().gap_6().items_center().children([
                    Self::cell(
                        cx,
                        "gallery.atoms.label.highlight_full",
                        Label::new(tr(cx, "gallery.atoms.demo.mega_image")).highlights(
                            gpui_kit::component::label::HighlightsMatch::Full(tr(
                                cx,
                                "gallery.atoms.demo.mega",
                            )),
                        ),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.label.highlight_prefix",
                        Label::new(tr(cx, "gallery.atoms.demo.netflix")).highlights(
                            gpui_kit::component::label::HighlightsMatch::Prefix(tr(
                                cx,
                                "gallery.atoms.demo.net",
                            )),
                        ),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.label.disabled",
                        Label::new(tr(cx, "gallery.atoms.demo.currency"))
                            .secondary(tr(cx, "gallery.atoms.demo.locked"))
                            .text_color(cx.theme().muted_foreground.opacity(0.5)),
                    ),
                    Self::cell(
                        cx,
                        "gallery.atoms.label.invalid",
                        Label::new(tr(cx, "gallery.atoms.demo.amount"))
                            .text_color(cx.theme().red_light),
                    ),
                ]),
            )
            .child(Self::overline(cx, tr(cx, "gallery.atoms.label.sizes")))
            .child(
                h_flex().gap_6().items_baseline().children(
                    [
                        "money", "overline", "caption", "label", "body-lg", "heading",
                    ]
                    .iter()
                    .map(|st| {
                        let style = bookkeepy_primary_data::theme::style(st);
                        let mut el = div()
                            .text_color(fg)
                            .child(tr(cx, "gallery.atoms.demo.payee"));
                        if let Some(s) = style {
                            el = el
                                .text_size(px(s.size_px as f32))
                                .font_weight(gpui_kit::FontWeight::from(s.weight as f32));
                        }
                        v_flex()
                            .gap_2()
                            .child(Self::overline(cx, SharedString::from(*st)))
                            .child(el)
                    }),
                ),
            );

        // ── kbd ──────────────────────────────────────────────────────────────
        // ATOMS·01 Kbd: keycap = raised key with a 2px base · plain = appearance(false)
        // inside menus/dense rows · inverse = on the light tooltip surface · echo shows
        // the platform text (Windows: Ctrl+Shift+P · macOS: ⇧⌘P) — never hard-coded keys.
        let kbd_keys = ["ctrl-k", "ctrl-n", "ctrl-shift-f", "ctrl-0", "esc", "enter"];
        let kbd_keycap = h_flex()
            .gap_2()
            .items_center()
            .children(kbd_keys.iter().map(|k| {
                bookkeepy_ui::api::language::controls::kbd(
                    Kbd::new(Keystroke::parse(k).unwrap_or_default()),
                    mode,
                )
            }));
        let kbd_plain = h_flex().gap_2().items_center().children(
            kbd_keys
                .iter()
                .map(|k| Kbd::new(Keystroke::parse(k).unwrap_or_default()).appearance(false)),
        );
        let tooltip_surface =
            bookkeepy_ui::api::language::role_hsla(mode, "tooltip").unwrap_or_default();
        let kbd_inverse = h_flex()
            .w(gpui_kit::Length::Auto)
            .bg(tooltip_surface)
            .rounded(px(bookkeepy_ui::api::language::radius_tier("sm")))
            .p_2()
            .child(
                Kbd::new(Keystroke::parse("ctrl-shift-p").unwrap_or_default()).text_color(
                    bookkeepy_ui::api::language::role_hsla(mode, "tooltip-foreground")
                        .unwrap_or_default(),
                ),
            );
        let kbd = v_flex()
            .gap_4()
            .items_start()
            .child(Self::overline(cx, tr(cx, "gallery.atoms.kbd.title")))
            .child(caption(cx, tr(cx, "gallery.atoms.kbd.hint")))
            .child(Self::cell(cx, "gallery.atoms.kbd.keycap", kbd_keycap))
            .child(Self::cell(cx, "gallery.atoms.kbd.plain", kbd_plain))
            .child(Self::cell(cx, "gallery.atoms.kbd.inverse", kbd_inverse));

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-20") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .child(Self::overline(cx, tr(cx, "gallery.atoms.sheet")))
            .child(title_el)
            .child(caption(cx, tr(cx, "gallery.atoms.button.subtitle")))
            .child(
                h_flex()
                    .gap_2()
                    .child(caption(cx, tr(cx, "gallery.atoms.clicks")))
                    .child(caption(cx, SharedString::from(self.clicks.to_string()))),
            )
            .child(matrix)
            .child(sizes)
            .child(anatomy)
            .child(icon_buttons)
            .child(link)
            .child(icon)
            .child(labels)
            .child(kbd)
    }
}
