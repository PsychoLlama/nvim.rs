//! Async clipboard operations
//!
//! This module provides types for asynchronous clipboard operations,
//! including request/response handling and operation state.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]

use std::ffi::c_int;

use crate::selection::SelectionType;
use crate::ClipboardRegister;

// =============================================================================
// Operation Type
// =============================================================================

/// Type of clipboard operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardOperation {
    /// Get/read from clipboard
    #[default]
    Get = 0,
    /// Set/write to clipboard
    Set = 1,
    /// Clear clipboard
    Clear = 2,
}

impl ClipboardOperation {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Get),
            1 => Some(Self::Set),
            2 => Some(Self::Clear),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is a read operation
    pub const fn is_read(self) -> bool {
        matches!(self, Self::Get)
    }

    /// Check if this is a write operation
    pub const fn is_write(self) -> bool {
        matches!(self, Self::Set | Self::Clear)
    }
}

// =============================================================================
// Request State
// =============================================================================

/// State of async clipboard request
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RequestState {
    /// Request pending
    #[default]
    Pending = 0,
    /// Request in progress
    InProgress = 1,
    /// Request completed successfully
    Completed = 2,
    /// Request failed
    Failed = 3,
    /// Request cancelled
    Cancelled = 4,
    /// Request timed out
    TimedOut = 5,
}

impl RequestState {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::InProgress),
            2 => Some(Self::Completed),
            3 => Some(Self::Failed),
            4 => Some(Self::Cancelled),
            5 => Some(Self::TimedOut),
            _ => None,
        }
    }

    /// Check if request is done
    pub const fn is_done(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }

    /// Check if request succeeded
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Check if request failed
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled | Self::TimedOut)
    }
}

// =============================================================================
// Clipboard Request
// =============================================================================

/// Clipboard request
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClipboardRequest {
    /// Request ID
    pub id: u64,
    /// Operation type
    pub operation: ClipboardOperation,
    /// Target register
    pub register: ClipboardRegister,
    /// Selection type
    pub selection: SelectionType,
    /// Request state
    pub state: RequestState,
    /// Error code (0 = none)
    pub error_code: c_int,
}

impl Default for ClipboardRequest {
    fn default() -> Self {
        Self {
            id: 0,
            operation: ClipboardOperation::Get,
            register: ClipboardRegister::None,
            selection: SelectionType::Clipboard,
            state: RequestState::Pending,
            error_code: 0,
        }
    }
}

impl ClipboardRequest {
    /// Create a new get request
    pub const fn get(id: u64, register: ClipboardRegister) -> Self {
        Self {
            id,
            operation: ClipboardOperation::Get,
            register,
            selection: SelectionType::Clipboard,
            state: RequestState::Pending,
            error_code: 0,
        }
    }

    /// Create a new set request
    pub const fn set(id: u64, register: ClipboardRegister) -> Self {
        Self {
            id,
            operation: ClipboardOperation::Set,
            register,
            selection: SelectionType::Clipboard,
            state: RequestState::Pending,
            error_code: 0,
        }
    }

    /// Check if request is done
    pub const fn is_done(&self) -> bool {
        self.state.is_done()
    }

    /// Check if request succeeded
    pub const fn is_success(&self) -> bool {
        self.state.is_success()
    }

    /// Mark as in progress
    pub fn mark_in_progress(&mut self) {
        self.state = RequestState::InProgress;
    }

    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.state = RequestState::Completed;
        self.error_code = 0;
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error_code: c_int) {
        self.state = RequestState::Failed;
        self.error_code = error_code;
    }

    /// Mark as cancelled
    pub fn mark_cancelled(&mut self) {
        self.state = RequestState::Cancelled;
    }

    /// Mark as timed out
    pub fn mark_timed_out(&mut self) {
        self.state = RequestState::TimedOut;
    }
}

// =============================================================================
// Clipboard Result
// =============================================================================

/// Result of clipboard operation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClipboardResult {
    /// Whether operation succeeded
    pub success: bool,
    /// Error code (0 = none)
    pub error_code: c_int,
    /// Number of lines (for get operations)
    pub line_count: usize,
    /// Motion type (for get operations)
    pub motion_type: c_int,
}

impl Default for ClipboardResult {
    fn default() -> Self {
        Self {
            success: false,
            error_code: 0,
            line_count: 0,
            motion_type: 0,
        }
    }
}

impl ClipboardResult {
    /// Create success result
    pub const fn success() -> Self {
        Self {
            success: true,
            error_code: 0,
            line_count: 0,
            motion_type: 0,
        }
    }

    /// Create success result with data info
    pub const fn success_with_data(line_count: usize, motion_type: c_int) -> Self {
        Self {
            success: true,
            error_code: 0,
            line_count,
            motion_type,
        }
    }

    /// Create failure result
    pub const fn failure(error_code: c_int) -> Self {
        Self {
            success: false,
            error_code,
            line_count: 0,
            motion_type: 0,
        }
    }
}

// =============================================================================
// Request Queue Entry
// =============================================================================

/// Entry in request queue
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QueueEntry {
    /// The request
    pub request: ClipboardRequest,
    /// Timestamp when request was created
    pub timestamp: u64,
    /// Timeout in milliseconds
    pub timeout_ms: u32,
}

impl QueueEntry {
    /// Create new queue entry
    pub const fn new(request: ClipboardRequest, timeout_ms: u32) -> Self {
        Self {
            request,
            timestamp: 0,
            timeout_ms,
        }
    }

