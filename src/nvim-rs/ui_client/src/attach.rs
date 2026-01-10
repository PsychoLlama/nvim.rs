//! Client attachment/detachment
//!
//! This module provides infrastructure for UI client connection
//! management, including attachment, detachment, and lifecycle.

use std::ffi::c_int;

use crate::UiClientOptions;

// =============================================================================
// Attachment Status
// =============================================================================

/// Status of attachment operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttachStatus {
    /// Attachment successful
    #[default]
    Ok = 0,
    /// Already attached
    AlreadyAttached = 1,
    /// Invalid options
    InvalidOptions = 2,
    /// Connection failed
    ConnectionFailed = 3,
    /// Protocol error
    ProtocolError = 4,
    /// Not supported
    NotSupported = 5,
    /// Resource limit reached
    ResourceLimit = 6,
    /// Permission denied
    PermissionDenied = 7,
}

impl AttachStatus {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            1 => Self::AlreadyAttached,
            2 => Self::InvalidOptions,
            3 => Self::ConnectionFailed,
            5 => Self::NotSupported,
            6 => Self::ResourceLimit,
            7 => Self::PermissionDenied,
            // 4 and unrecognized values
            _ => Self::ProtocolError,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if successful.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if retriable.
    #[must_use]
    pub const fn is_retriable(self) -> bool {
        matches!(self, Self::ConnectionFailed | Self::ResourceLimit)
    }
}

/// FFI: Check if attachment successful.
#[no_mangle]
pub extern "C" fn rs_attach_status_is_success(status: c_int) -> c_int {
    c_int::from(AttachStatus::from_c_int(status).is_success())
}

/// FFI: Check if attachment retriable.
#[no_mangle]
pub extern "C" fn rs_attach_status_is_retriable(status: c_int) -> c_int {
    c_int::from(AttachStatus::from_c_int(status).is_retriable())
}

// =============================================================================
// Detach Reason
// =============================================================================

/// Reason for client detachment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetachReason {
    /// Normal disconnect
    #[default]
    Normal = 0,
    /// Client requested
    ClientRequest = 1,
    /// Server requested
    ServerRequest = 2,
    /// Connection lost
    ConnectionLost = 3,
    /// Protocol error
    ProtocolError = 4,
    /// Timeout
    Timeout = 5,
    /// Server shutdown
    ServerShutdown = 6,
    /// Client replaced
    Replaced = 7,
}

impl DetachReason {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::ClientRequest,
            2 => Self::ServerRequest,
            3 => Self::ConnectionLost,
            4 => Self::ProtocolError,
            5 => Self::Timeout,
            6 => Self::ServerShutdown,
            7 => Self::Replaced,
            // 0 and unrecognized values
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this was a clean disconnect.
    #[must_use]
    pub const fn is_clean(self) -> bool {
        matches!(
            self,
            Self::Normal | Self::ClientRequest | Self::ServerRequest | Self::ServerShutdown
        )
    }

    /// Check if this was an error.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(
            self,
            Self::ConnectionLost | Self::ProtocolError | Self::Timeout
        )
    }
}

/// FFI: Check if detach was clean.
#[no_mangle]
pub extern "C" fn rs_detach_reason_is_clean(reason: c_int) -> c_int {
    c_int::from(DetachReason::from_c_int(reason).is_clean())
}

/// FFI: Check if detach was error.
#[no_mangle]
pub extern "C" fn rs_detach_reason_is_error(reason: c_int) -> c_int {
    c_int::from(DetachReason::from_c_int(reason).is_error())
}

// =============================================================================
// Attachment Request
// =============================================================================

/// Request to attach a UI client.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AttachRequest {
    /// Requested width
    pub width: c_int,
    /// Requested height
    pub height: c_int,
    /// Client options
    pub options: UiClientOptions,
    /// Client name hash (for identification)
    pub name_hash: u64,
    /// Protocol version
    pub protocol_version: c_int,
    /// Request ID (for tracking)
    pub request_id: u64,
}

impl AttachRequest {
    /// Create new attach request.
    #[must_use]
    pub const fn new(width: c_int, height: c_int) -> Self {
        Self {
            width,
            height,
            options: UiClientOptions::new(),
            name_hash: 0,
            protocol_version: 1,
            request_id: 0,
        }
    }

    /// Create request with options.
    #[must_use]
    pub const fn with_options(width: c_int, height: c_int, options: UiClientOptions) -> Self {
        Self {
            width,
            height,
            options,
            name_hash: 0,
            protocol_version: 1,
            request_id: 0,
        }
    }

    /// Check if request is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0 && self.protocol_version > 0
    }
}

/// FFI: Create attach request.
#[no_mangle]
pub extern "C" fn rs_attach_request_new(width: c_int, height: c_int) -> AttachRequest {
    AttachRequest::new(width, height)
}

