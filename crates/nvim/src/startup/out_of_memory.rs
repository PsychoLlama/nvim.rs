//! What a failed Rust allocation does: the `E41` exit `xmalloc` takes,
//! rather than the standard library's bare abort.
//!
//! The global allocator (`allocator.rs`) calls [`allocation_failed`] when
//! `malloc` says no. Left alone, the standard library would print "memory
//! allocation of N bytes failed" and abort: no swap files written, the
//! terminal left in raw mode. Instead this writes the swap files out,
//! gives the terminal back, says `E41: Out of memory!` and exits 1, as
//! upstream's `xmalloc` does through `preserve_exit`.
//!
//! Stable Rust has no hook for this (`alloc_error_hook` is nightly), so it
//! sits in the allocator itself, behind the null check the allocator's
//! callers already take on the cold side.
//!
//! # Differences from `xmalloc`
//!
//! - **No retry after freeing editor memory.** `xmalloc` first calls
//!   `try_to_free_memory` (drop the scrollback, release memfile blocks) and
//!   tries again. That is sound only where no reference into those is live,
//!   which `xmalloc`'s callers promise and a safe `Vec::push` cannot. What
//!   this frees instead is a small reserve taken at startup, so the way out
//!   has room to allocate.
//! - **No `getout()`.** The allocation failed in the middle of Rust code,
//!   possibly inside a `GlobalCell` borrow, so like the panic hook this does
//!   not run autocommands or write ShaDa from there: it writes the swap
//!   files and exits.
//!
//! A request that asked to fail (`try_reserve`, through [`fallibly`]) is
//! answered null as before, and so is every request in a process that never
//! ran `main` (lib tests, the unit suite), where the standard library's own
//! handling stays in charge.

#![forbid(unsafe_code)]

use std::cell::Cell;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use super::exit::{abandon_loans, begin_emergency_exit};
use super::panic_hook::{
    append_to_log, clean_up, hand_off_to_main, log_line_start, main_thread_is_current,
    print_off_thread_report,
};
use crate::global_cell::emergency;
use crate::message::E_OUTOFMEM;
use crate::os::cshim::write_c_stderr;

/// Room for the way out: released before anything else on an allocation
/// failure, so that writing the log line and the swap files can allocate.
/// `calloc`'d and never touched, so it costs address space, not memory.
static RESERVE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// How much [`RESERVE`] holds: enough for the log line, the hand-off from
/// another thread and the memfile sync's own buffers.
const RESERVE_BYTES: usize = 256 * 1024;

/// Set by the first allocation failure that takes the way out. A second
/// one (the way out itself running dry, or another thread) aborts.
static FAILING: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Whether this thread's allocations may fail: set by [`fallibly`]
    /// around a `try_reserve`, whose caller handles the error itself.
    static FALLIBLE: Cell<bool> = const { Cell::new(false) };
}

/// Take the reserve the way out frees. Called from `main` once the panic
/// hook is in, before any editor state exists.
pub(super) fn take_reserve() {
    let reserve = vec![0; RESERVE_BYTES];
    *RESERVE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(reserve);
}

/// Run `request` with this thread's allocation failures answered null
/// instead of ending the process: for a `try_reserve` whose caller reports
/// the failure itself (`E342`) and carries on.
pub(crate) fn fallibly<R>(request: impl FnOnce() -> R) -> R {
    let outer = FALLIBLE.replace(true);
    let answer = request();
    FALLIBLE.set(outer);
    answer
}

/// The global allocator could not serve `size` bytes. Returns, for the
/// allocator to answer null, only when the request was [`fallibly`] made or
/// the process has no emergency exit (no `main`); otherwise ends the
/// process.
pub(crate) fn allocation_failed(size: usize) {
    if FALLIBLE.get() {
        return;
    }
    let Some(on_main_thread) = main_thread_is_current() else {
        return;
    };
    if FAILING.swap(true, Ordering::SeqCst) {
        // The way out ran out of memory too, or another thread's request
        // failed meanwhile: nothing left to try.
        std::process::abort();
    }
    drop(
        RESERVE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take(),
    );

    let current = thread::current();
    let thread = current.name().unwrap_or("<unnamed>");
    let mut line = log_line_start("out of memory");
    // `writeln!` into a `Vec` cannot fail.
    _ = writeln!(line, "thread '{thread}' failed to allocate {size} bytes");
    append_to_log(&line);

    if !on_main_thread {
        let text = String::from_utf8_lossy(E_OUTOFMEM.to_bytes()).into_owned() + "\n";
        hand_off_to_main(text, exit_after_another_thread);
    }
    // Whatever the main thread was inside of never resumes, and may have
    // held a `GlobalCell` borrow or a buffer guard: abandon them, as the
    // panic hook does, so the cleanup does not trip a borrow check.
    emergency(|emergency| {
        abandon_loans(emergency);
        if !begin_emergency_exit() {
            // A deadly signal or a panic is already on its way out, and this
            // allocation cannot be answered.
            std::process::abort();
        }
        exit_out_of_memory(report_out_of_memory)
    })
}

/// The main thread's half of an allocation failure on another thread, run
/// from its loop's fast queue.
fn exit_after_another_thread() {
    if !begin_emergency_exit() {
        // A deadly signal or a panic is already on its way out, and ends
        // the process itself. Back to the loop, so nothing it was inside of
        // is abandoned.
        return;
    }
    emergency(|emergency| {
        abandon_loans(emergency);
        exit_out_of_memory(print_off_thread_report)
    })
}

/// Say `E41` on the terminal the way `preserve_exit` does.
fn report_out_of_memory() {
    write_c_stderr(E_OUTOFMEM.to_bytes());
    write_c_stderr(b"\n");
}

/// On the main thread, once [`begin_emergency_exit`] has said this is the
/// way out: the panic hook's cleanup with `report` for its message, then
/// exit 1, upstream's status for `E41`.
fn exit_out_of_memory(report: impl FnOnce()) -> ! {
    clean_up(report);
    std::process::exit(1);
}
