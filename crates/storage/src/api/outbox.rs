// SPDX-License-Identifier: GPL-3.0-or-later
//! The DURABLE outbox (ADR-0009): envelopes are persisted BEFORE side effects run, so a
//! crash never loses an audited action; after commit they are published to the in-process
//! `Bus` and marked published. The startup driver (`selfcheck`) republishes any leftover
//! pending rows — exactly-once at the app level.

use bookkeepy_bus::Bus;
use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_core::time::Timestamp;
use rusqlite::Connection;

use crate::api::conn::Db;
use crate::api::error::StorageError;

/// Persist an envelope in the outbox. Takes the connection directly so it can run inside
/// an explicit transaction together with the audited side effect (`tx::post`, ADR-0009);
/// `&Transaction` derefs to `&Connection`.
pub fn enqueue(conn: &Connection, ev: &EventEnvelope) -> Result<(), StorageError> {
    conn.execute(
        "INSERT OR IGNORE INTO outbox (id, kind, occurred_at, actor, reason, payload, published)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
        rusqlite::params![
            ev.id.to_string(),
            ev.kind,
            ev.occurred_at.to_string(),
            serde_json::to_string(&ev.actor).unwrap_or_default(),
            ev.reason,
            ev.payload.to_string(),
        ],
    )?;
    Ok(())
}

fn row_to_envelope(row: &rusqlite::Row) -> rusqlite::Result<EventEnvelope> {
    let id: String = row.get(0)?;
    let kind: String = row.get(1)?;
    let occurred_at: String = row.get(2)?;
    let actor_json: String = row.get(3)?;
    let reason: String = row.get(4)?;
    let payload: String = row.get(5)?;
    let conv = |m: &str| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(m.to_string())),
        )
    };
    Ok(EventEnvelope {
        id: id.parse().map_err(|_| conv("bad outbox id"))?,
        kind,
        occurred_at: occurred_at
            .parse::<Timestamp>()
            .map_err(|_| conv("bad timestamp"))?,
        actor: serde_json::from_str(&actor_json).unwrap_or(Actor::System),
        reason,
        payload: serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null),
    })
}

/// Publish all unpublished outbox rows to the bus and mark them published.
/// Returns how many were published.
pub fn publish_pending(db: &Db, bus: &Bus) -> Result<usize, StorageError> {
    db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, kind, occurred_at, actor, reason, payload FROM outbox WHERE published=0 ORDER BY rowid",
        )?;
        let rows = stmt.query_map([], row_to_envelope)?.collect::<Result<Vec<_>, _>>()?;
        let n = rows.len();
        for ev in &rows {
            bus.publish(ev);
            c.execute("UPDATE outbox SET published=1 WHERE id=?1", [&ev.id.to_string()])?;
        }
        Ok(n)
    })
}

/// Count unpublished outbox rows (used by self-check).
pub fn pending_count(db: &Db) -> Result<i64, StorageError> {
    db.with_conn(|c| {
        Ok(
            c.query_row("SELECT COUNT(*) FROM outbox WHERE published=0", [], |r| {
                r.get(0)
            })?,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_contracts::Actor;
    use std::sync::{Arc, Mutex};

    #[test]
    fn enqueue_then_publish_once() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::open(&tmp.path().join("app.db")).unwrap();
        let bus = Bus::new();
        let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let s2 = seen.clone();
        bus.subscribe(move |ev| s2.lock().unwrap().push(ev.kind.clone()));

        let ev = EventEnvelope::new(
            "ledger.posted",
            Actor::User,
            "test",
            serde_json::json!({"x":1}),
        );
        db.with_conn(|c| enqueue(c, &ev)).unwrap();
        assert_eq!(pending_count(&db).unwrap(), 1);

        assert_eq!(publish_pending(&db, &bus).unwrap(), 1);
        assert_eq!(pending_count(&db).unwrap(), 0);
        assert_eq!(seen.lock().unwrap().as_slice(), ["ledger.posted"]);
        // second run publishes nothing (exactly-once)
        assert_eq!(publish_pending(&db, &bus).unwrap(), 0);
    }
}
