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

mod termios;
mod wait_status;

use crate::src::nvim::eval::typval::tv_dict_to_env;
use crate::src::nvim::event::libuv::{
    uv_chdir, uv_disable_stdio_inheritance, uv_pipe_open, uv_signal_start, uv_signal_stop,
    uv_strerror,
};
use crate::src::nvim::event::r#loop::loop_children;
use crate::src::nvim::event::proc::{kProcTypePty, proc_get_exepath, proc_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::os::fs::os_set_cloexec;
use crate::src::nvim::os::libc::{
    __errno_location, _exit, cfsetispeed, cfsetospeed, close, dup, environ, execvp, fcntl, forkpty,
    ioctl, kill, killpg, ptsname, setsid, strerror, waitpid,
};
use crate::src::nvim::os::signal::{
    SIGALRM, SIGCHLD, SIGCONT, SIGHUP, SIGINT, SIGKILL, SIGQUIT, SIGTERM,
};
use crate::src::nvim::types::{Loop, Proc, PtyProc, speed_t, uv_file, uv_pipe_t, uv_signal_t};
use core::ffi::{c_char, c_int, c_short, c_ulong, c_void};
use core::ptr;
use wait_status::ChildState;

unsafe extern "C" {
    fn signal(sig: c_int, handler: SigHandler) -> SigHandler;
    fn poll(fds: *mut pollfd, nfds: c_ulong, timeout: c_int) -> c_int;
}

type SigHandler = Option<unsafe extern "C" fn(c_int)>;

/// Restore a signal to its default disposition.
const SIG_DFL: SigHandler = None;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: c_short,
    pub revents: c_short,
}

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

const LOGLVL_ERR: c_int = 4;

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

/// Fork the child onto a new pseudo-terminal. Returns zero, or a negative
/// error code.
pub unsafe fn pty_proc_spawn(ptyproc: *mut PtyProc) -> c_int {
    // Built at first use and reused, as upstream did.
    static TERMIOS_DEFAULT: GlobalCell<crate::src::nvim::types::termios> =
        GlobalCell::new(crate::src::nvim::types::termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; 32],
            c_ispeed: 0,
            c_ospeed: 0,
        });
    if (*TERMIOS_DEFAULT.ptr()).c_cflag == 0 {
        *TERMIOS_DEFAULT.ptr() = termios::default_termios();
        // Upstream passes the literal 38400 where these want a `B38400`
        // code, so both calls fail with EINVAL and leave the speeds — and
        // the baud bits of `c_cflag` — alone. Kept: passing `B38400`
        // instead would change what `tcsetattr` is given.
        cfsetispeed(TERMIOS_DEFAULT.ptr(), 38400 as speed_t);
        cfsetospeed(TERMIOS_DEFAULT.ptr(), 38400 as speed_t);
    }

    let proc = ptyproc as *mut Proc;
    // stderr is folded into the pty; there is one stream in each direction.
    assert!((*proc).err.s.closed);

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

    let mut master: c_int = 0;
    let pid = forkpty(
        &raw mut master,
        ptr::null_mut(),
        TERMIOS_DEFAULT.ptr(),
        &raw mut (*ptyproc).winsize,
    );
    if pid < 0 {
        let status = -errno();
        logmsg(
            LOGLVL_ERR,
            ptr::null(),
            c"pty_proc_spawn".as_ptr(),
            190,
            true,
            c"forkpty failed: %s".as_ptr(),
            strerror(errno()),
        );
        return status;
    }
    if pid == 0 {
        init_child(ptyproc); // never returns
    }

    let status = 'configure: {
        // The master must be non-blocking: libuv polls it.
        let flags = fcntl(master, F_GETFL);
        if flags == -1 {
            // Captured before the logging, which does I/O of its own.
            let status = -errno();
            logmsg(
                LOGLVL_ERR,
                ptr::null(),
                c"pty_proc_spawn".as_ptr(),
                200,
                true,
                c"Failed to get master descriptor status flags: %s".as_ptr(),
                strerror(errno()),
            );
            break 'configure status;
        }
        if fcntl(master, F_SETFL, flags | O_NONBLOCK) == -1 {
            let status = -errno();
            logmsg(
                LOGLVL_ERR,
                ptr::null(),
                c"pty_proc_spawn".as_ptr(),
                205,
                true,
                c"Failed to make master descriptor non-blocking: %s".as_ptr(),
                strerror(errno()),
            );
            break 'configure status;
        }
        // Other jobs and providers must not get a copy of this descriptor.
        if os_set_cloexec(master) == -1 {
            let status = -errno();
            logmsg(
                LOGLVL_ERR,
                ptr::null(),
                c"pty_proc_spawn".as_ptr(),
                212,
                true,
                c"Failed to set CLOEXEC on ptmx file descriptor".as_ptr(),
            );
            break 'configure status;
        }
        // Each direction gets its own copy of the master, so that closing
        // one pipe does not take the other with it.
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
        0
    };

    if status != 0 {
        close(master);
        kill(pid, SIGKILL);
        waitpid(pid, ptr::null_mut(), 0);
        return status;
    }

    (*ptyproc).tty_fd = master;
    (*proc).pid = pid;
    0
}

