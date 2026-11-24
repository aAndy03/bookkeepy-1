// SPDX-License-Identifier: GPL-3.0-or-later
//! Public API of `bookkeepy-profile` (RULE-SSOT-001).

mod registry;
mod roots;

pub use registry::{ProfileError, ProfileMeta, Registry};
pub use roots::{ProfileRoots, app_data_dir};
