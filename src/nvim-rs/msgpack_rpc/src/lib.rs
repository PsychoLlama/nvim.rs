//! MessagePack-RPC protocol for Neovim
//!
//! This crate provides Rust types and functions for the MessagePack-RPC protocol
//! used by Neovim for client-server communication.
//!
//! # MessagePack-RPC Protocol
//!
//! MessagePack-RPC messages are arrays with the following formats:
//!
//! - **Request**: `[0, msgid, method, params]`
//!   - msgid: Unique message ID for matching responses
//!   - method: Method name string
//!   - params: Array of arguments
//!
//! - **Response**: `[1, msgid, error, result]`
//!   - msgid: ID of the original request
//!   - error: Error object or nil
//!   - result: Result object or nil
//!
//! - **Notification**: `[2, method, params]`
//!   - method: Method name string
//!   - params: Array of arguments
//!
//! # Design
//!
//! This crate provides:
//! - Message type enums
//! - Request/response tracking types
//! - Server management types
//! - RPC initialization and lifecycle functions

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of server connections
pub const MAX_CONNECTIONS: usize = 32;

/// Maximum length for server addresses
pub const ADDRESS_MAX_SIZE: usize = 256;

/// Environment variable name for listen address (deprecated)
pub const ENV_LISTEN: &str = "NVIM_LISTEN_ADDRESS";

// =============================================================================
// Message Types
// =============================================================================

/// MessagePack-RPC message type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Request: [0, msgid, method, params]
    Request = 0,
    /// Response: [1, msgid, error, result]
    Response = 1,
    /// Notification: [2, method, params]
    Notification = 2,
    /// Special type for UI redraw events
    RedrawEvent = 3,
}

impl MessageType {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Request),
            1 => Some(Self::Response),
            2 => Some(Self::Notification),
            3 => Some(Self::RedrawEvent),
            _ => None,
        }
    }

    /// Check if this is a request or notification (methods that can be dispatched)
    #[must_use]
    pub const fn is_dispatchable(self) -> bool {
        matches!(self, Self::Request | Self::Notification)
    }

    /// Check if this requires a response
    #[must_use]
    pub const fn needs_response(self) -> bool {
        matches!(self, Self::Request)
    }
}

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to an Unpacker
///
/// The Unpacker manages msgpack stream parsing state.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnpackerHandle(*mut c_void);

impl UnpackerHandle {
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

/// Opaque handle to a PackerBuffer
///
/// The PackerBuffer manages msgpack serialization state.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackerBufferHandle(*mut c_void);

impl PackerBufferHandle {
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

/// Opaque handle to a SocketWatcher
///
/// SocketWatcher manages incoming connections on a server socket.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketWatcherHandle(*mut c_void);

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

/// Opaque handle to a MsgpackRpcRequestHandler
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestHandlerHandle(*mut c_void);

impl RequestHandlerHandle {
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
// C Accessor Functions
// =============================================================================

extern "C" {
    // Unpacker accessors
    fn nvim_unpacker_get_type(unpacker: UnpackerHandle) -> c_int;
    fn nvim_unpacker_get_request_id(unpacker: UnpackerHandle) -> u32;
    fn nvim_unpacker_get_state(unpacker: UnpackerHandle) -> c_int;
    fn nvim_unpacker_set_state(unpacker: UnpackerHandle, state: c_int);
    fn nvim_unpacker_is_closed(unpacker: UnpackerHandle) -> c_int;
    fn nvim_unpacker_get_read_ptr(unpacker: UnpackerHandle) -> *const c_char;
    fn nvim_unpacker_set_read_ptr(unpacker: UnpackerHandle, ptr: *const c_char);
    fn nvim_unpacker_get_read_size(unpacker: UnpackerHandle) -> usize;
    fn nvim_unpacker_set_read_size(unpacker: UnpackerHandle, size: usize);
    fn nvim_unpacker_get_method_name_len(unpacker: UnpackerHandle) -> usize;
    fn nvim_unpacker_has_grid_line_event(unpacker: UnpackerHandle) -> c_int;
    fn nvim_unpacker_get_handler_fn(unpacker: UnpackerHandle) -> *mut c_void;
    fn nvim_unpacker_get_handler_name(unpacker: UnpackerHandle) -> *const c_char;
    fn nvim_unpacker_is_handler_fast(unpacker: UnpackerHandle) -> c_int;

    // Server functions
    fn nvim_server_start(addr: *const c_char) -> c_int;
    fn nvim_server_stop(addr: *const c_char) -> c_int;
    fn nvim_server_address_new(name: *const c_char) -> *mut c_char;
    fn nvim_server_owns_pipe_address(addr: *const c_char) -> c_int;
}

// =============================================================================
// Rust Wrapper Functions - Unpacker
// =============================================================================

/// Get message type from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_type(unpacker: UnpackerHandle) -> c_int {
    if unpacker.is_null() {
        return -1;
    }
    nvim_unpacker_get_type(unpacker)
}

/// Get message type as enum from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle or null
#[must_use]
pub unsafe fn unpacker_get_message_type(unpacker: UnpackerHandle) -> Option<MessageType> {
    if unpacker.is_null() {
        return None;
    }
    MessageType::from_c_int(nvim_unpacker_get_type(unpacker))
}

/// Get request ID from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_request_id(unpacker: UnpackerHandle) -> u32 {
    if unpacker.is_null() {
        return 0;
    }
    nvim_unpacker_get_request_id(unpacker)
}

/// Get parser state from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_state(unpacker: UnpackerHandle) -> c_int {
    if unpacker.is_null() {
        return -1;
    }
    nvim_unpacker_get_state(unpacker)
}

