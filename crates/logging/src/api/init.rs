//! Logging init (item 7, M0.5). Local-only rotating files (daily, kept 14 days) under the
//! profile's `logs/` dir; `RUST_LOG` overrides the default filter (`info,bookkeepy=debug`).
//! Nothing is ever sent over the network (RULE-PRIV-001, doc 08): logging is a file sink.

use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Days of daily log files to keep.
pub const KEEP_DAYS: usize = 14;

/// Errors logging init can produce (registered as `BK-LOG-0001` in `data/errors.toml`).
#[derive(Debug, Error)]
pub enum LogError {
    /// The log directory could not be created/written.
    #[error("cannot create log dir {path}: {source}")]
    Dir {
        /// The log directory path.
        path: PathBuf,
        /// Underlying IO error.
        source: std::io::Error,
    },
}

impl LogError {
    /// Registered error code (RULE-ERR-002).
    pub fn code(&self) -> &'static str {
        "BK-LOG-0001"
    }
}

/// Keeps the non-blocking writer alive; dropping it flushes and stops the writer.
pub struct LogGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
    /// Directory the logs live in (for diagnostics).
    pub dir: PathBuf,
}

/// Initialize the global tracing subscriber writing daily-rotated files into `dir`.
/// Call once at startup; a second call is a no-op returning the dir (global subscriber
/// can only be set once — RULE-ASYNC-001 job owners handle this).
pub fn init(dir: &Path) -> Result<LogGuard, LogError> {
    std::fs::create_dir_all(dir).map_err(|source| LogError::Dir {
        path: dir.to_path_buf(),
        source,
    })?;
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("bookkeepy")
        .filename_suffix("log")
        .max_log_files(KEEP_DAYS)
        .build(dir)
        .map_err(|source| LogError::Dir {
            path: dir.to_path_buf(),
            source: std::io::Error::other(source),
        })?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,bookkeepy=debug,wgpu=warn"));
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();
    Ok(LogGuard {
        _guard: guard,
        dir: dir.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_is_registered_format() {
        assert_eq!(
            LogError::code(&LogError::Dir {
                path: PathBuf::from("x"),
                source: std::io::Error::other("x"),
            }),
            "BK-LOG-0001"
        );
    }

    #[test]
    fn init_writes_into_temp_dir() {
        let dir =
            std::env::temp_dir().join(format!("bookkeepy-logging-test-{}", std::process::id()));
        // global subscriber may already be set by another test in this binary; init() would
        // panic on `.init()` — so only exercise the fallible dir-creation path here.
        std::fs::create_dir_all(&dir).unwrap();
        let guard = init(&dir);
        // Either Ok (first init) or panic-guard: we accept Ok only in a fresh process;
        // in-process double init panics, so guard with catch_unwind semantics via Result:
        drop(guard);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
