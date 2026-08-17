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
//! owner (a `Channel`, or a stack frame in `os/shell.rs`) does that.
//!
//! # Aliasing
//!
//! libuv and the stream layer both hold the address of a `Proc` (or of a
//! field inside it) in a `data` pointer for as long as the child lives, so a
//! `Proc` must not move once spawned, and the callbacks reached from
//! [`proc_close_handles`] may re-enter this module while a caller further up
//! the stack still holds a `*mut Proc` to the same child.

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
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_DBG, LOGLVL_INF, logmsg_c};
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
    LibuvProc, Loop, MultiQueue, Proc, ProcType, PtyProc, RStream, Stream, uv_handle_t,
    uv_stream_t, uv_timer_t,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// A child started through libuv's process API.
pub const kProcTypeUv: ProcType = 0;
/// A child started through `forkpty`.
pub const kProcTypePty: ProcType = 1;

/// How long a stopped child has to exit cleanly before it is killed. A pty
/// child is sent SIGTERM at this point (SIGHUP having already failed) and
/// gets another interval before SIGKILL.
const KILL_TIMEOUT_MS: u64 = 2000;

/// Fallback for the read-ahead limit when libuv will not report the socket
/// receive buffer size.
const ARENA_BLOCK_SIZE: c_int = 4096;

/// Set for the whole of [`proc_teardown`]. Relaxes the "a child is closed
/// exactly once" assertion, because a detached or pty child can die while
/// being torn down and be closed a second time from its exit callback.
static PROC_IS_TEARING_DOWN: GlobalCell<bool> = GlobalCell::new(false);

/// Non-zero while [`proc_close_handles`] is draining a child's output.
/// Exiting in the middle of that deadlocks, so [`exit_event`] reschedules
/// itself instead.
static EXIT_NEED_DELAY: GlobalCell<c_int> = GlobalCell::new(0);

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
pub unsafe fn proc_get_exepath(proc: *mut Proc) -> *const c_char {
    if (*proc).exepath.is_null() {
        *(*proc).argv as *const c_char
    } else {
        (*proc).exepath
    }
}

/// Start `proc`, optionally with each of the three standard streams piped.
///
/// Returns zero, or a negative error code — in which case `proc` has already
/// been closed and freed and its status set to -1.
pub unsafe fn proc_spawn(proc: *mut Proc, has_in: bool, has_out: bool, has_err: bool) -> c_int {
    // Forwarding stderr contradicts processing it internally.
    debug_assert!(!(has_err && (*proc).fwd_err));

    let streams = [
        (has_in, &raw mut (*proc).in_0),
        (has_out, &raw mut (*proc).out.s),
        (has_err, &raw mut (*proc).err.s),
    ];

    // A stream the caller did not ask for starts out closed; the rest get a
    // pipe handle now, which the spawn hooks below fill with a descriptor.
    for (wanted, stream) in streams {
        if wanted {
            uv_pipe_init(&raw mut (*(*proc).loop_0).uv, &raw mut (*stream).uv.pipe, 0);
        } else {
            (*stream).closed = true;
        }
    }

    let status = match (*proc).type_0 {
        kProcTypePty => pty_proc_spawn(proc as *mut PtyProc),
        _ => libuv_proc_spawn(proc as *mut LibuvProc),
    };

    if status != 0 {
        for (wanted, stream) in streams {
            if wanted {
                uv_close(&raw mut (*stream).uv.pipe as *mut uv_handle_t, None);
            }
        }
        if (*proc).type_0 == kProcTypeUv {
            // The process handle was never started, so it is dropped
            // directly rather than through `proc_close`, which would run the
            // close callbacks of a child that does not exist.
            uv_close(
                &raw mut (*(proc as *mut LibuvProc)).uv as *mut uv_handle_t,
                None,
            );
        } else {
            proc_close(proc);
        }
        proc_free(proc);
        (*proc).status = -1;
        return status;
    }

    // One reference per piped stream, plus one for the child itself.
    for (wanted, stream) in streams {
        if wanted {
            stream_init(
                ptr::null_mut(),
                stream,
                -1,
                &raw mut (*stream).uv.pipe as *mut uv_stream_t,
            );
            (*stream).internal_data = proc as *mut c_void;
            (*stream).internal_close_cb = Some(on_proc_stream_close);
            (*proc).refcount += 1;
        }
    }
    (*proc).internal_exit_cb = Some(on_proc_exit);
    (*proc).internal_close_cb = Some(decref);
    (*proc).refcount += 1;

    (*loop_children((*proc).loop_0)).push(proc);
    logmsg_c!(
        LOGLVL_DBG,
        ptr::null(),
        c"proc_spawn".as_ptr(),
        127,
        true,
        c"new: pid=%d exepath=[%s]".as_ptr(),
        (*proc).pid,
        proc_get_exepath(proc),
    );
    0
}

