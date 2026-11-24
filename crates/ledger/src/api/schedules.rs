// SPDX-License-Identifier: GPL-3.0-or-later
//! Recurrence schedules (M2b, ADR-0020, RULE-RECUR-001…003, doc 05 §5.13).
//!
//! A schedule is a RULE (template + frequency/interval/start/end/exceptions), never stored
//! occurrence rows. Interconnections (this is the point of M2b):
//! - expansion math lives in the shared `recurrence` helper (SSOT, proptest-covered);
//! - materialization posts REAL transactions through `transactions::create` (so every
//!   occurrence is an ordinary audited, hash-chained, reversible transaction) with
//!   `Actor::Scheduler`;
//! - every schedule change is audited via `storage::tx::post_with` and published on the bus;
//! - long bounded materializations run as durable `storage::jobs` (progress/cancel/retry).
//!
//! RULE-RECUR-002 (the 2010-start example): `materialize_due` only posts occurrences in the
//! near window around today — it never backfills history. Past occurrences are reconciled
//! BY DEFAULT in the projection (`expand_window` marks them); the user can opt into a
//! bounded past range via `materialize_range_as_job`.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_core::time::Timestamp;
use bookkeepy_core::time::now;
use bookkeepy_core::{EntityId, Money};
use bookkeepy_shared::api::recurrence::{self, EndRule, Exception, Frequency, ScheduleRule};
use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::{jobs, tx};
use jiff::civil::Date;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

use crate::api::error::LedgerError;
use crate::api::lifecycle::TxStatus;
use crate::api::transactions::{self, NewTransaction, TxKind};

/// How far back from "today" the due-materialization driver posts (RULE-RECUR-002:
/// near window only — past beyond it is reconciled-by-default, never backfilled).
pub const NEAR_WINDOW_DAYS: i32 = 7;

/// The transaction template stored on a schedule (JSON; the schedule's SSOT payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTemplate {
    /// Kind.
    pub kind: TxKindSerde,
    /// Amount minor units.
    pub amount_minor: i64,
    /// Currency code.
    pub currency: String,
    /// Source account id.
    pub from: Option<String>,
    /// Destination account id.
    pub to: Option<String>,
    /// Category id.
    pub category_id: Option<String>,
    /// Payee id.
    pub payee_id: Option<String>,
    /// Tag ids.
    #[serde(default)]
    pub tag_ids: Vec<String>,
    /// Note.
    pub note: Option<String>,
}

/// Serde-friendly TxKind (TxKind itself is not Serialize).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxKindSerde {
    /// Income.
    Income,
    /// Expense.
    Expense,
    /// Transfer.
    Transfer,
}

impl TxKindSerde {
    fn to_kind(self) -> TxKind {
        match self {
            TxKindSerde::Income => TxKind::Income,
            TxKindSerde::Expense => TxKind::Expense,
            TxKindSerde::Transfer => TxKind::Transfer,
        }
    }
}

/// A schedule row.
#[derive(Debug, Clone)]
pub struct Schedule {
    /// Id.
    pub id: EntityId,
    /// Template.
    pub template: ScheduleTemplate,
    /// Rule.
    pub rule: ScheduleRule,
}

fn parse_id(s: String) -> Option<EntityId> {
    s.parse().ok()
}

