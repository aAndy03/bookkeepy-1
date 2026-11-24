// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-storage` — the storage engine (M1, ADR-0005): rusqlite connection (WAL),
//! forward-only migrations with `schema_version` + downgrade refusal (RULE-DATA-004),
//! durable outbox → bus, hash-chained audit log, local live backup/verify/restore/prune,
//! single-instance lock, crash-recovery self-check, `ui_state`/`settings` KV tables,
//! durable jobs state machine, atomic `tx::post`, and the `session` composition root.

pub mod api;

pub use api::audit;
pub use api::backup;
pub use api::conn::Db;
pub use api::error::StorageError;
pub use api::jobs;
pub use api::kv;
pub use api::lock::{InstanceLock, acquire as acquire_instance_lock};
pub use api::outbox;
pub use api::schema::{MAX_KNOWN, migrate};
pub use api::selfcheck::{SelfCheckReport, run as self_check};
pub use api::session::{Session, open_session};
pub use api::tx;
