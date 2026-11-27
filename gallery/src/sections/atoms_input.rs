// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 03 sheet (part 1) — input, number input, textarea
//! (references/designlanguage).
//!
//! FLAT on the page background like the design frames. The rest cell of each matrix is
//! a LIVE stock input (type in it); hover/focus/loading/disabled/invalid/readonly are
//! painted from `bookkeepy_ui::api::language::fields` — same tokens, same geometry.

use bookkeepy_ui::api::language::{atoms, fields, fields::FieldIx};
use gpui_kit::assets::IconName;
use gpui_kit::base::v_flex;
use gpui_kit::component::input::{Input, InputState, NumberInput, Textarea, TextareaState};
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Styled as _,
    Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let payee =
        cx.new(|cx| InputState::new(window, cx).placeholder(tr(cx, "gallery.atoms03.demo.payee")));
    let payee_val = cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(tr(cx, "gallery.atoms03.demo.payee"))
            .default_value("Mega Image")
    });
    let amount = cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder("0,00")
            .default_value("142,50")
            .step(0.01)
    });
    let note = cx.new(|cx| {
        TextareaState::new(window, cx).placeholder(tr(cx, "gallery.atoms03.demo.note_ph"))
    });
    cx.new(|_cx| AtomsInput {
        payee,
        payee_val,
        amount,
        note,
    })
    .into()
}

pub struct AtomsInput {
    payee: Entity<InputState>,
    payee_val: Entity<InputState>,
    amount: Entity<InputState>,
    note: Entity<TextareaState>,
}

/// The sheet's field chrome on a LIVE input: sunken `neutral-950` fill, hairline
/// `input` border, rx6, h32, padding 0 10px (instance refinement lands last on the
/// frame, so these win over the theme's input_background mix).
fn live_input(input: Input, mode: bookkeepy_primary_data::theme::Mode) -> Input {
    let mut i = input
        .with_size(Size::Large)
        .w(px(280.))
        .h(px(32.))
        .px(px(10.))
        .rounded(px(bookkeepy_ui::api::language::radius_tier("md")));
    if let Some(bg) = bookkeepy_ui::api::language::prim_hsla("neutral-950") {
        i = i.bg(bg);
    }
    if let Some(b) = bookkeepy_ui::api::language::role_hsla(mode, "input") {
        i = i.border_1().border_color(b);
    }
    i
}

const IX_KEYS: [&str; 7] = [
    "gallery.atoms.state.rest",
    "gallery.atoms.state.hover",
    "gallery.atoms.state.focus",
    "gallery.atoms03.state.loading",
    "gallery.atoms.state.disabled",
    "gallery.atoms03.state.invalid",
    "gallery.atoms03.state.readonly",
];

const IX: [FieldIx; 7] = [
    FieldIx::Rest,
    FieldIx::Hover,
    FieldIx::Focus,
    FieldIx::Loading,
    FieldIx::Disabled,
    FieldIx::Invalid,
    FieldIx::Readonly,
];

impl gpui_kit::Render for AtomsInput {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let l_payee = tr(cx, "gallery.atoms03.demo.payee");
        let l_help = tr(cx, "gallery.atoms03.demo.payee_help");
        let l_required = tr(cx, "gallery.atoms03.demo.required");

        // ── input: normal × interaction matrix (rest = live) ─────────────────
        let row = |_ix: FieldIx, live: Option<Entity<InputState>>| {
            kit::row(IX.iter().enumerate().map(|(i, ix)| {
                let el: gpui_kit::AnyElement = match (i, live.clone()) {
                    (0, Some(state)) => live_input(
                        Input::new(&state).prefix(atoms::icon(IconName::Search, 14.)),
                        mode,
                    )
                    .into_any_element(),
                    _ => fields::input_cell(
                        l_payee.clone(),
                        *ix,
                        Some(IconName::Search),
                        l_payee.clone(),
                        true,
                        None,
                        Some(l_help.clone()),
                        32.,
                        mode,
                    )
                    .into_any_element(),
                };
                kit::cell(cx, IX_KEYS[i], el)
            }))
        };
        // invalid row carries the red help text; readonly row carries the lock glyph
        let invalid_row = kit::row(IX.iter().enumerate().map(|(i, ix)| {
            let suffix = (*ix == FieldIx::Readonly).then_some(IconName::Lock);
            kit::cell(
                cx,
                IX_KEYS[i],
                fields::input_cell(
                    l_payee.clone(),
                    *ix,
                    Some(IconName::Search),
                    l_payee.clone(),
                    true,
                    suffix,
                    Some(l_required.clone()),
                    32.,
                    mode,
                ),
            )
        }));
        let input_matrix = v_flex()
            .gap_6()
            .child(row(FieldIx::Rest, Some(self.payee.clone())))
            .child(invalid_row);

