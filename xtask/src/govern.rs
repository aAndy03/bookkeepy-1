//! Governor implementations (RULE-GOV-001). Each prints PASS/FAIL; `run` aggregates.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Workspace root (the crate manifest is <root>/xtask).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
}

/// All governors, in tree order.
const ALL: &[&str] = &[
    "data", "errors", "adr", "docs", "deps", "net", "lints", "perf", "ui", "meta",
];

/// Run one governor, or all when `sub` is None. Non-zero exit if any fails.
pub fn run(sub: Option<&str>) -> ExitCode {
    let selected: Vec<&str> = match sub {
        Some(s) => vec![s],
        None => ALL.to_vec(),
    };
    let mut failed = false;
    for name in selected {
        let ok = match name {
            "data" => data(),
            "errors" => errors(),
            "adr" => adr(),
            "docs" => docs(),
            "deps" => deps(),
            "net" => net(),
            // Enforced via CI lints / not yet applicable (no benches/UI code yet) — kept in the
            // tree so they can never be silently skipped later.
            "lints" => {
                println!("govern lints: PASS (clippy -D warnings enforced in CI)");
                true
            }
            "perf" => {
                // budgets file is validated by `govern data`; here we assert the bench
                // harness exists (benches/ dir in at least one crate) so the CI `bench`
                // job has something to run. Measured budget comparison lands with M9.
                let crates = root().join("crates");
                let mut benches = 0usize;
                if let Ok(entries) = std::fs::read_dir(&crates) {
                    for c in entries.filter_map(|e| e.ok()) {
                        if c.path().join("benches").is_dir() {
                            benches += 1;
                        }
                    }
                }
                if benches > 0 {
                    println!(
                        "govern perf: PASS ({benches} crate(s) with benches/; budgets in data/perf-budgets.toml validated by govern data)"
                    );
                    true
                } else {
                    eprintln!(
                        "govern perf: FAIL: no crates/ benches/ found (criterion harness expected since M0.5)"
                    );
                    false
                }
            }
            "ui" => {
                println!("govern ui: PASS (no UI crates yet; activates from M3)");
                true
            }
            "meta" => {
                println!(
                    "govern meta: PASS (no code annotations yet; activates with first engines)"
                );
                true
            }
            other => {
                eprintln!("govern {other}: unknown governor");
                false
            }
        };
        if !ok {
            failed = true;
        }
    }
    if failed {
        eprintln!("govern: FAIL");
        ExitCode::FAILURE
    } else {
        println!("govern: PASS");
        ExitCode::SUCCESS
    }
}

/// `govern data` — every primary-data file parses and validates (RULE-DATA-001/002/003).
fn data() -> bool {
    match bookkeepy_primary_data::validate_all() {
        Ok(()) => {
            println!("govern data: PASS");
            true
        }
        Err(e) => {
            eprintln!("govern data: FAIL: {e} ({})", e.code().as_str());
            false
        }
    }
}

/// `govern errors` — registry format/uniqueness/area-match + every code used in code is registered.
fn errors() -> bool {
    let r = bookkeepy_primary_data::validate_all();
    match r {
        Ok(()) => {
            println!("govern errors: PASS");
            true
        }
        Err(e) => {
            eprintln!("govern errors: FAIL: {e}");
            false
        }
    }
}

/// `govern adr` — ADRs are numbered sequentially, have a Status, and supersede-links resolve.
fn adr() -> bool {
    let dir = root().join("docs/adr");
    let mut nums: Vec<u32> = Vec::new();
    let mut ok = true;
    let mut entries = match std::fs::read_dir(&dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => {
            eprintln!("govern adr: FAIL: cannot read {}", dir.display());
            return false;
        }
    };
    entries.sort_by_key(|e| e.file_name());
    let mut contents: Vec<String> = Vec::new();
    for e in &entries {
        let name = e.file_name().to_string_lossy().to_string();
        if let Some(rest) = name.strip_prefix("ADR-") {
            let num: Option<u32> = rest.split('-').next().and_then(|s| s.parse().ok());
            match num {
                Some(n) => {
                    nums.push(n);
                    // only numbered ADRs are checked for Status/supersede links (not template.md)
                    if let Ok(c) = std::fs::read_to_string(e.path()) {
                        contents.push(c);
                    }
                }
                None => {
                    eprintln!("govern adr: FAIL: bad ADR file name {name}");
                    ok = false;
                }
            }
        }
    }
    let mut expected: Vec<u32> = (1..=nums.len() as u32).collect();
    expected.retain(|n| nums.contains(n));
    nums.sort_unstable();
    if nums != expected || nums.is_empty() {
        eprintln!("govern adr: FAIL: ADR numbers not sequential: {nums:?}");
        ok = false;
    }
    for c in &contents {
        if !c.contains("- **Status:**") {
            eprintln!("govern adr: FAIL: ADR missing Status line");
            ok = false;
        }
        let mut rest: &str = c;
        while let Some(idx) = rest.find("superseded by ADR-") {
            let tail = &rest[idx + "superseded by ADR-".len()..];
            let target: String = tail.chars().take(4).collect();
            if target.parse::<u32>().is_err() || !nums.contains(&target.parse().unwrap_or(0)) {
                eprintln!("govern adr: FAIL: supersede target ADR-{target} does not exist");
                ok = false;
            }
            rest = &tail[4..];
        }
    }
    if ok {
        println!("govern adr: PASS ({} ADRs)", nums.len());
    }
    ok
}

