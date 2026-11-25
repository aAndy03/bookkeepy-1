// SPDX-License-Identifier: GPL-3.0-or-later
//! Molecules — navigation & conversation surfaces: TextView, Stepper, Carousel, Bubble, Message,
//! MessageScroller (design sheets: ORGANISMS · 01 sidebar/nav, MOLECULES conversation).
//! "MOLECULES · label, icon, kbd, stepper, carousel, chat bubbles, messages").

use std::rc::Rc;

use gpui_kit::component::avatar::Avatar;
use gpui_kit::component::bubble::{Bubble, BubbleGroup, BubbleVariant};
use gpui_kit::component::carousel::{
    Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, CarouselState,
};
use gpui_kit::component::message::{
    Message, MessageAlignment, MessageContent, MessageFooter, MessageGroup, MessageHeader,
};
use gpui_kit::component::message_scroller::{MessageScroller, MessageScrollerState};
use gpui_kit::component::stepper::{Stepper, StepperItem};
use gpui_kit::component::text::TextView;
use gpui_kit::component::{ActiveTheme as _, IconName, Sizable as _};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, Styled as _, Window, div,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// One localized conversation row (text is translated at render time, so the
/// demo stays correct after a locale switch).
#[derive(Clone)]
struct ChatMsg {
    incoming: bool,
    sender_key: &'static str,
    text_key: &'static str,
    footer_key: Option<&'static str>,
}

/// A composed chat row: sender avatar + header + bubble (+ footer).
fn chat_message(msg: &ChatMsg, cx: &App) -> Message {
    let sender = tr(cx, msg.sender_key);
    let bubble = if msg.incoming {
        Bubble::new().with_variant(BubbleVariant::Secondary)
    } else {
        Bubble::new()
    };
    let message = Message::new()
        .alignment(if msg.incoming {
            MessageAlignment::Start
        } else {
            MessageAlignment::End
        })
        .avatar(Avatar::new().name(sender.clone()).size_8())
        .header(MessageHeader::new().child(sender))
        .content(MessageContent::new().bubble(bubble.child(tr(cx, msg.text_key))));
    match msg.footer_key {
        Some(key) => message.footer(MessageFooter::new().child(tr(cx, key))),
        None => message,
    }
}

/// A themed carousel slide surface.
fn slide(cx: &App, key: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h_24()
        .w_full()
        .rounded_2xl()
        .bg(cx.theme().secondary)
        .text_color(cx.theme().secondary_foreground)
        .child(tr(cx, key))
}