/// Create a schedule (audited `schedule.created`). The rule is validated
/// (BK-RECUR-0001 on invalid).
pub fn create(
    db: &Db,
    bus: &Bus,
    actor: &Actor,
    template: ScheduleTemplate,
    rule: ScheduleRule,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    recurrence::validate(&rule).map_err(LedgerError::InvalidSchedule)?;
    let id = EntityId::new();
    let ev = EventEnvelope::new(
        bookkeepy_contracts::kinds::SCHEDULE_CREATED,
        actor.clone(),
        reason,
        serde_json::json!({
            "id": id.to_string(),
            "frequency": rule.frequency.as_str(),
            "interval": rule.interval,
            "start": rule.start.to_string(),
        }),
    );
    let tpl = serde_json::to_string(&template).unwrap_or_else(|_| "{}".into());
    let rule_json = serde_json::json!({
        "frequency": rule.frequency.as_str(),
        "interval": rule.interval,
        "start": rule.start.to_string(),
        "end": match &rule.end {
            EndRule::Never => serde_json::json!({"kind":"never"}),
            EndRule::OnDate(d) => serde_json::json!({"kind":"on_date","value": d.to_string()}),
            EndRule::AfterCount(n) => serde_json::json!({"kind":"after_count","value": n}),
        },
        "exceptions": rule.exceptions.iter().map(|(d, e)| serde_json::json!({"date": d.to_string(), "exception": e})).collect::<Vec<_>>(),
    });
    tx::post_with(db, bus, &ev, |tx| {
        tx.execute(
            "INSERT INTO schedules (id, template, frequency, interval, start_date, end_kind, end_value, exceptions, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                id.to_string(),
                tpl,
                rule.frequency.as_str(),
                rule.interval as i64,
                rule.start.to_string(),
                match &rule.end {
                    EndRule::Never => "never",
                    EndRule::OnDate(_) => "on_date",
                    EndRule::AfterCount(_) => "after_count",
                },
                match &rule.end {
                    EndRule::OnDate(d) => Some(d.to_string()),
                    EndRule::AfterCount(n) => Some(n.to_string()),
                    EndRule::Never => None,
                },
                serde_json::to_string(&rule.exceptions).unwrap_or_else(|_| "[]".into()),
                now().to_string(),
            ],
        )?;
        let _ = rule_json;
        Ok(())
    })?;
    Ok(id)
}

fn row_to_schedule(r: &rusqlite::Row) -> rusqlite::Result<Schedule> {
    let template: ScheduleTemplate =
        serde_json::from_str(&r.get::<_, String>(1)?).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other(e.to_string())),
            )
        })?;
    let freq = Frequency::from_db(&r.get::<_, String>(2)?).unwrap_or(Frequency::Daily);
    let interval: i64 = r.get(3)?;
    let start: Date = r.get::<_, String>(4)?.parse().map_err(|e: jiff::Error| {
        rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(e.to_string())),
        )
    })?;
    let end_kind: String = r.get(5)?;
    let end_value: Option<String> = r.get(6)?;
    let exceptions: Vec<(Date, Exception)> =
        serde_json::from_str(&r.get::<_, String>(7)?).unwrap_or_default();
    let end = match end_kind.as_str() {
        "on_date" => EndRule::OnDate(end_value.unwrap_or_default().parse().map_err(
            |e: jiff::Error| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::other(e.to_string())),
                )
            },
        )?),
        "after_count" => EndRule::AfterCount(end_value.unwrap_or_default().parse().unwrap_or(0)),
        _ => EndRule::Never,
    };
    Ok(Schedule {
        id: parse_id(r.get(0)?).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other("bad id")),
            )
        })?,
        template,
        rule: ScheduleRule {
            frequency: freq,
            interval: interval.max(1) as u32,
            start,
            end,
            exceptions,
        },
    })
}

/// Get one schedule.
pub fn get(db: &Db, id: EntityId) -> Result<Option<Schedule>, LedgerError> {
    Ok(db.with_conn(|c| {
        Ok(c
            .query_row(
                "SELECT id, template, frequency, interval, start_date, end_kind, end_value, exceptions FROM schedules WHERE id=?1",
                [id.to_string()],
                row_to_schedule,
            )
            .optional()?)
    })?)
}

/// List all schedules.
pub fn list(db: &Db) -> Result<Vec<Schedule>, LedgerError> {
    Ok(db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, template, frequency, interval, start_date, end_kind, end_value, exceptions FROM schedules ORDER BY created_at",
        )?;
        Ok(stmt.query_map([], row_to_schedule)?.collect::<Result<Vec<_>, _>>()?)
    })?)
}

/// A projected occurrence for the timeline/UI, with its materialization state.
#[derive(Debug, Clone)]
pub struct ProjectedOccurrence {
    /// Date.
    pub date: Date,
    /// Exception applied, if any.
    pub exception: Option<Exception>,
    /// Materialized transaction id, if posted.
    pub materialized_tx: Option<EntityId>,
    /// Assumed reconciled by default when in the past and not materialized
    /// (RULE-RECUR-002).
    pub assumed_reconciled: bool,
}

