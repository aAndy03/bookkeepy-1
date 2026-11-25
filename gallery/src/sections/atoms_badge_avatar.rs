// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 02 sheet (part 1) — badge, tag, avatar (references/designlanguage).
//!
//! Rendered FLAT on the page background exactly like the exported design frames: no
//! cards behind elements. Every specimen comes from `bookkeepy_ui::api::language`
//! (display/atoms) — the same presets the app reuses (doc 04 §4.9 / ADR-0017).

use bookkeepy_ui::api::language::{atoms, display, prim_hsla, role_hsla};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::avatar::Avatar;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement, ParentElement as _, SharedString, Styled as _,
    Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| AtomsBadgeAvatar).into()
}

pub struct AtomsBadgeAvatar;

const TONES: [display::TagTone; 6] = [
    display::TagTone::Neutral,
    display::TagTone::Accent,
    display::TagTone::Success,
    display::TagTone::Warning,
    display::TagTone::Danger,
    display::TagTone::Info,
];

const TONE_KEYS: [&str; 6] = [
    "gallery.atoms02.tone.neutral",
    "gallery.atoms02.tone.accent",
    "gallery.atoms02.tone.success",
    "gallery.atoms02.tone.warning",
    "gallery.atoms02.tone.danger",
    "gallery.atoms02.tone.info",
];

/// Category dots ride the data-* series (ATOMS · 02 categories row).
const CATEGORY_DOTS: [&str; 6] = ["data-1", "data-2", "data-3", "data-4", "data-8", "data-6"];

const CATEGORY_KEYS: [&str; 6] = [
    "gallery.atoms02.cat.uncategorized",
    "gallery.atoms02.cat.groceries",
    "gallery.atoms02.cat.housing",
    "gallery.atoms02.cat.transport",
    "gallery.atoms02.cat.fun",
    "gallery.atoms02.cat.income",
];

impl gpui_kit::Render for AtomsBadgeAvatar {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let ring = prim_hsla("neutral-925").unwrap_or_default();

