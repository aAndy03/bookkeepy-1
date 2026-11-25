// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 01 sheet — button, link, icon, label, kbd (references/designlanguage).
//!
//! Rendered FLAT on the page background exactly like the exported design frames: no
//! cards behind elements (a card only appears when it is part of a component's own
//! design). Groups are separated by spacing only. Every control is LIVE — hover,
//! press or tab the cells to see the designed interaction states (rest/hover/pressed/
//! focus are runtime states; selected/open/loading/disabled are forced via the
//! component flags).
//!
//! SCALING: this file only DECLARES the sheet as data (variant × state tables) and
//! calls the shared atoms from `bookkeepy_ui::api::language::atoms` — the same
//! components the app reuses (doc 04 §4.9 / ADR-0017). Layout scaffolding lives in
//! `super::kit`, so new sheets are new tables, not new layout code.

use bookkeepy_ui::api::language::atoms;
use bookkeepy_ui::api::language::controls::{ControlSize, button_size};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::button::{Button, ButtonGroup, ButtonRounded, ButtonVariants as _};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _};
use gpui_kit::{
    AnyView, App, AppContext as _, ClickEvent, Context, Keystroke, ParentElement as _,
    SharedString, Styled as _, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| AtomsButton::default()).into()
}

/// The ATOMS · 01 section view. Holds demo state so the specimens are visibly live
/// (click counter, mask reveal toggle — amounts start masked, as drawn).
pub struct AtomsButton {
    clicks: usize,
    masked: bool,
}

impl Default for AtomsButton {
    fn default() -> Self {
        Self {
            clicks: 0,
            masked: true,
        }
    }
}

impl AtomsButton {
    fn on_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<'_, Self>) {
        self.clicks += 1;
        cx.notify();
    }

    fn on_toggle_mask(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<'_, Self>) {
        self.masked = !self.masked;
        cx.notify();
    }

    /// One variant row of the state matrix: row label + the 8 state cells.
    fn variant_row(
        &mut self,
        cx: &mut Context<'_, Self>,
        variant_key: &str,
        make: &dyn Fn(String) -> Button,
    ) -> gpui_kit::Div {
        let cells = kit::STATE_KEYS.iter().enumerate().map(|(i, key)| {
            let id = format!("{variant_key}-cell-{i}");
            let mut b = make(id).on_click(cx.listener(Self::on_click));
            match i {
                4 => b = b.selected(true),                      // selected
                5 => b = b.selected(true).dropdown_caret(true), // open (menu showing)
                6 => b = b.loading(true),                       // loading
                7 => b = b.disabled(true),                      // disabled
                _ => {}                                         // rest/hover/pressed/focus: live
            }
            kit::cell(cx, key, b)
        });
        h_flex()
            .gap_6()
            .items_start()
            .child(
                div()
                    .w(px(120.))
                    .pt_1()
                    .child(kit::overline(cx, tr(cx, variant_key))),
            )
            // equal columns: the design matrix aligns every state column across rows
            .child(div().flex_1().grid().grid_cols(8).gap_4().children(cells))
    }
}

