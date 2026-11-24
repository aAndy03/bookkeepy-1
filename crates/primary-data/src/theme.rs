// SPDX-License-Identifier: GPL-3.0-or-later
//! `data/theme/` — the design-token directory scope (RULE-DATA-003, RULE-UI-005/007):
//! the "variables" of the Figma model (ADR-0017). ONE file per sub-concern
//! (`index.toml` lists them); the loader merges the whole scope and validates it.
//! `bookkeepy-ui` bridges these tokens into the gpui-kit `Theme` — no UI value is ever
//! hardcoded in source (RULE-DATA-001).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::PrimaryDataError;

const INDEX_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/index.toml"
));
const COLORS_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/colors.toml"
));
const SPACING_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/spacing.toml"
));
const RADII_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/radii.toml"
));
const TYPOGRAPHY_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/typography.toml"
));
const ELEVATION_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/elevation.toml"
));

/// Theme mode. Dark is the design language's native theme; light mirrors the same ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// The designed native theme.
    Dark,
    /// Ramp inversion with identical functional hues.
    Light,
}

/// Scope index (`[meta]`): the shipped themes.
#[derive(Debug, Deserialize)]
struct IndexFile {
    meta: IndexMeta,
}

/// `[meta]` of the index file.
#[derive(Debug, Deserialize)]
struct IndexMeta {
    /// Theme ids shipped in this scope — must match the `colors.toml` theme tables.
    themes: Vec<Mode>,
}

/// The semantic color tokens of one theme mode. Every UI surface resolves through these
/// (bg/fg/border/accent + functional hues + chart series) — raw values live only in
/// `[palette]` and are never referenced past token resolution.
#[derive(Debug, Clone, Deserialize)]
pub struct SemanticColors {
    /// Base background everywhere.
    pub bg: String,
    /// Inset wells: inputs, code, sunken areas.
    pub bg_canvas: String,
    /// Hover tint on lists/rows; subtle raised surfaces.
    pub bg_raised: String,
    /// Cards, dialogs, panels.
    pub bg_sheet: String,
    /// Title bar, buttons, selected rows, controls.
    pub bg_chrome: String,
    /// Pointer-hover on a base surface.
    pub bg_hover: String,
    /// Selected row/list item.
    pub bg_selected: String,
    /// Modal scrim (black @ 60%).
    pub overlay: String,
    /// Default body text.
    pub fg: String,
    /// Emphasized text.
    pub fg_primary: String,
    /// Secondary text.
    pub fg_muted: String,
    /// Tertiary text.
    pub fg_subtle: String,
    /// Input placeholders (RULE-UI-005).
    pub fg_placeholder: String,
    /// Disabled labels.
    pub fg_disabled: String,
    /// Brand accent — primary actions, focus, links.
    pub accent: String,
    /// Hovered accent surface.
    pub accent_hover: String,
    /// Quiet accent — selected indicators.
    pub accent_subtle: String,
    /// Text/icons on an accent surface.
    pub on_accent: String,
    /// Text-selection highlight.
    pub selection: String,
    /// Success hue.
    pub success: String,
    /// Success tint (hover/selected).
    pub success_tint: String,
    /// Success tinted well (container).
    pub success_well: String,
    /// Warning hue.
    pub warning: String,
    /// Warning tint.
    pub warning_tint: String,
    /// Warning well.
    pub warning_well: String,
    /// Danger hue.
    pub danger: String,
    /// Danger tint.
    pub danger_tint: String,
    /// Danger well.
    pub danger_well: String,
    /// Info hue.
    pub info: String,
    /// Info tint.
    pub info_tint: String,
    /// Info well.
    pub info_well: String,
    /// Default border.
    pub border: String,
    /// Separators inside a surface.
    pub border_subtle: String,
    /// Emphasized control border.
    pub border_strong: String,
    /// Highest-contrast border.
    pub border_heavy: String,
    /// Keyboard focus ring (RULE-INPUT-001).
    pub focus_ring: String,
    /// Charts: positive series.
    pub chart_positive: String,
    /// Charts: negative series.
    pub chart_negative: String,
    /// Charts: neutral series.
    pub chart_neutral: String,
    /// Charts: series 1.
    pub chart_series_1: String,
    /// Charts: series 2.
    pub chart_series_2: String,
    /// Charts: series 3.
    pub chart_series_3: String,
    /// Charts: series 4.
    pub chart_series_4: String,
    /// Charts: series 5.
    pub chart_series_5: String,
}

