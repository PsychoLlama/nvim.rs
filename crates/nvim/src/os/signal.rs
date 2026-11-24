//! The signals the editor watches, and what each one means to it.
//!
//! libuv owns the delivery: one `SignalWatcher` per signal, registered with
//! the main loop at start-up and torn down at exit. The watchers live in a
//! `static` because libuv keeps their addresses in its handles for the
//! process's lifetime. Handlers here run from the event loop, not from a
//! signal-handler context, so they may touch editor state freely.
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::autocmd::apply_autocmds;
use crate::autocmd::defs::AutoEvent;
use crate::eval::vars::set_vim_var_nr;
use crate::event::signal::SignalWatcher;
use crate::ex_cmds2::autowrite_all;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, LOGLVL_INF, logmsg};
use crate::memline::ml_sync_all;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::option::vars::p_awa;
use crate::startup::{V_DYING, preserve_exit};
use crate::types::Vv;
use crate::winlayer::Buf;
use ::libc::{pthread_sigmask, sigemptyset, sigset_t};
use core::ffi::{CStr, c_int};
use core::ptr;

// Signal numbers, for this module and for anything else that names one. Only
// the values Linux and the BSDs agree on are listed without qualification;
// these are the numbers the editor builds against.

/// The signals the editor watches, in the order upstream registered them.
/// `WATCHERS[i]` is the watcher for `WATCHED[i]`.
const WATCHED: [c_int; 9] = [
    libc::SIGPIPE,
    libc::SIGHUP,
    libc::SIGINT,
    libc::SIGQUIT,
    libc::SIGTERM,
    libc::SIGTSTP,
    libc::SIGPWR,
    libc::SIGUSR1,
    libc::SIGWINCH,
];

/// One watcher per signal: `WATCHERS[i]` watches `WATCHED[i]`.
static WATCHERS: GlobalCell<[SignalWatcher; WATCHED.len()]> =
    GlobalCell::new([const { SignalWatcher::unopened() }; WATCHED.len()]);

/// Whether deadly signals are currently being ignored — see
/// [`signal_reject_deadly`].
static REJECTING_DEADLY: GlobalCell<bool> = GlobalCell::new(false);

/// Register a watcher per signal in [`WATCHED`] with the main loop and start
/// them.
pub fn signal_init() {
    // `sigemptyset` fills it in; libc's `sigset_t` has no public fields, and
    // the C spelled this the same way (an uninitialised local).
    let mut mask: sigset_t = unsafe { core::mem::zeroed() };
    // SAFETY: `mask` is a live out-parameter; the log call takes no format
    // arguments.
    unsafe {
        // Ensure a clean slate by unblocking all signals. If SIGCHLD is
        // blocked, for instance, libuv may hang after spawning a subprocess
        // on Linux. #5230
        sigemptyset(&raw mut mask);
        if pthread_sigmask(libc::SIG_SETMASK, &raw mut mask, ptr::null_mut()) != 0 {
            logmsg!(
                LOGLVL_ERR,
                c"signal_init",
                47,
                "Could not unblock signals, nvim might behave strangely."
            );
        }
    }
    WATCHERS.with(|watchers| {
        for watcher in watchers {
            // SAFETY: the watchers live in a static, at their addresses for
            // the process.
            unsafe { watcher.open() };
        }
    });
    signal_start();
}

/// Stop and close every watcher.
pub fn signal_teardown() {
    signal_stop();
    WATCHERS.with(|watchers| watchers.iter().for_each(SignalWatcher::close));
}

/// Start delivering every watched signal to [`on_signal`].
pub fn signal_start() {
    WATCHERS.with(|watchers| {
        for (watcher, &signum) in watchers.iter().zip(&WATCHED) {
            watcher.start(handle_signal, signum);
        }
    });
}

/// Stop delivering watched signals.
pub fn signal_stop() {
    WATCHERS.with(|watchers| watchers.iter().for_each(SignalWatcher::stop));
}

/// Ignore deadly signals until [`signal_accept_deadly`]. Used while a child
/// process owns the terminal, so its CTRL-C does not kill the editor too.
pub fn signal_reject_deadly() {
    REJECTING_DEADLY.set(true);
}

/// Resume acting on deadly signals.
pub fn signal_accept_deadly() {
    REJECTING_DEADLY.set(false);
}

fn signal_name(signum: c_int) -> &'static CStr {
    match signum {
        libc::SIGPWR => c"SIGPWR",
        libc::SIGPIPE => c"SIGPIPE",
        libc::SIGTERM => c"SIGTERM",
        libc::SIGTSTP => c"SIGTSTP",
        libc::SIGQUIT => c"SIGQUIT",
        libc::SIGHUP => c"SIGHUP",
        libc::SIGINT => c"SIGINT",
        libc::SIGUSR1 => c"SIGUSR1",
        libc::SIGWINCH => c"SIGWINCH",
        _ => c"Unknown",
    }
}

/// Handle a deadly signal: preserve any swap files and exit properly (partly
/// from Elvis). Reached from the event loop, not from a signal handler.
fn deadly_signal(signum: c_int) -> ! {
    let name = signal_name(signum);
    let dying = crate::formatted!(
        crate::message::IOSIZE as usize,
        b"Nvim: Caught deadly signal '%s'\n",
        name
    );
    // Set the v:dying variable.
    set_vim_var_nr(Vv::Dying, 1);
    V_DYING.set(1);
    logmsg!(
        LOGLVL_INF,
        c"deadly_signal",
        196,
        "got signal {} ({})",
        signum,
        msg_cstr(name)
    );
    if p_awa() && signum != libc::SIGTERM && signum != libc::SIGINT {
        autowrite_all();
    }
    // Preserve files and exit.
    preserve_exit(Some(&crate::cstr::owned(&dying)))
}

/// What each watched signal means to the editor.
fn handle_signal(signum: c_int) {
    debug_assert!(signum >= 0);
    // Every call below is an ordinary main-thread editor call; this runs
    // from the event loop rather than from a signal handler.
    match signum {
        // Power failure (eg batteries low): flush the swap files to be
        // safe.
        libc::SIGPWR => ml_sync_all(0, 0, true),
        libc::SIGPIPE => {}
        libc::SIGTSTP => {
            if p_awa() {
                autowrite_all();
            }
        }
        libc::SIGHUP | libc::SIGINT | libc::SIGTERM | libc::SIGQUIT => {
            if !REJECTING_DEADLY.get() {
                deadly_signal(signum);
            }
        }
        // The autocommand is named for the signal, which is exactly what
        // signal_name yields.
        libc::SIGUSR1 | libc::SIGWINCH => {
            let arg = Buf::current().read().name.shown().map(XString::from_cstr);
            apply_autocmds(
                AutoEvent::Signal,
                Some(XString::from_cstr(signal_name(signum))),
                arg,
                true,
                Buf::current_or_none(),
            );
        }
        _ => {
            logmsg!(LOGLVL_ERR, c"on_signal", 254, "invalid signal: {}", signum);
        }
    }
}
