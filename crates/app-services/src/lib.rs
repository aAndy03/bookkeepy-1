// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-app-services` — the composition root (doc 03: `… ← ledger ← app-services ← ui
//! ← app`). This crate is the backend half of M3: it opens the profile session with the
//! higher-layer boot checks injected (ledger hash chains, schedules), runs the boot
//! `jobs_resume` driver (`schedules::materialize_due` — never backfills), and runs the
//! durable job worker in a background thread (claim → dispatch by kind → complete/fail
//! with persisted backoff). The UI (`ui` crate + `app` binary) consumes only this crate's
//! `api`. `api/` is the only public surface.

pub mod api;

pub use api::app::{App, AppError};
