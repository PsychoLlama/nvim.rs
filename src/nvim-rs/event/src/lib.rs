//! Event loop and async I/O wrappers for Neovim
//!
//! This module provides Rust wrappers around Neovim's libuv-based event system.
//! The approach is to keep using libuv but provide safe Rust abstractions.
//!
//! # Design Principles
//!
//! 1. **Wrap, don't replace**: Keep the existing libuv infrastructure, wrap with Rust
//! 2. **Opaque handles**: Use opaque handle types (like `WinHandle`, `BufHandle`)
//! 3. **Incremental**: Start with simple wrappers, expand over time
//!
//! # Architecture
//!
//! The C event system uses:
//! - `Loop` struct containing `uv_loop_t` and `MultiQueue` instances
//! - `MultiQueue` for hierarchical event queuing
//! - `TimeWatcher`, `SignalWatcher` etc. wrapping libuv handles
//! - `Event` struct with handler callback and arguments
//!
//! We provide:
//! - `LoopHandle` - opaque pointer to `Loop`
//! - `MultiQueueHandle` - opaque pointer to `MultiQueue`
//! - `TimeWatcherHandle` - opaque pointer to `TimeWatcher`
//! - Event creation and handling functions

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;

/// Maximum number of arguments in an Event handler
pub const EVENT_HANDLER_MAX_ARGC: usize = 10;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a Loop (event loop context)
///
/// This wraps a C `Loop` pointer. The Loop contains:
/// - `uv_loop_t` - the libuv event loop
/// - `MultiQueue` instances for events
/// - Various timer and signal handles
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopHandle(*mut std::ffi::c_void);

impl LoopHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a MultiQueue
///
/// MultiQueue is a hierarchical event queue system supporting:
/// - Parent-child relationships for selective event processing
/// - Link nodes for event propagation between queues
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiQueueHandle(*mut std::ffi::c_void);

impl MultiQueueHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a TimeWatcher
///
/// TimeWatcher wraps a libuv timer handle for delayed/repeated callbacks.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWatcherHandle(*mut std::ffi::c_void);

impl TimeWatcherHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Proc (OS process)
///
/// Proc represents a child process managed by the event loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcHandle(*mut std::ffi::c_void);

impl ProcHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Stream (I/O stream)
///
/// Stream represents an I/O stream (pipe, tcp, file, etc.)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamHandle(*mut std::ffi::c_void);

impl StreamHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to an RStream (read stream)
///
/// RStream wraps a Stream for reading data asynchronously.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RStreamHandle(*mut std::ffi::c_void);

impl RStreamHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a SignalWatcher
///
/// SignalWatcher wraps a libuv signal handle for signal handling.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalWatcherHandle(*mut std::ffi::c_void);

impl SignalWatcherHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a SocketWatcher
///
/// SocketWatcher wraps TCP/pipe sockets for accepting connections.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketWatcherHandle(*mut std::ffi::c_void);

impl SocketWatcherHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Event Structure
// =============================================================================

/// Event handler callback type
///
/// Takes an array of void pointers as arguments.
pub type ArgvCallback = Option<unsafe extern "C" fn(*mut *mut std::ffi::c_void)>;

/// Event structure matching C `Event` from event/defs.h
///
/// Events are the basic unit of async work in Neovim. They consist of:
/// - A handler callback function
/// - Up to 10 void pointer arguments
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Event {
    /// Handler function to call when event is processed
    pub handler: ArgvCallback,
    /// Arguments to pass to handler
    pub argv: [*mut std::ffi::c_void; EVENT_HANDLER_MAX_ARGC],
}

impl Default for Event {
    fn default() -> Self {
        Self {
            handler: None,
            argv: [std::ptr::null_mut(); EVENT_HANDLER_MAX_ARGC],
        }
    }
}

impl Event {
    /// Create a nil/empty event
    #[must_use]
    pub const fn nil() -> Self {
        Self {
            handler: None,
            argv: [std::ptr::null_mut(); EVENT_HANDLER_MAX_ARGC],
        }
    }

