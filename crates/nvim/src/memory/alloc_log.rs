//! The allocation recorder: an exact log of what the `xmalloc` family asked
//! the platform allocator for, in order.
//!
//! `test/unit`'s LuaJIT harness rebinds the four `mem_*` function pointers to
//! Lua callbacks and asserts, case by case, the exact sequence of
//! `malloc`/`calloc`/`realloc`/`free` calls a typval operation makes — the
//! sizes as much as the order, because a size derived from
//! `offsetof(dictitem_T, di_key) + len + 1` is the only evidence that the
//! over-allocation happened at all. That assertion is the sole remaining
//! reason the `mem_*` seam exists. This is its Rust twin, so the cases can
//! move into the crate and the seam can go.
//!
//! # Why not a `#[global_allocator]`
//!
//! A `GlobalAlloc` hook and this are not the same instrument, and only this
//! one can express what the cases assert:
//!
//! - A `GlobalAlloc` sees every Rust allocation in the process — the test
//!   harness's own `Vec`s, the panic machinery, `std`'s buffers. The cases
//!   are written about editor allocations only, so every unrelated event
//!   would have to be filtered out by guessing, and a case that asserts
//!   "this allocated *nothing*" could not be written at all.
//! - `GlobalAlloc` is handed a `Layout` (size and alignment). The
//!   `calloc(count, size)` pair and the `malloc(size)` singleton collapse to
//!   the same `Layout`, and the cases distinguish them:
//!   `calloc{1, size_of::<list_T>()}` is a different expectation from
//!   `malloc{size_of::<list_T>()}`. That distinction is unrecoverable below
//!   the seam.
//! - `arena_alloc(NULL, ..)` falls through to `xmalloc`, which is exactly
//!   the traffic an arena rewrite has to keep accounting for.
//!
//! So the recorder sits where the seam sat: at the four calls in
//! [`super`] that reach the platform allocator, *after* the size adjustments
//! (`size.max(1)`, `calloc`'s `(1, 1)` floor) and once per attempt, so a
//! retry after `try_to_free_memory` logs twice. That is what the Lua
//! callbacks saw, and expectations ported from them stay literally true.
//!
//! # Cost when nothing is recording
//!
//! One relaxed load of [`ARMED`], which is zero in every build that is not
//! running a recording test. Nothing else — no thread-local access, no
//! branchy bookkeeping — so the editor pays strictly less than it already
//! paid for the indirect call through `mem_malloc`.
//!
//! # Thread locality
//!
//! The log is thread-local and only the thread that started a [`Recorder`]
//! has one, so an allocation on a libuv thread cannot land in a test's
//! expectations, and `cargo test`'s parallel threads cannot see each other's
//! events. [`ARMED`] is a global count of live recorders purely so the
//! disarmed path can skip the thread-local entirely.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::RefCell;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

/// One call the `xmalloc` family made to the platform allocator.
///
/// The variants mirror `test/unit/testutil.lua`'s log entries one for one:
/// its `{func, args, ret}` table with the arguments named.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AllocEvent {
    /// `malloc(size)`. Note `size` is what was actually asked for, which for
    /// a zero-byte request is 1.
    Malloc { size: usize, ret: *mut c_void },
    /// `calloc(count, size)`, likewise after the `(1, 1)` floor.
    Calloc {
        count: usize,
        size: usize,
        ret: *mut c_void,
    },
    /// `realloc(ptr, size)`.
    Realloc {
        ptr: *mut c_void,
        size: usize,
        ret: *mut c_void,
    },
    /// `free(ptr)`. A null `ptr` is logged like any other, as the Lua
    /// harness logged it.
    Free { ptr: *mut c_void },
}

impl AllocEvent {
    /// The pointer this event handed out, for the allocating variants.
    pub fn allocated(&self) -> Option<*mut c_void> {
        match *self {
            AllocEvent::Malloc { ret, .. } | AllocEvent::Calloc { ret, .. } => Some(ret),
            AllocEvent::Realloc { .. } | AllocEvent::Free { .. } => None,
        }
    }

