//! Child processes: spawning them, waiting on them, stopping them, and
//! tearing the whole set down at exit.
//!
//! A [`Proc`] is the part of a child that does not depend on how it was
//! started. The two concrete kinds embed it as their first field —
//! `LibuvProc` (a `uv_process_t`) and `PtyProc` (a `forkpty` child) — so a
//! `*mut Proc` casts to and from either, and the `type_0` field says which.
//! That cast is the reason `Proc`, `LibuvProc` and `PtyProc` are all
//! `repr(C)`: `Channel` embeds all three in a union, by value.
//!
//! # Lifetime
//!
//! A spawned child is refcounted: one reference per open standard stream,
//! plus one for the child itself. libuv drops them by calling back — a
//! stream's close callback and the child's exit callback both land in
//! [`decref`] — and the last drop unlinks the child from `loop->children`
//! and queues the caller's exit callback. Nothing here frees a `Proc`; the
//! owner (a `Channel`, or a stack frame in `os/shell/`) does that.
//!
//! # Aliasing
//!
//! libuv and the stream layer both hold the address of a `Proc` (or of a
//! field inside it) in a `data` pointer for as long as the child lives, so a
//! `Proc` must not move once spawned, and the callbacks reached from
//! [`proc_close_handles`] may re-enter this module while a caller further up
//! the stack still holds a `*mut Proc` to the same child.
//!
//! That is what [`Child`] is for: a child is always reached through a raw
//! pointer that outlives any borrow of it, so the pointer is wrapped once —
//! paying the `unsafe` at construction — and every field access below is
//! ordinary Rust. [`Output`] is the same wrapper for one of the child's two
//! output streams.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::{
    uv_close, uv_pipe_init, uv_recv_buffer_size, uv_timer_start, uv_timer_stop, uv_unref,
};
use crate::event::libuv_proc::{libuv_proc_close, libuv_proc_spawn};
use crate::event::r#loop::{
    loop_children, loop_poll_events, one_arg_event, process_events, process_events_until,
};
use crate::event::multiqueue::{multiqueue_empty, multiqueue_process_events, multiqueue_put_event};
use crate::event::rstream::rstream_may_close;
use crate::event::stream::{stream_init, stream_may_close};
use crate::event::{pack_int, unpack_int};
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_DBG, LOGLVL_INF, logmsg};
use crate::main::{
    exiting, got_int, main_loop, os_exit, preserve_exit, ui_client_channel_id,
    ui_client_exit_status,
};
use crate::os::proc::os_proc_tree_kill;
use crate::os::pty_proc_unix::{
    pty_proc_close, pty_proc_close_master, pty_proc_flush_master, pty_proc_spawn, pty_proc_teardown,
};
use crate::os::shell::shell_free_argv;
use crate::os::signal::{SIGHUP, SIGKILL, SIGTERM};
use crate::os::time::os_hrtime;
use crate::types::{
    LibuvProc, Loop, MultiQueue, Proc, ProcType, PtyProc, RStream, Stream, uv_handle_t, uv_loop_t,
    uv_timer_t,
};
use core::ffi::{c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::ptr;

/// A child started through libuv's process API.
pub const kProcTypeUv: ProcType = 0;
/// A child started through `forkpty`.
pub const kProcTypePty: ProcType = 1;

/// How long a stopped child has to exit cleanly before it is killed. A pty
/// child is sent SIGTERM at this point (SIGHUP having already failed) and
/// gets another interval before SIGKILL.
const KILL_TIMEOUT_MS: u64 = 2000;

/// Set for the whole of [`proc_teardown`]. Relaxes the "a child is closed
/// exactly once" assertion, because a detached or pty child can die while
/// being torn down and be closed a second time from its exit callback.
static PROC_IS_TEARING_DOWN: GlobalCell<bool> = GlobalCell::new(false);

/// Non-zero while [`proc_close_handles`] is draining a child's output.
/// Exiting in the middle of that deadlocks, so [`exit_event`] reschedules
/// itself instead.
static EXIT_NEED_DELAY: GlobalCell<c_int> = GlobalCell::new(0);

// ---------------------------------------------------------------------------
// Handles
// ---------------------------------------------------------------------------

/// A child this module is working with, plus the promise that the pointer
/// behind it stays live for as long as the handle does. See the module's
/// aliasing note.
#[derive(Copy, Clone)]
struct Child(*mut Proc);

impl Child {
    /// # Safety
    /// `proc` is non-null and points at a live `Proc` for the whole life of
    /// the handle and of everything derived from it.
    unsafe fn new(proc: *mut Proc) -> Self {
        debug_assert!(!proc.is_null());
        Child(proc)
    }

    /// The pointer back, for the C-shaped callees that still want one.
    fn as_ptr(self) -> *mut Proc {
        self.0
    }

    /// The same child as a `LibuvProc`. `Proc` is its first field.
    fn as_libuv(self) -> *mut LibuvProc {
        self.0.cast()
    }

    /// The same child as a `PtyProc`. `Proc` is its first field.
    fn as_pty(self) -> *mut PtyProc {
        self.0.cast()
    }

    /// Was the child started by `forkpty` rather than by libuv?
    fn is_pty(self) -> bool {
        self.type_0 == kProcTypePty
    }

    /// The libuv loop the child runs on.
    fn uv_loop(self) -> *mut uv_loop_t {
        let uv_loop = self.loop_0;
        // SAFETY: a child's loop outlives it.
        unsafe { &raw mut (*uv_loop).uv }
    }

    /// The loop's deferred-event queue.
    fn loop_events(self) -> *mut MultiQueue {
        // SAFETY: a child's loop outlives it.
        unsafe { (*self.loop_0).events }
    }

    /// The one timer that kills every child that outstayed [`proc_stop`].
    fn kill_timer(self) -> *mut uv_timer_t {
        let uv_loop = self.loop_0;
        // SAFETY: a child's loop outlives it.
        unsafe { &raw mut (*uv_loop).children_kill_timer }
    }

    /// The child's standard input.
    fn stdin(self) -> *mut Stream {
        // SAFETY: a field of the live child.
        unsafe { &raw mut (*self.0).in_0 }
    }

    /// The child's three standard streams, in descriptor order.
    fn stdio(self) -> [*mut Stream; 3] {
        // SAFETY: the two output streams embed a `Stream` as their first
        // field, and both are fields of the live child.
        let outputs = unsafe { [&raw mut (*self.0).out.s, &raw mut (*self.0).err.s] };
        [self.stdin(), outputs[0], outputs[1]]
    }

    /// The child's standard output.
    fn stdout(self) -> Output {
        // SAFETY: a field of the live child.
        Output(unsafe { &raw mut (*self.0).out })
    }

    /// The child's standard error.
    fn stderr(self) -> Output {
        // SAFETY: a field of the live child.
        Output(unsafe { &raw mut (*self.0).err })
    }
}

impl Deref for Child {
    type Target = Proc;

    fn deref(&self) -> &Proc {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Child {
    fn deref_mut(&mut self) -> &mut Proc {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// One of a child's two output streams. Only ever built from [`Child`], so
/// it inherits that handle's promise.
#[derive(Copy, Clone)]
struct Output(*mut RStream);

impl Output {
    /// The pointer back, for the stream layer, which still wants one.
    fn as_ptr(self) -> *mut RStream {
        self.0
    }

    /// The stream's libuv handle. Every arm of the `uv` union shares an
    /// address, so the pipe/tcp/idle distinction does not matter here.
    fn uv_handle(self) -> *mut uv_handle_t {
        // SAFETY: a field of the live stream.
        unsafe { (&raw mut (*self.0).s.uv).cast() }
    }
}

impl Deref for Output {
    type Target = RStream;

    fn deref(&self) -> &RStream {
        // SAFETY: the promise inherited from the `Child` that built it.
        unsafe { &*self.0 }
    }
}

/// A signal number as `exit_signal` records it.
///
/// Every signal the editor sends a child is one of the three named below, so
/// the narrowing the C wrote as `(uint8_t)` cannot lose anything.
fn as_exit_signal(signal: c_int) -> u8 {
    u8::try_from(signal).expect("the editor only ever sends a small signal number")
}

// ---------------------------------------------------------------------------
// Starting, waiting, stopping
// ---------------------------------------------------------------------------

/// A fresh, unspawned child of the given kind.
///
/// Every field of `Proc` is a pointer, an integer, a `bool` or an
/// `Option<fn>`, for all of which all-zeroes is the "unset" value the
/// field-by-field initialiser upstream spelled out; only the four fields
/// below differ from it.
pub fn proc_init(uv_loop: *mut Loop, kind: ProcType, data: *mut c_void) -> Proc {
    // SAFETY: see above — `Proc` is inhabited by the all-zero bit pattern.
    let mut proc: Proc = unsafe { core::mem::zeroed() };
    proc.type_0 = kind;
    proc.loop_0 = uv_loop;
    proc.data = data;
    proc.status = -1;
    proc.out.s.fd = 1; // STDOUT_FILENO
    proc.err.s.fd = 2; // STDERR_FILENO
    proc
}

/// Whether a child has exited, or has been asked to and is being waited out.
pub fn proc_is_stopped(proc: &Proc) -> bool {
    proc.status >= 0 || proc.stopped_time != 0
}

/// The executable a child was (or is about to be) started from.
///
/// `exepath` is set when the caller resolved the program itself; otherwise
/// it is `argv[0]`, as passed to `execvp`.
///
/// # Safety
/// `proc` is a live child whose `argv` has been filled in.
pub unsafe fn proc_get_exepath(proc: *mut Proc) -> *const c_char {
    // SAFETY: the caller's promise.
    let child = unsafe { Child::new(proc) };
    if child.exepath.is_null() {
        // SAFETY: a child without an `exepath` was given an argv to exec.
        unsafe { *child.argv }
    } else {
        child.exepath
    }
}

/// Start `proc`, optionally with each of the three standard streams piped.
///
/// Returns zero, or a negative error code — in which case `proc` has already
/// been closed and freed and its status set to -1.
///
/// # Safety
/// `proc` is a live, unspawned child built by [`proc_init`] on a live loop.
pub unsafe fn proc_spawn(proc: *mut Proc, has_in: bool, has_out: bool, has_err: bool) -> c_int {
    // SAFETY: the caller's promise.
    let mut child = unsafe { Child::new(proc) };

    // Forwarding stderr contradicts processing it internally.
    debug_assert!(!(has_err && child.fwd_err));

    let stdio = child.stdio();
    let wanted = [has_in, has_out, has_err];

    // A stream the caller did not ask for starts out closed; the rest get a
    // pipe handle now, which the spawn hooks below fill with a descriptor.
    for (stream, wanted) in stdio.into_iter().zip(wanted) {
        if wanted {
            // SAFETY: a field of the live child, on the child's own loop.
            unsafe { uv_pipe_init(child.uv_loop(), &raw mut (*stream).uv.pipe, 0) };
        } else {
            // SAFETY: a field of the live child.
            unsafe { (*stream).closed = true };
        }
    }

    let status = if child.is_pty() {
        // SAFETY: a pty child is a `PtyProc`, and is not yet spawned.
        unsafe { pty_proc_spawn(child.as_pty()) }
    } else {
        // SAFETY: a libuv child is a `LibuvProc`, and is not yet spawned.
        unsafe { libuv_proc_spawn(child.as_libuv()) }
    };

    if status != 0 {
        for (stream, wanted) in stdio.into_iter().zip(wanted) {
            if wanted {
                // SAFETY: the handle this function initialised above.
                unsafe { uv_close((&raw mut (*stream).uv.pipe).cast(), None) };
            }
        }
        if child.type_0 == kProcTypeUv {
            // The process handle was never started, so it is dropped
            // directly rather than through `proc_close`, which would run the
            // close callbacks of a child that does not exist.
            // SAFETY: the handle `libuv_proc_spawn` initialised.
            unsafe { uv_close((&raw mut (*child.as_libuv()).uv).cast(), None) };
        } else {
            proc_close(child);
        }
        // SAFETY: the caller's child, which owns its argv.
        unsafe { proc_free(child.as_ptr()) };
        child.status = -1;
        return status;
    }

    // One reference per piped stream, plus one for the child itself.
    for (stream, wanted) in stdio.into_iter().zip(wanted) {
        if !wanted {
            continue;
        }
        // SAFETY: a field of the live child, whose pipe the spawn filled in.
        unsafe {
            stream_init(
                ptr::null_mut(),
                stream,
                -1,
                (&raw mut (*stream).uv.pipe).cast(),
            );
            (*stream).internal_data = child.as_ptr().cast();
            (*stream).internal_close_cb = Some(on_proc_stream_close);
        }
        child.refcount += 1;
    }
    child.internal_exit_cb = Some(on_proc_exit);
    child.internal_close_cb = Some(decref);
    child.refcount += 1;

    // SAFETY: the child's loop, which keeps the list of live children.
    unsafe { (*loop_children(child.loop_0)).push(child.as_ptr()) };

    let pid = child.pid;
    // SAFETY: the child, which has just been spawned.
    let exepath = unsafe { proc_get_exepath(child.as_ptr()) };
    let fmt = c"new: pid=%d exepath=[%s]";
    // SAFETY: `logmsg_begin` takes the log's lock and hands back its handle.
    unsafe { logmsg!(LOGLVL_DBG, c"proc_spawn", 127, fmt, pid, exepath) };
    0
}

/// Stop every child and wait for the loop to be free of them.
///
/// Detached and pty children are not killed — their handles are closed and
/// they are left to run — but everything else is asked to terminate.
///
/// # Safety
/// `uv_loop` is a live loop that has been through `loop_init`.
pub unsafe fn proc_teardown(uv_loop: *mut Loop) {
    PROC_IS_TEARING_DOWN.set(true);
    // Re-read the list each pass: closing a child's handles can run its exit
    // callback inline, which unlinks it from this very list.
    let mut i = 0;
    // SAFETY: the caller's loop.
    while let Some(child) = unsafe { nth_child(uv_loop, i) } {
        if child.detach || child.is_pty() {
            let arg = child.as_ptr().cast();
            // SAFETY: the loop's own queue, and the child the handler wants.
            unsafe { create_event((*uv_loop).events, proc_close_handles, arg) };
        } else {
            // SAFETY: a live child.
            unsafe { proc_stop(child.as_ptr()) };
        }
        i += 1;
    }

    let drained = || {
        // SAFETY: the caller's loop.
        unsafe { (*loop_children(uv_loop)).is_empty() && multiqueue_empty((*uv_loop).events) }
    };
    // SAFETY: the caller's loop.
    unsafe { process_events_until(uv_loop, (*uv_loop).events, -1, drained) };
    // SAFETY: as above.
    unsafe { pty_proc_teardown(uv_loop) };
}

/// Close all three of a child's standard streams.
///
/// # Safety
/// `proc` is a live child.
pub unsafe fn proc_close_streams(proc: *mut Proc) {
    // SAFETY: the caller's promise.
    let child = unsafe { Child::new(proc) };
    // SAFETY: the child's own streams, which live as long as it does.
    unsafe { stream_may_close(child.stdin()) };
    // SAFETY: as above.
    unsafe { rstream_may_close(child.stdout().as_ptr()) };
    // SAFETY: as above.
    unsafe { rstream_may_close(child.stderr().as_ptr()) };
}

/// Wait for `proc` to finish, for at most `ms` milliseconds.
///
/// `ms` of 0 polls once and returns; -1 waits indefinitely. `events`, if
/// given, is drained instead of the child's own queue.
///
/// Returns the child's exit status, -1 if the wait timed out with the child
/// still running, or -2 if the user interrupted it.
///
/// # Safety
/// `proc` is a live child, and `events`, if non-null, is a live queue.
pub unsafe fn proc_wait(proc: *mut Proc, ms: c_int, mut events: *mut MultiQueue) -> c_int {
    // SAFETY: the caller's promise.
    let mut child = unsafe { Child::new(proc) };

    if child.refcount == 0 {
        // Read the status before draining: an event may free the child.
        let status = child.status;
        // SAFETY: the child's loop and its own queue.
        unsafe { process_events(child.loop_0, child.events, 0) };
        return status;
    }

    if events.is_null() {
        events = child.events;
    }

    // Hold a reference of our own so the exit callback cannot free the child
    // before its status has been read.
    child.refcount += 1;
    let interrupted_or_last = move || got_int.get() || child.refcount == 1;
    // SAFETY: the child's loop, and the caller's queue.
    unsafe { process_events_until(child.loop_0, events, i64::from(ms), interrupted_or_last) };

    // A user hitting CTRL-C is assumed not to like the current job.
    if got_int.get() {
        got_int.set(false);
        // SAFETY: a live child.
        unsafe { proc_stop(child.as_ptr()) };
        if ms == -1 {
            // Returning is only safe once every handle is closed too.
            let last = move || child.refcount == 1;
            // SAFETY: the child's loop, and the caller's queue.
            unsafe { process_events_until(child.loop_0, events, -1, last) };
        } else {
            // SAFETY: as above.
            unsafe { process_events(child.loop_0, events, 0) };
        }
        child.status = -2;
    }

    if child.refcount == 1 {
        // SAFETY: a live child holding the reference taken above.
        unsafe { decref(child.as_ptr()) };
        if !child.events.is_null() {
            // `decref` queued the exit event; run it now.
            // SAFETY: the child's own queue.
            unsafe { multiqueue_process_events(child.events) };
        }
    } else {
        child.refcount -= 1;
    }
    child.status
}

/// Ask `proc` to terminate, and arm the timer that kills it if it does not.
///
/// # Safety
/// `proc` is a live child.
pub unsafe fn proc_stop(proc: *mut Proc) {
    // SAFETY: the caller's promise.
    let mut child = unsafe { Child::new(proc) };
    if proc_is_stopped(&child) {
        return;
    }
    child.stopped_time = os_hrtime();

    if child.is_pty() {
        // Closing every stream is what sends SIGHUP to a pty child.
        child.exit_signal = as_exit_signal(SIGHUP);
        // SAFETY: a live child.
        unsafe { proc_close_streams(child.as_ptr()) };
        // SAFETY: a pty child is a `PtyProc`.
        unsafe { pty_proc_close_master(child.as_pty()) };
    } else {
        child.exit_signal = as_exit_signal(SIGTERM);
        os_proc_tree_kill(child.pid, SIGTERM);
    }

    arm_kill_timer(child);
}

/// Release the resources the child itself owns. The `Proc` is the caller's.
///
/// # Safety
/// `proc` is a live child.
pub unsafe fn proc_free(proc: *mut Proc) {
    // SAFETY: the caller's promise.
    let mut child = unsafe { Child::new(proc) };
    if !child.argv.is_null() {
        // SAFETY: the argv the child was spawned with, which it owns.
        unsafe { shell_free_argv(child.argv) };
        child.argv = ptr::null_mut();
    }
}

/// Self-exit, because the primary RPC channel was closed.
pub fn exit_on_closed_chan(status: c_int) {
    let msg = c"self-exit triggered by closed RPC channel...";
    // SAFETY: `logmsg_begin` takes the log's lock and hands back its handle.
    unsafe { logmsg!(LOGLVL_DBG, c"exit_on_closed_chan", 440, msg) };
    let event = one_arg_event(Some(exit_event), pack_int(status));
    // SAFETY: `main_loop.fast_events` is live for as long as the editor is.
    unsafe { multiqueue_put_event((*main_loop.ptr()).fast_events, event) };
}

// ---------------------------------------------------------------------------
// The loop's list of live children
// ---------------------------------------------------------------------------

/// The `i`th child on the loop's list, or `None` once the list runs out.
///
/// The borrow is momentary on purpose: the list is re-entered while a child
/// is being closed, so a caller walks it by index rather than by iterator.
///
/// # Safety
/// `uv_loop` is a live loop that has been through `loop_init`.
unsafe fn nth_child(uv_loop: *mut Loop, i: usize) -> Option<Child> {
    // SAFETY: the caller's promise.
    let proc = *unsafe { &*loop_children(uv_loop) }.get(i)?;
    // SAFETY: everything on the list is a live child, until it unlinks itself.
    Some(unsafe { Child::new(proc) })
}

/// Remove `proc` from `children`, preserving the order of the rest.
fn remove_child(children: &mut Vec<*mut Proc>, proc: *mut Proc) {
    let i = children
        .iter()
        .position(|&child| child == proc)
        .expect("a child that is being closed is on the loop's child list");
    children.remove(i);
}

// ---------------------------------------------------------------------------
// Event-loop plumbing
// ---------------------------------------------------------------------------

/// Queue `handler` on `queue`, or run it immediately if there is no queue.
///
/// Upstream's `CREATE_EVENT`. A `Loop` without an event queue is one that is
/// driven synchronously (`os/shell/` builds one), so the handler simply
/// runs on the spot.
///
/// # Safety
/// `queue`, if non-null, is a live queue, and `arg` is what `handler` expects
/// to find in the first slot of its argv.
unsafe fn create_event(
    queue: *mut MultiQueue,
    handler: unsafe extern "C" fn(*mut *mut c_void),
    arg: *mut c_void,
) {
    if queue.is_null() {
        let mut argv = [arg];
        // SAFETY: the caller's promise about `arg`.
        unsafe { handler(argv.as_mut_ptr()) };
    } else {
        // SAFETY: the caller's queue.
        unsafe { multiqueue_put_event(queue, one_arg_event(Some(handler), arg)) };
    }
}

/// (Re)arm the loop's kill timer, which fires [`children_kill_cb`] once
/// `KILL_TIMEOUT_MS` has passed. There is one timer per loop, not per child.
fn arm_kill_timer(child: Child) {
    let timer = child.kill_timer();
    // SAFETY: the timer is a field of the child's loop, which outlives it.
    unsafe { uv_timer_start(timer, Some(children_kill_cb), KILL_TIMEOUT_MS, 0) };
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------

/// The kill timer: SIGKILL anything that did not exit after [`proc_stop`].
///
/// A pty child gets SIGTERM first (SIGHUP having already been sent by closing
/// its streams) and the timer is restarted; `stopped_time` is set to
/// `u64::MAX` to record that.
///
/// # Safety
/// libuv's: `handle` is the `children_kill_timer` of a live `Loop`.
unsafe extern "C" fn children_kill_cb(handle: *mut uv_timer_t) {
    // SAFETY: a loop stores itself in its libuv loop's `data`.
    let uv_loop: *mut Loop = unsafe { (*(*handle).loop_0).data }.cast();
    let mut i = 0;
    // SAFETY: the timer's loop is live for as long as the timer is.
    while let Some(mut child) = unsafe { nth_child(uv_loop, i) } {
        i += 1;
        let exited = child.status >= 0;
        if exited || child.stopped_time == 0 {
            continue;
        }
        let term_sent = child.stopped_time == u64::MAX;
        if !child.is_pty() || term_sent {
            child.exit_signal = as_exit_signal(SIGKILL);
            os_proc_tree_kill(child.pid, SIGKILL);
        } else {
            child.exit_signal = as_exit_signal(SIGTERM);
            os_proc_tree_kill(child.pid, SIGTERM);
            child.stopped_time = u64::MAX;
            arm_kill_timer(child);
        }
    }
}

/// The last thing that happens to a child: hand it to its owner's callback,
/// which is responsible for freeing it.
///
/// # Safety
/// Queued by [`decref`]: `argv[0]` is a live child that has been unlinked.
unsafe extern "C" fn proc_close_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise about the argv.
    let child = unsafe { Child::new((*argv).cast()) };
    if let Some(notify) = child.cb {
        // SAFETY: the owner's callback, given the child it was registered for.
        unsafe { notify(child.as_ptr(), child.status, child.data) };
    } else {
        // SAFETY: a live child nobody else will free.
        unsafe { proc_free(child.as_ptr()) };
    }
}

/// Drop one reference to `proc`; unlink and report it when the last one goes.
///
/// # Safety
/// `proc` is a live child holding at least one reference.
unsafe fn decref(proc: *mut Proc) {
    // SAFETY: the caller's promise.
    let mut child = unsafe { Child::new(proc) };
    child.refcount -= 1;
    if child.refcount != 0 {
        return;
    }
    // SAFETY: the child's loop keeps the list, and this child is on it.
    remove_child(unsafe { &mut *loop_children(child.loop_0) }, child.as_ptr());
    // SAFETY: the child's own queue; `proc_close_event` wants the child.
    unsafe { create_event(child.events, proc_close_event, child.as_ptr().cast()) };
}

/// Close the child's own handle (the process or the pty master).
fn proc_close(mut child: Child) {
    if PROC_IS_TEARING_DOWN.get() && child.closed && (child.detach || child.is_pty()) {
        // A detached or pty child that dies while being torn down gets here
        // twice: once from the teardown, once from its own exit callback.
        return;
    }
    debug_assert!(!child.closed);
    child.closed = true;

    if child.detach && child.type_0 == kProcTypeUv {
        // Let the loop exit without waiting for a child we no longer own.
        // SAFETY: a libuv child's process handle lives as long as it does.
        unsafe { uv_unref((&raw mut (*child.as_libuv()).uv).cast()) };
    }

    if child.is_pty() {
        // SAFETY: a pty child is a `PtyProc`.
        unsafe { pty_proc_close(child.as_pty()) };
    } else {
        // SAFETY: a libuv child is a `LibuvProc`.
        unsafe { libuv_proc_close(child.as_libuv()) };
    }
}

/// Read whatever a dead child left in one of its output streams.
///
/// The read is bounded so that a child which keeps its output open — or a
/// grandchild that inherited it — cannot block teardown forever. The bound is
/// the system receive buffer size on top of what has already been read, which
/// is the most a terminated process can still have queued.
fn flush_stream(child: Child, stream: Output) {
    if stream.s.closed {
        return;
    }

    let mut max_bytes = usize::MAX;
    // A pty master is exempt until teardown: on Linux it can hold far more
    // than one system buffer's worth. #3030
    if !child.is_pty() || PROC_IS_TEARING_DOWN.get() {
        // Upstream reuses `ARENA_BLOCK_SIZE` for this; it is a read-ahead
        // bound, not an allocation size, and libuv only fails to answer for
        // a handle that has no socket underneath it.
        const FALLBACK_READ_AHEAD: usize = 4096;
        let mut system_buffer_size: c_int = 0;
        let size = &raw mut system_buffer_size;
        // SAFETY: the stream's own libuv handle.
        let err = unsafe { uv_recv_buffer_size(stream.uv_handle(), size) };
        let read_ahead = match err {
            // libuv reports a byte count, never a negative one.
            0 => usize::try_from(system_buffer_size).unwrap_or(FALLBACK_READ_AHEAD),
            _ => FALLBACK_READ_AHEAD,
        };
        max_bytes = stream.num_bytes + read_ahead;
    }

    // Not immutable: the events processed in the body reach the stream's own
    // callbacks, which are what advance `num_bytes` and can close it.
    #[allow(clippy::while_immutable_condition)]
    while !stream.s.closed && stream.num_bytes < max_bytes {
        let num_bytes = stream.num_bytes;

        if child.is_pty() && !stream.did_eof {
            // SAFETY: a pty child is a `PtyProc`.
            unsafe { pty_proc_flush_master(child.as_pty()) };
        }
        // SAFETY: the child's loop, and the stream's own queue.
        unsafe {
            loop_poll_events(child.loop_0, 0);
            if !stream.s.events.is_null() {
                multiqueue_process_events(stream.s.events);
            }
        }

        if num_bytes != stream.num_bytes {
            continue;
        }
        // Nothing arrived, so the stream is empty. A child that keeps it open
        // would otherwise deny the reader its end-of-file.
        if let Some(read) = stream.read_cb
            && !stream.did_eof
        {
            // SAFETY: the stream's own reader, given the stream's own buffer.
            unsafe { read(stream.as_ptr(), stream.buffer, 0, stream.s.cb_data, true) };
        }
        break;
    }
}

/// Drain and close everything belonging to a child that has exited.
///
/// # Safety
/// Queued by [`on_proc_exit`] or [`proc_teardown`]: `argv[0]` is a live child.
unsafe extern "C" fn proc_close_handles(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise about the argv.
    let child = unsafe { Child::new((*argv).cast()) };

    EXIT_NEED_DELAY.with_mut(|pending| *pending += 1);
    flush_stream(child, child.stdout());
    flush_stream(child, child.stderr());
    // SAFETY: a live child.
    unsafe { proc_close_streams(child.as_ptr()) };
    proc_close(child);
    EXIT_NEED_DELAY.with_mut(|pending| *pending -= 1);
}

/// Retry an exit that had to wait for [`proc_close_handles`] to finish.
///
/// # Safety
/// libuv's: `handle` is the main loop's `exit_delay_timer`.
unsafe extern "C" fn exit_delay_cb(_handle: *mut uv_timer_t) {
    let main = main_loop.ptr();
    // SAFETY: the main loop is live for as long as the editor is.
    unsafe { uv_timer_stop(&raw mut (*main).exit_delay_timer) };
    // SAFETY: as above; the timer carries the status this event replays.
    let status = unsafe { (*main).exit_delay_timer.data };
    let event = one_arg_event(Some(exit_event), status);
    // SAFETY: as above.
    unsafe { multiqueue_put_event((*main).fast_events, event) };
}

/// Exit the editor with the status packed into `argv[0]`.
///
/// # Safety
/// Queued by [`exit_on_closed_chan`] or [`exit_delay_cb`]: `argv[0]` carries
/// an exit status packed by [`pack_int`].
unsafe extern "C" fn exit_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise about the argv.
    let packed = unsafe { *argv };
    let status = unpack_int(packed);
    if EXIT_NEED_DELAY.get() != 0 {
        let main = main_loop.ptr();
        // The exit timer doubles as the carrier for the status.
        // SAFETY: the main loop is live for as long as the editor is.
        unsafe { (*main).exit_delay_timer.data = packed };
        // SAFETY: as above.
        unsafe { uv_timer_start(&raw mut (*main).exit_delay_timer, Some(exit_delay_cb), 0, 0) };
        return;
    }

    if !exiting.get() {
        if ui_client_channel_id.get() != 0 {
            ui_client_exit_status.set(status);
            // SAFETY: the editor's own exit path; it does not return.
            unsafe { os_exit(status) };
        } else {
            // The only other caller is `rpc_close`, which passes 0.
            debug_assert!(status == 0);
            // SAFETY: as above.
            unsafe { preserve_exit(ptr::null()) };
        }
    }
}

/// libuv told us the child exited.
///
/// There may still be output to read, but we are inside the libuv loop and
/// cannot poll for more from here — so the draining is queued as an event.
///
/// # Safety
/// `proc` is the live child whose exit callback fired.
unsafe fn on_proc_exit(proc: *mut Proc) {
    // SAFETY: the caller's promise.
    let child = unsafe { Child::new(proc) };
    // The stray "lu" is upstream's: the C source concatenated PRIu64 onto a
    // format that had already consumed the argument with %d.
    let fmt = c"child exited: pid=%d status=%dlu";
    let (pid, status) = (child.pid, child.status);
    // SAFETY: `logmsg_begin` takes the log's lock and hands back its handle.
    unsafe { logmsg!(LOGLVL_INF, c"on_proc_exit", 447, fmt, pid, status) };

    let queue = if child.events.is_null() {
        child.loop_events()
    } else {
        child.events
    };
    // SAFETY: the child's queue, and `proc_close_handles` wants the child.
    unsafe { create_event(queue, proc_close_handles, child.as_ptr().cast()) };
}

/// One of the child's streams finished closing.
///
/// # Safety
/// `data` is the `*mut Proc` [`proc_spawn`] stored in the stream.
unsafe fn on_proc_stream_close(_stream: *mut Stream, data: *mut c_void) {
    // SAFETY: the caller's promise.
    unsafe { decref(data.cast()) };
}

#[cfg(test)]
mod tests {
    use super::{Proc, as_exit_signal, remove_child};
    use crate::os::signal::{SIGHUP, SIGKILL, SIGTERM};
    use core::ptr;

    /// Distinct, never-dereferenced `*mut Proc` stand-ins. `remove_child`
    /// only ever compares them.
    fn token(n: usize) -> *mut Proc {
        ptr::without_provenance_mut(n * size_of::<usize>() + size_of::<usize>())
    }

    #[test]
    fn removing_a_child_keeps_the_others_in_order() {
        let mut children: Vec<*mut Proc> = (0..4).map(token).collect();
        remove_child(&mut children, token(1));
        assert_eq!(children, vec![token(0), token(2), token(3)]);
        remove_child(&mut children, token(3));
        assert_eq!(children, vec![token(0), token(2)]);
        remove_child(&mut children, token(0));
        assert_eq!(children, vec![token(2)]);
        remove_child(&mut children, token(2));
        assert!(children.is_empty());
    }

    #[test]
    #[should_panic(expected = "on the loop's child list")]
    fn removing_a_child_that_is_not_there_is_a_bug() {
        let mut children: Vec<*mut Proc> = vec![token(0)];
        remove_child(&mut children, token(1));
    }

    #[test]
    fn a_signal_number_fits_in_the_exit_signal_byte() {
        assert_eq!(as_exit_signal(SIGHUP), 1);
        assert_eq!(as_exit_signal(SIGKILL), 9);
        assert_eq!(as_exit_signal(SIGTERM), 15);
        assert_eq!(as_exit_signal(0), 0);
    }
}
