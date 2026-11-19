//! The main-thread flag behind `GlobalCell`'s thread check, and the edit
//! loans behind the value handles' debug assertions.
//!
//! This is a separate crate solely so the root `Cargo.toml` can compile it
//! with optimizations in every profile (`[profile.dev.package.*]`): the
//! thread check sits on every global access, and unoptimized builds hit it
//! constantly. Optimized, `is_main_thread()` collapses to one TLS load
//! behind a direct call, whereas the same `thread_local!` compiled at
//! opt-level 0 costs ~3× per check through `LocalKey` and roughly doubled
//! search time in the timing-sensitive oldtests.
//!
//! Nothing here is `#[inline]` but [`is_main_thread_inline`]: cross-crate
//! inlining would paste the `LocalKey` machinery back into the caller at the
//! caller's opt-level, defeating the point. That one is for an optimised
//! caller, where the same machinery folds to one thread-local load.
//!
//! The cells' borrows are not here: each `GlobalCell` records its own
//! beside its value.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::cell::Cell;

thread_local! {
    /// True only on the thread that called [`mark_main_thread`].
    static IS_MAIN_THREAD: Cell<bool> = const { Cell::new(false) };
}

/// Record the calling thread as the main thread.
pub fn mark_main_thread() {
    IS_MAIN_THREAD.set(true);
}

/// Whether the calling thread is the one that called [`mark_main_thread`].
/// Out of line, for an unoptimised caller: one call into this optimised
/// crate, where the flag is one thread-local load.
pub fn is_main_thread() -> bool {
    IS_MAIN_THREAD.get()
}

/// [`is_main_thread`], inlined into the caller: for an optimised build,
/// where it is one thread-local byte read in place of a call.
#[inline]
pub fn is_main_thread_inline() -> bool {
    IS_MAIN_THREAD.get()
}

/// The value objects (lists, dictionaries, blobs, partials) lent out
/// writable through a shared handle on this thread, by address.
///
/// An edit can be held at every level of a walk over a nested value, so
/// this set has no fixed capacity. It is thread-local, and empty whenever
/// no edit is live.
#[derive(Debug)]
pub struct EditTable;

thread_local! {
    static EDITS: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl EditTable {
    /// Record that the object at `addr` is lent out writable. Answers
    /// `false`, recording nothing, when it already is.
    pub fn lend(addr: usize) -> bool {
        EDITS.with_borrow_mut(|lent| {
            if lent.contains(&addr) {
                return false;
            }
            lent.push(addr);
            true
        })
    }

    /// Give back the loan [`EditTable::lend`] recorded for `addr`.
    pub fn give_back(addr: usize) {
        EDITS.with_borrow_mut(|lent| {
            let at = lent
                .iter()
                .rposition(|&a| a == addr)
                .expect("an edit loan given back twice");
            lent.swap_remove(at);
        });
    }

    /// Whether the object at `addr` is lent out writable.
    pub fn is_lent(addr: usize) -> bool {
        EDITS.with_borrow(|lent| lent.contains(&addr))
    }
}
