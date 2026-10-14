//! Shared per-store accounting for individual reads and writes.

use crate::access::{Error, Lease, Mode, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

// Zero means free, positive counts mean shared borrows, and the maximum value
// is reserved for an exclusive borrow. The ledger covers the whole store.
const WRITER: usize = usize::MAX;

enum EntryMode {
    Read,
    Write(AtomicUsize),
}

pub(in crate::access) struct Entry {
    // The broker's frozen entry has no backend lease and is always read-only.
    _lease: Option<Box<dyn Lease>>,
    mode: EntryMode,
}

// SAFETY: shared references expose only the immutable mode and atomic ledger,
// never the Send-only lease. The owning allocation drops any lease exactly once,
// on whichever thread owns it. This permits sharing the private ledger between
// Send sessions without making sessions or guards Sync.
unsafe impl Sync for Entry {}

impl Entry {
    pub(in crate::access) fn new(lease: Box<dyn Lease>, mode: Mode) -> Self {
        Self {
            _lease: Some(lease),
            mode: match mode {
                Mode::Read => EntryMode::Read,
                Mode::Write => EntryMode::Write(AtomicUsize::new(0)),
            },
        }
    }

    pub(in crate::access) fn frozen() -> Self {
        Self {
            _lease: None,
            mode: EntryMode::Read,
        }
    }

    #[inline(always)]
    pub(in crate::access) fn mode(&self) -> Mode {
        match self.mode {
            EntryMode::Read => Mode::Read,
            EntryMode::Write(_) => Mode::Write,
        }
    }

    pub(in crate::access) fn claim_read(&self) -> Result<()> {
        if let EntryMode::Write(state) = &self.mode {
            // Acquire pairs with the previous writer's release before any
            // reference is formed, even when both guards use the same lease.
            state
                .try_update(Ordering::Acquire, Ordering::Relaxed, |count| {
                    (count < WRITER - 1).then(|| count + 1)
                })
                .map_err(|count| {
                    if count == WRITER {
                        Error::BorrowConflict
                    } else {
                        Error::CounterOverflow
                    }
                })?;
        }
        Ok(())
    }

    pub(in crate::access) fn claim_write(&self) -> Result<()> {
        match &self.mode {
            EntryMode::Read => Err(Error::UpgradeDenied),
            EntryMode::Write(state) => state
                .compare_exchange(0, WRITER, Ordering::Acquire, Ordering::Relaxed)
                .map(|_| ())
                .map_err(|_| Error::BorrowConflict),
        }
    }

    /// Releases one shared borrow claim.
    ///
    /// # Safety
    /// For a write-mode entry, the caller must own a claim granted by a
    /// successful [`Self::claim_read`] call and release it exactly once. No reference
    /// protected by that claim may be used after it is released. Read-mode
    /// entries track no claims; releasing a read on them is a no-op.
    pub(in crate::access) unsafe fn release_read(&self) {
        if let EntryMode::Write(state) = &self.mode {
            state
                .try_update(Ordering::Release, Ordering::Relaxed, |count| {
                    (count > 0 && count < WRITER).then(|| count - 1)
                })
                .unwrap_or_else(|count| {
                    panic!(
                        "Cannot release a shared access borrow: expected an active reader, \
                         found {}. This indicates an internal borrow-accounting bug; Please report it.",
                        if count == WRITER {
                            "an active writer"
                        } else {
                            "no active borrow"
                        }
                    )
                });
        }
    }

    /// Releases an exclusive borrow claim.
    ///
    /// # Safety
    /// The caller must own the claim granted by a successful [`Self::claim_write`]
    /// call and release it exactly once. No reference protected by that claim
    /// may be used after it is released.
    pub(in crate::access) unsafe fn release_write(&self) {
        if let EntryMode::Write(state) = &self.mode {
            state
                .compare_exchange(WRITER, 0, Ordering::Release, Ordering::Relaxed)
                .unwrap_or_else(|count| {
                    panic!(
                        "Cannot release an exclusive access borrow: expected an active writer, \
                         found {count} readers. This indicates an internal borrow-accounting bug; Please report it."
                    )
                });
        } else {
            panic!(
                "Cannot release an exclusive access borrow from a read lease. \
                 This indicates an internal borrow-accounting bug; Please report it."
            )
        }
    }
}