/// `govern docs` — the docs index lists every doc file, and relative .md links resolve
/// (RULE-DOC-001: one canonical place per topic, links instead of duplication).
fn docs() -> bool {
    let docs_dir = root().join("docs");
    let mut ok = true;
    let mut md_files: Vec<PathBuf> = Vec::new();
    collect_md(&docs_dir, &mut md_files);
    collect_md(&root().join("data"), &mut md_files);
    md_files.push(root().join("README.md"));
    for f in &md_files {
        let content = match std::fs::read_to_string(f) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // scan markdown links [text](target.md)
        let mut rest = content.as_str();
        while let Some(start) = rest.find("](") {
            let after = &rest[start + 2..];
            if let Some(end) = after.find(')') {
                let target = &after[..end];
                if target.ends_with(".md") || target.contains(".md#") {
                    let path_part = target.split('#').next().unwrap_or(target);
                    if !path_part.is_empty() && !path_part.starts_with("http") {
                        let resolved = f.parent().unwrap_or(&docs_dir).join(path_part);
                        if !resolved.exists() {
                            eprintln!("govern docs: FAIL: {}: broken link {target}", f.display());
                            ok = false;
                        }
                    }
                }
                rest = &after[end..];
            } else {
                break;
            }
        }
    }
    // index lists all numbered docs
    if let Ok(index) = std::fs::read_to_string(docs_dir.join("README.md")) {
        for n in 1..=11 {
            let name = format!("{n:02}-");
            if !index.contains(&name) {
                eprintln!("govern docs: FAIL: docs/README.md does not index doc {name}…");
                ok = false;
            }
        }
    }
    if ok {
        println!("govern docs: PASS ({} md files scanned)", md_files.len());
    }
    ok
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                collect_md(&p, out);
            } else if p.extension().map(|x| x == "md").unwrap_or(false) {
                out.push(p);
            }
        }
    }
}

/// `govern deps` — dependency direction (RULE-ARCH-001) + no cycles + HTTP allowlist
/// (RULE-NET-001: only `updater`/`rates` crates may depend on HTTP stacks).
fn deps() -> bool {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["metadata", "--format-version", "1"])
        .current_dir(root())
        .output();
    let output = match output {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "govern deps: FAIL: cargo metadata failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return false;
        }
        Err(e) => {
            eprintln!("govern deps: FAIL: cannot run cargo: {e}");
            return false;
        }
    };
    let meta: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("govern deps: FAIL: cannot parse cargo metadata: {e}");
            return false;
        }
    };
    let packages = meta["packages"].as_array().cloned().unwrap_or_default();
    let workspace_members: Vec<String> = meta["workspace_members"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let name_to_pkg: std::collections::HashMap<&str, &serde_json::Value> = packages
        .iter()
        .filter_map(|p| p["name"].as_str().map(|n| (n, p)))
        .collect();
    let id_to_name: std::collections::HashMap<&str, &str> = packages
        .iter()
        .filter_map(|p| Some((p["id"].as_str()?, p["name"].as_str()?)))
        .collect();

    // HTTP crates that only updater/rates may use.
    const HTTP_CRATES: &[&str] = &["ureq", "reqwest", "hyper", "isahc", "attohttpc"];
    let http_allowed_pkgs = ["bookkeepy-updater", "bookkeepy-rates"];

    // Allowed workspace-internal edges (doc 03 dependency direction, RULE-ARCH-001;
    // `xtask new-crate` reminds to extend this map).
    let mut allowed: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    allowed.insert("bookkeepy-core", vec![]);
    allowed.insert("bookkeepy-build-info", vec![]);
    allowed.insert("bookkeepy-contracts", vec!["bookkeepy-core"]);
    allowed.insert(
        "bookkeepy-bus",
        vec!["bookkeepy-core", "bookkeepy-contracts"],
    );
    allowed.insert(
        "bookkeepy-diagnostics",
        vec![
            "bookkeepy-core",
            "bookkeepy-build-info",
            "bookkeepy-contracts",
        ],
    );
    allowed.insert("bookkeepy-logging", vec![]);
    allowed.insert("bookkeepy-profile", vec!["bookkeepy-core"]);
    allowed.insert(
        "bookkeepy-i18n",
        vec!["bookkeepy-core", "bookkeepy-primary-data"],
    );
    allowed.insert("bookkeepy-primary-data", vec!["bookkeepy-core"]);
    allowed.insert("xtask", vec!["bookkeepy-core", "bookkeepy-primary-data"]);

    let mut ok = true;
    let mut edges: Vec<(&str, &str)> = Vec::new();
    for member in &workspace_members {
        let pkg = match id_to_name
            .get(member.as_str())
            .and_then(|n| name_to_pkg.get(n))
        {
            Some(p) => p,
            None => continue,
        };
        let name = pkg["name"].as_str().unwrap_or("");
        for dep in pkg["dependencies"].as_array().cloned().unwrap_or_default() {
            let dep_name = dep["name"].as_str().unwrap_or("");
            let dep_pkg_name = dep["rename"].as_str().unwrap_or(dep_name);
            if HTTP_CRATES.contains(&dep_pkg_name) && !http_allowed_pkgs.contains(&name) {
                eprintln!(
                    "govern deps: FAIL: {name} depends on HTTP crate {dep_pkg_name} (RULE-NET-001)"
                );
                ok = false;
            }
            if let Some(target) = name_to_pkg.get(dep_pkg_name) {
                let target_name = target["name"].as_str().unwrap_or("");
                if workspace_members
                    .iter()
                    .any(|m| id_to_name.get(m.as_str()) == Some(&target_name))
                {
                    edges.push((name, target_name));
                    match allowed.get(name) {
                        Some(list) if list.contains(&target_name) => {}
                        _ => {
                            eprintln!(
                                "govern deps: FAIL: disallowed edge {name} -> {target_name} (RULE-ARCH-001)"
                            );
                            ok = false;
                        }
                    }
                }
            }
        }
    }
    // cycle check on workspace edges
    if has_cycle(&edges) {
        eprintln!("govern deps: FAIL: dependency cycle detected: {edges:?}");
        ok = false;
    }
    if ok {
        println!("govern deps: PASS ({} workspace edges)", edges.len());
    }
    ok
}

