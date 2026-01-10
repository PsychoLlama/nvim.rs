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

/// Opaque handle to a WBuffer (write buffer)
///
/// WBuffer holds data for writing to streams with reference counting.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WBufferHandle(*mut std::ffi::c_void);

impl WBufferHandle {
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

/// Opaque handle to a libuv uv_loop_t
///
/// This is the raw libuv event loop, wrapped by Loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvLoopHandle(*mut std::ffi::c_void);

impl UvLoopHandle {
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

/// Opaque handle to a libuv uv_async_t
///
/// Used for cross-thread signaling in the event loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvAsyncHandle(*mut std::ffi::c_void);

impl UvAsyncHandle {
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

/// Opaque handle to a libuv uv_timer_t
///
/// Used for timer callbacks in the event loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvTimerHandle(*mut std::ffi::c_void);

impl UvTimerHandle {
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

/// Opaque handle to a libuv uv_pipe_t
///
/// Used for pipe I/O in the event loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvPipeHandle(*mut std::ffi::c_void);

impl UvPipeHandle {
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

/// Opaque handle to a libuv uv_tcp_t
///
/// Used for TCP connections in the event loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvTcpHandle(*mut std::ffi::c_void);

impl UvTcpHandle {
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

/// Opaque handle to a libuv uv_process_t
///
/// Used for child process management in the event loop.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvProcessHandle(*mut std::ffi::c_void);

impl UvProcessHandle {
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

#[allow(dead_code)]
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
    fn nvim_multiqueue_get_headtail(mq: MultiQueueHandle) -> *mut EventQueue;
    fn nvim_multiqueue_get_size_field(mq: MultiQueueHandle) -> usize;
    fn nvim_multiqueue_has_parent(mq: MultiQueueHandle) -> c_int;

