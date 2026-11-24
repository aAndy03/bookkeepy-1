// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-bus` — in-process event bus over the contracts envelope (doc 03 L2, ADR-0009).
//! The durable outbox lands with storage in M1. `api/` is the only public surface.

pub mod api;

pub use api::Bus;
