//! The main-thread flag behind `GlobalCell`'s debug assertion.
//!
//! This is a separate crate solely so the root `Cargo.toml` can compile it
//! with optimizations in every profile (`[profile.dev.package.*]`): the
//! assertion sits on every global access, and unoptimized builds hit it
//! constantly. Optimized, `is_main_thread()` collapses to one TLS load
//! behind a direct call — measured indistinguishable from the unstable bare
//! `#[thread_local]` attribute it replaced, whereas the same `thread_local!`
//! compiled at opt-level 0 costs ~3× per check through `LocalKey` and
//! roughly doubled search time in the timing-sensitive oldtests.
//!
//! Nothing here is `#[inline]`: cross-crate inlining would paste the
//! `LocalKey` machinery back into the caller at the caller's opt-level,
//! defeating the point.

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
pub fn is_main_thread() -> bool {
    IS_MAIN_THREAD.get()
}
