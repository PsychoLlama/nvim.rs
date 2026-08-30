//! Children started through libuv's process API.
//!
//! [`LibuvProc`] is a [`Proc`] with a `uv_process_t` and the options libuv
//! needs to spawn it bolted on. `Proc` is the first field, so `event/proc.rs`
//! casts freely between the two; libuv is handed the address of the `Proc`
//! (not of the `LibuvProc`) in the handle's `data` field, and the callbacks
//! here cast it back.
//!
//! Every one of those casts goes through [`Uv`], a wrapper whose construction
//! is the unsafe step: past it, `uvproc.uvopts.file` and `uvproc.proc.pid` are
//! ordinary Rust.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::tv_dict_to_env;
use crate::event::libuv::{uv_close, uv_pipe, uv_pipe_open, uv_spawn, uv_strerror};
use crate::event::proc::{kProcTypeUv, proc_get_exepath, proc_init};
use crate::log::{LOGLVL_INF, logmsg};
use crate::main::ui_client_forward_stdin;
use crate::message_fmt::c_str;
use crate::narrow::number_as_int;
use crate::os::env::os_free_fullenv;
use crate::types::libc::STDERR_FILENO;
use crate::types::{
    LibuvProc, Loop, Proc, uv_file, uv_handle_t, uv_loop_t, uv_pipe_t, uv_process_options_t,
    uv_process_t, uv_stdio_container_t, uv_stdio_flags,
};
use ::libc::close;
use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};
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

/// A libuv child, plus the promise that the pointer behind it stays live for
/// as long as the handle does.
///
/// The same address is a `*mut Proc` and a `*mut LibuvProc`, so one of these
/// is built from whichever of the two libuv or the process layer handed back.
#[derive(Copy, Clone)]
struct Uv(*mut LibuvProc);

impl Uv {
    /// # Safety
    /// `uvproc` is non-null and points at a live `LibuvProc` — equivalently,
    /// at the `Proc` that is its first field — for the whole life of the
    /// handle and of everything derived from it.
    unsafe fn new(uvproc: *mut LibuvProc) -> Self {
        debug_assert!(!uvproc.is_null());
        Uv(uvproc)
    }

    /// The child's kind-independent half, which is its first field.
    fn as_proc(self) -> *mut Proc {
        self.0.cast()
    }

    /// The libuv process handle, which is what `uv_spawn` fills in.
    fn handle(self) -> *mut uv_process_t {
        // SAFETY: a field of the live child.
        unsafe { &raw mut (*self.0).uv }
    }

    /// The spawn options, which libuv reads while `uv_spawn` runs.
    fn opts(self) -> *mut uv_process_options_t {
        // SAFETY: a field of the live child.
        unsafe { &raw mut (*self.0).uvopts }
    }

    /// The four stdio slots, as the pointer `uv_process_options_t` wants.
    fn stdio(self) -> *mut uv_stdio_container_t {
        // SAFETY: a field of the live child; an array decays to its first
        // element, which is what upstream's cast meant.
        unsafe { (&raw mut (*self.0).uvstdio).cast() }
    }

    /// The libuv loop the child will run on.
    fn uv_loop(self) -> *mut uv_loop_t {
        let uv_loop: *mut Loop = self.proc.loop_0;
        // SAFETY: a child's loop outlives it.
        unsafe { &raw mut (*uv_loop).uv }
    }

    /// The parent's end of the pipe for stdio slot `idx`. Only the three
    /// standard slots have one.
    fn parent_pipe(self, idx: usize) -> *mut uv_pipe_t {
        // SAFETY: all three are fields of the live child.
        unsafe {
            match idx {
                0 => &raw mut (*self.0).proc.in_0.uv.pipe,
                1 => &raw mut (*self.0).proc.out.s.uv.pipe,
                _ => &raw mut (*self.0).proc.err.s.uv.pipe,
            }
        }
    }
}

impl Deref for Uv {
    type Target = LibuvProc;