    /// Check if event handler is set
    #[must_use]
    pub const fn is_nil(&self) -> bool {
        self.handler.is_none()
    }
}

/// Check if an event is nil (has no handler)
///
/// Returns 1 if nil, 0 if not nil.
///
/// # Safety
///
/// `event` must be a valid pointer to an Event struct
#[no_mangle]
pub unsafe extern "C" fn rs_event_is_nil(event: *const Event) -> c_int {
    if event.is_null() {
        return 1;
    }
    c_int::from((*event).is_nil())
}

/// Create a nil event and write it to the output pointer
///
/// # Safety
///
/// `out` must be a valid pointer to Event memory
#[no_mangle]
pub unsafe extern "C" fn rs_event_nil(out: *mut Event) {
    if !out.is_null() {
        *out = Event::nil();
    }
}

// =============================================================================
// C Accessor Functions (defined in event/defs.c or similar)
// =============================================================================

/// QUEUE structure matching Neovim's lib/queue_defs.h
/// Named EventQueue to avoid collision with Queue from nvim-collections
#[repr(C)]
pub struct EventQueue {
    pub next: *mut EventQueue,
    pub prev: *mut EventQueue,
}

extern "C" {
    // Loop accessors
    fn nvim_loop_get_events(loop_: LoopHandle) -> MultiQueueHandle;
    fn nvim_loop_get_fast_events(loop_: LoopHandle) -> MultiQueueHandle;
    fn nvim_loop_get_thread_events(loop_: LoopHandle) -> MultiQueueHandle;
    fn nvim_loop_is_closing(loop_: LoopHandle) -> c_int;
    fn nvim_loop_get_recursive(loop_: LoopHandle) -> c_int;
    fn nvim_loop_children_count(loop_: LoopHandle) -> usize;

    // MultiQueue accessors
    fn nvim_multiqueue_empty(mq: MultiQueueHandle) -> c_int;
    fn nvim_multiqueue_size(mq: MultiQueueHandle) -> usize;
    fn nvim_multiqueue_get_headtail(mq: MultiQueueHandle) -> *mut EventQueue;
    fn nvim_multiqueue_get_size_field(mq: MultiQueueHandle) -> usize;
    fn nvim_multiqueue_has_parent(mq: MultiQueueHandle) -> c_int;

    // TimeWatcher accessors
    fn nvim_timewatcher_get_data(tw: TimeWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_timewatcher_get_events(tw: TimeWatcherHandle) -> MultiQueueHandle;
    fn nvim_timewatcher_is_blockable(tw: TimeWatcherHandle) -> c_int;

    // Proc accessors
    fn nvim_proc_get_status(proc: ProcHandle) -> c_int;
    fn nvim_proc_set_status(proc: ProcHandle, status: c_int);
    fn nvim_proc_get_stopped_time(proc: ProcHandle) -> u64;
    fn nvim_proc_get_pid(proc: ProcHandle) -> c_int;
    fn nvim_proc_set_pid(proc: ProcHandle, pid: c_int);
    fn nvim_proc_get_refcount(proc: ProcHandle) -> c_int;
    fn nvim_proc_is_closed(proc: ProcHandle) -> c_int;
    fn nvim_proc_get_loop(proc: ProcHandle) -> LoopHandle;
    fn nvim_proc_get_type(proc: ProcHandle) -> c_int;
    fn nvim_proc_get_detach(proc: ProcHandle) -> c_int;
    fn nvim_proc_set_detach(proc: ProcHandle, detach: c_int);
    fn nvim_proc_get_events(proc: ProcHandle) -> MultiQueueHandle;
    fn nvim_proc_set_events(proc: ProcHandle, events: MultiQueueHandle);
    fn nvim_proc_set_closed(proc: ProcHandle, closed: c_int);
    fn nvim_proc_incref(proc: ProcHandle);
    fn nvim_proc_decref(proc: ProcHandle) -> c_int;
    fn nvim_proc_get_argv(proc: ProcHandle) -> *mut *mut std::ffi::c_char;
    fn nvim_proc_set_argv(proc: ProcHandle, argv: *mut *mut std::ffi::c_char);
    fn nvim_proc_get_exepath(proc: ProcHandle) -> *const std::ffi::c_char;
    fn nvim_proc_set_exepath(proc: ProcHandle, exepath: *const std::ffi::c_char);
    fn nvim_proc_get_cwd(proc: ProcHandle) -> *const std::ffi::c_char;
    fn nvim_proc_set_cwd(proc: ProcHandle, cwd: *const std::ffi::c_char);
    fn nvim_proc_get_env(proc: ProcHandle) -> *mut std::ffi::c_void;
    fn nvim_proc_set_env(proc: ProcHandle, env: *mut std::ffi::c_void);
    fn nvim_proc_set_stopped_time(proc: ProcHandle, stopped_time: u64);
    fn nvim_proc_get_exit_signal(proc: ProcHandle) -> u8;
    fn nvim_proc_set_exit_signal(proc: ProcHandle, exit_signal: u8);
    fn nvim_proc_get_fwd_err(proc: ProcHandle) -> c_int;
    fn nvim_proc_set_fwd_err(proc: ProcHandle, fwd_err: c_int);
    fn nvim_proc_get_overlapped(proc: ProcHandle) -> c_int;
    fn nvim_proc_set_overlapped(proc: ProcHandle, overlapped: c_int);
    // Callback accessors - use void* for FFI compatibility
    fn nvim_proc_get_cb(proc: ProcHandle) -> *mut std::ffi::c_void;
    fn nvim_proc_set_cb(proc: ProcHandle, cb: *mut std::ffi::c_void);
    fn nvim_proc_get_internal_exit_cb(proc: ProcHandle) -> *mut std::ffi::c_void;
    fn nvim_proc_set_internal_exit_cb(proc: ProcHandle, cb: *mut std::ffi::c_void);
    fn nvim_proc_get_internal_close_cb(proc: ProcHandle) -> *mut std::ffi::c_void;
    fn nvim_proc_set_internal_close_cb(proc: ProcHandle, cb: *mut std::ffi::c_void);
    fn nvim_proc_call_cb(proc: ProcHandle, status: c_int, data: *mut std::ffi::c_void);
    fn nvim_proc_call_internal_exit_cb(proc: ProcHandle);
    fn nvim_proc_call_internal_close_cb(proc: ProcHandle);

    // Stream accessors
    fn nvim_stream_is_closed(stream: StreamHandle) -> c_int;
    fn nvim_stream_pending_reqs(stream: StreamHandle) -> usize;
    fn nvim_stream_pending_reqs_inc(stream: StreamHandle);
    fn nvim_stream_pending_reqs_dec(stream: StreamHandle);
    fn nvim_stream_get_fd(stream: StreamHandle) -> c_int;
    fn nvim_stream_get_curmem(stream: StreamHandle) -> usize;
    fn nvim_stream_get_maxmem(stream: StreamHandle) -> usize;
    fn nvim_stream_get_events(stream: StreamHandle) -> MultiQueueHandle;
    fn nvim_stream_set_closed(stream: StreamHandle, closed: c_int);
    fn nvim_stream_get_pending_reqs(stream: StreamHandle) -> usize;
    fn nvim_stream_set_maxmem(stream: StreamHandle, maxmem: usize);
    fn nvim_stream_set_curmem(stream: StreamHandle, curmem: usize);
    fn nvim_stream_curmem_add(stream: StreamHandle, amount: usize);
    fn nvim_stream_curmem_sub(stream: StreamHandle, amount: usize);
    fn nvim_stream_get_write_cb(stream: StreamHandle) -> *mut std::ffi::c_void;
    fn nvim_stream_set_write_cb(stream: StreamHandle, cb: *mut std::ffi::c_void);
    fn nvim_stream_call_write_cb(stream: StreamHandle, data: *mut std::ffi::c_void, status: c_int);

    // RStream accessors
    fn nvim_rstream_did_eof(stream: RStreamHandle) -> c_int;
    fn nvim_rstream_want_read(stream: RStreamHandle) -> c_int;
    fn nvim_rstream_num_bytes(stream: RStreamHandle) -> usize;
    fn nvim_rstream_available(stream: RStreamHandle) -> usize;
    fn nvim_rstream_get_stream(stream: RStreamHandle) -> StreamHandle;

    // SignalWatcher accessors
    fn nvim_signal_watcher_get_signum(watcher: SignalWatcherHandle) -> c_int;
    fn nvim_signal_watcher_get_events(watcher: SignalWatcherHandle) -> MultiQueueHandle;
    fn nvim_signal_watcher_get_data(watcher: SignalWatcherHandle) -> *mut std::ffi::c_void;

    // SocketWatcher accessors
    fn nvim_socket_watcher_get_addr(watcher: SocketWatcherHandle) -> *const std::ffi::c_char;
    fn nvim_socket_watcher_get_events(watcher: SocketWatcherHandle) -> MultiQueueHandle;
    fn nvim_socket_watcher_get_data(watcher: SocketWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_socket_watcher_is_tcp(watcher: SocketWatcherHandle) -> c_int;
}

// =============================================================================
// Rust Wrapper Functions
// =============================================================================

/// Check if the event loop is closing
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_is_closing(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 1;
    }
    nvim_loop_is_closing(loop_)
}

/// Get the events queue from a Loop
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_get_events(loop_: LoopHandle) -> MultiQueueHandle {
    if loop_.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_loop_get_events(loop_)
}

/// Get the fast_events queue from a Loop
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_get_fast_events(loop_: LoopHandle) -> MultiQueueHandle {
    if loop_.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_loop_get_fast_events(loop_)
}

/// Get the thread_events queue from a Loop
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_get_thread_events(loop_: LoopHandle) -> MultiQueueHandle {
    if loop_.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_loop_get_thread_events(loop_)
}

/// Get the recursive count from a Loop
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_get_recursive(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 0;
    }
    nvim_loop_get_recursive(loop_)
}

