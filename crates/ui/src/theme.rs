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
    // Flat controls — the design language draws no shadows on buttons, inputs or pickers
    // (ATOMS · 01/03, MOLECULES · 01/03 spec panels); elevation starts at popovers.
    config.shadow = Some(false);

    let hex = |v: &str| Some(SharedString::from(v.to_string()));
    // Base surfaces + text.
    config.colors.background = hex(&t.bg);
    config.colors.foreground = hex(&t.fg);
    config.colors.border = hex(&t.border);
    // Inputs and picker triggers are the design's canvas wells (#0d0d0d on dark).
    config.colors.input = hex(&t.bg_canvas);
    config.colors.ring = hex(&t.focus_ring);
    config.colors.caret = hex(&t.fg);
    config.colors.selection = hex(&t.selection);
    config.colors.overlay = hex(&t.overlay);
    // `primary` = the brand accent (the default commit); `accent` = quiet hover tint for
    // menus/lists — the design's pointer-hover row (#1c1c1c pill in MOLECULES · 01).
    config.colors.primary = hex(&t.accent);
    config.colors.primary_hover = hex(&t.accent_hover);
    config.colors.primary_active = hex(&t.accent_hover);
    config.colors.primary_foreground = hex(&t.on_accent);
    config.colors.accent = hex(&t.bg_hover);
    config.colors.accent_foreground = hex(&t.fg);
    // Neutral controls (Default and Secondary buttons, selected segments): the design's
    // flat #1c1c1c control fill with its #2a2a2a hover. Their ink is quiet — ghost and
    // secondary sit at muted, rising on hover (ATOMS · 01 ghost/secondary specs).
    config.colors.button = hex(&t.bg_control);
    config.colors.button_hover = hex(&t.bg_control_hover);
    config.colors.button_active = hex(&t.bg_control_hover);
    config.colors.button_foreground = hex(&t.fg_muted);
    config.colors.secondary = hex(&t.bg_control);
    config.colors.secondary_hover = hex(&t.bg_control_hover);
    config.colors.secondary_active = hex(&t.bg_control_hover);
    config.colors.secondary_foreground = hex(&t.fg_muted);
    // The primary button INVERTS — light fill, dark ink, brightest on press; one per
    // surface, the only button that does (ATOMS · 01 primary row). The brand accent stays
    // reserved for focus, selection highlights and indicators — never the primary button.
    config.colors.button_primary = hex(&t.bg_invert);
    config.colors.button_primary_hover = hex(&t.bg_invert_hover);
    config.colors.button_primary_active = hex(&t.bg_invert_hover);
    config.colors.button_primary_foreground = hex(&t.on_invert);
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
    // Functional presses go solid: danger is calm at rest (well + tint ink) and turns
    // solid red when pressed (ATOMS · 01 "calm until touched").
    config.colors.danger_active = hex(&t.danger);
    // Soft functional buttons (Danger/Success/Warning/Info variants): the design draws
    // them on the functional wells with the tint as ink (ATOMS · 01 functional rows).
    // The hover/active seats render as overlays on the well, so they carry the raw hue.
    config.colors.button_danger = hex(&t.danger_well);
    config.colors.button_danger_hover = hex(&t.danger);
    config.colors.button_danger_active = hex(&t.danger);
    config.colors.button_danger_foreground = hex(&t.danger_tint);
    config.colors.button_success = hex(&t.success_well);
    config.colors.button_success_hover = hex(&t.success);
    config.colors.button_success_active = hex(&t.success);
    config.colors.button_success_foreground = hex(&t.success_tint);
    config.colors.button_warning = hex(&t.warning_well);
    config.colors.button_warning_hover = hex(&t.warning);
    config.colors.button_warning_active = hex(&t.warning);
    config.colors.button_warning_foreground = hex(&t.warning_tint);
    config.colors.button_info = hex(&t.info_well);
    config.colors.button_info_hover = hex(&t.info);
    config.colors.button_info_active = hex(&t.info);
    config.colors.button_info_foreground = hex(&t.info_tint);
    // Links are underlined muted text that rises to the foreground on hover — they are
    // not accent-colored (ATOMS · 01 link atom).
    config.colors.link = hex(&t.fg_muted);
    config.colors.link_hover = hex(&t.fg);
    config.colors.link_active = hex(&t.fg);
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
    // Dropdown/menu/calendar panels float on their own surface (#222222 on dark,
    // MOLECULES · 01/03), apart from cards and dialogs.
    config.colors.popover = hex(&t.bg_popover);
    config.colors.popover_foreground = hex(&t.fg);
    config.colors.group_box = hex(&t.bg_sheet);
    config.colors.table = hex(&t.bg);
    config.colors.table_even = hex(&t.bg_raised);
    config.colors.table_head = hex(&t.bg_raised);
    config.colors.table_head_foreground = hex(&t.fg_muted);
    config.colors.table_active = hex(&t.bg_selected);
    config.colors.table_active_border = hex(&t.border_subtle);
    config.colors.table_hover = hex(&t.bg_hover);
    config.colors.list = hex(&t.bg);
    config.colors.list_head = hex(&t.bg_raised);
    // Selected rows are the design's chrome fill with a quiet edge — the brand accent
    // stays out of row selection (it is focus/highlight ink only).
    config.colors.list_active = hex(&t.bg_selected);
    config.colors.list_active_border = hex(&t.border_subtle);
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
