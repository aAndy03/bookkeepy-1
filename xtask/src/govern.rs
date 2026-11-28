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
    "data",
    "errors",
    "events",
    "adr",
    "docs",
    "deps",
    "net",
    "lints",
    "perf",
    "ui",
    "meta",
    "seams",
    "regressions",
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
            "ui" => ui(),
            "meta" => meta(),
            "seams" => seams(),
            "regressions" => regressions(),
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

/// `govern regressions` — RULE-TEST-001 (ADR-0026): every completed milestone / hardening /
/// audit section in `docs/10-process-roadmap.md` must name the tests guarding what it
/// introduced (bugs, edge cases, seams, optimizations, features). New sections without a
/// "test" mention fail the gate.
fn regressions() -> bool {
    let path = root().join("docs/10-process-roadmap.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        eprintln!("govern regressions: FAIL: cannot read {}", path.display());
        return false;
    };
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut cur_head = String::new();
    let mut cur_body = String::new();
    for line in content.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            if !cur_head.is_empty() {
                sections.push((cur_head.clone(), cur_body.clone()));
            }
            cur_head = h.to_string();
            cur_body = String::new();
        } else {
            cur_body.push_str(line);
            cur_body.push('\n');
        }
    }
    if !cur_head.is_empty() {
        sections.push((cur_head, cur_body));
    }
    let mut ok = true;
    let mut counted = 0usize;
    for (head, body) in &sections {
        let hl = head.to_lowercase();
        if hl.contains("completed") || hl.contains("hardening") || hl.contains("audit") {
            counted += 1;
            if !body.to_lowercase().contains("test") {
                eprintln!(
                    "govern regressions: FAIL: doc 10 section \"{head}\" introduces work without naming its guard tests (RULE-TEST-001)"
                );
                ok = false;
            }
        }
    }
    if ok {
        println!(
            "govern regressions: PASS ({counted} completed/hardening/audit sections in doc 10 all name guard tests)"
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
        "bookkeepy-app-services",
        vec![
            "bookkeepy-core",
            "bookkeepy-contracts",
            "bookkeepy-bus",
            "bookkeepy-storage",
            "bookkeepy-ledger",
            "bookkeepy-shared",
            "bookkeepy-profile",
        ],
    );
    // The UI layer (RULE-UI-004): app-services' api (presenters), contracts, and the
    // data layers. The M3 scaffold uses core + primary-data (+ gpui-kit, external).
    allowed.insert(
        "bookkeepy-ui",
        vec![
            "bookkeepy-core",
            "bookkeepy-contracts",
            "bookkeepy-primary-data",
            "bookkeepy-i18n",
            "bookkeepy-app-services",
        ],
    );
    // The gallery harness: the ui crate (the one component set) + the data layers
    // for locale/theme/shortcut source material — never app-services (demo-only UI).
    allowed.insert(
        "bookkeepy-gallery",
        vec![
            "bookkeepy-core",
            "bookkeepy-ui",
            "bookkeepy-primary-data",
            "bookkeepy-i18n",
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

/// `govern ui` — the gallery + ui crate gate (RULE-UI-005/006/007/008, ADR-0017):
/// 1. inventory ↔ gallery registry: every `covered` gpui-kit component points at a
///    registered gallery section; every `exception` carries a note; ids are unique.
/// 2. RULE-UI-005: every owned text-input construction carries a `.placeholder(...)`
///    (OtpState is exempt: the OTP segmented input has no placeholder API).
/// 3. i18n: en/ro locale key parity + every key used by gallery/ui code exists in
///    both locales (the inline half of the locale contract; `govern data` validates
///    the files themselves).
/// 4. widget metadata (RULE-UI-008): every `crates/ui/src/widgets/**/meta.toml` is
///    schema-valid, declares only REGISTERED capability traits, references sibling
///    `source.rs` + `variant_*.rs`/`edge_case_*.rs` files that exist, and its
///    changelog covers the current widget version.
fn ui() -> bool {
    let inventory = ui_inventory_sections();
    let placeholders = ui_input_placeholders();
    let i18n = ui_i18n_keys();
    let meta = ui_widget_meta();
    if inventory.0 && placeholders.0 && i18n.0 && meta.0 {
        println!(
            "govern ui: PASS ({} gpui-kit components: {} covered, {} exceptions; {} widget meta(s); {} inline i18n key use(s); text inputs carry placeholders)",
            inventory.1, inventory.2, inventory.3, meta.1, i18n.1
        );
        true
    } else {
        eprintln!("govern ui: FAIL");
        false
    }
}

/// Check 1 — inventory ↔ gallery sections. Returns (ok, total, covered, exceptions).
fn ui_inventory_sections() -> (bool, usize, usize, usize) {
    let path = root().join("docs/registry/gpui-kit-components.toml");
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("govern ui: FAIL: cannot read {}: {e}", path.display());
            return (false, 0, 0, 0);
        }
    };
    let table = match raw.parse::<toml::Table>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("govern ui: FAIL: {path:?} does not parse: {e}");
            return (false, 0, 0, 0);
        }
    };
    // The gallery's SECTIONS registry ids (gallery/src/sections/mod.rs `section!(...)`).
    let sections_path = root().join("gallery/src/sections/mod.rs");
    let sections_src = match std::fs::read_to_string(&sections_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "govern ui: FAIL: cannot read {}: {e}",
                sections_path.display()
            );
            return (false, 0, 0, 0);
        }
    };
    let mut section_ids: Vec<&str> = Vec::new();
    for line in sections_src.lines() {
        if line.contains("section!(")
            && let Some(q1) = line.find('"')
            && let Some(q2) = line[q1 + 1..].find('"')
        {
            section_ids.push(&line[q1 + 1..q1 + 1 + q2]);
        }
    }
    let entries = table
        .get("component")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let (mut covered, mut exceptions, mut ok) = (0usize, 0usize, true);
    let mut seen: Vec<String> = Vec::new();
    for entry in &entries {
        let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let title = entry
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let status = entry
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if id.is_empty() || title.is_empty() {
            eprintln!("govern ui: FAIL: inventory entry missing id/title: {entry}");
            ok = false;
            continue;
        }
        if seen.iter().any(|s| s == id) {
            eprintln!("govern ui: FAIL: duplicate inventory id: {id}");
            ok = false;
        }
        seen.push(id.to_string());
        let section = entry.get("section").and_then(|v| v.as_str());
        match status {
            "covered" => {
                covered += 1;
                let Some(section) = section else {
                    eprintln!("govern ui: FAIL: covered component {id} has no gallery section");
                    ok = false;
                    continue;
                };
                if !section_ids.contains(&section) {
                    eprintln!("govern ui: FAIL: {id} points at unknown gallery section {section}");
                    ok = false;
                }
            }
            "exception" => {
                exceptions += 1;
                let note = entry
                    .get("note")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if note.trim().is_empty() {
                    eprintln!("govern ui: FAIL: exception {id} must carry a note explaining it");
                    ok = false;
                }
            }
            other => {
                eprintln!("govern ui: FAIL: {id} has invalid status {other:?} (covered|exception)");
                ok = false;
            }
        }
    }
    if entries.is_empty() {
        eprintln!("govern ui: FAIL: component inventory is empty");
        ok = false;
    }
    (ok, entries.len(), covered, exceptions)
}