/// FFI: Check if request valid.
///
/// # Safety
/// `req` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_attach_request_is_valid(req: *const AttachRequest) -> c_int {
    if req.is_null() {
        return 0;
    }
    c_int::from((*req).is_valid())
}

// =============================================================================
// Attachment Response
// =============================================================================

/// Response to attach request.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AttachResponse {
    /// Status of attachment
    pub status: c_int,
    /// Assigned client ID
    pub client_id: c_int,
    /// Assigned channel ID
    pub channel_id: u64,
    /// Actual width granted
    pub width: c_int,
    /// Actual height granted
    pub height: c_int,
    /// Server protocol version
    pub protocol_version: c_int,
    /// Request ID (echoed back)
    pub request_id: u64,
}

impl AttachResponse {
    /// Create success response.
    #[must_use]
    pub const fn success(client_id: c_int, channel_id: u64, width: c_int, height: c_int) -> Self {
        Self {
            status: AttachStatus::Ok as c_int,
            client_id,
            channel_id,
            width,
            height,
            protocol_version: 1,
            request_id: 0,
        }
    }

    /// Create failure response.
    #[must_use]
    pub const fn failure(status: AttachStatus) -> Self {
        Self {
            status: status as c_int,
            client_id: 0,
            channel_id: 0,
            width: 0,
            height: 0,
            protocol_version: 1,
            request_id: 0,
        }
    }

    /// Get status.
    #[must_use]
    pub const fn get_status(&self) -> AttachStatus {
        AttachStatus::from_c_int(self.status)
    }

    /// Check if successful.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.get_status().is_success()
    }
}

/// FFI: Create success response.
#[no_mangle]
pub extern "C" fn rs_attach_response_success(
    client_id: c_int,
    channel_id: u64,
    width: c_int,
    height: c_int,
) -> AttachResponse {
    AttachResponse::success(client_id, channel_id, width, height)
}

/// FFI: Create failure response.
#[no_mangle]
pub extern "C" fn rs_attach_response_failure(status: c_int) -> AttachResponse {
    AttachResponse::failure(AttachStatus::from_c_int(status))
}

// =============================================================================
// Client Registry
// =============================================================================

/// Statistics for client registry.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ClientRegistryStats {
    /// Total clients ever attached
    pub total_attached: u64,
    /// Currently attached clients
    pub current_count: c_int,
    /// Maximum concurrent clients
    pub max_concurrent: c_int,
    /// Total disconnections
    pub total_disconnected: u64,
    /// Error disconnections
    pub error_disconnections: u64,
}

impl ClientRegistryStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_attached: 0,
            current_count: 0,
            max_concurrent: 0,
            total_disconnected: 0,
            error_disconnections: 0,
        }
    }

    /// Record client attached.
    pub fn record_attached(&mut self) {
        self.total_attached += 1;
        self.current_count += 1;
        if self.current_count > self.max_concurrent {
            self.max_concurrent = self.current_count;
        }
    }

    /// Record client detached.
    pub fn record_detached(&mut self, reason: DetachReason) {
        if self.current_count > 0 {
            self.current_count -= 1;
        }
        self.total_disconnected += 1;
        if reason.is_error() {
            self.error_disconnections += 1;
        }
    }
}

/// FFI: Create registry stats.
#[no_mangle]
pub extern "C" fn rs_client_registry_stats_new() -> ClientRegistryStats {
    ClientRegistryStats::new()
}

/// FFI: Record attached.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_client_registry_record_attached(stats: *mut ClientRegistryStats) {
    if !stats.is_null() {
        (*stats).record_attached();
    }
}

/// FFI: Record detached.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_client_registry_record_detached(
    stats: *mut ClientRegistryStats,
    reason: c_int,
) {
    if !stats.is_null() {
        (*stats).record_detached(DetachReason::from_c_int(reason));
    }
}

// =============================================================================
// Resize Request
// =============================================================================

/// Request to resize UI.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ResizeRequest {
    /// Client ID
    pub client_id: c_int,
    /// New width
    pub width: c_int,
    /// New height
    pub height: c_int,
    /// Request ID
    pub request_id: u64,
}

impl ResizeRequest {
    /// Create new resize request.
    #[must_use]
    pub const fn new(client_id: c_int, width: c_int, height: c_int) -> Self {
        Self {
            client_id,
            width,
            height,
            request_id: 0,
        }
    }

    /// Check if valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Check if dimensions changed from given values.
    #[must_use]
    pub const fn dimensions_changed(&self, current_width: c_int, current_height: c_int) -> bool {
        self.width != current_width || self.height != current_height
    }
}

/// FFI: Create resize request.
#[no_mangle]
pub extern "C" fn rs_resize_request_new(
    client_id: c_int,
    width: c_int,
    height: c_int,
) -> ResizeRequest {
    ResizeRequest::new(client_id, width, height)
}

