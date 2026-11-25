// SPDX-License-Identifier: GPL-3.0-or-later
//! `gallery` — the component gallery binary (RULE-UI-006/008, ADR-0017).
//!
//! Showcases ALL gpui-kit styled components plus Bookkeepy's versioned variants, arranged
//! by atomic layers — a harness/viewer over the same `bookkeepy-ui` widgets crate: ONE
//! component set only, never a second copy. Every component is approved here before it
//! is used in the app.
//!
//! - Theme: design tokens from `data/theme/` via `bookkeepy-ui` (never hardcoded).
//! - Strings: `data/locales` (EN/RO) via `bookkeepy-i18n`; every input has a localized
//!   placeholder (RULE-UI-005).
//! - Shortcuts: bound from `data/shortcuts.toml` (RULE-INPUT-001) — no hardcoded keys.

mod app;
mod sections;

use gpui_kit::{AppContext as _, KeyBinding, WindowBounds, WindowOptions};

/// Quit the gallery window (bound to the remappable key from `data/shortcuts.toml`).
pub(crate) const QUIT_ACTION_ID: &str = "app.quit";

// gpui's `actions!` macro expands to an action struct; the module docs describe it,
// so `missing_docs` is silenced for the generated item only.
#[allow(missing_docs)]
mod quit_action_shim {
    gpui_kit::actions!(gallery, [Quit]);
}
pub(crate) use quit_action_shim::Quit;

/// Bind the global mappings of `data/shortcuts.toml` (RULE-INPUT-001: SSOT, remappable —
/// the gallery binds quit; the app binds every scope as screens land).
fn bind_shortcuts(cx: &mut gpui_kit::App) {
    if let Some(shortcut) = bookkeepy_primary_data::shortcuts::get(QUIT_ACTION_ID) {
        // Primary data grammar is "ctrl+q"; gpui's grammar is "ctrl-q" ("secondary-"
        // resolves per OS — remapping later never touches this code).
        let keystrokes = shortcut.keys.replace('+', "-");
        if let Ok(binding) = KeyBinding::load(&keystrokes, Box::new(Quit), None, false, None, cx) {
            cx.bind_keys([binding]);
        }
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(bookkeepy_ui::api::assets::BookkeepyAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            // Bundled Inter Variable (all weights the type styles use) before first frame.
            bookkeepy_ui::api::fonts::register(cx);
            // Design tokens → gpui-kit theme (dark = the design language's native theme).
            bookkeepy_ui::theme::apply(bookkeepy_primary_data::theme::Mode::Dark, cx);
            // Runtime prefs (locale + mode) as a gpui global; switching refreshes windows.
            cx.set_global(app::Prefs::default());
            bind_shortcuts(cx);
            // `--section <id>` deep-links a gallery section (used by the screenshot
            // harness in references/ tooling; unknown ids fall back to the first).
            let args: Vec<String> = std::env::args().skip(1).collect();
            let arg_val = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone());
            let section = arg_val("--section");
            // `--height <px>` lets the screenshot harness capture a whole section.
            let height = arg_val("--height")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(900.);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds {
                    origin: gpui_kit::point(gpui_kit::px(160.), gpui_kit::px(120.)),
                    size: gpui_kit::size(gpui_kit::px(1600.), gpui_kit::px(height)),
                })),
                titlebar: Some(gpui_kit::TitlebarOptions {
                    title: Some("Bookkeepy gallery".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            if gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| app::Gallery::new(window, cx, section.as_deref()))
            })
            .is_err()
            {
                eprintln!("gallery: failed to open window");
            }
        });
}