    fn deref(&self) -> &LibuvProc {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Uv {
    fn deref_mut(&mut self) -> &mut LibuvProc {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

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
fn open_stdio_pipe(mut uvproc: Uv, idx: usize, child_readable: bool) -> c_int {
    let (read_flags, write_flags) = if child_readable {
        (0, UV_NONBLOCK_PIPE)
    } else {
        (UV_NONBLOCK_PIPE, 0)
    };
    let mut pipe_pair: [uv_file; 2] = [0; 2];
    // SAFETY: libuv fills in the two descriptors of a pair we own.
    unsafe { uv_pipe(pipe_pair.as_mut_ptr(), read_flags, write_flags) };

    let [read_end, write_end] = pipe_pair;
    let (child_fd, parent_fd) = if child_readable {
        (read_end, write_end)
    } else {
        (write_end, read_end)
    };
    uvproc.uvstdio[idx].flags = UV_INHERIT_FD;
    uvproc.uvstdio[idx].data.fd = child_fd;
    let parent_pipe = uvproc.parent_pipe(idx);
    // SAFETY: the child's own pipe handle, and a descriptor just opened.
    unsafe { uv_pipe_open(parent_pipe, parent_fd) };
    child_fd
}

/// Start the child. Returns zero, or a negative error code.
///
/// # Safety
/// `uvproc` is a live, unspawned `LibuvProc` on a live loop, with its `argv`
/// and `env` filled in.
pub unsafe fn libuv_proc_spawn(uvproc: *mut LibuvProc) -> c_int {
    // SAFETY: the caller's promise.
    let mut uvproc = unsafe { Uv::new(uvproc) };
    // SAFETY: the child, whose argv the caller filled in.
    let exepath = unsafe { proc_get_exepath(uvproc.as_proc()) };

    uvproc.uvopts.file = exepath;
    uvproc.uvopts.args = uvproc.proc.argv;
    uvproc.uvopts.flags = UV_PROCESS_WINDOWS_HIDE | UV_PROCESS_DETACHED;
    uvproc.uvopts.exit_cb = Some(exit_cb);
    uvproc.uvopts.cwd = uvproc.proc.cwd;
    uvproc.uvopts.stdio = uvproc.stdio();
    uvproc.uvopts.stdio_count = 3;
    for slot in &mut uvproc.uvstdio[..3] {
        slot.flags = UV_IGNORE;
    }

    if ui_client_forward_stdin.get() {
        // A UI client reads the editor's own stdin, on the descriptor the
        // remote side knows as UI_CLIENT_STDIN_FD.
        uvproc.uvopts.stdio_count = 4;
        uvproc.uvstdio[3].data.fd = 0;
        uvproc.uvstdio[3].flags = UV_INHERIT_FD;
    }
    uvproc.uv.data = uvproc.as_proc().cast();

    uvproc.uvopts.env = if uvproc.proc.env.is_null() {
        ptr::null_mut()
    } else {
        let env = uvproc.proc.env;
        // SAFETY: the caller's dictionary of environment variables.
        unsafe { tv_dict_to_env(env) }
    };

    // The parent's copies of the descriptors the child inherits. They are
    // dropped after the spawn, whether or not it succeeded.
    let mut to_close = [-1; 3];
    if !uvproc.proc.in_0.closed {
        to_close[0] = open_stdio_pipe(uvproc, 0, true);
    }
    if !uvproc.proc.out.s.closed {
        to_close[1] = open_stdio_pipe(uvproc, 1, false);
    }
    if !uvproc.proc.err.s.closed {
        to_close[2] = open_stdio_pipe(uvproc, 2, false);
    } else if uvproc.proc.fwd_err {
        // Not read by us, so let the child write straight to our stderr.
        uvproc.uvstdio[2].flags = UV_INHERIT_FD;
        uvproc.uvstdio[2].data.fd = STDERR_FILENO;
    }

    let (uv_loop, handle) = (uvproc.uv_loop(), uvproc.handle());
    let opts = uvproc.opts();
    // SAFETY: the child's loop, its own process handle, and the options
    // filled in above, which libuv keeps pointers into until it closes.
    let status = unsafe { uv_spawn(uv_loop, handle, opts) };
    if status != 0 {
        // SAFETY: the spawn options' own file name; libuv's error strings
        // are static.
        let (file, why) = unsafe { (c_str(uvproc.uvopts.file), c_str(uv_strerror(status))) };
        logmsg!(
            LOGLVL_INF,
            c"libuv_proc_spawn",
            141,
            "uv_spawn({file}) failed: {why}"
        );
        // Nothing will reach `close_cb` to free it.
        if !uvproc.uvopts.env.is_null() {
            let env = uvproc.uvopts.env;
            // SAFETY: the environment `tv_dict_to_env` built above.
            unsafe { os_free_fullenv(env) };
        }
    } else {
        uvproc.proc.pid = uvproc.uv.pid;
    }

    for fd in to_close {
        if fd > -1 {
            // SAFETY: a descriptor this function opened and the child owns
            // its own copy of.
            unsafe { close(fd) };
        }
    }
    status
}

/// Close the child's process handle.
///
/// # Safety
/// `uvproc` is a live `LibuvProc` whose handle libuv still knows about.
pub unsafe fn libuv_proc_close(uvproc: *mut LibuvProc) {
    // SAFETY: the caller's promise.
    let uvproc = unsafe { Uv::new(uvproc) };
    let handle: *mut uv_handle_t = uvproc.handle().cast();
    // SAFETY: the child's own process handle.
    unsafe { uv_close(handle, Some(close_cb)) };
}

/// The process handle finished closing: report it, then release the
/// environment libuv held a pointer into.
///
/// # Safety
/// libuv's: `handle` is the process handle of a live child, whose `data` is
/// the `Proc` [`libuv_proc_spawn`] stored there.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: the caller's promise; a `Proc` and its `LibuvProc` share an
    // address.
    let uvproc = unsafe { Uv::new((*handle).data.cast()) };
    if let Some(notify) = uvproc.proc.internal_close_cb {
        // SAFETY: the child's own close callback.
        unsafe { notify(uvproc.as_proc()) };
    }
    if !uvproc.uvopts.env.is_null() {
        let env = uvproc.uvopts.env;
        // SAFETY: the environment `libuv_proc_spawn` built for the child.
        unsafe { os_free_fullenv(env) };
    }
}

/// The child exited. A signalled death is reported the way a shell does,
/// as 128 plus the signal number.
///
/// # Safety
/// libuv's: `handle` is the process handle of a live child, whose `data` is
/// the `Proc` [`libuv_proc_spawn`] stored there.
unsafe extern "C" fn exit_cb(handle: *mut uv_process_t, status: i64, term_signal: c_int) {
    // SAFETY: the caller's promise; a `Proc` and its `LibuvProc` share an
    // address.
    let mut uvproc = unsafe { Uv::new((*handle).data.cast()) };
    uvproc.proc.status = exit_status(status, term_signal);
    let notify = uvproc.proc.internal_exit_cb;
    let notify = notify.expect("a spawned child has an exit callback");
    // SAFETY: the child's own exit callback.
    unsafe { notify(uvproc.as_proc()) };
}

/// How libuv's `(status, term_signal)` pair becomes the status the editor
/// reports to `jobwait()` and friends.
fn exit_status(status: i64, term_signal: c_int) -> c_int {
    if term_signal != 0 {
        128 + term_signal
    } else {
        number_as_int(status)
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
