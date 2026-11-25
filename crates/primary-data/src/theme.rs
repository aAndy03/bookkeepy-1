// SPDX-License-Identifier: GPL-3.0-or-later
//! `data/theme/` — the design-token directory scope (RULE-DATA-003, RULE-UI-005/007):
//! the "variables" of the Figma model (ADR-0017). ONE file per sub-concern
//! (`index.toml` lists them); the loader merges the whole scope and validates it.
//! The files are 1:1 with the design language's TOKENS sheet: color roles reference
//! primitives as "$neutral-925" aliases, resolved here. `bookkeepy-ui` bridges these
//! tokens into the gpui-kit `Theme` — no UI value is ever hardcoded in source
//! (RULE-DATA-001).

use std::collections::BTreeMap;
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
const MOTION_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/motion.toml"
));
const ICONS_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/icons.toml"
));
const LAYOUT_RAW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/theme/layout.toml"
));

/// Theme mode. Dark is the design language's native theme; light mirrors the same ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// The designed native theme.
    Dark,
    /// Ramp inversion with identical functional hues.
    Light,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
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

/// Raw `colors.toml`: primitives + per-mode role groups.
#[derive(Debug, Deserialize)]
struct ColorsFile {
    palette: PaletteRaw,
    theme: BTreeMap<Mode, BTreeMap<String, BTreeMap<String, String>>>,
}

/// Raw `[palette.*]` — flattened to `<group>-<step>` / `<hue>-<step>` aliases.
#[derive(Debug, Deserialize)]
struct PaletteRaw {
    neutral: BTreeMap<String, String>,
    alpha: BTreeMap<String, String>,
    hue: BTreeMap<String, BTreeMap<String, String>>,
}

/// The resolved color palette: alias → lowercase hex (e.g. `neutral-925` → `#111111`).
pub type Palette = BTreeMap<String, String>;

/// Semantic color roles of one theme mode, 1:1 with the TOKENS sheet §02.
/// Values in the file are `#hex` or `$alias`; [`ThemeColors::get`] returns the resolved
/// lowercase hex.
#[derive(Debug)]
pub struct ThemeColors {
    raw: BTreeMap<String, String>,
    resolved: OnceLock<BTreeMap<String, String>>,
}

impl ThemeColors {
    fn parse(groups: BTreeMap<String, BTreeMap<String, String>>) -> Self {
        let raw = groups.into_values().flatten().collect();
        Self {
            raw,
            resolved: OnceLock::new(),
        }
    }

    /// The raw (unresolved) role value — `#hex` or `$alias`.
    pub fn raw(&self, role: &str) -> Option<&str> {
        self.raw.get(role).map(|s| s.as_str())
    }

    /// The resolved role color as a lowercase `#rrggbb[aa]` hex; `None` if the role is
    /// absent or its alias does not resolve (corrupted data degrades, never panics).
    pub fn get(&self, role: &str) -> Option<&str> {
        let palette = palette();
        let resolved = self.resolved.get_or_init(|| {
            self.raw
                .iter()
                .filter_map(|(k, v)| {
                    let hex = resolve_color(v, palette)?;
                    Some((k.clone(), hex))
                })
                .collect()
        });
        resolved.get(role).map(|s| s.as_str())
    }

    /// Every role name defined for this mode.
    pub fn roles(&self) -> impl Iterator<Item = &str> {
        self.raw.keys().map(|s| s.as_str())
    }
}

/// Resolve a color value: `#hex` (normalized lowercase) or `$alias` into the palette.
fn resolve_color(value: &str, palette: &Palette) -> Option<String> {
    if let Some(alias) = value.strip_prefix('$') {
        return palette.get(alias).cloned();
    }
    if hex_ok(value) {
        return Some(value.to_ascii_lowercase());
    }
    None
}

/// Parsed `colors.toml`.
struct ColorsParsed {
    palette: Palette,
    themes: BTreeMap<Mode, ThemeColors>,
}