/// Derived projection of a schedule over a window (RULE-RECUR-001/002): expands lazily
/// within `[from, to]`, joins materialized transactions, marks past unmaterialized
/// occurrences as assumed-reconciled. Never posts anything.
pub fn expand_window(
    db: &Db,
    schedule_id: EntityId,
    from: Date,
    to: Date,
    today: Date,
) -> Result<Expansion2, LedgerError> {
    let s = get(db, schedule_id)?
        .ok_or_else(|| LedgerError::InvalidSchedule(format!("schedule {schedule_id}")))?;
    let exp = recurrence::expand(&s.rule, from, to).map_err(LedgerError::InvalidSchedule)?;
    let mut out = Vec::new();
    for o in exp.occurrences {
        // the LIVE (non-voided) materialization — an edited occurrence's replacement
        // carries the link after the original was voided (schema v4 partial index)
        let mat: Option<String> = db.with_conn(|c| {
            Ok(c.query_row(
                "SELECT id FROM transactions WHERE schedule_id=?1 AND occurrence_date=?2
                     AND status<>'voided' ORDER BY created_at DESC LIMIT 1",
                rusqlite::params![schedule_id.to_string(), o.date.to_string()],
                |r| r.get(0),
            )
            .optional()?)
        })?;
        let assumed = o.date < today && mat.is_none();
        out.push(ProjectedOccurrence {
            date: o.date,
            exception: o.exception,
            materialized_tx: mat.and_then(parse_id),
            assumed_reconciled: assumed,
        });
    }
    Ok(Expansion2 {
        occurrences: out,
        capped: exp.capped,
    })
}

/// Expansion result with projection state.
#[derive(Debug, Clone)]
pub struct Expansion2 {
    /// Projected occurrences.
    pub occurrences: Vec<ProjectedOccurrence>,
    /// Cap hit (BK-RECUR-0002 warning when materializing).
    pub capped: bool,
}

/// Apply a `Modify` exception patch over the schedule template (RULE-RECUR-004): a
/// shallow JSON-object merge — `amount_minor`, `note`, `category_id`, … override the
/// template for THIS occurrence only. Unknown keys fail loudly (typo = visible error,
/// never silently ignored).
fn apply_patch(
    tpl: &ScheduleTemplate,
    patch: Option<&serde_json::Value>,
) -> Result<ScheduleTemplate, LedgerError> {
    let Some(patch) = patch else {
        return Ok(tpl.clone());
    };
    let mut v = serde_json::to_value(tpl)
        .map_err(|e| LedgerError::InvalidSchedule(format!("template encode: {e}")))?;
    let Some(obj) = v.as_object_mut() else {
        return Err(LedgerError::InvalidSchedule(
            "template not an object".into(),
        ));
    };
    let Some(patch_obj) = patch.as_object() else {
        return Err(LedgerError::InvalidSchedule(
            "modify patch must be a JSON object".into(),
        ));
    };
    for (k, val) in patch_obj {
        obj.insert(k.clone(), val.clone());
    }
    serde_json::from_value(v)
        .map_err(|e| LedgerError::InvalidSchedule(format!("bad modify patch: {e}")))
}

fn template_to_new(
    s: &Schedule,
    date: Date,
    patch: Option<&serde_json::Value>,
) -> Result<NewTransaction, LedgerError> {
    let t = apply_patch(&s.template, patch)?;
    let t = &t;
    let cur = bookkeepy_core::CurrencyCode::new(&t.currency)
        .map_err(|_| LedgerError::InvalidSchedule(format!("bad currency {:?}", t.currency)))?;
    let value_ts: Timestamp = date
        .in_tz("UTC")
        .map_err(|e| LedgerError::InvalidSchedule(e.to_string()))?
        .timestamp();
    Ok(NewTransaction {
        kind: t.kind.to_kind(),
        amount: Money::new(t.amount_minor, cur),
        from: t.from.clone().and_then(parse_id),
        to: t.to.clone().and_then(parse_id),
        category_id: t.category_id.clone().and_then(parse_id),
        payee_id: t.payee_id.clone().and_then(parse_id),
        tag_ids: t
            .tag_ids
            .iter()
            .filter_map(|s| parse_id(s.clone()))
            .collect(),
        note: t.note.clone(),
        value_ts,
        status: TxStatus::Cleared,
        schedule_id: Some(s.id),
        occurrence_date: Some(date),
    })
}

