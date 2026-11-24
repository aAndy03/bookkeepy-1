// SPDX-License-Identifier: GPL-3.0-or-later
//! On-disk layout (doc 05 §5.2, F18): one app data dir holds `profiles.toml` (the
//! registry) plus one `profiles/<id>/` tree per profile. Profiles never share data
//! (INV-PROF-001); the only bridge between them is explicit export/import (F17).
//!
//! Uninstall never deletes this tree (INV-APP-001) — see `packaging/README.md`.

use std::path::{Path, PathBuf};

/// App data dir, e.g. `%APPDATA%/Bookkeepy 1/data` on Windows,
/// `~/Library/Application Support/Bookkeepy 1/data` on macOS, `~/.local/share/bookkeepy-1/data` on Linux.
pub fn app_data_dir() -> PathBuf {
    directories::ProjectDirs::from("com.bookkeepy", "Bookkeepy", "Bookkeepy 1")
        .map(|p| p.data_dir().to_path_buf())
        // On an OS without a data dir, fall back to a temp location rather than panic
        // (RULE-ERR-003: never crash on environment quirks).
        .unwrap_or_else(|| std::env::temp_dir().join("bookkeepy-1-data"))
}

/// Well-known paths inside one profile's root (`profiles/<id>/`).
#[derive(Debug, Clone)]
pub struct ProfileRoots {
    /// `…/profiles/<id>`
    pub root: PathBuf,
}

impl ProfileRoots {
    /// Paths for the profile rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `root/app.db` — the rusqlite database (M1, ADR-0005).
    pub fn db(&self) -> PathBuf {
        self.root.join("app.db")
    }

    /// `root/index/` — tantivy index (M5, ADR-0006).
    pub fn index(&self) -> PathBuf {
        self.root.join("index")
    }

    /// `root/rates/` — downloaded + manual fx packages (doc 11).
    pub fn rates(&self) -> PathBuf {
        self.root.join("rates")
    }

    /// `root/exports/` — user exports (F17).
    pub fn exports(&self) -> PathBuf {
        self.root.join("exports")
    }

    /// `root/logs/` — daily logs (bookkeepy-logging).
    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// `root/drafts/` — crash-recovery drafts (doc 10 round-2, item 2).
    pub fn drafts(&self) -> PathBuf {
        self.root.join("drafts")
    }

    /// Create the directory skeleton if missing. Returns the root.
    pub fn create(&self) -> std::io::Result<&Path> {
        for p in [
            self.index(),
            self.rates(),
            self.exports(),
            self.logs(),
            self.drafts(),
            self.root.clone(),
        ] {
            std::fs::create_dir_all(p)?;
        }
        Ok(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_layout_matches_doc05() {
        let r = ProfileRoots::new("/data/profiles/abc");
        assert!(r.db().ends_with("app.db"));
        assert!(r.index().ends_with("index"));
        assert!(r.rates().ends_with("rates"));
        assert!(r.exports().ends_with("exports"));
        assert!(r.logs().ends_with("logs"));
        assert!(r.drafts().ends_with("drafts"));
    }

    #[test]
    fn create_makes_skeleton() {
        let tmp = tempfile::tempdir().unwrap();
        let r = ProfileRoots::new(tmp.path().join("profiles/x"));
        r.create().unwrap();
        assert!(r.index().is_dir());
        assert!(r.logs().is_dir());
    }
}