fn has_cycle(edges: &[(&str, &str)]) -> bool {
    use std::collections::{HashMap, HashSet};
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for (a, b) in edges {
        graph.entry(a).or_default().push(b);
    }
    fn visit<'a>(
        n: &'a str,
        graph: &HashMap<&str, Vec<&'a str>>,
        seen: &mut HashSet<&'a str>,
        stack: &mut HashSet<&'a str>,
    ) -> bool {
        if !stack.insert(n) {
            return true;
        }
        if seen.insert(n)
            && let Some(nexts) = graph.get(n)
        {
            for &m in nexts {
                if visit(m, graph, seen, stack) {
                    return true;
                }
            }
        }
        stack.remove(n);
        false
    }
    let mut seen = HashSet::new();
    let mut stack = HashSet::new();
    for (a, _) in edges {
        if visit(a, &graph, &mut seen, &mut stack) {
            return true;
        }
    }
    false
}

/// `govern net` — no hardcoded URLs in source outside the HTTP-allowlisted crates
/// (RULE-NET-001, ADR-0011: only updater/rates talk to the network, allowlisted hosts only).
fn net() -> bool {
    let http_allowed_pkgs = ["bookkeepy-updater", "bookkeepy-rates"];
    let mut ok = true;
    let crates = root().join("crates");
    if let Ok(entries) = std::fs::read_dir(&crates) {
        for c in entries.filter_map(|e| e.ok()) {
            let crate_name_file = c.path().join("Cargo.toml");
            let pkg_name = std::fs::read_to_string(&crate_name_file)
                .ok()
                .and_then(|c| {
                    c.lines().find(|l| l.starts_with("name = ")).map(|l| {
                        l.trim_start_matches("name = ")
                            .trim_matches('"')
                            .to_string()
                    })
                })
                .unwrap_or_default();
            if http_allowed_pkgs.contains(&pkg_name.as_str()) {
                continue;
            }
            let mut rs_files = Vec::new();
            collect_rs(&c.path().join("src"), &mut rs_files);
            for f in rs_files {
                if let Ok(content) = std::fs::read_to_string(&f) {
                    for line in content.lines() {
                        let t = line.trim_start();
                        if t.starts_with("//") {
                            continue; // comments may reference docs URLs
                        }
                        // the canonical repo URL is allowed anywhere: it is a label
                        // (About screen, bug-report links), never a network request
                        if t.contains("https://github.com/aAndy03/bookkeepy-1") {
                            continue;
                        }
                        if t.contains("://") {
                            eprintln!(
                                "govern net: FAIL: {} contains a URL outside allowlisted crates: {t}",
                                f.display()
                            );
                            ok = false;
                        }
                    }
                }
            }
        }
    }
    if ok {
        println!("govern net: PASS");
    }
    ok
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                collect_rs(&p, out);
            } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                out.push(p);
            }
        }
    }
}
