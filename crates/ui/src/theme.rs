// SPDX-License-Identifier: GPL-3.0-or-later
//! Theme bridge: the `data/theme/` design tokens (via the `primary-data` crate) installed
//! into the gpui-kit `Theme` — the "variables" of the Figma model (RULE-UI-005/007,
//! ADR-0017). Nothing in the UI hardcodes a color, radius or spacing value: this module
//! is the ONLY place data-theme tokens meet gpui-kit's theme seam.
//!
//! Token names are the TOKENS · Bookkeepy v1 sheet's semantic roles (`background`,
//! `control`, `primary`, `ring-accent`, `data-1` …); raw primitives come from the
//! palette aliases (`white-a8`, `cyan-900` …). Dark is the designed native theme; light
//! mirrors the ramp with identical hues (validated by `primary-data::theme::validate_structure`).

use std::rc::Rc;

use bookkeepy_primary_data::theme::{Mode, colors, palette, radii, style};
use gpui_kit::component::theme::{Theme, ThemeConfig, ThemeMode};
use gpui_kit::{App, SharedString};

/// Build the gpui-kit [`ThemeConfig`] for `mode` from the design tokens.
fn config_for(mode: Mode) -> Rc<ThemeConfig> {
    let t = colors(mode);
    // Resolve a semantic role to a config string seat.
    let role = |name: &str| -> Option<SharedString> {
        t.get(name).map(|s| SharedString::from(s.to_string()))
    };
    // Resolve a raw palette alias (white-a8, cyan-900 …).
    let prim = |alias: &str| -> Option<SharedString> {
        palette()
            .get(alias)
            .map(|s| SharedString::from(s.to_string()))
    };
    // The design draws no borders on inputs' sunken well except the `input` border role;
    // fully transparent seat for the separators the design avoids.
    let transparent = || Some(SharedString::from("#00000000"));

    let mut config = ThemeConfig::default();
    config.name = match mode {
        Mode::Dark => "Bookkeepy (dark)".into(),
        Mode::Light => "Bookkeepy (light)".into(),
    };
    config.mode = match mode {
        Mode::Dark => ThemeMode::Dark,
        Mode::Light => ThemeMode::Light,
    };
    // Base type scale: the sheet's `body` style is the 12px root (TOKENS §03).
    if let Some(body) = style("body") {
        config.font_size = Some(body.size_px as f32);
    }
    if let Some(mono) = style("mono") {
        config.mono_font_size = Some(mono.size_px as f32);
    }
    let fonts = bookkeepy_primary_data::theme::typography().fonts().clone();
    config.font_family = Some(SharedString::from(fonts.ui));
    config.mono_font_family = Some(SharedString::from(fonts.mono));
    // Radii roles: control 6 / dialog 20 / card 16 (TOKENS §05 roles).
    if let Some(r) = radii().roles.get("control") {
        config.radius = Some(*r as usize);
    }
    if let Some(r) = radii().roles.get("dialog") {
        config.radius_lg = Some(*r as usize);
    }
    if let Some(r) = radii().roles.get("card") {
        config.window_radius = Some(*r as usize);
    }
    // The design is FLAT — cards/HUD carry no shadow; only popovers & dialogs float
    // (TOKENS §06), so the global shadow switch stays off.
    config.shadow = Some(false);

    // Base surfaces + text.
    config.colors.background = role("background");
    config.colors.foreground = role("foreground");
    config.colors.border = role("border");
    // Input: sunken-well fill is applied per-control by the language presets; the `input`
    // seat is the border role (#333 at rest), focus ring is the accent (ring 2, offset 2).
    config.colors.input = role("input");
    config.colors.ring = role("ring");
    config.colors.selection = role("selection");
    config.colors.overlay = role("scrim");
    // `primary` is the INVERTED neutral (light fill + dark ink on dark), not the brand
    // hue (TOKENS §02: primary = neutral-100, primary-hover = neutral-0). The brand
    // orange lives on ring-accent / indicators only.
    config.colors.primary = role("primary");
    config.colors.primary_hover = role("primary-hover");
    config.colors.primary_active = role("primary-active");
    config.colors.primary_foreground = role("primary-foreground");
    // `accent` = the white @ 8% overlay: menu item hover/selected, sidebar active plate.
    config.colors.accent = role("sidebar-selected");
    config.colors.accent_foreground = role("foreground");
    config.colors.caret = role("primary-hover");
    // Neutral controls: the flat `control` fill with its hover/active ladder; ink is the
    // `icon` role (neutral-200) — quiet at rest, white on hover via the component states.
    config.colors.button = role("control");
    config.colors.button_hover = role("control-hover");
    config.colors.button_active = role("control-active");
    config.colors.button_foreground = role("icon");
    config.colors.secondary = role("control");
    config.colors.secondary_hover = role("control-hover");
    config.colors.secondary_active = role("control-active");
    config.colors.secondary_foreground = role("foreground");
    // The button_secondary* seats must be set explicitly — unset, gpui falls back to the
    // shadcn light-gray secondary and secondary buttons render wrong on dark.
    config.colors.button_secondary = role("control");
    config.colors.button_secondary_hover = role("control-hover");
    config.colors.button_secondary_active = role("control-active");
    config.colors.button_secondary_foreground = role("foreground");
    // The primary button INVERTS — the only button that does (ATOMS · 01 primary row).
    config.colors.button_primary = role("primary");
    config.colors.button_primary_hover = role("primary-hover");
    // Pressed/selected steps back down (ATOMS · 01 matrix: rest #EDEDED → hover #FFFFFF
    // → pressed/selected #D1D1D1), with the focus ring on `ring`.
    config.colors.button_primary_active = role("primary-active");
    config.colors.button_primary_foreground = role("primary-foreground");
    config.colors.muted = role("surface-sunken");
    config.colors.muted_foreground = role("muted-foreground");
    // Functional hues: solid fills carry `on-solid` ink (white); the soft variant buttons
    // sit on the hue wells (900) with the light tint (300) as ink; pressed goes solid.
    config.colors.danger = role("danger");
    config.colors.danger_hover = role("danger-foreground");
    config.colors.danger_active = role("danger");
    config.colors.danger_foreground = role("on-solid");
    config.colors.success = role("success");
    config.colors.success_hover = role("success-foreground");
    config.colors.success_active = role("success");
    config.colors.success_foreground = role("on-solid");
    config.colors.warning = role("warning");
    config.colors.warning_hover = role("warning-foreground");
    config.colors.warning_active = role("warning");
    config.colors.warning_foreground = role("on-solid");
    config.colors.info = role("info");
    config.colors.info_hover = prim("cyan-300");
    config.colors.info_active = role("info");
    config.colors.info_foreground = role("on-solid");
    config.colors.button_danger = role("danger-subtle");
    config.colors.button_danger_hover = role("danger-subtle");
    config.colors.button_danger_active = role("danger");
    config.colors.button_danger_foreground = role("danger-foreground");
    config.colors.button_success = role("success-subtle");
    config.colors.button_success_hover = role("success");
    config.colors.button_success_active = role("success");
    config.colors.button_success_foreground = role("success-foreground");
    config.colors.button_warning = prim("amber-900");
    config.colors.button_warning_hover = role("warning");
    config.colors.button_warning_active = role("warning");
    config.colors.button_warning_foreground = role("warning-foreground");
    config.colors.button_info = prim("cyan-900");
    config.colors.button_info_hover = role("info");
    config.colors.button_info_active = role("info");
    config.colors.button_info_foreground = prim("cyan-300");
    // Links (ATOMS · 01): rest = underlined secondary text, hover rises to the `link`
    // role (white on dark) with a duotone glyph, pressed = muted. Never accent-colored.
    config.colors.link = role("secondary-foreground");
    config.colors.link_hover = role("link");
    config.colors.link_active = role("muted-foreground");
    // Chart palette — the sheet's data series (data-1..5 on the first five seats).
    config.colors.chart_1 = role("data-1");
    config.colors.chart_2 = role("data-2");
    config.colors.chart_3 = role("data-3");
    config.colors.chart_4 = role("data-4");
    config.colors.chart_5 = role("data-5");
    config.colors.chart_bullish = role("money-in");
    config.colors.chart_bearish = role("money-negative");
    config.colors.chart_grid = role("divider");
    // Shell chrome: title bar + sidebar ride the base background; status bar too, with a
    // hairline. Active nav = white @ 8% plate. Window corners follow the card radius.
    config.colors.title_bar = role("background");
    config.colors.title_bar_border = role("border-subtle");
    config.colors.status_bar = role("background");
    config.colors.status_bar_border = role("border-subtle");
    config.colors.sidebar = role("sidebar");
    config.colors.sidebar_foreground = role("sidebar-label");
    config.colors.sidebar_border = role("border-strong");
    config.colors.sidebar_accent = role("sidebar-selected");
    config.colors.sidebar_accent_foreground = role("foreground");
    config.colors.sidebar_primary = role("sidebar-selected");
    config.colors.sidebar_primary_foreground = role("foreground");
    // Dropdown/menu/calendar panels float on the popover surface (#222222 on dark).
    config.colors.popover = role("popover");
    config.colors.popover_foreground = role("foreground");
    config.colors.group_box = role("surface");
    config.colors.group_box_foreground = role("foreground");
    // Tables/lists: transparent rows — the design avoids hard separators; row states are
    // the white overlays (hover @4%, selected @12% + light indicator).
    config.colors.table = transparent();
    config.colors.table_even = transparent();
    // O6 master: the header row is transparent with a hairline bottom border, not a filled bar.
    config.colors.table_head = transparent();
    config.colors.table_head_foreground = role("muted-foreground");
    config.colors.table_foot = transparent();
    config.colors.table_foot_foreground = role("muted-foreground");
    config.colors.table_active = role("control-selected");
    config.colors.table_hover = role("ghost-hover");
    config.colors.table_row_border = transparent();
    config.colors.table_active_border = role("icon");
    config.colors.list = role("surface");
    config.colors.list_head = role("control");
    config.colors.list_active = role("control-selected");
    config.colors.list_hover = role("ghost-hover");
    config.colors.list_even = transparent();
    config.colors.list_active_border = role("icon");
    // Scrollbar: white @ 4% track, white @ 40% thumb (ATOMS · 03).
    config.colors.scrollbar = role("ghost-hover");
    config.colors.scrollbar_thumb = prim("white-a40");
    config.colors.scrollbar_thumb_hover = prim("white-a40");
    // Skeleton blocks ride the skeleton roles (TOKENS §02 board & loading).
    config.colors.skeleton = role("skeleton");
    // Switch off: `input` border-role track + muted knob; on reads `primary` (inverted).
    config.colors.switch = role("input");
    config.colors.switch_thumb = role("secondary-foreground");
    // Slider: the fill/thumb are the inverted neutral (TOKENS §02 progress-fill = primary).
    config.colors.slider_bar = role("progress-fill");
    config.colors.slider_thumb = role("progress-fill");
    // Tabs: quiet labels, active tab = ghost-hover chip.
    config.colors.tab = role("background");
    config.colors.tab_bar = role("background");
    config.colors.tab_bar_segmented = role("control");
    config.colors.tab_foreground = role("muted-foreground");
    config.colors.tab_active = role("ghost-hover");
    config.colors.tab_active_foreground = role("foreground");
    // Drag/drop + progress: drag outline is the light ink hairline; drop target keeps the
    // accent well; neutral progress fill is the inverted neutral.
    config.colors.drag_border = role("icon");
    config.colors.drop_target = role("accent-subtle");
    config.colors.progress_bar = role("progress-fill");
    config.colors.accordion = role("background");
    config.colors.window_border = role("border");
    config.colors.description_list_label = role("muted-foreground");
    // Gap seats closed by the design-language modifier pass (audit of gpui-kit 0.11.0
    // ThemeConfig): search-highlight ink, description-list label ink, group-box legend.
    config.colors.description_list_label_foreground = role("secondary-foreground");
    config.colors.list_highlight_foreground = role("info");
    config.colors.group_box_title_foreground = role("foreground");
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
    // The style modifier pass: seats ThemeConfig cannot express (motion, scrollbar
    // mode, focus ring) — still token-driven, still the single install path.
    Theme::update(cx, |t| design_language_modifier(t, mode));
    // The raw-hue slots (icon color roles, rating stars, color-picker swatches) are
    // private in `ThemeConfig` (serde-only `base.*`); set them on the resolved theme.
    // Design hues, not tailwind's red-600/green-600/yellow-600.
    let t = colors(mode);
    let p = palette();
    let hsla = |v: Option<&str>| {
        v.and_then(|v| gpui_kit::component::theme::try_parse_color(v).ok())
            .unwrap_or_default()
    };
    Theme::update(cx, |th| {
        th.red = hsla(t.get("danger"));
        th.red_light = hsla(t.get("danger-foreground"));
        th.green = hsla(t.get("success"));
        th.green_light = hsla(t.get("success-foreground"));
        th.blue = hsla(t.get("info"));
        th.blue_light = hsla(p.get("cyan-300").map(|s| s.as_str()));
        th.yellow = hsla(t.get("warning"));
        th.yellow_light = hsla(t.get("warning-foreground"));
        th.magenta = hsla(p.get("violet-500").map(|s| s.as_str()));
        th.magenta_light = hsla(p.get("violet-300").map(|s| s.as_str()));
        th.cyan = hsla(p.get("cyan-500").map(|s| s.as_str()));
        th.cyan_light = hsla(p.get("cyan-300").map(|s| s.as_str()));
    });
}