/// The path of the child's side of the pty.
pub unsafe fn pty_proc_tty_name(ptyproc: *mut PtyProc) -> *const c_char {
    ptsname((*ptyproc).tty_fd)
}

pub unsafe fn pty_proc_resize(ptyproc: *mut PtyProc, width: u16, height: u16) {
    (*ptyproc).winsize.ws_row = height;
    (*ptyproc).winsize.ws_col = width;
    (*ptyproc).winsize.ws_xpixel = 0;
    (*ptyproc).winsize.ws_ypixel = 0;
    ioctl((*ptyproc).tty_fd, TIOCSWINSZ, &raw mut (*ptyproc).winsize);
}

/// Resume a suspended child.
///
/// The signal goes to the whole process group: some shells (fish, for one) do
/// not propagate SIGCONT to their own suspended children.
pub unsafe fn pty_proc_resume(ptyproc: *mut PtyProc) {
    killpg((*(ptyproc as *mut Proc)).pid, SIGCONT);
}

/// Nudge the kernel into flushing the pty master's pending work.
///
/// On Linux libuv polls with epoll, which does not run the pty's workqueue;
/// an explicit `poll` does, but only when no data is immediately available.
/// So this is needed before *every* libuv poll in `flush_stream`. #37982
pub unsafe fn pty_proc_flush_master(ptyproc: *mut PtyProc) {
    let mut fds = pollfd {
        fd: (*ptyproc).tty_fd,
        events: POLLIN,
        revents: 0,
    };
    while poll(&raw mut fds, 1, 0) < 0 && errno() == EINTR {}
}

pub unsafe fn pty_proc_close(ptyproc: *mut PtyProc) {
    pty_proc_close_master(ptyproc);
    let proc = ptyproc as *mut Proc;
    if let Some(notify) = (*proc).internal_close_cb {
        notify(proc);
    }
}

/// Drop the master descriptor, which is what sends the child SIGHUP.
pub unsafe fn pty_proc_close_master(ptyproc: *mut PtyProc) {
    if (*ptyproc).tty_fd >= 0 {
        close((*ptyproc).tty_fd);
        (*ptyproc).tty_fd = -1;
    }
}

