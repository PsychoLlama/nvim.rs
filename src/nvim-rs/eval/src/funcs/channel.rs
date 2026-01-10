//! Channel and job functions for VimL.
//!
//! This module implements channel/job-related VimL functions from `src/nvim/eval/funcs.c`:
//! - Channel ID validation and lookup
//! - Job status helpers
//! - Channel mode handling
//!
//! These are helpers for the channel infrastructure. Actual channel operations
//! use the nvim-channel crate.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]

use std::ffi::c_int;

// =============================================================================
// Channel Types and IDs
// =============================================================================

/// Channel ID type (matches C's uint64_t channel id).
pub type ChannelId = u64;

/// Special channel IDs.
pub mod channel_ids {
    use super::ChannelId;

    /// Invalid channel ID.
    pub const INVALID: ChannelId = 0;
    /// Internal channel (Lua callbacks, etc.).
    pub const INTERNAL: ChannelId = u64::MAX;
}

/// Channel mode (stream type).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
    /// Bytes mode (raw data)
    Bytes = 0,
    /// Terminal mode (pty)
    Terminal = 1,
    /// RPC mode (msgpack-rpc)
    Rpc = 2,
    /// Internal (Lua integration)
    Internal = 3,
}

impl ChannelMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Bytes),
            1 => Some(Self::Terminal),
            2 => Some(Self::Rpc),
            3 => Some(Self::Internal),
            _ => None,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// Channel stream part.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPart {
    /// Standard input
    Stdin = 0,
    /// Standard output
    Stdout = 1,
    /// Standard error
    Stderr = 2,
    /// All streams
    All = 3,
}

impl ChannelPart {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Stdin),
            1 => Some(Self::Stdout),
            2 => Some(Self::Stderr),
            3 => Some(Self::All),
            _ => None,
        }
    }
}

// =============================================================================
// Job Status
// =============================================================================

/// Job exit status.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct JobStatus {
    /// Whether the job has exited.
    pub exited: bool,
    /// Exit code (valid only if exited is true).
    pub exit_code: c_int,
    /// Whether terminated by signal.
    pub signaled: bool,
    /// Signal number if signaled (0 otherwise).
    pub signal: c_int,
}

impl JobStatus {
    /// Create running status.
    pub const fn running() -> Self {
        Self {
            exited: false,
            exit_code: 0,
            signaled: false,
            signal: 0,
        }
    }

    /// Create exited status.
    pub const fn exited(exit_code: c_int) -> Self {
        Self {
            exited: true,
            exit_code,
            signaled: false,
            signal: 0,
        }
    }

    /// Create signaled status.
    pub const fn signaled(signal: c_int) -> Self {
        Self {
            exited: true,
            exit_code: 128 + signal,
            signaled: true,
            signal,
        }
    }

    /// Check if job is still running.
    pub const fn is_running(&self) -> bool {
        !self.exited
    }

    /// Check if job exited successfully.
    pub const fn is_success(&self) -> bool {
        self.exited && self.exit_code == 0 && !self.signaled
    }
}

/// FFI export: create running job status.
#[no_mangle]
pub extern "C" fn rs_job_status_running() -> JobStatus {
    JobStatus::running()
}

/// FFI export: create exited job status.
#[no_mangle]
pub extern "C" fn rs_job_status_exited(exit_code: c_int) -> JobStatus {
    JobStatus::exited(exit_code)
}

/// FFI export: create signaled job status.
#[no_mangle]
pub extern "C" fn rs_job_status_signaled(signal: c_int) -> JobStatus {
    JobStatus::signaled(signal)
}

/// FFI export: check if job is running.
#[no_mangle]
pub extern "C" fn rs_job_is_running(status: *const JobStatus) -> bool {
    if status.is_null() {
        return false;
    }
    unsafe { (*status).is_running() }
}

// =============================================================================
// Channel Validation
// =============================================================================

/// Check if a channel ID is valid.
pub const fn is_valid_channel_id(id: ChannelId) -> bool {
    id != channel_ids::INVALID
}

