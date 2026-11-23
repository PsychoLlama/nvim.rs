//! What a panic does: end the process, losing no more than a deadly signal
//! would.
//!
//! A panic means an invariant broke, and after that the editor's state is
//! not to be trusted, so nothing tries to recover into an error. The hook
//! [`install_panic_hook`] sets writes the message and a backtrace to the log,
//! restores the terminal, writes the swap files out as the deadly-signal
//! path ([`preserve_exit`](super::preserve_exit)) does, and calls `abort()`.
//! It never returns, so nothing unwinds: no destructor runs over the broken
//! state, and the unwind never reaches an `extern "C"` frame.
//!
//! A panic on another thread cannot touch the editor's state, so it hands
//! the same cleanup to the main thread's loop and waits for it to abort the
//! process (see [`hand_off_to_main`]).
//!
//! The policy, and why it is a hook rather than `panic = "abort"`, is
//! `docs/panic-policy.md`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::backtrace::Backtrace;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::UI_CLIENT_CHANNEL_ID;
use super::exit::{abandon_loans, begin_emergency_exit, preserve_swap_files, restore_stdin};
use crate::event::r#loop::schedule_fast_job;
use crate::global_cell::emergency;
use crate::ui_client::ui_client_stop;

/// The thread that installed the hook: the main thread, the only one that
/// may touch editor state. Kept here rather than asked of
/// `nvim_mainthread`, so that a process that never armed the cells' thread
/// check (lib tests, the unit suite) answers `None`.
static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

/// Set once the main thread has started the cleanup, from its own panic or
/// from another thread's hand-off. A thread waiting on it then gives the
/// cleanup longer to finish.
static CLEANING_UP: AtomicBool = AtomicBool::new(false);

/// The message of a panic off the main thread, for the main thread to print
/// once the terminal is back. Taken by whichever side prints it.
static OFF_THREAD_REPORT: Mutex<Option<String>> = Mutex::new(None);

/// How long a panic off the main thread waits for the main thread to pick
/// up the cleanup. The main thread polls its loop between commands and at
/// every `os_breakcheck()`, so this is only spent in full when it is stuck:
/// blocked in a system call, or waiting on the very thread that panicked.
const HANDOFF_GRACE: Duration = Duration::from_secs(5);

/// How long it then waits for a cleanup that has started to abort the
/// process. Writing the swap files out is a sync per buffer.
const CLEANUP_GRACE: Duration = Duration::from_secs(10);

/// Make every panic end the process through [`on_panic`]. Called first
/// thing in `main`, on the main thread.
#[expect(
    clippy::disallowed_methods,
    reason = "`on_panic` touches editor state only on the main thread; \
              another thread's panic hands the cleanup to it"
)]
pub(super) fn install_panic_hook() {
    MAIN_THREAD.get_or_init(|| thread::current().id());
    // The standard hook still writes the panic to stderr in its usual shape
    // (and honours `RUST_BACKTRACE`); this one runs it once the terminal
    // can show it.
    let report = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| on_panic(info, &*report)));
}

/// The panic hook. Never returns.
///
/// A panic raised while this runs (the cleanup tripping over the broken
/// state) does not come back here: the standard library aborts the process
/// on a panic inside the hook. Either way the process ends.
fn on_panic(info: &PanicHookInfo<'_>, report: &(dyn Fn(&PanicHookInfo<'_>) + Sync + Send)) -> ! {
    // The log first: it is not the terminal, and whatever goes wrong in the
    // cleanup below, the message and where it came from are already out.
    write_to_log(info);

    if main_thread_is_current() == Some(false) {
        let current = thread::current();
        let text = format!(
            "thread '{}' {info}\n",
            current.name().unwrap_or("<unnamed>")
        );
        hand_off_to_main(text, clean_up_after_another_thread);
    }

    // The panic may have fired inside a `GlobalCell::with`/`with_mut`, or
    // with a buffer's `edit()` or memline walk held. Their owner never runs
    // again, so they are dead; abandon them, or the first cell or buffer the
    // cleanup touches would panic on its borrow check.
    emergency(|emergency| {
        abandon_loans(emergency);
        if !begin_emergency_exit() {
            // The panic fired on a way out that has already started
            // preserving (a deadly signal, out of memory, or a panic during
            // one of them): going round again would only repeat what broke.
            report(info);
            std::process::abort();
        }
        clean_up_and_abort(|| report(info))
    })
}

/// Whether this is the main thread, the only one that may touch editor
/// state. `None` before [`install_panic_hook`] ran, which is also the case
/// in lib tests and the unit suite: no `main`, no emergency exit.
pub(super) fn main_thread_is_current() -> Option<bool> {
    MAIN_THREAD
        .get()
        .map(|main| *main == thread::current().id())
}