/// Stop every child and wait for the loop to be free of them.
///
/// Detached and pty children are not killed — their handles are closed and
/// they are left to run — but everything else is asked to terminate.
pub unsafe fn proc_teardown(uv_loop: *mut Loop) {
    PROC_IS_TEARING_DOWN.set(true);
    // Re-read the length each pass: closing a child's handles can run its
    // exit callback inline, which unlinks it from this very list.
    let mut i = 0;
    while i < (*loop_children(uv_loop)).len() {
        let proc = (&*loop_children(uv_loop))[i];
        if (*proc).detach || (*proc).type_0 == kProcTypePty {
            create_event((*uv_loop).events, proc_close_handles, proc as *mut c_void);
        } else {
            proc_stop(proc);
        }
        i += 1;
    }

    process_events_until(uv_loop, (*uv_loop).events, -1, || {
        (*loop_children(uv_loop)).is_empty() && multiqueue_empty((*uv_loop).events)
    });
    pty_proc_teardown(uv_loop);
}

pub unsafe fn proc_close_streams(proc: *mut Proc) {
    stream_may_close(&raw mut (*proc).in_0);
    rstream_may_close(&raw mut (*proc).out);
    rstream_may_close(&raw mut (*proc).err);
}

/// Wait for `proc` to finish, for at most `ms` milliseconds.
///
/// `ms` of 0 polls once and returns; -1 waits indefinitely. `events`, if
/// given, is drained instead of the child's own queue.
///
/// Returns the child's exit status, -1 if the wait timed out with the child
/// still running, or -2 if the user interrupted it.
pub unsafe fn proc_wait(proc: *mut Proc, ms: c_int, mut events: *mut MultiQueue) -> c_int {
    if (*proc).refcount == 0 {
        // Read the status before draining: an event may free the child.
        let status = (*proc).status;
        process_events((*proc).loop_0, (*proc).events, 0);
        return status;
    }

    if events.is_null() {
        events = (*proc).events;
    }

    // Hold a reference of our own so the exit callback cannot free the child
    // before its status has been read.
    (*proc).refcount += 1;
    process_events_until((*proc).loop_0, events, ms as i64, || {
        got_int.get() || (*proc).refcount == 1
    });

    // A user hitting CTRL-C is assumed not to like the current job.
    if got_int.get() {
        got_int.set(false);
        proc_stop(proc);
        if ms == -1 {
            // Returning is only safe once every handle is closed too.
            process_events_until((*proc).loop_0, events, -1, || (*proc).refcount == 1);
        } else {
            process_events((*proc).loop_0, events, 0);
        }
        (*proc).status = -2;
    }

    if (*proc).refcount == 1 {
        decref(proc);
        if !(*proc).events.is_null() {
            // `decref` queued the exit event; run it now.
            multiqueue_process_events((*proc).events);
        }
    } else {
        (*proc).refcount -= 1;
    }
    (*proc).status
}

/// Ask `proc` to terminate, and arm the timer that kills it if it does not.
pub unsafe fn proc_stop(proc: *mut Proc) {
    let exited = (*proc).status >= 0;
    if exited || (*proc).stopped_time != 0 {
        return;
    }
    (*proc).stopped_time = os_hrtime();

    if (*proc).type_0 == kProcTypePty {
        // Closing every stream is what sends SIGHUP to a pty child.
        (*proc).exit_signal = SIGHUP as u8;
        proc_close_streams(proc);
        pty_proc_close_master(proc as *mut PtyProc);
    } else {
        (*proc).exit_signal = SIGTERM as u8;
        os_proc_tree_kill((*proc).pid, SIGTERM);
    }

    uv_timer_start(
        &raw mut (*(*proc).loop_0).children_kill_timer,
        Some(children_kill_cb),
        KILL_TIMEOUT_MS,
        0,
    );
}

/// Release the resources the child itself owns. The `Proc` is the caller's.
pub unsafe fn proc_free(proc: *mut Proc) {
    if !(*proc).argv.is_null() {
        shell_free_argv((*proc).argv);
        (*proc).argv = ptr::null_mut();
    }
}

