// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Compensating ("revert") migration proof (doc 05 §5.2, INV-DATA-001): migrations are
//! forward-only; a revert is a NEW migration that undoes the previous one while preserving
//! data. This test applies a test-only v6/v7 pair on top of the current MAX_KNOWN, proves
//! user data survives, and proves the downgrade refusal (BK-DATA-0002) still guards older
//! builds.

use bookkeepy_storage::api::conn::Db;
use bookkeepy_storage::api::kv::ui_set;
use bookkeepy_storage::api::schema::{self, Migration};

#[test]
fn compensating_migration_preserves_data_and_downgrade_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("app.db");

    // current schema via the normal path, then user data
    {
        let db = Db::open(&path).unwrap();
        assert_eq!(db.schema_version().unwrap(), schema::MAX_KNOWN);
        ui_set(&db, "window", "0,0,1440,900").unwrap();
    }

    // dev experiment beyond MAX_KNOWN: v6 adds a column, v7 (the "revert") drops it again
    let experiment = [
        Migration {
            version: 6,
            name: "test_add_ui_note",
            sql: "ALTER TABLE ui_state ADD COLUMN note TEXT",
        },
        Migration {
            version: 7,
            name: "test_revert_ui_note",
            sql: "ALTER TABLE ui_state DROP COLUMN note",
        },
    ];
    {
        // a newer dev build applies the experiment through the public machinery
        let mut conn = rusqlite::Connection::open(&path).unwrap();
        schema::migrate_with(&mut conn, &experiment, 7).unwrap();
        // re-running is idempotent
        schema::migrate_with(&mut conn, &experiment, 7).unwrap();
        assert_eq!(schema::current_version(&conn).unwrap(), 7);
        // user data survived the add+drop round trip
        let v: String = conn
            .query_row("SELECT value FROM ui_state WHERE key='window'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(v, "0,0,1440,900");
    }

    // this build (MAX_KNOWN=5) refuses the v7 profile read-only
    let err = Db::open(&path).unwrap_err();
    assert_eq!(err.code(), "BK-DATA-0002");
}
