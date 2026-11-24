// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! End-to-end session flow (M1 addendum): registry → open_session → post → drop
//! (crash simulation) → reopen → audit chain verifies, outbox empty, ui_state persisted,
//! auto backups pruned to KEEP_BACKUPS.

use bookkeepy_contracts::{Actor, EventEnvelope};
use bookkeepy_profile::Registry;
use bookkeepy_storage::api::backup::KEEP_BACKUPS;
use bookkeepy_storage::api::{audit, kv, outbox, session};

#[test]
fn session_open_post_reopen_persists() {
    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path();
    let mut reg = Registry::load(data_dir).unwrap();
    let meta = reg.create_profile(data_dir, "Personal", "#3366ff").unwrap();

    {
        let s = session::open_session(data_dir, meta.id).unwrap();
        kv::ui_set(&s.db, "window", "0,0,1440,900").unwrap();
        let ev = EventEnvelope::new(
            "ledger.posted",
            Actor::User,
            "quick add",
            serde_json::json!({"amount": 100}),
        );
        let seq = s.post(&ev).unwrap();
        assert_eq!(seq, 1);
        assert_eq!(outbox::pending_count(&s.db).unwrap(), 0);
        assert!(s.report.healthy());
    } // drop session (crash-sim: lock released)

    {
        let s2 = session::open_session(data_dir, meta.id).unwrap();
        audit::verify(&s2.db).unwrap();
        assert_eq!(
            kv::ui_get(&s2.db, "window").unwrap().as_deref(),
            Some("0,0,1440,900")
        );
        assert_eq!(outbox::pending_count(&s2.db).unwrap(), 0);
        let backups = std::fs::read_dir(s2.roots.root.join("backups"))
            .unwrap()
            .count();
        assert!((1..=KEEP_BACKUPS).contains(&backups));
        // settings override validated against the SSOT keys
        kv::settings_set(&s2.db, "theme", "dark").unwrap();
        assert_eq!(
            kv::settings_get(&s2.db, "theme").unwrap().as_deref(),
            Some("dark")
        );
        assert_eq!(
            kv::settings_set(&s2.db, "bogus", "x").unwrap_err().code(),
            "BK-DATA-0003"
        );
    }
}