impl SemanticColors {
    /// (token name, raw value) for every semantic hex token — the validation projection.
    fn pairs(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("bg", &self.bg),
            ("bg_canvas", &self.bg_canvas),
            ("bg_raised", &self.bg_raised),
            ("bg_sheet", &self.bg_sheet),
            ("bg_chrome", &self.bg_chrome),
            ("bg_hover", &self.bg_hover),
            ("bg_selected", &self.bg_selected),
            ("overlay", &self.overlay),
            ("fg", &self.fg),
            ("fg_primary", &self.fg_primary),
            ("fg_muted", &self.fg_muted),
            ("fg_subtle", &self.fg_subtle),
            ("fg_placeholder", &self.fg_placeholder),
            ("fg_disabled", &self.fg_disabled),
            ("accent", &self.accent),
            ("accent_hover", &self.accent_hover),
            ("accent_subtle", &self.accent_subtle),
            ("on_accent", &self.on_accent),
            ("selection", &self.selection),
            ("success", &self.success),
            ("success_tint", &self.success_tint),
            ("success_well", &self.success_well),
            ("warning", &self.warning),
            ("warning_tint", &self.warning_tint),
            ("warning_well", &self.warning_well),
            ("danger", &self.danger),
            ("danger_tint", &self.danger_tint),
            ("danger_well", &self.danger_well),
            ("info", &self.info),
            ("info_tint", &self.info_tint),
            ("info_well", &self.info_well),
            ("border", &self.border),
            ("border_subtle", &self.border_subtle),
            ("border_strong", &self.border_strong),
            ("border_heavy", &self.border_heavy),
            ("focus_ring", &self.focus_ring),
            ("chart_positive", &self.chart_positive),
            ("chart_negative", &self.chart_negative),
            ("chart_neutral", &self.chart_neutral),
            ("chart_series_1", &self.chart_series_1),
            ("chart_series_2", &self.chart_series_2),
            ("chart_series_3", &self.chart_series_3),
            ("chart_series_4", &self.chart_series_4),
            ("chart_series_5", &self.chart_series_5),
        ]
    }
}

/// The two shipped theme maps (`[theme.dark]`, `[theme.light]`).
#[derive(Debug, Deserialize)]
struct ThemeMaps {
    dark: SemanticColors,
    light: SemanticColors,
}

/// Parsed `colors.toml`.
#[derive(Debug, Deserialize)]
struct ColorsFile {
    theme: ThemeMaps,
}

/// Parsed `spacing.toml` (`[source]` + `[spacing]`) — multiples of the base grid.
#[derive(Debug, Deserialize)]
struct SpacingFile {
    spacing: Spacing,
}

/// The spacing scale (`[spacing]`) — multiples of the base grid.
#[derive(Debug, Clone, Deserialize)]
pub struct Spacing {
    /// Base unit every spacing token is a multiple of.
    pub grid: u32,
    /// 4px — icon↔label inside a control.
    pub x1: u32,
    /// 8px — control inner padding (vertical), related-item gaps.
    pub x2: u32,
    /// 12px — compact group padding, dense-list rows.
    pub x3: u32,
    /// 16px — standard card/dialog padding, form-field gaps.
    pub x4: u32,
    /// 20px — generous card/panel padding.
    pub x5: u32,
    /// 24px — section padding, dialog margins.
    pub x6: u32,
    /// 32px — between groups/sections.
    pub x8: u32,
    /// 40px — major section spacing, wide page gutter.
    pub x10: u32,
    /// 48px — empty-state padding.
    pub x12: u32,
    /// 64px — page margins / hero spacing.
    pub x16: u32,
}

/// Parsed `radii.toml` (`[source]` + `[radii]`).
#[derive(Debug, Deserialize)]
struct RadiiFile {
    radii: Radii,
}

/// The radius scale (`[radii]`) — px.
#[derive(Debug, Clone, Deserialize)]
pub struct Radii {
    /// Fine detail edges.
    pub micro: u32,
    /// Tags, small badges.
    pub chip: u32,
    /// Checkboxes, kbd keys, small markers.
    pub mini: u32,
    /// List rows, dropdown/menu items.
    pub item: u32,
    /// Buttons, inputs, selects, cards-in-lists, window frame.
    pub control: u32,
    /// Cards, menus, popovers, date pickers.
    pub card: u32,
    /// Dialogs, sheets, alert dialogs.
    pub overlay: u32,
    /// Large side panels.
    pub banner: u32,
    /// Floating boards/windows.
    pub board: u32,
    /// Circular surfaces (radius = half of the side).
    pub circle: u32,
    /// Pills — renderers clamp to half the shorter side.
    pub full: u32,
}

