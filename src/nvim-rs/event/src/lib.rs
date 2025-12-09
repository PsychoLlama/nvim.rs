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
    fn nvim_proc_get_stopped_time(proc: ProcHandle) -> u64;
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
