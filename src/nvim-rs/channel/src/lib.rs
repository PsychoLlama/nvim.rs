//! Channel management for Neovim RPC
//!
//! This module provides type definitions for Neovim's channel infrastructure,
//! which handles communication between Neovim and external processes (plugins,
//! GUIs, remote clients).
//!
//! # Channel Types
//!
//! - `kChannelStreamProc` - Process (job) channel
//! - `kChannelStreamSocket` - TCP/pipe socket channel
//! - `kChannelStreamStdio` - Standard I/O channel
//! - `kChannelStreamStderr` - Standard error channel
//! - `kChannelStreamInternal` - Internal loopback channel

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(dead_code)]
#![allow(clippy::pub_underscore_fields)]

use std::ffi::{c_int, c_longlong, c_void};

use nvim_eval::typval::{CallbackReaderT, CallbackT};

// =============================================================================
// FFI: existing C symbols (no new C accessors)
// =============================================================================

extern "C" {
    /// Find channel by ID. Defined in eval_shim.c.
    fn nvim_find_channel(id: u64) -> *mut c_void;
    /// Check if channel is a valid running job (proc stream, not stopped).
    /// Defined in eval_shim.c.
    fn nvim_channel_is_valid_job(chan: *mut c_void) -> c_int;
    /// Delete channel from the channels map by ID. Defined in eval_shim.c.
    fn nvim_channels_del(id: u64);

    /// Free callback and clear its data. Defined in eval/typval.c.
    fn callback_free(cb: *mut CallbackT);
    /// Clear a garray_T. Defined in garray.c.
    fn ga_clear(ga: *mut c_void);
    /// Initialize a garray_T. Defined in garray.c.
    fn ga_init(ga: *mut c_void, itemsize: c_int, growsize: c_int);

    /// Allocate a new channel. Defined in channel.c.
    fn channel_alloc(stream_type: c_int) -> *mut c_void;
    /// Initialize RPC subsystem. Defined in msgpack_rpc/channel.c.
    fn rpc_init();
    /// Free RPC state for channel. Defined in msgpack_rpc/channel.c.
    fn rpc_free(chan: *mut c_void);
    /// Free proc resources. Defined in event/proc.c.
    fn proc_free(proc_ptr: *mut c_void);
    /// Free a MultiQueue. Defined in event/multiqueue.c.
    fn multiqueue_free(mq: *mut c_void);
    /// Put an event on a MultiQueue. Defined in event/multiqueue.c.
    fn multiqueue_put_event(mq: *mut c_void, event: EventT);
    /// Get the main loop events queue. Defined in event crate (rs_loop_get_events).
    fn rs_loop_get_events(lp: *mut c_void) -> *mut c_void;
    /// The main loop. Defined in main.c / globals.h.
    static main_loop: [u8; 0]; // accessed only as pointer

    /// Free memory. Defined in memory.c.
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// Repr(C) event struct matching C `Event` (88 bytes)
// =============================================================================

/// Rust mirror of C `Event` struct.
///
/// Layout:
/// ```text
/// offset 0:  handler  (function pointer, 8 bytes)
/// offset 8:  argv     ([*mut c_void; 10], 80 bytes)
/// ```
#[repr(C)]
pub struct EventT {
    pub handler: Option<unsafe extern "C" fn(*mut *mut c_void)>,
    pub argv: [*mut c_void; 10],
}

// SAFETY: EventT contains raw pointers; we only create/pass it on the C event loop thread.
unsafe impl Send for EventT {}

// =============================================================================
// Repr(C) ChannelT struct matching C `Channel` (1928 bytes)
// =============================================================================

/// Rust mirror of C `Channel` struct (1928 bytes on x86-64).
///
/// Layout verified by `_Static_assert` blocks in `eval_struct_check.c`.
///
/// ```text
/// offset    0: id              (u64)
/// offset    8: refcount        (usize)
/// offset   16: events          (*mut c_void / MultiQueue*)
/// offset   24: streamtype      (c_int, 4 bytes)
/// offset   28: _streamtype_pad ([u8; 4])
/// offset   32: stream          ([u8; 1640], opaque union)
/// offset 1672: is_rpc          (bool)
/// offset 1673: detach          (bool)
/// offset 1674: _rpc_pad        ([u8; 6])
/// offset 1680: rpc             ([u8; 88], opaque RpcState)
/// offset 1768: term            (*mut c_void / Terminal*)
/// offset 1776: on_data         (CallbackReaderT, 64 bytes)
/// offset 1840: on_stderr       (CallbackReaderT, 64 bytes)
/// offset 1904: on_exit         (CallbackT, 16 bytes)
/// offset 1920: exit_status     (c_int)
/// offset 1924: callback_busy   (bool)
/// offset 1925: callback_scheduled (bool)
/// [2 bytes trailing padding]
/// sizeof: 1928
/// ```
#[repr(C)]
pub struct ChannelT {
    pub id: u64,
    pub refcount: usize,
    pub events: *mut c_void,
    pub streamtype: c_int,
    pub _streamtype_pad: [u8; 4],
    pub stream: [u8; 1640],
    pub is_rpc: bool,
    pub detach: bool,
    pub _rpc_pad: [u8; 6],
    pub rpc: [u8; 88],
    pub term: *mut c_void,
    pub on_data: CallbackReaderT,
    pub on_stderr: CallbackReaderT,
    pub on_exit: CallbackT,
    pub exit_status: c_int,
    pub callback_busy: bool,
    pub callback_scheduled: bool,
}

// =============================================================================
// Constants
// =============================================================================

/// Channel ID for stdio
pub const CHAN_STDIO: u64 = 1;

/// Channel ID for stderr
pub const CHAN_STDERR: u64 = 2;

// kChannelStreamProc = 0
const K_CHANNEL_STREAM_PROC: c_int = 0;

// kCallbackNone = 0
const K_CALLBACK_NONE: c_int = 0;

// =============================================================================
// Enums
// =============================================================================

/// Channel stream type
///
/// Determines the underlying transport mechanism for a channel.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStreamType {
    /// Process (job) channel
    Proc = 0,
    /// TCP/pipe socket channel
    Socket = 1,
    /// Standard I/O channel
    Stdio = 2,
    /// Standard error channel
    Stderr = 3,
    /// Internal loopback channel
    Internal = 4,
}

