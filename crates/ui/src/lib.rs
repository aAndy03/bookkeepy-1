// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-ui` — the ONE component set: gpui-kit styled components arranged by Atomic
//! Design, with Bookkeepy variants = capability-trait toggles linked to their SSOT source
//! (ADR-0017, doc 04 §4.9). Gallery-first: every component is built and approved in the
//! `gallery` binary before app use; the gallery is a harness over this crate, never a
//! second copy (RULE-UI-006).
//!
//! Theme: design tokens from `data/theme/` (loaded via the `primary-data` crate — never
//! hardcoded); strings: `data/locales` via the `i18n` crate; shortcuts: `data/shortcuts.toml`.
//! The UI layer talks only to `app-services::api`, the data layers, and `contracts`
//! (RULE-UI-004).

pub mod api;
pub mod assets;
pub mod capabilities;
pub mod language;
pub mod theme;
pub mod widgets;