    // TimeWatcher accessors
    fn nvim_timewatcher_get_data(tw: TimeWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_timewatcher_get_events(tw: TimeWatcherHandle) -> MultiQueueHandle;
    fn nvim_timewatcher_is_blockable(tw: TimeWatcherHandle) -> c_int;
    fn nvim_timewatcher_set_data(tw: TimeWatcherHandle, data: *mut std::ffi::c_void);
    fn nvim_timewatcher_set_events(tw: TimeWatcherHandle, events: MultiQueueHandle);
    fn nvim_timewatcher_set_blockable(tw: TimeWatcherHandle, blockable: c_int);
    fn nvim_timewatcher_get_cb(tw: TimeWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_timewatcher_set_cb(tw: TimeWatcherHandle, cb: *mut std::ffi::c_void);
    fn nvim_timewatcher_get_close_cb(tw: TimeWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_timewatcher_set_close_cb(tw: TimeWatcherHandle, cb: *mut std::ffi::c_void);
    fn nvim_timewatcher_call_cb(tw: TimeWatcherHandle);
    fn nvim_timewatcher_call_close_cb(tw: TimeWatcherHandle);

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
    fn nvim_stream_get_cb_data(stream: StreamHandle) -> *mut std::ffi::c_void;
    fn nvim_stream_set_cb_data(stream: StreamHandle, data: *mut std::ffi::c_void);
    fn nvim_stream_get_fpos(stream: StreamHandle) -> i64;
    fn nvim_stream_set_fpos(stream: StreamHandle, fpos: i64);
    fn nvim_stream_fpos_add(stream: StreamHandle, amount: i64);
    fn nvim_stream_get_close_cb(stream: StreamHandle) -> *mut std::ffi::c_void;
    fn nvim_stream_set_close_cb(stream: StreamHandle, cb: *mut std::ffi::c_void);
    fn nvim_stream_get_close_cb_data(stream: StreamHandle) -> *mut std::ffi::c_void;
    fn nvim_stream_set_close_cb_data(stream: StreamHandle, data: *mut std::ffi::c_void);
    fn nvim_stream_get_internal_data(stream: StreamHandle) -> *mut std::ffi::c_void;
    fn nvim_stream_set_internal_data(stream: StreamHandle, data: *mut std::ffi::c_void);
    fn nvim_stream_get_internal_close_cb(stream: StreamHandle) -> *mut std::ffi::c_void;
    fn nvim_stream_set_internal_close_cb(stream: StreamHandle, cb: *mut std::ffi::c_void);
    fn nvim_stream_call_close_cb(stream: StreamHandle);
    fn nvim_stream_call_internal_close_cb(stream: StreamHandle);
    fn nvim_stream_set_pending_reqs(stream: StreamHandle, pending_reqs: usize);
    fn nvim_stream_set_events(stream: StreamHandle, events: MultiQueueHandle);

    // RStream accessors
    fn nvim_rstream_did_eof(stream: RStreamHandle) -> c_int;
    fn nvim_rstream_set_did_eof(stream: RStreamHandle, eof: c_int);
    fn nvim_rstream_want_read(stream: RStreamHandle) -> c_int;
    fn nvim_rstream_set_want_read(stream: RStreamHandle, want_read: c_int);
    fn nvim_rstream_num_bytes(stream: RStreamHandle) -> usize;
    fn nvim_rstream_set_num_bytes(stream: RStreamHandle, num_bytes: usize);
    fn nvim_rstream_num_bytes_add(stream: RStreamHandle, amount: usize);
    fn nvim_rstream_available(stream: RStreamHandle) -> usize;
    fn nvim_rstream_get_stream(stream: RStreamHandle) -> StreamHandle;

    // SignalWatcher accessors
    fn nvim_signal_watcher_get_signum(watcher: SignalWatcherHandle) -> c_int;
    fn nvim_signal_watcher_get_events(watcher: SignalWatcherHandle) -> MultiQueueHandle;
    fn nvim_signal_watcher_get_data(watcher: SignalWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_signal_watcher_set_data(watcher: SignalWatcherHandle, data: *mut std::ffi::c_void);
    fn nvim_signal_watcher_set_events(watcher: SignalWatcherHandle, events: MultiQueueHandle);
    fn nvim_signal_watcher_get_cb(watcher: SignalWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_signal_watcher_set_cb(watcher: SignalWatcherHandle, cb: *mut std::ffi::c_void);
    fn nvim_signal_watcher_get_close_cb(watcher: SignalWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_signal_watcher_set_close_cb(watcher: SignalWatcherHandle, cb: *mut std::ffi::c_void);
    fn nvim_signal_watcher_call_cb(watcher: SignalWatcherHandle);
    fn nvim_signal_watcher_call_close_cb(watcher: SignalWatcherHandle);

    // SocketWatcher accessors
    fn nvim_socket_watcher_get_addr(watcher: SocketWatcherHandle) -> *const std::ffi::c_char;
    fn nvim_socket_watcher_get_events(watcher: SocketWatcherHandle) -> MultiQueueHandle;
    fn nvim_socket_watcher_get_data(watcher: SocketWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_socket_watcher_is_tcp(watcher: SocketWatcherHandle) -> c_int;
    fn nvim_socket_watcher_set_data(watcher: SocketWatcherHandle, data: *mut std::ffi::c_void);
    fn nvim_socket_watcher_set_events(watcher: SocketWatcherHandle, events: MultiQueueHandle);
    fn nvim_socket_watcher_get_cb(watcher: SocketWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_socket_watcher_set_cb(watcher: SocketWatcherHandle, cb: *mut std::ffi::c_void);
    fn nvim_socket_watcher_get_close_cb(watcher: SocketWatcherHandle) -> *mut std::ffi::c_void;
    fn nvim_socket_watcher_set_close_cb(watcher: SocketWatcherHandle, cb: *mut std::ffi::c_void);
    fn nvim_socket_watcher_call_cb(watcher: SocketWatcherHandle, status: c_int);
    fn nvim_socket_watcher_call_close_cb(watcher: SocketWatcherHandle);

    // WBuffer accessors
    fn nvim_wbuffer_get_size(buffer: WBufferHandle) -> usize;
    fn nvim_wbuffer_get_refcount(buffer: WBufferHandle) -> usize;
    fn nvim_wbuffer_get_data(buffer: WBufferHandle) -> *mut std::ffi::c_char;
    fn nvim_wbuffer_get_cb(buffer: WBufferHandle) -> *mut std::ffi::c_void;
    fn nvim_wbuffer_set_size(buffer: WBufferHandle, size: usize);
    fn nvim_wbuffer_set_refcount(buffer: WBufferHandle, refcount: usize);
    fn nvim_wbuffer_decref(buffer: WBufferHandle) -> c_int;
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

/// Get the data pointer from a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_get_data(tw: TimeWatcherHandle) -> *mut std::ffi::c_void {
    if tw.is_null() {
        return std::ptr::null_mut();
    }
    nvim_timewatcher_get_data(tw)
}

/// Set the data pointer for a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_set_data(
    tw: TimeWatcherHandle,
    data: *mut std::ffi::c_void,
) {
    if !tw.is_null() {
        nvim_timewatcher_set_data(tw, data);
    }
}

/// Get the events queue from a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_get_events(tw: TimeWatcherHandle) -> MultiQueueHandle {
    if tw.is_null() {
        return MultiQueueHandle::null();
    }
    nvim_timewatcher_get_events(tw)
}

/// Set the events queue for a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_set_events(
    tw: TimeWatcherHandle,
    events: MultiQueueHandle,
) {
    if !tw.is_null() {
        nvim_timewatcher_set_events(tw, events);
    }
}

/// Check if a TimeWatcher is blockable
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_is_blockable(tw: TimeWatcherHandle) -> c_int {
    if tw.is_null() {
        return 0;
    }
    nvim_timewatcher_is_blockable(tw)
}

/// Set the blockable flag for a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_set_blockable(tw: TimeWatcherHandle, blockable: c_int) {
    if !tw.is_null() {
        nvim_timewatcher_set_blockable(tw, blockable);
    }
}

/// Get the cb (timer callback) from a TimeWatcher (as void* for FFI)
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_get_cb(tw: TimeWatcherHandle) -> *mut std::ffi::c_void {
    if tw.is_null() {
        return std::ptr::null_mut();
    }
    nvim_timewatcher_get_cb(tw)
}

/// Set the cb (timer callback) for a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_set_cb(tw: TimeWatcherHandle, cb: *mut std::ffi::c_void) {
    if !tw.is_null() {
        nvim_timewatcher_set_cb(tw, cb);
    }
}

/// Get the close_cb from a TimeWatcher (as void* for FFI)
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_get_close_cb(
    tw: TimeWatcherHandle,
) -> *mut std::ffi::c_void {
    if tw.is_null() {
        return std::ptr::null_mut();
    }
    nvim_timewatcher_get_close_cb(tw)
}

/// Set the close_cb for a TimeWatcher
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_set_close_cb(
    tw: TimeWatcherHandle,
    cb: *mut std::ffi::c_void,
) {
    if !tw.is_null() {
        nvim_timewatcher_set_close_cb(tw, cb);
    }
}

/// Call the timer callback if set
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_call_cb(tw: TimeWatcherHandle) {
    if !tw.is_null() {
        nvim_timewatcher_call_cb(tw);
    }
}

/// Call the close callback if set
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_call_close_cb(tw: TimeWatcherHandle) {
    if !tw.is_null() {
        nvim_timewatcher_call_close_cb(tw);
    }
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
pub unsafe extern "C" fn rs_proc_set_internal_close_cb(
    proc: ProcHandle,
    cb: *mut std::ffi::c_void,
) {
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

/// Get the cb_data from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_cb_data(stream: StreamHandle) -> *mut std::ffi::c_void {
    if stream.is_null() {
        return std::ptr::null_mut();
    }
    nvim_stream_get_cb_data(stream)
}

/// Set the cb_data for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_cb_data(stream: StreamHandle, data: *mut std::ffi::c_void) {
    if !stream.is_null() {
        nvim_stream_set_cb_data(stream, data);
    }
}

/// Get the fpos from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_fpos(stream: StreamHandle) -> i64 {
    if stream.is_null() {
        return 0;
    }
    nvim_stream_get_fpos(stream)
}

/// Set the fpos for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_fpos(stream: StreamHandle, fpos: i64) {
    if !stream.is_null() {
        nvim_stream_set_fpos(stream, fpos);
    }
}

/// Add to the fpos for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_fpos_add(stream: StreamHandle, amount: i64) {
    if !stream.is_null() {
        nvim_stream_fpos_add(stream, amount);
    }
}

/// Get the close_cb from a Stream (as void* for FFI)
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_close_cb(stream: StreamHandle) -> *mut std::ffi::c_void {
    if stream.is_null() {
        return std::ptr::null_mut();
    }
    nvim_stream_get_close_cb(stream)
}

/// Set the close_cb for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_close_cb(stream: StreamHandle, cb: *mut std::ffi::c_void) {
    if !stream.is_null() {
        nvim_stream_set_close_cb(stream, cb);
    }
}

/// Get the close_cb_data from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_close_cb_data(
    stream: StreamHandle,
) -> *mut std::ffi::c_void {
    if stream.is_null() {
        return std::ptr::null_mut();
    }
    nvim_stream_get_close_cb_data(stream)
}

/// Set the close_cb_data for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_close_cb_data(
    stream: StreamHandle,
    data: *mut std::ffi::c_void,
) {
    if !stream.is_null() {
        nvim_stream_set_close_cb_data(stream, data);
    }
}

/// Get the internal_data from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_internal_data(
    stream: StreamHandle,
) -> *mut std::ffi::c_void {
    if stream.is_null() {
        return std::ptr::null_mut();
    }
    nvim_stream_get_internal_data(stream)
}

/// Set the internal_data for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_internal_data(
    stream: StreamHandle,
    data: *mut std::ffi::c_void,
) {
    if !stream.is_null() {
        nvim_stream_set_internal_data(stream, data);
    }
}

/// Get the internal_close_cb from a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_get_internal_close_cb(
    stream: StreamHandle,
) -> *mut std::ffi::c_void {
    if stream.is_null() {
        return std::ptr::null_mut();
    }
    nvim_stream_get_internal_close_cb(stream)
}

