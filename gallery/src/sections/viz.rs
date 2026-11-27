// SPDX-License-Identifier: GPL-3.0-or-later
//! CHARTS sheet — line, area, bar, donut, candlestick, sparkline (references/designlanguage).
//!
//! FLAT on the page background; a card appears only where the card IS the chart's
//! own frame (chart cards are surface #171717 r16 hairline, flat, no shadow).
//! Live cells: AreaChart (P1), PieChart donut (P3), CandlestickChart (P4), sparkline
//! LineChart (P5); grouped in/out bars (P2) and the state demos (P6) are painted from
//! tokens because gpui-kit's BarChart is single-series.

use bookkeepy_ui::api::language::{prim_hsla, role_hsla};
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::chart::{AreaChart, CandlestickChart, LineChart, PieChart};
use gpui_kit::component::empty::{
    Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::{
    AnyView, AppContext as _, Context, ParentElement as _, SharedString, Styled as _, Window, div,
    px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::kit;

/// Section entry point (the `section!` macro's `new` fn).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_cx| Viz).into()
}

pub struct Viz;

/// Chart card chrome: surface r16 hairline white-a4, flat, padding 20 (P1–P4, P6).
fn chart_card(
    _cx: &gpui_kit::App,
    mode: bookkeepy_primary_data::theme::Mode,
    body: gpui_kit::Div,
) -> gpui_kit::Div {
    v_flex()
        .w(px(600.))
        .p(px(20.))
        .gap(px(16.))
        .rounded(px(16.))
        .bg(role_hsla(mode, "surface").unwrap_or_default())
        .border_1()
        .border_color(prim_hsla("white-a4").unwrap_or_default())
        .child(body)
}

/// Card header: eyebrow + big value (+ optional tag).
fn card_header(
    cx: &gpui_kit::App,
    mode: bookkeepy_primary_data::theme::Mode,
    eyebrow: SharedString,
    value: SharedString,
    tag: Option<SharedString>,
) -> gpui_kit::Div {
    let mut row = h_flex().items_end().gap(px(12.)).child(
        v_flex()
            .gap(px(2.))
            .child(kit::overline(cx, eyebrow))
            .child(
                div()
                    .text_color(role_hsla(mode, "foreground").unwrap_or_default())
                    .text_size(px(28.))
                    .line_height(px(32.))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(value),
            ),
    );
    if let Some(t) = tag {
        row = row.child(
            h_flex()
                .h(px(20.))
                .px(px(6.))
                .items_center()
                .rounded(px(4.))
                .bg(role_hsla(mode, "success-subtle").unwrap_or_default())
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .font_weight(gpui_kit::FontWeight::MEDIUM)
                        .text_color(role_hsla(mode, "success-foreground").unwrap_or_default())
                        .child(t),
                ),
        );
    }
    row
}

#[derive(Clone)]
struct Month {
    name: &'static str,
    income: f64,
    forecast: f64,
}

#[derive(Clone)]
struct Bucket {
    name: &'static str,
    amount: f32,
    color: &'static str,
}

#[derive(Clone)]
struct Day {
    date: &'static str,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

impl gpui_kit::Render for Viz {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let mode = cx.global::<Prefs>().mode;
        let fg = role_hsla(mode, "foreground").unwrap_or_default();
        let muted = role_hsla(mode, "muted-foreground").unwrap_or_default();
        let subtle = role_hsla(mode, "subtle-foreground").unwrap_or_default();
        let divider = role_hsla(mode, "divider").unwrap_or_default();
        let data1 = role_hsla(mode, "data-1").unwrap_or_default();
        let data2 = role_hsla(mode, "data-2").unwrap_or_default();
        let forecast = role_hsla(mode, "money-forecast").unwrap_or_default();
        let money_in = role_hsla(mode, "money-in").unwrap_or_default();
        let money_out = role_hsla(mode, "money-out").unwrap_or_default();
        let money_neg = role_hsla(mode, "money-negative").unwrap_or_default();