impl ChannelStreamType {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Proc),
            1 => Some(Self::Socket),
            2 => Some(Self::Stdio),
            3 => Some(Self::Stderr),
            4 => Some(Self::Internal),
            _ => None,
        }
    }
}

/// Channel part for partial close operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPart {
    /// stdin of a process channel
    Stdin = 0,
    /// stdout of a process channel
    Stdout = 1,
    /// stderr of a process channel
    Stderr = 2,
    /// RPC layer
    Rpc = 3,
    /// All parts (full close)
    All = 4,
}

impl ChannelPart {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Stdin),
            1 => Some(Self::Stdout),
            2 => Some(Self::Stderr),
            3 => Some(Self::Rpc),
            4 => Some(Self::All),
            _ => None,
        }
    }
}

/// Channel stdin mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStdinMode {
    /// stdin is connected via pipe
    Pipe = 0,
    /// stdin is disconnected
    Null = 1,
}

/// Client type for RPC channels
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientType {
    /// Unknown client type
    Unknown = -1,
    /// Remote client
    Remote = 0,
    /// UI client
    Ui = 1,
    /// Embedder client
    Embedder = 2,
    /// Host client
    Host = 3,
    /// Plugin client
    Plugin = 4,
    /// Msgpack-rpc client
    MsgpackRpc = 5,
}

impl ClientType {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Remote,
            1 => Self::Ui,
            2 => Self::Embedder,
            3 => Self::Host,
            4 => Self::Plugin,
            5 => Self::MsgpackRpc,
            _ => Self::Unknown,
        }
    }
}

// =============================================================================
// Helper: get main_loop pointer
// =============================================================================

/// Return a pointer to the global `main_loop` object.
///
/// # Safety
///
/// The global `main_loop` must be initialized before calling this.
#[inline]
unsafe fn main_loop_ptr() -> *mut c_void {
    (&raw const main_loop).cast::<u8>().cast_mut().cast()
}

// =============================================================================
// Migrated functions (Phase 1)
// =============================================================================

/// Check if a channel ID refers to a running job process.
///
/// Returns true if the channel exists, is a proc stream, and has not stopped.
///
/// # Safety
///
/// Accesses the global channels map.
#[export_name = "channel_job_running"]
pub unsafe extern "C" fn rs_channel_job_running(id: u64) -> bool {
    let chan = nvim_find_channel(id);
    !chan.is_null() && nvim_channel_is_valid_job(chan) != 0
}

