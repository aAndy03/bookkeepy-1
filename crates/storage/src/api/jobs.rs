// SPDX-License-Identifier: GPL-3.0-or-later
//! Durable job state machine (RULE-JOB-001, ADR-0015): jobs live in the `jobs` table so a
//! crash never loses one; the startup driver calls `resume_pending` (running → queued),
//! then workers `claim_next` (queued → running, attempts+1), then `complete` / `fail`.
//! `pause`/`resume`/`cancel` are user-facing controls. All transitions are audited
//! via the bus (tx::post) by the owning engine.
//!
//! RULE-JOB-002 (backoff): a failed job is re-queued with `not_before = now +
//! backoff_delay(attempts, id)` (exponential, capped, deterministic jitter) and
//! `claim_next` only picks jobs whose `not_before` has passed — a poison job can never
//! spin in a hot retry loop; after `max_attempts` it lands in `failed` and is surfaced to
//! the user for manual retry.

use bookkeepy_core::time::now;
use rusqlite::OptionalExtension;

use crate::api::conn::Db;
use crate::api::error::StorageError;

/// Job lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// Waiting to be claimed.
    Queued,
    /// Claimed by a worker (crash → back to Queued by resume_pending).
    Running,
    /// User-paused (progress UI: pause/resume/cancel).
    Paused,
    /// Finished successfully.
    Done,
    /// Failed past max attempts — surfaced for manual retry.
    Failed,
    /// User-cancelled.
    Cancelled,
}

impl JobState {
    fn as_str(self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::Paused => "paused",
            JobState::Done => "done",
            JobState::Failed => "failed",
            JobState::Cancelled => "cancelled",
        }
    }
    fn from_str(s: &str) -> Option<JobState> {
        Some(match s {
            "queued" => JobState::Queued,
            "running" => JobState::Running,
            "paused" => JobState::Paused,
            "done" => JobState::Done,
            "failed" => JobState::Failed,
            "cancelled" => JobState::Cancelled,
            _ => return None,
        })
    }
}

/// A job row.
#[derive(Debug, Clone)]
pub struct Job {
    /// Job id.
    pub id: String,
    /// Kind (dotted, e.g. `import.csv`).
    pub kind: String,
    /// Current state.
    pub state: JobState,
    /// Free-form JSON payload.
    pub payload: String,
    /// Attempts so far.
    pub attempts: i64,
    /// Earliest instant (RFC-3339 UTC) this job may be claimed again after a failure
    /// (RULE-JOB-002 backoff). None = immediately.
    pub not_before: Option<String>,
}

fn row_to_job(r: &rusqlite::Row) -> rusqlite::Result<Job> {
    let state: String = r.get(2)?;
    Ok(Job {
        id: r.get(0)?,
        kind: r.get(1)?,
        state: JobState::from_str(&state).unwrap_or(JobState::Queued),
        payload: r.get(3)?,
        attempts: r.get(4)?,
        not_before: r.get(5)?,
    })
}

/// Backoff before a failed job may be re-claimed (RULE-JOB-002): exponential
/// 2s, 4s, 8s … capped at 5 min, plus deterministic jitter derived from the job id
/// (no RNG dependency; stable in tests). `attempts` is the attempt that just failed.
pub fn backoff_delay(attempts: i64, id: &str) -> std::time::Duration {
    let exp = (attempts.max(1) - 1).min(8) as u32;
    let secs = 2u64.saturating_mul(1u64 << exp).min(300);
    let h = id
        .bytes()
        .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(u64::from(b)));
    std::time::Duration::from_millis(secs * 1000 + h % 1000)
}

/// Enqueue a new job (idempotent on id).
pub fn enqueue(db: &Db, id: &str, kind: &str, payload: &str) -> Result<(), StorageError> {
    db.with_conn(|c| {
        c.execute(
            "INSERT OR IGNORE INTO jobs (id, kind, state, payload, attempts, updated_at) VALUES (?1, ?2, 'queued', ?3, 0, ?4)",
            rusqlite::params![id, kind, payload, now().to_string()],
        )?;
        Ok(())
    })
}

/// Startup driver: interrupted `running` jobs go back to `queued`. Returns how many.
pub fn resume_pending(db: &Db) -> Result<usize, StorageError> {
    db.with_conn(|c| {
        Ok(c.execute(
            "UPDATE jobs SET state='queued', updated_at=?1 WHERE state='running'",
            rusqlite::params![now().to_string()],
        )?)
    })
}

/// Claim the next queued job whose backoff has elapsed (queued → running, attempts+1).
/// None if no work is due (RULE-JOB-002: `not_before` in the future is not due).
pub fn claim_next(db: &Db) -> Result<Option<Job>, StorageError> {
    db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, kind, state, payload, attempts, not_before FROM jobs
             WHERE state='queued' AND (not_before IS NULL OR not_before <= ?1)
             ORDER BY rowid LIMIT 1",
        )?;
        let found = stmt.query_row([now().to_string()], row_to_job).optional()?;
        if let Some(j) = &found {
            c.execute(
                "UPDATE jobs SET state='running', attempts=attempts+1, updated_at=?1 WHERE id=?2",
                rusqlite::params![now().to_string(), j.id],
            )?;
            tracing::debug!(job = %j.id, kind = %j.kind, attempts = j.attempts + 1, "job claimed");
        }
        Ok(found)
    })
}

