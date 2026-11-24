// SPDX-License-Identifier: GPL-3.0-or-later
//! Panic hook (item 2, M0.5; INV-ERR-001): a panic becomes a fatal diagnostic bundle
//! printed to stderr in copyable form, so a crash is always reportable even before
//! logging is initialized. Once `bookkeepy-logging` is up, the app can additionally
//! route the bundle through the log (kept decoupled to avoid a dependency cycle).

use crate::api::bundle::{DiagnosticBundle, DiagnosticEntry, Severity};

/// Install a panic hook that renders panics as a `bookkeepy-diagnostic@1` bundle.
/// Idempotent-ish: call once at startup. The hook must never panic itself.
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "non-string panic payload".to_string()
        };
        let bundle = DiagnosticBundle::new().push(
            DiagnosticEntry::new("BK-APP-0001", Severity::Fatal, payload)
                .with("location", location)
                .with("note", "panic — please report this bundle"),
        );
        // Copyable text to stderr; JSON would drown in noise, text is paste-ready.
        eprintln!("{}", bundle.to_copyable_text());
    }));
}

#[cfg(test)]
mod tests {
    #[test]
    fn panic_hook_does_not_break_normal_flow() {
        // Installing twice is fine (last hook wins); we only assert it doesn't panic here.
        super::install_panic_hook();
    }
}