/// Increment the channel reference count.
///
/// # Safety
///
/// `chan` must be a valid non-null pointer to a `Channel`.
#[no_mangle]
pub unsafe extern "C" fn channel_incref(chan: *mut ChannelT) {
    (*chan).refcount += 1;
}

/// Decrement the channel reference count.
///
/// When refcount reaches zero, schedules `free_channel_event` to run deferred
/// via the main loop queue.
///
/// # Safety
///
/// `chan` must be a valid non-null pointer to a `Channel`.
#[no_mangle]
pub unsafe extern "C" fn channel_decref(chan: *mut ChannelT) {
    (*chan).refcount -= 1;
    if (*chan).refcount == 0 {
        // delay free so that libuv is done with the handles
        let event = make_event(free_channel_event as _, chan.cast());
        multiqueue_put_event(rs_loop_get_events(main_loop_ptr()), event);
    }
}

/// Free a `CallbackReader`'s callback and internal buffer.
///
/// # Safety
///
/// `reader` must be a valid non-null pointer to a `CallbackReader`.
#[no_mangle]
pub unsafe extern "C" fn callback_reader_free(reader: *mut CallbackReaderT) {
    callback_free(&raw mut (*reader).cb);
    ga_clear((&raw mut (*reader).buffer).cast());
}

/// Initialize a `CallbackReader`'s buffer and set its type label.
///
/// # Safety
///
/// `reader` must be a valid non-null pointer to a `CallbackReader`.
/// `type_` must be a valid null-terminated C string that outlives the reader.
#[no_mangle]
pub unsafe extern "C" fn callback_reader_start(
    reader: *mut CallbackReaderT,
    type_: *const std::ffi::c_char,
) {
    // ga_init(ga, itemsize=sizeof(char*), growsize=32)
    // size_of::<*mut c_void>() is always 8 on 64-bit platforms
    ga_init((&raw mut (*reader).buffer).cast(), 8, 32);
    (*reader).type_ = type_;
}

/// Stream close callback: decrements the channel refcount.
///
/// Signature matches `stream_close_cb`: `fn(stream: *mut Stream, data: *mut c_void)`.
///
/// # Safety
///
/// `data` must be a valid `Channel *` cast to `*mut c_void`.
#[no_mangle]
pub unsafe extern "C" fn close_cb(_stream: *mut c_void, data: *mut c_void) {
    channel_decref(data.cast::<ChannelT>());
}

/// Event callback: removes channel from map and destroys it.
///
/// Signature matches `argv_callback`: `fn(argv: *mut *mut c_void)`.
///
/// # Safety
///
/// `argv[0]` must be a valid `Channel *`.
#[no_mangle]
pub unsafe extern "C" fn free_channel_event(argv: *mut *mut c_void) {
    let chan = (*argv).cast::<ChannelT>();
    nvim_channels_del((*chan).id);
    channel_destroy(chan);
}

/// Internal helper: destroy a channel and free all its resources.
///
/// # Safety
///
/// `chan` must be a valid non-null pointer to a `Channel` that has been removed
/// from the channels map.
unsafe fn channel_destroy(chan: *mut ChannelT) {
    if (*chan).is_rpc {
        rpc_free(chan.cast());
    }
    if (*chan).streamtype == K_CHANNEL_STREAM_PROC {
        // proc is at offset 32 (start of stream union) -- the Proc struct is the first union member
        let proc_ptr = chan.cast::<u8>().add(32).cast::<c_void>();
        proc_free(proc_ptr);
    }
    callback_reader_free(&raw mut (*chan).on_data);
    callback_reader_free(&raw mut (*chan).on_stderr);
    callback_free(&raw mut (*chan).on_exit);
    multiqueue_free((*chan).events);
    xfree(chan.cast());
}

/// Simple int64_t comparison function for use with `qsort()`.
///
/// # Safety
///
/// `pa` and `pb` must be valid non-null pointers to `int64_t` values.
#[no_mangle]
pub unsafe extern "C" fn int64_t_cmp(pa: *const c_void, pb: *const c_void) -> c_int {
    let a = *pa.cast::<c_longlong>();
    let b = *pb.cast::<c_longlong>();
    match a.cmp(&b) {
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Less => -1,
    }
}

