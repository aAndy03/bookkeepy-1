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