/// Get the number of children processes from a Loop
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_children_count(loop_: LoopHandle) -> usize {
    if loop_.is_null() {
        return 0;
    }
    nvim_loop_children_count(loop_)
}

/// Check if a Loop's events queue is empty
///
/// Convenience function combining rs_loop_get_events and rs_multiqueue_empty.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_events_empty(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 1;
    }
    let events = nvim_loop_get_events(loop_);
    if events.is_null() {
        return 1;
    }
    rs_multiqueue_empty(events)
}

/// Check if a Loop has pending events (events queue not empty)
///
/// Convenience function that returns true if there are events to process.
/// Matches the pattern: `!multiqueue_empty(loop->events)`
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_has_pending_events(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 0;
    }
    c_int::from(rs_loop_events_empty(loop_) == 0)
}

/// Check if a MultiQueue is empty (pure Rust implementation)
///
/// Uses the headtail accessor and checks if the queue is self-referential.
///
/// # Safety
///
/// `mq` must be a valid MultiQueue handle
#[no_mangle]
pub unsafe extern "C" fn rs_multiqueue_empty(mq: MultiQueueHandle) -> c_int {
    if mq.is_null() {
        return 1;
    }
    let headtail = nvim_multiqueue_get_headtail(mq);
    if headtail.is_null() {
        return 1;
    }
    // A queue is empty when it points to itself (circular reference to self)
    c_int::from(headtail == (*headtail).next)
}

