// SPDX-License-Identifier: GPL-3.0-or-later
//! Durable job state machine (RULE-JOB-001, ADR-0015): jobs live in the `jobs` table so a
//! crash never loses one; the startup driver calls `resume_pending` (running → queued),
//! then workers `claim_next` (queued → running, attempts+1), then `complete` / `fail`
//! (fail re-queues until `max_attempts`, then `failed` — surfaced to the user for manual
//! retry). `pause`/`resume`/`cancel` are user-facing controls. All transitions are audited
//! via the bus (tx::post) by the owning engine.

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
}

fn row_to_job(r: &rusqlite::Row) -> rusqlite::Result<Job> {
    let state: String = r.get(2)?;
    Ok(Job {
        id: r.get(0)?,
        kind: r.get(1)?,
        state: JobState::from_str(&state).unwrap_or(JobState::Queued),
        payload: r.get(3)?,
        attempts: r.get(4)?,
    })
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

/// Claim the next queued job (queued → running, attempts+1). None if no work.
pub fn claim_next(db: &Db) -> Result<Option<Job>, StorageError> {
    db.with_conn(|c| {
        let mut stmt = c.prepare("SELECT id, kind, state, payload, attempts FROM jobs WHERE state='queued' ORDER BY rowid LIMIT 1")?;
        let found = stmt.query_row([], row_to_job).optional()?;
        if let Some(j) = &found {
            c.execute(
                "UPDATE jobs SET state='running', attempts=attempts+1, updated_at=?1 WHERE id=?2",
                rusqlite::params![now().to_string(), j.id],
            )?;
        }
        Ok(found)
    })
}

/// Mark a job done.
pub fn complete(db: &Db, id: &str) -> Result<(), StorageError> {
    transition(db, id, JobState::Done)
}

/// Mark a job failed; re-queues until `max_attempts` is reached, then `failed`.
pub fn fail(db: &Db, id: &str, max_attempts: i64) -> Result<JobState, StorageError> {
    db.with_conn(|c| {
        let attempts: i64 =
            c.query_row("SELECT attempts FROM jobs WHERE id=?1", [id], |r| r.get(0))?;
        let next = if attempts >= max_attempts {
            JobState::Failed
        } else {
            JobState::Queued
        };
        c.execute(
            "UPDATE jobs SET state=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![next.as_str(), now().to_string(), id],
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
            "SELECT id, kind, state, payload, attempts FROM jobs WHERE state=?1 ORDER BY rowid",
        )?;
        Ok(stmt
            .query_map([state.as_str()], row_to_job)?
            .collect::<Result<Vec<_>, _>>()?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        claim_next(&db).unwrap();
        assert_eq!(fail(&db, "j1", 2).unwrap(), JobState::Failed); // attempts 2 >= 2
        pause(&db, "j1").ok(); // failed is not pausable in UI terms, but transition is mechanical
        cancel(&db, "j1").unwrap();
        assert_eq!(list(&db, JobState::Cancelled).unwrap().len(), 1);
        complete(&db, "j1").unwrap();
    }
}