        // sizes 24 / 32 / 40 + password (live mask toggle) + cleanable (live)
        let sizes = kit::row([
            kit::cell(
                cx,
                "gallery.atoms03.input.s24",
                fields::input_cell(
                    l_payee.clone(),
                    FieldIx::Rest,
                    Some(IconName::Search),
                    l_payee.clone(),
                    true,
                    None,
                    None,
                    24.,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.input.s32",
                fields::input_cell(
                    l_payee.clone(),
                    FieldIx::Rest,
                    Some(IconName::Search),
                    l_payee.clone(),
                    true,
                    None,
                    None,
                    32.,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.input.s40",
                fields::input_cell(
                    l_payee.clone(),
                    FieldIx::Rest,
                    Some(IconName::Search),
                    l_payee.clone(),
                    true,
                    None,
                    None,
                    40.,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.input.password",
                live_input(
                    Input::new(&self.payee_val)
                        .mask_toggle()
                        .prefix(atoms::icon(IconName::Lock, 14.)),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.input.cleanable",
                live_input(Input::new(&self.payee_val).cleanable(true), mode),
            ),
        ]);
        let input_group = kit::group(
            cx,
            "gallery.atoms03.input.title",
            "gallery.atoms03.input.hint",
            v_flex().gap_6().child(input_matrix).child(sizes),
        );

        // ── number input ─────────────────────────────────────────────────────
        let num_live = {
            let mut n = NumberInput::new(&self.amount)
                .suffix(
                    div()
                        .text_size(px(11.))
                        .text_color(
                            bookkeepy_ui::api::language::role_hsla(mode, "icon-muted")
                                .unwrap_or_default(),
                        )
                        .child("RON"),
                )
                .w(px(220.))
                .h(px(32.))
                .rounded(px(bookkeepy_ui::api::language::radius_tier("md")));
            if let Some(bg) = bookkeepy_ui::api::language::prim_hsla("neutral-950") {
                n = n.bg(bg);
            }
            if let Some(b) = bookkeepy_ui::api::language::role_hsla(mode, "input") {
                n = n.border_1().border_color(b);
            }
            n
        };
        let number_row = kit::row([
            kit::cell(cx, "gallery.atoms.state.rest", num_live),
            kit::cell(
                cx,
                "gallery.atoms03.state.invalid",
                fields::input_cell(
                    tr(cx, "gallery.atoms03.demo.amount"),
                    FieldIx::Invalid,
                    None,
                    "−9.999,99",
                    false,
                    Some(IconName::CircleAlert),
                    Some(tr(cx, "gallery.atoms03.demo.below_min")),
                    32.,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.state.readonly",
                fields::input_cell(
                    tr(cx, "gallery.atoms03.demo.amount"),
                    FieldIx::Readonly,
                    None,
                    "+142,50",
                    false,
                    Some(IconName::Lock),
                    None,
                    32.,
                    mode,
                ),
            ),
        ]);
        let number_group = kit::group(
            cx,
            "gallery.atoms03.number.title",
            "gallery.atoms03.number.hint",
            number_row,
        );

        // ── textarea ─────────────────────────────────────────────────────────
        let ta_live = {
            let mut t = Textarea::new(&self.note)
                .h(px(96.))
                .w(px(300.))
                .rounded(px(bookkeepy_ui::api::language::radius_tier("md")));
            if let Some(bg) = bookkeepy_ui::api::language::prim_hsla("neutral-950") {
                t = t.bg(bg);
            }
            if let Some(b) = bookkeepy_ui::api::language::role_hsla(mode, "input") {
                t = t.border_1().border_color(b);
            }
            t
        };
        let textarea_row = kit::row([
            kit::cell(cx, "gallery.atoms.state.rest", ta_live),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                fields::textarea_cell(
                    tr(cx, "gallery.atoms03.demo.note"),
                    FieldIx::Focus,
                    tr(cx, "gallery.atoms03.demo.note_text"),
                    Some("48 / 240".into()),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.state.invalid",
                fields::textarea_cell(
                    tr(cx, "gallery.atoms03.demo.note"),
                    FieldIx::Invalid,
                    tr(cx, "gallery.atoms03.demo.note_text"),
                    Some("240 / 240".into()),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.state.readonly",
                fields::textarea_cell(
                    tr(cx, "gallery.atoms03.demo.note"),
                    FieldIx::Readonly,
                    tr(cx, "gallery.atoms03.demo.note_text"),
                    None,
                    mode,
                ),
            ),
        ]);
        let textarea_group = kit::group(
            cx,
            "gallery.atoms03.textarea.title",
            "gallery.atoms03.textarea.hint",
            textarea_row,
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.atoms03.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.atoms03.title")))
            .child(kit::caption(cx, tr(cx, "gallery.atoms03.subtitle")))
            .child(input_group)
            .child(number_group)
            .child(textarea_group)
    }
}
