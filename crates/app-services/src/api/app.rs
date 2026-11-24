// SPDX-License-Identifier: GPL-3.0-or-later
//! The composition root: `App::open` wires the whole backend for one profile (M3 backend).
//!
//! Boot order (doc 05 §5.2 `[boot_sequence]`, step `jobs_resume` and the shell-registered
//! sub-steps): `session::open_session_with` runs storage's own boot (lock, DB, migrations,
//! settings, jobs resume, self-check, outbox replay, auto backup) PLUS the extra checks
//! injected here (ledger hash chains, schedules inventory) — storage never depends on
//! ledger, the shell injects plain fn pointers (RULE-ARCH-001, ADR-0025). Then the boot
//! driver posts due schedule occurrences (`schedules::materialize_due`, near window only)
//! and starts the background job worker.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bookkeepy_bus::Bus;
use bookkeepy_contracts::EventEnvelope;
use bookkeepy_core::EntityId;
use bookkeepy_ledger::api::{entries, schedules};
use bookkeepy_storage::StorageError;
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::selfcheck::CheckItem;
use bookkeepy_storage::api::session::{self, Session};
use bookkeepy_storage::api::tx;

use crate::api::worker;

/// App-shell errors (BK-APP area, RULE-ERR-002). Engine codes pass through. Variants are
/// boxed (clippy `result_large_err` — the wrapped errors are large).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Underlying storage failure (code passed through).
    #[error(transparent)]
    Store(Box<StorageError>),
    /// Underlying ledger failure (code passed through).
    #[error(transparent)]
    Ledger(Box<bookkeepy_ledger::LedgerError>),
    /// App-shell-level failure — BK-APP-0001.
    #[error("app shell: {0}")]
    Shell(String),
}

impl From<StorageError> for AppError {
    fn from(e: StorageError) -> Self {
        AppError::Store(Box::new(e))
    }
}

impl From<bookkeepy_ledger::LedgerError> for AppError {
    fn from(e: bookkeepy_ledger::LedgerError) -> Self {
        AppError::Ledger(Box::new(e))
    }
}

impl AppError {
    /// The registered BK-code (RULE-ERR-002).
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Store(e) => e.code(),
            AppError::Ledger(e) => e.code(),
            AppError::Shell(_) => "BK-APP-0001",
        }
    }
}

/// Extra boot check: ledger entry+posting hash chains (registered via `open_session_with`).
fn ledger_chains_check(db: &Db, _root: &Path) -> Result<CheckItem, StorageError> {
    match entries::verify_chains(db) {
        Ok(()) => Ok(CheckItem {
            name: "ledger.chains".into(),
            ok: true,
            detail: Some("entry+posting hash chains verify".into()),
        }),
        Err(e) => Ok(CheckItem {
            name: "ledger.chains".into(),
            ok: false,
            detail: Some(format!("{e} ({})", e.code())),
        }),
    }
}

/// Extra boot check: schedules inventory (informational; the due driver runs after open).
fn schedules_check(db: &Db, _root: &Path) -> Result<CheckItem, StorageError> {
    let n = schedules::list(db)
        .map_err(|e| StorageError::Foreign(Box::new(e)))?
        .len();
    Ok(CheckItem {
        name: "schedules".into(),
        ok: true,
        detail: Some(format!("{n} schedule(s)")),
    })
}

/// One open application profile: the session plus the background job worker.
pub struct App {
    /// The open profile session (db, bus, report, settings, roots…).
    pub session: Session,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl App {
    /// Open the app on `profile_id` inside `data_dir`: full boot sequence + due-schedule
    /// materialization + background job worker. Fails with BK-STORE-0004 if another
    /// instance has the profile open (single instance — the `app` binary focuses the
    /// existing window instead, ADR-0016).
    pub fn open(data_dir: &Path, profile_id: EntityId) -> Result<App, AppError> {
        let session = session::open_session_with(
            data_dir,
            profile_id,
            &[ledger_chains_check, schedules_check],
        )?;
        // boot step `jobs_resume` driver (doc 05 §5.9): post due schedule occurrences
        let today = jiff::Timestamp::now()
            .in_tz("UTC")
            .map_err(|e| AppError::Shell(format!("tz: {e}")))?
            .date(); // UTC date
        let posted = schedules::materialize_due(&session.db, &session.bus, today)?;
        if !posted.is_empty() {
            tracing::info!(
                n = posted.len(),
                "boot materialized due schedule occurrences"
            );
        }
        let stop = Arc::new(AtomicBool::new(false));
        let worker = worker::start(session.db.clone(), session.bus.clone(), stop.clone());
        Ok(App {
            session,
            stop,
            worker: Some(worker),
        })
    }

    /// The profile database.
    pub fn db(&self) -> &Db {
        &self.session.db
    }

    /// The in-process event bus.
    pub fn bus(&self) -> &Bus {
        &self.session.bus
    }

    /// Post an event atomically (outbox + audit in one transaction, then publish).
    pub fn post(&self, ev: &EventEnvelope) -> Result<i64, AppError> {
        Ok(tx::post(&self.session.db, &self.session.bus, ev)?)
    }

    /// Run ONE job-worker iteration synchronously (tests + manual drain). Returns whether
    /// a job was processed.
    pub fn drain_jobs_once(&self) -> Result<bool, AppError> {
        Ok(worker::run_one(
            &self.session.db,
            &self.session.bus,
            worker::DEFAULT_MAX_ATTEMPTS,
        )?)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}
