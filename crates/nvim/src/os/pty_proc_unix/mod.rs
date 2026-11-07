//! Children started on a pseudo-terminal.
//!
//! A [`PtyProc`] is a [`Proc`](crate::event::objects::Proc) with the pty master descriptor and the window
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
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod termios;
mod wait_status;

use crate::eval::typval::dict_to_env;
use crate::event::libuv::{uv_chdir, uv_disable_stdio_inheritance, uv_pipe_open};
use crate::event::r#loop::{loop_children, start_children_watcher};
use crate::event::objects::Loop;
use crate::event::objects::Proc;
use crate::event::proc::{kProcTypePty, proc_get_exepath, proc_init};
use crate::event::uv::{uv_pipe_t, uv_signal_t};
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::message_fmt::{msg_cstr, msg_cstr_opt};
use crate::os::cshim::environ;
use crate::os::fs::{errno_text, os_strerror, set_cloexec};
use crate::os::libc::{speed_t, uint16_t, winsize};
use crate::os::signal::{SIGALRM, SIGCHLD, SIGCONT, SIGHUP, SIGINT, SIGKILL, SIGQUIT, SIGTERM};
use crate::os::uv_error::uv_code;
use ::libc::{
    __errno_location, _exit, SIG_DFL, cfsetispeed, cfsetospeed, close, execvp, fcntl, forkpty,
    ioctl, kill, killpg, poll, pollfd, ptsname, setsid, signal, waitpid,
};
use core::ffi::{CStr, c_char, c_int, c_short, c_ulong, c_void};
use core::ptr;
use std::ffi::CString;
use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd, FromRawFd as _, IntoRawFd as _, OwnedFd};
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

/// `strerror(errno)`, for a log line.
fn why() -> CString {
    errno_text(errno())
}

/// One `LOGLVL_ERR` line about a failed syscall, with `strerror(errno)` for
/// its tail.
///
/// A helper rather than a `logmsg!` per site because most of these sit
/// inside a body-wide `unsafe` block, where the seven lines rustfmt wraps
/// the call over are seven *unchecked* lines. One call, one line.
fn warn_errno(who: &'static CStr, line: u32, what: &str) {
    logmsg!(LOGLVL_ERR, who, line, "{what}: {}", msg_cstr(&why()));
}

/// [`warn_errno`] for a failure that carries no errno.
fn warn(who: &'static CStr, line: u32, what: &str) {
    logmsg!(LOGLVL_ERR, who, line, "{what}");
}