/// Set the internal_close_cb for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_internal_close_cb(
    stream: StreamHandle,
    cb: *mut std::ffi::c_void,
) {
    if !stream.is_null() {
        nvim_stream_set_internal_close_cb(stream, cb);
    }
}

/// Call the close_cb if set
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_call_close_cb(stream: StreamHandle) {
    if !stream.is_null() {
        nvim_stream_call_close_cb(stream);
    }
}

/// Call the internal_close_cb if set
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_call_internal_close_cb(stream: StreamHandle) {
    if !stream.is_null() {
        nvim_stream_call_internal_close_cb(stream);
    }
}

/// Set the pending_reqs for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_pending_reqs(stream: StreamHandle, pending_reqs: usize) {
    if !stream.is_null() {
        nvim_stream_set_pending_reqs(stream, pending_reqs);
    }
}

/// Set the events queue for a Stream
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_set_events(stream: StreamHandle, events: MultiQueueHandle) {
    if !stream.is_null() {
        nvim_stream_set_events(stream, events);
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

/// Set the did_eof flag for an RStream
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_set_did_eof(stream: RStreamHandle, eof: c_int) {
    if !stream.is_null() {
        nvim_rstream_set_did_eof(stream, eof);
    }
}

/// Set the want_read flag for an RStream
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_set_want_read(stream: RStreamHandle, want_read: c_int) {
    if !stream.is_null() {
        nvim_rstream_set_want_read(stream, want_read);
    }
}

/// Set the num_bytes for an RStream
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_set_num_bytes(stream: RStreamHandle, num_bytes: usize) {
    if !stream.is_null() {
        nvim_rstream_set_num_bytes(stream, num_bytes);
    }
}

/// Add to the num_bytes for an RStream
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_num_bytes_add(stream: RStreamHandle, amount: usize) {
    if !stream.is_null() {
        nvim_rstream_num_bytes_add(stream, amount);
    }
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
pub unsafe extern "C" fn rs_signal_watcher_get_events(
    watcher: SignalWatcherHandle,
) -> MultiQueueHandle {
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
pub unsafe extern "C" fn rs_signal_watcher_get_data(
    watcher: SignalWatcherHandle,
) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_signal_watcher_get_data(watcher)
}

/// Set the user data for a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_set_data(
    watcher: SignalWatcherHandle,
    data: *mut std::ffi::c_void,
) {
    if !watcher.is_null() {
        nvim_signal_watcher_set_data(watcher, data);
    }
}

/// Set the events queue for a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_set_events(
    watcher: SignalWatcherHandle,
    events: MultiQueueHandle,
) {
    if !watcher.is_null() {
        nvim_signal_watcher_set_events(watcher, events);
    }
}

/// Get the cb from a SignalWatcher (as void* for FFI)
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_get_cb(
    watcher: SignalWatcherHandle,
) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_signal_watcher_get_cb(watcher)
}

