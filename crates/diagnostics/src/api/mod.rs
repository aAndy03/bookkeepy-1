//! Public API of `bookkeepy-diagnostics` (RULE-SSOT-001).

mod bundle;
mod panic;

pub use bundle::{DiagnosticBundle, DiagnosticEntry, Severity};
pub use panic::install_panic_hook;
