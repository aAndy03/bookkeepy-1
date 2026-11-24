// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-logging` — local-only rotating file logs (RULE-PRIV-001, doc 03 L1).
//! `api/` is the only public surface; items are re-exported at root.

pub mod api;

pub use api::{LogError, LogGuard, init};
