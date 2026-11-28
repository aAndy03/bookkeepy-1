// SPDX-License-Identifier: GPL-3.0-or-later
//! Capability-trait registry (RULE-UI-007, ADR-0017): the SSOT list of capability traits.
//! Variants are built ONLY by enabling/disabling traits from this registry on the source
//! component; a widget `meta.toml` may not declare a trait that is not registered here
//! (`cargo xtask govern ui` enforces it).

/// One registered capability trait: the id referenced by widget `meta.toml` `traits` lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability {
    /// Kebab-case trait id, as used in `meta.toml`.
    pub id: &'static str,
    /// What enabling the trait changes on the source component.
    pub doc: &'static str,
}

/// The complete capability-trait registry (SSOT). New traits are ADDED here first, then
/// enabled per widget in its `meta.toml` (and toggled in code by the widget's builder).
pub const REGISTERED: &[Capability] = &[
    Capability {
        id: "algebra",
        doc: "Input evaluates arithmetic expressions (+ − × ÷); amount fields only.",
    },
    Capability {
        id: "signed",
        doc: "Amount displays an explicit +/- sign, framing debits and credits as pairs.",
    },
    Capability {
        id: "selectable",
        doc: "Table gains a checkbox column + bulk action bar (F20).",
    },
    Capability {
        id: "timeline",
        doc: "Component renders per the global time scope (RULE-TL-001) and gains a compare variant (F19).",
    },
    Capability {
        id: "compare",
        doc: "Comparison variant bound to the global time scope (enabled with `timeline`).",
    },
    Capability {
        id: "menu",
        doc: "Component hosts the shared context-menu surface (right-click / long-press / menu key, F21).",
    },
];

/// Is `id` a registered capability trait?
pub fn is_registered(id: &str) -> bool {
    REGISTERED.iter().any(|c| c.id == id)
}
