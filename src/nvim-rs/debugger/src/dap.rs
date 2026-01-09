//! Debug Adapter Protocol (DAP) support
//!
//! This module provides types and utilities for DAP communication,
//! supporting the standard debug adapter protocol.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]

use std::ffi::c_int;

// =============================================================================
// DAP Message Types
// =============================================================================

/// DAP message type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DapMessageType {
    /// Request from client to adapter
    Request = 0,
    /// Response from adapter to client
    Response = 1,
    /// Event from adapter to client
    Event = 2,
}

impl DapMessageType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Request),
            1 => Some(Self::Response),
            2 => Some(Self::Event),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Get type string
    pub const fn type_string(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Response => "response",
            Self::Event => "event",
        }
    }
}

impl Default for DapMessageType {
    fn default() -> Self {
        Self::Request
    }
}

// =============================================================================
// DAP Request Types
// =============================================================================

/// DAP request command types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DapRequestType {
    /// Initialize the debug session
    Initialize = 0,
    /// Configuration done
    ConfigurationDone = 1,
    /// Launch debuggee
    Launch = 2,
    /// Attach to debuggee
    Attach = 3,
    /// Disconnect from debuggee
    Disconnect = 4,
    /// Terminate debuggee
    Terminate = 5,
    /// Restart debuggee
    Restart = 6,
    /// Set breakpoints
    SetBreakpoints = 7,
    /// Set function breakpoints
    SetFunctionBreakpoints = 8,
    /// Set exception breakpoints
    SetExceptionBreakpoints = 9,
    /// Continue execution
    Continue = 10,
    /// Step next
    Next = 11,
    /// Step in
    StepIn = 12,
    /// Step out
    StepOut = 13,
    /// Step back
    StepBack = 14,
    /// Pause execution
    Pause = 15,
    /// Get stack trace
    StackTrace = 16,
    /// Get scopes
    Scopes = 17,
    /// Get variables
    Variables = 18,
    /// Get source
    Source = 19,
    /// Evaluate expression
    Evaluate = 20,
    /// Get threads
    Threads = 21,
    /// Unknown request
    Unknown = 100,
}

impl DapRequestType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Initialize,
            1 => Self::ConfigurationDone,
            2 => Self::Launch,
            3 => Self::Attach,
            4 => Self::Disconnect,
            5 => Self::Terminate,
            6 => Self::Restart,
            7 => Self::SetBreakpoints,
            8 => Self::SetFunctionBreakpoints,
            9 => Self::SetExceptionBreakpoints,
            10 => Self::Continue,
            11 => Self::Next,
            12 => Self::StepIn,
            13 => Self::StepOut,
            14 => Self::StepBack,
            15 => Self::Pause,
            16 => Self::StackTrace,
            17 => Self::Scopes,
            18 => Self::Variables,
            19 => Self::Source,
            20 => Self::Evaluate,
            21 => Self::Threads,
            _ => Self::Unknown,
        }
    }

    /// Get command string
    pub const fn command_string(self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::ConfigurationDone => "configurationDone",
            Self::Launch => "launch",
            Self::Attach => "attach",
            Self::Disconnect => "disconnect",
            Self::Terminate => "terminate",
            Self::Restart => "restart",
            Self::SetBreakpoints => "setBreakpoints",
            Self::SetFunctionBreakpoints => "setFunctionBreakpoints",
            Self::SetExceptionBreakpoints => "setExceptionBreakpoints",
            Self::Continue => "continue",
            Self::Next => "next",
            Self::StepIn => "stepIn",
            Self::StepOut => "stepOut",
            Self::StepBack => "stepBack",
            Self::Pause => "pause",
            Self::StackTrace => "stackTrace",
            Self::Scopes => "scopes",
            Self::Variables => "variables",
            Self::Source => "source",
            Self::Evaluate => "evaluate",
            Self::Threads => "threads",
            Self::Unknown => "unknown",
        }
    }

    /// Check if this is a control request
    pub const fn is_control(self) -> bool {
        matches!(
            self,
            Self::Continue
                | Self::Next
                | Self::StepIn
                | Self::StepOut
                | Self::StepBack
                | Self::Pause
        )
    }

    /// Check if this is a session request
    pub const fn is_session(self) -> bool {
        matches!(
            self,
            Self::Initialize
                | Self::ConfigurationDone
                | Self::Launch
                | Self::Attach
                | Self::Disconnect
                | Self::Terminate
        )
    }
}

