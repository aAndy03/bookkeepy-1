// SPDX-License-Identifier: GPL-3.0-or-later
//! The background job worker (RULE-JOB-001/002): a thread that claims due jobs
//! (`claim_next` honors `not_before` backoff), dispatches by kind, and completes or
//! fails them. Unknown kinds fail fast (max_attempts=1) so they surface for manual retry
//! instead of spinning. `run_one` is the synchronous single iteration (tests, manual
//! drain); `start` spawns the loop.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use bookkeepy_bus::Bus;
use bookkeepy_ledger::api::schedules;
use bookkeepy_storage::StorageError;
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::jobs::{self, Job};

/// Default max attempts for a job before it lands in `failed` (manual retry surfaced).
pub const DEFAULT_MAX_ATTEMPTS: i64 = 5;

/// Dispatch one claimed job by kind. Returns the materialized count for
/// `schedule.materialize`, 0 otherwise.
fn dispatch(db: &Db, bus: &Bus, job: &Job, max_attempts: i64) -> Result<usize, StorageError> {
    match job.kind.as_str() {
        "schedule.materialize" => schedules::run_claimed_job(db, bus, job, max_attempts)
            .map(|v| v.len())
            .map_err(|e| StorageError::Foreign(Box::new(e))),
        other => {
            // unknown kind (engine not wired yet): fail fast — surfaced, no hot loop
            tracing::warn!(
                kind = other,
                job = job.id,
                "unknown job kind; failing fast for manual retry"
            );
            Ok(0)
        }
    }
}

/// Run ONE worker iteration: claim the next due job, dispatch, complete/fail. Returns
/// whether a job was processed (false = idle).
pub fn run_one(db: &Db, bus: &Bus, max_attempts: i64) -> Result<bool, StorageError> {
    let Some(job) = jobs::claim_next(db)? else {
        return Ok(false);
    };
    if job.kind != "schedule.materialize" {
        let _ = jobs::fail(db, &job.id, 1)?; // unknown kind → failed (surfaced), no spin
        return Ok(true);
    }
    match dispatch(db, bus, &job, max_attempts) {
        Ok(_) => {
            jobs::complete(db, &job.id)?;
        }
        Err(e) => {
            tracing::warn!(job = job.id, code = e.code(), "job failed: {e}");
            jobs::fail(db, &job.id, max_attempts)?;
        }
    }
    Ok(true)
}

/// Spawn the background worker loop. Stops when `stop` is set (App::drop).
pub fn start(db: Db, bus: Bus, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while !stop.load(Ordering::SeqCst) {
            match run_one(&db, &bus, DEFAULT_MAX_ATTEMPTS) {
                Ok(true) => {}
                Ok(false) => std::thread::sleep(Duration::from_millis(100)),
                Err(e) => {
                    tracing::error!(code = e.code(), "job worker error: {e}");
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }
        tracing::debug!("job worker stopped");
    })
}
