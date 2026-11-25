// SPDX-License-Identifier: GPL-3.0-or-later
//! SYSTEM sheet — cross-cutting behaviours: root, theme, motion, history, text
//! selection, focus trap, popup. The interactive instances live in their component
//! sections (dialog = focus trap, ours = history); this section shows the system
//! surfaces themselves: theme modes, motion, text selection.

use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _, h_flex};
use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Styled as _, Window, div,
    px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The SYSTEM section.
pub struct Section;

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_| Section).into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let mode_label = match mode {
            bookkeepy_primary_data::theme::Mode::Dark => tr(cx, "gallery.system.theme.dark"),
            bookkeepy_primary_data::theme::Mode::Light => tr(cx, "gallery.system.theme.light"),
        };
        section_page(
            cx,
            "gallery.section.system",
            [
                demo_card(
                    cx,
                    "gallery.system.theme.caption",
                    // SYSTEM · theme: dark is the design language's native theme; the
                    // title-bar button switches modes live (Prefs global).
                    h_flex()
                        .items_center()
                        .gap_3()
                        .child(Icon::new(IconName::Palette).small())
                        .child(div().text_sm().child(mode_label))
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.system.motion.caption",
                    // SYSTEM · motion: loading motion is the quiet #222 ring spinner on
                    // the raised surface (ATOMS · 03 spinner), not a bright glyph.
                    demo_row([
                        Spinner::new().color(cx.theme().skeleton).into_any_element(),
                        Spinner::new()
                            .small()
                            .color(cx.theme().skeleton)
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.system.selection.caption",
                    // SYSTEM · text selection: selection highlight is the accent hover
                    // tint; the block below is selectable text.
                    div()
                        .p_3()
                        .rounded(px(bookkeepy_primary_data::theme::radii().item as f32))
                        .bg(cx.theme().muted)
                        .child(
                            div()
                                .text_sm()
                                .child(tr(cx, "gallery.system.selection.text")),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