/// Parsed `typography.toml` (`[font]` + `[[style]]`).
#[derive(Debug, Deserialize)]
pub struct TypographyFile {
    font: FontStack,
    style: Vec<TextStyle>,
}

impl TypographyFile {
    /// The font stacks (`[font]`): `ui` for surfaces, `mono` for numeric/code.
    pub fn fonts(&self) -> &FontStack {
        &self.font
    }
}

/// The font stacks (`[font]`).
#[derive(Debug, Clone, Deserialize)]
pub struct FontStack {
    /// UI font stack with per-OS fallbacks.
    pub ui: String,
    /// Numeric/monospaced surface family.
    pub mono: String,
}

/// One named text style from `[[style]]`.
#[derive(Debug, Clone, Deserialize)]
pub struct TextStyle {
    /// Style id (SSOT name referenced by widgets).
    pub id: String,
    /// Font size, px.
    pub size_px: u32,
    /// Weight (400/500/600).
    pub weight: u32,
    /// Unitless line-height multiplier.
    pub line_height: f32,
    /// Letter spacing, px (negative = tighter).
    pub letter_spacing_px: f32,
    /// `ui` or `mono`.
    pub font: String,
    /// Tabular figures (amounts) — aligned in columns.
    #[serde(default)]
    pub tabular: bool,
}

/// One shadow step from `elevation.toml`.
#[derive(Debug, Deserialize)]
pub struct Shadow {
    /// Blur radius, px.
    pub blur_px: u32,
    /// Vertical offset, px.
    pub offset_y_px: u32,
    /// Horizontal offset, px.
    pub offset_x_px: u32,
    /// Shadow color.
    pub color: String,
    /// Opacity, 0..=1.
    pub opacity: f32,
}

/// The three shadow steps (`[shadow.*]`).
#[derive(Debug, Deserialize)]
struct Shadows {
    peek: Shadow,
    raised: Shadow,
    overlay: Shadow,
}

/// Parsed `elevation.toml` (`[shadow.*]` + `[z]` layers; z-index is consumed by the layout
/// contracts, shadows by the theme bridge).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ElevationFile {
    shadow: Shadows,
}

fn index() -> &'static IndexFile {
    static FILE: OnceLock<IndexFile> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(INDEX_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/index.toml failed to parse: {e}"))
    })
}

fn colors_file() -> &'static ColorsFile {
    static FILE: OnceLock<ColorsFile> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(COLORS_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/colors.toml failed to parse: {e}"))
    })
}

/// Semantic color tokens of `mode` (dark is the design language's native theme).
pub fn colors(mode: Mode) -> &'static SemanticColors {
    static DARK: OnceLock<&SemanticColors> = OnceLock::new();
    static LIGHT: OnceLock<&SemanticColors> = OnceLock::new();
    match mode {
        Mode::Dark => DARK.get_or_init(|| &colors_file().theme.dark),
        Mode::Light => LIGHT.get_or_init(|| &colors_file().theme.light),
    }
}

/// Spacing tokens (the base-grid scale).
pub fn spacing() -> &'static Spacing {
    static TOKENS: OnceLock<SpacingFile> = OnceLock::new();
    &TOKENS
        .get_or_init(|| {
            toml::from_str(SPACING_RAW)
                .unwrap_or_else(|e| panic!("primary data theme/spacing.toml failed to parse: {e}"))
        })
        .spacing
}

/// Corner-radius scale.
pub fn radii() -> &'static Radii {
    static TOKENS: OnceLock<RadiiFile> = OnceLock::new();
    &TOKENS
        .get_or_init(|| {
            toml::from_str(RADII_RAW)
                .unwrap_or_else(|e| panic!("primary data theme/radii.toml failed to parse: {e}"))
        })
        .radii
}

/// Typography: font stacks + named text styles.
pub fn typography() -> &'static TypographyFile {
    static TOKENS: OnceLock<TypographyFile> = OnceLock::new();
    TOKENS.get_or_init(|| {
        toml::from_str(TYPOGRAPHY_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/typography.toml failed to parse: {e}"))
    })
}