fn colors_parsed() -> &'static ColorsParsed {
    static FILE: OnceLock<ColorsParsed> = OnceLock::new();
    FILE.get_or_init(|| {
        let file: ColorsFile = toml::from_str(COLORS_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/colors.toml failed to parse: {e}"));
        let mut palette = Palette::new();
        // neutral/alpha keys are already full aliases (neutral-925, white-a8, black-a60)
        for (k, v) in file.palette.neutral.iter().chain(&file.palette.alpha) {
            palette.insert(k.to_ascii_lowercase(), v.to_ascii_lowercase());
        }
        for (hue, steps) in &file.palette.hue {
            for (step, hex) in steps {
                palette.insert(format!("{hue}-{step}"), hex.to_ascii_lowercase());
            }
        }
        let themes = file
            .theme
            .into_iter()
            .map(|(mode, groups)| (mode, ThemeColors::parse(groups)))
            .collect();
        ColorsParsed { palette, themes }
    })
}

/// The resolved color palette (`[palette.*]` flattened: `neutral-925`, `white-a8`,
/// `orange-500`, …).
pub fn palette() -> &'static Palette {
    &colors_parsed().palette
}

/// Semantic color roles of `mode` (dark is the design language's native theme).
pub fn colors(mode: Mode) -> &'static ThemeColors {
    colors_parsed().themes.get(&mode).unwrap_or_else(|| {
        panic!(
            "primary data theme/colors.toml has no theme {:?}",
            mode.as_str()
        )
    })
}

/// Parsed `spacing.toml`.
#[derive(Debug, Deserialize)]
struct SpacingFile {
    spacing: SpacingRaw,
}

#[derive(Debug, Deserialize)]
struct SpacingRaw {
    grid: u32,
    #[serde(default)]
    scale: BTreeMap<String, u32>,
    #[serde(default)]
    gap: BTreeMap<String, u32>,
    #[serde(default)]
    inset: BTreeMap<String, Inset>,
}

/// One container inset (TOKENS §04): horizontal × vertical padding, px.
#[derive(Debug, Clone, Deserialize)]
pub struct Inset {
    /// Horizontal padding, px.
    pub h_px: u32,
    /// Vertical padding, px.
    pub v_px: u32,
}

/// The spacing tokens: 2px-grid scale, gap roles, container insets.
#[derive(Debug, Clone)]
pub struct Spacing {
    /// Base unit every scale token is a multiple of (2px).
    pub grid: u32,
    /// Scale ladder (`space-0.5` = 2px … `space-20` = 80px).
    pub scale: BTreeMap<String, u32>,
    /// Gap roles by relationship (`gap-hair` … `gap-page`).
    pub gap: BTreeMap<String, u32>,
    /// Container insets (`inset-chip` … `inset-card`).
    pub inset: BTreeMap<String, Inset>,
}

/// Spacing tokens (the scale + gap roles + insets).
pub fn spacing() -> &'static Spacing {
    static TOKENS: OnceLock<Spacing> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let file: SpacingFile = toml::from_str(SPACING_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/spacing.toml failed to parse: {e}"));
        Spacing {
            grid: file.spacing.grid,
            scale: file.spacing.scale,
            gap: file.spacing.gap,
            inset: file.spacing.inset,
        }
    })
}

/// Parsed `radii.toml`.
#[derive(Debug, Deserialize)]
struct RadiiFile {
    radii: RadiiRaw,
}

#[derive(Debug, Deserialize)]
struct RadiiRaw {
    tiers: BTreeMap<String, u32>,
    #[serde(default)]
    roles: BTreeMap<String, String>,
}

/// The radius tokens: named tiers (px) + roles resolved to their tier's px.
#[derive(Debug, Clone)]
pub struct Radii {
    /// Radius tiers (`xs` … `viewport`, `full`).
    pub tiers: BTreeMap<String, u32>,
    /// Role assignments resolved to px (`control`, `card`, `dialog`, …).
    pub roles: BTreeMap<String, u32>,
}

