// SPDX-License-Identifier: GPL-3.0-or-later
//! MOLECULES · 03 sheet — calendar, time field, color picker, attachment
//! (references/designlanguage).
//!
//! FLAT on the page background like the design frames. Rest cells are LIVE
//! stock components (`Calendar`, `TimeField`, `ColorPicker`/`ColorSelect`,
//! `Attachment`); the hover/focus/disabled/open and lifecycle ladders are
//! painted from `language::temporal` — same tokens, same geometry.

use bookkeepy_ui::api::language::pickers::{self, DayIx};
use bookkeepy_ui::api::language::temporal::{self, AttIx, CsIx, DzIx, MonthIx, SwIx, TfIx};
use bookkeepy_ui::api::language::{prim_hsla, role_hsla};
use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::StyledExt as _;
use gpui_kit::component::attachment::{
    Attachment, AttachmentContent, AttachmentDescription, AttachmentMedia, AttachmentStatus,
    AttachmentTitle,
};
use gpui_kit::component::calendar::{Calendar, CalendarEvent, CalendarState, Date};
use gpui_kit::component::color_picker::{
    ColorPicker, ColorPickerEvent, ColorPickerState, ColorSelect,
};
use gpui_kit::component::time_field::{
    HourCycle, TimeField, TimeFieldEvent, TimeFieldState, TimePrecision,
};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, ParentElement as _, Styled as _, Subscription,
    Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    let mode = cx.global::<Prefs>().mode;

    let cal_state = cx.new(|cx| CalendarState::new(window, cx));
    cal_state.update(cx, |s, cx| {
        s.set_date(
            Date::Single(Some(jiff::civil::date(2026, 10, 23))),
            window,
            cx,
        );
    });

    let tf_state = cx.new(|cx| TimeFieldState::new(window, cx).precision(TimePrecision::Minute));
    tf_state.update(cx, |s, cx| {
        s.set_time(jiff::civil::time(14, 32, 0, 0), window, cx);
    });
    let tf12_state = cx.new(|cx| {
        TimeFieldState::new(window, cx)
            .precision(TimePrecision::Minute)
            .hour_cycle(HourCycle::H12)
    });
    tf12_state.update(cx, |s, cx| {
        s.set_time(jiff::civil::time(14, 32, 0, 0), window, cx);
    });

    let teal = role_hsla(mode, "data-1").unwrap_or_default();
    let cp_state = cx.new(|cx| ColorPickerState::new(window, cx).default_value(teal));
    let cs_state = cx.new(|cx| ColorPickerState::new(window, cx).default_value(teal));

    cx.new(|cx| {
        let mut v = MoleculesCalendar {
            cal_state,
            tf_state,
            tf12_state,
            cp_state,
            cs_state,
            _subs: Vec::new(),
        };
        // Re-render this section when a live state changes (pickers are controlled:
        // only user interaction emits).
        v._subs
            .push(cx.subscribe(&v.cal_state, |_, _, _: &CalendarEvent, cx| cx.notify()));
        v._subs
            .push(cx.subscribe(&v.tf_state, |_, _, _: &TimeFieldEvent, cx| cx.notify()));
        v._subs
            .push(cx.subscribe(&v.tf12_state, |_, _, _: &TimeFieldEvent, cx| cx.notify()));
        v._subs
            .push(cx.subscribe(&v.cp_state, |_, _, _: &ColorPickerEvent, cx| cx.notify()));
        v._subs
            .push(cx.subscribe(&v.cs_state, |_, _, _: &ColorPickerEvent, cx| cx.notify()));
        v
    })
    .into()
}

pub struct MoleculesCalendar {
    cal_state: Entity<CalendarState>,
    tf_state: Entity<TimeFieldState>,
    tf12_state: Entity<TimeFieldState>,
    cp_state: Entity<ColorPickerState>,
    cs_state: Entity<ColorPickerState>,
    _subs: Vec<Subscription>,
}

/// Which selection mark a painted October 2026 calendar carries (M20 instances).
enum CalMark {
    Single23,
    Range12_17,
    NoFuture,
    Hover8,
    Focus21,
}

