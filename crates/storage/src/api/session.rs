// SPDX-License-Identifier: GPL-3.0-or-later
//! Session composition root (doc 05 §5.2 boot sequence, `data/load-policies.toml`
//! `[boot_sequence]`): wires, in canonical order — panic hook (`diagnostics_logging`) →
//! profile registry (`registry`) → single-instance lock + DB open + migrations
//! (`profile_open`) → settings snapshot (`settings`) → jobs resume (`jobs_resume`) →
//! self-check → outbox replay → auto local live backup + prune → `registry.set_active`.
//! The executed order of [`BOOT_STEPS`] is asserted (unit test) to be a subsequence of
//! the canonical boot ids, so code and primary data cannot drift (RULE-SYS-001).
//!
//! Everything the app shell needs for one open profile is in [`Session`]. Nothing here
//! ever blocks the user on a recoverable condition (RULE-UX-001): logging and the auto
//! backup are best-effort and reported, not fatal.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once};
use std::time::Instant;

use bookkeepy_bus::Bus;
use bookkeepy_contracts::EventEnvelope;
use bookkeepy_core::EntityId;
use bookkeepy_diagnostics::install_panic_hook;
use bookkeepy_logging::{self as logging, LogGuard};
use bookkeepy_profile::{ProfileError, ProfileMeta, ProfileRoots, Registry};

use crate::api::backup::{self, KEEP_BACKUPS};
use crate::api::conn::Db;
use crate::api::error::StorageError;
use crate::api::jobs;
use crate::api::kv;
use crate::api::lock::{self, InstanceLock};
use crate::api::outbox;
use crate::api::selfcheck::{SelfCheckReport, run as self_check};
use crate::api::tx;

/// The canonical boot steps this session executes, as load-policies ids. Storage-internal
/// sub-steps (self-check, outbox replay, auto backup, set_active) run inside
/// `profile_open`/`jobs_resume` and are not listed here. The unit test below asserts this
/// list is a subsequence of `data/load-policies.toml [boot_sequence]` step ids.
pub const BOOT_STEPS: &[&str] = &[
    "diagnostics_logging",
    "registry",
    "profile_open",
    "settings",
    "jobs_resume",
];

/// One open profile: everything the app shell holds while the profile is active.
pub struct Session {
    /// App data dir the session was opened from.
    pub data_dir: PathBuf,
    /// Profile root paths (`app.db`, `backups/`, `logs/`, …).
    pub roots: ProfileRoots,
    /// The active profile's metadata.
    pub profile: ProfileMeta,
    /// The profile database.
    pub db: Db,
    /// The in-process event bus (outbox publishes land here).
    pub bus: Bus,
    /// Startup self-check report (integrity, audit chain, drafts, backup note, boot ms).
    pub report: SelfCheckReport,
    /// Settings snapshot at boot: every SSOT key resolved (override over default).
    pub settings: Vec<(String, String)>,
    /// Single-instance lock, held for the session lifetime.
    _lock: InstanceLock,
}

impl Session {
    /// Post an event atomically (outbox + audit in one transaction, then publish).
    /// See [`crate::api::tx::post`].
    pub fn post(&self, ev: &EventEnvelope) -> Result<i64, StorageError> {
        tx::post(&self.db, &self.bus, ev)
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("data_dir", &self.data_dir)
            .field("profile", &self.profile.name)
            .field("roots", &self.roots)
            .field("report", &self.report)
            .finish_non_exhaustive()
    }
}

/// Logging can only be initialized once per process (global subscriber); the guard is
/// kept for the process lifetime.
static LOG_GUARD: Mutex<Option<LogGuard>> = Mutex::new(None);
static PANIC_ONCE: Once = Once::new();

fn ensure_logging(dir: &Path) -> Result<(), StorageError> {
    let mut g = LOG_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    if g.is_none() {
        *g = Some(logging::init(dir)?);
    }
    Ok(())
}