impl gpui_kit::Render for AtomsButton {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── variant × interaction matrix ─────────────────────────────────────
        // ATOMS · 01 matrix: rows = variants, columns = interactions. Icons are the
        // sheet's glyphs (plus / funnel / export / — / x); upload maps to phosphor export.
        let l_add = tr(cx, "gallery.atoms.demo.add_transaction");
        let l_filter = tr(cx, "gallery.atoms.demo.filter");
        let l_export = tr(cx, "gallery.atoms.demo.export");
        let l_cancel = tr(cx, "gallery.atoms.demo.cancel");
        let l_delete = tr(cx, "gallery.atoms.demo.delete");
        let rows: [(&str, &dyn Fn(String) -> Button); 5] = [
            ("gallery.atoms.variant.primary", &|id: String| {
                Button::new(id)
                    .primary()
                    .icon(atoms::button_icon(IconName::Plus, ControlSize::Medium))
                    .label(l_add.clone())
            }),
            ("gallery.atoms.variant.secondary", &|id: String| {
                Button::new(id)
                    .icon(atoms::button_icon(IconName::Funnel, ControlSize::Medium))
                    .label(l_filter.clone())
            }),
            ("gallery.atoms.variant.outline", &|id: String| {
                Button::new(id)
                    .outline()
                    .icon(atoms::button_icon(IconName::Upload, ControlSize::Medium)) // → phosphor export
                    .label(l_export.clone())
            }),
            ("gallery.atoms.variant.ghost", &|id: String| {
                Button::new(id).ghost().label(l_cancel.clone())
            }),
            ("gallery.atoms.variant.danger", &|id: String| {
                Button::new(id)
                    .danger()
                    .icon(atoms::button_icon(IconName::X, ControlSize::Medium))
                    .label(l_delete.clone())
            }),
        ];
        let matrix = v_flex().gap_6().children(
            rows.iter()
                .map(|(key, make)| self.variant_row(cx, key, *make)),
        );
        let matrix_group = kit::group(
            cx,
            "gallery.atoms.matrix",
            "gallery.atoms.matrix.hint",
            matrix,
        );

