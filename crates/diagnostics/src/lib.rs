#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! `bookkeepy-diagnostics` — diagnostic bundles (`bookkeepy-diagnostic@1`) + panic hook, so
//! every fatal error is copyable and reportable (RULE-ERR-003, doc 03 L1). `api/` is the
//! only public surface; items are re-exported at root.

pub mod api;

pub use api::{DiagnosticBundle, DiagnosticEntry, Severity, install_panic_hook};