/// Painted October 2026 calendar (Mon-start) in the M20 card frame: head, weekday
/// row, 6 weeks of 32×32 day cells. Today = 19 (accent ring), per the sheet.
fn painted_cal(mode: bookkeepy_primary_data::theme::Mode, mark: CalMark) -> gpui_kit::Div {
    let base: [[(u32, DayIx); 7]; 6] = [
        [
            (28, DayIx::Outside),
            (29, DayIx::Outside),
            (30, DayIx::Outside),
            (1, DayIx::Default),
            (2, DayIx::Default),
            (3, DayIx::Default),
            (4, DayIx::Default),
        ],
        [
            (5, DayIx::Default),
            (6, DayIx::Default),
            (7, DayIx::Default),
            (8, DayIx::Default),
            (9, DayIx::Default),
            (10, DayIx::Default),
            (11, DayIx::Default),
        ],
        [
            (12, DayIx::Default),
            (13, DayIx::Default),
            (14, DayIx::Default),
            (15, DayIx::Default),
            (16, DayIx::Default),
            (17, DayIx::Default),
            (18, DayIx::Default),
        ],
        [
            (19, DayIx::Today),
            (20, DayIx::Default),
            (21, DayIx::Default),
            (22, DayIx::Default),
            (23, DayIx::Default),
            (24, DayIx::Default),
            (25, DayIx::Default),
        ],
        [
            (26, DayIx::Default),
            (27, DayIx::Default),
            (28, DayIx::Default),
            (29, DayIx::Default),
            (30, DayIx::Default),
            (31, DayIx::Default),
            (1, DayIx::Outside),
        ],
        [
            (2, DayIx::Outside),
            (3, DayIx::Outside),
            (4, DayIx::Outside),
            (5, DayIx::Outside),
            (6, DayIx::Outside),
            (7, DayIx::Outside),
            (8, DayIx::Outside),
        ],
    ];
    let weeks = match mark {
        CalMark::Single23 | CalMark::Hover8 | CalMark::Focus21 => base.map(|w| {
            w.map(|(n, ix)| {
                if n == 23 {
                    (n, DayIx::Selected)
                } else {
                    (n, ix)
                }
            })
        }),
        CalMark::Range12_17 => base.map(|w| {
            w.map(|(n, ix)| {
                if n == 12 || n == 17 {
                    (n, DayIx::Selected)
                } else if (13..=16).contains(&n) {
                    (n, DayIx::RangeMiddle)
                } else {
                    (n, ix)
                }
            })
        }),
        CalMark::NoFuture => base.map(|w| {
            w.map(|(n, ix)| {
                if n == 15 {
                    (n, DayIx::Selected)
                } else if (20..=31).contains(&n) {
                    (n, DayIx::Disabled)
                } else {
                    (n, ix)
                }
            })
        }),
    };
    let grid = v_flex().gap(px(2.)).children(weeks.into_iter().map(|w| {
        h_flex()
            .gap(px(2.))
            .children(w.map(|(n, ix)| pickers::day_cell(n, ix, mode)))
    }));
    temporal::cal_card(mode)
        .child(temporal::cal_head("October 2026", mode))
        .child(temporal::cal_weekday_row())
        .child(grid)
}

/// Painted months view ("calendar · months", M20): 3×4 grid of 76×36 cells,
/// Aug = current month, Oct = selected.
fn painted_months(mode: bookkeepy_primary_data::theme::Mode) -> gpui_kit::Div {
    const M: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let cell = |m: &'static str| {
        let ix = match m {
            "Aug" => MonthIx::Current,
            "Oct" => MonthIx::Selected,
            _ => MonthIx::Default,
        };
        temporal::month_cell(m, ix)
    };
    let grid = v_flex().gap(px(4.)).children((0..4).map(|r| {
        h_flex()
            .gap(px(4.))
            .children((0..3).map(|c| cell(M[r * 3 + c])))
    }));
    temporal::cal_card(mode)
        .child(temporal::cal_head("2026", mode))
        .child(grid)
}

/// One live attachment row (stock `Attachment`, refined to the sheet's frame).
fn live_att(
    id: &'static str,
    status: AttachmentStatus,
    progress: f32,
    name: &'static str,
    meta: gpui_kit::SharedString,
    mode: bookkeepy_primary_data::theme::Mode,
) -> Attachment {
    Attachment::new()
        .id(id)
        .status(status)
        .progress(progress)
        .on_remove(|_, _, _| {})
        .media(AttachmentMedia::new().child(IconName::Receipt))
        .content(
            AttachmentContent::new()
                .title(AttachmentTitle::new(name))
                .description(AttachmentDescription::new(meta)),
        )
        .w(px(340.))
        .h(px(52.))
        .bg(gpui_kit::transparent_white())
        .border_color(role_hsla(mode, "border-subtle").unwrap_or_default())
}

impl gpui_kit::Render for MoleculesCalendar {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;

