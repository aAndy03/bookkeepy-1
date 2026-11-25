// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — feedback primitives: Progress, ProgressCircle, Spinner, Skeleton, Shimmer,
//! Tooltip (design sheet: "ATOMS · 03 input, number input, textarea, progress, spinner,
//! skeleton, tooltip, scrollbar"; inputs live in `atoms_input`).

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::progress::{Progress, ProgressCircle};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    AnyView, AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The feedback-atom section.
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
        let banner: gpui_kit::SharedString = tr(cx, "gallery.atoms.feedback.tooltip_banner");
        let theme = cx.theme();
        // ATOMS · 03 progress: one row per tone (neutral fill = inverted neutral,
        // then success/warning/danger/accent), plus the height scale.
        let tones = [
            theme.progress_bar,
            theme.success,
            theme.warning,
            theme.danger,
            theme.ring,
        ];
        section_page(
            cx,
            "gallery.section.atoms-feedback",
            [demo_row([
                demo_card(
                    cx,
                    "gallery.atoms.feedback.progress",
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(tones.iter().enumerate().map(|(i, c)| {
                            Progress::new(format!("progress-tone-{i}"))
                                .value(75.)
                                .color(*c)
                                .into_any_element()
                        }))
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.feedback.progress_circle",
                    ProgressCircle::new("progress-circle-demo")
                        .value(55.)
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.feedback.spinner",
                    // ATOMS · 03 spinner: the quiet #222 ring on the raised surface.
                    Spinner::new().color(theme.skeleton).into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.feedback.skeleton",
                    Skeleton::new().into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.feedback.shimmer",
                    gpui_kit::component::shimmer::ShimmerText::new(tr(
                        cx,
                        "gallery.atoms.feedback.shimmer_text",
                    ))
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.feedback.tooltip",
                    div()
                        .id("tooltip-demo")
                        .child(tr(cx, "gallery.atoms.feedback.tooltip_trigger"))
                        .tooltip(move |window, cx| Tooltip::new(banner.clone()).build(window, cx))
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.feedback.scrollbar",
                    // ATOMS · 03 scrollbar: white@4% track, white@40% thumb — rides the
                    // bridge seats; here a real scrollable to show it.
                    div()
                        .h(px(96.))
                        .w(px(200.))
                        .overflow_y_scrollbar()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_2()
                                .children((0..12).map(|i| {
                                    div()
                                        .text_xs()
                                        .child(format!(
                                            "{}. {}",
                                            i + 1,
                                            tr(cx, "gallery.atoms.feedback.scrollbar_row")
                                        ))
                                        .into_any_element()
                                })),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
            ])
            .into_any_element()],
        )
    }
}
