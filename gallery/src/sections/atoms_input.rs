// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — the input family: Input, Textarea, NumberInput, OtpInput, InputGroup, Editor
//! (design sheet: "ATOMS · 03" + "04 otp input"). EVERY input has a localized
//! placeholder (RULE-UI-005 — `govern ui` fails on any input without a placeholder
//! i18n key); state is retained in this view (`InputState` etc. are entities).

use gpui_kit::component::input::{
    Editor, EditorState, Input, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText,
    InputState, NumberInput, OtpInput, OtpState, Textarea, TextareaState,
};
use gpui_kit::component::{Icon, IconName, Sizable as _};
use gpui_kit::{AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Window};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, section_page};

/// The input-atom section.
pub struct Section {
    search: gpui_kit::Entity<InputState>,
    notes: gpui_kit::Entity<TextareaState>,
    amount: gpui_kit::Entity<InputState>,
    otp: gpui_kit::Entity<OtpState>,
    code: gpui_kit::Entity<EditorState>,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr(cx, "gallery.atoms.input.placeholder_search"))
        });
        let notes = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(4)
                .placeholder(tr(cx, "gallery.atoms.input.placeholder_notes"))
        });
        let amount = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr(cx, "gallery.atoms.input.placeholder_amount"))
                .default_value("120")
        });
        let otp = cx.new(|cx| OtpState::new(6, window, cx));
        let code = cx.new(|cx| {
            EditorState::new(window, cx).placeholder(tr(cx, "gallery.atoms.input.placeholder_code"))
        });
        Section {
            search,
            notes,
            amount,
            otp,
            code,
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
        section_page(
            cx,
            "gallery.section.atoms-input",
            [
                demo_card(
                    cx,
                    "gallery.atoms.input.text",
                    Input::new(&self.search)
                        .prefix(Icon::new(IconName::Search).small())
                        .cleanable(true)
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.input.textarea",
                    Textarea::new(&self.notes).into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.input.number",
                    NumberInput::new(&self.amount).into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.input.otp",
                    OtpInput::new(&self.otp).into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.input.group",
                    InputGroup::new("demo-group")
                        .input(InputGroupInput::new(&self.search))
                        .addon(InputGroupAddon::new("group-prefix").child(
                            InputGroupText::new().child(tr(cx, "gallery.components.code_unit")),
                        ))
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.input.editor",
                    Editor::new(&self.code).into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
