// SPDX-License-Identifier: GPL-3.0-or-later
//! Public API of `bookkeepy-i18n` (RULE-SSOT-001).

mod errors;
mod lookup;

pub use errors::error_text;
pub use lookup::{Locale, t, t_with};
