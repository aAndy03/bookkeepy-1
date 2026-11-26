// SPDX-License-Identifier: GPL-3.0-or-later
//! ATOMS · 04 sheet — otp input, image, rating, marker
//! (references/designlanguage).
//!
//! FLAT on the page background like the design frames. Rest cells are LIVE stock
//! components (`OtpInput`, `img()` with loading/fallback, `Rating`, `Marker` — click,
//! type, hover); the sheet's pseudo-states (empty/filling/masked/invalid, hover preview,
//! focus ring) are painted from `bookkeepy_ui::api::language::media` — same tokens.

use bookkeepy_ui::api::language::{atoms, media, media::OtpContent, media::OtpIx, media::RatingIx};
use gpui_kit::ObjectFit;
use gpui_kit::assets::IconName;
use gpui_kit::base::v_flex;
use gpui_kit::component::input::{OtpEvent, OtpInput, OtpState};
use gpui_kit::component::marker::{Marker, MarkerContent, MarkerIcon, MarkerLoadingStyle};
use gpui_kit::component::rating::Rating;
use gpui_kit::component::{Disableable as _, Size};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, StyledImage as _, Subscription, Window, div,
    img, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let otp = cx.new(|ocx| OtpState::new(6, window, ocx).default_value("4481"));
        let otp_masked = cx.new(|ocx| {
            OtpState::new(6, window, ocx)
                .masked(true)
                .default_value("448102")
        });
        let otp_disabled = cx.new(|ocx| OtpState::new(6, window, ocx).default_value("448102"));
        let subs = vec![cx.subscribe(
            &otp,
            |this: &mut AtomsMedia, _state, event: &OtpEvent, cx| {
                this.last_event = match event {
                    OtpEvent::Change => "Change".into(),
                    OtpEvent::Complete => "Complete".into(),
                    OtpEvent::Focus => "Focus".into(),
                    OtpEvent::Blur => "Blur".into(),
                };
                cx.notify();
            },
        )];
        AtomsMedia {
            otp,
            otp_masked,
            otp_disabled,
            last_event: SharedString::from("—"),
            rating: 3,
            _subs: subs,
        }
    })
    .into()
}

pub struct AtomsMedia {
    otp: Entity<OtpState>,
    otp_masked: Entity<OtpState>,
    otp_disabled: Entity<OtpState>,
    last_event: SharedString,
    rating: usize,
    _subs: Vec<Subscription>,
}

/// The sheet's OTP cell chrome — 36×40 box in the rx8 2px ring wrapper.
fn cell(
    content: OtpContent,
    ix: OtpIx,
    mode: bookkeepy_primary_data::theme::Mode,
) -> gpui_kit::Div {
    media::otp_cell(content, ix, mode)
}

