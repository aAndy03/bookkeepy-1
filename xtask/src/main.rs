#![forbid(unsafe_code)]
#![warn(missing_docs)]
// xtask is a developer tool, not shipped code: an IO failure should abort the command
// with a non-zero exit (and a message), so expect/unwrap are allowed here only.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! `cargo xtask` — repo tooling (RULE-GOV-001, doc 04 §4.11).
//!
//! Commands:
//! - `cargo xtask govern [sub]` — the governor tree (data|errors|adr|docs|deps|net|lints|perf|ui|meta)
//! - `cargo xtask check-deps` — alias for `govern deps`
//! - `cargo xtask new-crate <name>` — scaffold a crate in the doc-04 shape (RULE-MOD-001)
//! - `cargo xtask hook install` / `cargo xtask hook pre-commit` — git hooks
//! - `cargo xtask dev|gallery|bench` — stubs; land with M3 (UI) / benches (M0.5)

mod govern;
mod hook;
mod scaffold;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("govern") => govern::run(args.get(1).map(String::as_str)),
        Some("check-deps") => govern::run(Some("deps")),
        Some("new-crate") => match args.get(1) {
            Some(name) => scaffold::new_crate(name),
            None => {
                eprintln!("usage: cargo xtask new-crate <kebab-name>");
                ExitCode::from(2)
            }
        },
        Some("hook") => hook::run(args.get(1).map(String::as_str)),
        Some("dev") => {
            println!("xtask dev: lands with M3 (the app binary) — `cargo run -p bookkeepy-app`");
            ExitCode::SUCCESS
        }
        Some("gallery") => {
            println!("xtask gallery: lands with M3 (`cargo run -p gallery`)");
            ExitCode::SUCCESS
        }
        Some("bench") => {
            println!(
                "xtask bench: `cargo bench --workspace` (harness in place since M0.5; govern perf validates budgets)"
            );
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "usage: cargo xtask <govern [sub]|check-deps|new-crate <name>|hook <install|pre-commit>|dev|gallery|bench>"
            );
            ExitCode::from(2)
        }
    }
}