/// Self-exit, because the primary RPC channel was closed.
pub fn exit_on_closed_chan(status: c_int) {
    // SAFETY: `main_loop.fast_events` is live for as long as the editor is.
    unsafe {
        logmsg_c!(
            LOGLVL_DBG,
            ptr::null(),
            c"exit_on_closed_chan".as_ptr(),
            440,
            true,
            c"self-exit triggered by closed RPC channel...".as_ptr(),
        );
        multiqueue_put_event(
            (*main_loop.ptr()).fast_events,
            one_arg_event(
                Some(exit_event),
                ptr::with_exposed_provenance_mut(status as isize as usize),
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// The loop's list of live children
// ---------------------------------------------------------------------------

/// Remove `proc` from `loop->children`, preserving the order of the rest.
///
/// The borrow is momentary on purpose: the list is re-entered while a child is
/// being closed.
unsafe fn remove_child(uv_loop: *mut Loop, proc: *mut Proc) {
    let children = &mut *loop_children(uv_loop);
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
/// driven synchronously (`os/shell.rs` builds one), so the handler simply
/// runs on the spot.
unsafe fn create_event(
    queue: *mut MultiQueue,
    handler: unsafe extern "C" fn(*mut *mut c_void),
    arg: *mut c_void,
) {
    if queue.is_null() {
        let mut argv = [arg];
        handler(argv.as_mut_ptr());
    } else {
        multiqueue_put_event(queue, one_arg_event(Some(handler), arg));
    }
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------

/// The kill timer: SIGKILL anything that did not exit after [`proc_stop`].
///
/// A pty child gets SIGTERM first (SIGHUP having already been sent by closing
/// its streams) and the timer is restarted; `stopped_time` is set to
/// `u64::MAX` to record that.
unsafe extern "C" fn children_kill_cb(handle: *mut uv_timer_t) {
    let uv_loop = (*(*handle).loop_0).data as *mut Loop;
    let mut i = 0;
    while i < (*loop_children(uv_loop)).len() {
        let proc = (&*loop_children(uv_loop))[i];
        i += 1;
        let exited = (*proc).status >= 0;
        if exited || (*proc).stopped_time == 0 {
            continue;
        }
        let term_sent = (*proc).stopped_time == u64::MAX;
        if (*proc).type_0 != kProcTypePty || term_sent {
            (*proc).exit_signal = SIGKILL as u8;
            os_proc_tree_kill((*proc).pid, SIGKILL);
        } else {
            (*proc).exit_signal = SIGTERM as u8;
            os_proc_tree_kill((*proc).pid, SIGTERM);
            (*proc).stopped_time = u64::MAX;
            uv_timer_start(
                &raw mut (*(*proc).loop_0).children_kill_timer,
                Some(children_kill_cb),
                KILL_TIMEOUT_MS,
                0,
            );
        }
    }
}

/// The last thing that happens to a child: hand it to its owner's callback,
/// which is responsible for freeing it.
unsafe extern "C" fn proc_close_event(argv: *mut *mut c_void) {
    let proc = *argv as *mut Proc;
    if let Some(notify) = (*proc).cb {
        notify(proc, (*proc).status, (*proc).data);
    } else {
        proc_free(proc);
    }
}

/// Drop one reference to `proc`; unlink and report it when the last one goes.
unsafe extern "C" fn decref(proc: *mut Proc) {
    (*proc).refcount -= 1;
    if (*proc).refcount != 0 {
        return;
    }
    remove_child((*proc).loop_0, proc);
    create_event((*proc).events, proc_close_event, proc as *mut c_void);
}

/// Close the child's own handle (the process or the pty master).
unsafe fn proc_close(proc: *mut Proc) {
    if PROC_IS_TEARING_DOWN.get()
        && (*proc).closed
        && ((*proc).detach || (*proc).type_0 == kProcTypePty)
    {
        // A detached or pty child that dies while being torn down gets here
        // twice: once from the teardown, once from its own exit callback.
        return;
    }
    debug_assert!(!(*proc).closed);
    (*proc).closed = true;

    if (*proc).detach && (*proc).type_0 == kProcTypeUv {
        // Let the loop exit without waiting for a child we no longer own.
        uv_unref(&raw mut (*(proc as *mut LibuvProc)).uv as *mut uv_handle_t);
    }

    if (*proc).type_0 == kProcTypePty {
        pty_proc_close(proc as *mut PtyProc);
    } else {
        libuv_proc_close(proc as *mut LibuvProc);
    }
}

/// Read whatever a dead child left in one of its output streams.
///
/// The read is bounded so that a child which keeps its output open — or a
/// grandchild that inherited it — cannot block teardown forever. The bound is
/// the system receive buffer size on top of what has already been read, which
/// is the most a terminated process can still have queued.
unsafe fn flush_stream(proc: *mut Proc, stream: *mut RStream) {
    if stream.is_null() || (*stream).s.closed {
        return;
    }

    let mut max_bytes = usize::MAX;
    // A pty master is exempt until teardown: on Linux it can hold far more
    // than one system buffer's worth. #3030
    if (*proc).type_0 != kProcTypePty || PROC_IS_TEARING_DOWN.get() {
        let mut system_buffer_size: c_int = 0;
        // Every arm of the `uv` union shares an address.
        let err = uv_recv_buffer_size(
            &raw mut (*stream).s.uv as *mut uv_handle_t,
            &raw mut system_buffer_size,
        );
        if err != 0 {
            system_buffer_size = ARENA_BLOCK_SIZE;
        }
        max_bytes = (*stream).num_bytes + system_buffer_size as usize;
    }

    // Not immutable: the events processed in the body reach the stream's own
    // callbacks, which are what advance `num_bytes` and can close it.
    #[allow(clippy::while_immutable_condition)]
    while !(*stream).s.closed && (*stream).num_bytes < max_bytes {
        let num_bytes = (*stream).num_bytes;

        if (*proc).type_0 == kProcTypePty && !(*stream).did_eof {
            pty_proc_flush_master(proc as *mut PtyProc);
        }
        loop_poll_events((*proc).loop_0, 0);
        if !(*stream).s.events.is_null() {
            multiqueue_process_events((*stream).s.events);
        }

        if num_bytes != (*stream).num_bytes {
            continue;
        }
        // Nothing arrived, so the stream is empty. A child that keeps it open
        // would otherwise deny the reader its end-of-file.
        if let Some(read) = (*stream).read_cb {
            if !(*stream).did_eof {
                read(stream, (*stream).buffer, 0, (*stream).s.cb_data, true);
            }
        }
        break;
    }
}

/// Drain and close everything belonging to a child that has exited.
unsafe extern "C" fn proc_close_handles(argv: *mut *mut c_void) {
    let proc = *argv as *mut Proc;

    *EXIT_NEED_DELAY.ptr() += 1;
    flush_stream(proc, &raw mut (*proc).out);
    flush_stream(proc, &raw mut (*proc).err);
    proc_close_streams(proc);
    proc_close(proc);
    *EXIT_NEED_DELAY.ptr() -= 1;
}

/// Retry an exit that had to wait for [`proc_close_handles`] to finish.
unsafe extern "C" fn exit_delay_cb(_handle: *mut uv_timer_t) {
    uv_timer_stop(&raw mut (*main_loop.ptr()).exit_delay_timer);
    multiqueue_put_event(
        (*main_loop.ptr()).fast_events,
        one_arg_event(Some(exit_event), (*main_loop.ptr()).exit_delay_timer.data),
    );
}

/// Exit the editor with the status packed into `argv[0]`.
unsafe extern "C" fn exit_event(argv: *mut *mut c_void) {
    let status = (*argv).expose_provenance() as c_int;
    if EXIT_NEED_DELAY.get() != 0 {
        // The exit timer doubles as the carrier for the status.
        (*main_loop.ptr()).exit_delay_timer.data = *argv;
        uv_timer_start(
            &raw mut (*main_loop.ptr()).exit_delay_timer,
            Some(exit_delay_cb),
            0,
            0,
        );
        return;
    }

    if !exiting.get() {
        if ui_client_channel_id.get() != 0 {
            ui_client_exit_status.set(status);
            os_exit(status);
        } else {
            // The only other caller is `rpc_close`, which passes 0.
            debug_assert!(status == 0);
            preserve_exit(ptr::null());
        }
    }
}

/// libuv told us the child exited.
///
/// There may still be output to read, but we are inside the libuv loop and
/// cannot poll for more from here — so the draining is queued as an event.
unsafe extern "C" fn on_proc_exit(proc: *mut Proc) {
    let uv_loop = (*proc).loop_0;
    logmsg_c!(
        LOGLVL_INF,
        ptr::null(),
        c"on_proc_exit".as_ptr(),
        447,
        true,
        // The stray "lu" is upstream's: the C source concatenated PRIu64
        // onto a format that had already consumed the argument with %d.
        c"child exited: pid=%d status=%dlu".as_ptr(),
        (*proc).pid,
        (*proc).status,
    );
    let queue = if (*proc).events.is_null() {
        (*uv_loop).events
    } else {
        (*proc).events
    };
    create_event(queue, proc_close_handles, proc as *mut c_void);
}

/// One of the child's streams finished closing.
unsafe extern "C" fn on_proc_stream_close(_stream: *mut Stream, data: *mut c_void) {
    decref(data as *mut Proc);
}