impl gpui_kit::Render for AtomsMedia {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── otp input ────────────────────────────────────────────────────────
        let digit = |c: char| cell(OtpContent::Digit(c), OtpIx::Rest, mode);
        let otp_masters = kit::row([
            kit::cell(
                cx,
                "gallery.atoms04.otp.live",
                v_flex()
                    .gap_2()
                    .child(OtpInput::new(&self.otp).groups(2))
                    .child(kit::caption(
                        cx,
                        SharedString::from(format!(
                            "{} · {}",
                            tr(cx, "gallery.atoms04.otp.event"),
                            self.last_event
                        )),
                    )),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.otp.live_masked",
                OtpInput::new(&self.otp_masked).groups(2),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                OtpInput::new(&self.otp_disabled).groups(2).disabled(true),
            ),
        ]);
        let empty = || cell(OtpContent::Empty, OtpIx::Rest, mode);
        let otp_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms04.state.empty",
                media::otp_row(
                    [
                        cell(OtpContent::Caret, OtpIx::Active, mode),
                        empty(),
                        empty(),
                    ],
                    [empty(), empty(), empty()],
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.state.filling",
                media::otp_row(
                    [digit('4'), digit('4'), digit('8')],
                    [
                        cell(OtpContent::Caret, OtpIx::Active, mode),
                        empty(),
                        empty(),
                    ],
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.state.complete",
                media::otp_row(
                    [digit('4'), digit('4'), digit('8')],
                    [digit('1'), digit('0'), digit('2')],
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.state.masked",
                media::otp_row(
                    [
                        cell(OtpContent::Masked, OtpIx::Rest, mode),
                        cell(OtpContent::Masked, OtpIx::Rest, mode),
                        cell(OtpContent::Masked, OtpIx::Rest, mode),
                    ],
                    [
                        cell(OtpContent::Masked, OtpIx::Rest, mode),
                        cell(OtpContent::Masked, OtpIx::Rest, mode),
                        cell(OtpContent::Masked, OtpIx::Rest, mode),
                    ],
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.state.invalid",
                v_flex()
                    .gap_2()
                    .child(media::otp_row(
                        [
                            cell(OtpContent::Digit('4'), OtpIx::Invalid, mode),
                            cell(OtpContent::Digit('8'), OtpIx::Invalid, mode),
                            cell(OtpContent::Digit('1'), OtpIx::Invalid, mode),
                        ],
                        [
                            cell(OtpContent::Digit('9'), OtpIx::Invalid, mode),
                            cell(OtpContent::Digit('0'), OtpIx::Invalid, mode),
                            cell(OtpContent::Digit('2'), OtpIx::Invalid, mode),
                        ],
                    ))
                    .child(media::otp_help(
                        tr(cx, "gallery.atoms04.otp.invalid_help"),
                        mode,
                    )),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                div().opacity(0.4).child(media::otp_row(
                    [digit('4'), digit('4'), digit('8')],
                    [digit('1'), digit('0'), digit('2')],
                )),
            ),
        ]);
        let otp_group = kit::group(
            cx,
            "gallery.atoms04.otp.title",
            "gallery.atoms04.otp.hint",
            v_flex().gap_6().child(otp_masters).child(otp_states),
        );

        // ── image ────────────────────────────────────────────────────────────
        let receipt = std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/receipt-demo.png"
        ));
        let cant_load = tr(cx, "gallery.atoms04.image.cant_load");
        let frame_bg = bookkeepy_ui::api::language::role_hsla(mode, "surface-raised");
        let card_r = bookkeepy_ui::api::language::radius_role("card");
        let img_loaded = {
            let mut el = img(receipt)
                .id("img-loaded")
                .w(px(160.))
                .h(px(120.))
                .rounded(px(card_r))
                .object_fit(ObjectFit::Cover)
                .with_loading(move || media::image_loading(160., 120., mode).into_any_element())
                .with_fallback({
                    let cant_load = cant_load.clone();
                    move || {
                        media::image_error(160., 120., cant_load.clone(), mode).into_any_element()
                    }
                });
            if let Some(bg) = frame_bg {
                el = el.bg(bg);
            }
            el
        };
        let img_error = {
            let mut el = img(std::path::PathBuf::from("assets/missing-receipt.png"))
                .id("img-error")
                .w(px(160.))
                .h(px(120.))
                .rounded(px(card_r))
                .object_fit(ObjectFit::Cover)
                .with_fallback(move || {
                    media::image_error(160., 120., cant_load.clone(), mode).into_any_element()
                });
            if let Some(bg) = frame_bg {
                el = el.bg(bg);
            }
            el
        };
        let image_group = kit::group(
            cx,
            "gallery.atoms04.image.title",
            "gallery.atoms04.image.hint",
            kit::row([
                kit::cell(
                    cx,
                    "gallery.atoms04.image.loading",
                    media::image_loading(160., 120., mode),
                ),
                kit::cell(cx, "gallery.atoms04.image.loaded", img_loaded),
                kit::cell(cx, "gallery.atoms04.image.error", img_error),
            ]),
        );

        // ── rating ───────────────────────────────────────────────────────────
        let warning = bookkeepy_ui::api::language::role_hsla(mode, "warning");
        let live_rating = {
            let mut r = Rating::new("rating-live")
                .value(self.rating)
                .max(5)
                .with_size(Size::Size(px(20.)))
                .on_click(cx.listener(|view, value: &usize, _window, cx| {
                    view.rating = *value;
                    cx.notify();
                }));
            if let Some(c) = warning {
                r = r.color(c);
            }
            r
        };
        let rating_masters = kit::row([
            kit::cell(
                cx,
                "gallery.atoms04.state.off",
                media::rating_star(false, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.rating.live",
                v_flex().gap_2().child(live_rating).child(kit::caption(
                    cx,
                    SharedString::from(format!(
                        "{} · {}/5",
                        tr(cx, "gallery.atoms04.rating.value"),
                        self.rating
                    )),
                )),
            ),
        ]);
        let rating_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms04.rating.0",
                media::rating_group(0, 5, RatingIx::Rest, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.rating.3",
                media::rating_group(3, 5, RatingIx::Rest, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.rating.5",
                media::rating_group(5, 5, RatingIx::Rest, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.rating.hover",
                media::rating_group(4, 5, RatingIx::Rest, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                media::rating_group(3, 5, RatingIx::Focus, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                media::rating_group(3, 5, RatingIx::Disabled, mode),
            ),
        ]);
        let rating_group = kit::group(
            cx,
            "gallery.atoms04.rating.title",
            "gallery.atoms04.rating.hint",
            v_flex().gap_6().child(rating_masters).child(rating_states),
        );

        // ── marker ───────────────────────────────────────────────────────────
        let marker_icon = || atoms::role_icon(IconName::CircleCheck, 14., "icon-muted", mode);
        let marker_event = |m: Marker| {
            m.icon(MarkerIcon::new().size(px(14.)).child(marker_icon()))
                .child(div().child(tr(cx, "gallery.atoms04.demo.imported")))
                .child(media::marker_meta(
                    tr(cx, "gallery.atoms04.demo.time"),
                    mode,
                ))
        };
        let marker_masters = kit::row([
            kit::cell(
                cx,
                "gallery.atoms04.marker.plain",
                media::marker_plain(marker_event(Marker::new()), mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.marker.separator",
                media::marker_separator(
                    Marker::new()
                        .content(MarkerContent::new().text(tr(cx, "gallery.atoms04.demo.today"))),
                    mode,
                ),
            ),
        ]);
        let marker_variants = kit::row([
            kit::cell(
                cx,
                "gallery.atoms04.marker.plain",
                media::marker_plain(marker_event(Marker::new()), mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.marker.border",
                media::marker_pill(marker_event(Marker::new()), mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.marker.separator",
                media::marker_separator(
                    Marker::new().content(
                        MarkerContent::new().text(tr(cx, "gallery.atoms04.demo.imported")),
                    ),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.marker.loading",
                media::marker_plain(
                    Marker::new()
                        .loading(true)
                        .with_loading_style(MarkerLoadingStyle::Spinner)
                        .content(
                            MarkerContent::new().text(tr(cx, "gallery.atoms04.demo.importing")),
                        ),
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms04.marker.shimmer",
                media::marker_plain(
                    Marker::new()
                        .loading(true)
                        .with_loading_style(MarkerLoadingStyle::Shimmer)
                        .content(
                            MarkerContent::new().text(tr(cx, "gallery.atoms04.demo.importing")),
                        ),
                    mode,
                ),
            ),
        ]);
        let marker_group = kit::group(
            cx,
            "gallery.atoms04.marker.title",
            "gallery.atoms04.marker.hint",
            v_flex()
                .gap_6()
                .child(marker_masters)
                .child(marker_variants),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::overline(cx, tr(cx, "gallery.atoms04.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.atoms04.title")))
            .child(kit::caption(cx, tr(cx, "gallery.atoms04.subtitle")))
            .child(otp_group)
            .child(image_group)
            .child(rating_group)
            .child(marker_group)
    }
}
