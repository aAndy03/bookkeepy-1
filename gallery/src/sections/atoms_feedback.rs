// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 03 sheet (part 2) — progress, spinner, skeleton, tooltip, scrollbar
//! (references/designlanguage).
//!
//! FLAT on the page background. Progress/spinner/skeleton/scrollbar/tooltip specimens
//! come from `bookkeepy_ui::api::language::feedback2` (or the live stock components);
//! tooltips are live on hover over the trigger buttons, plus painted plates for the
//! sheet's content variants.

use bookkeepy_ui::api::language::{
    display, feedback2, feedback2::ProgressTone, feedback2::ScrollIx, primitives,
};
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::StyledExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::progress::{Progress, ProgressCircle};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::shimmer::ShimmerText;
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{
    AnyView, AppContext as _, Context, ParentElement as _, SharedString, Styled as _, Window, div,
    gpui::InteractiveElement as _, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| AtomsFeedback).into()
}

pub struct AtomsFeedback;

impl gpui_kit::Render for AtomsFeedback {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── progress ─────────────────────────────────────────────────────────
        let bar = |pct: f32, tone: ProgressTone, h: f32, w: f32| {
            feedback2::progress_bar(pct, tone, h, w, mode)
        };
        let progress_determinate = v_flex()
            .gap_3()
            .child(bar(65., ProgressTone::Neutral, 4., 240.))
            .child(bar(62., ProgressTone::Neutral, 6., 160.))
            .child(bar(62., ProgressTone::Neutral, 8., 160.))
            .child(bar(62., ProgressTone::Neutral, 10., 160.));
        let progress_tones = kit::row([
            kit::cell(
                cx,
                "gallery.atoms03.progress.neutral",
                bar(65., ProgressTone::Neutral, 4., 240.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.success",
                bar(100., ProgressTone::Success, 4., 240.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.warning",
                bar(92., ProgressTone::Warning, 4., 240.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.danger",
                bar(40., ProgressTone::Danger, 4., 240.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.accent",
                bar(30., ProgressTone::Accent, 4., 240.),
            ),
        ]);
        let progress_indet = kit::cell(
            cx,
            "gallery.atoms03.progress.indeterminate",
            Progress::new("p-indet")
                .loading(true)
                .with_size(Size::XSmall)
                .w(px(240.)),
        );
        let progress_circle = kit::row([
            kit::cell(
                cx,
                "gallery.atoms03.progress.circle",
                ProgressCircle::new("pc1").value(65.).w(px(32.)).h(px(32.)),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.success",
                ProgressCircle::new("pc2")
                    .value(100.)
                    .color(
                        bookkeepy_ui::api::language::role_hsla(mode, "success").unwrap_or_default(),
                    )
                    .w(px(32.))
                    .h(px(32.)),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.danger",
                ProgressCircle::new("pc3")
                    .value(40.)
                    .color(
                        bookkeepy_ui::api::language::role_hsla(mode, "danger").unwrap_or_default(),
                    )
                    .w(px(32.))
                    .h(px(32.)),
            ),
        ]);
        let progress_label = kit::cell(
            cx,
            "gallery.atoms03.progress.label",
            v_flex()
                .gap_2()
                .child(feedback2::progress_label(
                    tr(cx, "gallery.atoms03.demo.syncing"),
                    156,
                    240,
                    240.,
                    mode,
                ))
                .child(bar(65., ProgressTone::Neutral, 4., 240.)),
        );
        let progress_group = kit::group(
            cx,
            "gallery.atoms03.progress.title",
            "gallery.atoms03.progress.hint",
            v_flex()
                .gap_6()
                .child(progress_determinate)
                .child(progress_tones)
                .child(progress_indet)
                .child(progress_circle)
                .child(progress_label),
        );

        // ── spinner ──────────────────────────────────────────────────────────
        let spinner = |tone: Option<&str>, size: f32| {
            let mut s = Spinner::new().with_size(Size::Size(px(size)));
            if let Some(t) = tone {
                if let Some(c) = bookkeepy_ui::api::language::role_hsla(mode, t) {
                    s = s.color(c);
                }
            } else if let Some(c) = bookkeepy_ui::api::language::role_hsla(mode, "icon-muted") {
                s = s.color(c);
            }
            s
        };
        let spinner_row = kit::row([
            kit::cell(cx, "gallery.atoms03.tone.neutral", spinner(None, 16.)),
            kit::cell(
                cx,
                "gallery.atoms03.tone.accent",
                spinner(Some("accent"), 16.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.success",
                spinner(Some("success"), 16.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.warning",
                spinner(Some("warning"), 16.),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tone.danger",
                spinner(Some("danger"), 16.),
            ),
            kit::cell(cx, "gallery.atoms03.tone.info", spinner(Some("info"), 16.)),
        ]);
        let spinner_sizes = kit::row([12., 16., 20., 24.].into_iter().map(|s| {
            kit::cell(
                cx,
                "gallery.atoms03.spinner.size",
                spinner(Some("accent"), s),
            )
        }));
        let spinner_group = kit::group(
            cx,
            "gallery.atoms03.spinner.title",
            "gallery.atoms03.spinner.hint",
            v_flex().gap_4().child(spinner_row).child(spinner_sizes),
        );

        // ── skeleton + shimmer ───────────────────────────────────────────────
        let sk = |w: f32, h: f32, r: f32, secondary: bool| {
            let mut s = Skeleton::new().w(px(w)).h(px(h)).rounded(px(r));
            if secondary {
                s = s.secondary();
            }
            s
        };
        let skeleton_shapes = kit::row([
            kit::cell(
                cx,
                "gallery.atoms03.skeleton.line",
                sk(200., 10., 2., false),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.skeleton.secondary",
                sk(120., 10., 2., true),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.skeleton.circle",
                sk(32., 32., 999., false),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.skeleton.block",
                sk(120., 64., 16., false),
            ),
        ]);
        let skeleton_layout = v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(sk(32., 32., 999., false))
                    .child(
                        v_flex()
                            .gap_2()
                            .child(sk(140., 10., 2., false))
                            .child(sk(80., 10., 2., true)),
                    ),
            )
            .child(sk(200., 10., 2., false))
            .child(sk(120., 10., 2., true));
        let shimmer = kit::cell(
            cx,
            "gallery.atoms03.skeleton.shimmer",
            ShimmerText::new(tr(cx, "gallery.atoms03.demo.loading_text")),
        );
        let skeleton_group = kit::group(
            cx,
            "gallery.atoms03.skeleton.title",
            "gallery.atoms03.skeleton.hint",
            v_flex()
                .gap_6()
                .child(skeleton_shapes)
                .child(kit::cell(
                    cx,
                    "gallery.atoms03.skeleton.layout",
                    skeleton_layout,
                ))
                .child(shimmer),
        );

        // ── tooltip ──────────────────────────────────────────────────────────
        let tooltip_triggers = kit::row([
            kit::cell(
                cx,
                "gallery.atoms03.tooltip.live",
                primitives::medium(
                    Button::new("tt1")
                        .label(tr(cx, "gallery.atoms03.demo.hover_me"))
                        .tooltip(tr(cx, "gallery.atoms03.demo.tooltip_text")),
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tooltip.shortcut_live",
                primitives::medium(
                    Button::new("tt2")
                        .label(tr(cx, "gallery.atoms03.demo.save"))
                        .tooltip(tr(cx, "gallery.atoms03.demo.save")),
                ),
            ),
        ]);
        let tooltip_plates = kit::row([
            kit::cell(
                cx,
                "gallery.atoms03.tooltip.text",
                feedback2::tooltip_plate(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .child(tr(cx, "gallery.atoms03.demo.tooltip_text")),
                    None,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tooltip.shortcut",
                feedback2::tooltip_plate(
                    h_flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(11.))
                                .line_height(px(15.))
                                .child(tr(cx, "gallery.atoms03.demo.save")),
                        )
                        .child(feedback2::tooltip_kbd("Ctrl S", mode)),
                    None,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms03.tooltip.multi",
                feedback2::tooltip_plate(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(11.))
                                .line_height(px(15.))
                                .child(tr(cx, "gallery.atoms03.demo.tooltip_multi")),
                        )
                        .child(display::tag_soft(
                            Tag::new().child(tr(cx, "gallery.atoms02.cat.fun")),
                            display::TagTone::Accent,
                            mode,
                        )),
                    Some(220.),
                    mode,
                ),
            ),
        ]);
        let tooltip_group = kit::group(
            cx,
            "gallery.atoms03.tooltip.title",
            "gallery.atoms03.tooltip.hint",
            v_flex()
                .gap_4()
                .child(tooltip_triggers)
                .child(tooltip_plates),
        );

        // ── scrollbar ────────────────────────────────────────────────────────
        let scrollbar_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                feedback2::scrollbar(200., 20., 64., ScrollIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                feedback2::scrollbar(200., 20., 64., ScrollIx::Hover),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.pressed",
                feedback2::scrollbar(200., 20., 64., ScrollIx::Pressed),
            ),
        ]);
        let scroll_content = v_flex().children((0..12).map(|i| {
            h_flex()
                .h(px(36.))
                .px(px(12.))
                .items_center()
                .border_b(px(1.))
                .border_color(
                    bookkeepy_ui::api::language::prim_hsla("white-a8").unwrap_or_default(),
                )
                .child(
                    div()
                        .refine_style(&kit::label_refinement())
                        .child(SharedString::from(format!(
                            "{}. {}",
                            i + 1,
                            tr(cx, "gallery.atoms03.demo.row")
                        ))),
                )
        }));
        let scrollbar_live = kit::cell(
            cx,
            "gallery.atoms03.scrollbar.context",
            div()
                .relative()
                .h(px(220.))
                .w(px(420.))
                .rounded(px(12.))
                .bg(bookkeepy_ui::api::language::role_hsla(mode, "surface").unwrap_or_default())
                .child(
                    div()
                        .id("sb-demo")
                        .size_full()
                        .overflow_y_scrollbar()
                        .child(scroll_content),
                ),
        );
        let scrollbar_group = kit::group(
            cx,
            "gallery.atoms03.scrollbar.title",
            "gallery.atoms03.scrollbar.hint",
            v_flex()
                .gap_4()
                .child(scrollbar_states)
                .child(scrollbar_live),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::eyebrow(cx, tr(cx, "gallery.atoms03.sheet")))
            .child(kit::group_title(
                cx,
                tr(cx, "gallery.atoms03.feedback_title"),
            ))
            .child(kit::caption(cx, tr(cx, "gallery.atoms03.subtitle")))
            .child(progress_group)
            .child(spinner_group)
            .child(skeleton_group)
            .child(tooltip_group)
            .child(scrollbar_group)
    }
}
