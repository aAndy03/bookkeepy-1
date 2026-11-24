// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask doctor` / `new-adr` / `licenses` / `dev-reset-profile`.

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

/// `cargo xtask changelog "<line>"` — append a bullet under `## [Unreleased]` → `### Added`
/// in CHANGELOG.md (round-5 item 3: makes the PR checklist's CHANGELOG rule effortless).
pub fn changelog(line: &str) -> ExitCode {
    let path = root().join("CHANGELOG.md");
    let mut text = std::fs::read_to_string(&path).expect("read CHANGELOG");
    let anchor = "## [Unreleased]\n";
    let Some(pos) = text.find(anchor) else {
        eprintln!("changelog: no [Unreleased] section");
        return ExitCode::FAILURE;
    };
    let insert_at = pos + anchor.len();
    let bullet = format!("- {line}\n");
    if text[insert_at..].contains("### Added\n") {
        // insert right after the "### Added" line
        let added = text[insert_at..]
            .find("### Added\n")
            .map(|i| insert_at + i + "### Added\n".len());
        if let Some(at) = added {
            text.insert_str(at, &bullet);
        }
    } else {
        text.insert_str(insert_at, &format!("\n### Added\n{bullet}"));
    }
    std::fs::write(&path, text).expect("write CHANGELOG");
    println!("changelog: added under [Unreleased] → Added");
    ExitCode::SUCCESS
}

/// `cargo xtask pr "<title>"` — branch off main, push, open a PR (round-5 item 1).
/// Requires a clean tree (commit first). Uses gh; branch protection on main forces PRs.
pub fn pr(title: &str) -> ExitCode {
    if title.trim().is_empty() {
        eprintln!("usage: cargo xtask pr \"<title>\"");
        return ExitCode::from(2);
    }
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    let branch = format!(
        "auto/{slug}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root())
        .output()
        .expect("git status");
    if !status.stdout.is_empty() {
        eprintln!("pr: working tree not clean — commit first (`git add -A && git commit …`)");
        return ExitCode::FAILURE;
    }
    for args in [
        vec!["checkout", "-b", &branch],
        vec!["push", "-u", "origin", &branch],
    ] {
        let out = Command::new("git")
            .args(&args)
            .current_dir(root())
            .output()
            .expect("git");
        if !out.status.success() {
            eprintln!(
                "pr: git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
            return ExitCode::FAILURE;
        }
    }
    let out = Command::new("gh")
        .args(["pr", "create", "--title", title, "--body", "<!-- fill per .github/pull_request_template.md: cargo xtask check ran locally and green -->", "--base", "main", "--head", &branch])
        .current_dir(root())
        .output()
        .expect("gh pr create");
    if out.status.success() {
        println!("pr: {}", String::from_utf8_lossy(&out.stdout).trim());
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "pr: gh pr create failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        ExitCode::FAILURE
    }
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

/// `cargo xtask dev-reset-profile <profile-id> --data-dir <path>` — dev-channel-only
/// "revert" for development (doc 10, M1 hardening round): deletes ONE dev profile tree and
/// removes it from the registry. There are no down-migrations by design (INV-DATA-001);
/// in dev, reverting a database = deleting the disposable dev profile. Guard rails:
/// refuses on a release-channel build, on path escape, and while the profile is open
/// (instance lock held). Never touches other profiles or app-level files.
pub fn dev_reset_profile(args: &[String]) -> ExitCode {
    use bookkeepy_core::EntityId;
    use bookkeepy_profile::Registry;
    use std::str::FromStr;

    if !bookkeepy_build_info::is_dev() {
        eprintln!("dev-reset-profile: REFUSED — this is not a dev-channel build.");
        eprintln!(
            "run with: BOOKKEEPY_CHANNEL=dev cargo xtask dev-reset-profile <profile-id> --data-dir <path>"
        );
        return ExitCode::from(2);
    }
    let mut id_arg: Option<String> = None;
    let mut data_dir: Option<String> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--data-dir" => data_dir = it.next().cloned(),
            _ if id_arg.is_none() => id_arg = Some(a.clone()),
            _ => {}
        }
    }
    let (Some(id_s), Some(dd)) = (id_arg, data_dir) else {
        eprintln!("usage: cargo xtask dev-reset-profile <profile-id> --data-dir <path>");
        return ExitCode::from(2);
    };
    let Ok(id) = EntityId::from_str(&id_s) else {
        eprintln!("dev-reset-profile: invalid profile id {id_s:?}");
        return ExitCode::from(2);
    };
    let data_dir = PathBuf::from(dd);
    let mut reg = match Registry::load(&data_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dev-reset-profile: cannot load registry: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(pos) = reg.profiles.iter().position(|p| p.id == id) else {
        eprintln!("dev-reset-profile: unknown profile {id}");
        return ExitCode::FAILURE;
    };
    let meta = reg.profiles[pos].clone();
    if meta.path != format!("profiles/{id}") {
        eprintln!("dev-reset-profile: REFUSED — profile path escapes the data dir");
        return ExitCode::from(2);
    }
    let root = data_dir.join(&meta.path);
    // refuse while a running instance holds the profile lock
    let lock_path = root.join("app.lock");
    if lock_path.exists() {
        use fs2::FileExt;
        let f = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("dev-reset-profile: cannot open lock file: {e}");
                return ExitCode::FAILURE;
            }
        };
        if f.try_lock_exclusive().is_err() {
            eprintln!(
                "dev-reset-profile: REFUSED — profile is open in a running Bookkeepy; close it first"
            );
            return ExitCode::FAILURE;
        }
    }
    if let Err(e) = std::fs::remove_dir_all(&root) {
        eprintln!("dev-reset-profile: cannot delete {}: {e}", root.display());
        return ExitCode::FAILURE;
    }
    reg.profiles.remove(pos);
    if reg.active == Some(id) {
        reg.active = None;
    }
    if let Err(e) = reg.save(&data_dir) {
        eprintln!("dev-reset-profile: deleted tree but registry save failed: {e}");
        return ExitCode::FAILURE;
    }
    println!(
        "dev-reset-profile: deleted {} (profile {:?}) and removed it from the registry",
        root.display(),
        meta.name
    );
    ExitCode::SUCCESS
}
