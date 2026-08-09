//! Children started through libuv's process API.
//!
//! [`LibuvProc`] is a [`Proc`] with a `uv_process_t` and the options libuv
//! needs to spawn it bolted on. `Proc` is the first field, so `event/proc.rs`
//! casts freely between the two; libuv is handed the address of the `Proc`
//! (not of the `LibuvProc`) in the handle's `data` field, and the callbacks
//! here cast it back.

use crate::src::nvim::eval::typval::tv_dict_to_env;
use crate::src::nvim::event::libuv::{uv_close, uv_pipe, uv_pipe_open, uv_spawn, uv_strerror};
use crate::src::nvim::event::proc::{kProcTypeUv, proc_get_exepath, proc_init};
use crate::src::nvim::log::{LOGLVL_INF, logmsg_c};
use crate::src::nvim::main::ui_client_forward_stdin;
use crate::src::nvim::os::env::os_free_fullenv;
use crate::src::nvim::os::libc::close;
use crate::src::nvim::types::libc::STDERR_FILENO;
use crate::src::nvim::types::{
    LibuvProc, Loop, Proc, uv_file, uv_handle_t, uv_pipe_t, uv_process_t, uv_stdio_container_t,
    uv_stdio_flags,
};
use core::ffi::{c_int, c_void};
use core::ptr;

/// The slot is unused; the child inherits nothing there.
const UV_IGNORE: uv_stdio_flags = 0;
/// The slot names a descriptor for the child to inherit.
const UV_INHERIT_FD: uv_stdio_flags = 2;
/// Open this end of the pipe non-blocking.
const UV_NONBLOCK_PIPE: c_int = 64;

/// No console window for the child. Windows-only in libuv, harmless here.
const UV_PROCESS_WINDOWS_HIDE: u32 = 16;
/// `setsid()` the child, which on unix-likes we always want. #8107
const UV_PROCESS_DETACHED: u32 = 8;

/// A fresh, unspawned libuv child.
pub fn libuv_proc_init(uv_loop: *mut Loop, data: *mut c_void) -> LibuvProc {
    // SAFETY: every field is a pointer, an integer or an `Option<fn>`, for
    // all of which the all-zero pattern is the "unset" value; the spawn path
    // below fills in the ones it needs.
    let mut rv: LibuvProc = unsafe { core::mem::zeroed() };
    rv.proc = proc_init(uv_loop, kProcTypeUv, data);
    rv
}

/// Wire stdio slot `idx` to a fresh pipe, and return the descriptor the
/// parent must close once the child has been spawned.
///
/// `child_readable` is true for the child's stdin — the child reads the read
/// end and the parent keeps the write end — and false for its output
/// streams. The parent's end is always non-blocking; the child's is not.
///
/// A `uv_pipe` pair is used rather than libuv's own `UV_CREATE_PIPE`, which
/// as of libuv 1.51 is a `socketpair` and so breaks `/proc/<pid>/fd/0`.
unsafe fn open_stdio_pipe(
    uvproc: *mut LibuvProc,
    idx: usize,
    parent_pipe: *mut uv_pipe_t,
    child_readable: bool,
) -> c_int {
    let (read_flags, write_flags) = if child_readable {
        (0, UV_NONBLOCK_PIPE)
    } else {
        (UV_NONBLOCK_PIPE, 0)
    };
    let mut pipe_pair: [uv_file; 2] = [0; 2];
    uv_pipe(pipe_pair.as_mut_ptr(), read_flags, write_flags);

    let [read_end, write_end] = pipe_pair;
    let (child_fd, parent_fd) = if child_readable {
        (read_end, write_end)
    } else {
        (write_end, read_end)
    };
    (*uvproc).uvstdio[idx].flags = UV_INHERIT_FD;
    (*uvproc).uvstdio[idx].data.fd = child_fd;
    uv_pipe_open(parent_pipe, parent_fd);
    child_fd
}