/// Set the cb for a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_set_cb(
    watcher: SignalWatcherHandle,
    cb: *mut std::ffi::c_void,
) {
    if !watcher.is_null() {
        nvim_signal_watcher_set_cb(watcher, cb);
    }
}

/// Get the close_cb from a SignalWatcher (as void* for FFI)
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_get_close_cb(
    watcher: SignalWatcherHandle,
) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_signal_watcher_get_close_cb(watcher)
}

/// Set the close_cb for a SignalWatcher
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_set_close_cb(
    watcher: SignalWatcherHandle,
    cb: *mut std::ffi::c_void,
) {
    if !watcher.is_null() {
        nvim_signal_watcher_set_close_cb(watcher, cb);
    }
}

/// Call the signal callback if set
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_call_cb(watcher: SignalWatcherHandle) {
    if !watcher.is_null() {
        nvim_signal_watcher_call_cb(watcher);
    }
}

/// Call the close callback if set
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_call_close_cb(watcher: SignalWatcherHandle) {
    if !watcher.is_null() {
        nvim_signal_watcher_call_close_cb(watcher);
    }
}

/// Get the address from a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_get_addr(
    watcher: SocketWatcherHandle,
) -> *const std::ffi::c_char {
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
pub unsafe extern "C" fn rs_socket_watcher_get_events(
    watcher: SocketWatcherHandle,
) -> MultiQueueHandle {
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
pub unsafe extern "C" fn rs_socket_watcher_get_data(
    watcher: SocketWatcherHandle,
) -> *mut std::ffi::c_void {
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

/// Set the user data for a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_set_data(
    watcher: SocketWatcherHandle,
    data: *mut std::ffi::c_void,
) {
    if !watcher.is_null() {
        nvim_socket_watcher_set_data(watcher, data);
    }
}

/// Set the events queue for a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_set_events(
    watcher: SocketWatcherHandle,
    events: MultiQueueHandle,
) {
    if !watcher.is_null() {
        nvim_socket_watcher_set_events(watcher, events);
    }
}

/// Get the cb from a SocketWatcher (as void* for FFI)
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_get_cb(
    watcher: SocketWatcherHandle,
) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_socket_watcher_get_cb(watcher)
}

/// Set the cb for a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_set_cb(
    watcher: SocketWatcherHandle,
    cb: *mut std::ffi::c_void,
) {
    if !watcher.is_null() {
        nvim_socket_watcher_set_cb(watcher, cb);
    }
}

/// Get the close_cb from a SocketWatcher (as void* for FFI)
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_get_close_cb(
    watcher: SocketWatcherHandle,
) -> *mut std::ffi::c_void {
    if watcher.is_null() {
        return std::ptr::null_mut();
    }
    nvim_socket_watcher_get_close_cb(watcher)
}

/// Set the close_cb for a SocketWatcher
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_set_close_cb(
    watcher: SocketWatcherHandle,
    cb: *mut std::ffi::c_void,
) {
    if !watcher.is_null() {
        nvim_socket_watcher_set_close_cb(watcher, cb);
    }
}

/// Call the socket callback if set
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_call_cb(watcher: SocketWatcherHandle, status: c_int) {
    if !watcher.is_null() {
        nvim_socket_watcher_call_cb(watcher, status);
    }
}

/// Call the close callback if set
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_call_close_cb(watcher: SocketWatcherHandle) {
    if !watcher.is_null() {
        nvim_socket_watcher_call_close_cb(watcher);
    }
}

// =============================================================================
// Event Creation and Management (Pure Rust)
// =============================================================================

/// Create an event with a handler and arguments
///
/// This is a helper for creating Event structures from Rust.
///
/// # Arguments
///
/// * `handler` - The callback function
/// * `arg0` - First argument (can be null)
///
/// # Safety
///
/// The handler must be a valid function pointer that expects the argument types provided.
#[no_mangle]
pub unsafe extern "C" fn rs_event_create(
    handler: ArgvCallback,
    arg0: *mut std::ffi::c_void,
) -> Event {
    let mut event = Event::nil();
    event.handler = handler;
    event.argv[0] = arg0;
    event
}

/// Create an event with a handler and two arguments
///
/// # Safety
///
/// The handler must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_event_create2(
    handler: ArgvCallback,
    arg0: *mut std::ffi::c_void,
    arg1: *mut std::ffi::c_void,
) -> Event {
    let mut event = Event::nil();
    event.handler = handler;
    event.argv[0] = arg0;
    event.argv[1] = arg1;
    event
}

/// Create an event with a handler and three arguments
///
/// # Safety
///
/// The handler must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_event_create3(
    handler: ArgvCallback,
    arg0: *mut std::ffi::c_void,
    arg1: *mut std::ffi::c_void,
    arg2: *mut std::ffi::c_void,
) -> Event {
    let mut event = Event::nil();
    event.handler = handler;
    event.argv[0] = arg0;
    event.argv[1] = arg1;
    event.argv[2] = arg2;
    event
}

