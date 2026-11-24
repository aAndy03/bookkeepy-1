// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — feedback primitives: Progress, ProgressCircle, Spinner, Skeleton, Shimmer,
//! Tooltip (design sheet: "ATOMS · 03 input, number input, textarea, progress, spinner,
//! skeleton, tooltip, scrollbar"; inputs live in `atoms_input`).

use gpui_kit::component::progress::{Progress, ProgressCircle};
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    AnyView, AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Window, div,
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
        section_page(
            cx,
            "gallery.section.atoms-feedback",
            [demo_row([
                demo_card(
                    cx,
                    "gallery.atoms.feedback.progress",
                    Progress::new("progress-demo").value(75.).into_any_element(),
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
                    Spinner::new().into_any_element(),
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
            ])
            .into_any_element()],
        )
    }
}