/// Start the child. Returns zero, or a negative error code.
pub unsafe fn libuv_proc_spawn(uvproc: *mut LibuvProc) -> c_int {
    let proc = uvproc as *mut Proc;
    let opts = &raw mut (*uvproc).uvopts;
    (*opts).file = proc_get_exepath(proc);
    (*opts).args = (*proc).argv;
    (*opts).flags = UV_PROCESS_WINDOWS_HIDE | UV_PROCESS_DETACHED;
    (*opts).exit_cb = Some(exit_cb);
    (*opts).cwd = (*proc).cwd;
    (*opts).stdio = (&raw mut (*uvproc).uvstdio) as *mut uv_stdio_container_t;
    (*opts).stdio_count = 3;
    for slot in &mut (&mut (*uvproc).uvstdio)[..3] {
        slot.flags = UV_IGNORE;
    }

    if ui_client_forward_stdin.get() {
        // A UI client reads the editor's own stdin, on the descriptor the
        // remote side knows as UI_CLIENT_STDIN_FD.
        (*opts).stdio_count = 4;
        (*uvproc).uvstdio[3].data.fd = 0;
        (*uvproc).uvstdio[3].flags = UV_INHERIT_FD;
    }
    (*uvproc).uv.data = proc as *mut c_void;

    (*opts).env = if (*proc).env.is_null() {
        ptr::null_mut()
    } else {
        tv_dict_to_env((*proc).env)
    };

    // The parent's copies of the descriptors the child inherits. They are
    // dropped after the spawn, whether or not it succeeded.
    let mut to_close = [-1; 3];
    if !(*proc).in_0.closed {
        to_close[0] = open_stdio_pipe(uvproc, 0, &raw mut (*proc).in_0.uv.pipe, true);
    }
    if !(*proc).out.s.closed {
        to_close[1] = open_stdio_pipe(uvproc, 1, &raw mut (*proc).out.s.uv.pipe, false);
    }
    if !(*proc).err.s.closed {
        to_close[2] = open_stdio_pipe(uvproc, 2, &raw mut (*proc).err.s.uv.pipe, false);
    } else if (*proc).fwd_err {
        // Not read by us, so let the child write straight to our stderr.
        (*uvproc).uvstdio[2].flags = UV_INHERIT_FD;
        (*uvproc).uvstdio[2].data.fd = STDERR_FILENO;
    }

    let status = uv_spawn(&raw mut (*(*proc).loop_0).uv, &raw mut (*uvproc).uv, opts);
    if status != 0 {
        logmsg_c!(
            LOGLVL_INF,
            ptr::null(),
            c"libuv_proc_spawn".as_ptr(),
            141,
            true,
            c"uv_spawn(%s) failed: %s".as_ptr(),
            (*opts).file,
            uv_strerror(status),
        );
        // Nothing will reach `close_cb` to free it.
        if !(*opts).env.is_null() {
            os_free_fullenv((*opts).env);
        }
    } else {
        (*proc).pid = (*uvproc).uv.pid;
    }

    for fd in to_close {
        if fd > -1 {
            close(fd);
        }
    }
    status
}

pub unsafe fn libuv_proc_close(uvproc: *mut LibuvProc) {
    uv_close(&raw mut (*uvproc).uv as *mut uv_handle_t, Some(close_cb));
}

/// The process handle finished closing: report it, then release the
/// environment libuv held a pointer into.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    let proc = (*handle).data as *mut Proc;
    if let Some(notify) = (*proc).internal_close_cb {
        notify(proc);
    }
    let uvproc = proc as *mut LibuvProc;
    if !(*uvproc).uvopts.env.is_null() {
        os_free_fullenv((*uvproc).uvopts.env);
    }
}

/// The child exited. A signalled death is reported the way a shell does,
/// as 128 plus the signal number.
unsafe extern "C" fn exit_cb(handle: *mut uv_process_t, status: i64, term_signal: c_int) {
    let proc = (*handle).data as *mut Proc;
    (*proc).status = exit_status(status, term_signal);
    (*proc)
        .internal_exit_cb
        .expect("a spawned child has an exit callback")(proc);
}

/// How libuv's `(status, term_signal)` pair becomes the status the editor
/// reports to `jobwait()` and friends.
fn exit_status(status: i64, term_signal: c_int) -> c_int {
    if term_signal != 0 {
        128 + term_signal
    } else {
        status as c_int
    }
}

#[cfg(test)]
mod tests {
    use super::exit_status;

    #[test]
    fn a_normal_exit_reports_its_own_code() {
        assert_eq!(exit_status(0, 0), 0);
        assert_eq!(exit_status(42, 0), 42);
        // libuv widens the code to 64 bits; the editor's status is an int.
        assert_eq!(exit_status(255, 0), 255);
    }

    #[test]
    fn a_signalled_exit_reports_128_plus_the_signal() {
        assert_eq!(exit_status(0, 9), 137);
        assert_eq!(exit_status(0, 15), 143);
    }

    #[test]
    fn a_signal_wins_over_the_exit_code() {
        assert_eq!(exit_status(3, 9), 137);
    }
}
