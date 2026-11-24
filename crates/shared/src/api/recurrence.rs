// SPDX-License-Identifier: GPL-3.0-or-later
//! The shared `recurrence` helper (doc 04 §4.2 helpers registry, RULE-RECUR-001…003,
//! ADR-0020) — pure schedule math, no I/O, so it is proptest-able and reusable by the
//! ledger (materialization) and the timeline (projection rendering).
//!
//! - RULE-RECUR-001: expansion is bounded and lazy — `expand` only produces dates inside
//!   the requested window, capped at [`MAX_PER_EXPANSION`]; the UI never sees unbounded
//!   ranges.
//! - RULE-RECUR-002: past occurrences are reconciled BY DEFAULT — that is a projection
//!   concern of the caller (ledger::schedules marks them so); this helper just yields dates.
//! - Exceptions (skip / modify one occurrence) are stored on the schedule and applied here.

use jiff::civil::Date;
use serde::{Deserialize, Serialize};

/// Max occurrences one expansion may yield (RULE-RECUR-001 cap).
pub const MAX_PER_EXPANSION: usize = 1000;

/// Recurrence frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    /// Every `interval` days.
    Daily,
    /// Every `interval` weeks (anchored to the start date's weekday).
    Weekly,
    /// Every `interval` months (day-of-month clamped, e.g. Jan 31 → Feb 28).
    Monthly,
    /// Every `interval` years (Feb 29 → Feb 28 in non-leap years).
    Yearly,
}

impl Frequency {
    /// DB string.
    pub fn as_str(self) -> &'static str {
        match self {
            Frequency::Daily => "daily",
            Frequency::Weekly => "weekly",
            Frequency::Monthly => "monthly",
            Frequency::Yearly => "yearly",
        }
    }

    /// Parse a DB string.
    pub fn from_db(s: &str) -> Option<Frequency> {
        Some(match s {
            "daily" => Frequency::Daily,
            "weekly" => Frequency::Weekly,
            "monthly" => Frequency::Monthly,
            "yearly" => Frequency::Yearly,
            _ => return None,
        })
    }
}

/// How a schedule ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum EndRule {
    /// Never ends.
    Never,
    /// Ends on this date (inclusive).
    OnDate(Date),
    /// Ends after this many occurrences (inclusive of the last).
    AfterCount(u32),
}

/// One exception on a schedule (linked both ways: the occurrence key is the date).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Exception {
    /// Skip this occurrence entirely.
    Skip,
    /// Modify this occurrence (JSON patch over the template, e.g. a different amount).
    Modify {
        /// Patch over the schedule template.
        patch: serde_json::Value,
    },
}

/// A recurrence rule (the schedule's SSOT — rules, not stored occurrence rows).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleRule {
    /// Frequency.
    pub frequency: Frequency,
    /// Every Nth frequency unit (≥ 1).
    pub interval: u32,
    /// First occurrence date (inclusive).
    pub start: Date,
    /// End rule.
    pub end: EndRule,
    /// Exceptions keyed by occurrence date.
    #[serde(default)]
    pub exceptions: Vec<(Date, Exception)>,
}

/// One expanded occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    /// Occurrence date.
    pub date: Date,
    /// Exception applied, if any.
    pub exception: Option<Exception>,
}

/// Expansion result — bounded by [`MAX_PER_EXPANSION`].
#[derive(Debug, Clone)]
pub struct Expansion {
    /// Occurrences in the window (exceptions applied; skip-exceptions removed).
    pub occurrences: Vec<Occurrence>,
    /// True when the cap cut the expansion short ("show more" in the UI).
    pub capped: bool,
}

/// Validate a rule (BK-RECUR-0001 conditions): interval ≥ 1, start ≤ end date, count ≥ 1.
pub fn validate(rule: &ScheduleRule) -> Result<(), String> {
    if rule.interval == 0 {
        return Err("interval must be >= 1".into());
    }
    match &rule.end {
        EndRule::OnDate(d) if *d < rule.start => Err("end date before start".into()),
        EndRule::AfterCount(0) => Err("count must be >= 1".into()),
        _ => Ok(()),
    }
}

/// Days in month `(year, month)`, or None if out of jiff range.
fn days_in_month(year: i64, month: i64) -> Option<i8> {
    Some(
        Date::new(i16::try_from(year).ok()?, i8::try_from(month).ok()?, 1)
            .ok()?
            .days_in_month(),
    )
}

