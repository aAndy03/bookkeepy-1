// SPDX-License-Identifier: GPL-3.0-or-later
//! Gallery section registry (RULE-UI-006, ADR-0017). `cargo xtask govern ui` reads THIS
//! file for the `section!(...)` ids and cross-checks them against
//! `docs/registry/gpui-kit-components.toml`, so the registry lives here even while the
//! section implementations are being rebuilt.
//!
//! STATUS: all section implementations were scrapped for the M3 gallery rebuild; the
//! inventory entries are marked `exception` ("re-demo pending") until the new
//! sheet-based sections land. Re-add a section with one `section!(module, "id", Layer)`
//! line below + the `gallery.section.<id>` keys in data/locales/{en,ro}.toml + flip the
//! component's registry entry back to `covered`.

use gpui_kit::{Context, Window};

use crate::app::Gallery;

/// Top-level sidebar group — mirrors the design language's sheet families
/// (references/designlanguage): ATOMS · 01–04, MOLECULES · 01–03, ORGANISMS · 01–02,
/// CHARTS, TEMPLATES, SYSTEM, TOKENS, and bookkeepy's custom COMPONENTS.
// Variants/macro are unused only while SECTIONS is empty during the rebuild.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// ATOMS · 01–04 sheets.
    Atoms,
    /// MOLECULES · 01–03 sheets.
    Molecules,
    /// ORGANISMS · 01–02 sheets.
    Organisms,
    /// CHARTS sheet (RULE-UI-009).
    Viz,
    /// TEMPLATES · shell, board, places.
    Templates,
    /// SYSTEM · root, theme, motion, history, selection, focus trap, popup.
    System,
    /// TOKENS sheet — the data/theme tokens rendered live.
    Tokens,
    /// COMPONENTS · bookkeepy custom + widget version history (RULE-UI-008).
    Ours,
}

impl Layer {
    /// Sidebar group title i18n key (`data/locales`).
    pub fn title_key(self) -> &'static str {
        match self {
            Layer::Atoms => "gallery.layer.atoms",
            Layer::Molecules => "gallery.layer.molecules",
            Layer::Organisms => "gallery.layer.organisms",
            Layer::Viz => "gallery.layer.viz",
            Layer::Templates => "gallery.layer.templates",
            Layer::System => "gallery.layer.system",
            Layer::Tokens => "gallery.layer.tokens",
            Layer::Ours => "gallery.layer.ours",
        }
    }
}

/// One gallery section spec: what renders in the content area when selected.
pub struct SectionSpec {
    /// Stable section id (the i18n key base for its title).
    pub id: &'static str,
    /// Sidebar group.
    pub layer: Layer,
    /// Section title i18n key (`data/locales`).
    pub title_key: &'static str,
    /// Creates the section view (lazy — created on first selection, state retained).
    pub new: fn(&mut Window, &mut Context<'_, Gallery>) -> gpui_kit::AnyView,
}

// Unused only while SECTIONS is empty during the rebuild.
#[allow(unused_macros)]
macro_rules! section {
    ($module:ident, $id:literal, $layer:ident) => {
        SectionSpec {
            id: $id,
            layer: Layer::$layer,
            title_key: concat!("gallery.section.", $id),
            new: $module::view,
        }
    };
}

/// The gallery's section list — EMPTY while the sheet-based gallery is being rebuilt.
/// Add sections here; `govern ui` validates the ids against the component inventory.
pub const SECTIONS: &[SectionSpec] = &[];

/// `Layer::sections` in registry order (sidebar groups).
pub fn sections_by_layer() -> Vec<(Layer, Vec<&'static SectionSpec>)> {
    let mut out: Vec<(Layer, Vec<&'static SectionSpec>)> = Vec::new();
    for spec in SECTIONS {
        match out.last_mut() {
            Some((layer, specs)) if *layer == spec.layer => specs.push(spec),
            _ => out.push((spec.layer, vec![spec])),
        }
    }
    out
}
