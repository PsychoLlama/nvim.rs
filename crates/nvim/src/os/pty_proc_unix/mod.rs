//! Children started on a pseudo-terminal.
//!
//! A [`PtyProc`] is a [`Proc`] with the pty master descriptor and the window
//! size bolted on. `Proc` is the first field, so `event/proc.rs` casts freely
//! between the two.
//!
//! Unlike a libuv child, this one is forked here: `forkpty` allocates the
//! master/slave pair, forks, and makes the slave the child's controlling
//! terminal. The parent keeps the master, `dup`s it once per direction it
//! wants to talk over, and hands each copy to a `uv_pipe_t`. Reaping is done
//! by hand — [`chld_handler`], driven by libuv's SIGCHLD watcher — because
//! libuv only reaps the children it spawned itself.
//!
//! Some of this code came from pangoterm and libuv.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod termios;
mod wait_status;

use crate::eval::typval::tv_dict_to_env;
use crate::event::libuv::{
    uv_chdir, uv_disable_stdio_inheritance, uv_pipe_open, uv_signal_start, uv_signal_stop,
    uv_strerror,
};
use crate::event::r#loop::loop_children;
use crate::event::proc::{kProcTypePty, proc_get_exepath, proc_init};
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::message_fmt::{CDisplay, c_str};
use crate::os::cshim::environ;
use crate::os::fs::os_set_cloexec;
use crate::os::signal::{SIGALRM, SIGCHLD, SIGCONT, SIGHUP, SIGINT, SIGKILL, SIGQUIT, SIGTERM};
use crate::types::{Loop, Proc, PtyProc, speed_t, uv_pipe_t, uv_signal_t};
use ::libc::{
    __errno_location, _exit, SIG_DFL, cfsetispeed, cfsetospeed, close, dup, execvp, fcntl, forkpty,
    ioctl, kill, killpg, poll, pollfd, ptsname, setsid, signal, strerror, waitpid,
};
use core::ffi::{CStr, c_char, c_int, c_short, c_ulong, c_void};
use core::ptr;
use wait_status::ChildState;

const POLLIN: c_short = 0x1;
const EINTR: c_int = 4;

const O_NONBLOCK: c_int = 0o4000;
const F_GETFL: c_int = 3;
const F_SETFL: c_int = 4;

const TIOCSWINSZ: c_ulong = 0x5414;

/// `waitpid` flags: report stopped and continued children too, and never
/// block.
const WNOHANG: c_int = 1;
const WUNTRACED: c_int = 2;
const WCONTINUED: c_int = 8;

/// The most recent `errno`.
fn errno() -> c_int {
    // SAFETY: glibc's per-thread errno slot is always live.
    unsafe { *__errno_location() }
}

/// A fresh, unspawned pty child. The size is the traditional 80x24 until the
/// caller resizes it.
pub fn pty_proc_init(uv_loop: *mut Loop, data: *mut c_void) -> PtyProc {
    // SAFETY: every field is a pointer, an integer or an `Option<fn>`, for
    // all of which all-zeroes is the "unset" value.
    let mut rv: PtyProc = unsafe { core::mem::zeroed() };
    rv.proc = proc_init(uv_loop, kProcTypePty, data);
    rv.width = 80;
    rv.height = 24;
    rv.tty_fd = -1;
    rv
}

/// `strerror(errno)` as something a log line can print. Safe: `strerror`
/// answers a static string for any errno.
fn why() -> CDisplay<'static> {
    // SAFETY: a static, NUL-terminated string for any errno.
    unsafe { c_str(strerror(errno())) }
}

/// One `LOGLVL_ERR` line about a failed syscall, with `strerror(errno)` for
/// its tail.
///
/// A helper rather than a `logmsg!` per site because most of these sit
/// inside a body-wide `unsafe` block, where the seven lines rustfmt wraps
/// the call over are seven *unchecked* lines. One call, one line.
fn warn_errno(who: &'static CStr, line: c_int, what: &str) {
    logmsg!(LOGLVL_ERR, who, line, "{what}: {}", why());
}

/// [`warn_errno`] for a failure that carries no errno.
fn warn(who: &'static CStr, line: c_int, what: &str) {
    logmsg!(LOGLVL_ERR, who, line, "{what}");
}

