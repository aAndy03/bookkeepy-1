// SPDX-License-Identifier: GPL-3.0-or-later
//! Atoms — display primitives: Badge, Tag, Avatar, Marker, Image (design sheet:
//! "ATOMS · 02 badge, tag, avatar, checkbox, radio, switch, toggle, slider" — the badge/
//! tag/avatar family; the choice controls live in `atoms_choice`).

use gpui_kit::component::avatar::{Avatar, AvatarGroup};
use gpui_kit::component::badge::Badge;
use gpui_kit::component::marker::{Marker, MarkerContent, MarkerIcon};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme as _, Icon, IconName};
use gpui_kit::{
    AnyView, AppContext as _, Context, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, Styled as _, StyledImage as _, Window, img,
};

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
                    demo_row([
                        // Count badge on the accent (ORGANISMS · 01 sidebar spec:
                        // 22x16 rx8 #f5823a pill).
                        Badge::new()
                            .count(3)
                            .color(cx.theme().ring)
                            .child(Icon::new(IconName::Bell))
                            .into_any_element(),
                        Badge::new()
                            .dot()
                            .color(cx.theme().ring)
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
                    demo_row([
                        bookkeepy_ui::api::language::display::avatar(
                            Avatar::new().name(name.clone()),
                        )
                        .into_any_element(),
                        bookkeepy_ui::api::language::display::avatar(
                            Avatar::new().placeholder(IconName::User),
                        )
                        .into_any_element(),
                        AvatarGroup::new()
                            .child(bookkeepy_ui::api::language::display::avatar(
                                Avatar::new().name(name.clone()),
                            ))
                            .child(bookkeepy_ui::api::language::display::avatar(
                                Avatar::new().placeholder(IconName::User),
                            ))
                            .into_any_element(),
                    ])
                    .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.display.marker",
                    Marker::new()
                        .icon(MarkerIcon::new().child(Icon::new(IconName::CircleCheck)))
                        .content(
                            MarkerContent::new()
                                .text(tr(cx, "gallery.atoms.display.marker_online")),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.atoms.display.image",
                    // Asset-backed image (a shipped icon; remote URLs stay offline-free —
                    // the app never loads remote content, doc 08).
                    img("icons/inbox.svg")
                        .id("image-demo")
                        .size_16()
                        .object_fit(ObjectFit::Contain)
                        .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
