// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — display primitives: Badge, Tag, Avatar (design sheet:
//! "ATOMS · 02 badge, tag, avatar, checkbox, radio, switch, toggle, slider" — the
//! badge/tag/avatar family; Marker and Image live in the ATOMS · 04 section
//! `atoms-media`, Rating in `atoms-media` too, choice controls in `atoms_choice`).

use gpui_kit::Styled as _;
use gpui_kit::component::avatar::{Avatar, AvatarGroup};
use gpui_kit::component::badge::Badge;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _};
use gpui_kit::{AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Window};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The display-atom section.
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
        let name: gpui_kit::SharedString = tr(cx, "gallery.atoms.display.sample_name");
        let mode = cx.global::<crate::app::Prefs>().mode;
        section_page(
            cx,
            "gallery.section.atoms-badge-avatar",
            [
                demo_card(
                    cx,
                    "gallery.atoms.display.badge",
                    // ATOMS · 02 badge: solid-hue pills on the 6 functional hues
                    // (count pills) + dot badges, all on the accent/semantic seats.
                    demo_row([
                        Badge::new()
                            .count(3)
                            .color(cx.theme().ring)
                            .child(Icon::new(IconName::Bell))
                            .into_any_element(),
                        Badge::new()
                            .count(12)
                            .color(cx.theme().success)
                            .child(Icon::new(IconName::CircleCheck))
                            .into_any_element(),
                        Badge::new()
                            .count(7)
                            .color(cx.theme().danger)
                            .child(Icon::new(IconName::TriangleAlert))
                            .into_any_element(),
                        Badge::new()
                            .dot()
                            .color(cx.theme().ring)
                            .child(Icon::new(IconName::Inbox))
                            .into_any_element(),
                        Badge::new()
                            .dot()
                            .color(cx.theme().info)
                            .child(Icon::new(IconName::Inbox))
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.display.tag",
                    demo_row([
                        bookkeepy_ui::api::language::display::tag_soft(
                            Tag::primary(),
                            bookkeepy_ui::api::language::display::TagTone::Accent,
                            mode,
                        )
                        .child(tr(cx, "gallery.atoms.display.tag_primary"))
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::tag_soft(
                            Tag::secondary(),
                            bookkeepy_ui::api::language::display::TagTone::Neutral,
                            mode,
                        )
                        .child(tr(cx, "gallery.atoms.display.tag_secondary"))
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::tag_soft(
                            Tag::success(),
                            bookkeepy_ui::api::language::display::TagTone::Success,
                            mode,
                        )
                        .child(tr(cx, "gallery.atoms.display.tag_success"))
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::tag_soft(
                            Tag::warning(),
                            bookkeepy_ui::api::language::display::TagTone::Warning,
                            mode,
                        )
                        .child(tr(cx, "gallery.atoms.display.tag_warning"))
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::tag_soft(
                            Tag::danger(),
                            bookkeepy_ui::api::language::display::TagTone::Danger,
                            mode,
                        )
                        .child(tr(cx, "gallery.atoms.display.tag_danger"))
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::tag_soft(
                            Tag::info(),
                            bookkeepy_ui::api::language::display::TagTone::Info,
                            mode,
                        )
                        .child(tr(cx, "gallery.atoms.display.tag_info"))
                        .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.display.avatar",
                    // ATOMS · 02 avatar: size scale + borderless singles; the overlap
                    // group keeps the #2a2a2a separator ring.
                    demo_row([
                        bookkeepy_ui::api::language::display::avatar(
                            Avatar::new().name(name.clone()),
                        )
                        .with_size(gpui_kit::component::Size::XSmall)
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::avatar(
                            Avatar::new().name(name.clone()),
                        )
                        .with_size(gpui_kit::component::Size::Small)
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::avatar(
                            Avatar::new().name(name.clone()),
                        )
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::avatar(
                            Avatar::new().placeholder(IconName::User),
                        )
                        .into_any_element(),
                        AvatarGroup::new()
                            .child(
                                bookkeepy_ui::api::language::display::avatar(
                                    Avatar::new().name(name.clone()),
                                )
                                .border_2()
                                .border_color(
                                    bookkeepy_ui::api::language::token_hsla("#2a2a2a")
                                        .unwrap_or(cx.theme().border),
                                ),
                            )
                            .child(
                                bookkeepy_ui::api::language::display::avatar(
                                    Avatar::new().placeholder(IconName::User),
                                )
                                .border_2()
                                .border_color(
                                    bookkeepy_ui::api::language::token_hsla("#2a2a2a")
                                        .unwrap_or(cx.theme().border),
                                ),
                            )
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
