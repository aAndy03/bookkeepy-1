#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! `cargo xtask` — the repo governor tree (RULE-GOV-001, doc 04 §4.11).
//!
//! Usage: `cargo xtask govern` runs all governors; `cargo xtask govern <name>` runs one:
//! `data` `errors` `adr` `docs` `deps` `net` `lints` `perf` `ui` `meta`.
//! Exit code is non-zero if any governor fails; CI runs the whole tree.

mod govern;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("govern") => {
            let sub = args.get(1).map(String::as_str);
            govern::run(sub)
        }
        _ => {
            eprintln!(
                "usage: cargo xtask govern [data|errors|adr|docs|deps|net|lints|perf|ui|meta]"
            );
            ExitCode::from(2)
        }
    }
}