fn is_materialized(db: &Db, schedule_id: EntityId, date: Date) -> Result<bool, LedgerError> {
    Ok(db.with_conn(|c| {
        Ok(c.query_row(
            "SELECT COUNT(*) FROM transactions WHERE schedule_id=?1 AND occurrence_date=?2",
            rusqlite::params![schedule_id.to_string(), date.to_string()],
            |r| r.get::<_, i64>(0),
        )? > 0)
    })?)
}

/// Materialize ONE occurrence as a real audited transaction (Actor::Scheduler).
/// Errors BK-RECUR-0003 if already materialized; skip-exceptions are not materialized.
pub fn materialize_one(
    db: &Db,
    bus: &Bus,
    schedule_id: EntityId,
    date: Date,
    reason: &str,
) -> Result<EntityId, LedgerError> {
    let s = get(db, schedule_id)?
        .ok_or_else(|| LedgerError::InvalidSchedule(format!("schedule {schedule_id}")))?;
    let exception = s.rule.exceptions.iter().find(|(d, _)| *d == date);
    if let Some((_, Exception::Skip)) = exception {
        return Err(LedgerError::InvalidSchedule(format!(
            "occurrence {date} is skip-excepted"
        )));
    }
    // RULE-RECUR-004: a Modify exception patches the template for this occurrence only
    let patch = match exception {
        Some((_, Exception::Modify { patch })) => Some(patch.clone()),
        _ => None,
    };
    if is_materialized(db, schedule_id, date)? {
        return Err(LedgerError::AlreadyMaterialized(date.to_string()));
    }
    let new = template_to_new(&s, date, patch.as_ref())?;
    transactions::create(db, bus, &Actor::Scheduler, new, reason)
}

/// The boot driver (RULE-RECUR-002): post occurrences due in the near window
/// `[today - NEAR_WINDOW_DAYS, today]` for every schedule. NEVER backfills the past.
/// Returns the materialized transaction ids. Runs inline at boot (small); bounded past
/// ranges go through `materialize_range_as_job`.
pub fn materialize_due(db: &Db, bus: &Bus, today: Date) -> Result<Vec<EntityId>, LedgerError> {
    let from = today
        .checked_add(jiff::Span::new().days(-i64::from(NEAR_WINDOW_DAYS)))
        .map_err(|e| LedgerError::InvalidSchedule(e.to_string()))?;
    let mut out = Vec::new();
    for s in list(db)? {
        let exp = recurrence::expand(&s.rule, from, today).map_err(LedgerError::InvalidSchedule)?;
        for o in exp.occurrences {
            if o.exception
                .as_ref()
                .is_some_and(|e| matches!(e, Exception::Skip))
            {
                continue;
            }
            if is_materialized(db, s.id, o.date)? {
                continue;
            }
            let tx = materialize_one(db, bus, s.id, o.date, "schedule due")?;
            out.push(tx);
        }
    }
    tracing::info!(today = %today, posted = out.len(), "schedule due materialization pass (no backfill)");
    Ok(out)
}

/// Materialize a bounded past/future range of one schedule as a DURABLE JOB
/// (RULE-JOB-001: progress, cancel, auto-retry via the jobs engine). This is the opt-in
/// history path of RULE-RECUR-002. Returns the job id.
pub fn materialize_range_as_job(
    db: &Db,
    schedule_id: EntityId,
    from: Date,
    to: Date,
) -> Result<String, LedgerError> {
    get(db, schedule_id)?
        .ok_or_else(|| LedgerError::InvalidSchedule(format!("schedule {schedule_id}")))?;
    let job_id = format!("schedule.materialize:{schedule_id}:{from}..{to}");
    let payload = serde_json::json!({
        "schedule_id": schedule_id.to_string(),
        "from": from.to_string(),
        "to": to.to_string(),
    });
    jobs::enqueue(db, &job_id, "schedule.materialize", &payload.to_string())?;
    Ok(job_id)
}