/// An emergency on another thread (a `vim.uv.new_thread` state, libuv pool
/// work): editor state belongs to the main thread, which is still running
/// whatever it was running, so reaching into it from here would race.
/// Instead ask the main thread to run `cleanup` from its loop, and give it
/// time to end the process. If it never does (stuck, or waiting on this
/// thread), print `text` and abort anyway: the swap files are then as
/// recent as their last sync.
///
/// `cleanup` starts with [`begin_emergency_exit`] and then runs
/// [`clean_up`], which tells this thread to give it the longer grace.
pub(super) fn hand_off_to_main(text: String, cleanup: fn()) -> ! {
    *OFF_THREAD_REPORT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(text);
    schedule_fast_job(Box::new(cleanup));

    let started = Instant::now();
    while !CLEANING_UP.load(Ordering::SeqCst) && started.elapsed() < HANDOFF_GRACE {
        thread::sleep(Duration::from_millis(20));
    }
    if CLEANING_UP.load(Ordering::SeqCst) {
        thread::sleep(CLEANUP_GRACE);
    }
    print_off_thread_report();
    std::process::abort();
}

/// The main thread's half of [`hand_off_to_main`], run from its loop's fast
/// queue.
fn clean_up_after_another_thread() {
    if !begin_emergency_exit() {
        // A deadly signal or an out-of-memory exit is already preserving,
        // and ends the process itself. Back to the loop, so nothing it was
        // inside of is abandoned.
        return;
    }
    // Whatever the loop was inside of never resumes: see `on_panic`.
    emergency(|emergency| {
        abandon_loans(emergency);
        clean_up_and_abort(print_off_thread_report)
    })
}

/// Tell a thread waiting in [`hand_off_to_main`] that the main thread has
/// started the cleanup, so it should give it the longer grace.
fn mark_cleaning_up() {
    CLEANING_UP.store(true, Ordering::SeqCst);
}

/// Print the message [`hand_off_to_main`] left, if nobody has yet.
pub(super) fn print_off_thread_report() {
    let text = OFF_THREAD_REPORT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(text) = text {
        _ = std::io::stderr().write_all(text.as_bytes());
    }
}

/// On the main thread, once [`begin_emergency_exit`] has said this is the
/// way out: give the terminal back, `report` the panic on it, write the swap
/// files out and abort.
fn clean_up_and_abort(report: impl FnOnce()) -> ! {
    clean_up(report);
    std::process::abort();
}

/// What every emergency exit does on the main thread, once
/// [`begin_emergency_exit`] has said it is the way out: give the terminal
/// back, `report` on it, write the swap files out and put stdin back. The
/// caller then ends the process.
pub(super) fn clean_up(report: impl FnOnce()) {
    mark_cleaning_up();
    let ui_client = UI_CLIENT_CHANNEL_ID.get() != 0;
    if ui_client {
        // Leave the alternate screen and raw mode, so the message can be
        // read and the shell gets a sane terminal back.
        ui_client_stop();
    }
    report();
    if !ui_client {
        // A UI client has no buffers of its own; its server keeps them.
        preserve_swap_files(true);
    }
    restore_stdin();
}

/// Append the panic, its thread and a backtrace to the log file.
///
/// Goes around `logmsg!` on purpose: that reads editor state (which this
/// may not be the thread for) and refuses a line while another is being
/// written (which the panic may have interrupted). The file is the one
/// `log_init` chose and published as `$NVIM_LOG_FILE`; with none, the
/// backtrace goes to stderr instead.
fn write_to_log(info: &PanicHookInfo<'_>) {
    let current = thread::current();
    let thread = current.name().unwrap_or("<unnamed>");
    let mut text = log_line_start("panic");
    // `writeln!` into a `Vec` cannot fail.
    _ = writeln!(text, "thread '{thread}' {info}");
    _ = writeln!(text, "stack backtrace:\n{}", Backtrace::force_capture());
    append_to_log(&text);
}

/// The start of an emergency's log line, in the shape `logmsg!` writes:
/// `ERR <unix seconds> nvim.<pid> <what>: `.
pub(super) fn log_line_start(what: &str) -> Vec<u8> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    format!(
        "ERR {seconds} nvim.{pid} {what}: ",
        pid = std::process::id()
    )
    .into_bytes()
}

/// Append `text` to the log file `log_init` chose and published as
/// `$NVIM_LOG_FILE`, or write it to stderr when there is none.
pub(super) fn append_to_log(text: &[u8]) {
    let appended = std::env::var_os("NVIM_LOG_FILE")
        .filter(|path| !path.is_empty())
        .and_then(|path| OpenOptions::new().append(true).create(true).open(path).ok())
        .is_some_and(|mut file| file.write_all(text).is_ok());
    if !appended {
        _ = std::io::stderr().write_all(text);
    }
}
