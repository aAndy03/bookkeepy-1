// SPDX-License-Identifier: GPL-3.0-or-later
//! Public API of this crate — the only import other crates may use (RULE-SSOT-001).
//! The gallery and the future `app` binary compose ui surfaces through this seam only.
//!
//! M3 scaffold ships: the theme bridge (`theme`), the design-language builders
//! (`language`), the bundled fonts (`fonts`), the capability registry (`capabilities`),
//! and the widget layers (`widgets`).

pub use crate::assets;
pub use crate::capabilities;
pub use crate::fonts;
pub use crate::language;
pub use crate::theme;
pub use crate::widgets;