/// Corner-radius tokens.
pub fn radii() -> &'static Radii {
    static TOKENS: OnceLock<Radii> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let file: RadiiFile = toml::from_str(RADII_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/radii.toml failed to parse: {e}"));
        let roles = file
            .radii
            .roles
            .iter()
            .filter_map(|(role, tier)| file.radii.tiers.get(tier).map(|px| (role.clone(), *px)))
            .collect();
        Radii {
            tiers: file.radii.tiers,
            roles,
        }
    })
}

/// Parsed `typography.toml` (`[font]` + `[[style]]` + `[typography.*]`).
#[derive(Debug, Deserialize)]
pub struct TypographyFile {
    font: FontStack,
    style: Vec<TextStyle>,
    #[serde(default)]
    typography: TypographyAux,
}

/// `[typography.para]` + `[typography.measure]`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TypographyAux {
    #[serde(default)]
    para: BTreeMap<String, u32>,
    #[serde(default)]
    measure: BTreeMap<String, u32>,
}

impl TypographyFile {
    /// The font stacks (`[font]`): `ui` for surfaces, `mono` for numeric/code.
    pub fn fonts(&self) -> &FontStack {
        &self.font
    }

    /// Paragraph-gap tokens (`type-body-para` …), px.
    pub fn para_gaps(&self) -> &BTreeMap<String, u32> {
        &self.typography.para
    }

    /// Measure tokens (max line length: `narrow`/`body`/`wide`), px.
    pub fn measure(&self) -> &BTreeMap<String, u32> {
        &self.typography.measure
    }
}

/// The font stacks (`[font]`).
#[derive(Debug, Clone, Deserialize)]
pub struct FontStack {
    /// UI font stack (Inter first, as drawn).
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
    /// Absolute line height, px (as drawn in the sheet).
    pub line_height_px: u32,
    /// Letter spacing, px (negative = tighter).
    pub letter_spacing_px: f32,
    /// `ui` or `mono`.
    pub font: String,
    /// Tabular figures (amounts) — aligned in columns.
    #[serde(default)]
    pub tabular: bool,
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

/// The two drawn shadow steps (`[shadow.popover]`, `[shadow.dialog]`).
#[derive(Debug, Deserialize)]
struct Shadows {
    popover: Shadow,
    dialog: Shadow,
}

/// The focus-ring spec (`[focus]` in `elevation.toml`) — color resolved to hex.
#[derive(Debug, Clone)]
pub struct FocusRing {
    /// Ring width, px (2 as drawn).
    pub ring_width_px: u32,
    /// Ring offset from the control, px (2 as drawn).
    pub ring_offset_px: u32,
    /// Ring color, resolved lowercase hex.
    pub color: String,
}

/// Elevation tokens: the two shadows, the z-index layers, the focus ring.
#[derive(Debug)]
pub struct Elevation {
    /// The two drawn shadows.
    pub shadow: ShadowsPub,
    /// Z-index layers (`flat` … `tooltip`).
    pub z: BTreeMap<String, u32>,
    /// Focus-ring spec.
    pub focus: FocusRing,
}

/// Public shadow pair (popover, dialog).
#[derive(Debug)]
pub struct ShadowsPub {
    /// Popover shadow: `0 8 24` black @ 60%.
    pub popover: Shadow,
    /// Dialog shadow: `0 16 48` black @ 60%.
    pub dialog: Shadow,
}

/// Parsed `elevation.toml`.
#[derive(Debug, Deserialize)]
struct ElevationFile {
    shadow: Shadows,
    #[serde(default)]
    z: BTreeMap<String, u32>,
    focus: FocusRaw,
}

#[derive(Debug, Deserialize)]
struct FocusRaw {
    ring_width_px: u32,
    ring_offset_px: u32,
    color: String,
}

/// Elevation tokens (`data/theme/elevation.toml`).
pub fn elevation() -> &'static Elevation {
    static TOKENS: OnceLock<Elevation> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let file: ElevationFile = toml::from_str(ELEVATION_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/elevation.toml failed to parse: {e}"));
        let color = resolve_color(&file.focus.color, palette())
            .unwrap_or_else(|| file.focus.color.to_ascii_lowercase());
        Elevation {
            shadow: ShadowsPub {
                popover: file.shadow.popover,
                dialog: file.shadow.dialog,
            },
            z: file.z,
            focus: FocusRing {
                ring_width_px: file.focus.ring_width_px,
                ring_offset_px: file.focus.ring_offset_px,
                color,
            },
        }
    })
}