/// Initialize the channel module.
///
/// Allocates the stderr channel and initializes RPC.
///
/// # Safety
///
/// Must be called once during Nvim startup, after the main loop is initialized.
#[no_mangle]
pub unsafe extern "C" fn channel_init() {
    // kChannelStreamStderr = 3
    channel_alloc(3);
    rpc_init();
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Build an `EventT` with a single pointer argument.
///
/// Equivalent to `event_create(handler, arg)` C macro.
#[inline]
fn make_event(handler: unsafe extern "C" fn(*mut *mut c_void), arg: *mut c_void) -> EventT {
    let mut e = EventT {
        handler: Some(handler),
        argv: [std::ptr::null_mut(); 10],
    };
    e.argv[0] = arg;
    e
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_stream_type_from_c_int() {
        assert_eq!(
            ChannelStreamType::from_c_int(0),
            Some(ChannelStreamType::Proc)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(1),
            Some(ChannelStreamType::Socket)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(2),
            Some(ChannelStreamType::Stdio)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(3),
            Some(ChannelStreamType::Stderr)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(4),
            Some(ChannelStreamType::Internal)
        );
        assert_eq!(ChannelStreamType::from_c_int(99), None);
    }

    #[test]
    fn test_channel_part_from_c_int() {
        assert_eq!(ChannelPart::from_c_int(0), Some(ChannelPart::Stdin));
        assert_eq!(ChannelPart::from_c_int(1), Some(ChannelPart::Stdout));
        assert_eq!(ChannelPart::from_c_int(2), Some(ChannelPart::Stderr));
        assert_eq!(ChannelPart::from_c_int(3), Some(ChannelPart::Rpc));
        assert_eq!(ChannelPart::from_c_int(4), Some(ChannelPart::All));
        assert_eq!(ChannelPart::from_c_int(99), None);
    }

    #[test]
    fn test_client_type_from_c_int() {
        assert_eq!(ClientType::from_c_int(0), ClientType::Remote);
        assert_eq!(ClientType::from_c_int(1), ClientType::Ui);
        assert_eq!(ClientType::from_c_int(5), ClientType::MsgpackRpc);
        assert_eq!(ClientType::from_c_int(99), ClientType::Unknown);
        assert_eq!(ClientType::from_c_int(-1), ClientType::Unknown);
    }

    #[test]
    fn test_constants() {
        assert_eq!(CHAN_STDIO, 1);
        assert_eq!(CHAN_STDERR, 2);
    }

    #[test]
    fn test_int64_t_cmp() {
        let a: i64 = 1;
        let b: i64 = 2;
        let c: i64 = 1;
        unsafe {
            assert!(int64_t_cmp((&raw const a).cast(), (&raw const b).cast()) < 0);
            assert!(int64_t_cmp((&raw const b).cast(), (&raw const a).cast()) > 0);
            assert_eq!(int64_t_cmp((&raw const a).cast(), (&raw const c).cast()), 0);
        }
    }

    #[test]
    fn test_channel_t_size() {
        assert_eq!(std::mem::size_of::<ChannelT>(), 1928);
    }

    #[test]
    fn test_channel_t_offsets() {
        assert_eq!(std::mem::offset_of!(ChannelT, id), 0);
        assert_eq!(std::mem::offset_of!(ChannelT, refcount), 8);
        assert_eq!(std::mem::offset_of!(ChannelT, events), 16);
        assert_eq!(std::mem::offset_of!(ChannelT, streamtype), 24);
        assert_eq!(std::mem::offset_of!(ChannelT, stream), 32);
        assert_eq!(std::mem::offset_of!(ChannelT, is_rpc), 1672);
        assert_eq!(std::mem::offset_of!(ChannelT, detach), 1673);
        assert_eq!(std::mem::offset_of!(ChannelT, rpc), 1680);
        assert_eq!(std::mem::offset_of!(ChannelT, term), 1768);
        assert_eq!(std::mem::offset_of!(ChannelT, on_data), 1776);
        assert_eq!(std::mem::offset_of!(ChannelT, on_stderr), 1840);
        assert_eq!(std::mem::offset_of!(ChannelT, on_exit), 1904);
        assert_eq!(std::mem::offset_of!(ChannelT, exit_status), 1920);
        assert_eq!(std::mem::offset_of!(ChannelT, callback_busy), 1924);
        assert_eq!(std::mem::offset_of!(ChannelT, callback_scheduled), 1925);
    }

    #[test]
    fn test_event_t_size() {
        assert_eq!(std::mem::size_of::<EventT>(), 88);
    }
}
