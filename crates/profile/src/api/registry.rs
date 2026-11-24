// SPDX-License-Identifier: GPL-3.0-or-later
//! The profile registry (F18, item 9, M0.5): `profiles.toml` in the app data dir lists
//! all profiles and which one is active. Profiles are fully isolated (INV-PROF-001);
//! switching only changes the `active` pointer and touches `last_opened_at`.

use bookkeepy_core::EntityId;
use bookkeepy_core::time::{Timestamp, now};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::roots::ProfileRoots;

/// Registry file name inside the app data dir.
pub const REGISTRY_FILE: &str = "profiles.toml";

/// Errors the registry can produce (registered as `BK-PROFILE-0001` in `data/errors.toml`).
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    /// IO failure reading/writing the registry or creating a profile dir.
    #[error("profile io error at {path}: {source}")]
    Io {
        /// Path involved.
        path: PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// The registry file failed to parse.
    #[error("registry parse error at {path}: {source}")]
    Parse {
        /// Path involved.
        path: PathBuf,
        /// Underlying error.
        source: toml::de::Error,
    },
    /// Validation failed (duplicate id/name, path escape, unknown active id…).
    #[error("registry invalid: {0}")]
    Invalid(String),
}

impl ProfileError {
    /// Registered error code (RULE-ERR-002).
    pub fn code(&self) -> &'static str {
        "BK-PROFILE-0001"
    }
}

/// One profile entry. `path` is always relative (`profiles/<id>`) — INV-PROF-002:
/// the registry never points outside the app data dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    /// Profile id (uuid v7).
    pub id: EntityId,
    /// Display name (unique).
    pub name: String,
    /// Accent color as `#rrggbb` (UI token consumer validates format).
    pub color: String,
    /// Relative path under the app data dir, `profiles/<id>`.
    pub path: String,
    /// Created at (UTC).
    pub created_at: Timestamp,
    /// Last time this profile was opened (UTC).
    pub last_opened_at: Timestamp,
}

/// The registry file: active profile + all profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    /// Active profile id, if any.
    pub active: Option<EntityId>,
    /// All profiles.
    pub profiles: Vec<ProfileMeta>,
}