pub unsafe fn pty_proc_teardown(uv_loop: *mut Loop) {
    uv_signal_stop(&raw mut (*uv_loop).children_watcher);
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
unsafe fn init_child(ptyproc: *mut PtyProc) -> ! {
    /// The code Vim has always used when a child could not be started.
    const EXEC_FAILED: c_int = 122;

    // New session and process group, so the child owns its terminal. #6530
    setsid();

    // Whatever the editor was ignoring or handling, the child should not.
    for sig in [SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGALRM] {
        signal(sig, SIG_DFL);
    }

    let proc = ptyproc as *mut Proc;
    // Not `os_chdir`: that would buffer UI events for nobody.
    if !(*proc).cwd.is_null() {
        let err = uv_chdir((*proc).cwd);
        if err != 0 {
            logmsg(
                LOGLVL_ERR,
                ptr::null(),
                c"init_child".as_ptr(),
                318,
                true,
                c"chdir(%s) failed: %s".as_ptr(),
                (*proc).cwd,
                uv_strerror(err),
            );
            _exit(EXEC_FAILED);
        }
    }

    let prog = proc_get_exepath(proc);
    assert!(!(*proc).env.is_null());
    environ = tv_dict_to_env((*proc).env);
    execvp(prog, (*proc).argv as *const *mut c_char);
    logmsg(
        LOGLVL_ERR,
        ptr::null(),
        c"init_child".as_ptr(),
        327,
        true,
        c"execvp(%s) failed: %s".as_ptr(),
        prog,
        strerror(errno()),
    );
    _exit(EXEC_FAILED);
}

/// Give `pipe` its own copy of `fd`. Returns zero, or a negative error code.
unsafe fn open_duplicate(fd: c_int, pipe: *mut uv_pipe_t) -> c_int {
    let fd_dup = dup(fd);
    if fd_dup < 0 {
        let status = -errno();
        logmsg(
            LOGLVL_ERR,
            ptr::null(),
            c"open_duplicate".as_ptr(),
            398,
            true,
            c"Failed to dup descriptor %d: %s".as_ptr(),
            fd,
            strerror(errno()),
        );
        return status;
    }

    let status = if os_set_cloexec(fd_dup) == -1 {
        let status = -errno();
        logmsg(
            LOGLVL_ERR,
            ptr::null(),
            c"open_duplicate".as_ptr(),
            404,
            true,
            c"Failed to set CLOEXEC on duplicate fd".as_ptr(),
        );
        status
    } else {
        let status = uv_pipe_open(pipe, fd_dup as uv_file);
        if status == 0 {
            return 0;
        }
        logmsg(
            LOGLVL_ERR,
            ptr::null(),
            c"open_duplicate".as_ptr(),
            411,
            true,
            c"Failed to set pipe to descriptor %d: %s".as_ptr(),
            fd_dup,
            uv_strerror(status),
        );
        status
    };
    close(fd_dup);
    status
}

/// SIGCHLD: reap whichever of the loop's children have news.
///
/// Every child is polled, not just the pty ones — libuv reaps the children it
/// spawned itself, so those simply report nothing here.
unsafe extern "C" fn chld_handler(handle: *mut uv_signal_t, _signum: c_int) {
    let uv_loop = (*(*handle).loop_0).data as *mut Loop;
    let mut i = 0;
    while i < (*loop_children(uv_loop)).len() {
        let proc = (&*loop_children(uv_loop))[i];
        i += 1;

        let mut stat: c_int = 0;
        let mut pid;
        loop {
            pid = waitpid((*proc).pid, &raw mut stat, WNOHANG | WUNTRACED | WCONTINUED);
            if pid >= 0 || errno() != EINTR {
                break;
            }
        }
        if pid <= 0 {
            continue;
        }

        match wait_status::decode(stat) {
            ChildState::Stopped => {
                let notify = (*proc).state_cb.expect("a pty child reports its state");
                notify(proc, true, (*proc).data);
            }
            ChildState::Continued => {
                let notify = (*proc).state_cb.expect("a pty child reports its state");
                notify(proc, false, (*proc).data);
            }
            ChildState::Exited { status } => {
                if let Some(status) = status {
                    (*proc).status = status;
                }
                let notify = (*proc)
                    .internal_exit_cb
                    .expect("a spawned child has an exit callback");
                notify(proc);
            }
        }
    }
}