/// Get the size of a MultiQueue (pure Rust implementation)
///
/// Directly reads the size field via accessor.
///
/// # Safety
///
/// `mq` must be a valid MultiQueue handle
#[no_mangle]
pub unsafe extern "C" fn rs_multiqueue_size(mq: MultiQueueHandle) -> usize {
    if mq.is_null() {
        return 0;
    }
    nvim_multiqueue_get_size_field(mq)
}

/// Check if a MultiQueue has a parent
///
/// Returns 1 if the queue has a parent, 0 otherwise.
///
/// # Safety
///
/// `mq` must be a valid MultiQueue handle
#[no_mangle]
pub unsafe extern "C" fn rs_multiqueue_has_parent(mq: MultiQueueHandle) -> c_int {
    if mq.is_null() {
        return 0;
    }
    nvim_multiqueue_has_parent(mq)
}

/// Check if there are pending events in a queue
///
/// Combines null check with empty check: returns true if queue exists and has events.
/// This matches the C pattern: `events && !multiqueue_empty(events)`
///
/// Returns 1 if there are pending events, 0 otherwise.
///
/// # Safety
///
/// `mq` may be null (will return 0 if null)
#[no_mangle]
pub unsafe extern "C" fn rs_pending_events(mq: MultiQueueHandle) -> c_int {
    if mq.is_null() {
        return 0;
    }
    c_int::from(rs_multiqueue_empty(mq) == 0)
}