fn ensure_panic_hook() {
    PANIC_ONCE.call_once(install_panic_hook);
}

/// Open a session on `profile_id` inside `data_dir`. Fails with `LockHeld`
/// (BK-STORE-0004) if another instance has the profile open — the app shell then focuses
/// the existing window (single instance, ADR-0016).
pub fn open_session(data_dir: &Path, profile_id: EntityId) -> Result<Session, StorageError> {
    let boot_start = Instant::now();
    // boot step `diagnostics_logging` (panic hook is process-level and needs no profile)
    ensure_panic_hook();
    // boot step `registry`
    let mut registry = Registry::load(data_dir)?;
    let profile = registry
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .cloned()
        .ok_or_else(|| {
            StorageError::Profile(ProfileError::Invalid(format!(
                "unknown profile {profile_id}"
            )))
        })?;
    let roots = ProfileRoots::new(data_dir.join(&profile.path));
    // boot step `profile_open` (single-instance lock first: second launch -> BK-STORE-0004)
    let lock_guard = lock::acquire(&roots.root)?;
    let db = Db::open_profile(&roots)?;
    // file logging needs the profile dir; best-effort, never blocking (RULE-UX-001)
    let _ = ensure_logging(&roots.logs());
    // boot step `settings`: resolve every SSOT key once (override over default)
    let settings = kv::settings_snapshot(&db)?;
    // boot step `jobs_resume`: interrupted `running` jobs -> queued
    jobs::resume_pending(&db)?;
    // storage-internal sub-steps of profile_open/jobs_resume: self-check, outbox replay,
    // auto local live backup + prune
    let mut report = self_check(&db, &roots.root)?;
    let bus = Bus::new();
    outbox::publish_pending(&db, &bus)?;
    let name = backup::auto_name(&roots.root);
    match backup::backup(&db, &roots.root, &name).and_then(|p| {
        backup::prune(&roots.root, KEEP_BACKUPS)?;
        backup::verify_backup(&p)?;
        Ok::<_, StorageError>(p)
    }) {
        Ok(p) => report.backup_note = Some(format!("auto backup ok: {}", p.display())),
        Err(e) => report.backup_note = Some(format!("auto backup skipped: {} ({})", e, e.code())),
    }
    report.boot_ms = Some(boot_start.elapsed().as_millis() as u64);
    registry.set_active(data_dir, profile_id)?;
    Ok(Session {
        data_dir: data_dir.to_path_buf(),
        roots,
        profile,
        db,
        bus,
        report,
        settings,
        _lock: lock_guard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_profile::Registry;

    #[test]
    fn boot_steps_are_subsequence_of_load_policies() {
        let canonical = bookkeepy_primary_data::load_policies::get()
            .boot_sequence
            .step_ids();
        let mut it = canonical.iter();
        for step in BOOT_STEPS {
            assert!(
                it.clone().any(|c| c == step),
                "BOOT_STEPS entry {step:?} is not in load-policies boot_sequence (or out of order)"
            );
            // advance past the matched id
            while it.next().is_some_and(|c| c != step) {}
        }
    }

    #[test]
    fn open_session_wires_everything_and_lock_blocks_second_open() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let mut reg = Registry::load(dir).unwrap();
        let meta = reg.create_profile(dir, "Personal", "#3366ff").unwrap();
        let s = open_session(dir, meta.id).unwrap();
        assert!(s.report.healthy());
        assert!(s.report.migrations_match);
        assert!(s.report.boot_ms.is_some());
        assert!(s.report.backup_note.is_some());
        assert!(s.settings.iter().any(|(k, _)| k == "theme"));
        // second open on the same profile must fail with the lock (single instance)
        let err = open_session(dir, meta.id).unwrap_err();
        assert_eq!(err.code(), "BK-STORE-0004");
        drop(s);
        // after drop the lock is released
        let _s2 = open_session(dir, meta.id).unwrap();
    }
}