/// Copy an event structure
///
/// # Safety
///
/// `src` must be a valid pointer to an Event struct.
/// `dst` must be a valid pointer to writable Event memory.
#[no_mangle]
pub unsafe extern "C" fn rs_event_copy(dst: *mut Event, src: *const Event) {
    if !dst.is_null() && !src.is_null() {
        *dst = *src;
    }
}

// =============================================================================
// Loop Management (Pure Rust Logic)
// =============================================================================

/// Check if a Loop is in a state where it can accept new events
///
/// Returns 1 if the loop is active and not closing, 0 otherwise.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_is_active(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 0;
    }
    // Loop is active if it's not closing
    c_int::from(nvim_loop_is_closing(loop_) == 0)
}

/// Check if a Loop can be re-entered (recursive == 0)
///
/// Returns 1 if the loop can be entered, 0 if already in use.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_can_enter(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 0;
    }
    c_int::from(nvim_loop_get_recursive(loop_) == 0)
}

/// Get the combined size of all event queues in a Loop
///
/// Returns the sum of events, fast_events, and thread_events sizes.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_total_events(loop_: LoopHandle) -> usize {
    if loop_.is_null() {
        return 0;
    }

    let mut total = 0usize;

    let events = nvim_loop_get_events(loop_);
    if !events.is_null() {
        total += nvim_multiqueue_get_size_field(events);
    }

    let fast_events = nvim_loop_get_fast_events(loop_);
    if !fast_events.is_null() {
        total += nvim_multiqueue_get_size_field(fast_events);
    }

    let thread_events = nvim_loop_get_thread_events(loop_);
    if !thread_events.is_null() {
        total += nvim_multiqueue_get_size_field(thread_events);
    }

    total
}

/// Check if all event queues in a Loop are empty
///
/// Returns 1 if all queues are empty, 0 if any has events.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_all_empty(loop_: LoopHandle) -> c_int {
    c_int::from(rs_loop_total_events(loop_) == 0)
}

/// Check if Loop has any fast events pending
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_has_fast_events(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 0;
    }
    let fast_events = nvim_loop_get_fast_events(loop_);
    if fast_events.is_null() {
        return 0;
    }
    c_int::from(rs_multiqueue_empty(fast_events) == 0)
}

/// Check if Loop has any thread events pending
///
/// Note: Caller should hold the loop mutex when calling this.
///
/// # Safety
///
/// `loop_` must be a valid Loop handle
#[no_mangle]
pub unsafe extern "C" fn rs_loop_has_thread_events(loop_: LoopHandle) -> c_int {
    if loop_.is_null() {
        return 0;
    }
    let thread_events = nvim_loop_get_thread_events(loop_);
    if thread_events.is_null() {
        return 0;
    }
    c_int::from(rs_multiqueue_empty(thread_events) == 0)
}

// =============================================================================
// MultiQueue Extended Operations (Pure Rust)
// =============================================================================

/// Get the combined size of a MultiQueue and its parent (if any)
///
/// If the queue has a parent, returns the parent's size (which includes
/// link nodes for all children). Otherwise returns the queue's own size.
///
/// # Safety
///
/// `mq` must be a valid MultiQueue handle
#[no_mangle]
pub unsafe extern "C" fn rs_multiqueue_total_size(mq: MultiQueueHandle) -> usize {
    if mq.is_null() {
        return 0;
    }
    nvim_multiqueue_get_size_field(mq)
}

/// Check if a MultiQueue is a child queue (has a parent)
///
/// Returns 1 if this is a child queue, 0 if it's a root queue.
///
/// # Safety
///
/// `mq` must be a valid MultiQueue handle
#[no_mangle]
pub unsafe extern "C" fn rs_multiqueue_is_child(mq: MultiQueueHandle) -> c_int {
    if mq.is_null() {
        return 0;
    }
    nvim_multiqueue_has_parent(mq)
}

/// Check if a MultiQueue is a root queue (no parent)
///
/// Returns 1 if this is a root queue, 0 if it's a child.
///
/// # Safety
///
/// `mq` must be a valid MultiQueue handle
#[no_mangle]
pub unsafe extern "C" fn rs_multiqueue_is_root(mq: MultiQueueHandle) -> c_int {
    if mq.is_null() {
        return 0; // null is neither root nor child
    }
    c_int::from(nvim_multiqueue_has_parent(mq) == 0)
}

// =============================================================================
// TimeWatcher Extended Operations (Pure Rust)
// =============================================================================

/// Check if a TimeWatcher has a callback set
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_has_cb(tw: TimeWatcherHandle) -> c_int {
    if tw.is_null() {
        return 0;
    }
    c_int::from(!nvim_timewatcher_get_cb(tw).is_null())
}

/// Check if a TimeWatcher has a close callback set
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_has_close_cb(tw: TimeWatcherHandle) -> c_int {
    if tw.is_null() {
        return 0;
    }
    c_int::from(!nvim_timewatcher_get_close_cb(tw).is_null())
}

/// Check if a TimeWatcher has an events queue
///
/// # Safety
///
/// `tw` must be a valid TimeWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_timewatcher_has_events(tw: TimeWatcherHandle) -> c_int {
    if tw.is_null() {
        return 0;
    }
    c_int::from(!nvim_timewatcher_get_events(tw).is_null())
}

// =============================================================================
// Stream Extended Operations (Pure Rust)
// =============================================================================

/// Check if a Stream has pending write requests
///
/// Returns 1 if there are pending requests, 0 otherwise.
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_has_pending(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    c_int::from(nvim_stream_pending_reqs(stream) > 0)
}