/// Get the size of thread_events from a Loop (pure Rust implementation)
///
/// Combines rs_loop_get_thread_events and rs_multiqueue_size.
/// Note: This does NOT handle locking - caller must hold the mutex.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle. Caller must ensure thread safety.
#[no_mangle]
pub unsafe extern "C" fn rs_loop_thread_events_size(loop_: LoopHandle) -> usize {
    if loop_.is_null() {
        return 0;
    }
    let thread_events = nvim_loop_get_thread_events(loop_);
    if thread_events.is_null() {
        return 0;
    }
    nvim_multiqueue_get_size_field(thread_events)
}

/// Check if TimeWatcher event queue is empty
///
/// Used in time_watcher_cb to check if blockable watcher should skip creating event.
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_events_pending(tw: TimeWatcherHandle) -> c_int {
    if tw.is_null() {
        return 0;
    }
    let events = nvim_timewatcher_get_events(tw);
    if events.is_null() {
        return 0;
    }
    // Events are pending if queue is NOT empty
    c_int::from(nvim_multiqueue_empty(events) == 0)
}

/// Check if TimeWatcher is blockable and has pending events
///
/// This implements the check from time_watcher_cb:
/// `if (watcher->blockable && !multiqueue_empty(watcher->events))`
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_should_skip(tw: TimeWatcherHandle) -> c_int {
    if tw.is_null() {
        return 0;
    }
    let blockable = nvim_timewatcher_is_blockable(tw);
    if blockable == 0 {
        return 0;
    }
    rs_timewatcher_events_pending(tw)
}

/// Check if a process has stopped (exited or stopped_time != 0)
///
/// This implements the logic from proc.h proc_is_stopped():
/// ```c
/// bool exited = (proc->status >= 0);
/// return exited || (proc->stopped_time != 0);
/// ```
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_stopped(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    let status = nvim_proc_get_status(proc);
    let stopped_time = nvim_proc_get_stopped_time(proc);
    let exited = status >= 0;
    c_int::from(exited || stopped_time != 0)
}

/// Get the process ID from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_pid(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return -1;
    }
    nvim_proc_get_pid(proc)
}

/// Set the pid on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_pid(proc: ProcHandle, pid: c_int) {
    if !proc.is_null() {
        nvim_proc_set_pid(proc, pid);
    }
}

/// Get the reference count from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_refcount(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_refcount(proc)
}

/// Get the status from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_status(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return -1;
    }
    nvim_proc_get_status(proc)
}

/// Check if a process is closed
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_closed(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 1;
    }
    nvim_proc_is_closed(proc)
}

/// Get the stopped_time from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_stopped_time(proc: ProcHandle) -> u64 {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_stopped_time(proc)
}

/// Set the status field of a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_status(proc: ProcHandle, status: c_int) {
    if !proc.is_null() {
        nvim_proc_set_status(proc, status);
    }
}

