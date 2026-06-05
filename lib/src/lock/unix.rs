// Copyright 2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![expect(missing_docs)]

use std::fs::File;
use std::path::PathBuf;

use rustix::fs::FlockOperation;
use tracing::instrument;

use super::FileLockError;

pub struct FileLock {
    path: PathBuf,
    file: File,
}

impl FileLock {
    pub fn lock(path: PathBuf) -> Result<Self, FileLockError> {
        tracing::info!("Attempting to lock {path:?}");
        loop {
            match Self::lock_attempt(&path, FlockOperation::LockExclusive)? {
                LockAttempt::Acquired(file) => {
                    tracing::info!("Locked {path:?}");
                    return Ok(Self { path, file });
                }
                LockAttempt::StaleRetry => continue,
                // `LockExclusive` blocks until acquired, so this is impossible.
                LockAttempt::WouldBlock => unreachable!(),
            }
        }
    }

    /// Try to acquire the lock without blocking.
    ///
    /// Returns `Ok(Some(_))` if the lock was acquired, `Ok(None)` if another
    /// holder currently owns it, and `Err(_)` for IO errors that aren't
    /// "would block".
    pub fn try_lock(path: PathBuf) -> Result<Option<Self>, FileLockError> {
        tracing::info!("Attempting to try-lock {path:?}");
        loop {
            match Self::lock_attempt(&path, FlockOperation::NonBlockingLockExclusive)? {
                LockAttempt::Acquired(file) => {
                    tracing::info!("Try-locked {path:?}");
                    return Ok(Some(Self { path, file }));
                }
                LockAttempt::WouldBlock => return Ok(None),
                // The stale/unlinked case is a race with another holder's
                // `Drop`; retry once to either acquire cleanly or observe a
                // `WouldBlock` from a fresh contender.
                LockAttempt::StaleRetry => continue,
            }
        }
    }

    fn lock_attempt(path: &PathBuf, op: FlockOperation) -> Result<LockAttempt, FileLockError> {
        // Create lockfile, or open pre-existing one
        let file = File::create(path).map_err(|err| FileLockError {
            message: "Failed to open lock file",
            path: path.clone(),
            err,
        })?;
        match rustix::fs::flock(&file, op) {
            Ok(()) => {}
            Err(rustix::io::Errno::WOULDBLOCK) => return Ok(LockAttempt::WouldBlock),
            Err(errno) => {
                return Err(FileLockError {
                    message: "Failed to lock lock file",
                    path: path.clone(),
                    err: errno.into(),
                });
            }
        }

        match rustix::fs::fstat(&file) {
            Ok(stat) => {
                if stat.st_nlink == 0 {
                    // Lockfile was deleted, probably by the previous holder's `Drop` impl;
                    // create a new one so our ownership is visible,
                    // rather than hidden in an unlinked file. Not
                    // always necessary, since the previous holder might
                    // have exited abruptly.
                    return Ok(LockAttempt::StaleRetry);
                }
            }
            Err(rustix::io::Errno::STALE) => {
                // The file handle is stale.
                // This can happen when using NFS,
                // likely caused by a remote deletion of the lockfile.
                // Treat this like a normal lockfile deletion and retry.
                return Ok(LockAttempt::StaleRetry);
            }
            Err(errno) => {
                return Err(FileLockError {
                    message: "failed to stat lock file",
                    path: path.clone(),
                    err: errno.into(),
                });
            }
        }

        Ok(LockAttempt::Acquired(file))
    }
}

enum LockAttempt {
    Acquired(File),
    WouldBlock,
    StaleRetry,
}

impl Drop for FileLock {
    #[instrument(skip_all)]
    fn drop(&mut self) {
        // Removing the file isn't strictly necessary, but reduces confusion.
        std::fs::remove_file(&self.path).ok();
        // Unblock any processes that tried to acquire the lock while we held it.
        // They're responsible for creating and locking a new lockfile, since we
        // just deleted this one.
        rustix::fs::flock(&self.file, FlockOperation::Unlock).ok();
    }
}
