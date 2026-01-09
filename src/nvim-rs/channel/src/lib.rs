//! Channel management for Neovim RPC
//!
//! This module provides Rust wrappers for Neovim's channel infrastructure,
//! which handles communication between Neovim and external processes (plugins,
//! GUIs, remote clients).
//!
//! # Design
//!
//! The approach is to wrap the existing C channel infrastructure rather than
//! replace it. This provides:
//! - Opaque handle types for Channel, RpcState, etc.
//! - Accessor functions for channel state
//! - Type-safe enums for channel types and parts
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

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Channel ID for stdio
pub const CHAN_STDIO: u64 = 1;

/// Channel ID for stderr
pub const CHAN_STDERR: u64 = 2;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a Channel
///
/// Channels manage communication with external processes, sockets, or stdio.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelHandle(*mut c_void);

impl ChannelHandle {
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

    /// Get the raw pointer
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to RpcState
///
/// RpcState manages the msgpack-rpc protocol state for a channel.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpcStateHandle(*mut c_void);

impl RpcStateHandle {
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

/// Opaque handle to CallbackReader
///
/// CallbackReader manages data callbacks for channel I/O.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallbackReaderHandle(*mut c_void);

impl CallbackReaderHandle {
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
// C Accessor Functions (defined in channel.c or accessor file)
// =============================================================================

extern "C" {
    // Channel accessors
    fn nvim_channel_get_id(chan: ChannelHandle) -> u64;
    fn nvim_channel_get_refcount(chan: ChannelHandle) -> usize;
    fn nvim_channel_get_streamtype(chan: ChannelHandle) -> c_int;
    fn nvim_channel_is_rpc(chan: ChannelHandle) -> c_int;
    fn nvim_channel_get_detach(chan: ChannelHandle) -> c_int;
    fn nvim_channel_set_detach(chan: ChannelHandle, detach: c_int);
    fn nvim_channel_get_exit_status(chan: ChannelHandle) -> c_int;
    fn nvim_channel_set_exit_status(chan: ChannelHandle, status: c_int);
    fn nvim_channel_get_callback_busy(chan: ChannelHandle) -> c_int;
    fn nvim_channel_set_callback_busy(chan: ChannelHandle, busy: c_int);
    fn nvim_channel_get_callback_scheduled(chan: ChannelHandle) -> c_int;
    fn nvim_channel_set_callback_scheduled(chan: ChannelHandle, scheduled: c_int);

    // Channel functions from C
    fn nvim_find_channel(id: u64) -> ChannelHandle;
    fn nvim_channel_incref(chan: ChannelHandle);
    fn nvim_channel_decref(chan: ChannelHandle);

    // RpcState accessors
    fn nvim_rpc_state_is_closed(rpc: RpcStateHandle) -> c_int;
    fn nvim_rpc_state_set_closed(rpc: RpcStateHandle, closed: c_int);
    fn nvim_rpc_state_get_next_request_id(rpc: RpcStateHandle) -> u32;
    fn nvim_rpc_state_set_next_request_id(rpc: RpcStateHandle, id: u32);
    fn nvim_rpc_state_get_client_type(rpc: RpcStateHandle) -> c_int;
    fn nvim_rpc_state_set_client_type(rpc: RpcStateHandle, client_type: c_int);

    // CallbackReader accessors
    fn nvim_callback_reader_is_set(reader: CallbackReaderHandle) -> c_int;
    fn nvim_callback_reader_get_eof(reader: CallbackReaderHandle) -> c_int;
    fn nvim_callback_reader_set_eof(reader: CallbackReaderHandle, eof: c_int);
    fn nvim_callback_reader_get_buffered(reader: CallbackReaderHandle) -> c_int;
    fn nvim_callback_reader_get_fwd_err(reader: CallbackReaderHandle) -> c_int;
    fn nvim_callback_reader_get_type(reader: CallbackReaderHandle) -> *const c_char;
}

// =============================================================================
// Rust Wrapper Functions - Channel
// =============================================================================

/// Get the channel ID
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_id(chan: ChannelHandle) -> u64 {
    if chan.is_null() {
        return 0;
    }
    nvim_channel_get_id(chan)
}

/// Get the channel reference count
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_refcount(chan: ChannelHandle) -> usize {
    if chan.is_null() {
        return 0;
    }
    nvim_channel_get_refcount(chan)
}

/// Get the channel stream type
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_streamtype(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return -1;
    }
    nvim_channel_get_streamtype(chan)
}

/// Get the channel stream type as enum
///
/// Returns None if the handle is null or stream type is invalid.
///
/// # Safety
///
/// `chan` must be a valid Channel handle or null
#[must_use]
pub unsafe fn channel_get_streamtype_enum(chan: ChannelHandle) -> Option<ChannelStreamType> {
    if chan.is_null() {
        return None;
    }
    ChannelStreamType::from_c_int(nvim_channel_get_streamtype(chan))
}

/// Check if channel uses RPC
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_is_rpc(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    nvim_channel_is_rpc(chan)
}

/// Get the detach flag
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_detach(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    nvim_channel_get_detach(chan)
}

/// Set the detach flag
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_set_detach(chan: ChannelHandle, detach: c_int) {
    if !chan.is_null() {
        nvim_channel_set_detach(chan, detach);
    }
}

/// Get the exit status
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_exit_status(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return -1;
    }
    nvim_channel_get_exit_status(chan)
}

/// Set the exit status
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_set_exit_status(chan: ChannelHandle, status: c_int) {
    if !chan.is_null() {
        nvim_channel_set_exit_status(chan, status);
    }
}

/// Get the callback_busy flag
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_callback_busy(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    nvim_channel_get_callback_busy(chan)
}

/// Set the callback_busy flag
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_set_callback_busy(chan: ChannelHandle, busy: c_int) {
    if !chan.is_null() {
        nvim_channel_set_callback_busy(chan, busy);
    }
}

/// Get the callback_scheduled flag
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_get_callback_scheduled(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    nvim_channel_get_callback_scheduled(chan)
}

/// Set the callback_scheduled flag
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_set_callback_scheduled(chan: ChannelHandle, scheduled: c_int) {
    if !chan.is_null() {
        nvim_channel_set_callback_scheduled(chan, scheduled);
    }
}

/// Find a channel by ID
///
/// # Safety
///
/// This function accesses the global channels map.
#[no_mangle]
pub unsafe extern "C" fn rs_find_channel(id: u64) -> ChannelHandle {
    nvim_find_channel(id)
}

/// Increment channel reference count
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_incref(chan: ChannelHandle) {
    if !chan.is_null() {
        nvim_channel_incref(chan);
    }
}

/// Decrement channel reference count
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_decref(chan: ChannelHandle) {
    if !chan.is_null() {
        nvim_channel_decref(chan);
    }
}

/// Check if a channel is a process channel
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_is_proc(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    c_int::from(nvim_channel_get_streamtype(chan) == ChannelStreamType::Proc as c_int)
}

/// Check if a channel is a socket channel
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_is_socket(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    c_int::from(nvim_channel_get_streamtype(chan) == ChannelStreamType::Socket as c_int)
}

/// Check if a channel is a stdio channel
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_is_stdio(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    c_int::from(nvim_channel_get_streamtype(chan) == ChannelStreamType::Stdio as c_int)
}

/// Check if a channel is an internal channel
///
/// # Safety
///
/// `chan` must be a valid Channel handle
#[no_mangle]
pub unsafe extern "C" fn rs_channel_is_internal(chan: ChannelHandle) -> c_int {
    if chan.is_null() {
        return 0;
    }
    c_int::from(nvim_channel_get_streamtype(chan) == ChannelStreamType::Internal as c_int)
}

// =============================================================================
// Rust Wrapper Functions - RpcState
// =============================================================================

/// Check if RPC state is closed
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_is_closed(rpc: RpcStateHandle) -> c_int {
    if rpc.is_null() {
        return 1;
    }
    nvim_rpc_state_is_closed(rpc)
}

/// Set RPC state closed flag
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_set_closed(rpc: RpcStateHandle, closed: c_int) {
    if !rpc.is_null() {
        nvim_rpc_state_set_closed(rpc, closed);
    }
}

/// Get next request ID from RPC state
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_get_next_request_id(rpc: RpcStateHandle) -> u32 {
    if rpc.is_null() {
        return 0;
    }
    nvim_rpc_state_get_next_request_id(rpc)
}

/// Set next request ID in RPC state
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_set_next_request_id(rpc: RpcStateHandle, id: u32) {
    if !rpc.is_null() {
        nvim_rpc_state_set_next_request_id(rpc, id);
    }
}

/// Increment and get the next request ID
///
/// Returns the current ID and increments for the next call.
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_next_request_id(rpc: RpcStateHandle) -> u32 {
    if rpc.is_null() {
        return 0;
    }
    let id = nvim_rpc_state_get_next_request_id(rpc);
    nvim_rpc_state_set_next_request_id(rpc, id.wrapping_add(1));
    id
}

/// Get client type from RPC state
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_get_client_type(rpc: RpcStateHandle) -> c_int {
    if rpc.is_null() {
        return ClientType::Unknown as c_int;
    }
    nvim_rpc_state_get_client_type(rpc)
}

/// Set client type in RPC state
///
/// # Safety
///
/// `rpc` must be a valid RpcState handle
#[no_mangle]
pub unsafe extern "C" fn rs_rpc_state_set_client_type(rpc: RpcStateHandle, client_type: c_int) {
    if !rpc.is_null() {
        nvim_rpc_state_set_client_type(rpc, client_type);
    }
}

// =============================================================================
// Rust Wrapper Functions - CallbackReader
// =============================================================================

/// Check if callback reader is set (has callback or self)
///
/// # Safety
///
/// `reader` must be a valid CallbackReader handle
#[no_mangle]
pub unsafe extern "C" fn rs_callback_reader_is_set(reader: CallbackReaderHandle) -> c_int {
    if reader.is_null() {
        return 0;
    }
    nvim_callback_reader_is_set(reader)
}

/// Get EOF flag from callback reader
///
/// # Safety
///
/// `reader` must be a valid CallbackReader handle
#[no_mangle]
pub unsafe extern "C" fn rs_callback_reader_get_eof(reader: CallbackReaderHandle) -> c_int {
    if reader.is_null() {
        return 0;
    }
    nvim_callback_reader_get_eof(reader)
}

/// Set EOF flag on callback reader
///
/// # Safety
///
/// `reader` must be a valid CallbackReader handle
#[no_mangle]
pub unsafe extern "C" fn rs_callback_reader_set_eof(reader: CallbackReaderHandle, eof: c_int) {
    if !reader.is_null() {
        nvim_callback_reader_set_eof(reader, eof);
    }
}

/// Get buffered flag from callback reader
///
/// # Safety
///
/// `reader` must be a valid CallbackReader handle
#[no_mangle]
pub unsafe extern "C" fn rs_callback_reader_get_buffered(reader: CallbackReaderHandle) -> c_int {
    if reader.is_null() {
        return 0;
    }
    nvim_callback_reader_get_buffered(reader)
}

/// Get fwd_err flag from callback reader
///
/// # Safety
///
/// `reader` must be a valid CallbackReader handle
#[no_mangle]
pub unsafe extern "C" fn rs_callback_reader_get_fwd_err(reader: CallbackReaderHandle) -> c_int {
    if reader.is_null() {
        return 0;
    }
    nvim_callback_reader_get_fwd_err(reader)
}

/// Get type string from callback reader
///
/// # Safety
///
/// `reader` must be a valid CallbackReader handle
#[no_mangle]
pub unsafe extern "C" fn rs_callback_reader_get_type(
    reader: CallbackReaderHandle,
) -> *const c_char {
    if reader.is_null() {
        return std::ptr::null();
    }
    nvim_callback_reader_get_type(reader)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Parse client type from string
///
/// # Safety
///
/// `type_str` must be a valid null-terminated string or null
#[no_mangle]
pub unsafe extern "C" fn rs_parse_client_type(type_str: *const c_char) -> c_int {
    if type_str.is_null() {
        return ClientType::Remote as c_int;
    }

    let cstr = std::ffi::CStr::from_ptr(type_str);
    match cstr.to_bytes() {
        b"remote" => ClientType::Remote as c_int,
        b"msgpack-rpc" => ClientType::MsgpackRpc as c_int,
        b"ui" => ClientType::Ui as c_int,
        b"embedder" => ClientType::Embedder as c_int,
        b"host" => ClientType::Host as c_int,
        b"plugin" => ClientType::Plugin as c_int,
        _ => ClientType::Unknown as c_int,
    }
}

/// Check if channel ID is valid (not reserved)
///
/// Returns 1 if the ID is usable for regular channels, 0 if reserved.
#[no_mangle]
pub extern "C" fn rs_channel_id_is_valid(id: u64) -> c_int {
    c_int::from(id > CHAN_STDERR)
}

/// Check if channel ID is the stdio channel
#[no_mangle]
pub extern "C" fn rs_channel_id_is_stdio(id: u64) -> c_int {
    c_int::from(id == CHAN_STDIO)
}

/// Check if channel ID is the stderr channel
#[no_mangle]
pub extern "C" fn rs_channel_id_is_stderr(id: u64) -> c_int {
    c_int::from(id == CHAN_STDERR)
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
    fn test_channel_handle_null() {
        let handle = ChannelHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_rpc_state_handle_null() {
        let handle = RpcStateHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_callback_reader_handle_null() {
        let handle = CallbackReaderHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_channel_id_checks() {
        assert_eq!(rs_channel_id_is_stdio(1), 1);
        assert_eq!(rs_channel_id_is_stdio(2), 0);
        assert_eq!(rs_channel_id_is_stderr(2), 1);
        assert_eq!(rs_channel_id_is_stderr(1), 0);
        assert_eq!(rs_channel_id_is_valid(1), 0);
        assert_eq!(rs_channel_id_is_valid(2), 0);
        assert_eq!(rs_channel_id_is_valid(3), 1);
    }

    #[test]
    fn test_constants() {
        assert_eq!(CHAN_STDIO, 1);
        assert_eq!(CHAN_STDERR, 2);
    }
}