/// Get the loop from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_loop(proc: ProcHandle) -> LoopHandle {
    if proc.is_null() {
        return LoopHandle::null();
    }
    nvim_proc_get_loop(proc)
}

/// Get the type from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_type(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return -1;
    }
    nvim_proc_get_type(proc)
}

/// Get the detach flag from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_detach(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_detach(proc)
}

/// Set the detach flag on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_detach(proc: ProcHandle, detach: c_int) {
    if !proc.is_null() {
        nvim_proc_set_detach(proc, detach);
    }
}

/// Get the events queue from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_events(proc: ProcHandle) -> MultiQueueHandle {
    if proc.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_proc_get_events(proc)
}

/// Set the events queue on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_events(proc: ProcHandle, events: MultiQueueHandle) {
    if !proc.is_null() {
        nvim_proc_set_events(proc, events);
    }
}

/// Set closed field of a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_closed(proc: ProcHandle, closed: c_int) {
    if !proc.is_null() {
        nvim_proc_set_closed(proc, closed);
    }
}

/// Increment the refcount of a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_incref(proc: ProcHandle) {
    if !proc.is_null() {
        nvim_proc_incref(proc);
    }
}

/// Decrement the refcount of a Proc and return new value
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_decref(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_decref(proc)
}

/// Get the argv from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_argv(proc: ProcHandle) -> *mut *mut std::ffi::c_char {
    if proc.is_null() {
        return std::ptr::null_mut();
    }
    nvim_proc_get_argv(proc)
}

/// Set the argv on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_argv(proc: ProcHandle, argv: *mut *mut std::ffi::c_char) {
    if !proc.is_null() {
        nvim_proc_set_argv(proc, argv);
    }
}

/// Get the exepath from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_exepath_raw(proc: ProcHandle) -> *const std::ffi::c_char {
    if proc.is_null() {
        return std::ptr::null();
    }
    nvim_proc_get_exepath(proc)
}

/// Set the exepath on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_exepath(proc: ProcHandle, exepath: *const std::ffi::c_char) {
    if !proc.is_null() {
        nvim_proc_set_exepath(proc, exepath);
    }
}

/// Get the cwd from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_cwd(proc: ProcHandle) -> *const std::ffi::c_char {
    if proc.is_null() {
        return std::ptr::null();
    }
    nvim_proc_get_cwd(proc)
}

/// Set the cwd on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_cwd(proc: ProcHandle, cwd: *const std::ffi::c_char) {
    if !proc.is_null() {
        nvim_proc_set_cwd(proc, cwd);
    }
}

/// Get the env from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_env(proc: ProcHandle) -> *mut std::ffi::c_void {
    if proc.is_null() {
        return std::ptr::null_mut();
    }
    nvim_proc_get_env(proc)
}

/// Set the env on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_env(proc: ProcHandle, env: *mut std::ffi::c_void) {
    if !proc.is_null() {
        nvim_proc_set_env(proc, env);
    }
}

/// Set the stopped_time on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_stopped_time(proc: ProcHandle, stopped_time: u64) {
    if !proc.is_null() {
        nvim_proc_set_stopped_time(proc, stopped_time);
    }
}

/// Get the exit_signal from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_exit_signal(proc: ProcHandle) -> u8 {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_exit_signal(proc)
}

/// Set the exit_signal on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_exit_signal(proc: ProcHandle, exit_signal: u8) {
    if !proc.is_null() {
        nvim_proc_set_exit_signal(proc, exit_signal);
    }
}

/// Get the fwd_err flag from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_fwd_err(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_fwd_err(proc)
}

/// Set the fwd_err flag on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_fwd_err(proc: ProcHandle, fwd_err: c_int) {
    if !proc.is_null() {
        nvim_proc_set_fwd_err(proc, fwd_err);
    }
}

/// Get the overlapped flag from a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_overlapped(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_overlapped(proc)
}

/// Set the overlapped flag on a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_overlapped(proc: ProcHandle, overlapped: c_int) {
    if !proc.is_null() {
        nvim_proc_set_overlapped(proc, overlapped);
    }
}