/// Recursively collect every `.rs` source file under `dir`.
fn ui_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                ui_rust_sources(&p, out);
            } else if p.extension().is_some_and(|ext| ext == "rs") {
                out.push(p);
            }
        }
    }
}

/// Check 2 — RULE-UI-005: owned text-input constructions carry placeholders.
/// Returns (ok, count of construction sites checked).
fn ui_input_placeholders() -> (bool, usize) {
    // Constructors whose owning statement must localize a placeholder. OtpState is
    // deliberately absent: the OTP input is segmented and has no placeholder API
    // (documented in gallery/src/sections/atoms_input.rs).
    const CONSTRUCTORS: &[&str] = &[
        "InputState::new(",
        "TextareaState::new(",
        "EditorState::new(",
    ];
    const WINDOW: usize = 10;
    let mut sources = Vec::new();
    ui_rust_sources(&root().join("gallery/src"), &mut sources);
    ui_rust_sources(&root().join("crates/ui/src"), &mut sources);
    let (mut ok, mut checked) = (true, 0usize);
    for path in &sources {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("govern ui: FAIL: cannot read {}: {e}", path.display());
                ok = false;
                continue;
            }
        };
        let lines: Vec<&str> = src.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            for ctor in CONSTRUCTORS {
                if !line.contains(ctor) || line.trim_start().starts_with("//") {
                    continue;
                }
                checked += 1;
                let window: String = lines[idx..(idx + WINDOW).min(lines.len())].join("\n");
                if !window.contains(".placeholder(") {
                    eprintln!(
                        "govern ui: FAIL: RULE-UI-005: {}:{} constructs {ctor} without a localized .placeholder(...)",
                        path.display(),
                        idx + 1
                    );
                    ok = false;
                }
            }
        }
    }
    (ok, checked)
}