impl Registry {
    /// Load `dir/profiles.toml`, or return an empty registry if it doesn't exist yet.
    pub fn load(dir: &Path) -> Result<Self, ProfileError> {
        let path = dir.join(REGISTRY_FILE);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let reg: Registry =
                    toml::from_str(&text).map_err(|source| ProfileError::Parse { path, source })?;
                reg.validate(dir)?;
                Ok(reg)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Registry {
                active: None,
                profiles: Vec::new(),
            }),
            Err(source) => Err(ProfileError::Io { path, source }),
        }
    }

    /// Persist to `dir/profiles.toml` (atomic: write temp + rename, RULE-STATE-002).
    pub fn save(&self, dir: &Path) -> Result<(), ProfileError> {
        self.validate(dir)?;
        std::fs::create_dir_all(dir).map_err(|source| ProfileError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = dir.join(REGISTRY_FILE);
        let text = toml::to_string_pretty(self)
            .map_err(|e| ProfileError::Invalid(format!("serialize: {e}")))?;
        let tmp = dir.join(format!("{REGISTRY_FILE}.tmp"));
        std::fs::write(&tmp, text).map_err(|source| ProfileError::Io {
            path: tmp.clone(),
            source,
        })?;
        std::fs::rename(&tmp, &path).map_err(|source| ProfileError::Io {
            path: path.clone(),
            source,
        })?;
        harden_file(&path);
        Ok(())
    }

    /// Structural invariants: unique ids/names, relative in-data-dir paths, active exists.
    pub fn validate(&self, dir: &Path) -> Result<(), ProfileError> {
        for p in &self.profiles {
            if p.path.contains("..") || Path::new(&p.path).is_absolute() {
                return Err(ProfileError::Invalid(format!(
                    "profile {} path escapes data dir: {}",
                    p.id, p.path
                )));
            }
            if !p.color.starts_with('#') || p.color.len() != 7 {
                return Err(ProfileError::Invalid(format!(
                    "profile {} color must be #rrggbb, got {}",
                    p.id, p.color
                )));
            }
            if self.profiles.iter().filter(|q| q.id == p.id).count() > 1 {
                return Err(ProfileError::Invalid(format!(
                    "duplicate profile id {}",
                    p.id
                )));
            }
            if self.profiles.iter().filter(|q| q.name == p.name).count() > 1 {
                return Err(ProfileError::Invalid(format!(
                    "duplicate profile name {}",
                    p.name
                )));
            }
        }
        if let Some(active) = self.active
            && !self.profiles.iter().any(|p| p.id == active)
        {
            return Err(ProfileError::Invalid(format!(
                "active profile {active} not in profiles"
            )));
        }
        let _ = dir;
        Ok(())
    }

    /// Create a new profile (dir `profiles/<id>` created too), make it active, persist.
    pub fn create_profile(
        &mut self,
        dir: &Path,
        name: &str,
        color: &str,
    ) -> Result<ProfileMeta, ProfileError> {
        let id = EntityId::new();
        let meta = ProfileMeta {
            id,
            name: name.to_string(),
            color: color.to_string(),
            path: format!("profiles/{id}"),
            created_at: now(),
            last_opened_at: now(),
        };
        ProfileRoots::new(dir.join(&meta.path))
            .create()
            .map_err(|source| ProfileError::Io {
                path: dir.join(&meta.path),
                source,
            })?;
        self.profiles.push(meta.clone());
        self.active = Some(id);
        self.save(dir)?;
        Ok(meta)
    }

    /// Switch the active profile (must exist); stamps `last_opened_at`; persists.
    pub fn set_active(&mut self, dir: &Path, id: EntityId) -> Result<(), ProfileError> {
        let p = self
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| ProfileError::Invalid(format!("unknown profile {id}")))?;
        p.last_opened_at = now();
        self.active = Some(id);
        self.save(dir)
    }

    /// The active profile's metadata, if any.
    pub fn active(&self) -> Option<&ProfileMeta> {
        self.active
            .and_then(|a| self.profiles.iter().find(|p| p.id == a))
    }
}

/// Best-effort privacy hardening: on unix, restrict the registry file to the owner (0600)
/// — it lists all profiles (RULE-PRIV-001). No-op elsewhere. (Same shim as in storage;
/// core is no-I/O by design, so the cfg(unix) helper is intentionally duplicated.)
#[cfg(unix)]
fn harden_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(m) = std::fs::metadata(path) {
        let mut p = m.permissions();
        p.set_mode(0o600);
        let _ = std::fs::set_permissions(path, p);
    }
}

#[cfg(not(unix))]
fn harden_file(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_switch_persist_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let mut reg = Registry::load(dir).unwrap();
        assert!(reg.active().is_none());

        let a = reg.create_profile(dir, "Personal", "#3b82f6").unwrap();
        let b = reg.create_profile(dir, "Business", "#22c55e").unwrap();
        assert_eq!(reg.active().unwrap().id, b.id);
        assert!(dir.join(&a.path).join("index").is_dir());

        reg.set_active(dir, a.id).unwrap();
        let reloaded = Registry::load(dir).unwrap();
        assert_eq!(reloaded.active().unwrap().name, "Personal");
        assert_eq!(reloaded.profiles.len(), 2);
    }

    #[test]
    fn rejects_duplicate_names_and_bad_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let mut reg = Registry {
            active: None,
            profiles: vec![
                ProfileMeta {
                    id: EntityId::new(),
                    name: "X".into(),
                    color: "#ffffff".into(),
                    path: "profiles/one".into(),
                    created_at: now(),
                    last_opened_at: now(),
                },
                ProfileMeta {
                    id: EntityId::new(),
                    name: "X".into(),
                    color: "#ffffff".into(),
                    path: "profiles/two".into(),
                    created_at: now(),
                    last_opened_at: now(),
                },
            ],
        };
        assert!(reg.validate(tmp.path()).is_err());
        reg.profiles.pop();
        reg.profiles[0].path = "../escape".into();
        assert!(reg.validate(tmp.path()).is_err());
    }
}
