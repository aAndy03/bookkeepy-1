// SPDX-License-Identifier: GPL-3.0-or-later
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
    "data", "errors", "events", "adr", "docs", "deps", "net", "lints", "perf", "ui", "meta",
    "seams",
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
            "events" => events(),
            "adr" => adr(),
            "docs" => docs(),
            "deps" => deps(),
            "net" => net(),
            // Enforced via CI lints / not yet applicable (no benches/UI code yet) — kept in the
            // tree so they can never be silently skipped later.
            "lints" => lints(),
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
            "meta" => meta(),
            "seams" => seams(),
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

/// `govern events` — RULE-EVT-001: event kinds are defined ONLY as consts in
/// `bookkeepy_contracts::kinds` and registered in `data/events.toml`. The const set and
/// the registry must be identical, and no other source file may contain a kind literal
/// (first arg of `EventEnvelope::new(...)` or a `kind: "..."` field).
fn events() -> bool {
    let reg: Vec<String> = match bookkeepy_primary_data::events::get() {
        Ok(list) => list.iter().map(|e| e.kind.clone()).collect(),
        Err(e) => {
            eprintln!("govern events: FAIL: registry invalid: {e}");
            return false;
        }
    };
    let kinds_path = root().join("crates/contracts/src/api/kinds.rs");
    let kinds_src = match std::fs::read_to_string(&kinds_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "govern events: FAIL: cannot read {}: {e}",
                kinds_path.display()
            );
            return false;
        }
    };
    let mut consts: Vec<String> = Vec::new();
    for line in kinds_src.lines() {
        if line.contains("pub const")
            && let Some(q1) = line.find("\"")
            && let Some(q2) = line[q1 + 1..].find("\"")
        {
            consts.push(line[q1 + 1..q1 + 1 + q2].to_string());
        }
    }
    let mut reg_sorted = reg.clone();
    reg_sorted.sort_unstable();
    let mut consts_sorted = consts.clone();
    consts_sorted.sort_unstable();
    let mut ok = true;
    if reg_sorted != consts_sorted {
        eprintln!("govern events: FAIL: kinds consts != data/events.toml registry");
        eprintln!(
            "  only in code: {:?}",
            consts_sorted
                .iter()
                .filter(|c| !reg_sorted.contains(c))
                .collect::<Vec<_>>()
        );
        eprintln!(
            "  only in registry: {:?}",
            reg_sorted
                .iter()
                .filter(|c| !consts_sorted.contains(c))
                .collect::<Vec<_>>()
        );
        ok = false;
    }
    // no kind literals outside kinds.rs
    let mut files = Vec::new();
    collect_rs(&root().join("crates"), &mut files);
    for f in &files {
        if f.file_name().is_some_and(|n| n == "kinds.rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(f) else {
            continue;
        };
        let mut rest = content.as_str();
        while let Some(idx) = rest.find("EventEnvelope::new(") {
            let tail = &rest[idx + "EventEnvelope::new(".len()..];
            let trimmed = tail.trim_start();
            if let Some(stripped) = trimmed.strip_prefix('"') {
                let lit: String = stripped.chars().take_while(|c| *c != '"').collect();
                eprintln!(
                    "govern events: FAIL: kind literal {lit:?} in {} — use bookkeepy_contracts::kinds const",
                    f.display()
                );
                ok = false;
            }
            rest = tail;
        }
        for line in content.lines() {
            let t = line.trim_start();
            if t.starts_with("kind:") && t.contains('"') {
                let q1 = t.find('"').unwrap_or(0);
                let lit: String = t[q1 + 1..].chars().take_while(|c| *c != '"').collect();
                if lit.contains('.') {
                    eprintln!(
                        "govern events: FAIL: kind literal {lit:?} in {} — use bookkeepy_contracts::kinds const",
                        f.display()
                    );
                    ok = false;
                }
            }
        }
    }
    if ok {
        println!(
            "govern events: PASS ({} kinds, registry == consts, no stray literals)",
            reg.len()
        );
    }
    ok
}