/// Collect every key of a parsed TOML document table as dotted flat paths.
fn ui_toml_keys(table: &toml::Table, prefix: &str, out: &mut Vec<String>) {
    for (k, v) in table {
        let path = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        match v {
            toml::Value::Table(t) => ui_toml_keys(t, &path, out),
            _ => out.push(path),
        }
    }
}

/// The substring inside the balanced parentheses starting at `open` (points at the
/// first inner byte); string literals are skipped so parens inside them don't count.
fn paren_region(line: &str, open: usize) -> Option<String> {
    let bytes = line.as_bytes();
    let mut depth = 1usize;
    let mut i = open;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if b == b'\\' {
                i += 1;
            } else if b == b'"' {
                in_string = false;
            }
        } else if b == b'"' {
            in_string = true;
        } else if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(line[open..i].to_string());
            }
        }
        i += 1;
    }
    None
}

/// Every double-quoted literal in `s`.
fn quoted_literals(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let mut literal = String::new();
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    literal.push(bytes[i] as char);
                    i += 1;
                } else if bytes[i] == b'"' {
                    break;
                }
                literal.push(bytes[i] as char);
                i += 1;
            }
            out.push(literal);
        }
        i += 1;
    }
    out
}

/// Check 3 — inline i18n contract: en/ro parity + used keys exist. Returns (ok, used).
fn ui_i18n_keys() -> (bool, usize) {
    let en = match std::fs::read_to_string(root().join("data/locales/en.toml")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("govern ui: FAIL: cannot read data/locales/en.toml: {e}");
            return (false, 0);
        }
    };
    let ro = match std::fs::read_to_string(root().join("data/locales/ro.toml")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("govern ui: FAIL: cannot read data/locales/ro.toml: {e}");
            return (false, 0);
        }
    };
    let mut ok = true;
    // The catalogs are `[messages]` tables — collect only leaf paths inside them
    // (`meta` never holds messages).
    let collect = |raw: &str| -> Option<Vec<String>> {
        raw.parse::<toml::Table>().ok().and_then(|t| {
            t.get("messages")
                .and_then(|v| v.as_table())
                .map(|messages| {
                    let mut keys = Vec::new();
                    ui_toml_keys(messages, "", &mut keys);
                    keys
                })
        })
    };
    let (mut en_keys, mut ro_keys) = (Vec::new(), Vec::new());
    if let Some(v) = collect(&en) {
        en_keys = v;
    } else {
        eprintln!("govern ui: FAIL: data/locales/en.toml does not parse (or has no [messages])");
        ok = false;
    }
    if let Some(v) = collect(&ro) {
        ro_keys = v;
    } else {
        eprintln!("govern ui: FAIL: data/locales/ro.toml does not parse (or has no [messages])");
        ok = false;
    }
    en_keys.sort_unstable();
    ro_keys.sort_unstable();
    if en_keys != ro_keys {
        eprintln!("govern ui: FAIL: locale key parity broken (en vs ro differs)");
        for k in en_keys.iter().filter(|k| !ro_keys.contains(k)).take(10) {
            eprintln!("govern ui:   only in en: {k}");
        }
        for k in ro_keys.iter().filter(|k| !en_keys.contains(k)).take(10) {
            eprintln!("govern ui:   only in ro: {k}");
        }
        ok = false;
    }
    // Inline uses: every "gallery.*" literal, and every quoted tr(...) argument.
    let mut sources = Vec::new();
    ui_rust_sources(&root().join("gallery/src"), &mut sources);
    ui_rust_sources(&root().join("crates/ui/src"), &mut sources);
    let (mut used_count, mut missing) = (0usize, Vec::<String>::new());
    for path in &sources {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for (idx, line) in src.lines().enumerate() {
            // Every quoted literal on the line (needed for the `gallery.*` rule).
            let mut quotes: Vec<String> = Vec::new();
            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    let mut literal = String::new();
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' && i + 1 < bytes.len() {
                            literal.push(bytes[i] as char);
                            i += 1;
                        } else if bytes[i] == b'"' {
                            break;
                        }
                        literal.push(bytes[i] as char);
                        i += 1;
                    }
                    quotes.push(literal);
                }
                i += 1;
            }
            // Quoted literals passed to a `tr(...)` call: only the ones INSIDE its
            // parens count (element ids like "doc" on the same line are not keys).
            let mut tr_args: Vec<String> = Vec::new();
            for (pos, _) in line.match_indices("tr(") {
                if pos != 0 && line.as_bytes()[pos - 1].is_ascii_alphanumeric() {
                    continue;
                }
                let Some(region) = paren_region(line, pos + 2) else {
                    continue;
                };
                tr_args.extend(quoted_literals(&region));
            }
            let check = |key: &str, used: &mut usize, missing: &mut Vec<String>| {
                // Fragments used in concat!/format! key builders end with '.' — not keys.
                if key.is_empty() || key.contains(' ') || key.ends_with('.') {
                    return;
                }
                *used += 1;
                if !en_keys.contains(&key.to_string()) || !ro_keys.contains(&key.to_string()) {
                    missing.push(format!("{}:{}: {key}", path.display(), idx + 1));
                }
            };
            for q in quotes {
                if q.starts_with("gallery.") {
                    check(&q, &mut used_count, &mut missing);
                }
            }
            for q in tr_args {
                check(&q, &mut used_count, &mut missing);
            }
        }
    }
    if !missing.is_empty() {
        missing.sort_unstable();
        missing.dedup();
        eprintln!("govern ui: FAIL: i18n keys used in code but missing from locales:");
        for m in missing.iter().take(20) {
            eprintln!("govern ui:   {m}");
        }
        if missing.len() > 20 {
            eprintln!("govern ui:   ... and {} more", missing.len() - 20);
        }
        ok = false;
    }
    (ok, used_count)
}