/// Parsed `motion.toml`.
#[derive(Debug, Deserialize)]
struct MotionFile {
    #[serde(default)]
    duration: BTreeMap<String, u32>,
    #[serde(default)]
    easing: BTreeMap<String, EasingRaw>,
    #[serde(default)]
    delay: BTreeMap<String, u32>,
    #[serde(default)]
    skeleton: SkeletonRaw,
}

#[derive(Debug, Default, Deserialize)]
struct SkeletonRaw {
    #[serde(rename = "cycle-ms", default)]
    cycle_ms: u32,
}

#[derive(Debug, Deserialize)]
struct EasingRaw {
    bezier: [f32; 4],
}

/// Motion tokens: durations/easings/delays (ms) + the skeleton cycle.
#[derive(Debug, Clone)]
pub struct Motion {
    /// Durations, ms (`instant` … `camera`, `snap`).
    pub duration: BTreeMap<String, u32>,
    /// Easings as cubic-bezier `[x1, y1, x2, y2]` (`standard` … `camera`).
    pub easing: BTreeMap<String, [f32; 4]>,
    /// Delays, ms (`spinner`, `loading-min`, `tooltip`).
    pub delay: BTreeMap<String, u32>,
    /// One skeleton highlight sweep, ms.
    pub skeleton_cycle_ms: u32,
}

/// Motion tokens (`data/theme/motion.toml`).
pub fn motion() -> &'static Motion {
    static TOKENS: OnceLock<Motion> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let file: MotionFile = toml::from_str(MOTION_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/motion.toml failed to parse: {e}"));
        Motion {
            duration: file.duration,
            easing: file
                .easing
                .into_iter()
                .map(|(k, v)| (k, v.bezier))
                .collect(),
            delay: file.delay,
            skeleton_cycle_ms: file.skeleton.cycle_ms,
        }
    })
}

/// Parsed `icons.toml`.
#[derive(Debug, Deserialize)]
struct IconsFile {
    icons: IconsRaw,
}

#[derive(Debug, Deserialize)]
struct IconsRaw {
    set: String,
    rest: String,
    active: String,
    #[serde(rename = "hit-min-px")]
    hit_min_px: u32,
    #[serde(default)]
    size: BTreeMap<String, u32>,
}

/// Iconography tokens: the glyph set, state weights, size ladder, hit target.
#[derive(Debug, Clone)]
pub struct Icons {
    /// Glyph set (`phosphor`).
    pub set: String,
    /// Weight at rest (`regular`).
    pub rest: String,
    /// Weight for hover/pressed/selected/attention (`duotone`).
    pub active: String,
    /// Size ladder, px (`xs` … `xl`).
    pub size: BTreeMap<String, u32>,
    /// Minimum hit target, px.
    pub hit_min_px: u32,
}

/// Iconography tokens (`data/theme/icons.toml`).
pub fn icons() -> &'static Icons {
    static TOKENS: OnceLock<Icons> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let file: IconsFile = toml::from_str(ICONS_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/icons.toml failed to parse: {e}"));
        Icons {
            set: file.icons.set,
            rest: file.icons.rest,
            active: file.icons.active,
            size: file.icons.size,
            hit_min_px: file.icons.hit_min_px,
        }
    })
}

/// Parsed `layout.toml` (`[source]` + `[layout]`) — fixed shell geometry in px.
#[derive(Debug, Deserialize)]
struct LayoutFile {
    layout: Layout,
}

