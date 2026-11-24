// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-contracts` — shared DTOs and the domain event envelope every engine talks
//! (doc 03 L1, ADR-0009). `api/` is the only public surface; items are re-exported at root.

pub mod api;

pub use api::{Actor, EventEnvelope, kinds};