/// Check 4 — widget `meta.toml` (RULE-UI-008). Returns (ok, widget count).
fn ui_widget_meta() -> (bool, usize) {
    // Registered capability-trait ids, from crates/ui/src/capabilities.rs (`id: "..."`).
    let caps_path = root().join("crates/ui/src/capabilities.rs");
    let caps_src = match std::fs::read_to_string(&caps_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("govern ui: FAIL: cannot read {}: {e}", caps_path.display());
            return (false, 0);
        }
    };
    let mut registered: Vec<String> = Vec::new();
    for line in caps_src.lines() {
        if line.contains("id: \"")
            && let Some(q1) = line.find('"')
            && let Some(q2) = line[q1 + 1..].find('"')
        {
            registered.push(line[q1 + 1..q1 + 1 + q2].to_string());
        }
    }
    // Every meta.toml under the widget layers.
    let mut metas: Vec<(&str, PathBuf, PathBuf, PathBuf)> = Vec::new();
    for layer in ["atoms", "molecules", "organisms"] {
        let dir = root().join("crates/ui/src/widgets").join(layer);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.filter_map(|e| e.ok()) {
                let meta = e.path().join("meta.toml");
                let source = e.path().join("source.rs");
                if meta.is_file() {
                    metas.push((layer, e.path(), meta, source));
                }
            }
        }
    }
    let count = metas.len();
    let mut ok = true;
    for (layer, widget_dir, meta_path, source_path) in &metas {
        let raw = match std::fs::read_to_string(meta_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("govern ui: FAIL: cannot read {}: {e}", meta_path.display());
                ok = false;
                continue;
            }
        };
        let table = match raw.parse::<toml::Table>() {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "govern ui: FAIL: {} does not parse: {e}",
                    meta_path.display()
                );
                ok = false;
                continue;
            }
        };
        let where_ = meta_path.display();
        let Some(widget) = table.get("widget").and_then(|v| v.as_table()) else {
            eprintln!("govern ui: FAIL: {where_}: missing [widget]");
            ok = false;
            continue;
        };
        let id = widget
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let version = widget
            .get("version")
            .and_then(|v| v.as_integer())
            .unwrap_or(-1);
        let widget_layer = widget
            .get("layer")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let traits = widget
            .get("traits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if id.is_empty() || version < 1 {
            eprintln!("govern ui: FAIL: {where_}: [widget] needs id + version >= 1");
            ok = false;
        }
        if widget_layer != *layer {
            eprintln!(
                "govern ui: FAIL: {where_}: layer {widget_layer:?} does not match its folder {layer:?}"
            );
            ok = false;
        }
        if id
            != widget_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
        {
            eprintln!("govern ui: FAIL: {where_}: [widget] id must equal the folder name");
            ok = false;
        }
        for t in &traits {
            let tid = t.as_str().unwrap_or_default();
            if !registered.iter().any(|r| r == tid) {
                eprintln!(
                    "govern ui: FAIL: {where_}: trait {tid:?} is not in capabilities::REGISTERED"
                );
                ok = false;
            }
        }
        if !source_path.is_file() {
            eprintln!("govern ui: FAIL: {where_}: missing source.rs (the SSOT)");
            ok = false;
        }
        // Variants + edge cases: files exist and follow the naming contract.
        if let Some(variants) = table.get("variant").and_then(|v| v.as_array()) {
            for v in variants {
                let file = v.get("file").and_then(|f| f.as_str()).unwrap_or_default();
                let trait_id = v.get("trait").and_then(|t| t.as_str()).unwrap_or_default();
                if !file.starts_with("variant_") || !widget_dir.join(file).is_file() {
                    eprintln!("govern ui: FAIL: {where_}: bad variant file {file:?}");
                    ok = false;
                }
                if !registered.iter().any(|r| r == trait_id) {
                    eprintln!(
                        "govern ui: FAIL: {where_}: variant trait {trait_id:?} is not registered"
                    );
                    ok = false;
                }
            }
        }
        if let Some(edges) = table.get("edge_case").and_then(|v| v.as_array()) {
            for e in edges {
                let file = e.get("file").and_then(|f| f.as_str()).unwrap_or_default();
                if !file.starts_with("edge_case_") || !widget_dir.join(file).is_file() {
                    eprintln!("govern ui: FAIL: {where_}: bad edge-case file {file:?}");
                    ok = false;
                }
            }
        }
        // Changelog: non-empty and covering exactly up to widget.version.
        let changelog = table
            .get("changelog")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if changelog.is_empty() {
            eprintln!("govern ui: FAIL: {where_}: [[changelog]] must not be empty (RULE-UI-008)");
            ok = false;
            continue;
        }
        let mut versions: Vec<i64> = changelog
            .iter()
            .filter_map(|c| c.get("version").and_then(|v| v.as_integer()))
            .collect();
        versions.sort_unstable();
        let current = versions.last().cloned().unwrap_or(-1);
        if current != version || versions.len() != changelog.len() {
            eprintln!(
                "govern ui: FAIL: {where_}: changelog must cover exactly up to widget.version {version}"
            );
            ok = false;
        }
    }
    (ok, count)
}
