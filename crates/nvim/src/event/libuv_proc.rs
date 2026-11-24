//! Children started through libuv's process API.
//!
//! [`LibuvProc`] is a [`Proc`] with a `uv_process_t` bolted on. `Proc` is
//! the first field, so `event/proc.rs` casts freely between the two; libuv is handed the address of the `Proc`
//! (not of the `LibuvProc`) in the handle's `data` field, and the callbacks
//! here cast it back.
//!
//! Every one of those casts goes through [`Uv`], a wrapper whose construction
//! is the unsafe step: past it, reading `uvproc.proc.command` is ordinary Rust,
//! and a write is a field writer (`set_proc_pid`), never a `&mut LibuvProc`.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `event/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::{uv_close, uv_pipe, uv_pipe_open, uv_spawn};
use crate::event::r#loop::{loop_poll_events, main_loop_raw};
use crate::event::objects::Loop;
use crate::event::objects::{Command, Proc, Stream, stream_read_cb};
use crate::event::proc::{PROC_TYPE_UV, proc_init, proc_spawn, proc_stop, proc_wait};
use crate::event::rstream::{Arrived, rstream_init, rstream_start};
use crate::event::stream::stream_may_close;
use crate::event::uv::{
    uv_file, uv_handle_t, uv_loop_t, uv_pipe_t, uv_process_options_t, uv_process_t,
    uv_stdio_container_s_data, uv_stdio_container_t, uv_stdio_flags,
};
use crate::event::wstream::{SharedBuffer, wstream_init, wstream_set_write_cb, wstream_write};
use crate::event_queue::{self, ChildQueue};
use crate::log::{LOGLVL_INF, logmsg};
use crate::memory::cstr_vec::CStrVec;
use crate::message_fmt::msg_cstr;
use crate::narrow::number_as_int;
use crate::os::fs::os_strerror;
use crate::os::libc::STDERR_FILENO;
use crate::shell::Argv;
use crate::startup::UI_CLIENT_FORWARD_STDIN;
use ::libc::close;
use core::ffi::{CStr, c_int, c_void};
use core::ops::Deref;
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
#[derive(Copy, Clone, Debug)]
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

crate::event::stream::field_writers!(Uv {
    fn set_uv_data => uv.data: *mut c_void;
    fn set_proc_pid => proc.pid: c_int;
    fn set_proc_status => proc.status: c_int;
});

/// A fresh, unspawned libuv child.
pub fn libuv_proc_init(uv_loop: *mut Loop, data: *mut c_void) -> LibuvProc {
    // SAFETY: every field is a pointer, an integer or an `Option<fn>`, for
    // all of which the all-zero pattern is the "unset" value; the spawn path
    // below fills in the ones it needs.
    let mut rv: LibuvProc = unsafe { core::mem::zeroed() };
    rv.proc = proc_init(uv_loop, PROC_TYPE_UV, data);
    rv
}

/// Wire stdio slot `idx` to a fresh pipe: fill in `slot` with the child's
/// end, and return that descriptor, which the parent must close once the
/// child has been spawned.
///
/// `child_readable` is true for the child's stdin — the child reads the read
/// end and the parent keeps the write end — and false for its output
/// streams. The parent's end is always non-blocking; the child's is not.
///
/// A `uv_pipe` pair is used rather than libuv's own `UV_CREATE_PIPE`, which
/// as of libuv 1.51 is a `socketpair` and so breaks `/proc/<pid>/fd/0`.
fn open_stdio_pipe(
    uvproc: Uv,
    slot: &mut uv_stdio_container_t,
    idx: usize,
    child_readable: bool,
) -> c_int {
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
    slot.flags = UV_INHERIT_FD;
    slot.data.fd = child_fd;
    let parent_pipe = uvproc.parent_pipe(idx);
    // SAFETY: the child's own pipe handle, and a descriptor just opened.
    unsafe { uv_pipe_open(parent_pipe, parent_fd) };
    child_fd
}

