// SPDX-License-Identifier: GPL-3.0-or-later
//! Theme bridge: the `data/theme/` design tokens (via the `primary-data` crate) installed
//! into the gpui-kit `Theme` — the "variables" of the Figma model (RULE-UI-005/007,
//! ADR-0017). Nothing in the UI hardcodes a color, radius or spacing value: this module
//! is the ONLY place data-theme tokens meet gpui-kit's theme seam.
//!
//! Dark is the design language's native theme; light keeps the same accent + functional
//! hues (validated by `primary-data::theme::validate_structure`).

use std::rc::Rc;

use bookkeepy_primary_data::theme::Mode;
use gpui_kit::component::theme::{Theme, ThemeConfig, ThemeMode};
use gpui_kit::{App, SharedString};

/// Build the gpui-kit [`ThemeConfig`] for `mode` from the design tokens.
fn config_for(mode: Mode) -> Rc<ThemeConfig> {
    let t = bookkeepy_primary_data::theme::colors(mode);
    let mut config = ThemeConfig::default();
    config.name = match mode {
        Mode::Dark => "Bookkeepy (dark)".into(),
        Mode::Light => "Bookkeepy (light)".into(),
    };
    config.mode = match mode {
        Mode::Dark => ThemeMode::Dark,
        Mode::Light => ThemeMode::Light,
    };
    config.radius = Some(bookkeepy_primary_data::theme::radii().control as usize);
    config.radius_lg = Some(bookkeepy_primary_data::theme::radii().card as usize);

    let hex = |v: &str| Some(SharedString::from(v.to_string()));
    // Base surfaces + text.
    config.colors.background = hex(&t.bg);
    config.colors.foreground = hex(&t.fg);
    config.colors.border = hex(&t.border);
    config.colors.input = hex(&t.border);
    config.colors.ring = hex(&t.focus_ring);
    config.colors.caret = hex(&t.fg);
    config.colors.selection = hex(&t.selection);
    config.colors.overlay = hex(&t.overlay);
    // `primary` = the brand accent (the default commit); `accent` = quiet hover tint for
    // menus/lists — matching the design language's chrome surfaces.
    config.colors.primary = hex(&t.accent);
    config.colors.primary_hover = hex(&t.accent_hover);
    config.colors.primary_active = hex(&t.accent_hover);
    config.colors.primary_foreground = hex(&t.on_accent);
    config.colors.accent = hex(&t.bg_raised);
    config.colors.accent_foreground = hex(&t.fg);
    config.colors.secondary = hex(&t.bg_chrome);
    config.colors.secondary_hover = hex(&t.bg_hover);
    config.colors.secondary_foreground = hex(&t.fg);
    config.colors.muted = hex(&t.bg_raised);
    config.colors.muted_foreground = hex(&t.fg_muted);
    // Functional hues: text on a functional surface is the design's on-accent ink.
    config.colors.danger = hex(&t.danger);
    config.colors.danger_hover = hex(&t.danger_tint);
    config.colors.danger_foreground = hex(&t.on_accent);
    config.colors.success = hex(&t.success);
    config.colors.success_hover = hex(&t.success_tint);
    config.colors.success_foreground = hex(&t.on_accent);
    config.colors.warning = hex(&t.warning);
    config.colors.warning_hover = hex(&t.warning_tint);
    config.colors.warning_foreground = hex(&t.on_accent);
    config.colors.info = hex(&t.info);
    config.colors.info_hover = hex(&t.info_tint);
    config.colors.info_foreground = hex(&t.on_accent);
    // Links + focus follow the accent.
    config.colors.link = hex(&t.accent);
    config.colors.link_hover = hex(&t.accent_hover);
    config.colors.link_active = hex(&t.accent_hover);
    // Chart palette — the design-language series.
    config.colors.chart_1 = hex(&t.chart_series_1);
    config.colors.chart_2 = hex(&t.chart_series_2);
    config.colors.chart_3 = hex(&t.chart_series_3);
    config.colors.chart_4 = hex(&t.chart_series_4);
    config.colors.chart_5 = hex(&t.chart_series_5);
    config.colors.chart_bullish = hex(&t.chart_positive);
    config.colors.chart_bearish = hex(&t.chart_negative);
    config.colors.chart_grid = hex(&t.border_subtle);
    // Shell chrome on the design ramp.
    config.colors.title_bar = hex(&t.bg_chrome);
    config.colors.title_bar_border = hex(&t.border);
    config.colors.status_bar = hex(&t.bg_chrome);
    config.colors.status_bar_border = hex(&t.border);
    config.colors.sidebar = hex(&t.bg_sheet);
    config.colors.sidebar_foreground = hex(&t.fg_muted);
    config.colors.sidebar_border = hex(&t.border);
    config.colors.popover = hex(&t.bg_sheet);
    config.colors.popover_foreground = hex(&t.fg);
    config.colors.group_box = hex(&t.bg_sheet);
    config.colors.table = hex(&t.bg);
    config.colors.table_even = hex(&t.bg_raised);
    config.colors.table_head = hex(&t.bg_raised);
    config.colors.table_head_foreground = hex(&t.fg_muted);
    config.colors.list = hex(&t.bg);
    config.colors.list_head = hex(&t.bg_raised);
    config.colors.window_border = hex(&t.border);
    Rc::new(config)
}

/// The gpui-kit `ThemeConfig` for `mode` (exposed for tests/meta inspection).
pub fn config(mode: Mode) -> Rc<ThemeConfig> {
    config_for(mode)
}

/// Install the Bookkeepy theme tokens for `mode` and refresh every window.
///
/// `apply_config` also switches the theme's mode in one step (see the gpui-kit theme
/// docs): call this AFTER `gpui_kit::init`, before opening windows.
pub fn apply(mode: Mode, cx: &mut App) {
    let theme = config(mode);
    Theme::update(cx, |t| t.apply_config(&theme));
}