/// The fixed shell dimensions (`[layout]`) — px.
#[derive(Debug, Clone, Deserialize)]
pub struct Layout {
    /// Sidebar expanded width (ORGANISMS · 01).
    pub sidebar_expanded_px: u32,
    /// Sidebar collapsed-to-icons width (ORGANISMS · 01).
    pub sidebar_collapsed_px: u32,
    /// Sidebar trailing-edge divider/resize line width (ORGANISMS · 01).
    pub sidebar_divider_px: u32,
}

/// Fixed shell geometry (`data/theme/layout.toml`).
pub fn layout() -> &'static Layout {
    static TOKENS: OnceLock<Layout> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let file: LayoutFile = toml::from_str(LAYOUT_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/layout.toml failed to parse: {e}"));
        file.layout
    })
}

fn hex_ok(v: &str) -> bool {
    let h = v.strip_prefix('#').unwrap_or(v);
    (h.len() == 6 || h.len() == 8) && h.chars().all(|c| c.is_ascii_hexdigit())
}

/// Semantic roles every theme mode must define (TOKENS §02 — the sheet's full list).
const REQUIRED_ROLES: &[&str] = &[
    // surfaces
    "background",
    "surface",
    "surface-raised",
    "surface-sunken",
    "surface-overlay",
    "popover",
    "tooltip",
    "hud",
    "scrim",
    "sidebar",
    // text
    "foreground",
    "secondary-foreground",
    "muted-foreground",
    "subtle-foreground",
    "disabled-foreground",
    "placeholder",
    "sidebar-label",
    "tooltip-foreground",
    "primary-foreground",
    "link",
    "on-solid",
    // structure
    "border",
    "border-subtle",
    "border-strong",
    "divider",
    "input",
    "input-hover",
    "input-focus",
    "ring",
    "ring-accent",
    "selection",
    // interaction
    "control",
    "control-hover",
    "control-active",
    "control-selected",
    "ghost-hover",
    "ghost-active",
    "sidebar-hover",
    "sidebar-selected",
    "primary",
    "primary-hover",
    // icons
    "icon",
    "icon-muted",
    "icon-hover",
    "icon-selected",
    "icon-disabled",
    "icon-attention",
    // accent & status
    "accent",
    "accent-foreground",
    "accent-subtle",
    "success",
    "success-foreground",
    "success-subtle",
    "danger",
    "danger-foreground",
    "danger-subtle",
    "warning",
    "warning-foreground",
    "info",
    // money / time / board & loading / data
    "money-in",
    "money-out",
    "money-negative",
    "money-transfer",
    "money-pending",
    "money-forecast",
    "time-now",
    "time-past",
    "time-past-surface",
    "time-future",
    "time-future-surface",
    "board-bg",
    "board-dot",
    "board-region",
    "board-card",
    "board-connector",
    "board-note",
    "skeleton",
    "skeleton-highlight",
    "progress-track",
    "progress-fill",
    "spinner",
    "data-1",
    "data-2",
    "data-3",
    "data-4",
    "data-5",
    "data-6",
    "data-7",
    "data-8",
];