/// Start the child. Returns zero, or a negative error code.
///
/// libuv reads the options, and the command line, directory and
/// environment they point into, only while `uv_spawn` runs (the fork's
/// child has its own copy), so they are this frame's.
///
/// # Safety
/// `uvproc` is a live, unspawned `LibuvProc` on a live loop, with its
/// command filled in.
pub unsafe fn libuv_proc_spawn(uvproc: *mut LibuvProc) -> c_int {
    // SAFETY: the caller's promise.
    let uvproc = unsafe { Uv::new(uvproc) };
    let command = uvproc
        .proc
        .command
        .as_deref()
        .expect("a child is spawned with a command");

    let unused = uv_stdio_container_t {
        flags: UV_IGNORE,
        data: uv_stdio_container_s_data { fd: 0 },
    };
    let mut stdio = [unused; 4];
    let mut stdio_count = 3;
    if UI_CLIENT_FORWARD_STDIN.get() {
        // A UI client reads the editor's own stdin, on the descriptor the
        // remote side knows as UI_CLIENT_STDIN_FD.
        stdio_count = 4;
        stdio[3].data.fd = 0;
        stdio[3].flags = UV_INHERIT_FD;
    }
    uvproc.set_uv_data(uvproc.as_proc().cast());

    // The parent's copies of the descriptors the child inherits. They are
    // dropped after the spawn, whether or not it succeeded.
    let mut to_close = [-1; 3];
    if !uvproc.proc.in_0.closed {
        to_close[0] = open_stdio_pipe(uvproc, &mut stdio[0], 0, true);
    }
    if !uvproc.proc.out.s.closed {
        to_close[1] = open_stdio_pipe(uvproc, &mut stdio[1], 1, false);
    }
    if !uvproc.proc.err.s.closed {
        to_close[2] = open_stdio_pipe(uvproc, &mut stdio[2], 2, false);
    } else if uvproc.proc.fwd_err {
        // Not read by us, so let the child write straight to our stderr.
        stdio[2].flags = UV_INHERIT_FD;
        stdio[2].data.fd = STDERR_FILENO;
    }

    let program = command.program();
    let opts = uv_process_options_t {
        exit_cb: Some(exit_cb),
        file: program.as_ptr(),
        args: command.argv.as_ptr(),
        env: command
            .env
            .as_ref()
            .map_or(ptr::null_mut(), CStrVec::as_ptr),
        cwd: command.cwd.as_deref().map_or(ptr::null(), CStr::as_ptr),
        flags: UV_PROCESS_WINDOWS_HIDE | UV_PROCESS_DETACHED,
        stdio_count,
        stdio: stdio.as_mut_ptr(),
        uid: 0,
        gid: 0,
    };
    // SAFETY: the child's loop, its own process handle, and options that
    // point into this frame and the child's command, both live until the
    // call returns.
    let status = unsafe { uv_spawn(uvproc.uv_loop(), uvproc.handle(), &raw const opts) };
    if status == 0 {
        uvproc.set_proc_pid(uvproc.uv.pid);
    } else {
        let (file, why) = (msg_cstr(program), msg_cstr(os_strerror(status)));
        logmsg!(
            LOGLVL_INF,
            c"libuv_proc_spawn",
            141,
            "uv_spawn({file}) failed: {why}"
        );
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

/// The process handle finished closing: report it.
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
    let uvproc = unsafe { Uv::new((*handle).data.cast()) };
    uvproc.set_proc_status(exit_status(status, term_signal));
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

// ---------------------------------------------------------------------------
// Running a child to completion
// ---------------------------------------------------------------------------

/// What reads a child's output in [`run_to_exit`]: each chunk, the
/// descriptor it came from (1 or 2), and whether the stream ended. Answers
/// how much it took.
pub(crate) type OutputSink<'a> = &'a mut dyn FnMut(&[u8], c_int, bool) -> usize;

/// How a [`run_to_exit`] went.
#[derive(Debug)]
pub(crate) enum ChildRun {
    /// The child could not be started: libuv's error.
    NotStarted(c_int),
    /// Its stdin would not take the input; it has been asked to stop.
    InputRefused,
    /// It exited with this status (as `proc_wait` answers it).
    Exited(c_int),
}

/// Spawn `argv` on the editor's loop with `input` on its stdin, hand its
/// stdout and stderr to `output` (or drop them when there is none), and pump
/// the loop until it exits: what `system()`, `:!` and wildcard expansion
/// run on. `on_input_written` is told libuv's status once the input is
/// written, after which stdin is closed; `before_wait` runs once the child
/// is going and before the wait.
///
/// The child, its streams and its queue live on this frame, which outlives
/// every callback they register.
pub(crate) fn run_to_exit(
    argv: Argv,
    input: &[u8],
    output: Option<OutputSink<'_>>,
    on_input_written: fn(c_int),
    before_wait: impl FnOnce(),
) -> ChildRun {
    let mut sink = output;
    let has_input = !input.is_empty();
    let reader: stream_read_cb = if sink.is_some() {
        Some(sink_shim)
    } else {
        None
    };
    let sink_data = (&raw mut sink).cast::<c_void>();
    let events = ChildQueue::of_main_loop();
    let mut uvproc = libuv_proc_init(main_loop_raw(), ptr::null_mut());
    uvproc.proc.events = Some(events.id());
    uvproc.proc.command = Some(Box::new(Command::new(argv)));
    let proc = &raw mut uvproc.proc;

    // SAFETY: the child on this frame, unspawned, with its command set.
    let status = unsafe { proc_spawn(proc, has_input, true, true) };
    if status != 0 {
        // SAFETY: the editor's loop, live for the process; this lets the
        // spawn's closes finish.
        unsafe { loop_poll_events(main_loop_raw(), 0) };
        return ChildRun::NotStarted(status);
    }

    // Unlike process events, stream events are not queued: they are dealt
    // with as fast as possible so the streams are not closed while the OS
    // buffer still holds data the child wrote before exiting.
    // SAFETY: the spawned child's streams, on this frame, which outlives
    // them; `sink_shim` reads `sink`, which does too.
    unsafe {
        if has_input {
            wstream_init(&raw mut (*proc).in_0, 0);
        }
        for out in [&raw mut (*proc).out, &raw mut (*proc).err] {
            rstream_init(out);
            rstream_start(out, reader, sink_data);
        }
    }

    if has_input {
        let buffer = SharedBuffer::from_vec(input.to_vec());
        // SAFETY: the child's stdin, initialised above, and the one
        // reference the write takes over.
        if unsafe { wstream_write(&raw mut (*proc).in_0, buffer) } != 0 {
            // SAFETY: the spawned child.
            unsafe { proc_stop(proc) };
            return ChildRun::InputRefused;
        }
        // Close stdin once everything has been written.
        let report = on_input_written as *mut c_void;
        // SAFETY: as above; `input_written_shim` reads the hook back.
        unsafe { wstream_set_write_cb(&raw mut (*proc).in_0, Some(input_written_shim), report) };
    }

    before_wait();
    // SAFETY: the spawned child, which this frame keeps for the wait.
    let code = unsafe { proc_wait(proc, -1, None) };
    debug_assert!(event_queue::is_empty(events.id()));
    ChildRun::Exited(code)
}

/// [`run_to_exit`]'s read callback: hand the chunk to the caller's sink,
/// the frame's `Option<OutputSink>` that `data` points at.
fn sink_shim(arrived: &Arrived<'_>) -> usize {
    let count = arrived.bytes.len();
    if count == 0 {
        return count;
    }
    // SAFETY: `run_to_exit` started the read with its frame's sink as the
    // data word; the frame outlives the read, and the sink is not otherwise
    // borrowed while the child runs.
    let sink = unsafe { &mut *arrived.data.cast::<Option<OutputSink<'_>>>() };
    sink.as_mut()
        .map_or(count, |sink| sink(arrived.bytes, arrived.fd, arrived.eof))
}

/// [`run_to_exit`]'s write callback: tell the caller's hook, and close
/// stdin.
///
/// # Safety
/// A `stream_write_cb` set by `run_to_exit`: `stream` is the child's live
/// stdin and `data` a `fn(c_int)`.
unsafe fn input_written_shim(stream: *mut Stream, data: *mut c_void, status: c_int) {
    // SAFETY: `run_to_exit` stored a `fn(c_int)` here.
    let report = unsafe { core::mem::transmute::<*mut c_void, fn(c_int)>(data) };
    report(status);
    // SAFETY: the caller's stream.
    unsafe { stream_may_close(stream) };
}

#[repr(C)]
#[derive(Debug)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
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