/// Check if a channel ID is the internal channel.
pub const fn is_internal_channel(id: ChannelId) -> bool {
    id == channel_ids::INTERNAL
}

/// FFI export: validate channel ID.
#[no_mangle]
pub extern "C" fn rs_is_valid_channel_id(id: u64) -> bool {
    is_valid_channel_id(id)
}

/// FFI export: check if internal channel.
#[no_mangle]
pub extern "C" fn rs_is_internal_channel(id: u64) -> bool {
    is_internal_channel(id)
}

// =============================================================================
// Channel Options
// =============================================================================

/// Channel callback types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackType {
    /// No callback
    None = 0,
    /// VimL function name
    Function = 1,
    /// Lambda/partial
    Partial = 2,
    /// Lua function reference
    LuaRef = 3,
}

impl CallbackType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::Function),
            2 => Some(Self::Partial),
            3 => Some(Self::LuaRef),
            _ => None,
        }
    }
}

/// Channel options flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ChannelOptions {
    /// Detach from channel after starting.
    pub detach: bool,
    /// Use pty for job.
    pub pty: bool,
    /// Keep environment from parent.
    pub env: bool,
    /// Clear environment before setting.
    pub clear_env: bool,
    /// Overlapped I/O (Windows).
    pub overlapped: bool,
    /// Stdin is a file descriptor.
    pub stdin_fd: bool,
    /// RPC mode.
    pub rpc: bool,
}

impl ChannelOptions {
    /// Create default options.
    pub const fn new() -> Self {
        Self {
            detach: false,
            pty: false,
            env: true, // Default to inherit environment
            clear_env: false,
            overlapped: false,
            stdin_fd: false,
            rpc: false,
        }
    }

    /// Check if using terminal mode.
    pub const fn is_terminal(&self) -> bool {
        self.pty
    }

    /// Check if using RPC mode.
    pub const fn is_rpc(&self) -> bool {
        self.rpc
    }
}

/// FFI export: create default channel options.
#[no_mangle]
pub extern "C" fn rs_channel_options_new() -> ChannelOptions {
    ChannelOptions::new()
}

/// FFI export: check if terminal mode.
#[no_mangle]
pub extern "C" fn rs_channel_options_is_terminal(opts: *const ChannelOptions) -> bool {
    if opts.is_null() {
        return false;
    }
    unsafe { (*opts).is_terminal() }
}

// =============================================================================
// Job Control
// =============================================================================

/// Signals that can be sent to jobs.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobSignal {
    /// Terminate gracefully (SIGTERM on Unix)
    Term = 0,
    /// Kill immediately (SIGKILL on Unix)
    Kill = 1,
    /// Interrupt (SIGINT on Unix)
    Int = 2,
    /// Hangup (SIGHUP on Unix)
    Hup = 3,
}

impl JobSignal {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Term),
            1 => Some(Self::Kill),
            2 => Some(Self::Int),
            3 => Some(Self::Hup),
            _ => None,
        }
    }

    /// Convert to Unix signal number.
    #[cfg(unix)]
    pub const fn to_unix_signal(self) -> c_int {
        match self {
            Self::Term => 15, // SIGTERM
            Self::Kill => 9,  // SIGKILL
            Self::Int => 2,   // SIGINT
            Self::Hup => 1,   // SIGHUP
        }
    }
}

/// FFI export: convert job signal to Unix signal number.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn rs_job_signal_to_unix(signal: c_int) -> c_int {
    JobSignal::from_c_int(signal)
        .map(|s| s.to_unix_signal())
        .unwrap_or(-1)
}

#[cfg(not(unix))]
#[no_mangle]
pub extern "C" fn rs_job_signal_to_unix(_signal: c_int) -> c_int {
    -1
}

// =============================================================================
// Message Encoding
// =============================================================================

/// Channel message format.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageFormat {
    /// Raw bytes
    Raw = 0,
    /// Newline-delimited
    Lines = 1,
    /// JSON messages
    Json = 2,
    /// MessagePack
    Msgpack = 3,
}

