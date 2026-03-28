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

    // --- Phase 2 FFI ---

    /// Allocate a VimL list. Defined in eval/typval.c.
    /// kListLenMayKnow = -3
    fn tv_list_alloc(len: isize) -> *mut c_void;
    /// Append a string item to a list. Defined in eval/typval.c.
    fn tv_list_append_string(l: *mut c_void, s: *const std::ffi::c_char, len: isize);
    /// Write bytes to a list (encode_list_write). Defined in eval/encode.c.
    fn encode_list_write(data: *mut c_void, buf: *const std::ffi::c_char, len: usize);
    /// Increment list reference count. Defined in eval_shim.c (nvim_tv_list_ref shim).
    fn nvim_tv_list_ref(l: *mut c_void);
    /// Find a key in a dict. Defined in eval/typval.c.
    fn tv_dict_find(d: *mut c_void, key: *const std::ffi::c_char, len: isize) -> *mut c_void;
    /// Add a list to a dict. Defined in eval/typval.c.
    fn tv_dict_add_list(
        d: *mut c_void,
        key: *const std::ffi::c_char,
        key_len: usize,
        list: *mut c_void,
    ) -> c_int;
    /// Emit an error message (variadic). Defined in message.c.
    fn semsg(fmt: *const std::ffi::c_char, ...) -> bool;
    /// Call a Callback with arguments. Defined in eval.c.
    fn callback_call(
        cb: *mut CallbackT,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> bool;
    /// Clear a typval_T value. Defined in eval/typval.c.
    fn tv_clear(tv: *mut c_void);
    /// Append bytes to a garray. Defined in garray.c.
    fn ga_concat_len(ga: *mut c_void, s: *const std::ffi::c_char, len: usize);
    /// Compute UTF-8 sequence length with upper bound. Defined in mbyte.c.
    fn utf_ptr2len_len(p: *const std::ffi::c_char, size: c_int) -> c_int;
    /// Feed data into a terminal. Defined in terminal.c.
    fn terminal_receive(term: *mut c_void, data: *const std::ffi::c_char, len: usize);
    /// Close a terminal. Defined in terminal.c.
    fn terminal_close(termpp: *mut *mut c_void, status: c_int);

    /// e_streamkey error string. Defined in errors.h.
    static e_streamkey: std::ffi::c_char;

    // --- Phase 3 FFI ---

    /// Check whether any autocmds are registered for an event. Defined in autocmd.c.
    fn has_event(event: c_int) -> c_int;
    /// Deferred autocommand handler for channel info changes. Defined in channel.c (stays in C).
    fn set_info_event(argv: *mut *mut c_void);
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

/// Destroy a channel and free all its resources.
///
/// Called from `free_channel_event` (Rust) and `channel_free_all_mem` (C).
///
/// # Safety
///
/// `chan` must be a valid non-null pointer to a `Channel` that has been removed
/// from the channels map.
#[no_mangle]
pub unsafe extern "C" fn channel_destroy(chan: *mut ChannelT) {
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
// Migrated functions (Phase 3): lifecycle helpers
// =============================================================================

// event_T enum values (from auevents_enum.generated.h)
const EVENT_CHANINFO: c_int = 23;
const EVENT_CHANOPEN: c_int = 24;

/// Fire a ChanOpen or ChanInfo autocommand if any are registered.
///
/// `new_chan` = true  => EVENT_CHANOPEN
/// `new_chan` = false => EVENT_CHANINFO
///
/// # Safety
///
/// `chan` must be a valid non-null `Channel *`. Calls into the event loop.
#[no_mangle]
pub unsafe extern "C" fn channel_info_changed(chan: *mut ChannelT, new_chan: bool) {
    let event = if new_chan {
        EVENT_CHANOPEN
    } else {
        EVENT_CHANINFO
    };
    if has_event(event) != 0 {
        channel_incref(chan);
        // queue a two-argument event: (chan, event)
        let mut event_argv: EventT = EventT {
            handler: Some(set_info_event as _),
            argv: [std::ptr::null_mut(); 10],
        };
        event_argv.argv[0] = chan.cast();
        // pass event value as pointer-sized integer (matches the C `(void *)(intptr_t)event`)
        // event is always EVENT_CHANINFO (23) or EVENT_CHANOPEN (24), both positive
        #[allow(clippy::cast_sign_loss)]
        let event_ptr: *mut c_void = (event as usize) as *mut c_void;
        event_argv.argv[1] = event_ptr;
        multiqueue_put_event(rs_loop_get_events(main_loop_ptr()), event_argv);
    }
}

// =============================================================================
// Migrated functions (Phase 2): callback/event processing chain
// =============================================================================

/// Convert a binary byte array to a readfile()-style VimL list.
///
/// The list always starts with an empty string `['']`, and newlines within
/// `buf` delimit additional list entries.
///
/// # Safety
///
/// `buf` must point to `len` valid bytes (or be null if `len == 0`).
// Pointer to empty C string for use in buffer_to_tv_list
const EMPTY_CSTR: *const std::ffi::c_char = c"".as_ptr();
// Pointer to "exit\0" C string for use in channel_callback_call
const EXIT_CSTR: *const std::ffi::c_char = c"exit".as_ptr();

unsafe fn buffer_to_tv_list(buf: *const std::ffi::c_char, len: usize) -> *mut c_void {
    // kListLenMayKnow = -3
    let l = tv_list_alloc(-3isize);
    // Empty buffer represented as [''] - same as tv_list_append_string(l, "", 0)
    tv_list_append_string(l, EMPTY_CSTR, 0);
    if len > 0 {
        encode_list_write(l, buf, len);
    }
    l
}

/// Schedule a deferred `on_channel_event` call for the channel.
///
/// This is a static helper; it does not need to be exported.
///
/// # Safety
///
/// `chan` must be a valid pointer to a `Channel`.
unsafe fn schedule_channel_event(chan: *mut ChannelT) {
    if !(*chan).callback_scheduled {
        if !(*chan).callback_busy {
            let event = make_event(on_channel_event as _, chan.cast());
            multiqueue_put_event((*chan).events, event);
            channel_incref(chan);
        }
        (*chan).callback_scheduled = true;
    }
}

/// Deferred event callback: processes buffered channel events.
///
/// Signature matches `argv_callback`.
///
/// # Safety
///
/// `argv[0]` must be a valid `Channel *`.
#[no_mangle]
pub unsafe extern "C" fn on_channel_event(argv: *mut *mut c_void) {
    let chan = (*argv).cast::<ChannelT>();

    (*chan).callback_busy = true;
    (*chan).callback_scheduled = false;

    let exit_status = (*chan).exit_status;
    channel_reader_callbacks(chan, &raw mut (*chan).on_data);
    channel_reader_callbacks(chan, &raw mut (*chan).on_stderr);
    if exit_status > -1 {
        channel_callback_call(chan, std::ptr::null_mut());
        (*chan).exit_status = -1;
    }

    (*chan).callback_busy = false;
    if (*chan).callback_scheduled {
        // further callback was deferred to avoid recursion
        let event = make_event(on_channel_event as _, chan.cast());
        multiqueue_put_event((*chan).events, event);
        channel_incref(chan);
    }

    channel_decref(chan);
}

/// Dispatch reader callbacks (buffered or unbuffered).
///
/// # Safety
///
/// `chan` and `reader` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn channel_reader_callbacks(
    chan: *mut ChannelT,
    reader: *mut CallbackReaderT,
) {
    if (*reader).buffered {
        if (*reader).eof {
            if (*reader).self_.is_null() {
                channel_callback_call(chan, reader);
            } else if tv_dict_find((*reader).self_, (*reader).type_, -1).is_null() {
                let buf_len = (*reader).buffer.ga_len;
                let data = buffer_to_tv_list(
                    (*reader).buffer.ga_data.cast(),
                    if buf_len > 0 {
                        buf_len.unsigned_abs() as usize
                    } else {
                        0
                    },
                );
                let key_len = libc_strlen((*reader).type_);
                tv_dict_add_list((*reader).self_, (*reader).type_, key_len, data);
            } else {
                // e_streamkey is a format string with %s and %" PRIu64
                semsg(&raw const e_streamkey, (*reader).type_, (*chan).id);
            }
            (*reader).eof = false;
        }
    } else {
        let is_eof = (*reader).eof;
        if (*reader).buffer.ga_len > 0 {
            channel_callback_call(chan, reader);
        }
        // if the stream reached eof, invoke extra callback with no data
        if is_eof {
            channel_callback_call(chan, reader);
            (*reader).eof = false;
        }
    }
}

/// Build typval arguments and call the channel's callback.
///
/// If `reader` is non-null, builds a list from the reader's buffer and calls
/// the reader's callback. Otherwise calls `on_exit` with the exit status.
///
/// # Safety
///
/// `chan` must be valid. `reader` may be null (exit callback case).
#[no_mangle]
pub unsafe extern "C" fn channel_callback_call(chan: *mut ChannelT, reader: *mut CallbackReaderT) {
    use nvim_eval::typval::TypvalT;

    // Stack-allocate 3 typval_T arguments (16 bytes each)
    let mut argv: [TypvalT; 3] = [
        TypvalT {
            v_type: 0,
            v_lock: 0,
            vval: nvim_eval::typval::TypvalVval { v_number: 0 },
        },
        TypvalT {
            v_type: 0,
            v_lock: 0,
            vval: nvim_eval::typval::TypvalVval { v_number: 0 },
        },
        TypvalT {
            v_type: 0,
            v_lock: 0,
            vval: nvim_eval::typval::TypvalVval { v_number: 0 },
        },
    ];

    // argv[0] = channel id (VAR_NUMBER = 1)
    argv[0].v_type = 1; // VAR_NUMBER
    argv[0].v_lock = 0; // VAR_UNLOCKED
    argv[0].vval.v_number = i64::from_ne_bytes((*chan).id.to_ne_bytes());

    let cb: *mut CallbackT = if reader.is_null() {
        // exit callback: argv[1] = exit status (VAR_NUMBER = 1)
        argv[1].v_type = 1; // VAR_NUMBER
        argv[1].v_lock = 0; // VAR_UNLOCKED
        argv[1].vval.v_number = i64::from((*chan).exit_status);
        // argv[2] = "exit" string (VAR_STRING = 2)
        argv[2].vval.v_string = EXIT_CSTR.cast_mut();
        &raw mut (*chan).on_exit
    } else {
        // argv[1] = list (VAR_LIST = 4)
        let buf_len = (*reader).buffer.ga_len;
        let list = buffer_to_tv_list(
            (*reader).buffer.ga_data.cast(),
            if buf_len > 0 {
                buf_len.unsigned_abs() as usize
            } else {
                0
            },
        );
        nvim_tv_list_ref(list);
        ga_clear((&raw mut (*reader).buffer).cast());
        argv[1].v_type = 4; // VAR_LIST
        argv[1].v_lock = 0; // VAR_UNLOCKED
        argv[1].vval.v_list = list;
        // argv[2] = type string (VAR_STRING = 2)
        argv[2].vval.v_string = (*reader).type_.cast_mut();
        &raw mut (*reader).cb
    };

    argv[2].v_type = 2; // VAR_STRING
    argv[2].v_lock = 0; // VAR_UNLOCKED

    // rettv as a zero-initialized typval_T on the stack (VAR_UNKNOWN = 0)
    let mut rettv = TypvalT {
        v_type: 0,
        v_lock: 0,
        vval: nvim_eval::typval::TypvalVval { v_number: 0 },
    };
    callback_call(cb, 3, argv.as_mut_ptr().cast(), (&raw mut rettv).cast());
    tv_clear((&raw mut rettv).cast());
}

/// Handle incoming data from a channel stream.
///
/// If the channel has a terminal, feeds data to it (handling incomplete UTF-8).
/// If a callback reader is set, buffers the data and schedules the callback.
///
/// Returns number of bytes consumed.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn on_channel_output_impl(
    chan: *mut ChannelT,
    buf: *const std::ffi::c_char,
    mut count: usize,
    eof: bool,
    reader: *mut CallbackReaderT,
) -> usize {
    if !(*chan).term.is_null() {
        if count > 0 {
            let mut p = buf;
            let end = buf.add(count);
            while p < end {
                // end.offset_from(p) is always positive here (p < end)
                let remaining = c_int::try_from(end.offset_from(p)).unwrap_or(c_int::MAX);
                let clen = utf_ptr2len_len(p, remaining);
                if clen > remaining {
                    count = p.offset_from(buf).unsigned_abs();
                    break;
                }
                p = p.add(clen.unsigned_abs() as usize);
            }
        }
        terminal_receive((*chan).term, buf, count);
    }

    if eof {
        (*reader).eof = true;
    }

    // callback_reader_set: cb.type != kCallbackNone || self != NULL
    if (*reader).cb.cb_type != 0 || !(*reader).self_.is_null() {
        ga_concat_len((&raw mut (*reader).buffer).cast(), buf, count);
        schedule_channel_event(chan);
    }

    count
}

