// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-profile` — profile registry + on-disk data-dir layout (F18, doc 05 §5.2,
//! INV-PROF-001: profiles never share data). `api/` is the only public surface.

pub mod api;

pub use api::{ProfileError, ProfileMeta, ProfileRoots, Registry, app_data_dir};
