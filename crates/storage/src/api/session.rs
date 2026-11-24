// SPDX-License-Identifier: GPL-3.0-or-later
//! Session composition root (doc 05 §5.2 boot sequence, `data/load-policies.toml`
//! `[boot_sequence]`): wires, in order — profile registry → profile roots → single-instance
//! lock → DB open + migrations → logging (once per process) → panic hook (once) → jobs
//! resume → self-check → outbox replay → auto local live backup + prune → registry
//! `set_active`. Everything the app shell needs for one open profile is in [`Session`].
//! Nothing here ever blocks the user on a recoverable condition (RULE-UX-001): logging and
//! the auto backup are best-effort and reported, not fatal.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once};

use bookkeepy_bus::Bus;
use bookkeepy_contracts::EventEnvelope;
use bookkeepy_core::EntityId;
use bookkeepy_core::time::now;
use bookkeepy_diagnostics::install_panic_hook;
use bookkeepy_logging::{self as logging, LogGuard};
use bookkeepy_profile::{ProfileError, ProfileMeta, ProfileRoots, Registry};

use crate::api::backup::{self, KEEP_BACKUPS};
use crate::api::conn::Db;
use crate::api::error::StorageError;
use crate::api::jobs;
use crate::api::lock::{self, InstanceLock};
use crate::api::outbox;
use crate::api::selfcheck::{SelfCheckReport, run as self_check};
use crate::api::tx;

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
    /// Startup self-check report (integrity, audit chain, drafts, backup note).
    pub report: SelfCheckReport,
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
    let lock_guard = lock::acquire(&roots.root)?;
    let db = Db::open_profile(&roots)?;
    // Best-effort, never blocking (RULE-UX-001): BK-LOG-0001 is warning severity.
    let _ = ensure_logging(&roots.logs());
    ensure_panic_hook();
    // Startup drivers: interrupted jobs → queued; leftover outbox rows → bus (exactly-once).
    jobs::resume_pending(&db)?;
    let mut report = self_check(&db, &roots.root)?;
    let bus = Bus::new();
    outbox::publish_pending(&db, &bus)?;
    // Auto local live backup on every open (offline, WAL-safe) + prune. Failure is
    // recorded in the report, never fatal.
    let name = format!("auto-{}.db", now().strftime("%Y%m%d-%H%M%S"));
    match backup::backup(&db, &roots.root, &name).and_then(|p| {
        backup::prune(&roots.root, KEEP_BACKUPS)?;
        backup::verify_backup(&p)?;
        Ok::<_, StorageError>(p)
    }) {
        Ok(p) => report.backup_note = Some(format!("auto backup ok: {}", p.display())),
        Err(e) => report.backup_note = Some(format!("auto backup skipped: {} ({})", e, e.code())),
    }
    registry.set_active(data_dir, profile_id)?;
    Ok(Session {
        data_dir: data_dir.to_path_buf(),
        roots,
        profile,
        db,
        bus,
        report,
        _lock: lock_guard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_profile::Registry;

    #[test]
    fn open_session_wires_everything_and_lock_blocks_second_open() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let mut reg = Registry::load(dir).unwrap();
        let meta = reg.create_profile(dir, "Personal", "#3366ff").unwrap();
        let s = open_session(dir, meta.id).unwrap();
        assert!(s.report.healthy());
        assert!(s.report.backup_note.is_some());
        // second open on the same profile must fail with the lock (single instance)
        let err = open_session(dir, meta.id).unwrap_err();
        assert_eq!(err.code(), "BK-STORE-0004");
        drop(s);
        // after drop the lock is released
        let _s2 = open_session(dir, meta.id).unwrap();
    }
}
