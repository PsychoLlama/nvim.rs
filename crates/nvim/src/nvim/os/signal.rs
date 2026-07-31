//! The signals the editor watches, and what each one means to it.
//!
//! libuv owns the delivery: one `SignalWatcher` per signal, registered with
//! the main loop at start-up and torn down at exit. The watchers live in a
//! `static` because libuv keeps their addresses in its handles for the
//! process's lifetime. Handlers here run from the event loop, not from a
//! signal-handler context, so they may touch editor state freely.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::autocmd::{EVENT_SIGNAL, apply_autocmds};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::event::signal::{
    signal_watcher_close, signal_watcher_init, signal_watcher_start, signal_watcher_stop,
};
use crate::src::nvim::ex_cmds2::autowrite_all;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_ERR, LOGLVL_INF, logmsg};
use crate::src::nvim::main::{IObuff, curbuf, main_loop, p_awa, preserve_exit, v_dying};
use crate::src::nvim::memline::ml_sync_all;
use crate::src::nvim::os::libc::snprintf;
pub use crate::src::nvim::types::{
    SignalWatcher, VimVarIndex, auto_event, uv__queue, uv_handle_type, uv_signal_s_tree_entry,
    uv_signal_s_u, uv_signal_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

unsafe extern "C" {
    fn sigemptyset(set: *mut sigset_t) -> c_int;
    fn pthread_sigmask(how: c_int, newmask: *const sigset_t, oldmask: *mut sigset_t) -> c_int;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sigset_t {
    pub __val: [::core::ffi::c_ulong; 16],
}
pub type sigset_t = __sigset_t;

// Signal numbers, for this module and for anything else that names one. Only
// the values Linux and the BSDs agree on are listed without qualification;
// these are the numbers the editor builds against.
pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGKILL: c_int = 9;
pub const SIGUSR1: c_int = 10;
pub const SIGPIPE: c_int = 13;
pub const SIGALRM: c_int = 14;
pub const SIGTERM: c_int = 15;
pub const SIGCHLD: c_int = 17;
pub const SIGCONT: c_int = 18;
pub const SIGTSTP: c_int = 20;
pub const SIGWINCH: c_int = 28;
pub const SIGPWR: c_int = 30;

const SIG_SETMASK: c_int = 2;
const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
const IOSIZE: usize = 1025;
const VV_DYING: VimVarIndex = 29;

/// The signals the editor watches, in the order upstream registered them.
/// `WATCHERS[i]` is the watcher for `WATCHED[i]`.
const WATCHED: [c_int; 9] = [
    SIGPIPE, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP, SIGPWR, SIGUSR1, SIGWINCH,
];

/// A watcher before `signal_watcher_init` has filled it in.
const WATCHER_UNSET: SignalWatcher = SignalWatcher {
    uv: uv_signal_t {
        data: ptr::null_mut(),
        loop_0: ptr::null_mut(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        },
        u: uv_signal_s_u { fd: 0 },
        next_closing: ptr::null_mut(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: uv_signal_s_tree_entry {
            rbe_left: ptr::null_mut(),
            rbe_right: ptr::null_mut(),
            rbe_parent: ptr::null_mut(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ptr::null_mut(),
    cb: None,
    close_cb: None,
    events: ptr::null_mut(),
};

static WATCHERS: GlobalCell<[SignalWatcher; WATCHED.len()]> =
    GlobalCell::new([WATCHER_UNSET; WATCHED.len()]);

/// Whether deadly signals are currently being ignored — see
/// [`signal_reject_deadly`].
static REJECTING_DEADLY: GlobalCell<bool> = GlobalCell::new(false);

/// The watcher for the `i`th watched signal. `wrapping_add` keeps this a safe
/// `fn`; `i` is always an index into `WATCHED`, so the result is in bounds.
fn watcher(i: usize) -> *mut SignalWatcher {
    WATCHERS.ptr().cast::<SignalWatcher>().wrapping_add(i)
}

/// Register a watcher per signal in [`WATCHED`] with the main loop and start
/// them.
pub fn signal_init() {
    let mut mask = sigset_t { __val: [0; 16] };
    // SAFETY: `mask` is a live out-parameter; the log call takes no format
    // arguments; the main loop is initialised before this runs and every
    // watcher address comes from the static array.
    unsafe {
        // Ensure a clean slate by unblocking all signals. If SIGCHLD is
        // blocked, for instance, libuv may hang after spawning a subprocess
        // on Linux. #5230
        sigemptyset(&raw mut mask);
        if pthread_sigmask(SIG_SETMASK, &raw mut mask, ptr::null_mut()) != 0 {
            logmsg(
                LOGLVL_ERR,
                ptr::null(),
                c"signal_init".as_ptr(),
                47,
                true,
                c"Could not unblock signals, nvim might behave strangely.".as_ptr(),
            );
        }
        for i in 0..WATCHED.len() {
            signal_watcher_init(main_loop.ptr(), watcher(i), ptr::null_mut());
        }
    }
    signal_start();
}

/// Stop and close every watcher.
pub fn signal_teardown() {
    signal_stop();
    // SAFETY: every watcher was initialised by signal_init and outlives the
    // close, which libuv completes on a later loop tick.
    unsafe {
        for i in 0..WATCHED.len() {
            signal_watcher_close(watcher(i), None);
        }
    }
}

/// Start delivering every watched signal to [`on_signal`].
pub fn signal_start() {
    // SAFETY: every watcher was initialised by signal_init.
    unsafe {
        for (i, &signum) in WATCHED.iter().enumerate() {
            signal_watcher_start(watcher(i), Some(on_signal), signum);
        }
    }
}

/// Stop delivering watched signals.
pub fn signal_stop() {
    // SAFETY: every watcher was initialised by signal_init.
    unsafe {
        for i in 0..WATCHED.len() {
            signal_watcher_stop(watcher(i));
        }
    }
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
        SIGPWR => c"SIGPWR",
        SIGPIPE => c"SIGPIPE",
        SIGTERM => c"SIGTERM",
        SIGTSTP => c"SIGTSTP",
        SIGQUIT => c"SIGQUIT",
        SIGHUP => c"SIGHUP",
        SIGINT => c"SIGINT",
        SIGUSR1 => c"SIGUSR1",
        SIGWINCH => c"SIGWINCH",
        _ => c"Unknown",
    }
}

/// Handle a deadly signal: preserve any swap files and exit properly (partly
/// from Elvis). Reached from the event loop, not from a signal handler.
fn deadly_signal(signum: c_int) -> ! {
    let name = signal_name(signum).as_ptr();
    // SAFETY: main-thread editor state; both log/format calls' arguments
    // match their format strings, and IObuff is IOSIZE chars.
    unsafe {
        // Set the v:dying variable.
        set_vim_var_nr(VV_DYING, 1);
        v_dying.set(1);
        logmsg(
            LOGLVL_INF,
            ptr::null(),
            c"deadly_signal".as_ptr(),
            196,
            true,
            c"got signal %d (%s)".as_ptr(),
            signum,
            name,
        );
        snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE,
            c"Nvim: Caught deadly signal '%s'\n".as_ptr(),
            name,
        );
        if p_awa.get() != 0 && signum != SIGTERM && signum != SIGINT {
            autowrite_all();
        }
        // Preserve files and exit.
        preserve_exit(IObuff.ptr() as *mut c_char)
    }
}

/// The `signal_cb` libuv's watchers are started with: unpack and hand over.
unsafe extern "C" fn on_signal(_watcher: *mut SignalWatcher, signum: c_int, _data: *mut c_void) {
    handle_signal(signum);
}

/// What each watched signal means to the editor.
fn handle_signal(signum: c_int) {
    assert!(signum >= 0);
    // SAFETY: every call below is an ordinary main-thread editor call; this
    // runs from the event loop rather than from a signal handler. `curbuf` is
    // non-null for as long as the editor is running.
    unsafe {
        match signum {
            // Power failure (eg batteries low): flush the swap files to be
            // safe.
            SIGPWR => ml_sync_all(0, 0, true),
            SIGPIPE => {}
            SIGTSTP => {
                if p_awa.get() != 0 {
                    autowrite_all();
                }
            }
            SIGHUP | SIGINT | SIGTERM | SIGQUIT => {
                if !REJECTING_DEADLY.get() {
                    deadly_signal(signum);
                }
            }
            // The autocommand is named for the signal, which is exactly what
            // signal_name yields.
            SIGUSR1 | SIGWINCH => {
                apply_autocmds(
                    EVENT_SIGNAL,
                    signal_name(signum).as_ptr() as *mut c_char,
                    (*curbuf.get()).b_fname,
                    true,
                    curbuf.get(),
                );
            }
            _ => {
                logmsg(
                    LOGLVL_ERR,
                    ptr::null(),
                    c"on_signal".as_ptr(),
                    254,
                    true,
                    c"invalid signal: %d".as_ptr(),
                    signum,
                );
            }
        }
    }
}
