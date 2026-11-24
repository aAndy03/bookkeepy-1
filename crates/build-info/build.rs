// SPDX-License-Identifier: GPL-3.0-or-later
//! Build-time facts embedded into the binary (item 6, M0.5): version, git sha, rustc,
//! build time. Deterministic when SOURCE_DATE_EPOCH is set (reproducible release builds).

use std::process::Command;

fn git_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn rustc_version() -> String {
    Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn build_unix() -> u64 {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        })
}

fn main() {
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rerun-if-env-changed=BOOKKEEPY_CHANNEL");
    // ADR-0021 (RULE-CHANNEL-001): the channel is fixed at BUILD time.
    // `cargo xtask dev` builds with BOOKKEEPY_CHANNEL=dev; release.yml builds with
    // BOOKKEEPY_CHANNEL=release (default). Dev-only code must additionally be behind the
    // `dev-channel` cargo feature so it is not even compiled into release binaries.
    let channel = match std::env::var("BOOKKEEPY_CHANNEL").as_deref() {
        Ok("dev") => "dev",
        _ => "release",
    };
    println!("cargo:rustc-env=BOOKKEEPY_CHANNEL={channel}");
    println!(
        "cargo:rustc-env=BOOKKEEPY_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
    println!("cargo:rustc-env=BOOKKEEPY_GIT_SHA={}", git_sha());
    println!("cargo:rustc-env=BOOKKEEPY_RUSTC={}", rustc_version());
    println!("cargo:rustc-env=BOOKKEEPY_BUILD_UNIX={}", build_unix());
}