/// Fork the child onto a new pseudo-terminal. Returns zero, or a negative
/// error code.
///
/// # Safety
///
/// `ptyproc` points to a live, unspawned [`PtyProc`] whose `Proc` half has
/// been initialized (`proc_init`) and whose loop outlives the child.
pub unsafe fn pty_proc_spawn(ptyproc: *mut PtyProc) -> c_int {
    // Built at first use and reused, as upstream did.
    static TERMIOS_DEFAULT: GlobalCell<crate::types::termios> =
        GlobalCell::new(crate::types::termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; 32],
            c_ispeed: 0,
            c_ospeed: 0,
        });
    if TERMIOS_DEFAULT.with(|t| t.c_cflag) == 0 {
        TERMIOS_DEFAULT.set(termios::default_termios());
        // Upstream passes the literal 38400 where these want a `B38400`
        // code, so both calls fail with EINVAL and leave the speeds — and
        // the baud bits of `c_cflag` — alone. Kept: passing `B38400`
        // instead would change what `tcsetattr` is given.
        const BOGUS_SPEED: speed_t = 38400;
        TERMIOS_DEFAULT.with_mut(|t| {
            // SAFETY: `t` is the cell's own live `termios`; both calls
            // reject the speed and leave it untouched.
            unsafe {
                cfsetispeed(t, BOGUS_SPEED);
                cfsetospeed(t, BOGUS_SPEED);
            }
        });
    }

    let proc = ptyproc.cast::<Proc>();
    // SAFETY: the caller's process. `Proc` is `PtyProc`'s first field, so
    // the cast addresses the same object; every field touched below is one
    // `proc_init` filled in.
    unsafe {
        // stderr is folded into the pty; there is one stream in each
        // direction.
        debug_assert!((*proc).err.s.closed);

        uv_signal_start(
            &raw mut (*(*proc).loop_0).children_watcher,
            Some(chld_handler),
            SIGCHLD,
        );
        (*ptyproc).winsize.ws_row = (*ptyproc).height;
        (*ptyproc).winsize.ws_col = (*ptyproc).width;
        (*ptyproc).winsize.ws_xpixel = 0;
        (*ptyproc).winsize.ws_ypixel = 0;
        uv_disable_stdio_inheritance();
    }

    let mut master: c_int = 0;
    // SAFETY: `forkpty` writes the master descriptor into `master` and reads
    // the two structs; both are live. In the child it returns 0 having
    // replaced this process's stdio with the slave.
    let pid = unsafe {
        forkpty(
            &raw mut master,
            ptr::null_mut(),
            TERMIOS_DEFAULT.ptr(),
            &raw mut (*ptyproc).winsize,
        )
    };
    if pid < 0 {
        let status = -errno();
        warn_errno(c"pty_proc_spawn", 190, "forkpty failed");
        return status;
    }
    if pid == 0 {
        // SAFETY: the child half of the fork; never returns.
        unsafe { init_child(ptyproc) };
    }

    let status = 'configure: {
        // SAFETY: `master` is the descriptor `forkpty` just handed back, and
        // the two pipes are this process's own.
        unsafe {
            // The master must be non-blocking: libuv polls it.
            let flags = fcntl(master, F_GETFL);
            if flags == -1 {
                // Captured before the logging, which does I/O of its own.
                let status = -errno();
                let what = "Failed to get master descriptor status flags";
                warn_errno(c"pty_proc_spawn", 200, what);
                break 'configure status;
            }
            if fcntl(master, F_SETFL, flags | O_NONBLOCK) == -1 {
                let status = -errno();
                let what = "Failed to make master descriptor non-blocking";
                warn_errno(c"pty_proc_spawn", 205, what);
                break 'configure status;
            }
            // Other jobs and providers must not get a copy of this
            // descriptor.
            if os_set_cloexec(master) == -1 {
                let status = -errno();
                let what = "Failed to set CLOEXEC on ptmx file descriptor";
                warn(c"pty_proc_spawn", 212, what);
                break 'configure status;
            }
            // Each direction gets its own copy of the master, so that
            // closing one pipe does not take the other with it.
            for (closed, pipe) in [
                ((*proc).in_0.closed, &raw mut (*proc).in_0.uv.pipe),
                ((*proc).out.s.closed, &raw mut (*proc).out.s.uv.pipe),
            ] {
                if !closed {
                    let status = open_duplicate(master, pipe);
                    if status != 0 {
                        break 'configure status;
                    }
                }
            }
        }
        0
    };

    // SAFETY: `master` and `pid` are this call's own descriptor and child.
    unsafe {
        if status != 0 {
            close(master);
            kill(pid, SIGKILL);
            waitpid(pid, ptr::null_mut(), 0);
            return status;
        }
        (*ptyproc).tty_fd = master;
        (*proc).pid = pid;
    }
    0
}

/// The path of the child's side of the pty.
///
/// # Safety
///
/// `ptyproc` points to a live, spawned [`PtyProc`]. The result points into
/// libc's static buffer and dies at the next `ptsname`.
pub unsafe fn pty_proc_tty_name(ptyproc: *mut PtyProc) -> *const c_char {
    // SAFETY: the caller's process, whose `tty_fd` is the pty master.
    unsafe { ptsname((*ptyproc).tty_fd) }
}