impl Default for DapRequestType {
    fn default() -> Self {
        Self::Unknown
    }
}

// =============================================================================
// DAP Event Types
// =============================================================================

/// DAP event types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DapEventType {
    /// Debugger initialized
    Initialized = 0,
    /// Program stopped
    Stopped = 1,
    /// Program continued
    Continued = 2,
    /// Program exited
    Exited = 3,
    /// Program terminated
    Terminated = 4,
    /// Thread created
    Thread = 5,
    /// Output produced
    Output = 6,
    /// Breakpoint changed
    Breakpoint = 7,
    /// Module loaded
    Module = 8,
    /// Loaded source
    LoadedSource = 9,
    /// Process info
    Process = 10,
    /// Capabilities changed
    Capabilities = 11,
    /// Progress started
    ProgressStart = 12,
    /// Progress updated
    ProgressUpdate = 13,
    /// Progress ended
    ProgressEnd = 14,
    /// Invalid
    Invalidated = 15,
    /// Memory changed
    Memory = 16,
}

impl DapEventType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Initialized),
            1 => Some(Self::Stopped),
            2 => Some(Self::Continued),
            3 => Some(Self::Exited),
            4 => Some(Self::Terminated),
            5 => Some(Self::Thread),
            6 => Some(Self::Output),
            7 => Some(Self::Breakpoint),
            8 => Some(Self::Module),
            9 => Some(Self::LoadedSource),
            10 => Some(Self::Process),
            11 => Some(Self::Capabilities),
            12 => Some(Self::ProgressStart),
            13 => Some(Self::ProgressUpdate),
            14 => Some(Self::ProgressEnd),
            15 => Some(Self::Invalidated),
            16 => Some(Self::Memory),
            _ => None,
        }
    }

    /// Get event string
    pub const fn event_string(self) -> &'static str {
        match self {
            Self::Initialized => "initialized",
            Self::Stopped => "stopped",
            Self::Continued => "continued",
            Self::Exited => "exited",
            Self::Terminated => "terminated",
            Self::Thread => "thread",
            Self::Output => "output",
            Self::Breakpoint => "breakpoint",
            Self::Module => "module",
            Self::LoadedSource => "loadedSource",
            Self::Process => "process",
            Self::Capabilities => "capabilities",
            Self::ProgressStart => "progressStart",
            Self::ProgressUpdate => "progressUpdate",
            Self::ProgressEnd => "progressEnd",
            Self::Invalidated => "invalidated",
            Self::Memory => "memory",
        }
    }
}

// =============================================================================
// DAP Message
// =============================================================================

/// DAP message structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DapMessage {
    /// Sequence number
    pub seq: c_int,
    /// Message type
    pub mtype: DapMessageType,
    /// Request/response ID (for Response type)
    pub request_seq: c_int,
    /// Success flag (for Response type)
    pub success: bool,
}

impl Default for DapMessage {
    fn default() -> Self {
        Self {
            seq: 0,
            mtype: DapMessageType::Request,
            request_seq: 0,
            success: false,
        }
    }
}

impl DapMessage {
    /// Create a request message
    pub const fn request(seq: c_int) -> Self {
        Self {
            seq,
            mtype: DapMessageType::Request,
            request_seq: 0,
            success: false,
        }
    }

    /// Create a success response
    pub const fn response_ok(seq: c_int, request_seq: c_int) -> Self {
        Self {
            seq,
            mtype: DapMessageType::Response,
            request_seq,
            success: true,
        }
    }

    /// Create an error response
    pub const fn response_err(seq: c_int, request_seq: c_int) -> Self {
        Self {
            seq,
            mtype: DapMessageType::Response,
            request_seq,
            success: false,
        }
    }

    /// Create an event message
    pub const fn event(seq: c_int) -> Self {
        Self {
            seq,
            mtype: DapMessageType::Event,
            request_seq: 0,
            success: false,
        }
    }

    /// Check if this is a request
    pub const fn is_request(&self) -> bool {
        matches!(self.mtype, DapMessageType::Request)
    }

    /// Check if this is a response
    pub const fn is_response(&self) -> bool {
        matches!(self.mtype, DapMessageType::Response)
    }

    /// Check if this is an event
    pub const fn is_event(&self) -> bool {
        matches!(self.mtype, DapMessageType::Event)
    }
}

// =============================================================================
// DAP State
// =============================================================================