        // ── P1 · line + area ────────────────────────────────────────────────
        let months = vec![
            Month {
                name: "Nov",
                income: 6400.,
                forecast: 6400.,
            },
            Month {
                name: "Jan",
                income: 7100.,
                forecast: 7100.,
            },
            Month {
                name: "Mar",
                income: 6800.,
                forecast: 6800.,
            },
            Month {
                name: "May",
                income: 7600.,
                forecast: 7600.,
            },
            Month {
                name: "Jul",
                income: 6781.,
                forecast: 6781.,
            },
            Month {
                name: "Sep",
                income: 6781.,
                forecast: 7400.,
            },
            Month {
                name: "Nov",
                income: 6781.,
                forecast: 8100.,
            },
        ];
        let p1_chart = div().w(px(560.)).h(px(204.)).child(
            AreaChart::new(months)
                .x(|d| d.name)
                .y(|d| d.forecast)
                .stroke(forecast)
                .fill(forecast.opacity(0.0))
                .name(tr(cx, "gallery.charts.p1.s_fc"))
                .y(|d| d.income)
                .stroke(data1)
                .fill(data1.opacity(0.12))
                .name(tr(cx, "gallery.charts.p1.s_actual"))
                .y_axis(true)
                .y_tick_count(4)
                .y_tick_format(|v: f64| SharedString::from(format!("{:.0}k", v / 1000.)))
                .tick_margin(2)
                .grid_columns(7)
                .tooltip_value(|_, _ix, v| SharedString::from(format!("{v:.2} RON"))),
        );
        let p1_group = kit::group(
            cx,
            "gallery.charts.p1.title",
            "gallery.charts.p1.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.charts.p1.live",
                chart_card(
                    cx,
                    mode,
                    v_flex()
                        .gap(px(16.))
                        .child(card_header(
                            cx,
                            mode,
                            tr(cx, "gallery.charts.p1.eyebrow"),
                            tr(cx, "gallery.charts.p1.value"),
                            Some(tr(cx, "gallery.charts.p1.tag")),
                        ))
                        .child(p1_chart),
                ),
            )),
        );

        // ── P2 · bar (grouped in/out pairs, painted geometry) ───────────────
        let pairs: [(&str, f32, f32); 7] = [
            ("Nov", 149.6, 129.8),
            ("Dec", 149.6, 140.8),
            ("Jan", 149.6, 156.2),
            ("Feb", 158.4, 123.2),
            ("Mar", 149.6, 136.4),
            ("Apr", 149.6, 92.4),
            ("May", 4.0, 4.0),
        ];
        let bar = |h: f32, color: gpui_kit::Hsla, stub: bool| {
            div().w(px(16.)).h(px(h)).rounded_t(px(2.)).bg(if stub {
                prim_hsla("neutral-750").unwrap_or_default()
            } else {
                color
            })
        };
        let p2_bars = h_flex()
            .h(px(179.5))
            .items_end()
            .gap(px(22.))
            .border_b_1()
            .border_color(divider)
            .px(px(8.))
            .children(pairs.iter().map(|(_m, i, o)| {
                let stub = *i <= 4.0;
                h_flex()
                    .items_end()
                    .gap(px(3.))
                    .child(bar(*i, money_in, stub))
                    .child(bar(*o, money_neg, stub))
            }));
        let p2_labels = h_flex()
            .px(px(8.))
            .gap(px(22.))
            .children(pairs.iter().map(|(m, _, _)| {
                div()
                    .w(px(35.))
                    .text_center()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .text_color(subtle)
                    .child(*m)
            }));
        let legend_key =
            |color: gpui_kit::Hsla| div().w(px(8.)).h(px(8.)).rounded(px(2.)).bg(color);
        let legend_item = |color: gpui_kit::Hsla, label: SharedString| {
            h_flex()
                .items_center()
                .gap(px(6.))
                .child(legend_key(color))
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(muted)
                        .child(label),
                )
        };
        let p2_legend = h_flex().gap(px(16.)).children([
            legend_item(money_in, tr(cx, "gallery.charts.p2.in")),
            legend_item(money_neg, tr(cx, "gallery.charts.p2.out")),
            legend_item(
                prim_hsla("neutral-750").unwrap_or_default(),
                tr(cx, "gallery.charts.p2.none"),
            ),
        ]);
        let p2_group = kit::group(
            cx,
            "gallery.charts.p2.title",
            "gallery.charts.p2.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.charts.p2.master",
                chart_card(
                    cx,
                    mode,
                    v_flex()
                        .gap(px(16.))
                        .child(card_header(
                            cx,
                            mode,
                            tr(cx, "gallery.charts.p2.eyebrow"),
                            tr(cx, "gallery.charts.p2.value"),
                            None,
                        ))
                        .child(p2_bars)
                        .child(p2_labels)
                        .child(p2_legend),
                ),
            )),
        );

        // ── P3 · donut ──────────────────────────────────────────────────────
        let buckets = vec![
            Bucket {
                name: "Rent",
                amount: 34.,
                color: "data-1",
            },
            Bucket {
                name: "Groceries",
                amount: 28.,
                color: "data-2",
            },
            Bucket {
                name: "Utilities",
                amount: 11.,
                color: "data-3",
            },
            Bucket {
                name: "Other",
                amount: 10.,
                color: "data-8",
            },
            Bucket {
                name: "Transport",
                amount: 9.,
                color: "data-5",
            },
            Bucket {
                name: "Fun",
                amount: 8.,
                color: "data-6",
            },
        ];
        let pcts: [(&str, &str, f32); 6] = [
            ("Rent", "34%", 34.),
            ("Groceries", "28%", 28.),
            ("Utilities", "11%", 11.),
            ("Other", "10%", 10.),
            ("Transport", "9%", 9.),
            ("Fun", "8%", 8.),
        ];
        let donut = div()
            .relative()
            .w(px(200.))
            .h(px(200.))
            .child(
                PieChart::new(buckets.clone())
                    .value(|d| d.amount)
                    .color(move |d| role_hsla(mode, d.color).unwrap_or_default())
                    .inner_radius_ratio(0.573)
                    .pad_angle(0.035)
                    .tooltip_name(|d| SharedString::from(d.name))
                    .tooltip_value(|_d, v, share| {
                        SharedString::from(format!("{v:.0} · {share:.0}%"))
                    }),
            )
            .child(
                v_flex()
                    .absolute()
                    .inset_0()
                    .items_center()
                    .justify_center()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(muted)
                            .child(tr(cx, "gallery.charts.p3.month")),
                    )
                    .child(
                        div()
                            .text_size(px(16.))
                            .line_height(px(21.))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .text_color(fg)
                            .child(tr(cx, "gallery.charts.p3.total")),
                    ),
            );
        let legend_row = |name: &str, pct: &str, color: &str, amount: &str| {
            h_flex()
                .w(px(260.))
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .w(px(8.))
                        .h(px(8.))
                        .rounded_full()
                        .bg(role_hsla(mode, color).unwrap_or_default()),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.))
                        .line_height(px(14.))
                        .text_color(fg)
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .line_height(px(15.))
                        .text_color(muted)
                        .child(pct.to_string()),
                )
                .child(
                    div()
                        .w(px(70.))
                        .text_right()
                        .text_size(px(12.))
                        .line_height(px(14.))
                        .text_color(fg)
                        .child(amount.to_string()),
                )
        };
        let p3_legend = v_flex().gap(px(8.)).children([
            legend_row("Rent", "34%", "data-1", "2.441,32"),
            legend_row("Groceries", "28%", "data-2", "2.010,50"),
            legend_row("Utilities", "11%", "data-3", "789,84"),
            legend_row("Other", "10%", "data-8", "718,04"),
            legend_row("Transport", "9%", "data-5", "646,23"),
            legend_row("Fun", "8%", "data-6", "574,43"),
        ]);
        let p3_group = kit::group(
            cx,
            "gallery.charts.p3.title",
            "gallery.charts.p3.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.charts.p3.live",
                v_flex()
                    .w(px(560.))
                    .p(px(20.))
                    .gap(px(16.))
                    .rounded(px(16.))
                    .bg(role_hsla(mode, "surface").unwrap_or_default())
                    .border_1()
                    .border_color(prim_hsla("white-a4").unwrap_or_default())
                    .child(card_header(
                        cx,
                        mode,
                        tr(cx, "gallery.charts.p3.eyebrow"),
                        tr(cx, "gallery.charts.p3.value"),
                        None,
                    ))
                    .child(h_flex().gap(px(24.)).child(donut).child(p3_legend)),
            )),
        );
        let _ = pcts;

        // ── P4 · candlestick ────────────────────────────────────────────────
        let days = vec![
            Day {
                date: "01",
                open: 100.,
                high: 106.,
                low: 98.,
                close: 104.,
            },
            Day {
                date: "02",
                open: 104.,
                high: 107.,
                low: 101.,
                close: 102.,
            },
            Day {
                date: "03",
                open: 102.,
                high: 105.,
                low: 99.,
                close: 103.5,
            },
            Day {
                date: "04",
                open: 103.5,
                high: 108.,
                low: 103.,
                close: 107.,
            },
            Day {
                date: "05",
                open: 107.,
                high: 109.,
                low: 104.,
                close: 105.,
            },
            Day {
                date: "06",
                open: 105.,
                high: 106.,
                low: 100.,
                close: 101.,
            },
            Day {
                date: "07",
                open: 101.,
                high: 104.,
                low: 99.5,
                close: 103.8,
            },
            Day {
                date: "08",
                open: 103.8,
                high: 107.,
                low: 103.,
                close: 106.2,
            },
            Day {
                date: "09",
                open: 106.2,
                high: 108.5,
                low: 104.,
                close: 104.8,
            },
            Day {
                date: "10",
                open: 104.8,
                high: 106.,
                low: 102.,
                close: 102.5,
            },
            Day {
                date: "11",
                open: 102.5,
                high: 105.5,
                low: 101.5,
                close: 105.,
            },
            Day {
                date: "12",
                open: 105.,
                high: 109.,
                low: 104.5,
                close: 108.4,
            },
            Day {
                date: "13",
                open: 108.4,
                high: 110.,
                low: 106.,
                close: 107.,
            },
            Day {
                date: "14",
                open: 107.,
                high: 108.,
                low: 103.5,
                close: 104.,
            },
            Day {
                date: "15",
                open: 104.,
                high: 106.5,
                low: 102.,
                close: 106.,
            },
            Day {
                date: "16",
                open: 106.,
                high: 110.5,
                low: 105.5,
                close: 110.,
            },
            Day {
                date: "17",
                open: 110.,
                high: 112.,
                low: 108.,
                close: 109.,
            },
            Day {
                date: "18",
                open: 109.,
                high: 111.,
                low: 106.5,
                close: 107.5,
            },
            Day {
                date: "19",
                open: 107.5,
                high: 109.,
                low: 104.,
                close: 108.2,
            },
            Day {
                date: "20",
                open: 108.2,
                high: 111.5,
                low: 107.,
                close: 111.,
            },
            Day {
                date: "21",
                open: 111.,
                high: 113.,
                low: 109.5,
                close: 110.,
            },
            Day {
                date: "22",
                open: 110.,
                high: 112.5,
                low: 108.5,
                close: 112.2,
            },
        ];
        let p4_chart = div().w(px(560.)).h(px(159.5)).child(
            CandlestickChart::new(days)
                .x(|d| d.date)
                .open(|d| d.open)
                .high(|d| d.high)
                .low(|d| d.low)
                .close(|d| d.close)
                .bullish(money_in)
                .bearish(money_out)
                .tick_margin(2)
                .tooltip_value(|_, ix, v| {
                    SharedString::from(format!("{}: {:.2}", ["O", "H", "L", "C"][ix], v))
                }),
        );
        let p4_group = kit::group(
            cx,
            "gallery.charts.p4.title",
            "gallery.charts.p4.hint",
            v_flex().gap_6().child(kit::cell(
                cx,
                "gallery.charts.p4.live",
                chart_card(
                    cx,
                    mode,
                    v_flex()
                        .gap(px(16.))
                        .child(card_header(
                            cx,
                            mode,
                            tr(cx, "gallery.charts.p4.eyebrow"),
                            tr(cx, "gallery.charts.p4.value"),
                            Some(tr(cx, "gallery.charts.p4.tag")),
                        ))
                        .child(p4_chart),
                ),
            )),
        );

        // ── P5 · sparkline · KPI tile ───────────────────────────────────────
        let spark_vals: Vec<f64> = vec![
            42., 48., 45., 52., 50., 58., 55., 61., 57., 64., 60., 66., 62., 70., 68., 74., 71.,
            78., 75., 82., 79., 86., 83., 90., 88., 95., 92., 99., 96., 104.,
        ];
        let kpi = v_flex()
            .w(px(260.))
            .p(px(16.))
            .gap(px(8.))
            .rounded(px(16.))
            .bg(role_hsla(mode, "surface").unwrap_or_default())
            .border_1()
            .border_color(prim_hsla("white-a4").unwrap_or_default())
            .child(
                h_flex()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(muted)
                            .child(tr(cx, "gallery.charts.p5.label")),
                    )
                    .child(
                        h_flex()
                            .h(px(20.))
                            .px(px(6.))
                            .items_center()
                            .rounded(px(4.))
                            .bg(role_hsla(mode, "success-subtle").unwrap_or_default())
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .line_height(px(15.))
                                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                                    .text_color(
                                        role_hsla(mode, "success-foreground").unwrap_or_default(),
                                    )
                                    .child(tr(cx, "gallery.charts.p5.delta")),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .items_end()
                    .gap(px(1.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(16.))
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child(tr(cx, "gallery.charts.p5.amount")),
                    )
                    .child(
                        div()
                            .pl(px(3.))
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(muted)
                            .child(tr(cx, "gallery.charts.p5.cur")),
                    ),
            )
            .child(
                div().w(px(224.)).h(px(36.)).child(
                    LineChart::new(
                        spark_vals
                            .iter()
                            .enumerate()
                            .map(|(i, v)| (i, *v))
                            .collect::<Vec<_>>(),
                    )
                    .x(|(i, _)| i.to_string())
                    .y(|d| d.1)
                    .stroke(data2)
                    .x_axis(false)
                    .grid(false)
                    .y_axis(false)
                    .interactive(false)
                    .appear(false),
                ),
            );
        let p5_group = kit::group(
            cx,
            "gallery.charts.p5.title",
            "gallery.charts.p5.hint",
            v_flex()
                .gap_6()
                .child(kit::cell(cx, "gallery.charts.p5.live", kpi)),
        );

        // ── P6 · chart states (painted) ─────────────────────────────────────
        let tooltip = v_flex()
            .absolute()
            .left(px(391.7))
            .top(px(108.))
            .p(px(8.))
            .pl(px(10.))
            .pr(px(10.))
            .gap(px(2.))
            .rounded(px(12.))
            .bg(role_hsla(mode, "surface-overlay").unwrap_or_default())
            .border_1()
            .border_color(role_hsla(mode, "border-subtle").unwrap_or_default())
            .child(
                div()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .text_color(muted)
                    .child(tr(cx, "gallery.charts.p6.tt_title")),
            )
            .child(
                h_flex()
                    .items_end()
                    .gap(px(1.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(16.))
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child(tr(cx, "gallery.charts.p6.tt_value")),
                    )
                    .child(
                        div()
                            .pl(px(3.))
                            .text_size(px(11.))
                            .line_height(px(15.))
                            .text_color(muted)
                            .child(tr(cx, "gallery.charts.p5.cur")),
                    ),
            );
        let hover_demo = div()
            .relative()
            .w(px(600.))
            .h(px(330.))
            .child(
                v_flex()
                    .w(px(600.))
                    .p(px(20.))
                    .gap(px(16.))
                    .rounded(px(16.))
                    .bg(role_hsla(mode, "surface").unwrap_or_default())
                    .border_1()
                    .border_color(prim_hsla("white-a4").unwrap_or_default())
                    .child(div().h(px(180.)).w(px(560.)).ml(px(20.))),
            )
            .child(
                div()
                    .absolute()
                    .left(px(379.7))
                    .top(px(90.))
                    .w(px(1.))
                    .h(px(180.))
                    .bg(role_hsla(mode, "border-strong").unwrap_or_default()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(374.7))
                    .top(px(159.))
                    .w(px(10.))
                    .h(px(10.))
                    .rounded_full()
                    .bg(data1)
                    .border_2()
                    .border_color(role_hsla(mode, "surface").unwrap_or_default()),
            )
            .child(tooltip);
        let loading_bars = h_flex().items_end().gap(px(22.)).children(
            [60., 90., 70., 120., 100., 140., 80., 110.]
                .iter()
                .map(|h| {
                    div()
                        .w(px(36.))
                        .h(px(*h))
                        .rounded(px(4.))
                        .bg(role_hsla(mode, "skeleton").unwrap_or_default())
                }),
        );
        let loading_demo = v_flex()
            .w(px(600.))
            .h(px(200.))
            .p(px(20.))
            .rounded(px(16.))
            .bg(role_hsla(mode, "surface").unwrap_or_default())
            .border_1()
            .border_color(prim_hsla("white-a4").unwrap_or_default())
            .child(loading_bars);
        let empty_demo = v_flex()
            .w(px(600.))
            .p(px(20.))
            .rounded(px(16.))
            .bg(role_hsla(mode, "surface").unwrap_or_default())
            .border_1()
            .border_color(prim_hsla("white-a4").unwrap_or_default())
            .child(
                Empty::new().w(px(360.)).header(
                    EmptyHeader::new()
                        .media(
                            EmptyMedia::new()
                                .with_variant(EmptyMediaVariant::Default)
                                .child(bookkeepy_ui::api::language::atoms::icon(
                                    gpui_kit::assets::IconName::Receipt,
                                    24.,
                                )),
                        )
                        .title(EmptyTitle::new().child(tr(cx, "gallery.charts.p6.empty_t")))
                        .description(
                            EmptyDescription::new().child(tr(cx, "gallery.charts.p6.empty_d")),
                        ),
                ),
            );
        let p6_group = kit::group(
            cx,
            "gallery.charts.p6.title",
            "gallery.charts.p6.hint",
            v_flex().gap_6().children([
                kit::cell(cx, "gallery.charts.p6.hover", hover_demo),
                kit::cell(cx, "gallery.charts.p6.loading", loading_demo),
                kit::cell(cx, "gallery.charts.p6.empty", empty_demo),
            ]),
        );

        v_flex()
            .size_full()
            .p(px(48.))
            .gap_12()
            .child(p1_group)
            .child(p2_group)
            .child(p3_group)
            .child(p4_group)
            .child(p5_group)
            .child(p6_group)
    }
}
