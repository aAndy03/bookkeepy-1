// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — ATOMS · 04 sheet: OtpInput, Image, Rating, Marker (moved here from
//! atoms-input / atoms-badge-avatar / atoms-choice so the section mirrors the design
//! sheet one-to-one).

use gpui_kit::component::input::{OtpInput, OtpState};
use gpui_kit::component::marker::{Marker, MarkerContent, MarkerIcon};
use gpui_kit::component::rating::Rating;
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _, Size};
use gpui_kit::{
    AnyView, AppContext as _, Context, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, Styled as _, StyledImage as _, Window, div, img, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The ATOMS · 04 section.
pub struct Section {
    otp: gpui_kit::Entity<OtpState>,
    rating: usize,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let otp = cx.new(|cx| OtpState::new(6, window, cx));
        Section { otp, rating: 3 }
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
            "gallery.section.atoms-media",
            [
                demo_row([
                    demo_card(
                        cx,
                        "gallery.atoms.input.otp",
                        // ATOMS · 04: 36x40 rx6 cells on the #0d0d0d well, pitch 46.
                        OtpInput::new(&self.otp)
                            .with_size(Size::Size(px(40.)))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.atoms.choice.rating",
                        Rating::new("rating")
                            .value(self.rating)
                            .max(5)
                            .on_click(cx.listener(|this, value, _, cx| {
                                this.rating = *value;
                                cx.notify();
                            }))
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.atoms.display.image",
                        // ATOMS · 04 image: 160x120 rx16 #1c1c1c frame; asset-backed
                        // image (a shipped icon; remote URLs stay offline-free, doc 08).
                        div()
                            .w(px(160.))
                            .h(px(120.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(bookkeepy_primary_data::theme::radii().overlay as f32))
                            .bg(cx.theme().popover)
                            .child(
                                img("icons/inbox.svg")
                                    .id("image-demo")
                                    .size_16()
                                    .object_fit(ObjectFit::Contain),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.atoms.display.marker",
                        demo_row([
                            // Status-row marker (ORGANISMS row dot + label).
                            Marker::new()
                                .icon(MarkerIcon::new().child(Icon::new(IconName::CircleCheck)))
                                .content(
                                    MarkerContent::new()
                                        .text(tr(cx, "gallery.atoms.display.marker_online")),
                                )
                                .into_any_element(),
                            // ATOMS · 04 map marker: accent pin with a 40% halo.
                            div()
                                .size(px(24.))
                                .rounded_full()
                                .bg(cx.theme().ring.opacity(0.4))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .size(px(10.))
                                        .rounded_full()
                                        .bg(cx.theme().ring)
                                        .border_2()
                                        .border_color(cx.theme().background),
                                )
                                .into_any_element(),
                        ])
                        .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
            ],
        )
    }
}