/// Structural + semantic validation (RULE-DATA-002: scope index ↔ files agree).
pub fn validate_structure() -> Result<(), PrimaryDataError> {
    let idx = index();
    if idx.meta.themes != vec![Mode::Dark, Mode::Light] {
        return Err(PrimaryDataError::Invalid(
            "theme/index.toml must ship themes = [\"dark\", \"light\"]".into(),
        ));
    }
    let pal = palette();
    for (alias, value) in pal {
        if !hex_ok(value) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/colors.toml palette.{alias} is not a #rrggbb[aa] value: {value:?}"
            )));
        }
    }
    for mode in [Mode::Dark, Mode::Light] {
        let map = colors(mode);
        for role in REQUIRED_ROLES {
            match map.get(role) {
                Some(hex) if hex_ok(hex) => {}
                _ => {
                    return Err(PrimaryDataError::Invalid(format!(
                        "theme/colors.toml theme.{}.{} must resolve to a #rrggbb[aa] value (raw {:?})",
                        mode.as_str(),
                        role,
                        map.raw(role)
                    )));
                }
            }
        }
    }
    if colors(Mode::Dark).get("accent") != colors(Mode::Light).get("accent") {
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
    for (name, v) in s.scale.iter().chain(s.gap.iter()) {
        if *v == 0 || v % s.grid != 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/spacing.toml {name}={v} must be a positive multiple of grid {}",
                s.grid
            )));
        }
    }
    for (name, inset) in &s.inset {
        if inset.h_px == 0 || inset.v_px == 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/spacing.toml inset.{name} h_px/v_px must be > 0"
            )));
        }
    }
    let r = radii();
    if r.tiers.is_empty() {
        return Err(PrimaryDataError::Invalid(
            "theme/radii.toml must define tiers".into(),
        ));
    }
    for (name, v) in &r.tiers {
        if *v == 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/radii.toml tier {name} must be > 0"
            )));
        }
    }
    for role in [
        "control", "row", "chip", "tooltip", "hud", "popover", "card", "dialog",
    ] {
        if !r.roles.contains_key(role) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/radii.toml role {role} must reference a tier"
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
        if st.size_px == 0 || st.line_height_px == 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/typography.toml style {:?} size_px/line_height_px must be > 0",
                st.id
            )));
        }
    }
    if style("body").is_none() {
        return Err(PrimaryDataError::Invalid(
            "theme/typography.toml must define the `body` text style".into(),
        ));
    }
    let e = elevation();
    for (name, shadow) in [("popover", &e.shadow.popover), ("dialog", &e.shadow.dialog)] {
        if !(0.0..=1.0).contains(&shadow.opacity) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/elevation.toml shadow.{name} opacity must be within 0..=1"
            )));
        }
    }
    if !hex_ok(&e.focus.color) {
        return Err(PrimaryDataError::Invalid(
            "theme/elevation.toml focus.color must resolve to a #rrggbb[aa] value".into(),
        ));
    }
    for layer in [
        "flat", "raised", "hud", "popover", "dialog", "toast", "tooltip",
    ] {
        if !e.z.contains_key(layer) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/elevation.toml z.{layer} must be defined"
            )));
        }
    }
    let m = motion();
    for (name, v) in m.duration.iter().chain(m.delay.iter()) {
        if *v == 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/motion.toml {name} must be > 0 ms"
            )));
        }
    }
    if m.skeleton_cycle_ms == 0 {
        return Err(PrimaryDataError::Invalid(
            "theme/motion.toml skeleton-cycle-ms must be > 0".into(),
        ));
    }
    for name in ["instant", "fast", "base", "slow", "camera"] {
        if !m.duration.contains_key(name) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/motion.toml duration.{name} must be defined"
            )));
        }
    }
    for name in ["standard", "enter", "exit", "camera"] {
        if !m.easing.contains_key(name) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/motion.toml easing.{name} must be defined"
            )));
        }
    }
    let ic = icons();
    for name in ["xs", "sm", "md", "lg", "xl"] {
        if ic.size.get(name).is_none_or(|v| *v == 0) {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/icons.toml size.{name} must be > 0"
            )));
        }
    }
    if ic.hit_min_px == 0 {
        return Err(PrimaryDataError::Invalid(
            "theme/icons.toml hit-min-px must be > 0".into(),
        ));
    }
    let l = layout();
    for (name, v) in [
        ("sidebar_expanded_px", l.sidebar_expanded_px),
        ("sidebar_collapsed_px", l.sidebar_collapsed_px),
        ("sidebar_divider_px", l.sidebar_divider_px),
    ] {
        if v == 0 {
            return Err(PrimaryDataError::Invalid(format!(
                "theme/layout.toml {name} must be > 0"
            )));
        }
    }
    if l.sidebar_collapsed_px >= l.sidebar_expanded_px {
        return Err(PrimaryDataError::Invalid(
            "theme/layout.toml sidebar_collapsed_px must be < sidebar_expanded_px".into(),
        ));
    }
    Ok(())
}

