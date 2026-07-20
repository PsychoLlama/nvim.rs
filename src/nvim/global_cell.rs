//! `GlobalCell<T>`: the checked cell that replaces `static mut` globals.
//!
//! The transpiled editor state is a web of C globals that c2rust rendered as
//! `static mut`. Any two overlapping references into one of those — an
//! autocmd firing mid-operation and touching the same global, say — is
//! undefined behavior even single-threaded. `GlobalCell` funnels every
//! access through a single `UnsafeCell` so the compiler stops assuming
//! uniqueness, and adds debug-build enforcement of the invariants the
//! soundness argument rests on.
//!
//! # Soundness contract
//!
//! - **Main-thread only.** Editor state is only touched from the main
//!   thread. `unsafe impl Sync` below is justified by that invariant alone.
//!   Debug builds assert it on every access (see [`init_main_thread`]);
//!   release builds compile the check out.
//! - **No outstanding references across accesses.** `get`/`set` copy values
//!   in and out without forming references. `with`/`with_mut` hand out a
//!   scoped reference and track it (debug builds panic on conflicting
//!   reentry, RefCell-style). `ptr` is the raw escape hatch for the
//!   mechanically converted call sites: writes/reads through it are exempt
//!   from borrow tracking, and callers must not let a reference from
//!   `with`/`with_mut` overlap such an access.
//!
//! The layout is `repr(transparent)`, so an unmangled `pub static X:
//! GlobalCell<T>` exports the same symbol with the same object layout as
//! the mutable static it replaces — the C deps and the LuaJIT-FFI unit
//! suite keep resolving and poking the same bytes.

use core::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// A global editor-state cell. See the module docs for the contract.
#[repr(transparent)]
pub struct GlobalCell<T>(UnsafeCell<T>);

// SAFETY: editor globals are only accessed from the main thread (checked in
// debug builds). The `Sync` bound is required for `static` items; it is not
// a claim of actual thread safety.
unsafe impl<T> Sync for GlobalCell<T> {}

impl<T> GlobalCell<T> {
    pub const fn new(value: T) -> Self {
        GlobalCell(UnsafeCell::new(value))
    }

    /// Raw pointer to the cell contents, with the debug main-thread check.
    ///
    /// This is what the mechanical `static mut` conversion emits for place
    /// expressions (field projections, `&raw mut`, index, ...). Accesses
    /// through it carry exactly the obligations the old `static mut` access
    /// did, minus the reference-uniqueness landmine.
    pub fn ptr(&self) -> *mut T {
        check_main_thread(self.0.get() as usize);
        self.0.get()
    }

    /// Like [`ptr`](Self::ptr) but callable in const initializers (some
    /// globals hold the address of another global). No debug checks.
    pub const fn as_raw(&self) -> *mut T {
        self.0.get()
    }

    /// Copy the value out.
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        check_main_thread(self.0.get() as usize);
        check_no_exclusive_borrow(self.0.get() as usize);
        // SAFETY: main-thread invariant + no outstanding exclusive borrow.
        unsafe { *self.0.get() }
    }

    /// Overwrite the value.
    pub fn set(&self, value: T) {
        check_main_thread(self.0.get() as usize);
        check_no_borrow(self.0.get() as usize);
        // SAFETY: main-thread invariant + no outstanding borrow.
        unsafe { *self.0.get() = value }
    }

    /// Swap in a new value, returning the old one.
    pub fn replace(&self, value: T) -> T
    where
        T: Copy,
    {
        let old = self.get();
        self.set(value);
        old
    }

    /// Run `f` with a shared reference to the contents.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        check_main_thread(self.0.get() as usize);
        let _guard = BorrowGuard::shared(self.0.get() as usize);
        // SAFETY: main-thread invariant + borrow tracking (debug).
        f(unsafe { &*self.0.get() })
    }

    /// Run `f` with an exclusive reference to the contents.
    ///
    /// Debug builds panic if the cell is already borrowed — this is the
    /// probe that turns vim's historic autocmd-reentrancy corruption into a
    /// loud failure instead of silent UB.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        check_main_thread(self.0.get() as usize);
        let _guard = BorrowGuard::exclusive(self.0.get() as usize);
        // SAFETY: main-thread invariant + borrow tracking (debug).
        f(unsafe { &mut *self.0.get() })
    }
}

/// Set once the binary's `main` has recorded its thread. Before that (unit
/// tests FFI-loading the symbols, `.init_array` constructors, `cargo test`)
/// the main-thread check is inert.
static STARTED: AtomicBool = AtomicBool::new(false);