/// The k-th occurrence date of a rule (k = 0 is `start`). ANCHOR-BASED (RULE-RECUR-003):
/// monthly/yearly clamp the ANCHOR day into each target month, so 2026-01-31 monthly
/// yields 02-28, 03-31, 04-30… — never drifts to the 28th forever.
pub fn nth(freq: Frequency, interval: u32, start: Date, k: u64) -> Option<Date> {
    let n = (interval.max(1) as i64).checked_mul(k as i64)?;
    match freq {
        Frequency::Daily => start.checked_add(jiff::Span::new().days(n)).ok(),
        Frequency::Weekly => start.checked_add(jiff::Span::new().weeks(n)).ok(),
        Frequency::Monthly => {
            let total = i64::from(start.year()) * 12 + i64::from(start.month()) - 1 + n;
            let y = total.div_euclid(12);
            let m = total.rem_euclid(12) + 1;
            let day = start.day().min(days_in_month(y, m)?);
            Date::new(i16::try_from(y).ok()?, i8::try_from(m).ok()?, day).ok()
        }
        Frequency::Yearly => {
            let y = i64::from(start.year()) + n;
            let day = start.day().min(days_in_month(y, i64::from(start.month()))?);
            Date::new(i16::try_from(y).ok()?, start.month(), day).ok()
        }
    }
}

/// Step one frequency unit from `d` (day-of-month clamped for monthly/yearly).
/// Equivalent to `nth(freq, interval, d, 1)`.
pub fn step(freq: Frequency, interval: u32, d: Date) -> Option<Date> {
    nth(freq, interval, d, 1)
}

/// Expand occurrences of `rule` inside the window `[from, to]` (inclusive), bounded by
/// [`MAX_PER_EXPANSION`]. Pure: no I/O, no "today" hidden inside — callers pass the window
/// (timeline scope for projection; near-window for materialization).
pub fn expand(rule: &ScheduleRule, from: Date, to: Date) -> Result<Expansion, String> {
    validate(rule)?;
    if from > to {
        return Ok(Expansion {
            occurrences: Vec::new(),
            capped: false,
        });
    }
    let mut out = Vec::new();
    let mut capped = false;
    // Anchor-indexed iteration: occurrence k is `nth(rule, k)` — clamping never drifts.
    // Hard iteration guard keeps expansion O(window) even for daily rules from 2005.
    const ITER_GUARD: u64 = 100_000;
    let mut k: u64 = 0;
    while let Some(d) = nth(rule.frequency, rule.interval, rule.start, k) {
        if d > to {
            break;
        }
        match &rule.end {
            EndRule::Never => {}
            EndRule::OnDate(end) if d > *end => break,
            EndRule::AfterCount(n) if k >= u64::from(*n) => break,
            _ => {}
        }
        if d >= from {
            let exception = rule
                .exceptions
                .iter()
                .find(|(ed, _)| *ed == d)
                .map(|(_, e)| e.clone());
            if !matches!(exception, Some(Exception::Skip)) {
                out.push(Occurrence {
                    date: d,
                    exception: exception.clone(),
                });
            }
            if out.len() >= MAX_PER_EXPANSION {
                capped = true;
                break;
            }
        }
        k += 1;
        if k > ITER_GUARD {
            capped = true;
            break;
        }
    }
    Ok(Expansion {
        occurrences: out,
        capped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    #[test]
    fn monthly_clamps_month_end() {
        let rule = ScheduleRule {
            frequency: Frequency::Monthly,
            interval: 1,
            start: d("2026-01-31"),
            end: EndRule::Never,
            exceptions: vec![],
        };
        let e = expand(&rule, d("2026-01-01"), d("2026-04-30")).unwrap();
        let dates: Vec<String> = e.occurrences.iter().map(|o| o.date.to_string()).collect();
        assert_eq!(
            dates,
            ["2026-01-31", "2026-02-28", "2026-03-31", "2026-04-30"]
        );
    }

    #[test]
    fn window_is_bounded_and_exceptions_apply() {
        let rule = ScheduleRule {
            frequency: Frequency::Daily,
            interval: 1,
            start: d("2026-01-01"),
            end: EndRule::AfterCount(10),
            exceptions: vec![(d("2026-01-03"), Exception::Skip)],
        };
        let e = expand(&rule, d("2026-01-02"), d("2026-01-05")).unwrap();
        assert_eq!(e.occurrences.len(), 3); // 2,4,5 (3 skipped)
        assert!(!e.capped);
        // after_count caps total occurrences
        let e = expand(&rule, d("2026-01-01"), d("2027-01-01")).unwrap();
        assert_eq!(e.occurrences.len(), 9); // 10 minus the skipped one
    }

    proptest! {
        #[test]
        fn expansion_never_exceeds_cap_and_stays_in_window(
            interval in 1u32..5,
            start_day in 1i8..28,
            window_start in 0i32..400,
            window_len in 0i32..60,
        ) {
            let rule = ScheduleRule {
                frequency: Frequency::Daily,
                interval,
                start: d("2026-01-01").with().day(start_day).build().unwrap(),
                end: EndRule::Never,
                exceptions: vec![],
            };
            let from = d("2026-01-01").checked_add(jiff::Span::new().days(window_start)).unwrap();
            let to = from.checked_add(jiff::Span::new().days(window_len)).unwrap();
            let e = expand(&rule, from, to).unwrap();
            prop_assert!(e.occurrences.len() <= MAX_PER_EXPANSION);
            for o in &e.occurrences {
                prop_assert!(o.date >= from && o.date <= to);
            }
            // strictly increasing
            for w in e.occurrences.windows(2) {
                prop_assert!(w[0].date < w[1].date);
            }
        }
    }
}