/// The design-language **style modifier** for gpui-kit primitives (RULE-UI-005/007):
/// everything `ThemeConfig` cannot express is set here on the resolved [`Theme`] —
/// motion tokens from `data/theme/motion.toml`, the scrollbar mode, and the focus-ring
/// switch. Called by [`apply`] on every install, so the app and the gallery share it.
pub fn design_language_modifier(theme: &mut Theme, _mode: Mode) {
    use gpui_kit::base::motion::Easing;
    use gpui_kit::component::scroll::ScrollbarMode;
    use std::time::Duration;

    let m = bookkeepy_primary_data::theme::motion();
    let dur = |name: &str| -> Duration {
        Duration::from_millis(m.duration.get(name).copied().unwrap_or(200) as u64)
    };
    let bez = |name: &str| -> Easing {
        let b = m.easing.get(name).copied().unwrap_or([0.2, 0.0, 0.0, 1.0]);
        Easing::cubic_bezier(b[0], b[1], b[2], b[3]).unwrap_or(Easing::Linear)
    };
    // TOKENS §10: instant 80 / fast 120 / base 200 / slow 320 map onto the kit's
    // instant/fast/normal/slow ladder; enter/exit/standard easings from the sheet.
    theme.motion.duration_instant = dur("instant");
    theme.motion.duration_fast = dur("fast");
    theme.motion.duration_normal = dur("base");
    theme.motion.duration_slow = dur("slow");
    theme.motion.easing_enter = bez("enter");
    theme.motion.easing_exit = bez("exit");
    theme.motion.easing_move = bez("standard");
    // O18: scrollbar idle-hidden, appears while scrolling (never on desktop).
    theme.scrollbar_mode = ScrollbarMode::Scrolling;
    // Design focus ring: 2px in the ring/accent color — geometry is per-component;
    // the global switch stays on so every focusable primitive shows one.
    theme.focus_ring = true;
    // ORGANISMS · 01 O5: toasts are 360px wide, stack bottom-right, max 3 visible.
    theme.notification.placement = gpui_kit::Anchor::BottomRight;
    theme.notification.max_items = 3;
    theme.notification.width = gpui_kit::px(360.);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The modifier must make the resolved theme match the tokens for BOTH modes
    /// (RULE-DATA-003: tokens are the only source; dark is native, light mirrors).
    #[test]
    fn bridge_maps_core_seats_to_tokens_both_modes() {
        for mode in [Mode::Dark, Mode::Light] {
            let cfg = config(mode);
            let t = colors(mode);
            let eq = |seat: &Option<SharedString>, role_name: &str| {
                let want = t.get(role_name).map(|s| s.to_string());
                assert_eq!(
                    seat.as_ref().map(|s| s.to_string()),
                    want,
                    "seat != token role {role_name} in {mode:?}"
                );
            };
            eq(&cfg.colors.background, "background");
            eq(&cfg.colors.foreground, "foreground");
            eq(&cfg.colors.primary, "primary");
            eq(&cfg.colors.primary_foreground, "primary-foreground");
            eq(&cfg.colors.popover, "popover");
            eq(&cfg.colors.sidebar_accent, "sidebar-selected");
            eq(&cfg.colors.table_active, "control-selected");
            eq(&cfg.colors.list_hover, "ghost-hover");
            eq(&cfg.colors.chart_1, "data-1");
            eq(&cfg.colors.chart_bullish, "money-in");
            eq(&cfg.colors.danger, "danger");
            eq(&cfg.colors.success, "success");
            // Gap seats closed by the modifier pass.
            eq(&cfg.colors.list_highlight_foreground, "info");
            eq(
                &cfg.colors.description_list_label_foreground,
                "secondary-foreground",
            );
            eq(&cfg.colors.group_box_title_foreground, "foreground");
        }
    }

    #[test]
    fn motion_modifier_uses_token_durations() {
        // The modifier reads motion.toml — assert the mapping lands (base=200ms).
        let m = bookkeepy_primary_data::theme::motion();
        assert_eq!(m.duration.get("base").copied(), Some(200));
        assert_eq!(m.duration.get("slow").copied(), Some(320));
        assert!(m.easing.contains_key("enter"));
    }
}