#[cfg(debug_assertions)]
thread_local! {
    static IS_MAIN: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

/// Record the calling thread as the main thread and arm the debug
/// main-thread assertion. Called from the binary entry point before any
/// editor code runs. Idempotent; re-marking from another thread is a bug.
pub fn init_main_thread() {
    #[cfg(debug_assertions)]
    IS_MAIN.with(|is_main| is_main.set(true));
    STARTED.store(true, Ordering::Release);
}

#[cfg(debug_assertions)]
fn check_main_thread(addr: usize) {
    if STARTED.load(Ordering::Acquire) && !IS_MAIN.with(|is_main| is_main.get()) {
        panic!(
            "GlobalCell accessed off the main thread (cell @ {addr:#x}, thread {:?})",
            std::thread::current().id()
        );
    }
}

#[cfg(not(debug_assertions))]
fn check_main_thread(_addr: usize) {}

// Borrow tracking. Only `with`/`with_mut` create tracked borrows, so the
// common get/set path just checks a counter and bails while the table is
// empty. Keyed by cell address; positive = shared count, -1 = exclusive.
#[cfg(debug_assertions)]
mod borrows {
    use core::cell::{Cell, RefCell};
    use std::collections::HashMap;

    thread_local! {
        pub static ACTIVE: Cell<usize> = const { Cell::new(0) };
        pub static TABLE: RefCell<HashMap<usize, isize>> = RefCell::new(HashMap::new());
    }
}

#[cfg(debug_assertions)]
fn check_no_exclusive_borrow(addr: usize) {
    if borrows::ACTIVE.with(|active| active.get()) == 0 {
        return;
    }
    borrows::TABLE.with(|table| {
        if let Some(&state) = table.borrow().get(&addr) {
            if state < 0 {
                panic!("GlobalCell::get during an active with_mut borrow (cell @ {addr:#x})");
            }
        }
    });
}

#[cfg(not(debug_assertions))]
fn check_no_exclusive_borrow(_addr: usize) {}

#[cfg(debug_assertions)]
fn check_no_borrow(addr: usize) {
    if borrows::ACTIVE.with(|active| active.get()) == 0 {
        return;
    }
    borrows::TABLE.with(|table| {
        if table.borrow().get(&addr).is_some() {
            panic!("GlobalCell::set during an active with/with_mut borrow (cell @ {addr:#x})");
        }
    });
}

#[cfg(not(debug_assertions))]
fn check_no_borrow(_addr: usize) {}

struct BorrowGuard {
    #[cfg(debug_assertions)]
    addr: usize,
    #[cfg(debug_assertions)]
    exclusive: bool,
}

impl BorrowGuard {
    fn shared(addr: usize) -> Self {
        #[cfg(debug_assertions)]
        {
            borrows::TABLE.with(|table| {
                let mut table = table.borrow_mut();
                let state = table.entry(addr).or_insert(0);
                if *state < 0 {
                    panic!(
                        "GlobalCell::with while exclusively borrowed (cell @ {addr:#x}) — \
                         reentrant global access"
                    );
                }
                *state += 1;
            });
            borrows::ACTIVE.with(|active| active.set(active.get() + 1));
            BorrowGuard {
                addr,
                exclusive: false,
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = addr;
            BorrowGuard {}
        }
    }

    fn exclusive(addr: usize) -> Self {
        #[cfg(debug_assertions)]
        {
            borrows::TABLE.with(|table| {
                let mut table = table.borrow_mut();
                let state = table.entry(addr).or_insert(0);
                if *state != 0 {
                    panic!(
                        "GlobalCell::with_mut while already borrowed (cell @ {addr:#x}) — \
                         reentrant global access"
                    );
                }
                *state = -1;
            });
            borrows::ACTIVE.with(|active| active.set(active.get() + 1));
            BorrowGuard {
                addr,
                exclusive: true,
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = addr;
            BorrowGuard {}
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for BorrowGuard {
    fn drop(&mut self) {
        borrows::TABLE.with(|table| {
            let mut table = table.borrow_mut();
            let state = table.get_mut(&self.addr).expect("borrow table entry lost");
            if self.exclusive {
                debug_assert_eq!(*state, -1);
                *state = 0;
            } else {
                debug_assert!(*state > 0);
                *state -= 1;
            }
            if *state == 0 {
                table.remove(&self.addr);
            }
        });
        borrows::ACTIVE.with(|active| active.set(active.get() - 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_replace_roundtrip() {
        static CELL: GlobalCell<i32> = GlobalCell::new(7);
        assert_eq!(CELL.get(), 7);
        CELL.set(9);
        assert_eq!(CELL.replace(11), 9);
        assert_eq!(CELL.get(), 11);
    }

    #[test]
    fn ptr_and_as_raw_agree_and_are_transparent() {
        static CELL: GlobalCell<u64> = GlobalCell::new(0xdead);
        assert_eq!(CELL.ptr(), CELL.as_raw());
        // repr(transparent): the cell's address IS the value's address.
        assert_eq!(&CELL as *const _ as usize, CELL.as_raw() as usize);
        assert_eq!(core::mem::size_of::<GlobalCell<u64>>(), 8);
    }

    #[test]
    fn with_mut_scopes_nest_across_distinct_cells() {
        static A: GlobalCell<i32> = GlobalCell::new(1);
        static B: GlobalCell<i32> = GlobalCell::new(2);
        let sum = A.with_mut(|a| {
            *a += 10;
            B.with(|b| *a + *b)
        });
        assert_eq!(sum, 13);
        assert_eq!(A.get(), 11);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "reentrant global access")]
    fn reentrant_with_mut_panics_in_debug() {
        static CELL: GlobalCell<i32> = GlobalCell::new(0);
        CELL.with_mut(|_| CELL.with_mut(|inner| *inner));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "with_mut borrow")]
    fn get_during_with_mut_panics_in_debug() {
        static CELL: GlobalCell<i32> = GlobalCell::new(0);
        CELL.with_mut(|_| CELL.get());
    }

    #[test]
    fn shared_borrows_stack() {
        static CELL: GlobalCell<i32> = GlobalCell::new(5);
        let v = CELL.with(|a| CELL.with(|b| *a + *b));
        assert_eq!(v, 10);
        // Fully released: an exclusive borrow works again afterwards.
        CELL.with_mut(|x| *x = 6);
        assert_eq!(CELL.get(), 6);
    }
}