/// Tell the child its terminal changed size.
///
/// # Safety
///
/// `ptyproc` points to a live [`PtyProc`].
pub unsafe fn pty_proc_resize(ptyproc: *mut PtyProc, width: u16, height: u16) {
    // SAFETY: the caller's process; `TIOCSWINSZ` reads the `winsize` this
    // hands it, which is the process's own field.
    unsafe {
        (*ptyproc).winsize.ws_row = height;
        (*ptyproc).winsize.ws_col = width;
        (*ptyproc).winsize.ws_xpixel = 0;
        (*ptyproc).winsize.ws_ypixel = 0;
        ioctl((*ptyproc).tty_fd, TIOCSWINSZ, &raw mut (*ptyproc).winsize);
    }
}

/// Resume a suspended child.
///
/// The signal goes to the whole process group: some shells (fish, for one) do
/// not propagate SIGCONT to their own suspended children.
///
/// # Safety
///
/// `ptyproc` points to a live, spawned [`PtyProc`].
pub unsafe fn pty_proc_resume(ptyproc: *mut PtyProc) {
    // SAFETY: `Proc` is `PtyProc`'s first field, and `pid` is the child.
    unsafe { killpg((*ptyproc.cast::<Proc>()).pid, SIGCONT) };
}

/// Nudge the kernel into flushing the pty master's pending work.
///
/// On Linux libuv polls with epoll, which does not run the pty's workqueue;
/// an explicit `poll` does, but only when no data is immediately available.
/// So this is needed before *every* libuv poll in `flush_stream`. #37982
///
/// # Safety
///
/// `ptyproc` points to a live [`PtyProc`].
pub unsafe fn pty_proc_flush_master(ptyproc: *mut PtyProc) {
    // SAFETY: the caller's process; the descriptor is its own.
    let mut fds = pollfd {
        fd: unsafe { (*ptyproc).tty_fd },
        events: POLLIN,
        revents: 0,
    };
    // SAFETY: one live `pollfd`, and a zero timeout so this cannot block.
    while unsafe { poll(&raw mut fds, 1, 0) } < 0 && errno() == EINTR {}
}

/// Drop the master and tell the owner the process is closed.
///
/// # Safety
///
/// `ptyproc` points to a live [`PtyProc`] whose close callback is prepared
/// to run.
pub unsafe fn pty_proc_close(ptyproc: *mut PtyProc) {
    // SAFETY: the caller's process. The callback is read into a local before
    // it runs, because it is the owner's code and may touch the process.
    unsafe { pty_proc_close_master(ptyproc) };
    let proc = ptyproc.cast::<Proc>();
    let close_cb = unsafe { (*proc).internal_close_cb };
    if let Some(notify) = close_cb {
        unsafe { notify(proc) };
    }
}

/// Drop the master descriptor, which is what sends the child SIGHUP.
///
/// # Safety
///
/// `ptyproc` points to a live [`PtyProc`].
pub unsafe fn pty_proc_close_master(ptyproc: *mut PtyProc) {
    // SAFETY: the caller's process; `tty_fd` is a descriptor it owns, and
    // it is marked spent before anything else can close it twice.
    unsafe {
        if (*ptyproc).tty_fd >= 0 {
            close((*ptyproc).tty_fd);
            (*ptyproc).tty_fd = -1;
        }
    }
}

/// Stop watching for SIGCHLD when the loop shuts down.
///
/// # Safety
///
/// `uv_loop` points to a live `Loop` whose children watcher was started.
pub unsafe fn pty_proc_teardown(uv_loop: *mut Loop) {
    // SAFETY: the caller's loop and its own watcher.
    unsafe { uv_signal_stop(&raw mut (*uv_loop).children_watcher) };
}