        // ── M20 · calendar ───────────────────────────────────────────────────
        let live_cal = Calendar::new(&self.cal_state)
            .first_day_of_week(jiff::civil::Weekday::Monday)
            .w(px(260.))
            .p(px(12.))
            .bg(role_hsla(mode, "popover").unwrap_or_default())
            .border_color(role_hsla(mode, "border-subtle").unwrap_or_default())
            .rounded(px(bookkeepy_ui::api::language::radius_tier("xl")));
        let cal_row1 = kit::row([
            kit::cell(cx, "gallery.m03.cal.live", live_cal),
            kit::cell(cx, "gallery.m03.cal.months", painted_months(mode)),
        ]);
        let cal_row2 = kit::row([
            kit::cell(
                cx,
                "gallery.m03.cal.single",
                painted_cal(mode, CalMark::Single23),
            ),
            kit::cell(
                cx,
                "gallery.m03.cal.range",
                painted_cal(mode, CalMark::Range12_17),
            ),
            kit::cell(
                cx,
                "gallery.m03.cal.nofuture",
                painted_cal(mode, CalMark::NoFuture),
            ),
            kit::cell(
                cx,
                "gallery.m03.cal.hover",
                painted_cal(mode, CalMark::Hover8),
            ),
            kit::cell(
                cx,
                "gallery.m03.cal.focus",
                painted_cal(mode, CalMark::Focus21),
            ),
        ]);
        let cal_group = kit::group(
            cx,
            "gallery.m03.cal.title",
            "gallery.m03.cal.hint",
            v_flex().gap_6().child(cal_row1).child(cal_row2),
        );

