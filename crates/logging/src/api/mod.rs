// SPDX-License-Identifier: GPL-3.0-or-later
//! Public API of `bookkeepy-logging` (RULE-SSOT-001).

mod init;

pub use init::{LogError, LogGuard, init};
