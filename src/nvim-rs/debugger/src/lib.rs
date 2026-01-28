//! Debugger infrastructure
//!
//! This crate provides Rust implementations for debugging support,
//! including DAP (Debug Adapter Protocol), breakpoints, and stepping.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::use_self)]

pub mod breakpoint;
pub mod dap;
pub mod state;
pub mod stepping;

use std::ffi::c_int;

// Re-export key types
pub use breakpoint::{Breakpoint, BreakpointFlags, BreakpointType};
pub use dap::{DapMessage, DapMessageType, DapState};
pub use state::{DebugStatus, DebuggerState};
pub use stepping::{StepMode, StepResult};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of breakpoints
pub const MAX_BREAKPOINTS: usize = 1000;

/// Maximum stack depth for debugging
pub const MAX_STACK_DEPTH: c_int = 256;

// =============================================================================
// Debug Level
// =============================================================================

/// Debug output level
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugLevel {
    /// No debug output
    #[default]
    None = 0,
    /// Errors only
    Error = 1,
    /// Errors and warnings
    Warning = 2,
    /// Errors, warnings, and info
    Info = 3,
    /// All messages including debug
    Debug = 4,
    /// Trace level (very verbose)
    Trace = 5,
}

impl DebugLevel {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Error),
            2 => Some(Self::Warning),
            3 => Some(Self::Info),
            4 => Some(Self::Debug),
            5 => Some(Self::Trace),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if level should show message
    pub const fn should_log(self, msg_level: DebugLevel) -> bool {
        (msg_level as c_int) <= (self as c_int)
    }
}

// =============================================================================
// Debug Event Type
// =============================================================================

/// Types of debugger events
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugEventType {
    /// Breakpoint hit
    BreakpointHit = 0,
    /// Step completed
    StepComplete = 1,
    /// Exception occurred
    Exception = 2,
    /// Program started
    Started = 3,
    /// Program stopped
    Stopped = 4,
    /// Program exited
    Exited = 5,
    /// Thread created
    ThreadCreated = 6,
    /// Thread exited
    ThreadExited = 7,
    /// Output produced
    Output = 8,
    /// Module loaded
    ModuleLoaded = 9,
}

impl DebugEventType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::BreakpointHit),
            1 => Some(Self::StepComplete),
            2 => Some(Self::Exception),
            3 => Some(Self::Started),
            4 => Some(Self::Stopped),
            5 => Some(Self::Exited),
            6 => Some(Self::ThreadCreated),
            7 => Some(Self::ThreadExited),
            8 => Some(Self::Output),
            9 => Some(Self::ModuleLoaded),
            _ => None,
        }
    }

    /// Check if this is a stopping event
    pub const fn is_stopping(self) -> bool {
        matches!(
            self,
            Self::BreakpointHit | Self::StepComplete | Self::Exception | Self::Stopped
        )
    }

    /// Check if this terminates debugging
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Exited)
    }
}

// =============================================================================
// Debug Capability Flags
// =============================================================================

/// Debugger capability flags
pub const CAP_SUPPORTS_BREAKPOINTS: u32 = 0x0001;
pub const CAP_SUPPORTS_CONDITIONAL: u32 = 0x0002;
pub const CAP_SUPPORTS_HITCOUNT: u32 = 0x0004;
pub const CAP_SUPPORTS_LOGPOINTS: u32 = 0x0008;
pub const CAP_SUPPORTS_STEP_IN: u32 = 0x0010;
pub const CAP_SUPPORTS_STEP_OUT: u32 = 0x0020;
pub const CAP_SUPPORTS_STEP_BACK: u32 = 0x0040;
pub const CAP_SUPPORTS_GOTO: u32 = 0x0080;
pub const CAP_SUPPORTS_EVALUATE: u32 = 0x0100;
pub const CAP_SUPPORTS_MODULES: u32 = 0x0200;
pub const CAP_SUPPORTS_EXCEPTION: u32 = 0x0400;
pub const CAP_SUPPORTS_TERMINATE: u32 = 0x0800;

/// Debugger capabilities wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DebugCapabilities {
    flags: u32,
}