        // ── M21 · time field ─────────────────────────────────────────────────
        let tf_live = v_flex()
            .gap(px(6.))
            .child(
                div()
                    .refine_style(&kit::label_refinement())
                    .line_height(px(14.))
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .child(tr(cx, "gallery.m03.tf.label")),
            )
            .child(TimeField::new(&self.tf_state).w(px(160.)));
        let tf12_live = v_flex()
            .gap(px(6.))
            .child(
                div()
                    .refine_style(&kit::label_refinement())
                    .line_height(px(14.))
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .child(tr(cx, "gallery.m03.tf.label")),
            )
            .child(TimeField::new(&self.tf12_state).w(px(160.)));
        let tf_states = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                temporal::time_field(TfIx::Rest, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                temporal::time_field(TfIx::Hover, mode),
            ),
            kit::cell(
                cx,
                "gallery.m03.tf.focus_hours",
                temporal::time_field(TfIx::FocusHours, mode),
            ),
            kit::cell(
                cx,
                "gallery.m03.tf.focus_minutes",
                temporal::time_field(TfIx::FocusMinutes, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.invalid",
                temporal::time_field(TfIx::Invalid, mode),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                temporal::time_field(TfIx::Disabled, mode),
            ),
            kit::cell(
                cx,
                "gallery.m03.tf.h12",
                temporal::time_field(TfIx::H12, mode),
            ),
        ]);
        let tf_group = kit::group(
            cx,
            "gallery.m03.tf.title",
            "gallery.m03.tf.hint",
            v_flex()
                .gap_6()
                .child(kit::row([
                    kit::cell(cx, "gallery.m03.tf.live", tf_live),
                    kit::cell(cx, "gallery.m03.tf.live12", tf12_live),
                ]))
                .child(tf_states),
        );

        // ── M22 · color picker ───────────────────────────────────────────────
        let data_colors: Vec<_> = (1..=8)
            .map(|i| role_hsla(mode, &format!("data-{i}")).unwrap_or_default())
            .collect();
        let cp_live = ColorPicker::new(&self.cp_state).featured_colors(data_colors.clone());
        let cs_live = ColorSelect::new(&self.cs_state)
            .featured_colors(data_colors)
            .placeholder(tr(cx, "gallery.m03.cp.teal"))
            .w(px(200.));
        let swatches = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                temporal::swatch(prim_hsla("cyan-500").unwrap_or_default(), SwIx::Rest),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.hover",
                temporal::swatch(prim_hsla("cyan-500").unwrap_or_default(), SwIx::Hover),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                temporal::swatch(prim_hsla("cyan-500").unwrap_or_default(), SwIx::Focus),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.selected",
                temporal::swatch(prim_hsla("cyan-500").unwrap_or_default(), SwIx::Selected),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.disabled",
                temporal::swatch(prim_hsla("cyan-500").unwrap_or_default(), SwIx::Disabled),
            ),
        ]);
        let cp_card = temporal::cp_card(mode)
            .child(
                div()
                    .text_size(px(10.))
                    .line_height(px(12.))
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .text_color(role_hsla(mode, "muted-foreground").unwrap_or_default())
                    .child(tr(cx, "gallery.m03.cp.card_title")),
            )
            .child(temporal::cp_featured(mode))
            .child(temporal::cp_custom("#3FB8D4", mode));
        let cp_group = kit::group(
            cx,
            "gallery.m03.cp.title",
            "gallery.m03.cp.hint",
            v_flex()
                .gap_6()
                .child(kit::row([
                    kit::cell(cx, "gallery.m03.cp.live", cp_live),
                    kit::cell(cx, "gallery.m03.cp.live_select", cs_live),
                ]))
                .child(swatches)
                .child(kit::row([
                    kit::cell(cx, "gallery.m03.cp.card", cp_card),
                    kit::cell(
                        cx,
                        "gallery.atoms.state.rest",
                        temporal::color_select(tr(cx, "gallery.m03.cp.teal"), CsIx::Rest, mode),
                    ),
                    kit::cell(
                        cx,
                        "gallery.atoms.state.open",
                        temporal::color_select(tr(cx, "gallery.m03.cp.teal"), CsIx::Open, mode),
                    ),
                ])),
        );

        // ── M23 · attachment ─────────────────────────────────────────────────
        let live_rows = v_flex().gap_2().children([
            live_att(
                "m3-att-q",
                AttachmentStatus::Pending,
                0.,
                "receipt-lidl-18-oct.jpg",
                tr(cx, "gallery.m03.att.waiting"),
                mode,
            ),
            live_att(
                "m3-att-u",
                AttachmentStatus::Uploading,
                64.,
                "invoice-enel-oct.pdf",
                tr(cx, "gallery.m03.att.uploading"),
                mode,
            ),
            live_att(
                "m3-att-d",
                AttachmentStatus::Complete,
                0.,
                "receipt-lidl-18-oct.jpg",
                tr(cx, "gallery.m03.att.meta_done"),
                mode,
            ),
            live_att(
                "m3-att-f",
                AttachmentStatus::Failed,
                0.,
                "scan-0042.heic",
                tr(cx, "gallery.m03.att.fail_reason"),
                mode,
            ),
        ]);
        let att_states = kit::row([
            kit::cell(
                cx,
                "gallery.m03.att.queued",
                temporal::att_row(
                    "receipt-lidl-18-oct.jpg".into(),
                    tr(cx, "gallery.m03.att.waiting"),
                    AttIx::Queued,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m03.att.uploading",
                temporal::att_row(
                    "invoice-enel-oct.pdf".into(),
                    tr(cx, "gallery.m03.att.uploading"),
                    AttIx::Uploading,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m03.att.done",
                temporal::att_row(
                    "receipt-lidl-18-oct.jpg".into(),
                    tr(cx, "gallery.m03.att.meta_done"),
                    AttIx::Done,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m03.att.done_hover",
                temporal::att_row(
                    "receipt-lidl-18-oct.jpg".into(),
                    tr(cx, "gallery.m03.att.meta_done"),
                    AttIx::DoneHover,
                    mode,
                ),
            ),
            kit::cell(
                cx,
                "gallery.m03.att.failed",
                temporal::att_row(
                    "scan-0042.heic".into(),
                    tr(cx, "gallery.m03.att.fail_reason"),
                    AttIx::Failed,
                    mode,
                ),
            ),
        ]);
        let dropzones = kit::row([
            kit::cell(
                cx,
                "gallery.atoms.state.rest",
                temporal::dropzone(
                    DzIx::Rest,
                    mode,
                    tr(cx, "gallery.m03.att.dz_title"),
                    tr(cx, "gallery.m03.att.dz_hint"),
                ),
            ),
            kit::cell(
                cx,
                "gallery.m03.att.drag_over",
                temporal::dropzone(
                    DzIx::DragOver,
                    mode,
                    tr(cx, "gallery.m03.att.dz_title"),
                    tr(cx, "gallery.m03.att.dz_hint"),
                ),
            ),
            kit::cell(
                cx,
                "gallery.atoms.state.focus",
                temporal::dropzone(
                    DzIx::Focus,
                    mode,
                    tr(cx, "gallery.m03.att.dz_title"),
                    tr(cx, "gallery.m03.att.dz_hint"),
                ),
            ),
        ]);
        let att_group = kit::group(
            cx,
            "gallery.m03.att.title",
            "gallery.m03.att.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.m03.att.live", live_rows))
                .child(att_states)
                .child(dropzones),
        );

        // ── page ─────────────────────────────────────────────────────────────
        v_flex()
            .size_full()
            .p(px(bookkeepy_ui::api::language::space("space-12") as f32))
            .gap(px(bookkeepy_ui::api::language::space("space-8") as f32))
            .child(kit::eyebrow(cx, tr(cx, "gallery.m03.sheet")))
            .child(kit::group_title(cx, tr(cx, "gallery.m03.title")))
            .child(kit::caption(cx, tr(cx, "gallery.m03.subtitle")))
            .child(cal_group)
            .child(tf_group)
            .child(cp_group)
            .child(att_group)
    }
}
