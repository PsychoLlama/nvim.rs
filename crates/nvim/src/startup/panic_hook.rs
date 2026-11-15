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
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, ThreadId};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::UI_CLIENT_CHANNEL_ID;
use super::exit::{begin_emergency_exit, preserve_swap_files, restore_stdin};
use crate::ui_client::ui_client_stop;

/// The thread that installed the hook: the main thread, the only one that
/// may touch editor state. Kept here rather than asked of
/// `nvim_mainthread`, whose flag a release build never sets.
static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

/// Set by the first panic to reach the hook, on any thread.
static PANICKING: AtomicBool = AtomicBool::new(false);

/// How long a second thread's panic waits for the first one's cleanup to
/// abort the process before it aborts it itself.
const SECOND_PANIC_GRACE: Duration = Duration::from_secs(10);

/// Make every panic end the process through [`on_panic`]. Called first
/// thing in `main`, on the main thread.
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
    if PANICKING.swap(true, Ordering::SeqCst) {
        // Another thread is already handling a panic and will abort the
        // process when its cleanup is done; stay out of its way, but do not
        // trust it to finish.
        thread::sleep(SECOND_PANIC_GRACE);
        std::process::abort();
    }

    // The log first: it is not the terminal, and whatever goes wrong in the
    // cleanup below, the message and where it came from are already out.
    write_to_log(info);

    let on_main_thread = MAIN_THREAD.get() == Some(&thread::current().id());
    if !on_main_thread {
        // Editor state belongs to the main thread, which is still running
        // whatever it was running; reaching into it from here would race.
        // The swap files on disk are as recent as the last sync.
        report(info);
        std::process::abort();
    }

    // The panic may have fired inside a `GlobalCell::with`/`with_mut`. Its
    // owner never runs again, so the borrow is dead; forget it, or the
    // first cell the cleanup touches would panic on the borrow tracker.
    #[cfg(debug_assertions)]
    nvim_mainthread::BorrowTable::forget_all();

    if !begin_emergency_exit() {
        // The panic fired on a way out that has already started preserving
        // (a deadly signal, out of memory, or a panic during one of them):
        // going round again would only repeat what broke.
        report(info);
        std::process::abort();
    }

    let ui_client = UI_CLIENT_CHANNEL_ID.get() != 0;
    if ui_client {
        // Leave the alternate screen and raw mode, so the message can be
        // read and the shell gets a sane terminal back.
        ui_client_stop();
    }
    report(info);
    if !ui_client {
        // A UI client has no buffers of its own; its server keeps them.
        preserve_swap_files(true);
    }
    restore_stdin();
    std::process::abort();
}

/// Append the panic, its thread and a backtrace to the log file.
///
/// Goes around `logmsg!` on purpose: that reads editor state (which this
/// may not be the thread for) and refuses a line while another is being
/// written (which the panic may have interrupted). The file is the one
/// `log_init` chose and published as `$NVIM_LOG_FILE`; with none, the
/// backtrace goes to stderr instead.
fn write_to_log(info: &PanicHookInfo<'_>) {
    let mut text = Vec::new();
    // `writeln!` into a `Vec` cannot fail.
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let current = thread::current();
    let thread = current.name().unwrap_or("<unnamed>");
    _ = writeln!(
        text,
        "ERR {seconds} nvim.{pid} panic: thread '{thread}' {info}",
        pid = std::process::id(),
    );
    _ = writeln!(text, "stack backtrace:\n{}", Backtrace::force_capture());

    let appended = std::env::var_os("NVIM_LOG_FILE")
        .filter(|path| !path.is_empty())
        .and_then(|path| OpenOptions::new().append(true).create(true).open(path).ok())
        .is_some_and(|mut file| file.write_all(&text).is_ok());
    if !appended {
        _ = std::io::stderr().write_all(&text);
    }
}