/// Check if a Stream is ready for operations (not closed and no pending requests)
///
/// Returns 1 if ready, 0 otherwise.
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_is_ready(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    let closed = nvim_stream_is_closed(stream) != 0;
    let pending = nvim_stream_pending_reqs(stream) > 0;
    c_int::from(!closed && !pending)
}

/// Check if a Stream is within its memory limit
///
/// Returns 1 if curmem <= maxmem, 0 otherwise.
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_within_limit(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    let curmem = nvim_stream_get_curmem(stream);
    let maxmem = nvim_stream_get_maxmem(stream);
    c_int::from(curmem <= maxmem)
}

/// Get the available write capacity for a Stream
///
/// Returns the number of bytes that can be written (maxmem - curmem),
/// or 0 if over limit or if stream is null.
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_write_capacity(stream: StreamHandle) -> usize {
    if stream.is_null() {
        return 0;
    }
    let curmem = nvim_stream_get_curmem(stream);
    let maxmem = nvim_stream_get_maxmem(stream);
    if curmem >= maxmem {
        return 0;
    }
    maxmem - curmem
}

/// Check if a Stream has a write callback set
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_has_write_cb(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    c_int::from(!nvim_stream_get_write_cb(stream).is_null())
}

/// Check if a Stream has a close callback set
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_has_close_cb(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    c_int::from(!nvim_stream_get_close_cb(stream).is_null())
}

/// Check if a Stream has an events queue
///
/// # Safety
///
/// `stream` must be a valid Stream handle
#[no_mangle]
pub unsafe extern "C" fn rs_stream_has_events(stream: StreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    c_int::from(!nvim_stream_get_events(stream).is_null())
}

// =============================================================================
// RStream Extended Operations (Pure Rust)
// =============================================================================

/// Check if an RStream is ready to process data
///
/// Returns 1 if the stream has data available and hasn't reached EOF.
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_has_data(stream: RStreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    let available = nvim_rstream_available(stream);
    c_int::from(available > 0)
}

/// Check if an RStream is in a terminal state (EOF or closed)
///
/// Returns 1 if the stream has reached EOF or is closed.
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_is_terminal(stream: RStreamHandle) -> c_int {
    if stream.is_null() {
        return 1; // Null is terminal
    }
    let did_eof = nvim_rstream_did_eof(stream) != 0;
    let inner_stream = nvim_rstream_get_stream(stream);
    let is_closed = if inner_stream.is_null() {
        true
    } else {
        nvim_stream_is_closed(inner_stream) != 0
    };
    c_int::from(did_eof || is_closed)
}

/// Check if an RStream can accept more data (has buffer space)
///
/// Returns 1 if the stream wants to read and hasn't paused due to full buffer.
///
/// # Safety
///
/// `stream` must be a valid RStream handle
#[no_mangle]
pub unsafe extern "C" fn rs_rstream_can_receive(stream: RStreamHandle) -> c_int {
    if stream.is_null() {
        return 0;
    }
    // Check if wants to read and not at EOF
    let wants_read = nvim_rstream_want_read(stream) != 0;
    let at_eof = nvim_rstream_did_eof(stream) != 0;
    c_int::from(wants_read && !at_eof)
}

// =============================================================================
// SocketWatcher Extended Operations (Pure Rust)
// =============================================================================

/// Check if a SocketWatcher has a callback set
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_has_cb(watcher: SocketWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_socket_watcher_get_cb(watcher).is_null())
}

/// Check if a SocketWatcher has a close callback set
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_has_close_cb(watcher: SocketWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_socket_watcher_get_close_cb(watcher).is_null())
}

/// Check if a SocketWatcher has an events queue
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_has_events(watcher: SocketWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_socket_watcher_get_events(watcher).is_null())
}

/// Check if a SocketWatcher has user data set
///
/// # Safety
///
/// `watcher` must be a valid SocketWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_socket_watcher_has_data(watcher: SocketWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_socket_watcher_get_data(watcher).is_null())
}

// =============================================================================
// WBuffer Operations (Pure Rust)
// =============================================================================

/// Check if a WBuffer has a finalizer callback set
///
/// # Safety
///
/// `buffer` must be a valid WBuffer handle
#[no_mangle]
pub unsafe extern "C" fn rs_wbuffer_has_finalizer(buffer: WBufferHandle) -> c_int {
    if buffer.is_null() {
        return 0;
    }
    c_int::from(!nvim_wbuffer_get_cb(buffer).is_null())
}

/// Get the refcount from a WBuffer
///
/// # Safety
///
/// `buffer` must be a valid WBuffer handle
#[no_mangle]
pub unsafe extern "C" fn rs_wbuffer_refcount(buffer: WBufferHandle) -> usize {
    if buffer.is_null() {
        return 0;
    }
    nvim_wbuffer_get_refcount(buffer)
}

/// Check if a WBuffer can be freed (refcount == 0)
///
/// # Safety
///
/// `buffer` must be a valid WBuffer handle
#[no_mangle]
pub unsafe extern "C" fn rs_wbuffer_can_free(buffer: WBufferHandle) -> c_int {
    if buffer.is_null() {
        return 0;
    }
    c_int::from(nvim_wbuffer_get_refcount(buffer) == 0)
}

// =============================================================================
// Proc Extended Operations (Pure Rust)
// =============================================================================

/// Check if a Proc has exited (status >= 0)
///
/// Returns 1 if the process has exited, 0 if still running.
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_exited(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(nvim_proc_get_status(proc) >= 0)
}

