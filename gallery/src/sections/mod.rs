// SPDX-License-Identifier: GPL-3.0-or-later
//! Section registry — the gallery's SSOT list of what is showcased (RULE-UI-006): every
//! gpui-kit styled component (ADR-0007 "use ALL the library's styled components") plus
//! Bookkeepy's own versioned variants, arranged by atomic layers.
//!
//! `docs/registry/gpui-kit-components.toml` is the external inventory; `cargo xtask govern
//! ui` cross-checks it against this registry, so a gpui-kit upgrade updates the inventory
//! + sections together.

pub mod atoms_badge_avatar;
pub mod atoms_button;
pub mod atoms_choice;
pub mod atoms_feedback;
pub mod atoms_input;
pub mod atoms_media;
pub mod molecules_calendar;
pub mod molecules_data_list;
pub mod molecules_nav;
pub mod molecules_pickers;
pub mod molecules_popups;
pub mod molecules_structure;
pub mod organisms_overlay;
pub mod organisms_shell;
pub mod organisms_table;
pub mod organisms_workspaces;
pub mod ours;
pub mod system;
pub mod templates;
pub mod tokens;
pub mod viz;

use gpui_kit::{Context, Window};

use crate::app::Gallery;

/// Top-level sidebar group — mirrors the design language's sheet families
/// (references/designlanguage): ATOMS · 01–04, MOLECULES · 01–03, ORGANISMS · 01–02,
/// CHARTS, TEMPLATES, SYSTEM, TOKENS, and bookkeepy's custom COMPONENTS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// Our wrappers over gpui-kit styled components (ATOMS · 01–04 sheets).
    Atoms,
    /// Composed controls (MOLECULES · 01–03 sheets).
    Molecules,
    /// Feature blocks (ORGANISMS · 01–02 sheets).
    Organisms,
    /// Charts, plots, diffs — the rich, interactive visual language (CHARTS sheet, RULE-UI-009).
    Viz,
    /// Whole-app arrangements (TEMPLATES · shell, board, places).
    Templates,
    /// Cross-cutting behaviours (SYSTEM · root, theme, motion, history, selection, focus trap).
    System,
    /// The design tokens themselves, rendered from data/theme (TOKENS sheet).
    Tokens,
    /// Bookkeepy's custom components + every widget's version history (RULE-UI-008, COMPONENTS sheet).
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

/// One gallery section spec: what to render in the content area when selected.
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

/// The whole gallery: every gpui-kit styled component + our versioned variants
/// (RULE-UI-006, ADR-0007/0017). The sidebar mirrors the design language's sheets
/// (references/designlanguage/png): each section id is one sheet's component set —
/// ATOMS · 01 button/link/icon/label/kbd, ATOMS · 02 badge…slider, ATOMS · 03
/// input…scrollbar, ATOMS · 04 otp/image/rating/marker, MOLECULES · 01 popover…empty,
/// MOLECULES · 02 toggle-group…list, MOLECULES · 03 calendar…attachment,
/// ORGANISMS · 01/02, CHARTS, TEMPLATES, SYSTEM, TOKENS, COMPONENTS (ours).
/// Order = sidebar order inside each layer. Adding a sheet = one `section!` line +
/// the en/ro title keys + registry entries in docs/registry/gpui-kit-components.toml.
pub const SECTIONS: &[SectionSpec] = &[
    section!(atoms_button, "atoms-button", Atoms),
    section!(atoms_badge_avatar, "atoms-badge-avatar", Atoms),
    section!(atoms_choice, "atoms-choice", Atoms),
    section!(atoms_input, "atoms-input", Atoms),
    section!(atoms_feedback, "atoms-feedback", Atoms),
    section!(atoms_media, "atoms-media", Atoms),
    section!(molecules_popups, "molecules-popups", Molecules),
    section!(molecules_pickers, "molecules-pickers", Molecules),
    section!(molecules_calendar, "molecules-calendar", Molecules),
    section!(molecules_structure, "molecules-structure", Molecules),
    section!(molecules_data_list, "molecules-data-list", Molecules),
    section!(molecules_nav, "molecules-nav", Molecules),
    section!(organisms_shell, "organisms-shell", Organisms),
    section!(organisms_overlay, "organisms-overlay", Organisms),
    section!(organisms_table, "organisms-table", Organisms),
    section!(organisms_workspaces, "organisms-workspaces", Organisms),
    section!(viz, "viz", Viz),
    section!(templates, "templates", Templates),
    section!(system, "system", System),
    section!(tokens, "tokens", Tokens),
    section!(ours, "ours", Ours),
];

/// Layer::sections in registry order (sidebar groups).
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

/// Layout helpers every section uses for one consistent, design-language-faithful demo
/// pattern: a page of demo cards (caption + showcased variants). Per the design guides:
/// related label/control pairs closer than separate groups; no card-in-card nesting.
pub mod layout {
    use gpui_kit::component::ActiveTheme as _;
    use gpui_kit::component::h_flex;
    use gpui_kit::{
        App, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
    };

    /// A section page: one heading + demo cards.
    pub fn section_page(
        cx: &App,
        heading_key: &str,
        cards: impl IntoIterator<Item = gpui_kit::AnyElement>,
    ) -> impl IntoElement {
        let heading = crate::app::tr(cx, heading_key);
        // (heading-only page: no theme tokens needed yet)
        div()
            .id("section-page")
            .flex()
            .flex_col()
            .gap_6()
            .p_6()
            .child(div().text_xl().child(heading))
            .children(cards)
    }

    /// A demo card: caption + content (the showcased component variants). The card
    /// surface is the design's spec panel — via the ui crate's `surface::spec_panel`
    /// preset (sheet caption panels are `#171717` rx16 borderless, ATOMS/ORGANISMS/
    /// CHARTS sheets; values come from data/theme tokens, never raw).
    pub fn demo_card(cx: &App, caption_key: &str, demo: gpui_kit::AnyElement) -> impl IntoElement {
        let caption = crate::app::tr(cx, caption_key);
        let mode = cx.global::<crate::app::Prefs>().mode;
        let theme = cx.theme();
        bookkeepy_ui::api::language::surface::spec_panel(
            div().flex().flex_col().gap_3().p_4(),
            mode,
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(caption),
        )
        .child(demo)
    }

    /// A row of showcased variants (horizontal, wrapped).
    pub fn demo_row(items: impl IntoIterator<Item = gpui_kit::AnyElement>) -> impl IntoElement {
        h_flex().flex_wrap().gap_3().children(items)
    }
}
