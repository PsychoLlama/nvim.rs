//! The main-thread flag behind `GlobalCell`'s thread check.
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
//! beside its value, as each value object records its edit loan.

#![forbid(unsafe_code)]

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
