// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-build-info` — build-time facts + license text (item 6 + item 11, M0.5).
//! Every build embeds its version, git sha, toolchain and the GPL text so the About
//! screen and diagnostic bundles can show exactly what build is running (RULE-ERR-003:
//! fatal errors are reportable). No dependencies by design.

/// Crate version from the workspace (`CARGO_PKG_VERSION`).
pub const VERSION: &str = env!("BOOKKEEPY_VERSION");
/// Short git sha of the build, or `"unknown"` outside a git checkout.
pub const GIT_SHA: &str = env!("BOOKKEEPY_GIT_SHA");
/// `rustc --version` string of the toolchain that built the binary.
pub const RUSTC: &str = env!("BOOKKEEPY_RUSTC");
/// Unix seconds at build time (SOURCE_DATE_EPOCH when set, else build time).
pub fn build_unix() -> u64 {
    env!("BOOKKEEPY_BUILD_UNIX").parse().unwrap_or(0)
}
/// Canonical source repository (ADR-0001).
pub const SOURCE_URL: &str = "https://github.com/aAndy03/bookkeepy-1";
/// Build channel, fixed at build time by `BOOKKEEPY_CHANNEL` (ADR-0021): `"dev"` or
/// `"release"`. The About screen and diagnostic bundles show it; the updater never
/// crosses channels.
pub const CHANNEL: &str = env!("BOOKKEEPY_CHANNEL");

/// Whether this is a dev-channel build (dev-only tooling like perf overlays is allowed
/// to exist here; release builds must never show it — RULE-CHANNEL-001).
pub fn is_dev() -> bool {
    CHANNEL == "dev"
}
/// Full GPL-3.0-or-later text (item 11): embedded so the app can show it offline
/// and every release artifact carries it. Licensed GPL-3.0-or-later (root LICENSE).
pub const LICENSE_TEXT: &str = include_str!("../../../LICENSE");

/// One-line identity used in logs, window titles and diagnostic bundles.
pub fn full() -> String {
    format!("Bookkeepy 1 v{VERSION} ({GIT_SHA}, {CHANNEL}) rustc {RUSTC}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_is_gpl3() {
        assert!(LICENSE_TEXT.contains("GNU GENERAL PUBLIC LICENSE"));
        assert!(LICENSE_TEXT.contains("Version 3"));
    }

    #[test]
    fn full_contains_version_and_source() {
        let f = full();
        assert!(f.contains(VERSION));
        assert!(f.contains(GIT_SHA));
        assert!(f.contains(CHANNEL));
        assert!(CHANNEL == "dev" || CHANNEL == "release");
        assert_eq!(SOURCE_URL, "https://github.com/aAndy03/bookkeepy-1");
    }
}
