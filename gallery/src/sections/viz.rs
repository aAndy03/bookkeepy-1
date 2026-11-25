// SPDX-License-Identifier: GPL-3.0-or-later
//! Viz — the rich, interactive visual language (RULE-UI-009): the Chart family
//! (LineChart, AreaChart, BarChart, PieChart, RadarChart, CandlestickChart,
//! SankeyChart), a hand-rolled Plot built from the low-level primitives
//! (scales, shapes, axes), and the Diff review surface.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::chart::{
    AreaChart, BarChart, CandlestickChart, LineChart, PieChart, RadarChart, SankeyChart,
};
use gpui_kit::component::diff::{Diff, DiffFile, DiffState};
use gpui_kit::component::plot::scale::{Scale as _, ScaleLinear};
use gpui_kit::component::plot::shape::{Line as PlotLine, SankeyLink};
use gpui_kit::component::plot::{AxisText, IntoPlot, Plot, PlotAxis};
use gpui_kit::{
    AnyView, App, AppContext as _, Bounds, Context, Entity, Hsla, IntoElement, ParentElement as _,
    Pixels, SharedString, Styled as _, TextAlign, Window, div, px, rems,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// A labeled numeric point (neutral demo data; labels come from i18n).
#[derive(Clone)]
struct Point {
    label: SharedString,
    value: f64,
}

/// The months used as the x axis of the time-series demos.
const MONTH_KEYS: [&str; 6] = [
    "gallery.viz.month.1",
    "gallery.viz.month.2",
    "gallery.viz.month.3",
    "gallery.viz.month.4",
    "gallery.viz.month.5",
    "gallery.viz.month.6",
];

/// The monthly values of the line/area demos.
const MONTH_VALUES: [f64; 6] = [12.4, 9.8, 13.5, 11.2, 14.8, 13.1];

/// The OHLC (open, high, low, close) values of the candlestick demo.
const MONTH_OHLC: [[f64; 4]; 6] = [
    [10.2, 11.4, 9.8, 11.1],
    [11.1, 12.0, 10.6, 11.8],
    [11.8, 11.9, 10.9, 11.2],
    [11.2, 13.1, 11.0, 12.8],
    [12.8, 13.4, 12.1, 12.4],
    [12.4, 14.6, 12.3, 14.2],
];

/// An OHLC candle of the candlestick demo.
struct Candle {
    label: SharedString,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

/// The months of the bar demo (in/out direction per month, CHARTS bar card).
const BAR_VALUES: [f64; 6] = [1200., -840., 2400., -1500., 450., -620.];

/// A bar of the bar demo: months on the band axis, signed values colored by direction
/// (money-in = bullish green, money-out = bearish red — CHARTS sheet direction rule).
#[derive(Clone)]
struct Bar {
    label: SharedString,
    value: f64,
}

/// The slices of the pie demo — six shares in the donut legend's data order
/// (cyan, lime, pink, sand, orange, green).
const PIE_KEYS: [&str; 6] = [
    "gallery.viz.pie.income",
    "gallery.viz.pie.expenses",
    "gallery.viz.pie.savings",
    "gallery.viz.pie.rent",
    "gallery.viz.pie.transport",
    "gallery.viz.pie.fun",
];

/// The share values of the pie demo.
const PIE_VALUES: [f64; 6] = [5200., 3300., 1900., 1250., 640., 410.];

/// A donut slice, carrying its data-palette color (CHARTS donut legend order).
struct Slice {
    label: SharedString,
    value: f64,
    color: Hsla,
}

/// The dimensions of the radar demo.
const RADAR_KEYS: [&str; 5] = [
    "gallery.viz.radar.bookkeeping",
    "gallery.viz.radar.invoicing",
    "gallery.viz.radar.payroll",
    "gallery.viz.radar.taxes",
    "gallery.viz.radar.reporting",
];

/// The radar values of the radar demo.
const RADAR_VALUES: [f64; 5] = [82., 90., 74., 68., 88.];

/// A node of the sankey flow demo.
struct FlowNode {
    name: SharedString,
}

/// The nodes of the sankey demo, in link-index order.
const SANKEY_KEYS: [&str; 4] = [
    "gallery.viz.sankey.revenue",
    "gallery.viz.sankey.costs",
    "gallery.viz.sankey.gross_profit",
    "gallery.viz.sankey.net_profit",
];

/// The monthly series of the line/area demos.
fn monthly_points(cx: &App) -> Vec<Point> {
    MONTH_KEYS
        .iter()
        .zip(MONTH_VALUES)
        .map(|(key, value)| Point {
            label: tr(cx, key),
            value,
        })
        .collect()
}

/// The candle series of the candlestick demo.
fn candle_points(cx: &App) -> Vec<Candle> {
    MONTH_KEYS
        .iter()
        .zip(MONTH_OHLC)
        .map(|(key, [open, high, low, close])| Candle {
            label: tr(cx, key),
            open,
            high,
            low,
            close,
        })
        .collect()
}

/// The bars of the bar demo, one per month (direction shown by sign).
fn bar_points(cx: &App) -> Vec<Bar> {
    MONTH_KEYS
        .iter()
        .zip(BAR_VALUES)
        .map(|(key, value)| Bar {
            label: tr(cx, key),
            value,
        })
        .collect()
}

/// The slices of the pie demo, colored in the donut legend's data order.
fn pie_points(cx: &App) -> Vec<Slice> {
    let palette = [
        cx.theme().chart_1,
        cx.theme().chart_2,
        cx.theme().chart_3,
        cx.theme().chart_4,
        cx.theme().chart_5,
        cx.theme().chart_bullish,
    ];
    PIE_KEYS
        .iter()
        .zip(PIE_VALUES)
        .enumerate()
        .map(|(i, (key, value))| Slice {
            label: tr(cx, key),
            value,
            color: palette[i % palette.len()],
        })
        .collect()
}

/// The dimensions of the radar demo.
fn radar_points(cx: &App) -> Vec<Point> {
    RADAR_KEYS
        .iter()
        .zip(RADAR_VALUES)
        .map(|(key, value)| Point {
            label: tr(cx, key),
            value,
        })
        .collect()
}

/// The nodes of the sankey demo.
fn sankey_nodes(cx: &App) -> Vec<FlowNode> {
    SANKEY_KEYS
        .iter()
        .map(|key| FlowNode { name: tr(cx, key) })
        .collect()
}

/// A fixed-size frame for a chart (plot elements fill their container).
fn chart_frame(demo: impl IntoElement) -> gpui_kit::AnyElement {
    div().w(px(320.)).h(px(200.)).child(demo).into_any_element()
}

/// A short strip frame — the CHARTS sparkline card (600x200 sheet strip, drawn small).
fn strip_frame(demo: impl IntoElement) -> gpui_kit::AnyElement {
    div().w(px(320.)).h(px(64.)).child(demo).into_any_element()
}

/// A hand-rolled mini chart showing the low-level `plot` building blocks
/// (`ScaleLinear`, the `Line` shape, `PlotAxis`/`AxisText`) — the primitives
/// behind the styled Chart components.
#[derive(IntoPlot)]
struct MiniPlot {
    points: Vec<Point>,
}

impl Plot for MiniPlot {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        const X_AXIS_GAP: f32 = 22.;
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32() - X_AXIS_GAP;

        // Scales: point indices across the width, values from 0 up the height.
        let x = ScaleLinear::new((0..self.points.len()).map(|i| i as f32), [0., width]);
        let y = ScaleLinear::new(
            self.points.iter().map(|p| p.value as f32).chain([0.]),
            [height, 0.],
        );

        // The x axis: one localized tick label per point.
        let labels = self.points.iter().enumerate().map(|(i, p)| {
            AxisText::new(
                p.label.clone(),
                px(x.tick(&(i as f32)).unwrap_or(0.)),
                cx.theme().muted_foreground,
            )
            .align(TextAlign::Center)
        });
        PlotAxis::new()
            .stroke(cx.theme().border)
            .x(height)
            .x_label(labels)
            .paint(&bounds, window, cx);

        // The series line with dots, drawn from the pixel space of the scales.
        let (x_line, y_line) = (x, y);
        PlotLine::new()
            .data(self.points.iter().enumerate())
            .x(move |(i, _)| x_line.tick(&(*i as f32)))
            .y(move |(_, p)| y_line.tick(&(p.value as f32)))
            .stroke(cx.theme().chart_1)
            .stroke_width(2.)
            .dot()
            .dot_size(px(4.))
            .dot_fill(cx.theme().chart_1)
            .paint(&bounds, window);
    }
}

/// The viz section.
pub struct Section {
    diff: Entity<DiffState>,
}

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        // The Diff's sample patch: a two-line change in one translated file.
        let file = tr(cx, "gallery.viz.diff.file");
        let context = tr(cx, "gallery.viz.diff.context");
        let removed = tr(cx, "gallery.viz.diff.removed");
        let added = tr(cx, "gallery.viz.diff.added");
        let patch = format!(
            "--- a/{file}\n+++ b/{file}\n@@ -1,3 +1,3 @@\n {context}\n-{removed}\n+{added}\n {context}\n"
        );
        // Parsed once when the section is created (never during rendering).
        let files = DiffFile::parse(&patch).unwrap_or_default();
        let diff = cx.new(|cx| DiffState::new(files, cx));
        Section { diff }
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let months = monthly_points(cx);
        let candles = candle_points(cx);
        let bars = bar_points(cx);
        let slices = pie_points(cx);
        let radar = radar_points(cx);
        let nodes = sankey_nodes(cx);

        section_page(
            cx,
            "gallery.section.viz",
            [
                demo_row([
                    demo_card(
                        cx,
                        "gallery.viz.chart.line",
                        chart_frame(
                            LineChart::new(months.clone())
                                .id("viz-line-chart")
                                .x(|d| d.label.clone())
                                .y(|d| d.value)
                                // CHARTS sheet: series in data-1, gridlines solid dividers.
                                .stroke(cx.theme().chart_1)
                                .grid_dashed(false)
                                .dot(),
                        ),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.chart.area",
                        chart_frame(
                            AreaChart::new(months.clone())
                                .id("viz-area-chart")
                                .x(|d| d.label.clone())
                                .y(|d| d.value)
                                // CHARTS sheet: data-1 stroke over a white @ 8% area fill.
                                .stroke(cx.theme().chart_1)
                                .fill(cx.theme().accent)
                                .grid_dashed(false),
                        ),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.chart.bar",
                        chart_frame({
                            let (bullish, bearish) =
                                (cx.theme().chart_bullish, cx.theme().chart_bearish);
                            BarChart::new(bars.clone())
                                .id("viz-bar-chart")
                                .band(|d| d.label.clone())
                                .value(|d| d.value)
                                .label(|d| format!("{:.0}", d.value))
                                // CHARTS bar card: direction colors — in = bullish
                                // green, out = bearish red; 2px bar tips.
                                .fill(
                                    move |d, _, _, _| {
                                        if d.value >= 0. { bullish } else { bearish }
                                    },
                                )
                                .corner_radii(px(2.))
                                .grid_dashed(false)
                        }),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.chart.pie",
                        chart_frame(
                            PieChart::new(slices)
                                .id("viz-pie-chart")
                                .value(|d| d.value as f32)
                                .label(|d| d.label.clone())
                                // CHARTS donut: data-palette slices with 2° gaps.
                                .color(|d| d.color)
                                .pad_angle(2_f32.to_radians())
                                .inner_radius_ratio(0.55),
                        ),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.viz.chart.radar",
                        chart_frame(
                            RadarChart::new(radar)
                                .id("viz-radar-chart")
                                .label(|d| d.label.clone())
                                .value(|d| d.value)
                                // Single-series overview: the accent orange, softly filled.
                                .stroke(cx.theme().chart_5)
                                .fill(cx.theme().chart_5.opacity(0.16))
                                .dot(),
                        ),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.chart.candle",
                        chart_frame(
                            CandlestickChart::new(candles)
                                .id("viz-candle-chart")
                                .x(|d| d.label.clone())
                                .open(|d| d.open)
                                .high(|d| d.high)
                                .low(|d| d.low)
                                .close(|d| d.close),
                        ),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.chart.sankey",
                        chart_frame(
                            SankeyChart::new(
                                nodes,
                                vec![
                                    SankeyLink::new(0, 1, 32.),
                                    SankeyLink::new(0, 2, 45.),
                                    SankeyLink::new(2, 3, 30.),
                                ],
                            )
                            .id("viz-sankey-chart")
                            .node_label(|d| d.name.clone()),
                        ),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.viz.plot",
                        chart_frame(MiniPlot {
                            points: months.clone(),
                        }),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.chart.sparkline",
                        // CHARTS sparkline: a bare in/out bar strip, no axes or labels.
                        strip_frame({
                            let (bullish, bearish) =
                                (cx.theme().chart_bullish, cx.theme().chart_bearish);
                            BarChart::new(bars)
                                .id("viz-sparkline")
                                .band(|d| d.label.clone())
                                .value(|d| d.value)
                                .fill(
                                    move |d, _, _, _| {
                                        if d.value >= 0. { bullish } else { bearish }
                                    },
                                )
                                .label_axis(false)
                                .value_axis(false)
                                .grid(false)
                        }),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.viz.diff",
                        Diff::new(&self.diff)
                            .h(rems(14.))
                            .w_full()
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
            ],
        )
    }
}