/// Mark a job done.
pub fn complete(db: &Db, id: &str) -> Result<(), StorageError> {
    transition(db, id, JobState::Done)
}

/// Mark a job failed; re-queues with backoff (`not_before`, RULE-JOB-002) until
/// `max_attempts` is reached, then `failed` (surfaced for manual retry).
pub fn fail(db: &Db, id: &str, max_attempts: i64) -> Result<JobState, StorageError> {
    db.with_conn(|c| {
        let attempts: i64 =
            c.query_row("SELECT attempts FROM jobs WHERE id=?1", [id], |r| r.get(0))?;
        let next = if attempts >= max_attempts {
            JobState::Failed
        } else {
            JobState::Queued
        };
        let not_before = match next {
            JobState::Queued => {
                let d = backoff_delay(attempts, id);
                let ts = now()
                    .checked_add(jiff::Span::new().milliseconds(d.as_millis() as i64))
                    .map_err(|e| {
                        StorageError::Foreign(Box::new(std::io::Error::other(e.to_string())))
                    })?;
                tracing::warn!(
                    job = id,
                    attempts,
                    retry_in_ms = d.as_millis() as u64,
                    "job failed; re-queued with backoff"
                );
                Some(ts.to_string())
            }
            _ => {
                tracing::error!(job = id, attempts, "job failed permanently");
                None
            }
        };
        c.execute(
            "UPDATE jobs SET state=?1, not_before=?2, updated_at=?3 WHERE id=?4",
            rusqlite::params![next.as_str(), not_before, now().to_string(), id],
        )?;
        Ok(next)
    })
}

/// Pause / resume / cancel (user controls).
pub fn pause(db: &Db, id: &str) -> Result<(), StorageError> {
    transition(db, id, JobState::Paused)
}

/// Resume a paused job → queued.
pub fn resume(db: &Db, id: &str) -> Result<(), StorageError> {
    transition(db, id, JobState::Queued)
}

/// Cancel a job (terminal).
pub fn cancel(db: &Db, id: &str) -> Result<(), StorageError> {
    transition(db, id, JobState::Cancelled)
}

fn transition(db: &Db, id: &str, to: JobState) -> Result<(), StorageError> {
    db.with_conn(|c| {
        c.execute(
            "UPDATE jobs SET state=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![to.as_str(), now().to_string(), id],
        )?;
        Ok(())
    })
}

/// List jobs by state (progress UI).
pub fn list(db: &Db, state: JobState) -> Result<Vec<Job>, StorageError> {
    db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, kind, state, payload, attempts, not_before FROM jobs WHERE state=?1 ORDER BY rowid",
        )?;
        Ok(stmt
            .query_map([state.as_str()], row_to_job)?
            .collect::<Result<Vec<_>, _>>()?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simulate time passing: clear backoff so the job is due again.
    fn clear_backoff(db: &Db, id: &str) {
        db.with_conn(|c| {
            c.execute("UPDATE jobs SET not_before=NULL WHERE id=?1", [id])?;
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn lifecycle_queued_running_done_and_retry_then_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        enqueue(&db, "j1", "import.csv", "{}").unwrap();
        assert_eq!(resume_pending(&db).unwrap(), 0);
        let j = claim_next(&db).unwrap().unwrap();
        assert_eq!(j.state, JobState::Queued); // claimed row snapshot is pre-transition
        assert!(list(&db, JobState::Running).unwrap().len() == 1);
        // crash simulation: running -> queued
        assert_eq!(resume_pending(&db).unwrap(), 1);
        assert_eq!(fail(&db, "j1", 2).unwrap(), JobState::Queued); // attempts 1 < 2
        // RULE-JOB-002: the re-queued job is NOT due until not_before passes
        assert!(claim_next(&db).unwrap().is_none());
        assert!(list(&db, JobState::Queued).unwrap()[0].not_before.is_some());
        clear_backoff(&db, "j1"); // time passes…
        claim_next(&db).unwrap();
        assert_eq!(fail(&db, "j1", 2).unwrap(), JobState::Failed); // attempts 2 >= 2
        pause(&db, "j1").ok(); // failed is not pausable in UI terms, but transition is mechanical
        cancel(&db, "j1").unwrap();
        assert_eq!(list(&db, JobState::Cancelled).unwrap().len(), 1);
        complete(&db, "j1").unwrap();
    }

    #[test]
    fn backoff_grows_exponentially_and_is_capped() {
        let d1 = backoff_delay(1, "j").as_secs();
        let d2 = backoff_delay(2, "j").as_secs();
        let d3 = backoff_delay(3, "j").as_secs();
        assert!(d1 >= 2 && d2 >= 4 && d3 >= 8);
        assert!(backoff_delay(100, "j").as_secs() <= 301); // cap 300s + ≤1s jitter
        // deterministic: same id → same delay
        assert_eq!(backoff_delay(3, "job-x"), backoff_delay(3, "job-x"));
    }
}