/// FFI: Check if resize valid.
///
/// # Safety
/// `req` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_resize_request_is_valid(req: *const ResizeRequest) -> c_int {
    if req.is_null() {
        return 0;
    }
    c_int::from((*req).is_valid())
}

// =============================================================================
// Connection Info
// =============================================================================

/// Information about a connection.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ConnectionInfo {
    /// Channel ID
    pub channel_id: u64,
    /// Connection type (0=stdio, 1=socket, 2=pipe)
    pub conn_type: c_int,
    /// Is local connection
    pub is_local: bool,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_recv: u64,
    /// Messages sent
    pub msgs_sent: u64,
    /// Messages received
    pub msgs_recv: u64,
    /// Connection start time (unix timestamp)
    pub start_time: i64,
}

impl ConnectionInfo {
    /// Create new connection info.
    #[must_use]
    pub const fn new(channel_id: u64, conn_type: c_int) -> Self {
        Self {
            channel_id,
            conn_type,
            is_local: true,
            bytes_sent: 0,
            bytes_recv: 0,
            msgs_sent: 0,
            msgs_recv: 0,
            start_time: 0,
        }
    }

    /// Record sent data.
    pub fn record_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
        self.msgs_sent += 1;
    }

    /// Record received data.
    pub fn record_recv(&mut self, bytes: u64) {
        self.bytes_recv += bytes;
        self.msgs_recv += 1;
    }
}

/// FFI: Create connection info.
#[no_mangle]
pub extern "C" fn rs_connection_info_new(channel_id: u64, conn_type: c_int) -> ConnectionInfo {
    ConnectionInfo::new(channel_id, conn_type)
}

/// FFI: Record sent data.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_connection_record_sent(info: *mut ConnectionInfo, bytes: u64) {
    if !info.is_null() {
        (*info).record_sent(bytes);
    }
}

/// FFI: Record received data.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_connection_record_recv(info: *mut ConnectionInfo, bytes: u64) {
    if !info.is_null() {
        (*info).record_recv(bytes);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_status() {
        assert!(AttachStatus::Ok.is_success());
        assert!(!AttachStatus::ConnectionFailed.is_success());
        assert!(AttachStatus::ConnectionFailed.is_retriable());
        assert!(AttachStatus::ResourceLimit.is_retriable());
        assert!(!AttachStatus::PermissionDenied.is_retriable());
    }

    #[test]
    fn test_detach_reason() {
        assert!(DetachReason::Normal.is_clean());
        assert!(DetachReason::ClientRequest.is_clean());
        assert!(DetachReason::ServerShutdown.is_clean());

        assert!(DetachReason::ConnectionLost.is_error());
        assert!(DetachReason::ProtocolError.is_error());
        assert!(DetachReason::Timeout.is_error());
    }

    #[test]
    fn test_attach_request() {
        let req = AttachRequest::new(80, 24);
        assert!(req.is_valid());
        assert_eq!(req.width, 80);
        assert_eq!(req.height, 24);

        let invalid = AttachRequest::new(0, 24);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_attach_response() {
        let success = AttachResponse::success(1, 100, 80, 24);
        assert!(success.is_success());
        assert_eq!(success.client_id, 1);

        let failure = AttachResponse::failure(AttachStatus::ConnectionFailed);
        assert!(!failure.is_success());
    }

    #[test]
    fn test_client_registry_stats() {
        let mut stats = ClientRegistryStats::new();
        assert_eq!(stats.current_count, 0);

        stats.record_attached();
        assert_eq!(stats.total_attached, 1);
        assert_eq!(stats.current_count, 1);
        assert_eq!(stats.max_concurrent, 1);

        stats.record_attached();
        assert_eq!(stats.current_count, 2);
        assert_eq!(stats.max_concurrent, 2);

        stats.record_detached(DetachReason::Normal);
        assert_eq!(stats.current_count, 1);
        assert_eq!(stats.total_disconnected, 1);
        assert_eq!(stats.error_disconnections, 0);

        stats.record_detached(DetachReason::ConnectionLost);
        assert_eq!(stats.error_disconnections, 1);
    }

    #[test]
    fn test_resize_request() {
        let req = ResizeRequest::new(1, 120, 40);
        assert!(req.is_valid());
        assert!(req.dimensions_changed(80, 24));
        assert!(!req.dimensions_changed(120, 40));
    }

    #[test]
    fn test_connection_info() {
        let mut info = ConnectionInfo::new(100, 1);
        assert_eq!(info.channel_id, 100);
        assert_eq!(info.bytes_sent, 0);

        info.record_sent(1024);
        assert_eq!(info.bytes_sent, 1024);
        assert_eq!(info.msgs_sent, 1);

        info.record_recv(512);
        assert_eq!(info.bytes_recv, 512);
        assert_eq!(info.msgs_recv, 1);
    }
}
