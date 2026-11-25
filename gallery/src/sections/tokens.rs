// SPDX-License-Identifier: GPL-3.0-or-later
//! TOKENS sheet — the design tokens themselves, rendered straight from
//! `data/theme/*.toml` (the SSOT, RULE-DATA-001/003): colors, radii, spacing,
//! typography, elevation. Reading the raw files (not the parsed structs) keeps this
//! section honest: what you see is exactly what ships.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::h_flex;
use gpui_kit::component::theme::try_parse_color;
use gpui_kit::{
    AnyView, AppContext as _, Context, IntoElement, ParentElement as _, Styled as _, Window, div,
    px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, section_page};

const COLORS_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/theme/colors.toml"
));
const RADII_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/theme/radii.toml"
));
const SPACING_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/theme/spacing.toml"
));
const TYPOGRAPHY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/theme/typography.toml"
));
const ELEVATION_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/theme/elevation.toml"
));

/// The TOKENS section.
pub struct Section;

pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_| Section).into()
}

/// One color row: swatch + token name + raw value.
fn color_row(cx: &gpui_kit::App, key: &str, value: &str) -> gpui_kit::AnyElement {
    let swatch = try_parse_color(value).unwrap_or_default();
    h_flex()
        .items_center()
        .gap_3()
        .child(div().size(px(16.)).rounded(px(4.)).bg(swatch))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(key.to_string()),
        )
        .child(div().text_xs().child(value.to_string()))
        .into_any_element()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        // Colors: the dark palette's semantic + group tokens.
        let colors: toml::Value =
            toml::from_str(COLORS_TOML).unwrap_or(toml::Value::Table(Default::default()));
        let mut color_rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if let Some(dark) = colors
            .get("theme")
            .and_then(|t| t.get("dark"))
            .and_then(|t| t.as_table())
        {
            for (k, v) in dark {
                if let Some(s) = v.as_str() {
                    color_rows.push(color_row(cx, k, s));
                }
            }
        }
        if let Some(group) = colors
            .get("palette")
            .and_then(|p| p.get("group"))
            .and_then(|t| t.as_table())
        {
            for (k, v) in group {
                if let Some(s) = v.as_str() {
                    color_rows.push(color_row(cx, &format!("palette.group.{k}"), s));
                }
            }
        }
        // Radii: one chip per token, drawn at its own radius.
        let radii: toml::Value =
            toml::from_str(RADII_TOML).unwrap_or(toml::Value::Table(Default::default()));
        let mut radius_rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if let Some(r) = radii.get("radii").and_then(|t| t.as_table()) {
            for (k, v) in r {
                if let Some(n) = v.as_integer() {
                    radius_rows.push(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(28.))
                                    .h(px(20.))
                                    .rounded(px(n as f32))
                                    .bg(theme.accent),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{k} = {n}px")),
                            )
                            .into_any_element(),
                    );
                }
            }
        }
        // Spacing: one bar per token, drawn at its own width on the 4px grid.
        let spacing: toml::Value =
            toml::from_str(SPACING_TOML).unwrap_or(toml::Value::Table(Default::default()));
        let mut spacing_rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if let Some(s) = spacing.get("spacing").and_then(|t| t.as_table()) {
            for (k, v) in s {
                if k == "grid" {
                    continue;
                }
                if let Some(n) = v.as_integer() {
                    spacing_rows.push(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .h(px(8.))
                                    .w(px(n as f32))
                                    .rounded(px(2.))
                                    .bg(theme.accent),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{k} = {n}px")),
                            )
                            .into_any_element(),
                    );
                }
            }
        }
        // Typography: each named style rendered as a sample line.
        let typo: toml::Value =
            toml::from_str(TYPOGRAPHY_TOML).unwrap_or(toml::Value::Table(Default::default()));
        let mut type_rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if let Some(styles) = typo.get("style").and_then(|t| t.as_array()) {
            for st in styles {
                let id = st.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let size = st.get("size_px").and_then(|v| v.as_float()).unwrap_or(14.);
                let weight = st.get("weight").and_then(|v| v.as_integer()).unwrap_or(400);
                type_rows.push(
                    div()
                        .flex()
                        .items_baseline()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(size as f32))
                                .font_weight(gpui_kit::FontWeight::from(weight as f32))
                                .child(tr(cx, "gallery.tokens.type_sample")),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("{id} · {size}px/{weight}")),
                        )
                        .into_any_element(),
                );
            }
        }
        // Elevation: z-index rows.
        let elev: toml::Value =
            toml::from_str(ELEVATION_TOML).unwrap_or(toml::Value::Table(Default::default()));
        let mut z_rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if let Some(z) = elev.get("z").and_then(|t| t.as_table()) {
            for (k, v) in z {
                if let Some(n) = v.as_integer() {
                    z_rows.push(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(div().w(px(28.)).h(px(10.)).rounded(px(2.)).bg(theme.accent))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("z.{k} = {n}")),
                            )
                            .into_any_element(),
                    );
                }
            }
        }
        section_page(
            cx,
            "gallery.section.tokens",
            [
                demo_card(
                    cx,
                    "gallery.tokens.colors.caption",
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(color_rows)
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.tokens.radii.caption",
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_4()
                        .children(radius_rows)
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.tokens.spacing.caption",
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(spacing_rows)
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.tokens.typography.caption",
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .children(type_rows)
                        .into_any_element(),
                )
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.tokens.elevation.caption",
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(z_rows)
                        .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