/// The child side of the fork, which either `execvp`s or exits.
///
/// # Async-signal safety
///
/// Everything here runs between `fork` and `exec` in a process that had more
/// than one thread, so in principle it may only call async-signal-safe
/// functions: no allocation, no locks. Upstream does not honour that — both
/// `tv_dict_to_env` and the failure logging allocate — and the behaviour is
/// preserved rather than fixed, because building the environment before the
/// fork would change when the child's variables are read. Do not add
/// anything further that allocates, formats or takes a lock.
///
/// # Safety
///
/// Runs in the child half of a `forkpty`, with `ptyproc` still addressing the
/// (copied) process the parent set up.
unsafe fn init_child(ptyproc: *mut PtyProc) -> ! {
    /// The code Vim has always used when a child could not be started.
    const EXEC_FAILED: c_int = 122;

    // SAFETY: this is the whole child. `setsid`, `signal` and `_exit` touch
    // only this process; `cwd`, `argv` and `env` are the copies the parent
    // built, and `execvp` replaces the image or returns having done nothing.
    unsafe {
        // New session and process group, so the child owns its terminal.
        // #6530
        setsid();

        // Whatever the editor was ignoring or handling, the child should
        // not.
        for sig in [SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGALRM] {
            signal(sig, SIG_DFL);
        }

        let proc = ptyproc.cast::<Proc>();
        // Not `os_chdir`: that would buffer UI events for nobody.
        if !(*proc).cwd.is_null() {
            let err = uv_chdir((*proc).cwd);
            if err != 0 {
                let (cwd, why) = (c_str((*proc).cwd), c_str(uv_strerror(err)));
                logmsg!(LOGLVL_ERR, c"init_child", 318, "chdir({cwd}) failed: {why}");
                _exit(EXEC_FAILED);
            }
        }

        let prog = proc_get_exepath(proc);
        debug_assert!(!(*proc).env.is_null());
        environ = tv_dict_to_env((*proc).env);
        execvp(prog, (*proc).argv.cast::<*const c_char>());
        let (at, prog, why) = (c"init_child", c_str(prog), c_str(strerror(errno())));
        logmsg!(LOGLVL_ERR, at, 327, "execvp({prog}) failed: {why}");
        _exit(EXEC_FAILED);
    }
}

/// Give `pipe` its own copy of `fd`. Returns zero, or a negative error code.
///
/// # Safety
///
/// `fd` is a descriptor this process owns and `pipe` points to a live,
/// unopened `uv_pipe_t`.
unsafe fn open_duplicate(fd: c_int, pipe: *mut uv_pipe_t) -> c_int {
    // SAFETY: the caller's descriptor and pipe. `fd_dup` is this call's own
    // until `uv_pipe_open` takes it, and is closed on every path that does
    // not hand it over.
    unsafe {
        let fd_dup = dup(fd);
        if fd_dup < 0 {
            let status = -errno();
            let what = format!("Failed to dup descriptor {fd}");
            warn_errno(c"open_duplicate", 398, &what);
            return status;
        }

        let status = if os_set_cloexec(fd_dup) == -1 {
            let status = -errno();
            let what = "Failed to set CLOEXEC on duplicate fd";
            warn(c"open_duplicate", 404, what);
            status
        } else {
            let status = uv_pipe_open(pipe, fd_dup);
            if status == 0 {
                return 0;
            }
            let why = c_str(uv_strerror(status));
            let at = c"open_duplicate";
            logmsg!(
                LOGLVL_ERR,
                at,
                411,
                "Failed to set pipe to descriptor {fd_dup}: {why}"
            );
            status
        };
        close(fd_dup);
        status
    }
}

/// SIGCHLD: reap whichever of the loop's children have news.
///
/// Every child is polled, not just the pty ones — libuv reaps the children it
/// spawned itself, so those simply report nothing here.
///
/// # Safety
///
/// libuv's, from the loop's children watcher: `handle` is the live watcher
/// whose loop's `data` is the editor's [`Loop`].
unsafe extern "C" fn chld_handler(handle: *mut uv_signal_t, _signum: c_int) {
    // SAFETY: the watcher libuv is calling back, and the loop it belongs to.
    let uv_loop = unsafe { (*(*handle).loop_0).data }.cast::<Loop>();
    let mut i = 0;
    // The list is re-read every step: a callback below runs the editor's own
    // code, which may spawn or reap a child and reallocate it.
    while i < unsafe { (*loop_children(uv_loop)).len() } {
        let proc = unsafe { &*loop_children(uv_loop) }[i];
        i += 1;

        let mut stat: c_int = 0;
        let mut pid;
        loop {
            // SAFETY: the loop's own child, and a local status word.
            pid = unsafe { waitpid((*proc).pid, &raw mut stat, WNOHANG | WUNTRACED | WCONTINUED) };
            if pid >= 0 || errno() != EINTR {
                break;
            }
        }
        if pid <= 0 {
            continue;
        }

        // Every callback is the owner's code and may touch `proc`, so what
        // it needs is read out first and it is called with nothing borrowed.
        match wait_status::decode(stat) {
            state @ (ChildState::Stopped | ChildState::Continued) => {
                // SAFETY: a live child of this loop.
                let (notify, data) = unsafe { ((*proc).state_cb, (*proc).data) };
                let notify = notify.expect("a pty child reports its state");
                unsafe { notify(proc, state == ChildState::Stopped, data) };
            }
            ChildState::Exited { status } => {
                // SAFETY: as above.
                let notify = unsafe {
                    if let Some(status) = status {
                        (*proc).status = status;
                    }
                    (*proc).internal_exit_cb
                };
                let notify = notify.expect("a spawned child has an exit callback");
                unsafe { notify(proc) };
            }
        }
    }
}