/// Fork the child onto a new pseudo-terminal. Returns zero, or a negative
/// error code.
///
/// `ptyproc` is unspawned, its `Proc` half initialised (`proc_init`), and
/// it stays where it is while the child lives: its two pipes are handed to
/// libuv here.
pub fn pty_proc_spawn(ptyproc: &mut PtyProc) -> c_int {
    // Built at first use and reused, as upstream did.
    static TERMIOS_DEFAULT: GlobalCell<crate::os::libc::termios> =
        GlobalCell::new(crate::os::libc::termios {
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

    // stderr is folded into the pty; there is one stream in each direction.
    debug_assert!(ptyproc.proc.err.s.closed);
    start_children_watcher(chld_handler);
    ptyproc.winsize.ws_row = ptyproc.height;
    ptyproc.winsize.ws_col = ptyproc.width;
    ptyproc.winsize.ws_xpixel = 0;
    ptyproc.winsize.ws_ypixel = 0;
    // SAFETY: marks this process's own descriptors close-on-exec.
    unsafe { uv_disable_stdio_inheritance() };

    let mut master: c_int = 0;
    // SAFETY: `forkpty` writes the master descriptor into `master` and reads
    // the two structs; both are live. In the child it returns 0 having
    // replaced this process's stdio with the slave.
    let pid = unsafe {
        forkpty(
            &raw mut master,
            ptr::null_mut(),
            TERMIOS_DEFAULT.ptr(),
            &raw mut ptyproc.winsize,
        )
    };
    if pid < 0 {
        let status = -errno();
        warn_errno(c"pty_proc_spawn", 190, "forkpty failed");
        return status;
    }
    if pid == 0 {
        init_child(ptyproc);
    }

    // SAFETY: the descriptor `forkpty` just handed back, which this process
    // owns and nothing else holds.
    let master = unsafe { OwnedFd::from_raw_fd(master) };
    let status = configure_master(ptyproc, master.as_fd());
    if status != 0 {
        drop(master);
        // SAFETY: `pid` is this call's own child, and the status word is
        // not asked for.
        unsafe {
            kill(pid, SIGKILL);
            waitpid(pid, ptr::null_mut(), 0);
        }
        return status;
    }
    // The master is kept as the raw descriptor the C layout holds, closed by
    // `pty_proc_close_master`.
    ptyproc.tty_fd = master.into_raw_fd();
    ptyproc.proc.pid = pid;
    0
}

/// Make the new master non-blocking and close-on-exec, and give each
/// direction the child talks over a copy of it. Zero, or a negative error
/// code.
fn configure_master(ptyproc: &mut PtyProc, master: BorrowedFd<'_>) -> c_int {
    // The master must be non-blocking: libuv polls it.
    // SAFETY: `master` is open for as long as it is borrowed here.
    let flags = unsafe { fcntl(master.as_raw_fd(), F_GETFL) };
    if flags == -1 {
        // Captured before the logging, which does I/O of its own.
        let status = -errno();
        let what = "Failed to get master descriptor status flags";
        warn_errno(c"pty_proc_spawn", 200, what);
        return status;
    }
    // SAFETY: as above.
    if unsafe { fcntl(master.as_raw_fd(), F_SETFL, flags | O_NONBLOCK) } == -1 {
        let status = -errno();
        let what = "Failed to make master descriptor non-blocking";
        warn_errno(c"pty_proc_spawn", 205, what);
        return status;
    }
    // Other jobs and providers must not get a copy of this descriptor.
    if let Err(status) = set_cloexec(master) {
        let what = "Failed to set CLOEXEC on ptmx file descriptor";
        warn(c"pty_proc_spawn", 212, what);
        return status;
    }
    // Each direction gets its own copy of the master, so that closing one
    // pipe does not take the other with it.
    let proc = &mut ptyproc.proc;
    for (closed, pipe) in [
        (proc.in_0.closed, &raw mut proc.in_0.uv.pipe),
        (proc.out.s.closed, &raw mut proc.out.s.uv.pipe),
    ] {
        if !closed {
            // SAFETY: the child's own unopened pipe handle.
            let status = unsafe { open_duplicate(master, pipe) };
            if status != 0 {
                return status;
            }
        }
    }
    0
}

/// The path of the child's side of the pty, copied out of libc's buffer;
/// `None` once the master is closed.
pub fn pty_proc_tty_name(ptyproc: &PtyProc) -> Option<CString> {
    // SAFETY: `ptsname` answers null or its own NUL-terminated buffer,
    // copied before anything else can call it.
    let name = unsafe { ptsname(ptyproc.tty_fd) };
    // SAFETY: as above.
    (!name.is_null()).then(|| unsafe { CStr::from_ptr(name) }.to_owned())
}

/// Tell the child its terminal changed size.
pub fn pty_proc_resize(ptyproc: &mut PtyProc, width: u16, height: u16) {
    ptyproc.winsize.ws_row = height;
    ptyproc.winsize.ws_col = width;
    ptyproc.winsize.ws_xpixel = 0;
    ptyproc.winsize.ws_ypixel = 0;
    // SAFETY: `TIOCSWINSZ` reads the `winsize` this hands it, the process's
    // own field.
    unsafe { ioctl(ptyproc.tty_fd, TIOCSWINSZ, &raw mut ptyproc.winsize) };
}

/// Resume a suspended child.
///
/// The signal goes to the whole process group: some shells (fish, for one) do
/// not propagate SIGCONT to their own suspended children.
pub fn pty_proc_resume(ptyproc: &PtyProc) {
    // SAFETY: signals the child's own process group.
    unsafe { killpg(ptyproc.proc.pid, SIGCONT) };
}

/// Nudge the kernel into flushing the pty master's pending work.
///
/// On Linux libuv polls with epoll, which does not run the pty's workqueue;
/// an explicit `poll` does, but only when no data is immediately available.
/// So this is needed before *every* libuv poll in `flush_stream`. #37982
pub fn pty_proc_flush_master(ptyproc: &PtyProc) {
    let mut fds = pollfd {
        fd: ptyproc.tty_fd,
        events: POLLIN,
        revents: 0,
    };
    // SAFETY: one live `pollfd`, and a zero timeout so this cannot block.
    while unsafe { poll(&raw mut fds, 1, 0) } < 0 && errno() == EINTR {}
}

/// Drop the master descriptor, which is what sends the child SIGHUP.
pub fn pty_proc_close_master(ptyproc: &mut PtyProc) {
    if ptyproc.tty_fd >= 0 {
        // SAFETY: a descriptor the process owns, marked spent below before
        // anything else can close it twice.
        unsafe { close(ptyproc.tty_fd) };
        ptyproc.tty_fd = -1;
    }
}

/// The child side of the fork, which either `execvp`s or exits.
///
/// # Async-signal safety
///
/// Everything here runs between `fork` and `exec` in a process that had more
/// than one thread, so in principle it may only call async-signal-safe
/// functions: no allocation, no locks. Upstream does not honour that — both
/// `dict_to_env` and the failure logging allocate — and the behaviour is
/// preserved rather than fixed, because building the environment before the
/// fork would change when the child's variables are read. Do not add
/// anything further that allocates, formats or takes a lock.
///
/// Runs in the child half of a `forkpty`, on the (copied) process the
/// parent set up.
fn init_child(ptyproc: &PtyProc) -> ! {
    /// The code Vim has always used when a child could not be started.
    const EXEC_FAILED: c_int = 122;

    // New session and process group, so the child owns its terminal. #6530
    // SAFETY: this process's own session.
    unsafe { setsid() };
    // Whatever the editor was ignoring or handling, the child should not.
    for sig in [SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGALRM] {
        // SAFETY: the default disposition, in this process.
        unsafe { signal(sig, SIG_DFL) };
    }

    let proc = &ptyproc.proc;
    // Not `os_chdir`: that would buffer UI events for nobody.
    if !proc.cwd.is_null() {
        // SAFETY: the parent's NUL-terminated copy of the directory.
        let cwd = unsafe { CStr::from_ptr(proc.cwd) };
        // SAFETY: as above.
        let err = unsafe { uv_chdir(cwd.as_ptr()) };
        if err != 0 {
            let (cwd, why) = (msg_cstr(cwd), msg_cstr(os_strerror(err)));
            logmsg!(LOGLVL_ERR, c"init_child", 318, "chdir({cwd}) failed: {why}");
            // SAFETY: ends this process without running the parent's exit
            // handlers.
            unsafe { _exit(EXEC_FAILED) };
        }
    }

    // SAFETY: the parent's copy of the child, whose argv it filled in.
    let prog = unsafe { proc_get_exepath(ptr::from_ref(proc).cast_mut()) };
    debug_assert!(!proc.env.is_null());
    // SAFETY: the parent's dictionary of variables; `environ` is this
    // process's own, about to be replaced by the exec.
    unsafe { environ = dict_to_env(&*proc.env) };
    // SAFETY: the program and the NULL-terminated argv the parent built;
    // `execvp` replaces the image or returns having done nothing.
    unsafe { execvp(prog, proc.argv.cast::<*const c_char>()) };
    let (at, why) = (c"init_child", why());
    // SAFETY: as above, the program's NUL-terminated name.
    let prog = msg_cstr_opt((!prog.is_null()).then(|| unsafe { CStr::from_ptr(prog) }));
    let why = msg_cstr(&why);
    logmsg!(LOGLVL_ERR, at, 327, "execvp({prog}) failed: {why}");
    // SAFETY: as above.
    unsafe { _exit(EXEC_FAILED) }
}

/// Give `pipe` its own copy of `fd`. Returns zero, or a negative error code.
///
/// The copy is made close-on-exec in the same call (`F_DUPFD_CLOEXEC`),
/// which upstream did as a `dup` and an `os_set_cloexec` with a log line for
/// each failure.
///
/// # Safety
///
/// `pipe` points to a live, unopened `uv_pipe_t`.
unsafe fn open_duplicate(fd: BorrowedFd<'_>, pipe: *mut uv_pipe_t) -> c_int {
    let fd_dup = match fd.try_clone_to_owned() {
        Ok(fd_dup) => fd_dup,
        Err(error) => {
            let what = format!("Failed to dup descriptor {}", fd.as_raw_fd());
            warn_errno(c"open_duplicate", 398, &what);
            return uv_code(&error);
        }
    };
    let raw = fd_dup.as_raw_fd();
    // SAFETY: the caller's pipe; libuv takes `raw` over only on success.
    let status = unsafe { uv_pipe_open(pipe, raw) };
    if status == 0 {
        // The pipe owns the descriptor now.
        let _ = fd_dup.into_raw_fd();
        return 0;
    }
    let why = os_strerror(status);
    let why = msg_cstr(why);
    logmsg!(
        LOGLVL_ERR,
        c"open_duplicate",
        411,
        "Failed to set pipe to descriptor {raw}: {why}"
    );
    status
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

#[repr(C)]
pub struct PtyProc {
    pub proc: Proc,
    pub width: uint16_t,
    pub height: uint16_t,
    pub winsize: winsize,
    pub tty_fd: ::core::ffi::c_int,
}