/// Set parser state in unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_set_state(unpacker: UnpackerHandle, state: c_int) {
    if !unpacker.is_null() {
        nvim_unpacker_set_state(unpacker, state);
    }
}

/// Check if unpacker is in closed/error state
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_is_closed(unpacker: UnpackerHandle) -> c_int {
    if unpacker.is_null() {
        return 1;
    }
    nvim_unpacker_is_closed(unpacker)
}

/// Get read pointer from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_read_ptr(unpacker: UnpackerHandle) -> *const c_char {
    if unpacker.is_null() {
        return std::ptr::null();
    }
    nvim_unpacker_get_read_ptr(unpacker)
}

/// Set read pointer in unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle and `ptr` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_set_read_ptr(unpacker: UnpackerHandle, ptr: *const c_char) {
    if !unpacker.is_null() {
        nvim_unpacker_set_read_ptr(unpacker, ptr);
    }
}

/// Get read size from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_read_size(unpacker: UnpackerHandle) -> usize {
    if unpacker.is_null() {
        return 0;
    }
    nvim_unpacker_get_read_size(unpacker)
}

/// Set read size in unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_set_read_size(unpacker: UnpackerHandle, size: usize) {
    if !unpacker.is_null() {
        nvim_unpacker_set_read_size(unpacker, size);
    }
}

/// Get method name length from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_method_name_len(unpacker: UnpackerHandle) -> usize {
    if unpacker.is_null() {
        return 0;
    }
    nvim_unpacker_get_method_name_len(unpacker)
}

/// Check if unpacker has a grid line event
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_has_grid_line_event(unpacker: UnpackerHandle) -> c_int {
    if unpacker.is_null() {
        return 0;
    }
    nvim_unpacker_has_grid_line_event(unpacker)
}

/// Get handler function pointer from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_handler_fn(unpacker: UnpackerHandle) -> *mut c_void {
    if unpacker.is_null() {
        return std::ptr::null_mut();
    }
    nvim_unpacker_get_handler_fn(unpacker)
}

/// Get handler name from unpacker
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_get_handler_name(unpacker: UnpackerHandle) -> *const c_char {
    if unpacker.is_null() {
        return std::ptr::null();
    }
    nvim_unpacker_get_handler_name(unpacker)
}

/// Check if handler is fast (can be invoked immediately)
///
/// # Safety
///
/// `unpacker` must be a valid Unpacker handle
#[no_mangle]
pub unsafe extern "C" fn rs_unpacker_is_handler_fast(unpacker: UnpackerHandle) -> c_int {
    if unpacker.is_null() {
        return 0;
    }
    nvim_unpacker_is_handler_fast(unpacker)
}

// =============================================================================
// Rust Wrapper Functions - Server
// =============================================================================

/// Start an RPC server on the given address
///
/// Returns:
/// - 0: success
/// - 1: validation error (empty/null address)
/// - 2: already listening on this address
/// - negative: libuv error code
///
/// # Safety
///
/// `addr` must be a valid null-terminated string or null
#[no_mangle]
pub unsafe extern "C" fn rs_server_start(addr: *const c_char) -> c_int {
    nvim_server_start(addr)
}