/// Get the cb (exit callback) field of a Proc (as void* for FFI)
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_cb(proc: ProcHandle) -> *mut std::ffi::c_void {
    if proc.is_null() {
        return std::ptr::null_mut();
    }
    nvim_proc_get_cb(proc)
}

/// Set the cb (exit callback) field of a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_cb(proc: ProcHandle, cb: *mut std::ffi::c_void) {
    if !proc.is_null() {
        nvim_proc_set_cb(proc, cb);
    }
}

/// Get the internal_exit_cb field of a Proc (as void* for FFI)
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_internal_exit_cb(proc: ProcHandle) -> *mut std::ffi::c_void {
    if proc.is_null() {
        return std::ptr::null_mut();
    }
    nvim_proc_get_internal_exit_cb(proc)
}

/// Set the internal_exit_cb field of a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_internal_exit_cb(proc: ProcHandle, cb: *mut std::ffi::c_void) {
    if !proc.is_null() {
        nvim_proc_set_internal_exit_cb(proc, cb);
    }
}

/// Get the internal_close_cb field of a Proc (as void* for FFI)
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_get_internal_close_cb(proc: ProcHandle) -> *mut std::ffi::c_void {
    if proc.is_null() {
        return std::ptr::null_mut();
    }
    nvim_proc_get_internal_close_cb(proc)
}

/// Set the internal_close_cb field of a Proc
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_set_internal_close_cb(proc: ProcHandle, cb: *mut std::ffi::c_void) {
    if !proc.is_null() {
        nvim_proc_set_internal_close_cb(proc, cb);
    }
}

/// Call proc->cb if set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_call_cb(
    proc: ProcHandle,
    status: c_int,
    data: *mut std::ffi::c_void,
) {
    if !proc.is_null() {
        nvim_proc_call_cb(proc, status, data);
    }
}

/// Call proc->internal_exit_cb if set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_call_internal_exit_cb(proc: ProcHandle) {
    if !proc.is_null() {
        nvim_proc_call_internal_exit_cb(proc);
    }
}

/// Call proc->internal_close_cb if set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_call_internal_close_cb(proc: ProcHandle) {
    if !proc.is_null() {
        nvim_proc_call_internal_close_cb(proc);
    }
}

/// Check if a Stream is closed
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_is_closed(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 1;
    }
    nvim_stream_is_closed(stream)
}

/// Get the pending requests count from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_pending_reqs(stream: StreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    nvim_stream_pending_reqs(stream)
}

/// Increment the pending requests count for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_pending_reqs_inc(stream: StreamHandle) {
    if !stream.is_null() {
        nvim_stream_pending_reqs_inc(stream);
    }
}

/// Decrement the pending requests count for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_pending_reqs_dec(stream: StreamHandle) {
    if !stream.is_null() {
        nvim_stream_pending_reqs_dec(stream);
    }
}

/// Get the file descriptor from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_fd(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return -1;
    }
    nvim_stream_get_fd(stream)
}

/// Get the current memory usage from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_curmem(stream: StreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    nvim_stream_get_curmem(stream)
}

/// Get the maximum memory limit from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_maxmem(stream: StreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    nvim_stream_get_maxmem(stream)
}

/// Get the events queue from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_events(stream: StreamHandle) -> MultiQueueHandle {
    if stream.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_stream_get_events(stream)
}

/// Set the closed flag for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_closed(stream: StreamHandle, closed: c_int) {
    if !stream.is_null() {
        nvim_stream_set_closed(stream, closed);
    }
}

/// Get the pending requests count for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_pending_reqs(stream: StreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    nvim_stream_get_pending_reqs(stream)
}

/// Set the maxmem field for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_maxmem(stream: StreamHandle, maxmem: usize) {
    if !stream.is_null() {
        nvim_stream_set_maxmem(stream, maxmem);
    }
}