/// Lines of a file with `#[cfg(test)]` modules elided (brace-counting state machine), so
/// seam rules apply to shipped code only.
fn non_test_lines(content: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut skipping = false;
    let mut depth = 0i64;
    let mut seen_open = false;
    for line in content.lines() {
        if skipping {
            let opens = line.matches('{').count() as i64;
            let closes = line.matches('}').count() as i64;
            depth += opens - closes;
            if opens > 0 {
                seen_open = true;
            }
            if seen_open && depth <= 0 {
                skipping = false;
            }
            continue;
        }
        if line.trim() == "#[cfg(test)]" {
            skipping = true;
            depth = 0;
            seen_open = false;
            continue;
        }
        out.push(line);
    }
    out
}

/// `govern seams` — RULE-SEAM-001 (ADR-0025): the cross-layer seams are guarded
/// mechanically, so the bug classes found in the M2-hardening audit cannot recur:
/// - **S1 risky deps**: `rusqlite` only in storage/ledger/shared; `fs2` only in storage;
///   `sha2`/`hex` only in storage/ledger. No other crate may touch the DB driver.
/// - **S2 boundary error mapping**: in ledger/shared, every `tx::post`/`tx::post_with`
///   statement must route its error through `from_storage`, so engine BK-codes survive
///   the `StorageError::Foreign` closure boundary.
/// - **S3 table ownership**: a crate may only write/read its OWN tables in literal SQL.
///   storage owns meta/schema_migrations/settings/ui_state/audit_log/outbox/jobs (plus
///   CREATE TRIGGER/INDEX guard lines referencing ledger tables); ledger owns
///   accounts/categories/tags/parties/tx_tags/journal_entries/postings/transactions/
///   schedules; shared owns NONE (it acts through the `Actionable` allowlist, dynamic).
/// - **S4 connection construction**: only storage may open connections (`Db::open`,
///   `Connection::open`) outside test modules.
fn seams() -> bool {
    let crates = root().join("crates");
    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs(&crates, &mut files);
    let mut ok = true;

    // S1 — risky external deps per crate (from Cargo.toml [dependencies])
    let risky: &[(&str, &[&str])] = &[
        (
            "rusqlite",
            &["bookkeepy-storage", "bookkeepy-ledger", "bookkeepy-shared"],
        ),
        ("fs2", &["bookkeepy-storage"]),
        ("sha2", &["bookkeepy-storage", "bookkeepy-ledger"]),
        ("hex", &["bookkeepy-storage", "bookkeepy-ledger"]),
    ];
    if let Ok(entries) = std::fs::read_dir(&crates) {
        for c in entries.filter_map(|e| e.ok()) {
            let manifest = c.path().join("Cargo.toml");
            let Ok(content) = std::fs::read_to_string(&manifest) else {
                continue;
            };
            let pkg = content
                .lines()
                .find_map(|l| {
                    l.strip_prefix("name = ")
                        .map(|s| s.trim_matches('"').to_string())
                })
                .unwrap_or_default();
            for (dep, allowed) in risky {
                // a risky dep must not be listed at all (any section) in a non-allowed crate
                let used = content.lines().any(|l| {
                    let t = l.trim();
                    t.starts_with(&format!("{dep}.workspace")) || t.starts_with(&format!("{dep} ="))
                });
                if used && !allowed.contains(&pkg.as_str()) {
                    eprintln!(
                        "govern seams: FAIL (S1): {pkg} depends on {dep} but is not in the allowed set {allowed:?}"
                    );
                    ok = false;
                }
            }
        }
    }

    // ownership table for S3
    let mut owned: std::collections::HashMap<String, Vec<&str>> = std::collections::HashMap::new();
    owned.insert(
        "bookkeepy-storage".into(),
        vec![
            "meta",
            "schema_migrations",
            "settings",
            "ui_state",
            "audit_log",
            "outbox",
            "jobs",
            "sqlite_master",
        ],
    );
    owned.insert(
        "bookkeepy-ledger".into(),
        vec![
            "accounts",
            "categories",
            "tags",
            "parties",
            "tx_tags",
            "journal_entries",
            "postings",
            "transactions",
            "schedules",
        ],
    );
    owned.insert("bookkeepy-shared".into(), vec![]);

    let sql_kw = [
        "INSERT INTO ",
        "IGNORE INTO ",
        "UPDATE ",
        "DELETE FROM ",
        " FROM ",
        " JOIN ",
    ];
    for f in &files {
        // integration tests (tests/) and benches (benches/) are not shipped code
        if f.components()
            .any(|c| c.as_os_str() == "tests" || c.as_os_str() == "benches")
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(f) else {
            continue;
        };
        // crate package name: walk up from the file until a Cargo.toml is found
        let mut dir = f.parent();
        let pkg = loop {
            match dir {
                Some(d) if d.join("Cargo.toml").is_file() => {
                    break std::fs::read_to_string(d.join("Cargo.toml"))
                        .ok()
                        .and_then(|c| {
                            c.lines().find_map(|l| {
                                l.strip_prefix("name = ")
                                    .map(|s| s.trim_matches('"').to_string())
                            })
                        })
                        .unwrap_or_default();
                }
                Some(d) => dir = d.parent(),
                None => break String::new(),
            }
        };
        let lines = non_test_lines(&content);
        let joined = lines.join("\n");
        // S2 — boundary error mapping in ledger/shared
        if pkg == "bookkeepy-ledger" || pkg == "bookkeepy-shared" {
            let mut rest = joined.as_str();
            while let Some(idx) = rest
                .find("tx::post_with(")
                .or_else(|| rest.find("tx::post("))
            {
                let after = &rest[idx..];
                // closure ends at the first 4-space-indented `})`
                let end = after
                    .find("\n    })")
                    .map(|e| e + 80)
                    .unwrap_or(after.len().min(4000));
                let stmt = &after[..end.min(after.len())];
                if !stmt.contains("from_storage") {
                    eprintln!(
                        "govern seams: FAIL (S2): {} calls tx::post(_with) without from_storage error mapping",
                        f.display()
                    );
                    ok = false;
                }
                rest = &after[1..];
            }
        }
        // S3 — table ownership (skip trigger/index/raise lines and dynamic format! SQL)
        if let Some(allowed) = owned.get(&pkg) {
            for line in &lines {
                let t = line.trim_start();
                if t.starts_with("//")
                    || line.contains("CREATE TRIGGER")
                    || line.contains("CREATE INDEX")
                    || line.contains("CREATE UNIQUE")
                    || line.contains("RAISE")
                    || line.contains("format!")
                {
                    continue;
                }
                let mut rest: &str = line;
                while let Some(kw) = sql_kw.iter().find_map(|k| rest.find(k).map(|i| (i, *k))) {
                    let (i, k) = kw;
                    let after = &rest[i + k.len()..];
                    let ident: String = after
                        .chars()
                        .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                        .collect();
                    if !ident.is_empty() && !allowed.contains(&ident.as_str()) {
                        eprintln!(
                            "govern seams: FAIL (S3): {} touches table '{ident}' but owns only {allowed:?}",
                            f.display()
                        );
                        ok = false;
                    }
                    rest = after;
                }
            }
        }
        // S4 — only storage constructs connections (outside tests)
        if pkg != "bookkeepy-storage" && pkg != "xtask" {
            for line in &lines {
                if line.contains("Connection::open(") || line.contains("Db::open(") {
                    eprintln!(
                        "govern seams: FAIL (S4): {} constructs a DB connection; only bookkeepy-storage may",
                        f.display()
                    );
                    ok = false;
                }
            }
        }
    }
    if ok {
        println!(
            "govern seams: PASS (S1 risky deps, S2 boundary from_storage, S3 table ownership, S4 connection construction)"
        );
    }
    ok
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
    allowed.insert(
        "bookkeepy-shared",
        vec![
            "bookkeepy-core",
            "bookkeepy-contracts",
            "bookkeepy-bus",
            "bookkeepy-storage",
        ],
    );
    allowed.insert(
        "bookkeepy-ledger",
        vec![
            "bookkeepy-core",
            "bookkeepy-contracts",
            "bookkeepy-bus",
            "bookkeepy-storage",
            "bookkeepy-primary-data",
            "bookkeepy-shared",
        ],
    );
    allowed.insert(
        "bookkeepy-storage",
        vec![
            "bookkeepy-core",
            "bookkeepy-contracts",
            "bookkeepy-bus",
            "bookkeepy-profile",
            "bookkeepy-primary-data",
            "bookkeepy-logging",
            "bookkeepy-diagnostics",
        ],
    );
    allowed.insert(
        "xtask",
        vec![
            "bookkeepy-core",
            "bookkeepy-primary-data",
            "bookkeepy-build-info",
            "bookkeepy-profile",
        ],
    );

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

/// `govern lints` — SPDX license header on every Rust source file (GPL-3.0-or-later,
/// round-4 item 6); clippy -D warnings itself is enforced by `xtask check`/CI.
fn lints() -> bool {
    let mut rs_files = Vec::new();
    for c in ["crates", "xtask"] {
        collect_rs(&root().join(c), &mut rs_files);
    }
    // build.rs files too
    for c in std::fs::read_dir(root().join("crates"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let b = c.path().join("build.rs");
        if b.is_file() {
            rs_files.push(b);
        }
    }
    let mut ok = true;
    for f in &rs_files {
        let content = std::fs::read_to_string(f).unwrap_or_default();
        let head: String = content.lines().take(3).collect::<Vec<_>>().join("\n");
        if !head.contains("SPDX-License-Identifier: GPL-3.0-or-later") {
            eprintln!(
                "govern lints: FAIL: {} is missing the SPDX header",
                f.display()
            );
            ok = false;
        }
    }
    if ok {
        println!(
            "govern lints: PASS ({} rust files carry SPDX GPL-3.0-or-later; clippy -D warnings enforced by xtask check)",
            rs_files.len()
        );
    }
    ok
}

/// `govern meta` — the code registry `docs/registry/code-map.toml` is the SSOT for what
/// exists in the codebase (RULE-META-001): every crates/* dir is registered, entries
/// point at real paths, statuses are valid, deprecated/removed entries name a successor
/// or removal note, ADR refs resolve to files.
fn meta() -> bool {
    let path = root().join("docs/registry/code-map.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("govern meta: FAIL: cannot read {}: {e}", path.display());
            return false;
        }
    };
    #[derive(serde::Deserialize)]
    struct Item {
        id: String,
        path: String,
        status: String,
        #[serde(default)]
        successor: Option<String>,
        #[serde(default)]
        removal_note: Option<String>,
        #[serde(default)]
        adr: Vec<String>,
    }
    #[derive(serde::Deserialize)]
    struct Map {
        #[serde(default)]
        item: Vec<Item>,
    }
    let map: Map = match toml::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("govern meta: FAIL: code-map.toml parse: {e}");
            return false;
        }
    };
    let mut ok = true;
    let mut ids = std::collections::HashSet::new();
    for item in &map.item {
        if !ids.insert(item.id.clone()) {
            eprintln!("govern meta: FAIL: duplicate id {}", item.id);
            ok = false;
        }
        if !root().join(&item.path).exists() {
            eprintln!("govern meta: FAIL: {} path {} missing", item.id, item.path);
            ok = false;
        }
        match item.status.as_str() {
            "active" => {}
            "deprecated" | "removed" => {
                if item.successor.is_none() && item.removal_note.is_none() {
                    eprintln!(
                        "govern meta: FAIL: {} is {} but has neither successor nor removal_note",
                        item.id, item.status
                    );
                    ok = false;
                }
            }
            other => {
                eprintln!("govern meta: FAIL: {} has invalid status {other}", item.id);
                ok = false;
            }
        }
        for a in &item.adr {
            let num = a.strip_prefix("ADR-").unwrap_or(a);
            let file = format!("ADR-{num}-");
            let dir = root().join("docs/adr");
            let found = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(&file));
            if !found {
                eprintln!("govern meta: FAIL: {} references unknown ADR-{a}", item.id);
                ok = false;
            }
        }
    }
    // every crates/* dir + xtask must be registered
    for c in std::fs::read_dir(root().join("crates"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let dir = c.path().join("src");
        if dir.is_dir() {
            let rel = format!("crates/{}", c.file_name().to_string_lossy());
            if !map.item.iter().any(|i| i.path == rel) {
                eprintln!("govern meta: FAIL: {rel} is not registered in code-map.toml");
                ok = false;
            }
        }
    }
    if !map.item.iter().any(|i| i.path == "xtask") {
        eprintln!("govern meta: FAIL: xtask is not registered in code-map.toml");
        ok = false;
    }
    if ok {
        println!(
            "govern meta: PASS ({} registered items in docs/registry/code-map.toml)",
            map.item.len()
        );
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
