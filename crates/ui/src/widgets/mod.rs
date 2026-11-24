// SPDX-License-Identifier: GPL-3.0-or-later
//! Widget layers (Atomic Design, ADR-0017): `atoms` = our wrappers over gpui-kit styled
//! components; `molecules` = composed controls; `organisms` = feature blocks.
//! Every component folder carries a `meta.toml` (id, version, capability traits,
//! variants, edge cases, changelog — RULE-UI-008) and the gallery renders the version
//! history. Variants are `variant_*.rs` files extending the source — never copies.

pub mod atoms;
pub mod molecules;
pub mod organisms;