    /// The pointer this event consumed, for the releasing variants.
    pub fn released(&self) -> Option<*mut c_void> {
        match *self {
            AllocEvent::Realloc { ptr, .. } | AllocEvent::Free { ptr } => Some(ptr),
            AllocEvent::Malloc { .. } | AllocEvent::Calloc { .. } => None,
        }
    }
}

/// How many [`Recorder`]s are alive, anywhere. The disarmed fast path reads
/// only this.
static ARMED: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// This thread's log, or `None` when it has no live [`Recorder`].
    static LOG: RefCell<Option<Vec<AllocEvent>>> = const { RefCell::new(None) };
}

/// Log `event` if this thread is recording.
///
/// Called from [`super`]'s four seam wrappers and nowhere else.
#[inline]
pub(super) fn record(event: AllocEvent) {
    if ARMED.load(Ordering::Relaxed) != 0 {
        record_slow(event);
    }
}

/// The armed path, kept out of line so the disarmed one is a load and a
/// branch.
#[cold]
fn record_slow(event: AllocEvent) {
    // `try_with` because a thread tearing down its thread-locals may still
    // allocate, and `try_borrow_mut` because the push below allocates through
    // Rust's global allocator, which bottoms out in libc `malloc` rather than
    // in `xmalloc` — so it cannot re-enter, but a future one that did would
    // be dropped rather than deadlocking.
    let _ = LOG.try_with(|log| {
        if let Ok(mut slot) = log.try_borrow_mut()
            && let Some(events) = slot.as_mut()
        {
            events.push(event);
        }
    });
}

/// A live recording on the calling thread. Allocations made by this thread
/// while it exists are logged; the log is dropped with it.
///
/// This is the twin of the Lua harness's `alloc_log`, minus the mocking: the
/// real allocator still serves every request, exactly as the Lua callbacks
/// forwarded to the saved originals.
pub struct Recorder {
    /// A recorder belongs to the thread that started it.
    _not_send: PhantomData<*const ()>,
}

impl Recorder {
    /// Start recording on this thread.
    ///
    /// # Panics
    ///
    /// If this thread is already recording. Two overlapping recorders would
    /// share one log, which no caller could make sense of.
    pub fn start() -> Recorder {
        LOG.with(|log| {
            let mut slot = log.borrow_mut();
            assert!(slot.is_none(), "this thread is already recording");
            *slot = Some(Vec::new());
        });
        ARMED.fetch_add(1, Ordering::Relaxed);
        Recorder {
            _not_send: PhantomData,
        }
    }

    /// Everything recorded since the last [`take`](Self::take), oldest
    /// first, leaving the log empty.
    pub fn take(&self) -> Vec<AllocEvent> {
        LOG.with(|log| {
            core::mem::take(
                log.borrow_mut()
                    .as_mut()
                    .expect("the recorder is alive, so this thread has a log"),
            )
        })
    }

    /// Drop everything recorded so far.
    pub fn clear(&self) {
        self.take();
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        ARMED.fetch_sub(1, Ordering::Relaxed);
        LOG.with(|log| *log.borrow_mut() = None);
    }
}

