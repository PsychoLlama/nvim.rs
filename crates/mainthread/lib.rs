//! The main-thread flag and the borrow table behind `GlobalCell`'s debug
//! assertions.
//!
//! This is a separate crate solely so the root `Cargo.toml` can compile it
//! with optimizations in every profile (`[profile.dev.package.*]`): the
//! assertions sit on every global access, and unoptimized builds hit them
//! constantly. Optimized, `is_main_thread()` collapses to one TLS load
//! behind a direct call — measured indistinguishable from the unstable bare
//! `#[thread_local]` attribute it replaced, whereas the same `thread_local!`
//! compiled at opt-level 0 costs ~3× per check through `LocalKey` and
//! roughly doubled search time in the timing-sensitive oldtests.
//!
//! [`BorrowTable`] is here for the same reason and was measured the same
//! way. It used to be a `RefCell<HashMap<usize, isize>>` `thread_local!` in
//! the editor crate, where one `GlobalCell::with` cost **~930 ns** in a
//! debug build against ~15 ns for `get` — enough that the choice between
//! `with` and the raw escape hatch was being made by the borrow *tracker*
//! rather than by the design. A const-initialised fixed array, compiled
//! here, brings the same acquisition to **~48 ns**.
//!
//! Nothing here is `#[inline]`: cross-crate inlining would paste the
//! `LocalKey` machinery back into the caller at the caller's opt-level,
//! defeating the point.

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
pub fn is_main_thread() -> bool {
    IS_MAIN_THREAD.get()
}

/// The cells borrowed by `GlobalCell::with`/`with_mut` on this thread.
///
/// Thread-local, not shared: the main-thread assertion is inert in a process
/// that never calls [`mark_main_thread`] (`cargo test`, the FFI unit suite),
/// so two threads really can be inside a `with` at once there, and each must
/// see only its own borrows.
///
/// A namespace rather than free functions because the editor crate's ratchet
/// counts items at a crate's boundary, and this is one boundary item however
/// many operations it carries.
pub struct BorrowTable;

/// How many cells may be borrowed at once, per thread. The editor nests a
/// handful; a table that fills up is a finding, not a limit to raise
/// quietly, so [`BorrowTable::acquire`] panics rather than stop tracking.
const SLOTS: usize = 64;

/// The borrows held, as a contiguous prefix of `slots`: each entry is a
/// cell's address and its state — a positive shared count, or -1 for an
/// exclusive borrow. The prefix is almost always empty or one entry long,
/// so the scan is shorter than one hash of the address would be.
struct Table {
    len: Cell<usize>,
    slots: [Cell<(usize, isize)>; SLOTS],
}

impl Table {
    const fn new() -> Self {
        Table {
            len: Cell::new(0),
            slots: [const { Cell::new((0, 0)) }; SLOTS],
        }
    }

    /// Where `addr` is recorded, if it is borrowed at all.
    fn find(&self, addr: usize) -> Option<usize> {
        (0..self.len.get()).find(|&i| self.slots[i].get().0 == addr)
    }
}

thread_local! {
    static TABLE: Table = const { Table::new() };
}

impl BorrowTable {
    /// The state recorded for the cell at `addr`: a shared count, -1 for an
    /// exclusive borrow, or 0 when it is not borrowed at all.
    pub fn state(addr: usize) -> isize {
        TABLE.with(|table| table.find(addr).map_or(0, |i| table.slots[i].get().1))
    }

    /// Record one more borrow of the cell at `addr`, and answer the state it
    /// had *before* — which is what the caller checks the borrow against.
    pub fn acquire(addr: usize, exclusive: bool) -> isize {
        TABLE.with(|table| {
            let at = table.find(addr).unwrap_or_else(|| {
                let at = table.len.get();
                assert!(at < SLOTS, "the GlobalCell borrow table is full");
                table.len.set(at + 1);
                table.slots[at].set((addr, 0));
                at
            });
            let was = table.slots[at].get().1;
            table.slots[at].set((addr, if exclusive { -1 } else { was + 1 }));
            was
        })
    }

    /// Give one borrow of the cell at `addr` back, and answer the state it
    /// had before. The last borrow of a cell drops its entry, so the table
    /// is empty whenever no `with` is in flight.
    pub fn release(addr: usize, exclusive: bool) -> isize {
        TABLE.with(|table| {
            let at = table.find(addr).expect("borrow table entry lost");
            let was = table.slots[at].get().1;
            let now = if exclusive { 0 } else { was - 1 };
            if now == 0 {
                // Move the last entry down, so the prefix stays contiguous.
                let last = table.len.get() - 1;
                table.slots[at].set(table.slots[last].get());
                table.slots[last].set((0, 0));
                table.len.set(last);
            } else {
                table.slots[at].set((addr, now));
            }
            was
        })
    }
}