/// RStream read callback for stdout data.
///
/// Signature matches `stream_read_cb`.
///
/// # Safety
///
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn on_channel_data(
    _stream: *mut c_void,
    buf: *const std::ffi::c_char,
    count: usize,
    data: *mut c_void,
    eof: bool,
) -> usize {
    let chan = data.cast::<ChannelT>();
    on_channel_output_impl(chan, buf, count, eof, &raw mut (*chan).on_data)
}

/// RStream read callback for stderr data.
///
/// Signature matches `stream_read_cb`.
///
/// # Safety
///
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn on_job_stderr(
    _stream: *mut c_void,
    buf: *const std::ffi::c_char,
    count: usize,
    data: *mut c_void,
    eof: bool,
) -> usize {
    let chan = data.cast::<ChannelT>();
    on_channel_output_impl(chan, buf, count, eof, &raw mut (*chan).on_stderr)
}

/// Process exit callback for proc channels.
///
/// Signature matches `proc_exit_cb`: `fn(proc: *mut Proc, status: c_int, data: *mut c_void)`.
///
/// # Safety
///
/// `data` must be a valid `Channel *`.
#[no_mangle]
pub unsafe extern "C" fn channel_proc_exit_cb(
    _proc: *mut c_void,
    status: c_int,
    data: *mut c_void,
) {
    let chan = data.cast::<ChannelT>();
    if !(*chan).term.is_null() {
        terminal_close(&raw mut (*chan).term, status);
    }

    // If process did not exit, we only closed the handle of a detached process.
    let exited = status >= 0;
    if exited && (*chan).on_exit.cb_type != 0 {
        // kCallbackNone = 0
        schedule_channel_event(chan);
        (*chan).exit_status = status;
    }

    channel_decref(chan);
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Compute `strlen` for a C string pointer (safe wrapper).
#[inline]
unsafe fn libc_strlen(s: *const std::ffi::c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

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