/// Check if a Proc is still running (not closed and not exited)
///
/// Returns 1 if still running, 0 if exited or closed.
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_running(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    let exited = nvim_proc_get_status(proc) >= 0;
    let closed = nvim_proc_is_closed(proc) != 0;
    c_int::from(!exited && !closed)
}

/// Check if a Proc is waiting to be killed (has stopped_time > 0)
///
/// Returns 1 if stop has been requested, 0 otherwise.
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_stopping(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    let stopped_time = nvim_proc_get_stopped_time(proc);
    let exited = nvim_proc_get_status(proc) >= 0;
    c_int::from(stopped_time > 0 && !exited)
}

/// Check if a Proc has any references
///
/// Returns 1 if refcount > 0, 0 otherwise.
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_refs(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(nvim_proc_get_refcount(proc) > 0)
}

/// Check if a Proc has a callback set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_cb(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_cb(proc).is_null())
}

/// Check if a Proc has an internal exit callback set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_internal_exit_cb(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_internal_exit_cb(proc).is_null())
}

/// Check if a Proc has an internal close callback set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_internal_close_cb(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_internal_close_cb(proc).is_null())
}

/// Check if a Proc has an events queue
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_events(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_events(proc).is_null())
}

/// Check if a Proc is a PTY process (type == kProcTypePty)
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_pty(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(nvim_proc_get_type(proc) == 1) // kProcTypePty == 1
}

/// Check if a Proc is a libuv process (type == kProcTypeUv)
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_uv(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(nvim_proc_get_type(proc) == 0) // kProcTypeUv == 0
}

/// Check if a Proc is detached
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_is_detached(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    nvim_proc_get_detach(proc)
}

/// Check if a Proc has an exepath set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_exepath(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_exepath(proc).is_null())
}

/// Check if a Proc has a cwd set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_cwd(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_cwd(proc).is_null())
}

/// Check if a Proc has argv set
///
/// # Safety
///
/// `proc` must be a valid Proc handle
#[no_mangle]
pub unsafe extern "C" fn rs_proc_has_argv(proc: ProcHandle) -> c_int {
    if proc.is_null() {
        return 0;
    }
    c_int::from(!nvim_proc_get_argv(proc).is_null())
}

// =============================================================================
// SignalWatcher Extended Operations (Pure Rust)
// =============================================================================

/// Check if a SignalWatcher has a callback set
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_has_cb(watcher: SignalWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_signal_watcher_get_cb(watcher).is_null())
}

/// Check if a SignalWatcher has a close callback set
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_has_close_cb(watcher: SignalWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_signal_watcher_get_close_cb(watcher).is_null())
}

/// Check if a SignalWatcher has an events queue
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_has_events(watcher: SignalWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_signal_watcher_get_events(watcher).is_null())
}

/// Check if a SignalWatcher has user data set
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_has_data(watcher: SignalWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    c_int::from(!nvim_signal_watcher_get_data(watcher).is_null())
}

/// Get the signal number from a SignalWatcher
///
/// Returns 0 if the watcher is null.
///
/// # Safety
///
/// `watcher` must be a valid SignalWatcher handle
#[no_mangle]
pub unsafe extern "C" fn rs_signal_watcher_signum(watcher: SignalWatcherHandle) -> c_int {
    if watcher.is_null() {
        return 0;
    }
    nvim_signal_watcher_get_signum(watcher)
}

// =============================================================================
// Tests
// =============================================================================

// =============================================================================
// State Functions
// =============================================================================

extern "C" {
    fn nvim_get_was_safe() -> c_int;
    fn nvim_get_state() -> c_int;
    fn nvim_get_visual_active() -> c_int;
    fn nvim_get_visual_select() -> c_int;
    fn nvim_get_finish_op() -> c_int;
}

// Mode constants (from state_defs.h)
const MODE_NORMAL: c_int = 0x01;
const MODE_VISUAL: c_int = 0x02;
const MODE_OP_PENDING: c_int = 0x04;
const MODE_SELECT: c_int = 0x40;

/// Get whether the editor was in a safe state.
///
/// # Safety
/// Calls C accessor function for was_safe static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_was_safe_state() -> c_int {
    nvim_get_was_safe()
}

/// Get the real state of the editor.
///
/// MODE_VISUAL, MODE_SELECT and MODE_OP_PENDING State are never set directly,
/// they are equal to MODE_NORMAL State with a condition. This function returns
/// the real State.
#[inline]
fn get_real_state_impl() -> c_int {
    // SAFETY: These are safe accessors to C globals
    unsafe {
        let state = nvim_get_state();

        if (state & MODE_NORMAL) != 0 {
            if nvim_get_visual_active() != 0 {
                if nvim_get_visual_select() != 0 {
                    return MODE_SELECT;
                }
                return MODE_VISUAL;
            } else if nvim_get_finish_op() != 0 {
                return MODE_OP_PENDING;
            }
        }
        state
    }
}