/// Named text style by id (e.g. `body`, `amount`).
pub fn style(id: &str) -> Option<&'static TextStyle> {
    typography().style.iter().find(|s| s.id == id)
}

/// The three shadow steps: `(peek, raised, overlay)` — black @ 60%, doubling steps.
pub fn shadows() -> (&'static Shadow, &'static Shadow, &'static Shadow) {
    static TOKENS: OnceLock<ElevationFile> = OnceLock::new();
    let file = TOKENS.get_or_init(|| {
        toml::from_str(ELEVATION_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/elevation.toml failed to parse: {e}"))
    });
    (&file.shadow.peek, &file.shadow.raised, &file.shadow.overlay)
}

fn hex_ok(v: &str) -> bool {
    let h = v.strip_prefix('#').unwrap_or(v);
    (h.len() == 6 || h.len() == 8) && h.chars().all(|c| c.is_ascii_hexdigit())
}

/// Structural + semantic validation (RULE-DATA-002: scope index ↔ files agree).
pub fn validate_structure() -> Result<(), PrimaryDataError> {
    let idx = index();
    if idx.meta.themes != vec![Mode::Dark, Mode::Light] {
        return Err(PrimaryDataError::Invalid(
            "theme/index.toml must ship themes = [\"dark\", \"light\"]".into(),
        ));
    }
    for (mode, map) in [
        (Mode::Dark, colors(Mode::Dark)),
        (Mode::Light, colors(Mode::Light)),
    ] {
        let mode_name = match mode {
            Mode::Dark => "dark",
            Mode::Light => "light",
        };
        for (name, value) in map.pairs() {
            if !hex_ok(value) {
                return Err(PrimaryDataError::Invalid(format!(
                    "theme/colors.toml theme.{mode_name}.{name} is not a #rrggbb[aa] value: {value:?}"
                )));
            }
        }
    }
    if colors(Mode::Dark).accent != colors(Mode::Light).accent {
        return Err(PrimaryDataError::Invalid(
            "theme/colors.toml the light theme must keep the dark theme's accent hue".into(),
        ));
    }
    let s = spacing();
    if s.grid == 0 {
        return Err(PrimaryDataError::Invalid(
            "theme/spacing.toml grid must be > 0".into(),
        ));
    }
    for (name, v) in [
        ("x1", s.x1),
        ("x2", s.x2),
        ("x3", s.x3),
        ("x4", s.x4),
        ("x5", s.x5),
        ("x6", s.x6),
        ("x8", s.x8),
        ("x10", s.x10),
        ("x12", s.x12),
        ("x16", s.x16),
    ] {
        if v == 0 || v % s.grid != 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/spacing.toml {name}={v} must be a positive multiple of grid {}",
                s.grid
            )));
        }
    }
    let t = typography();
    if t.font.ui.is_empty() || t.font.mono.is_empty() {
        return Err(PrimaryDataError::Invalid(
            "theme/typography.toml font.ui and font.mono must not be empty".into(),
        ));
    }
    let mut ids: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for st in &t.style {
        if !ids.insert(st.id.as_str()) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/typography.toml duplicate style id {:?}",
                st.id
            )));
        }
        if !["ui", "mono"].contains(&st.font.as_str()) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/typography.toml style {:?} unknown font {:?} (ui|mono)",
                st.id, st.font
            )));
        }
    }
    if style("body").is_none() {
        return Err(PrimaryDataError::Invalid(
            "theme/typography.toml must define the `body` text style".into(),
        ));
    }
    let (peek, raised, overlay) = shadows();
    for (name, shadow) in [("peek", peek), ("raised", raised), ("overlay", overlay)] {
        if !(0.0..=1.0).contains(&shadow.opacity) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/elevation.toml shadow.{name} opacity must be within 0..=1"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn theme_validates() {
        super::validate_structure().unwrap();
        let dark = super::colors(super::Mode::Dark);
        assert_eq!(dark.bg, "#111111");
        assert_eq!(dark.accent, "#f5823a");
        let light = super::colors(super::Mode::Light);
        assert_eq!(light.bg, "#ededed");
        assert_eq!(dark.accent, super::colors(super::Mode::Light).accent);
        assert_eq!(super::spacing().grid, 4);
        assert_eq!(super::radii().control, 8);
        let body = super::style("body").unwrap();
        assert_eq!(body.size_px, 13);
        assert!(super::style("nonexistent").is_none());
    }
}