/// Set the curmem field for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_curmem(stream: StreamHandle, curmem: usize) {
    if !stream.is_null() {
        nvim_stream_set_curmem(stream, curmem);
    }
}

/// Add to the curmem field for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_curmem_add(stream: StreamHandle, amount: usize) {
    if !stream.is_null() {
        nvim_stream_curmem_add(stream, amount);
    }
}

/// Subtract from the curmem field for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_curmem_sub(stream: StreamHandle, amount: usize) {
    if !stream.is_null() {
        nvim_stream_curmem_sub(stream, amount);
    }
}

/// Get the write callback from a Stream (as void* for FFI)
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_write_cb(stream: StreamHandle) -> *mut std::ffi::c_void {
    if stream.is_null() {
        return std::ptr::null_mut();
    }
    nvim_stream_get_write_cb(stream)
}

/// Set the write callback for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_write_cb(stream: StreamHandle, cb: *mut std::ffi::c_void) {
    if !stream.is_null() {
        nvim_stream_set_write_cb(stream, cb);
    }
}

/// Call the write callback if set
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_call_write_cb(
    stream: StreamHandle,
    data: *mut std::ffi::c_void,
    status: c_int,
) {
    if !stream.is_null() {
        nvim_stream_call_write_cb(stream, data, status);
    }
}

/// Check if an RStream has reached EOF
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_did_eof(stream: RStreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    nvim_rstream_did_eof(stream)
}

/// Check if an RStream wants to read
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_want_read(stream: RStreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    nvim_rstream_want_read(stream)
}

/// Get the number of bytes read by RStream
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_num_bytes(stream: RStreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    nvim_rstream_num_bytes(stream)
}

/// Get the number of bytes available in RStream buffer
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_available(stream: RStreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    nvim_rstream_available(stream)
}

/// Get the underlying Stream from RStream
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_get_stream(stream: RStreamHandle) -> StreamHandle {
    if stream.is_null() {
        return StreamHandle::null();
    }
    nvim_rstream_get_stream(stream)
}

/// Check if an RStream is closed (via its underlying Stream)
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_is_closed(stream: RStreamHandle) -> c_int {
    if stream.is_null() {
        return 1; // Treat null as closed
    }
    let inner_stream = nvim_rstream_get_stream(stream);
    if inner_stream.is_null() {
        return 1;
    }
    nvim_stream_is_closed(inner_stream)
}

/// Get the signal number from a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_get_signum(watcher: SignalWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    nvim_signal_watcher_get_signum(watcher)
}

/// Get the events queue from a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_get_events(watcher: SignalWatcherHandle) -> MultiQueueHandle {
    if watcher.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_signal_watcher_get_events(watcher)
}

/// Get the user data from a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_get_data(watcher: SignalWatcherHandle) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_signal_watcher_get_data(watcher)
}

/// Get the address from a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_get_addr(watcher: SocketWatcherHandle) -> *const std::ffi::c_char {
    if watcher.is_null() {
        return std::ptr::null();
    }
    nvim_socket_watcher_get_addr(watcher)
}

/// Get the events queue from a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_get_events(watcher: SocketWatcherHandle) -> MultiQueueHandle {
    if watcher.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_socket_watcher_get_events(watcher)
}

/// Get the user data from a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_get_data(watcher: SocketWatcherHandle) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_socket_watcher_get_data(watcher)
}

/// Check if a SocketWatcher is TCP type
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_is_tcp(watcher: SocketWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    nvim_socket_watcher_is_tcp(watcher)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_nil() {
        let event = Event::nil();
        assert!(event.is_nil());
        assert!(event.handler.is_none());
        for arg in &event.argv {
            assert!(arg.is_null());
        }
    }

    #[test]
    fn test_event_default() {
        let event = Event::default();
        assert!(event.is_nil());
    }

    #[test]
    fn test_handles_null() {
        assert!(LoopHandle::null().is_null());
        assert!(MultiQueueHandle::null().is_null());
        assert!(TimeWatcherHandle::null().is_null());
    }
}