impl DebugCapabilities {
    /// Create with no capabilities
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw flags
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw flags
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if breakpoints are supported
    pub const fn supports_breakpoints(self) -> bool {
        (self.flags & CAP_SUPPORTS_BREAKPOINTS) != 0
    }

    /// Check if conditional breakpoints are supported
    pub const fn supports_conditional(self) -> bool {
        (self.flags & CAP_SUPPORTS_CONDITIONAL) != 0
    }

    /// Check if hit count is supported
    pub const fn supports_hitcount(self) -> bool {
        (self.flags & CAP_SUPPORTS_HITCOUNT) != 0
    }

    /// Check if step in is supported
    pub const fn supports_step_in(self) -> bool {
        (self.flags & CAP_SUPPORTS_STEP_IN) != 0
    }

    /// Check if step out is supported
    pub const fn supports_step_out(self) -> bool {
        (self.flags & CAP_SUPPORTS_STEP_OUT) != 0
    }

    /// Check if step back is supported
    pub const fn supports_step_back(self) -> bool {
        (self.flags & CAP_SUPPORTS_STEP_BACK) != 0
    }

    /// Check if evaluate is supported
    pub const fn supports_evaluate(self) -> bool {
        (self.flags & CAP_SUPPORTS_EVALUATE) != 0
    }

    /// Check if terminate is supported
    pub const fn supports_terminate(self) -> bool {
        (self.flags & CAP_SUPPORTS_TERMINATE) != 0
    }
}

// =============================================================================
// Debug Command Types (for Vim debug mode)
// =============================================================================

/// Debug command types for Vim script debugger
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugCommand {
    /// No command / unknown
    #[default]
    None = 0,
    /// Continue execution
    Continue = 1,
    /// Step to next statement (over functions)
    Next = 2,
    /// Step into functions
    Step = 3,
    /// Finish current function
    Finish = 4,
    /// Quit debugging
    Quit = 5,
    /// Interrupt execution
    Interrupt = 6,
    /// Show backtrace
    Backtrace = 7,
    /// Frame selection
    Frame = 8,
    /// Go up one frame
    Up = 9,
    /// Go down one frame
    Down = 10,
}

impl DebugCommand {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Continue),
            2 => Some(Self::Next),
            3 => Some(Self::Step),
            4 => Some(Self::Finish),
            5 => Some(Self::Quit),
            6 => Some(Self::Interrupt),
            7 => Some(Self::Backtrace),
            8 => Some(Self::Frame),
            9 => Some(Self::Up),
            10 => Some(Self::Down),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this command exits debug mode
    pub const fn exits_debug_mode(self) -> bool {
        matches!(
            self,
            Self::Continue | Self::Next | Self::Step | Self::Finish | Self::Quit | Self::Interrupt
        )
    }

    /// Check if this command continues in debug loop
    pub const fn continues_debug_loop(self) -> bool {
        matches!(self, Self::Backtrace | Self::Frame | Self::Up | Self::Down)
    }

    /// Get the tail suffix for command matching
    pub const fn tail_suffix(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Continue => "ont",
            Self::Next => "ext",
            Self::Step => "tep",
            Self::Finish => "inish",
            Self::Quit => "uit",
            Self::Interrupt => "nterrupt",
            Self::Backtrace => "acktrace",
            Self::Frame => "rame",
            Self::Up => "p",
            Self::Down => "own",
        }
    }
}

/// Vim script breakpoint types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimBreakpointType {
    /// Function breakpoint
    #[default]
    Func = 1,
    /// File breakpoint
    File = 2,
    /// Expression watchpoint
    Expr = 3,
}

