// SPDX-License-Identifier: GPL-3.0-or-later
//! The `amount` atom — the seed Bookkeepy widget (ADR-0017): the SSOT money display,
//! extended ONLY by capability-trait-toggled variants (never copies). `meta.toml` is the
//! widget's version registry that `cargo xtask govern ui` cross-checks and the gallery's
//! `ours` section renders as version history (RULE-UI-008).

pub mod edge_case_zero;
pub mod source;
pub mod variant_signed;

pub use source::Amount;

/// The widget's `meta.toml` (RULE-UI-008), embedded for the gallery's version-history
/// panel — parse with the `toml` crate; `govern ui` validates schema + cross-refs.
pub const META_TOML: &str = include_str!("meta.toml");
