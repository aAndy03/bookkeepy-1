// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-i18n` — locale lookup with English fallback over the primary-data locale
//! catalogs (item 8, M0.5). `api/` is the only public surface; items re-exported at root.

pub mod api;

pub use api::{Locale, t, t_with};