/// Stop an RPC server at the given address
///
/// Returns 1 if successful, 0 if not listening on that address.
///
/// # Safety
///
/// `addr` must be a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_server_stop(addr: *const c_char) -> c_int {
    nvim_server_stop(addr)
}

/// Generate a new unique server address
///
/// The returned string must be freed by the caller.
///
/// # Safety
///
/// `name` can be null for auto-generated name, or a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_server_address_new(name: *const c_char) -> *mut c_char {
    nvim_server_address_new(name)
}

/// Check if this Neovim instance owns the given pipe address
///
/// # Safety
///
/// `addr` must be a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_server_owns_pipe_address(addr: *const c_char) -> c_int {
    nvim_server_owns_pipe_address(addr)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Parse a MessagePack-RPC message type from the type field value
///
/// Returns the MessageType corresponding to the value, or None if invalid.
#[must_use]
pub const fn parse_message_type(type_val: u32) -> Option<MessageType> {
    match type_val {
        0 => Some(MessageType::Request),
        1 => Some(MessageType::Response),
        2 => Some(MessageType::Notification),
        _ => None,
    }
}

/// Get the expected array length for a message type
///
/// - Request: 4 elements [type, msgid, method, params]
/// - Response: 4 elements [type, msgid, error, result]
/// - Notification: 3 elements [type, method, params]
#[must_use]
pub const fn message_array_length(msg_type: MessageType) -> usize {
    match msg_type {
        MessageType::Request | MessageType::Response => 4,
        MessageType::Notification | MessageType::RedrawEvent => 3,
    }
}

/// Check if a MessagePack array length is valid for the message type
#[must_use]
pub const fn is_valid_array_length(msg_type: MessageType, length: usize) -> bool {
    length == message_array_length(msg_type)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_from_c_int() {
        assert_eq!(MessageType::from_c_int(0), Some(MessageType::Request));
        assert_eq!(MessageType::from_c_int(1), Some(MessageType::Response));
        assert_eq!(MessageType::from_c_int(2), Some(MessageType::Notification));
        assert_eq!(MessageType::from_c_int(3), Some(MessageType::RedrawEvent));
        assert_eq!(MessageType::from_c_int(99), None);
    }

    #[test]
    fn test_message_type_is_dispatchable() {
        assert!(MessageType::Request.is_dispatchable());
        assert!(MessageType::Notification.is_dispatchable());
        assert!(!MessageType::Response.is_dispatchable());
        assert!(!MessageType::RedrawEvent.is_dispatchable());
    }

    #[test]
    fn test_message_type_needs_response() {
        assert!(MessageType::Request.needs_response());
        assert!(!MessageType::Response.needs_response());
        assert!(!MessageType::Notification.needs_response());
        assert!(!MessageType::RedrawEvent.needs_response());
    }

    #[test]
    fn test_parse_message_type() {
        assert_eq!(parse_message_type(0), Some(MessageType::Request));
        assert_eq!(parse_message_type(1), Some(MessageType::Response));
        assert_eq!(parse_message_type(2), Some(MessageType::Notification));
        assert_eq!(parse_message_type(99), None);
    }

    #[test]
    fn test_message_array_length() {
        assert_eq!(message_array_length(MessageType::Request), 4);
        assert_eq!(message_array_length(MessageType::Response), 4);
        assert_eq!(message_array_length(MessageType::Notification), 3);
        assert_eq!(message_array_length(MessageType::RedrawEvent), 3);
    }

    #[test]
    fn test_is_valid_array_length() {
        assert!(is_valid_array_length(MessageType::Request, 4));
        assert!(!is_valid_array_length(MessageType::Request, 3));
        assert!(is_valid_array_length(MessageType::Notification, 3));
        assert!(!is_valid_array_length(MessageType::Notification, 4));
    }

    #[test]
    fn test_unpacker_handle_null() {
        let handle = UnpackerHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_packer_buffer_handle_null() {
        let handle = PackerBufferHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_socket_watcher_handle_null() {
        let handle = SocketWatcherHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CONNECTIONS, 32);
        assert_eq!(ADDRESS_MAX_SIZE, 256);
        assert_eq!(ENV_LISTEN, "NVIM_LISTEN_ADDRESS");
    }
}