    /// Check if entry has timed out
    pub const fn is_timed_out(&self, current_time: u64) -> bool {
        if self.timeout_ms == 0 {
            return false; // No timeout
        }
        let elapsed = current_time.saturating_sub(self.timestamp);
        elapsed >= self.timeout_ms as u64
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if operation is valid
#[no_mangle]
pub extern "C" fn rs_clipboard_op_valid(op: c_int) -> c_int {
    c_int::from(ClipboardOperation::from_raw(op).is_some())
}

/// FFI export: Check if operation is read
#[no_mangle]
pub extern "C" fn rs_clipboard_op_is_read(op: c_int) -> c_int {
    ClipboardOperation::from_raw(op).map_or(0, |o| c_int::from(o.is_read()))
}

/// FFI export: Check if operation is write
#[no_mangle]
pub extern "C" fn rs_clipboard_op_is_write(op: c_int) -> c_int {
    ClipboardOperation::from_raw(op).map_or(0, |o| c_int::from(o.is_write()))
}

/// FFI export: Check if request state is done
#[no_mangle]
pub extern "C" fn rs_clipboard_state_is_done(state: c_int) -> c_int {
    RequestState::from_raw(state).map_or(0, |s| c_int::from(s.is_done()))
}

/// FFI export: Check if request state is success
#[no_mangle]
pub extern "C" fn rs_clipboard_state_is_success(state: c_int) -> c_int {
    RequestState::from_raw(state).map_or(0, |s| c_int::from(s.is_success()))
}

/// FFI export: Create get request
#[no_mangle]
pub extern "C" fn rs_clipboard_request_get(
    id: u64,
    register: ClipboardRegister,
) -> ClipboardRequest {
    ClipboardRequest::get(id, register)
}

/// FFI export: Create set request
#[no_mangle]
pub extern "C" fn rs_clipboard_request_set(
    id: u64,
    register: ClipboardRegister,
) -> ClipboardRequest {
    ClipboardRequest::set(id, register)
}

/// FFI export: Check if request is done
#[no_mangle]
pub extern "C" fn rs_clipboard_request_is_done(req: *const ClipboardRequest) -> c_int {
    if req.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*req).is_done() })
}

/// FFI export: Create success result
#[no_mangle]
pub extern "C" fn rs_clipboard_result_success() -> ClipboardResult {
    ClipboardResult::success()
}

/// FFI export: Create failure result
#[no_mangle]
pub extern "C" fn rs_clipboard_result_failure(error_code: c_int) -> ClipboardResult {
    ClipboardResult::failure(error_code)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
mod tests {
    use super::*;

    #[test]
    fn test_operation() {
        assert_eq!(
            ClipboardOperation::from_raw(0),
            Some(ClipboardOperation::Get)
        );
        assert_eq!(
            ClipboardOperation::from_raw(1),
            Some(ClipboardOperation::Set)
        );
        assert_eq!(ClipboardOperation::from_raw(100), None);

        assert!(ClipboardOperation::Get.is_read());
        assert!(!ClipboardOperation::Get.is_write());
        assert!(ClipboardOperation::Set.is_write());
    }

    #[test]
    fn test_request_state() {
        assert_eq!(RequestState::from_raw(0), Some(RequestState::Pending));
        assert_eq!(RequestState::from_raw(2), Some(RequestState::Completed));
        assert_eq!(RequestState::from_raw(100), None);

        assert!(!RequestState::Pending.is_done());
        assert!(RequestState::Completed.is_done());
        assert!(RequestState::Completed.is_success());
        assert!(RequestState::Failed.is_failure());
    }

    #[test]
    fn test_clipboard_request() {
        let req = ClipboardRequest::get(1, ClipboardRegister::Star);
        assert_eq!(req.operation, ClipboardOperation::Get);
        assert_eq!(req.register, ClipboardRegister::Star);
        assert!(!req.is_done());

        let mut req = ClipboardRequest::set(2, ClipboardRegister::Plus);
        assert_eq!(req.operation, ClipboardOperation::Set);

        req.mark_in_progress();
        assert_eq!(req.state, RequestState::InProgress);

        req.mark_completed();
        assert!(req.is_done());
        assert!(req.is_success());
    }

    #[test]
    fn test_request_failure() {
        let mut req = ClipboardRequest::get(1, ClipboardRegister::Star);
        req.mark_failed(42);
        assert!(req.is_done());
        assert!(!req.is_success());
        assert_eq!(req.error_code, 42);
    }

    #[test]
    fn test_clipboard_result() {
        let success = ClipboardResult::success();
        assert!(success.success);
        assert_eq!(success.error_code, 0);

        let failure = ClipboardResult::failure(1);
        assert!(!failure.success);
        assert_eq!(failure.error_code, 1);

        let with_data = ClipboardResult::success_with_data(5, 1);
        assert!(with_data.success);
        assert_eq!(with_data.line_count, 5);
    }

    #[test]
    fn test_queue_entry() {
        let req = ClipboardRequest::get(1, ClipboardRegister::Star);
        let entry = QueueEntry::new(req, 1000);
        assert_eq!(entry.timeout_ms, 1000);

        // No timeout if timeout_ms is 0
        let entry_no_timeout = QueueEntry::new(req, 0);
        assert!(!entry_no_timeout.is_timed_out(10000));
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_clipboard_op_valid(0), 1);
        assert_eq!(rs_clipboard_op_valid(100), 0);

        assert_eq!(rs_clipboard_op_is_read(0), 1);
        assert_eq!(rs_clipboard_op_is_write(1), 1);

        assert_eq!(rs_clipboard_state_is_done(2), 1);
        assert_eq!(rs_clipboard_state_is_success(2), 1);

        let success = rs_clipboard_result_success();
        assert!(success.success);

        let failure = rs_clipboard_result_failure(5);
        assert_eq!(failure.error_code, 5);
    }
}
