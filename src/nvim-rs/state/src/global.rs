//! Global state management
//!
//! This module provides infrastructure for managing Neovim's global state,
//! including state initialization, access, and modification.

use std::ffi::c_int;

// =============================================================================
// Global State Status
// =============================================================================

/// Status of global state initialization.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlobalStateStatus {
    /// Not initialized
    #[default]
    Uninitialized = 0,
    /// Initializing
    Initializing = 1,
    /// Fully initialized
    Ready = 2,
    /// Shutting down
    ShuttingDown = 3,
    /// Terminated
    Terminated = 4,
}

impl GlobalStateStatus {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Initializing,
            2 => Self::Ready,
            3 => Self::ShuttingDown,
            4 => Self::Terminated,
            _ => Self::Uninitialized,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if state is usable.
    #[must_use]
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Check if state is transitioning.
    #[must_use]
    pub const fn is_transitioning(self) -> bool {
        matches!(self, Self::Initializing | Self::ShuttingDown)
    }
}

/// FFI: Check if global state is usable.
#[no_mangle]
pub extern "C" fn rs_global_state_is_usable(status: c_int) -> c_int {
    c_int::from(GlobalStateStatus::from_c_int(status).is_usable())
}

/// FFI: Check if global state is transitioning.
#[no_mangle]
pub extern "C" fn rs_global_state_is_transitioning(status: c_int) -> c_int {
    c_int::from(GlobalStateStatus::from_c_int(status).is_transitioning())
}

// =============================================================================
// Global Counts
// =============================================================================

/// Counters for global state tracking.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalCounts {
    /// Number of open buffers
    pub buffers: c_int,
    /// Number of open windows
    pub windows: c_int,
    /// Number of tab pages
    pub tabpages: c_int,
    /// Number of active channels
    pub channels: c_int,
    /// Number of pending jobs
    pub jobs: c_int,
    /// Number of active timers
    pub timers: c_int,
}

impl GlobalCounts {
    /// Create new counts.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffers: 0,
            windows: 0,
            tabpages: 0,
            channels: 0,
            jobs: 0,
            timers: 0,
        }
    }

    /// Check if there are pending async operations.
    #[must_use]
    pub const fn has_pending_async(&self) -> bool {
        self.channels > 0 || self.jobs > 0 || self.timers > 0
    }
}

/// FFI: Create global counts.
#[no_mangle]
pub extern "C" fn rs_global_counts_new() -> GlobalCounts {
    GlobalCounts::new()
}

/// FFI: Check if has pending async.
///
/// # Safety
/// `counts` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_global_has_pending_async(counts: *const GlobalCounts) -> c_int {
    if counts.is_null() {
        return 0;
    }
    c_int::from((*counts).has_pending_async())
}

// =============================================================================
// Global Options State
// =============================================================================

/// State of global options.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalOptionsState {
    /// Options are locked
    pub locked: bool,
    /// Options are being restored
    pub restoring: bool,
    /// Number of options set
    pub set_count: c_int,
    /// Number of options with non-default values
    pub changed_count: c_int,
}

impl GlobalOptionsState {
    /// Create new options state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            locked: false,
            restoring: false,
            set_count: 0,
            changed_count: 0,
        }
    }

    /// Check if options can be modified.
    #[must_use]
    pub const fn can_modify(&self) -> bool {
        !self.locked && !self.restoring
    }
}

/// FFI: Create options state.
#[no_mangle]
pub extern "C" fn rs_global_options_state_new() -> GlobalOptionsState {
    GlobalOptionsState::new()
}

/// FFI: Check if options can be modified.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_global_options_can_modify(state: *const GlobalOptionsState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).can_modify())
}

// =============================================================================
// Global Flags
// =============================================================================

/// Miscellaneous global flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalFlags {
    /// Currently reading commands
    pub reading: bool,
    /// In the main loop
    pub in_main_loop: bool,
    /// Processing messages
    pub processing_msgs: bool,
    /// Exiting
    pub exiting: bool,
    /// Full screen update needed
    pub need_redraw: bool,
    /// Clear screen needed
    pub need_clear: bool,
    /// Status line update needed
    pub need_status: bool,
    /// Command line update needed
    pub need_cmdline: bool,
}