/// The nav/conversation-molecule section.
pub struct Section {
    step: usize,
    chat: Vec<ChatMsg>,
    chat_scroller: gpui_kit::Entity<MessageScrollerState>,
    carousel: gpui_kit::Entity<CarouselState>,
}

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        const CLIENT: &str = "gallery.molecules.nav.chat.client";
        const YOU: &str = "gallery.molecules.nav.chat.you";
        let chat = vec![
            ChatMsg {
                incoming: true,
                sender_key: CLIENT,
                text_key: "gallery.molecules.nav.chat.msg1",
                footer_key: None,
            },
            ChatMsg {
                incoming: false,
                sender_key: YOU,
                text_key: "gallery.molecules.nav.chat.msg2",
                footer_key: None,
            },
            ChatMsg {
                incoming: true,
                sender_key: CLIENT,
                text_key: "gallery.molecules.nav.chat.msg3",
                footer_key: None,
            },
            ChatMsg {
                incoming: false,
                sender_key: YOU,
                text_key: "gallery.molecules.nav.chat.msg4",
                footer_key: None,
            },
            ChatMsg {
                incoming: true,
                sender_key: CLIENT,
                text_key: "gallery.molecules.nav.chat.msg5",
                footer_key: None,
            },
            ChatMsg {
                incoming: false,
                sender_key: YOU,
                text_key: "gallery.molecules.nav.chat.msg6",
                footer_key: Some("gallery.molecules.nav.chat.delivered"),
            },
        ];
        let chat_scroller = cx.new(|cx| MessageScrollerState::new(chat.len(), cx));
        cx.observe(&chat_scroller, |_, _, cx| cx.notify()).detach();
        let carousel = cx.new(|_| CarouselState::new(3));
        Section {
            step: 1,
            chat,
            chat_scroller,
            carousel,
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
        let rows = Rc::new(self.chat.clone());
        section_page(
            cx,
            "gallery.section.molecules-nav",
            [
                // Static text view primitive (Label, Icon, Kbd live in the ATOMS · 01
                // section `atoms-button`, mirroring the design sheet).
                demo_row([demo_card(
                    cx,
                    "gallery.molecules.nav.text_view",
                    TextView::markdown(
                        "nav-text-view",
                        tr(cx, "gallery.molecules.nav.text_view.sample"),
                    )
                    .into_any_element(),
                )
                .into_any_element()])
                .into_any_element(),
                // Navigation: step flow + carousel.
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.nav.stepper",
                        Stepper::new("nav-stepper")
                            .selected_index(self.step)
                            .w_full()
                            .items([
                                StepperItem::new()
                                    .icon(IconName::User)
                                    .child(tr(cx, "gallery.molecules.nav.stepper.client")),
                                StepperItem::new()
                                    .icon(IconName::FileText)
                                    .child(tr(cx, "gallery.molecules.nav.stepper.invoice")),
                                StepperItem::new()
                                    .icon(IconName::CircleCheck)
                                    .child(tr(cx, "gallery.molecules.nav.stepper.done")),
                            ])
                            .on_click(cx.listener(|this: &mut Self, step: &usize, _window, cx| {
                                this.step = *step;
                                cx.notify();
                            }))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.nav.carousel",
                        Carousel::new("nav-carousel", &self.carousel)
                            .w_full()
                            .max_w_96()
                            .child(
                                CarouselContent::new(&self.carousel)
                                    .child(
                                        CarouselItem::new(
                                            "nav-carousel-slide-1",
                                            0,
                                            &self.carousel,
                                        )
                                        .child(slide(cx, "gallery.molecules.nav.carousel.slide1")),
                                    )
                                    .child(
                                        CarouselItem::new(
                                            "nav-carousel-slide-2",
                                            1,
                                            &self.carousel,
                                        )
                                        .child(slide(cx, "gallery.molecules.nav.carousel.slide2")),
                                    )
                                    .child(
                                        CarouselItem::new(
                                            "nav-carousel-slide-3",
                                            2,
                                            &self.carousel,
                                        )
                                        .child(slide(cx, "gallery.molecules.nav.carousel.slide3")),
                                    ),
                            )
                            .child(CarouselPrevious::new(&self.carousel).small())
                            .child(CarouselNext::new(&self.carousel).small())
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                // Conversation surfaces.
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.nav.bubble",
                        BubbleGroup::new()
                            .child(
                                Bubble::new()
                                    .alignment(MessageAlignment::Start)
                                    .with_variant(BubbleVariant::Secondary)
                                    .child(tr(cx, "gallery.molecules.nav.bubble.incoming")),
                            )
                            .child(
                                Bubble::new()
                                    .alignment(MessageAlignment::End)
                                    .with_variant(BubbleVariant::Tinted)
                                    .child(tr(cx, "gallery.molecules.nav.bubble.outgoing")),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.nav.message",
                        MessageGroup::new()
                            .child(chat_message(&self.chat[0], cx))
                            .child(chat_message(&self.chat[1], cx))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.nav.message_scroller",
                        MessageScroller::new(
                            "nav-chat-scroller",
                            self.chat_scroller.clone(),
                            move |index, _window, cx| {
                                let Some(msg) = rows.get(index) else {
                                    return div().into_any_element();
                                };
                                div()
                                    .id(("nav-chat-row", index))
                                    .min_w_0()
                                    .child(chat_message(msg, cx))
                                    .into_any_element()
                            },
                        )
                        .w(gpui_kit::px(380.))
                        .h_80()
                        .with_jump_button_label(tr(cx, "gallery.molecules.nav.chat.jump"))
                        .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
            ],
        )
    }
}
