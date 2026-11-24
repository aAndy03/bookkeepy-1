//! `cargo xtask new-crate <name>` — scaffolds a crate in the doc-04 §4.2 shape so every
//! crate is mechanically consistent (RULE-MOD-001): api/ surface, forbid-unsafe lib.rs,
//! README, workspace membership. After scaffolding, add allowed dependency edges in
//! `xtask/src/govern.rs` (`allowed` map) — `govern deps` enforces them.

use std::path::PathBuf;
use std::process::ExitCode;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Create `crates/<name>` and register it in the workspace.
pub fn new_crate(name: &str) -> ExitCode {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        eprintln!("new-crate: name must be kebab-case (a-z, 0-9, '-')");
        return ExitCode::FAILURE;
    }
    let dir = root().join("crates").join(name);
    if dir.exists() {
        eprintln!("new-crate: {} already exists", dir.display());
        return ExitCode::FAILURE;
    }
    let pkg = format!("bookkeepy-{name}");
    let src = dir.join("src");
    std::fs::create_dir_all(src.join("api")).expect("create crate dirs");

    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{pkg}\"\ndescription = \"TODO: one line — see crates/{name}/README.md\"\n\
             version.workspace = true\nedition.workspace = true\nrust-version.workspace = true\n\
             license.workspace = true\nrepository.workspace = true\nauthors.workspace = true\n\n\
             [lints]\nworkspace = true\n\n[dependencies]\n"
        ),
    )
    .expect("write Cargo.toml");
    std::fs::write(
        src.join("lib.rs"),
        format!(
            "#![forbid(unsafe_code)]\n#![warn(missing_docs)]\n\
             #![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]\n\n\
             //! `{pkg}` — TODO: what this crate owns (doc 04 §4.2). `api/` is the only public surface.\n\n\
             pub mod api;\n"
        ),
    )
    .expect("write lib.rs");
    std::fs::write(
        src.join("api/mod.rs"),
        "//! Public API of this crate — the only import other crates may use (RULE-SSOT-001).\n",
    )
    .expect("write api/mod.rs");
    std::fs::write(
        dir.join("README.md"),
        format!(
            "# {pkg}\n\nTODO: what this crate owns, its invariants (INV-…), its ADR links (doc 04 §4.2).\n"
        ),
    )
    .expect("write README.md");

    // register in workspace members
    let manifest_path = root().join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read root manifest");
    let anchor = "members = [";
    let Some(pos) = manifest.find(anchor) else {
        eprintln!("new-crate: cannot find members list in root Cargo.toml");
        return ExitCode::FAILURE;
    };
    // insert after the line break that ends the `members = [` line (LF or CRLF)
    let rest = &manifest[pos + anchor.len()..];
    let nl = rest.find('\n').map(|i| i + 1).unwrap_or(0);
    let insert_at = pos + anchor.len() + nl;
    let mut new_manifest = String::with_capacity(manifest.len() + 64);
    new_manifest.push_str(&manifest[..insert_at]);
    new_manifest.push_str(&format!("    \"crates/{name}\",\n"));
    new_manifest.push_str(&manifest[insert_at..]);
    std::fs::write(&manifest_path, new_manifest).expect("write root manifest");

    println!("new-crate: created crates/{name} ({pkg}) and registered it in the workspace");
    println!(
        "new-crate: next — add allowed dependency edges for {pkg} in xtask/src/govern.rs, then `cargo xtask govern deps`"
    );
    ExitCode::SUCCESS
}