/// Drop the events for allocations that were made and released again within
/// the log — the twin of the Lua harness's `clear_tmp_allocs`.
///
/// An allocation is temporary when a later `free`/`realloc` names the pointer
/// it returned: both events go, so what remains is the net effect of the
/// operation under test. `realloc` keeps its own result live under the new
/// pointer. With `clear_null_frees`, `free(NULL)` is dropped too.
///
/// This is a pure function of the log, which is what makes it testable
/// without an allocator at all.
pub fn clear_tmp_allocs(events: &mut Vec<AllocEvent>, clear_null_frees: bool) {
    // Index of the surviving allocation that handed out each live pointer.
    // A pointer may be handed out again after a free, so the *most recent*
    // allocation of an address is the one a release matches.
    let mut live: Vec<(*mut c_void, usize)> = Vec::new();
    let mut dropped = vec![false; events.len()];
    for (i, event) in events.iter().copied().enumerate() {
        if let Some(ret) = event.allocated() {
            live.push((ret, i));
            continue;
        }
        let Some(ptr) = event.released() else {
            continue;
        };
        let is_free = matches!(event, AllocEvent::Free { .. });
        if let Some(slot) = live.iter().rposition(|&(q, _)| q == ptr) {
            dropped[live[slot].1] = true;
            live.remove(slot);
            if is_free {
                dropped[i] = true;
            }
        } else if clear_null_frees && is_free && ptr.is_null() {
            dropped[i] = true;
        }
        if let AllocEvent::Realloc { ret, .. } = event {
            live.push((ret, i));
        }
    }
    let mut kept = dropped.iter().map(|&gone| !gone);
    events.retain(|_| kept.next().unwrap_or(true));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: usize) -> *mut c_void {
        core::ptr::without_provenance_mut(n)
    }

    fn malloc(size: usize, ret: usize) -> AllocEvent {
        AllocEvent::Malloc { size, ret: p(ret) }
    }

    fn free(ptr: usize) -> AllocEvent {
        AllocEvent::Free { ptr: p(ptr) }
    }

    #[test]
    fn a_matched_alloc_and_free_both_disappear() {
        let mut log = vec![malloc(8, 1), malloc(16, 2), free(1)];
        clear_tmp_allocs(&mut log, false);
        assert_eq!(log, vec![malloc(16, 2)]);
    }

    #[test]
    fn a_free_of_something_allocated_earlier_than_the_log_survives() {
        // The `free` names a pointer no logged allocation handed out, so it
        // is part of the operation's visible effect and stays.
        let mut log = vec![free(99), malloc(8, 1), free(1)];
        clear_tmp_allocs(&mut log, false);
        assert_eq!(log, vec![free(99)]);
    }

    #[test]
    fn a_realloc_retires_its_input_and_stays_live_under_its_result() {
        let realloc = AllocEvent::Realloc {
            ptr: p(1),
            size: 32,
            ret: p(2),
        };
        // malloc(1) is consumed by the realloc, so the malloc goes but the
        // realloc — which is the surviving allocation — stays.
        let mut log = vec![malloc(8, 1), realloc];
        clear_tmp_allocs(&mut log, false);
        assert_eq!(log, vec![realloc]);
        // ... and freeing the realloc's result retires that too.
        let mut log = vec![malloc(8, 1), realloc, free(2)];
        clear_tmp_allocs(&mut log, false);
        assert_eq!(log, vec![]);
    }

    #[test]
    fn null_frees_go_only_when_asked() {
        let mut log = vec![free(0), malloc(8, 1)];
        clear_tmp_allocs(&mut log, false);
        assert_eq!(log, vec![free(0), malloc(8, 1)]);
        let mut log = vec![free(0), malloc(8, 1)];
        clear_tmp_allocs(&mut log, true);
        assert_eq!(log, vec![malloc(8, 1)]);
    }

    #[test]
    fn a_reused_address_matches_the_most_recent_allocation_of_it() {
        // The allocator may hand the same address back after a free. The
        // second free must retire the second malloc, not the first, or a
        // pair would survive that never existed.
        let mut log = vec![malloc(8, 1), free(1), malloc(8, 1), free(1)];
        clear_tmp_allocs(&mut log, false);
        assert_eq!(log, vec![]);
    }

    #[test]
    fn recording_is_off_until_a_recorder_starts() {
        // No recorder here: `record` must be a no-op, and must not create a
        // log this thread would then keep.
        record(malloc(8, 1));
        let recorder = Recorder::start();
        assert_eq!(recorder.take(), vec![]);
        record(malloc(8, 1));
        record(free(1));
        assert_eq!(recorder.take(), vec![malloc(8, 1), free(1)]);
        // `take` empties.
        assert_eq!(recorder.take(), vec![]);
        drop(recorder);
        record(malloc(8, 1));
        let recorder = Recorder::start();
        assert_eq!(recorder.take(), vec![]);
    }

    #[test]
    fn another_thread_records_into_its_own_log() {
        let recorder = Recorder::start();
        record(malloc(8, 1));
        std::thread::scope(|scope| {
            scope.spawn(|| {
                // Armed globally, but this thread has no log of its own, so
                // the event is dropped rather than landing in the parent's.
                record(malloc(16, 2));
            });
        });
        assert_eq!(recorder.take(), vec![malloc(8, 1)]);
    }
}