fn index() -> &'static IndexFile {
    static FILE: OnceLock<IndexFile> = OnceLock::new();
    FILE.get_or_init(|| {
        toml::from_str(INDEX_RAW)
            .unwrap_or_else(|e| panic!("primary data theme/index.toml failed to parse: {e}"))
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn theme_validates() {
        super::validate_structure().unwrap();
        let dark = super::colors(super::Mode::Dark);
        assert_eq!(dark.get("background"), Some("#111111"));
        assert_eq!(dark.get("surface"), Some("#171717"));
        assert_eq!(dark.get("popover"), Some("#222222"));
        assert_eq!(dark.get("accent"), Some("#f5823a"));
        assert_eq!(dark.get("ring-accent"), Some("#f5823a"));
        assert_eq!(dark.get("primary"), Some("#ededed"));
        assert_eq!(dark.get("data-4"), Some("#6944f0"));
        assert_eq!(dark.get("selection"), Some("#ffffff1f"));
        let light = super::colors(super::Mode::Light);
        assert_eq!(light.get("background"), Some("#ededed"));
        assert_eq!(dark.get("accent"), light.get("accent"));
        assert_eq!(
            super::palette().get("neutral-925").map(|s| s.as_str()),
            Some("#111111")
        );
        assert_eq!(
            super::palette().get("orange-500").map(|s| s.as_str()),
            Some("#f5823a")
        );
        let s = super::spacing();
        assert_eq!(s.grid, 2);
        assert_eq!(s.scale.get("space-8"), Some(&32));
        assert_eq!(s.gap.get("gap-block"), Some(&16));
        assert_eq!(s.inset.get("control").map(|i| i.h_px), Some(12));
        assert_eq!(s.inset.get("control").map(|i| i.v_px), Some(6));
        let r = super::radii();
        assert_eq!(r.tiers.get("md"), Some(&6));
        assert_eq!(r.roles.get("control"), Some(&6));
        assert_eq!(r.roles.get("card"), Some(&16));
        assert_eq!(r.roles.get("dialog"), Some(&20));
        assert_eq!(r.roles.get("chip"), Some(&999));
        let body = super::style("body").unwrap();
        assert_eq!(body.size_px, 12);
        assert_eq!(body.line_height_px, 18);
        let display = super::style("display").unwrap();
        assert_eq!(display.letter_spacing_px, -1.4);
        assert_eq!(super::typography().measure().get("body"), Some(&560));
        assert!(super::style("nonexistent").is_none());
        let e = super::elevation();
        assert_eq!(e.shadow.popover.blur_px, 24);
        assert_eq!(e.shadow.dialog.blur_px, 48);
        assert_eq!(e.z.get("tooltip"), Some(&60));
        assert_eq!(e.focus.ring_width_px, 2);
        assert_eq!(e.focus.ring_offset_px, 2);
        assert_eq!(e.focus.color, "#f5823a");
        let m = super::motion();
        assert_eq!(m.duration.get("base"), Some(&200));
        assert_eq!(m.easing.get("standard"), Some(&[0.2, 0.0, 0.0, 1.0]));
        assert_eq!(m.delay.get("tooltip"), Some(&1400));
        assert_eq!(m.skeleton_cycle_ms, 900);
        let ic = super::icons();
        assert_eq!(ic.size.get("md"), Some(&16));
        assert_eq!(ic.hit_min_px, 24);
        assert_eq!(ic.active, "duotone");
        assert_eq!(super::layout().sidebar_expanded_px, 284);
        assert_eq!(super::layout().sidebar_collapsed_px, 72);
        assert_eq!(super::layout().sidebar_divider_px, 2);
    }
}