impl GlobalFlags {
    /// Create new global flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reading: false,
            in_main_loop: false,
            processing_msgs: false,
            exiting: false,
            need_redraw: false,
            need_clear: false,
            need_status: false,
            need_cmdline: false,
        }
    }

    /// Check if any redraw is needed.
    #[must_use]
    pub const fn needs_any_redraw(&self) -> bool {
        self.need_redraw || self.need_clear || self.need_status || self.need_cmdline
    }

    /// Clear all redraw flags.
    pub fn clear_redraw_flags(&mut self) {
        self.need_redraw = false;
        self.need_clear = false;
        self.need_status = false;
        self.need_cmdline = false;
    }
}

/// FFI: Create global flags.
#[no_mangle]
pub extern "C" fn rs_global_flags_new() -> GlobalFlags {
    GlobalFlags::new()
}

/// FFI: Check if any redraw needed.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_global_needs_any_redraw(flags: *const GlobalFlags) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from((*flags).needs_any_redraw())
}

/// FFI: Clear redraw flags.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_global_clear_redraw_flags(flags: *mut GlobalFlags) {
    if !flags.is_null() {
        (*flags).clear_redraw_flags();
    }
}

// =============================================================================
// Session Info
// =============================================================================

/// Information about the current session.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SessionInfo {
    /// Session ID
    pub id: c_int,
    /// Start time (ms since epoch)
    pub start_time_ms: i64,
    /// Number of commands executed
    pub command_count: u64,
    /// Number of keystrokes
    pub keystroke_count: u64,
    /// Number of edits
    pub edit_count: u64,
}

impl SessionInfo {
    /// Create new session info.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            id,
            start_time_ms: 0,
            command_count: 0,
            keystroke_count: 0,
            edit_count: 0,
        }
    }

    /// Record a command.
    pub fn record_command(&mut self) {
        self.command_count += 1;
    }

    /// Record keystrokes.
    pub fn record_keystrokes(&mut self, count: u64) {
        self.keystroke_count += count;
    }

    /// Record an edit.
    pub fn record_edit(&mut self) {
        self.edit_count += 1;
    }
}

/// FFI: Create session info.
#[no_mangle]
pub extern "C" fn rs_session_info_new(id: c_int) -> SessionInfo {
    SessionInfo::new(id)
}

/// FFI: Record command.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_session_record_command(info: *mut SessionInfo) {
    if !info.is_null() {
        (*info).record_command();
    }
}

/// FFI: Get command count.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_session_command_count(info: *const SessionInfo) -> u64 {
    if info.is_null() {
        return 0;
    }
    (*info).command_count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_state_status() {
        assert!(!GlobalStateStatus::Uninitialized.is_usable());
        assert!(GlobalStateStatus::Ready.is_usable());
        assert!(GlobalStateStatus::Initializing.is_transitioning());
        assert!(GlobalStateStatus::ShuttingDown.is_transitioning());
    }

    #[test]
    fn test_global_counts() {
        let mut counts = GlobalCounts::new();
        assert!(!counts.has_pending_async());

        counts.jobs = 1;
        assert!(counts.has_pending_async());
    }

    #[test]
    fn test_global_options_state() {
        let mut state = GlobalOptionsState::new();
        assert!(state.can_modify());

        state.locked = true;
        assert!(!state.can_modify());
    }

    #[test]
    fn test_global_flags() {
        let mut flags = GlobalFlags::new();
        assert!(!flags.needs_any_redraw());

        flags.need_redraw = true;
        assert!(flags.needs_any_redraw());

        flags.clear_redraw_flags();
        assert!(!flags.needs_any_redraw());
    }

    #[test]
    fn test_session_info() {
        let mut info = SessionInfo::new(1);
        assert_eq!(info.command_count, 0);

        info.record_command();
        info.record_command();
        assert_eq!(info.command_count, 2);
    }
}
