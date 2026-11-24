// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask hook …` — git hooks (item 14, M0.5). `install` copies `.githooks/pre-commit`
//! into `.git/hooks`; `pre-commit` runs the fast local gates: fmt, fast governors, clippy.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Run a hook subcommand.
pub fn run(sub: Option<&str>) -> ExitCode {
    match sub {
        Some("install") => install(),
        Some("pre-commit") => pre_commit(),
        _ => {
            eprintln!("usage: cargo xtask hook <install|pre-commit>");
            ExitCode::from(2)
        }
    }
}

fn install() -> ExitCode {
    let hooks = root().join(".git/hooks");
    std::fs::create_dir_all(&hooks).expect("create .git/hooks");
    let src = root().join(".githooks/pre-commit");
    let dst = hooks.join("pre-commit");
    std::fs::copy(&src, &dst).expect("copy pre-commit hook");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755)).expect("chmod hook");
    }
    println!("hook: installed .githooks/pre-commit -> .git/hooks/pre-commit");
    ExitCode::SUCCESS
}

/// The full local gate — identical to what `ci.yml` runs on releases/manual dispatch.
/// `cargo deny check` runs only when cargo-deny is on PATH (install once:
/// `cargo install cargo-deny --locked` or a taiki-e release binary); otherwise it is
/// reported as SKIP so the gate never blocks on a missing optional tool.
pub fn full_check() -> ExitCode {
    let root = root();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let run = |args: &[&str]| {
        let status = Command::new(&cargo).args(args).current_dir(&root).status();
        matches!(status, Ok(s) if s.success())
    };
    let mut ok = true;
    // nextest when installed (round-4 item 16), cargo test as fallback
    let test_step: Vec<&str> = if Command::new("cargo")
        .args(["nextest", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        vec!["nextest", "run", "--workspace"]
    } else {
        vec!["test", "--workspace"]
    };
    let steps: Vec<Vec<&str>> = vec![
        vec!["fmt", "--all", "--check"],
        vec![
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        test_step,
        vec!["xtask", "govern"],
        vec!["bench", "--workspace", "--", "--test"],
    ];
    for step in &steps {
        let name = step.join(" ");
        if run(step) {
            println!("check: PASS {name}");
        } else {
            eprintln!("check: FAIL {name}");
            ok = false;
        }
    }
    // optional supply-chain gate when cargo-deny is installed locally
    let deny = Command::new("cargo")
        .args(["deny", "check"])
        .current_dir(&root)
        .status();
    match deny {
        Ok(s) if s.success() => println!("check: PASS cargo deny check"),
        Ok(_) => {
            eprintln!("check: FAIL cargo deny check");
            ok = false;
        }
        Err(_) => println!(
            "check: SKIP cargo deny check (cargo-deny not installed; CI runs it on releases)"
        ),
    }
    if ok {
        println!("check: PASS");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn pre_commit() -> ExitCode {
    let root = root();
    let run = |args: &[&str]| {
        let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .args(args)
            .current_dir(&root)
            .status();
        matches!(status, Ok(s) if s.success())
    };
    let mut ok = true;
    for step in [
        vec!["fmt", "--all"],
        vec!["xtask", "govern", "data"],
        vec!["xtask", "govern", "errors"],
        vec!["xtask", "govern", "adr"],
        vec!["xtask", "govern", "docs"],
        vec!["xtask", "govern", "deps"],
        vec!["xtask", "govern", "net"],
        vec![
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ] {
        let name = step.join(" ");
        if run(&step) {
            println!("pre-commit: PASS {name}");
        } else {
            eprintln!("pre-commit: FAIL {name}");
            ok = false;
        }
    }
    if ok {
        println!("pre-commit: PASS");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
