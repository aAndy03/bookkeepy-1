// SPDX-License-Identifier: GPL-3.0-or-later
//! Bundled fonts (RULE-UI-005: the design's type is Inter, and every weight its text
//! styles use — 400/500/600 — must actually render, so the app ships the variable face
//! instead of hoping the OS has it). Inter Variable 4.001, OFL — license kept beside
//! the file in `fonts/Inter-LICENSE` (same copy cosmic-text bundles as a fallback).
//!
//! Register BEFORE the first frame (`gpui` resolves fonts lazily per layout):
//! `gallery`/`app` call [`register`] right after `gpui_kit::init`, before
//! [`crate::theme::apply`] and before opening any window.

use std::borrow::Cow;

use gpui_kit::App;

/// The bundled Inter Variable face (internal family name `"Inter Variable"` — the
/// `data/theme/typography.toml` `[font] ui` stack lists it first).
pub const INTER_VARIABLE: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

/// Register the bundled font faces with the text system. Idempotent enough for startup:
/// call once per process, before any window opens. A failed registration degrades to
/// the system font stack (RULE-DATA-003 — never panic the UI over assets).
pub fn register(cx: &mut App) {
    if let Err(err) = cx
        .text_system()
        .add_fonts(vec![Cow::Borrowed(INTER_VARIABLE)])
    {
        eprintln!("bookkeepy-ui: failed to register bundled Inter Variable: {err}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bundled_inter_variable_is_a_variable_font() {
        // Variable fonts carry an `fvar` table — one file, all weights 100..900.
        assert!(super::INTER_VARIABLE.windows(4).any(|w| w == b"fvar"));
        assert!(super::INTER_VARIABLE.len() > 100_000);
    }
}
