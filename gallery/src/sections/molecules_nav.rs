// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 02 (nav) sheet — text view, carousel, bubble/message (references/designlanguage).
//!
//! FLAT on the page background. Live cells: `TextView::markdown`, `Carousel` with
//! prev/next + pagination, and `Message`/`Bubble` incoming vs outgoing.

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::bubble::{Bubble, BubbleVariant};
use gpui_kit::component::carousel::{
    Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPagination, CarouselPrevious,
    CarouselState,
};
use gpui_kit::component::message::{Message, MessageAlignment, MessageContent, MessageHeader};
use gpui_kit::component::text::TextView;
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, ParentElement as _, SharedString, Styled as _,
    Window, div, px,
};

use crate::app::{Gallery, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let carousel = cx.new(|_| CarouselState::new(3));
    cx.new(|_cx| MoleculesNav { carousel }).into()
}

pub struct MoleculesNav {
    carousel: Entity<CarouselState>,
}

impl gpui_kit::Render for MoleculesNav {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let md = "## October summary\n\n**1.240** transactions imported from *BT Pay*.\n\n- Groceries 378,00 RON\n- Transport 96,20 RON\n\nSee [the full report](#).";

        // ── O21 · text view ─────────────────────────────────────────────────
        let tv_group = kit::group(
            cx,
            "gallery.o02.tv.title",
            "gallery.o02.tv.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.o02.tv.live",
                div()
                    .w(px(520.))
                    .child(TextView::markdown("o2-tv", md).scrollable(false)),
            )),
        );

        // ── O24 · carousel ──────────────────────────────────────────────────
        let card = |ix: usize, label: &'static str, value: &'static str| {
            v_flex()
                .w(px(264.))
                .h(px(150.))
                .p(px(16.))
                .gap_2()
                .rounded(px(16.))
                .bg(bookkeepy_ui::api::language::prim_hsla("neutral-900").unwrap_or_default())
                .border_1()
                .border_color(
                    bookkeepy_ui::api::language::prim_hsla("white-a4").unwrap_or_default(),
                )
                .child(kit::overline(cx, label))
                .child(
                    div()
                        .text_color(
                            bookkeepy_ui::api::language::role_hsla(
                                bookkeepy_primary_data::theme::Mode::Dark,
                                "foreground",
                            )
                            .unwrap_or_default(),
                        )
                        .text_size(px(28.))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(value),
                )
                .child(kit::caption(cx, SharedString::from(format!("slide {ix}"))))
        };
        let carousel_group = kit::group(
            cx,
            "gallery.o02.car.title",
            "gallery.o02.car.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.o02.car.live",
                v_flex()
                    .gap_3()
                    .child(
                        Carousel::new("o2-carousel", &self.carousel).child(
                            CarouselContent::new(&self.carousel)
                                .child(
                                    CarouselItem::new("o2-c0", 0, &self.carousel)
                                        .child(card(0, "Spent", "1.812,40")),
                                )
                                .child(
                                    CarouselItem::new("o2-c1", 1, &self.carousel)
                                        .child(card(1, "Income", "8.400,00")),
                                )
                                .child(
                                    CarouselItem::new("o2-c2", 2, &self.carousel)
                                        .child(card(2, "Saved", "6.587,60")),
                                ),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(CarouselPrevious::new(&self.carousel))
                            .child(CarouselNext::new(&self.carousel))
                            .child(CarouselPagination::new()),
                    ),
            )),
        );

        // ── O25 · bubble / message ──────────────────────────────────────────
        let msg_group = kit::group(
            cx,
            "gallery.o02.msg.title",
            "gallery.o02.msg.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.o02.msg.live",
                v_flex()
                    .w(px(480.))
                    .gap_4()
                    .child(
                        Message::new()
                            .alignment(MessageAlignment::Start)
                            .header(MessageHeader::new().child("Andy").child("14:32"))
                            .content(
                                MessageContent::new().bubble(
                                    Bubble::new()
                                        .with_variant(BubbleVariant::Secondary)
                                        .child(tr(cx, "gallery.o02.msg.in")),
                                ),
                            ),
                    )
                    .child(
                        Message::new()
                            .alignment(MessageAlignment::End)
                            .header(MessageHeader::new().child("You").child("14:33"))
                            .content(
                                MessageContent::new()
                                    .bubble(Bubble::new().child(tr(cx, "gallery.o02.msg.out"))),
                            ),
                    ),
            )),
        );

        v_flex()
            .size_full()
            .p(px(48.))
            .gap_12()
            .child(tv_group)
            .child(carousel_group)
            .child(msg_group)
    }
}
