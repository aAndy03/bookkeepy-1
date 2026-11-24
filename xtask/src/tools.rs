// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask doctor` / `new-adr` / `licenses` (round-4 items 17, and B7/B8).

use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// `cargo xtask doctor` — dev-environment self-check for any contributor (CONTRIBUTING.md).
pub fn doctor() -> ExitCode {
    let mut ok = true;
    let mut check = |name: &str, good: bool, hint: &str| {
        if good {
            println!("doctor: PASS {name}");
        } else {
            println!("doctor: FAIL {name} — {hint}");
            ok = false;
        }
    };
    let rustc = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    check(
        "rustc 1.100.0 (rust-toolchain.toml pins it)",
        rustc.contains("1.100.0"),
        "install via rustup; the toolchain file selects it automatically",
    );
    check(
        "git pre-commit hook installed",
        root().join(".git/hooks/pre-commit").exists(),
        "run `cargo xtask hook install`",
    );
    // optional tools: warn only, never fail the doctor
    for (name, present, hint) in [
        (
            "cargo-deny",
            have("cargo-deny"),
            "cargo install cargo-deny --locked",
        ),
        (
            "cargo-nextest",
            have("cargo-nextest"),
            "cargo install cargo-nextest --locked",
        ),
    ] {
        if present {
            println!("doctor: PASS {name}");
        } else {
            println!("doctor: WARN {name} not installed — {hint} (optional)");
        }
    }
    check(
        "git identity set",
        Command::new("git")
            .args(["config", "user.email"])
            .current_dir(root())
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false),
        "git config user.name / user.email",
    );
    if ok {
        println!("doctor: PASS");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// `cargo xtask new-adr <slug>` — create the next ADR-NNNN-<slug>.md from the template.
pub fn new_adr(slug: &str) -> ExitCode {
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        eprintln!("new-adr: slug must be kebab-case");
        return ExitCode::from(2);
    }
    let dir = root().join("docs/adr");
    let mut max = 0u32;
    for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if let Some(rest) = name.strip_prefix("ADR-")
            && let Some(n) = rest.split('-').next().and_then(|s| s.parse::<u32>().ok())
        {
            max = max.max(n);
        }
    }
    let num = max + 1;
    let file = dir.join(format!("ADR-{num:04}-{slug}.md"));
    let template = std::fs::read_to_string(dir.join("template.md")).expect("read ADR template");
    let content = template.replace("ADR-NNNN", &format!("ADR-{num:04}"));
    std::fs::write(&file, content).expect("write ADR");
    println!(
        "new-adr: created {} (fill Status/Context/Decision; `cargo xtask govern adr` validates)",
        file.display()
    );
    ExitCode::SUCCESS
}

/// `cargo xtask licenses` — regenerate THIRD-PARTY-LICENSES.md via cargo-about when
/// installed (release.yml runs this too and attaches the file to the release).
pub fn licenses() -> ExitCode {
    if !have("cargo-about") {
        println!(
            "licenses: SKIP (cargo-about not installed; `cargo install cargo-about --locked`)"
        );
        println!("licenses: release.yml generates THIRD-PARTY-LICENSES.md on releases regardless");
        return ExitCode::SUCCESS;
    }
    let out = root().join("THIRD-PARTY-LICENSES.md");
    let status = Command::new("cargo")
        .args(["about", "generate", "about.hbs", "-o"])
        .arg(&out)
        .current_dir(root())
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("licenses: wrote {}", out.display());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("licenses: FAIL (cargo about generate failed; about.hbs present?)");
            ExitCode::FAILURE
        }
    }
}