        // ── badge ────────────────────────────────────────────────────────────
        let badge_masters = kit::row([
            kit::cell(
                cx,
                "gallery.atoms02.badge.count",
                display::badge_count_el(3, display::BadgeTone::Neutral),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.badge.dot",
                display::badge_dot(display::BadgeTone::Neutral, ring),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.badge.icon",
                display::badge_icon(IconName::Bell, display::BadgeTone::Danger),
            ),
        ]);
        let badge_tone_of = |tone: &display::TagTone| match tone {
            display::TagTone::Neutral => display::BadgeTone::Neutral,
            display::TagTone::Accent => display::BadgeTone::Accent,
            display::TagTone::Success => display::BadgeTone::Success,
            display::TagTone::Warning => display::BadgeTone::Warning,
            display::TagTone::Danger => display::BadgeTone::Danger,
            display::TagTone::Info => display::BadgeTone::Info,
        };
        let badge_tones = v_flex().gap_4().children(
            [
                ("gallery.atoms02.badge.count", 0usize),
                ("gallery.atoms02.badge.dot", 1),
                ("gallery.atoms02.badge.icon", 2),
            ]
            .into_iter()
            .map(|(kind_key, kind)| {
                v_flex()
                    .gap_2()
                    .child(kit::overline(cx, tr(cx, kind_key)))
                    .child(kit::row(TONES.iter().enumerate().map(|(i, tone)| {
                        let bt = badge_tone_of(tone);
                        let el: gpui_kit::AnyElement = match kind {
                            0 => display::badge_count_el(3, bt).into_any_element(),
                            1 => display::badge_dot(bt, ring).into_any_element(),
                            _ => display::badge_icon(IconName::Bell, bt).into_any_element(),
                        };
                        kit::cell(cx, TONE_KEYS[i], el)
                    })))
            }),
        );
        // anchored: count pill on a bell icon, dot on an avatar
        let bell = atoms::icon(IconName::Bell, 20.);
        let anchored = kit::row([
            kit::cell(
                cx,
                "gallery.atoms02.badge.anchor_count",
                display::badge_anchored(
                    bell,
                    display::badge_count_el(3, display::BadgeTone::Accent),
                    true,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.badge.anchor_dot",
                display::badge_anchored(
                    display::avatar_initials("AM", 32),
                    display::badge_dot(display::BadgeTone::Danger, ring),
                    false,
                ),
            ),
        ]);
        let badge = v_flex()
            .gap_4()
            .child(badge_masters)
            .child(badge_tones)
            .child(anchored);
        let badge_group = kit::group(
            cx,
            "gallery.atoms02.badge.title",
            "gallery.atoms02.badge.hint",
            badge,
        );

        // ── tag ──────────────────────────────────────────────────────────────
        let tag_subtle = kit::row(TONES.iter().enumerate().map(|(i, tone)| {
            kit::cell(
                cx,
                TONE_KEYS[i],
                display::tag_soft(Tag::new().child(tr(cx, TONE_KEYS[i])), *tone, mode),
            )
        }));
        let tag_outline = kit::row(TONES.iter().enumerate().map(|(i, tone)| {
            kit::cell(
                cx,
                TONE_KEYS[i],
                display::tag_outline(Tag::new().child(tr(cx, TONE_KEYS[i])), *tone, mode),
            )
        }));
        let tag_categories = kit::row(CATEGORY_DOTS.iter().enumerate().map(|(i, dot)| {
            let dot_color = role_hsla(mode, dot).unwrap_or_default();
            let tag = display::tag_soft(
                Tag::new().child(
                    h_flex()
                        .gap(px(4.))
                        .child(div().size(px(6.)).rounded_full().bg(dot_color)),
                ),
                display::TagTone::Neutral,
                mode,
            )
            .child(tr(cx, CATEGORY_KEYS[i]));
            kit::cell(cx, CATEGORY_KEYS[i], tag)
        }));
        // sizes & shapes: 20 / 24 / accent full pill / removable filter (tag + xs x)
        let tag_sizes = kit::row([
            kit::cell(
                cx,
                "gallery.atoms02.tag.size20",
                display::tag_soft(
                    Tag::new().child(tr(cx, "gallery.atoms02.cat.uncategorized")),
                    display::TagTone::Neutral,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.tag.size24",
                display::tag_tall(display::tag_soft(
                    Tag::new().child(tr(cx, "gallery.atoms02.cat.uncategorized")),
                    display::TagTone::Neutral,
                    mode,
                )),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.tag.full",
                display::tag_soft(
                    Tag::new()
                        .rounded_full()
                        .child(tr(cx, "gallery.atoms02.cat.fun")),
                    display::TagTone::Accent,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.tag.removable",
                display::tag_tall(display::tag_soft(
                    Tag::new().rounded_full().child(
                        h_flex()
                            .gap(px(4.))
                            .child(tr(cx, "gallery.atoms02.cat.fun"))
                            .child(atoms::icon(IconName::X, 12.)),
                    ),
                    display::TagTone::Accent,
                    mode,
                ))
                .border_1()
                .border_color(
                    prim_hsla("orange-500")
                        .map(|c| c.opacity(0.4))
                        .unwrap_or_default(),
                )
                .pl(px(8.))
                .pr(px(4.)),
            ),
        ]);
        let tag = v_flex()
            .gap_6()
            .child(tag_subtle)
            .child(tag_outline)
            .child(tag_categories)
            .child(tag_sizes);
        let tag_group = kit::group(
            cx,
            "gallery.atoms02.tag.title",
            "gallery.atoms02.tag.hint",
            tag,
        );

        // ── avatar ───────────────────────────────────────────────────────────
        let sizes_row = kit::row([16u32, 20, 24, 32, 48].map(|s| {
            kit::cell(
                cx,
                "gallery.atoms02.avatar.size",
                h_flex()
                    .items_end()
                    .gap_2()
                    .child(display::avatar_initials("AM", s))
                    .child(kit::overline(cx, SharedString::from(s.to_string()))),
            )
        }));
        let hues_row = kit::row(["AM", "TE", "IO", "RV", "MB"].map(|n| {
            kit::cell(
                cx,
                "gallery.atoms02.avatar.initials",
                display::avatar_initials(n, 32),
            )
        }));
        let image_src = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/avatar-demo.png"
        ));
        let kinds_row = kit::row([
            kit::cell(
                cx,
                "gallery.atoms02.avatar.image",
                display::avatar(Avatar::new().src(image_src).with_size(Size::Size(px(32.)))),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.avatar.placeholder",
                display::avatar_placeholder(32, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.avatar.loading",
                div()
                    .size(px(32.))
                    .rounded_full()
                    .bg(role_hsla(mode, "skeleton").unwrap_or_default()),
            ),
            kit::cell(
                cx,
                "gallery.atoms02.avatar.current",
                display::avatar_current(display::avatar_initials("AM", 32)),
            ),
        ]);
        // group: −6 overlap, 2px surface ring, +N disc
        let stack = h_flex().items_center().children(
            ["AM", "TE", "IO"]
                .into_iter()
                .enumerate()
                .map(|(i, n)| {
                    display::avatar_stack_item(display::avatar_initials(n, 24), ring, i == 0)
                })
                .chain(std::iter::once(display::avatar_stack_item(
                    display::avatar_overflow(2, 24, mode),
                    ring,
                    false,
                ))),
        );
        let group_row = kit::cell(cx, "gallery.atoms02.avatar.group", stack);
        // in context: profile switcher (the sheet draws its own plate here — it IS part
        // of the component's context frame)
        let switcher = div()
            .bg(prim_hsla("neutral-925").unwrap_or_default())
            .rounded(px(12.))
            .p(px(12.))
            .child(
                h_flex()
                    .items_center()
                    .gap(px(8.))
                    .child(display::avatar_initials("AM", 32))
                    .child(
                        v_flex()
                            .child(kit::overline(cx, tr(cx, "gallery.atoms02.demo.profile")))
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(kit::caption(cx, tr(cx, "gallery.atoms02.demo.andy")))
                                    .child(atoms::icon(IconName::ChevronsUpDown, 14.)),
                            ),
                    ),
            );
        let avatar = v_flex()
            .gap_6()
            .child(sizes_row)
            .child(hues_row)
            .child(kinds_row)
            .child(group_row)
            .child(kit::cell(cx, "gallery.atoms02.avatar.context", switcher));
        let avatar_group = kit::group(
            cx,
            "gallery.atoms02.avatar.title",
            "gallery.atoms02.avatar.hint",
            avatar,
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.atoms02.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.atoms02.title")))
            .child(kit::caption(cx, tr(cx, "gallery.atoms02.subtitle")))
            .child(badge_group)
            .child(tag_group)
            .child(avatar_group)
    }
}
