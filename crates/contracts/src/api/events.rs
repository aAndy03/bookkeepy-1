// SPDX-License-Identifier: GPL-3.0-or-later
//! The domain event envelope (ADR-0009): every state-changing action in Bookkeepy
//! produces one `EventEnvelope` on the bus, and the audit engine (F15, M1) persists
//! envelopes immutably. `kind` is a dotted stable string (`ledger.posted`,
//! `rates.package_installed`, …); `payload` is free-form JSON refined per kind by the
//! owning engine's contract module (variants live next to the engine, RULE-MOD-002).

use bookkeepy_core::EntityId;
use bookkeepy_core::time::{Timestamp, now};
use serde::{Deserialize, Serialize};

/// Who/what caused the event. The audit chain records this verbatim (INV-AUD-001).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Actor {
    /// A human at the keyboard/UI.
    User,
    /// An import batch (id = batch id).
    Import(EntityId),
    /// The scheduler (recurring rules, reminders).
    Scheduler,
    /// A rates-package install/sync from the mirror repo.
    RatesSync,
    /// The updater (app updates only — RULE-NET-001).
    Updater,
    /// Bookkeepy itself (startup, migrations, self-check).
    System,
}

/// One thing that happened. Envelopes are immutable once published (INV-AUD-001);
/// corrections are new envelopes, never edits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique id of the event itself.
    pub id: EntityId,
    /// Stable dotted kind, e.g. `ledger.posted`.
    pub kind: String,
    /// When it happened (UTC, jiff `Timestamp`).
    pub occurred_at: Timestamp,
    /// Who/what caused it.
    pub actor: Actor,
    /// Why — free text or reference (import batch, rule id, package id).
    pub reason: String,
    /// Kind-specific payload (JSON). Engines validate against their own contract.
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Build a new envelope with a fresh uuid-v7 id and `now()` timestamp.
    pub fn new(
        kind: impl Into<String>,
        actor: Actor,
        reason: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: EntityId::new(),
            kind: kind.into(),
            occurred_at: now(),
            actor,
            reason: reason.into(),
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_serde_roundtrip() {
        let ev = EventEnvelope::new(
            "ledger.posted",
            Actor::User,
            "quick add",
            serde_json::json!({"amount": "10.00", "currency": "RON"}),
        );
        let s = serde_json::to_string(&ev).unwrap();
        let back: EventEnvelope = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, ev.id);
        assert_eq!(back.kind, "ledger.posted");
        assert_eq!(back.actor, Actor::User);
        assert_eq!(back.payload["currency"], "RON");
    }

    #[test]
    fn actor_import_carries_batch_id() {
        let batch = EntityId::new();
        let ev = EventEnvelope::new(
            "import.applied",
            Actor::Import(batch),
            "csv",
            serde_json::json!({}),
        );
        let s = serde_json::to_string(&ev.actor).unwrap();
        assert!(s.contains(&batch.as_uuid().to_string()));
    }
}
