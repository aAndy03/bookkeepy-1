// SPDX-License-Identifier: GPL-3.0-or-later
//! In-process publish/subscribe over `EventEnvelope` (item 1, M0.5; ADR-0009).
//!
//! Layer note (doc 03 L2): this is the *transient* bus. The *durable* outbox —
//! envelopes persisted before side effects run, so a crash never loses an audited
//! action — lands with the storage engine in M1 (`bookkeepy-storage::outbox`), which
//! will publish through this bus after commit. Until then, in-memory only.

use bookkeepy_contracts::EventEnvelope;
use std::sync::{Arc, Mutex};

type Subscriber = Arc<dyn Fn(&EventEnvelope) + Send + Sync>;

/// The app-wide bus. Cloneable handle; all clones share one subscriber list.
#[derive(Clone, Default)]
pub struct Bus {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

impl Bus {
    /// Create an empty bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a subscriber; it receives every envelope published after registration.
    pub fn subscribe(&self, f: impl Fn(&EventEnvelope) + Send + Sync + 'static) {
        self.subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(Arc::new(f));
    }

    /// Publish an envelope to all current subscribers, in subscription order.
    pub fn publish(&self, ev: &EventEnvelope) {
        let subs = self
            .subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        for sub in &subs {
            sub(ev);
        }
    }

    /// Number of subscribers (used by tests and self-check).
    pub fn subscriber_count(&self) -> usize {
        self.subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookkeepy_contracts::Actor;
    use std::sync::Mutex as StdMutex;

    #[test]
    fn subscribers_receive_in_order() {
        let bus = Bus::new();
        let seen: Arc<StdMutex<Vec<String>>> = Arc::new(StdMutex::new(Vec::new()));
        let seen2 = seen.clone();
        bus.subscribe(move |ev| seen2.lock().unwrap().push(format!("a:{}", ev.kind)));
        let seen3 = seen.clone();
        bus.subscribe(move |ev| seen3.lock().unwrap().push(format!("b:{}", ev.kind)));

        bus.publish(&EventEnvelope::new(
            "ledger.posted",
            Actor::User,
            "t",
            serde_json::json!({}),
        ));

        assert_eq!(
            seen.lock().unwrap().as_slice(),
            ["a:ledger.posted", "b:ledger.posted"]
        );
    }

    #[test]
    fn late_subscriber_misses_earlier_events() {
        let bus = Bus::new();
        bus.publish(&EventEnvelope::new(
            "early",
            Actor::System,
            "",
            serde_json::json!({}),
        ));
        assert_eq!(bus.subscriber_count(), 0);
        let count = Arc::new(StdMutex::new(0usize));
        let c2 = count.clone();
        bus.subscribe(move |_| *c2.lock().unwrap() += 1);
        bus.publish(&EventEnvelope::new(
            "late",
            Actor::System,
            "",
            serde_json::json!({}),
        ));
        assert_eq!(*count.lock().unwrap(), 1);
    }
}