/// DAP session state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DapState {
    /// Not connected
    #[default]
    Disconnected = 0,
    /// Connecting
    Connecting = 1,
    /// Initializing
    Initializing = 2,
    /// Ready (configured)
    Ready = 3,
    /// Running (debugging)
    Running = 4,
    /// Stopped (at breakpoint/step)
    Stopped = 5,
    /// Terminated
    Terminated = 6,
}

impl DapState {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Disconnected),
            1 => Some(Self::Connecting),
            2 => Some(Self::Initializing),
            3 => Some(Self::Ready),
            4 => Some(Self::Running),
            5 => Some(Self::Stopped),
            6 => Some(Self::Terminated),
            _ => None,
        }
    }

    /// Check if connected
    pub const fn is_connected(self) -> bool {
        !matches!(self, Self::Disconnected)
    }

    /// Check if active (running or stopped)
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Stopped)
    }

    /// Check if can send control commands
    pub const fn can_control(self) -> bool {
        matches!(self, Self::Running | Self::Stopped)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if message type is valid
#[no_mangle]
pub extern "C" fn rs_dap_message_type_valid(mtype: c_int) -> c_int {
    c_int::from(DapMessageType::from_raw(mtype).is_some())
}

/// FFI export: Check if request type is control
#[no_mangle]
pub extern "C" fn rs_dap_request_is_control(rtype: c_int) -> c_int {
    c_int::from(DapRequestType::from_raw(rtype).is_control())
}

/// FFI export: Check if request type is session
#[no_mangle]
pub extern "C" fn rs_dap_request_is_session(rtype: c_int) -> c_int {
    c_int::from(DapRequestType::from_raw(rtype).is_session())
}

/// FFI export: Create request message
#[no_mangle]
pub extern "C" fn rs_dap_message_request(seq: c_int) -> DapMessage {
    DapMessage::request(seq)
}

/// FFI export: Create success response
#[no_mangle]
pub extern "C" fn rs_dap_message_response_ok(seq: c_int, request_seq: c_int) -> DapMessage {
    DapMessage::response_ok(seq, request_seq)
}

/// FFI export: Create event message
#[no_mangle]
pub extern "C" fn rs_dap_message_event(seq: c_int) -> DapMessage {
    DapMessage::event(seq)
}

/// FFI export: Check if state is connected
#[no_mangle]
pub extern "C" fn rs_dap_state_is_connected(state: c_int) -> c_int {
    DapState::from_raw(state).map_or(0, |s| c_int::from(s.is_connected()))
}

/// FFI export: Check if state can control
#[no_mangle]
pub extern "C" fn rs_dap_state_can_control(state: c_int) -> c_int {
    DapState::from_raw(state).map_or(0, |s| c_int::from(s.can_control()))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dap_message_type() {
        assert_eq!(DapMessageType::from_raw(0), Some(DapMessageType::Request));
        assert_eq!(DapMessageType::from_raw(100), None);

        assert_eq!(DapMessageType::Request.type_string(), "request");
    }

    #[test]
    fn test_dap_request_type() {
        assert!(DapRequestType::Continue.is_control());
        assert!(DapRequestType::StepIn.is_control());
        assert!(!DapRequestType::SetBreakpoints.is_control());

        assert!(DapRequestType::Initialize.is_session());
        assert!(DapRequestType::Launch.is_session());
        assert!(!DapRequestType::Continue.is_session());
    }

    #[test]
    fn test_dap_message() {
        let req = DapMessage::request(1);
        assert!(req.is_request());
        assert!(!req.is_response());

        let resp = DapMessage::response_ok(2, 1);
        assert!(resp.is_response());
        assert!(resp.success);

        let event = DapMessage::event(3);
        assert!(event.is_event());
    }

    #[test]
    fn test_dap_state() {
        assert!(!DapState::Disconnected.is_connected());
        assert!(DapState::Running.is_connected());

        assert!(DapState::Running.is_active());
        assert!(DapState::Stopped.is_active());
        assert!(!DapState::Ready.is_active());

        assert!(DapState::Running.can_control());
        assert!(!DapState::Initializing.can_control());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_dap_message_type_valid(0), 1);
        assert_eq!(rs_dap_message_type_valid(100), 0);

        assert_eq!(rs_dap_request_is_control(10), 1); // Continue
        assert_eq!(rs_dap_request_is_control(7), 0); // SetBreakpoints

        assert_eq!(rs_dap_state_is_connected(4), 1); // Running
        assert_eq!(rs_dap_state_is_connected(0), 0); // Disconnected
    }
}
