// SPDX-License-Identifier: GPL-3.0-or-later
//! Single-instance lock (ADR-0016): an exclusive lockfile `<root>/app.lock` held for the
//! process lifetime. A second launch fails with `LockHeld` (BK-STORE-0004) and the caller
//! (app shell) focuses the existing window instead.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use fs2::FileExt;

use crate::api::error::StorageError;

/// Name of the lockfile inside the profile root.
pub const LOCK_FILE: &str = "app.lock";

/// Guard holding the exclusive lock; released (and file removed) on drop.
pub struct InstanceLock {
    file: File,
    path: PathBuf,
}

/// Try to acquire the single-instance lock at `root/app.lock`.
pub fn acquire(root: &Path) -> Result<InstanceLock, StorageError> {
    let path = root.join(LOCK_FILE);
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| StorageError::Backup {
            path: path.clone(),
            message: e.to_string(),
        })?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(InstanceLock { file, path }),
        Err(_) => Err(StorageError::LockHeld),
    }
}

impl std::fmt::Debug for InstanceLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceLock")
            .field("path", &self.path)
            .finish()
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_fails_then_succeeds_after_drop() {
        let tmp = tempfile::tempdir().unwrap();
        let lock = acquire(tmp.path()).unwrap();
        let err = acquire(tmp.path()).unwrap_err();
        assert_eq!(err.code(), "BK-STORE-0004");
        drop(lock);
        assert!(acquire(tmp.path()).is_ok());
    }
}