impl VimBreakpointType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            1 => Some(Self::Func),
            2 => Some(Self::File),
            3 => Some(Self::Expr),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Get type string for parsing
    pub const fn type_string(self) -> &'static str {
        match self {
            Self::Func => "func",
            Self::File => "file",
            Self::Expr => "expr",
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if debug level is valid
#[no_mangle]
pub extern "C" fn rs_debug_level_valid(level: c_int) -> c_int {
    c_int::from(DebugLevel::from_raw(level).is_some())
}

/// FFI export: Check if level should log
#[no_mangle]
pub extern "C" fn rs_debug_level_should_log(current: c_int, msg_level: c_int) -> c_int {
    let current = DebugLevel::from_raw(current).unwrap_or(DebugLevel::None);
    let msg = DebugLevel::from_raw(msg_level).unwrap_or(DebugLevel::Error);
    c_int::from(current.should_log(msg))
}

/// FFI export: Check if event type is stopping
#[no_mangle]
pub extern "C" fn rs_debug_event_is_stopping(event: c_int) -> c_int {
    DebugEventType::from_raw(event).map_or(0, |e| c_int::from(e.is_stopping()))
}

/// FFI export: Check if event type is terminal
#[no_mangle]
pub extern "C" fn rs_debug_event_is_terminal(event: c_int) -> c_int {
    DebugEventType::from_raw(event).map_or(0, |e| c_int::from(e.is_terminal()))
}

/// FFI export: Check if capabilities support breakpoints
#[no_mangle]
pub extern "C" fn rs_debug_caps_breakpoints(flags: u32) -> c_int {
    c_int::from(DebugCapabilities::from_raw(flags).supports_breakpoints())
}

/// FFI export: Check if capabilities support step in
#[no_mangle]
pub extern "C" fn rs_debug_caps_step_in(flags: u32) -> c_int {
    c_int::from(DebugCapabilities::from_raw(flags).supports_step_in())
}

/// FFI export: Check if capabilities support evaluate
#[no_mangle]
pub extern "C" fn rs_debug_caps_evaluate(flags: u32) -> c_int {
    c_int::from(DebugCapabilities::from_raw(flags).supports_evaluate())
}

/// FFI export: Parse debug command from first character
///
/// Returns the command type based on the first character of input.
/// This mirrors the switch statement in do_debug().
#[no_mangle]
pub extern "C" fn rs_debug_parse_command(first_char: u8) -> c_int {
    let cmd = match first_char {
        b'c' => DebugCommand::Continue,
        b'n' => DebugCommand::Next,
        b's' => DebugCommand::Step,
        b'f' => DebugCommand::Finish, // Note: 'fr' is Frame, handled separately
        b'q' => DebugCommand::Quit,
        b'i' => DebugCommand::Interrupt,
        b'b' | b'w' => DebugCommand::Backtrace,
        b'u' => DebugCommand::Up,
        b'd' => DebugCommand::Down,
        _ => DebugCommand::None,
    };
    cmd.as_raw()
}

/// FFI export: Check if debug command exits debug mode
#[no_mangle]
pub extern "C" fn rs_debug_command_exits(cmd: c_int) -> c_int {
    DebugCommand::from_raw(cmd).map_or(0, |c| c_int::from(c.exits_debug_mode()))
}

/// FFI export: Check if debug command continues in debug loop
#[no_mangle]
pub extern "C" fn rs_debug_command_continues_loop(cmd: c_int) -> c_int {
    DebugCommand::from_raw(cmd).map_or(0, |c| c_int::from(c.continues_debug_loop()))
}

/// FFI export: Get Vim breakpoint type from raw value
#[no_mangle]
pub extern "C" fn rs_vim_breakpoint_type_valid(btype: c_int) -> c_int {
    c_int::from(VimBreakpointType::from_raw(btype).is_some())
}

/// FFI export: Get DBG_FUNC constant
#[no_mangle]
pub extern "C" fn rs_vim_dbg_func() -> c_int {
    VimBreakpointType::Func.as_raw()
}

/// FFI export: Get DBG_FILE constant
#[no_mangle]
pub extern "C" fn rs_vim_dbg_file() -> c_int {
    VimBreakpointType::File.as_raw()
}

/// FFI export: Get DBG_EXPR constant
#[no_mangle]
pub extern "C" fn rs_vim_dbg_expr() -> c_int {
    VimBreakpointType::Expr.as_raw()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_level() {
        assert_eq!(DebugLevel::from_raw(0), Some(DebugLevel::None));
        assert_eq!(DebugLevel::from_raw(4), Some(DebugLevel::Debug));
        assert_eq!(DebugLevel::from_raw(100), None);

        let level = DebugLevel::Warning;
        assert!(level.should_log(DebugLevel::Error));
        assert!(level.should_log(DebugLevel::Warning));
        assert!(!level.should_log(DebugLevel::Info));
    }

    #[test]
    fn test_debug_event_type() {
        assert!(DebugEventType::BreakpointHit.is_stopping());
        assert!(DebugEventType::StepComplete.is_stopping());
        assert!(!DebugEventType::Started.is_stopping());

        assert!(DebugEventType::Exited.is_terminal());
        assert!(!DebugEventType::Stopped.is_terminal());
    }

    #[test]
    fn test_debug_capabilities() {
        let caps = DebugCapabilities::none();
        assert!(!caps.supports_breakpoints());

        let caps = DebugCapabilities::from_raw(CAP_SUPPORTS_BREAKPOINTS | CAP_SUPPORTS_STEP_IN);
        assert!(caps.supports_breakpoints());
        assert!(caps.supports_step_in());
        assert!(!caps.supports_evaluate());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_debug_level_valid(0), 1);
        assert_eq!(rs_debug_level_valid(100), 0);

        assert_eq!(rs_debug_level_should_log(2, 1), 1); // Warning level, Error msg
        assert_eq!(rs_debug_level_should_log(2, 3), 0); // Warning level, Info msg

        assert_eq!(rs_debug_event_is_stopping(0), 1);
        assert_eq!(rs_debug_event_is_stopping(5), 0);
    }

    #[test]
    fn test_debug_command() {
        assert_eq!(DebugCommand::from_raw(1), Some(DebugCommand::Continue));
        assert_eq!(DebugCommand::from_raw(100), None);

        assert!(DebugCommand::Continue.exits_debug_mode());
        assert!(DebugCommand::Step.exits_debug_mode());
        assert!(!DebugCommand::Backtrace.exits_debug_mode());

        assert!(DebugCommand::Backtrace.continues_debug_loop());
        assert!(DebugCommand::Up.continues_debug_loop());
        assert!(!DebugCommand::Continue.continues_debug_loop());
    }

    #[test]
    fn test_vim_breakpoint_type() {
        assert_eq!(
            VimBreakpointType::from_raw(1),
            Some(VimBreakpointType::Func)
        );
        assert_eq!(
            VimBreakpointType::from_raw(2),
            Some(VimBreakpointType::File)
        );
        assert_eq!(
            VimBreakpointType::from_raw(3),
            Some(VimBreakpointType::Expr)
        );
        assert_eq!(VimBreakpointType::from_raw(0), None);

        assert_eq!(VimBreakpointType::Func.type_string(), "func");
        assert_eq!(VimBreakpointType::File.type_string(), "file");
        assert_eq!(VimBreakpointType::Expr.type_string(), "expr");
    }

    #[test]
    fn test_debug_command_ffi() {
        // Test command parsing
        assert_eq!(
            rs_debug_parse_command(b'c'),
            DebugCommand::Continue.as_raw()
        );
        assert_eq!(rs_debug_parse_command(b'n'), DebugCommand::Next.as_raw());
        assert_eq!(rs_debug_parse_command(b's'), DebugCommand::Step.as_raw());
        assert_eq!(rs_debug_parse_command(b'q'), DebugCommand::Quit.as_raw());
        assert_eq!(rs_debug_parse_command(b'x'), DebugCommand::None.as_raw());

        // Test exits debug mode
        assert_eq!(rs_debug_command_exits(1), 1); // Continue
        assert_eq!(rs_debug_command_exits(7), 0); // Backtrace

        // Test continues loop
        assert_eq!(rs_debug_command_continues_loop(7), 1); // Backtrace
        assert_eq!(rs_debug_command_continues_loop(1), 0); // Continue
    }

    #[test]
    fn test_vim_breakpoint_ffi() {
        assert_eq!(rs_vim_dbg_func(), 1);
        assert_eq!(rs_vim_dbg_file(), 2);
        assert_eq!(rs_vim_dbg_expr(), 3);

        assert_eq!(rs_vim_breakpoint_type_valid(1), 1);
        assert_eq!(rs_vim_breakpoint_type_valid(0), 0);
    }
}