impl MessageFormat {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Raw),
            1 => Some(Self::Lines),
            2 => Some(Self::Json),
            3 => Some(Self::Msgpack),
            _ => None,
        }
    }

    /// Check if format buffers messages.
    pub const fn is_buffered(self) -> bool {
        matches!(self, Self::Lines | Self::Json | Self::Msgpack)
    }
}

/// FFI export: check if message format is buffered.
#[no_mangle]
pub extern "C" fn rs_message_format_is_buffered(format: c_int) -> bool {
    MessageFormat::from_c_int(format)
        .map(|f| f.is_buffered())
        .unwrap_or(false)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_mode() {
        assert_eq!(ChannelMode::from_c_int(0), Some(ChannelMode::Bytes));
        assert_eq!(ChannelMode::from_c_int(2), Some(ChannelMode::Rpc));
        assert_eq!(ChannelMode::from_c_int(99), None);
        assert_eq!(ChannelMode::Terminal.to_c_int(), 1);
    }

    #[test]
    fn test_channel_part() {
        assert_eq!(ChannelPart::from_c_int(0), Some(ChannelPart::Stdin));
        assert_eq!(ChannelPart::from_c_int(1), Some(ChannelPart::Stdout));
        assert_eq!(ChannelPart::from_c_int(3), Some(ChannelPart::All));
    }

    #[test]
    fn test_job_status() {
        let running = JobStatus::running();
        assert!(running.is_running());
        assert!(!running.is_success());

        let success = JobStatus::exited(0);
        assert!(!success.is_running());
        assert!(success.is_success());

        let failed = JobStatus::exited(1);
        assert!(!failed.is_running());
        assert!(!failed.is_success());

        let killed = JobStatus::signaled(9);
        assert!(!killed.is_running());
        assert!(!killed.is_success());
        assert!(killed.signaled);
        assert_eq!(killed.signal, 9);
        assert_eq!(killed.exit_code, 137); // 128 + 9
    }

    #[test]
    fn test_channel_ids() {
        assert!(!is_valid_channel_id(channel_ids::INVALID));
        assert!(is_valid_channel_id(1));
        assert!(is_valid_channel_id(channel_ids::INTERNAL));

        assert!(is_internal_channel(channel_ids::INTERNAL));
        assert!(!is_internal_channel(1));
    }

    #[test]
    fn test_channel_options() {
        let opts = ChannelOptions::new();
        assert!(!opts.is_terminal());
        assert!(!opts.is_rpc());
        assert!(opts.env); // Default to inherit

        let mut opts = ChannelOptions::new();
        opts.pty = true;
        assert!(opts.is_terminal());

        opts.rpc = true;
        assert!(opts.is_rpc());
    }

    #[test]
    fn test_job_signal() {
        assert_eq!(JobSignal::from_c_int(0), Some(JobSignal::Term));
        assert_eq!(JobSignal::from_c_int(1), Some(JobSignal::Kill));

        #[cfg(unix)]
        {
            assert_eq!(JobSignal::Term.to_unix_signal(), 15);
            assert_eq!(JobSignal::Kill.to_unix_signal(), 9);
            assert_eq!(JobSignal::Int.to_unix_signal(), 2);
        }
    }

    #[test]
    fn test_message_format() {
        assert_eq!(MessageFormat::from_c_int(0), Some(MessageFormat::Raw));
        assert_eq!(MessageFormat::from_c_int(1), Some(MessageFormat::Lines));

        assert!(!MessageFormat::Raw.is_buffered());
        assert!(MessageFormat::Lines.is_buffered());
        assert!(MessageFormat::Json.is_buffered());
        assert!(MessageFormat::Msgpack.is_buffered());
    }

    #[test]
    fn test_callback_type() {
        assert_eq!(CallbackType::from_c_int(0), Some(CallbackType::None));
        assert_eq!(CallbackType::from_c_int(1), Some(CallbackType::Function));
        assert_eq!(CallbackType::from_c_int(3), Some(CallbackType::LuaRef));
    }
}