/// FFI wrapper for get_real_state.
///
/// Returns the real state of the editor, accounting for virtual states like
/// MODE_VISUAL, MODE_SELECT, and MODE_OP_PENDING.
#[no_mangle]
pub extern "C" fn rs_get_real_state() -> c_int {
    get_real_state_impl()
}

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

    #[test]
    fn test_event_handler_max_argc() {
        // Verify EVENT_HANDLER_MAX_ARGC matches C definition
        assert_eq!(EVENT_HANDLER_MAX_ARGC, 10);
        // Ensure Event::argv has correct size
        let event = Event::nil();
        assert_eq!(event.argv.len(), EVENT_HANDLER_MAX_ARGC);
    }

    #[test]
    fn test_all_handle_types_null() {
        // Test all handle types have working null() and is_null() methods
        assert!(ProcHandle::null().is_null());
        assert!(StreamHandle::null().is_null());
        assert!(RStreamHandle::null().is_null());
        assert!(SignalWatcherHandle::null().is_null());
        assert!(SocketWatcherHandle::null().is_null());
        assert!(WBufferHandle::null().is_null());
        assert!(UvLoopHandle::null().is_null());
        assert!(UvAsyncHandle::null().is_null());
        assert!(UvTimerHandle::null().is_null());
        assert!(UvPipeHandle::null().is_null());
        assert!(UvTcpHandle::null().is_null());
        assert!(UvProcessHandle::null().is_null());
    }

    #[test]
    fn test_handle_sizes() {
        use std::ffi::c_void;
        // All handles should be pointer-sized (transparent wrapper over *mut c_void)
        let ptr_size = std::mem::size_of::<*mut c_void>();
        assert_eq!(std::mem::size_of::<LoopHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<MultiQueueHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<TimeWatcherHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<ProcHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<StreamHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<RStreamHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<SignalWatcherHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<SocketWatcherHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<WBufferHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<UvLoopHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<UvAsyncHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<UvTimerHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<UvPipeHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<UvTcpHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<UvProcessHandle>(), ptr_size);
    }

    #[test]
    fn test_event_queue_size() {
        // EventQueue should be 2 pointers
        let ptr_size = std::mem::size_of::<*mut EventQueue>();
        assert_eq!(std::mem::size_of::<EventQueue>(), 2 * ptr_size);
    }

    #[test]
    fn test_event_structure_size() {
        // Event should be: 1 Option<fn ptr> + 10 pointers
        // On 64-bit: 8 + 80 = 88 bytes (but Option<fn ptr> may have alignment)
        let event_size = std::mem::size_of::<Event>();
        // Just verify it's reasonable (at least handler + 10 pointers)
        assert!(event_size >= 11 * std::mem::size_of::<*mut std::ffi::c_void>());
    }

    #[test]
    fn test_null_event_is_nil() {
        // Null pointer should be treated as nil
        unsafe {
            assert_eq!(rs_event_is_nil(std::ptr::null()), 1);
        }
    }

    #[test]
    fn test_event_nil_writes_correctly() {
        // rs_event_nil should initialize to nil event
        let mut event = Event {
            handler: None, // Just use None - we can't create fake handler safely
            argv: [std::ptr::null_mut(); EVENT_HANDLER_MAX_ARGC],
        };
        unsafe {
            rs_event_nil(&raw mut event);
        }
        assert!(event.is_nil());
    }

    #[test]
    fn test_event_create() {
        // Test event creation with handler
        unsafe extern "C" fn dummy_handler(_argv: *mut *mut std::ffi::c_void) {}

        let arg0 = 123usize as *mut std::ffi::c_void;
        let event = unsafe { rs_event_create(Some(dummy_handler), arg0) };

        assert!(!event.is_nil());
        assert!(event.handler.is_some());
        assert_eq!(event.argv[0], arg0);
        // Other args should be null
        for i in 1..EVENT_HANDLER_MAX_ARGC {
            assert!(event.argv[i].is_null());
        }
    }

    #[test]
    fn test_event_create2() {
        unsafe extern "C" fn dummy_handler(_argv: *mut *mut std::ffi::c_void) {}

        let arg0 = 111usize as *mut std::ffi::c_void;
        let arg1 = 222usize as *mut std::ffi::c_void;
        let event = unsafe { rs_event_create2(Some(dummy_handler), arg0, arg1) };

        assert!(!event.is_nil());
        assert_eq!(event.argv[0], arg0);
        assert_eq!(event.argv[1], arg1);
        // Other args should be null
        for i in 2..EVENT_HANDLER_MAX_ARGC {
            assert!(event.argv[i].is_null());
        }
    }

    #[test]
    fn test_event_create3() {
        unsafe extern "C" fn dummy_handler(_argv: *mut *mut std::ffi::c_void) {}

        let arg0 = 111usize as *mut std::ffi::c_void;
        let arg1 = 222usize as *mut std::ffi::c_void;
        let arg2 = 333usize as *mut std::ffi::c_void;
        let event = unsafe { rs_event_create3(Some(dummy_handler), arg0, arg1, arg2) };

        assert!(!event.is_nil());
        assert_eq!(event.argv[0], arg0);
        assert_eq!(event.argv[1], arg1);
        assert_eq!(event.argv[2], arg2);
        // Other args should be null
        for i in 3..EVENT_HANDLER_MAX_ARGC {
            assert!(event.argv[i].is_null());
        }
    }

    #[test]
    fn test_event_copy() {
        unsafe extern "C" fn dummy_handler(_argv: *mut *mut std::ffi::c_void) {}

        let arg0 = 42usize as *mut std::ffi::c_void;
        let src = Event {
            handler: Some(dummy_handler),
            argv: {
                let mut arr = [std::ptr::null_mut(); EVENT_HANDLER_MAX_ARGC];
                arr[0] = arg0;
                arr
            },
        };

        let mut dst = Event::nil();
        unsafe {
            rs_event_copy(&raw mut dst, &raw const src);
        }

        assert!(dst.handler.is_some());
        assert_eq!(dst.argv[0], arg0);
    }

    // Note: Tests for rs_loop_*, rs_multiqueue_*, rs_timewatcher_* functions that
    // call C accessor functions are tested via the full build with C integration.
    // Pure Rust unit tests can only test functions that don't call into C.
}