        // ── sizes ────────────────────────────────────────────────────────────
        // xsmall 20 · small 24 · medium 32 · large 40 (radius 6/6/6/8) + compact override.
        let sizes_row = |cx: &mut Context<'_, Self>,
                         make: &dyn Fn(String, ControlSize) -> Button| {
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
                    let b = make(format!("size-{i}"), *size).on_click(cx.listener(Self::on_click));
                    button_size(b, *size)
                }),
            )
        };
        let sizes = v_flex()
            .gap_4()
            .child(sizes_row(cx, &|id: String, size: ControlSize| {
                Button::new(id)
                    .primary()
                    .icon(atoms::button_icon(IconName::Plus, size))
                    .label(l_add.clone())
            }))
            .child(sizes_row(cx, &|id: String, size: ControlSize| {
                Button::new(id)
                    .icon(atoms::button_icon(IconName::Funnel, size))
                    .label(l_filter.clone())
            }))
            .child(sizes_row(cx, &|id: String, _size: ControlSize| {
                Button::new(id).ghost().label(l_cancel.clone())
            }))
            .child(
                h_flex().gap_4().child(
                    Button::new("compact")
                        .label(tr(cx, "gallery.atoms.demo.compact"))
                        .compact()
                        .on_click(cx.listener(Self::on_click)),
                ),
            );
        let sizes_group = kit::group(
            cx,
            "gallery.atoms.button.sizes",
            "gallery.atoms.sizes.hint",
            sizes,
        );

        // ── anatomy ──────────────────────────────────────────────────────────
        let l_save = tr(cx, "gallery.atoms.demo.save");
        let anatomy = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.anatomy.label_only",
                Button::new("anat-label")
                    .label(l_save.clone())
                    .on_click(cx.listener(Self::on_click)),
            ),
            kit::cell(
                cx,
                "gallery.atoms.anatomy.icon_label",
                Button::new("anat-icon-label")
                    .icon(atoms::button_icon(IconName::Plus, ControlSize::Medium))
                    .label(l_add.clone())
                    .on_click(cx.listener(Self::on_click)),
            ),
            kit::cell(
                cx,
                "gallery.atoms.anatomy.dropdown",
                Button::new("anat-dropdown")
                    .label(tr(cx, "gallery.atoms.anatomy.dropdown"))
                    .dropdown_caret(true)
                    .on_click(cx.listener(Self::on_click)),
            ),
            kit::cell(
                cx,
                "gallery.atoms.anatomy.all_three",
                Button::new("anat-all")
                    .icon(atoms::button_icon(IconName::Funnel, ControlSize::Medium))
                    .label(tr(cx, "gallery.atoms.anatomy.all_three"))
                    .dropdown_caret(true)
                    .on_click(cx.listener(Self::on_click)),
            ),
            kit::cell(
                cx,
                "gallery.atoms.anatomy.rounded_full",
                button_size(
                    Button::new("anat-full")
                        .label(l_save.clone())
                        .rounded(ButtonRounded::Size(px(999.)))
                        .on_click(cx.listener(Self::on_click)),
                    ControlSize::Small,
                ),
            ),
        ])
        .child(kit::cell(
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
        let anatomy_group = kit::group(
            cx,
            "gallery.atoms.anatomy",
            "gallery.atoms.anatomy.hint",
            anatomy,
        );

        // ── icon buttons ─────────────────────────────────────────────────────
        // 4 glyphs × 8 states, ghost (no plate; glyph switches regular → duotone on
        // interaction — the duotone rows below show the target weight).
        let glyphs = [
            IconName::Settings,
            IconName::Search,
            IconName::EyeOff,
            IconName::X,
        ];
        let icon_btn_rows =
            v_flex()
                .gap_6()
                .children(glyphs.iter().enumerate().map(|(g, icon)| {
                    let cells = kit::STATE_KEYS.iter().enumerate().map(|(i, key)| {
                        let mut b = Button::new(format!("ib-{g}-{i}"))
                            .ghost()
                            .icon(atoms::icon(*icon, 16.)) // 32px control → 16px glyph
                            .on_click(cx.listener(Self::on_click));
                        match i {
                            4 | 5 => b = b.selected(true),
                            6 => b = b.loading(true),
                            7 => b = b.disabled(true),
                            _ => {}
                        }
                        kit::cell(cx, key, b)
                    });
                    div().grid().grid_cols(8).gap_4().children(cells)
                }));
        let icon_buttons_group = kit::group(
            cx,
            "gallery.atoms.icon_button",
            "gallery.atoms.icon_button.hint",
            icon_btn_rows,
        );

        // ── link ─────────────────────────────────────────────────────────────
        // ATOMS·01 Link: external URLs only, underlined secondary text + 12px
        // arrow-up-right glyph. Shared atom: `language::atoms::external_link`.
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
            .items_start()
            .children(link_states.iter().enumerate().map(|(i, key)| {
                let mut b = atoms::external_link(format!("link-{i}"), l_bank.clone())
                    .on_click(cx.listener(Self::on_click));
                if i == 4 {
                    b = b.disabled(true);
                }
                kit::cell(cx, key, b)
            }));
        let link_inline = h_flex()
            .gap_1()
            .items_baseline()
            .child(kit::caption(cx, tr(cx, "gallery.atoms.link.inline_lead")))
            .child(
                atoms::external_link("link-inline", tr(cx, "gallery.atoms.link.inline_link"))
                    .on_click(cx.listener(Self::on_click)),
            )
            .child(kit::caption(cx, tr(cx, "gallery.atoms.link.inline_tail")));
        let link = v_flex().gap_4().child(link_row).child(link_inline);
        let link_group = kit::group(
            cx,
            "gallery.atoms.link.title",
            "gallery.atoms.link.hint",
            link,
        );

        // ── icon ─────────────────────────────────────────────────────────────
        // Size × weight: wallet at xs 12 … xl 24 in BOTH weights (regular = rest,
        // duotone = interaction), then the semantic color roles with their glyphs.
        let sizes = [
            ("xs", 12.),
            ("sm", 14.),
            ("md", 16.),
            ("lg", 20.),
            ("xl", 24.),
        ];
        let weight_row = |cx: &App, duotone: bool, caption_key: &str| {
            h_flex()
                .gap_6()
                .items_end()
                .child(kit::overline(cx, tr(cx, caption_key)))
                .children(sizes.iter().map(|(name, s)| {
                    v_flex()
                        .gap_2()
                        .items_center()
                        .child(kit::overline(
                            cx,
                            SharedString::from(format!("{name} {s:.0}")),
                        ))
                        .child(if duotone {
                            atoms::duotone_icon(IconName::Wallet, *s)
                        } else {
                            atoms::icon(IconName::Wallet, *s)
                        })
                }))
        };
        // color roles: (role, glyph, duotone?) — ATOMS · 01 icon sheet.
        let roles: [(&str, IconName, bool); 10] = [
            ("icon", IconName::Bell, false),
            ("icon-muted", IconName::Bell, false),
            ("icon-hover", IconName::Bell, true),
            ("icon-selected", IconName::Bell, true),
            ("icon-disabled", IconName::Bell, false),
            ("icon-attention", IconName::Bell, true),
            ("success", IconName::CircleCheck, true),
            ("danger", IconName::CircleAlert, true),
            ("warning", IconName::CircleAlert, true),
            ("money-in", IconName::TrendingUp, false),
        ];
        let roles_row =
            h_flex()
                .gap_6()
                .items_end()
                .children(roles.iter().map(|(role, glyph, duo)| {
                    v_flex()
                        .gap_2()
                        .items_center()
                        .child(kit::overline(cx, SharedString::from(*role)))
                        .child(if *duo {
                            let mut ic = atoms::duotone_icon(*glyph, 20.);
                            if let Some(c) = bookkeepy_ui::api::language::role_hsla(mode, role) {
                                ic = ic.text_color(c);
                            }
                            ic
                        } else {
                            atoms::role_icon(*glyph, 20., role, mode)
                        })
                }));
        let icon = v_flex()
            .gap_4()
            .child(weight_row(cx, false, "gallery.atoms.icon.weight_regular"))
            .child(weight_row(cx, true, "gallery.atoms.icon.weight_duotone"))
            .child(kit::overline(cx, tr(cx, "gallery.atoms.icon.roles")))
            .child(roles_row);
        let icon_group = kit::group(
            cx,
            "gallery.atoms.icon.title",
            "gallery.atoms.icon.hint",
            icon,
        );

        // ── label ────────────────────────────────────────────────────────────
        // Shared atoms from `language::atoms`: label (12/500), field_label (label +
        // caption suffix — the suffix is NOT on the same level), masked_label (+ eye
        // toggle), highlighted (match on accent-subtle/accent-foreground).
        let l_account = tr(cx, "gallery.atoms.demo.account");
        let l_note = tr(cx, "gallery.atoms.demo.note");
        let l_optional = tr(cx, "gallery.atoms.demo.optional");
        let l_amount = tr(cx, "gallery.atoms.demo.amount");
        let l_value = tr(cx, "gallery.atoms.demo.amount_value");
        let l_ron = tr(cx, "gallery.atoms.demo.ron");
        let l_currency = tr(cx, "gallery.atoms.demo.currency");
        let l_locked = tr(cx, "gallery.atoms.demo.locked");
        let l_required = tr(cx, "gallery.atoms.demo.required");
        let disabled_fg = bookkeepy_ui::api::language::role_hsla(mode, "disabled-foreground");
        let danger_fg = bookkeepy_ui::api::language::role_hsla(mode, "danger-foreground");

        // masked cell: live eye toggle reveals/hides the amount
        let masked_cell = if self.masked {
            h_flex()
                .gap_1()
                .items_baseline()
                .child(atoms::masked_label(l_value.clone(), l_ron.clone(), mode))
                .child(
                    Button::new("mask-toggle")
                        .ghost()
                        .icon(IconName::EyeOff)
                        .with_size(gpui_kit::component::Size::Size(px(14.)))
                        .on_click(cx.listener(Self::on_toggle_mask)),
                )
        } else {
            h_flex()
                .gap_1()
                .items_baseline()
                .child(atoms::field_label(l_value.clone(), l_ron.clone(), mode))
                .child(
                    Button::new("mask-toggle")
                        .ghost()
                        .icon(IconName::Eye)
                        .with_size(gpui_kit::component::Size::Size(px(14.)))
                        .on_click(cx.listener(Self::on_toggle_mask)),
                )
        };

        let labels_row1 = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.label.default",
                atoms::label(l_account.clone()),
            ),
            kit::cell(
                cx,
                "gallery.atoms.label.secondary",
                atoms::field_label(l_note.clone(), l_optional.clone(), mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.label.required",
                atoms::field_label(l_amount.clone(), "*", mode),
            ),
            kit::cell(cx, "gallery.atoms.label.masked", masked_cell),
        ]);
        let mut disabled_label = atoms::label(l_currency.clone());
        if let Some(c) = disabled_fg {
            disabled_label = disabled_label.text_color(c);
        }
        let mut disabled_suffix = atoms::suffix(l_locked.clone(), mode);
        if let Some(c) = disabled_fg {
            disabled_suffix = disabled_suffix.text_color(c);
        }
        let mut invalid_label = atoms::label(l_amount.clone());
        if let Some(c) = danger_fg {
            invalid_label = invalid_label.text_color(c);
        }
        let mut invalid_suffix = atoms::suffix(l_required.clone(), mode);
        if let Some(c) = danger_fg {
            invalid_suffix = invalid_suffix.text_color(c);
        }
        let labels_row2 = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.label.highlight_full",
                atoms::highlighted(
                    tr(cx, "gallery.atoms.demo.mega_image"),
                    &tr(cx, "gallery.atoms.demo.mega"),
                    false,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.label.highlight_prefix",
                atoms::highlighted(
                    tr(cx, "gallery.atoms.demo.netflix"),
                    &tr(cx, "gallery.atoms.demo.net"),
                    true,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.label.disabled",
                h_flex()
                    .gap_1()
                    .items_baseline()
                    .child(disabled_label)
                    .child(disabled_suffix),
            ),
            kit::cell(
                cx,
                "gallery.atoms.label.invalid",
                h_flex()
                    .gap_1()
                    .items_baseline()
                    .child(invalid_label)
                    .child(invalid_suffix),
            ),
        ]);
        // sizes: labels use the named type styles (money now included).
        let sizes_ladder = h_flex().gap_6().items_end().children(
            [
                "money", "overline", "caption", "label", "body-lg", "heading",
            ]
            .iter()
            .map(|st| {
                v_flex()
                    .gap_2()
                    .child(kit::overline(cx, SharedString::from(*st)))
                    .child(kit::with_style(
                        div()
                            .text_color(cx.theme().foreground)
                            .child(tr(cx, "gallery.atoms.demo.payee")),
                        st,
                    ))
            }),
        );
        let labels = v_flex()
            .gap_4()
            .child(labels_row1)
            .child(labels_row2)
            .child(kit::overline(cx, tr(cx, "gallery.atoms.label.sizes")))
            .child(sizes_ladder);
        let label_group = kit::group(
            cx,
            "gallery.atoms.label.title",
            "gallery.atoms.label.hint",
            labels,
        );

        // ── kbd ──────────────────────────────────────────────────────────────
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
        let kbd_inverse = div()
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
            .child(kit::cell(cx, "gallery.atoms.kbd.keycap", kbd_keycap))
            .child(kit::cell(cx, "gallery.atoms.kbd.plain", kbd_plain))
            .child(kit::cell(cx, "gallery.atoms.kbd.inverse", kbd_inverse));
        let kbd_group = kit::group(cx, "gallery.atoms.kbd.title", "gallery.atoms.kbd.hint", kbd);

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.atoms.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.atoms.button.title")))
            .child(kit::caption(cx, tr(cx, "gallery.atoms.button.subtitle")))
            .child(
                h_flex()
                    .gap_2()
                    .child(kit::caption(cx, tr(cx, "gallery.atoms.clicks")))
                    .child(kit::caption(
                        cx,
                        SharedString::from(self.clicks.to_string()),
                    )),
            )
            .child(matrix_group)
            .child(sizes_group)
            .child(anatomy_group)
            .child(icon_buttons_group)
            .child(link_group)
            .child(icon_group)
            .child(label_group)
            .child(kbd_group)
    }
}