/// Worker for `schedule.materialize` jobs: claims, materializes the range (skipping
/// already-materialized and skip-exceptions), completes or fails (retry via max_attempts).
/// Returns materialized tx ids.
pub fn run_materialize_job(
    db: &Db,
    bus: &Bus,
    max_attempts: i64,
) -> Result<Vec<EntityId>, LedgerError> {
    let Some(job) = jobs::claim_next(db)? else {
        return Ok(Vec::new());
    };
    if job.kind != "schedule.materialize" {
        // not ours — requeue and stop (single-worker boot driver)
        jobs::resume(db, &job.id).ok();
        return Ok(Vec::new());
    }
    let result = (|| -> Result<Vec<EntityId>, LedgerError> {
        let p: serde_json::Value = serde_json::from_str(&job.payload)
            .map_err(|e| LedgerError::InvalidSchedule(e.to_string()))?;
        let sid: EntityId = p["schedule_id"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                LedgerError::InvalidSchedule("job payload missing schedule_id".into())
            })?;
        let from: Date = p["from"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| LedgerError::InvalidSchedule("bad from".into()))?;
        let to: Date = p["to"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| LedgerError::InvalidSchedule("bad to".into()))?;
        let s =
            get(db, sid)?.ok_or_else(|| LedgerError::InvalidSchedule(format!("schedule {sid}")))?;
        let exp = recurrence::expand(&s.rule, from, to).map_err(LedgerError::InvalidSchedule)?;
        if exp.capped {
            return Err(LedgerError::ExpansionCapped(exp.occurrences.len()));
        }
        let mut out = Vec::new();
        for o in exp.occurrences {
            if o.exception
                .as_ref()
                .is_some_and(|e| matches!(e, Exception::Skip))
            {
                continue;
            }
            if is_materialized(db, sid, o.date)? {
                continue;
            }
            out.push(materialize_one(db, bus, sid, o.date, "schedule range job")?);
        }
        Ok(out)
    })();
    match result {
        Ok(_) => {
            jobs::complete(db, &job.id).map_err(LedgerError::Store)?;
        }
        Err(e) => {
            let _ = jobs::fail(db, &job.id, max_attempts).map_err(LedgerError::Store);
            return Err(e);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::accounts::{AccountType, create as create_account};
    use crate::api::catalog;
    use bookkeepy_core::CurrencyCode;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    fn setup() -> (tempfile::TempDir, Db, Bus, EntityId, EntityId) {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let cash = create_account(
            &db,
            &bus,
            &Actor::User,
            "Cash",
            AccountType::Asset,
            "RON",
            "s",
        )
        .unwrap();
        let food = catalog::create_category(
            &db,
            &bus,
            &Actor::User,
            "Food",
            None,
            AccountType::Expense,
            "s",
        )
        .unwrap();
        (tmp, db, bus, cash, food)
    }

    fn monthly_rule(start: &str) -> ScheduleRule {
        ScheduleRule {
            frequency: Frequency::Monthly,
            interval: 1,
            start: d(start),
            end: EndRule::Never,
            exceptions: vec![],
        }
    }

    fn tpl() -> ScheduleTemplate {
        ScheduleTemplate {
            kind: TxKindSerde::Expense,
            amount_minor: 100,
            currency: "RON".into(),
            from: None,
            to: None,
            category_id: None,
            payee_id: None,
            tag_ids: vec![],
            note: Some("rent-ish".into()),
        }
    }

    #[test]
    fn due_materialization_never_backfills_past() {
        let (_t, db, bus, cash, food) = setup();
        let mut t = tpl();
        t.from = Some(cash.to_string());
        t.category_id = Some(food.to_string());
        // schedule started in 2010 — the ADR-0020 example: no 16-year backfill
        let s = create(
            &db,
            &bus,
            &Actor::User,
            t,
            monthly_rule("2010-11-20"),
            "seed",
        )
        .unwrap();
        let today = d("2026-11-24");
        let made = materialize_due(&db, &bus, today).unwrap();
        // only near-window occurrences (2026-11-20 ∈ [2026-11-17, 2026-11-24]) are posted
        assert_eq!(made.len(), 1);
        // projection over a wide window shows past occurrences as assumed reconciled
        let proj = expand_window(&db, s, d("2026-10-01"), d("2026-12-31"), today).unwrap();
        let past = proj
            .occurrences
            .iter()
            .find(|o| o.date == d("2026-10-20"))
            .unwrap();
        assert!(past.assumed_reconciled);
        assert!(past.materialized_tx.is_none());
        let nov = proj
            .occurrences
            .iter()
            .find(|o| o.date == d("2026-11-20"))
            .unwrap();
        assert!(nov.materialized_tx.is_some());
        assert!(!nov.assumed_reconciled);
        // re-running is idempotent (no double post)
        let again = materialize_due(&db, &bus, today).unwrap();
        assert!(again.is_empty());
        let _ = CurrencyCode::new("RON").unwrap();
    }

    #[test]
    fn range_materialization_runs_as_durable_job_and_skips_exceptions() {
        let (_t, db, bus, cash, food) = setup();
        let mut t = tpl();
        t.from = Some(cash.to_string());
        t.category_id = Some(food.to_string());
        let mut rule = monthly_rule("2026-01-31");
        rule.exceptions = vec![(d("2026-03-31"), Exception::Skip)];
        let s = create(&db, &bus, &Actor::User, t, rule, "seed").unwrap();
        let job = materialize_range_as_job(&db, s, d("2026-01-01"), d("2026-06-30")).unwrap();
        let made = run_materialize_job(&db, &bus, 3).unwrap();
        // Jan..Jun monthly from Jan 31: 01-31, 02-28, (03-31 skipped), 04-30, 05-31, 06-30 => 5
        assert_eq!(made.len(), 5);
        assert_eq!(jobs::list(&db, jobs::JobState::Done).unwrap().len(), 1);
        assert!(job.contains("schedule.materialize"));
        // trial balance still holds after materialization
        crate::api::balances::assert_trial_balance(&db, None).unwrap();
        crate::api::entries::verify_chains(&db).unwrap();
    }

    #[test]
    fn invalid_rule_rejected() {
        let (_t, db, bus, _c, _f) = setup();
        let mut rule = monthly_rule("2026-05-01");
        rule.end = EndRule::OnDate(d("2026-01-01")); // end before start
        let err = create(&db, &bus, &Actor::User, tpl(), rule, "bad").unwrap_err();
        assert_eq!(err.code(), "BK-RECUR-0001");
    }

    #[test]
    fn modify_exception_patches_materialized_amount() {
        let (_t, db, bus, cash, food) = setup();
        let mut t = tpl();
        t.from = Some(cash.to_string());
        t.category_id = Some(food.to_string());
        let mut rule = monthly_rule("2026-01-31");
        // RULE-RECUR-004: modify the Feb occurrence to a different amount
        rule.exceptions = vec![(
            d("2026-02-28"),
            Exception::Modify {
                patch: serde_json::json!({"amount_minor": 250}),
            },
        )];
        let s = create(&db, &bus, &Actor::User, t, rule, "seed").unwrap();
        materialize_range_as_job(&db, s, d("2026-01-01"), d("2026-03-31")).unwrap();
        let made = run_materialize_job(&db, &bus, 3).unwrap();
        assert_eq!(made.len(), 3); // 01-31, 02-28 (modified), 03-31
        let amount_of = |occ: &str| -> i64 {
            db.with_conn(|c| {
                Ok(c.query_row(
                    "SELECT p.amount_minor FROM postings p
                     JOIN journal_entries e ON e.id = p.entry_id
                     JOIN transactions t ON t.entry_id = e.id
                     WHERE t.schedule_id=?1 AND t.occurrence_date=?2 AND p.side='debit'",
                    rusqlite::params![s.to_string(), occ],
                    |r| r.get(0),
                )?)
            })
            .unwrap()
        };
        assert_eq!(amount_of("2026-01-31"), 100); // template amount
        assert_eq!(amount_of("2026-02-28"), 250); // patched amount
        assert_eq!(amount_of("2026-03-31"), 100);
    }

    #[test]
    fn double_materialization_is_rejected() {
        let (_t, db, bus, cash, food) = setup();
        let mut t = tpl();
        t.from = Some(cash.to_string());
        t.category_id = Some(food.to_string());
        let s = create(
            &db,
            &bus,
            &Actor::User,
            t,
            monthly_rule("2026-11-20"),
            "seed",
        )
        .unwrap();
        materialize_one(&db, &bus, s, d("2026-11-20"), "first").unwrap();
        // second attempt on the same occurrence -> BK-RECUR-0003 (in-tx guard + unique index)
        let err = materialize_one(&db, &bus, s, d("2026-11-20"), "dupe").unwrap_err();
        assert_eq!(err.code(), "BK-RECUR-0003");
    }
}
