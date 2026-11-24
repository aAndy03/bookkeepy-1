// SPDX-License-Identifier: GPL-3.0-or-later
//! Atomic posting (ADR-0009): one transaction writes the durable outbox row AND the audit
//! row; after commit the envelope is published to the in-process bus. A crash between
//! commit and publish is healed by `publish_pending` at next startup (exactly-once).

use bookkeepy_bus::Bus;
use bookkeepy_contracts::EventEnvelope;

use crate::api::audit;
use crate::api::conn::Db;
use crate::api::error::StorageError;
use crate::api::outbox;

/// Post an event atomically: outbox + audit in one transaction, then publish to the bus.
/// Returns the audit seq.
pub fn post(db: &Db, bus: &Bus, ev: &EventEnvelope) -> Result<i64, StorageError> {
    let seq = db.with_conn(|c| {
        let tx = c.transaction()?;
        outbox::enqueue(&tx, ev)?;
        let seq = audit::append(
            &tx,
            &ev.id.to_string(),
            &ev.kind,
            &ev.occurred_at.to_string(),
            &serde_json::to_string(&ev.actor).unwrap_or_default(),
            &ev.reason,
            &ev.payload.to_string(),
        )?;
        tx.commit()?;
        Ok(seq)
    })?;
    outbox::publish_pending(db, bus)?;
    Ok(seq)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_contracts::Actor;
    use std::sync::{Arc, Mutex};

    #[test]
    fn post_writes_outbox_and_audit_and_publishes() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let s2 = seen.clone();
        bus.subscribe(move |ev| s2.lock().unwrap().push(ev.kind.clone()));

        let ev = EventEnvelope::new(
            "ledger.posted",
            Actor::User,
            "quick add",
            serde_json::json!({"a": 1}),
        );
        let seq = post(&db, &bus, &ev).unwrap();
        assert_eq!(seq, 1);
        audit::verify(&db).unwrap();
        assert_eq!(outbox::pending_count(&db).unwrap(), 0);
        assert_eq!(seen.lock().unwrap().as_slice(), ["ledger.posted"]);
    }
}
