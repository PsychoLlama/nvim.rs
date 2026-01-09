//! Command-line entry point utilities
//!
//! This module provides types and utilities for the main command-line
//! entry functions (getcmdline, command_line_enter, etc.).

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Entry Result
// =============================================================================

/// Result of command-line entry.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryResult {
    /// Command line was entered successfully
    #[default]
    Success = 0,
    /// User pressed ESC or Ctrl-C to abort
    Aborted = 1,
    /// Command line is too deeply nested
    TooRecursive = 2,
    /// Error occurred during entry
    Error = 3,
}

impl EntryResult {
    /// Check if entry was successful.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if entry was aborted by user.
    #[must_use]
    pub const fn is_aborted(self) -> bool {
        matches!(self, Self::Aborted)
    }
}

// =============================================================================
// Entry Flags
// =============================================================================

/// Flags for command-line entry behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryFlags {
    /// Clear ccline before entry.
    pub clear_ccline: bool,
    /// Break on Ctrl-C even when caught by try/catch.
    pub break_ctrl_c: bool,
    /// Use right-to-left command line.
    pub cmdmsg_rl: bool,
}

impl EntryFlags {
    /// Create default entry flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            clear_ccline: true,
            break_ctrl_c: false,
            cmdmsg_rl: false,
        }
    }

    /// Create flags for normal command entry.
    #[must_use]
    pub const fn for_command() -> Self {
        Self {
            clear_ccline: true,
            break_ctrl_c: false,
            cmdmsg_rl: false,
        }
    }

    /// Create flags for prompt entry.
    #[must_use]
    pub const fn for_prompt() -> Self {
        Self {
            clear_ccline: false,
            break_ctrl_c: false,
            cmdmsg_rl: false,
        }
    }
}

// =============================================================================
// Entry Context
// =============================================================================

/// Context for command-line entry.
///
/// This captures the state needed when entering command-line mode,
/// including saved state that needs to be restored on exit.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryContext {
    /// First character determining command type (: / ? = > @ or NUL).
    pub firstc: i32,
    /// Count argument (used for incremental search).
    pub count: i32,
    /// Indent for inside conditionals.
    pub indent: i32,
    /// Command-line level when entered.
    pub level: i32,
    /// History type for this command line.
    pub histype: i32,
    /// Whether entry was triggered by -1 firstc (special case).
    pub special_firstc: bool,
}

impl EntryContext {
    /// Create a new entry context.
    #[must_use]
    pub const fn new(firstc: i32, count: i32, indent: i32) -> Self {
        Self {
            firstc,
            count,
            indent,
            level: 0,
            histype: 0,
            special_firstc: firstc == -1,
        }
    }

    /// Get the effective firstc (converts -1 to NUL).
    #[must_use]
    pub const fn effective_firstc(&self) -> i32 {
        if self.firstc == -1 {
            0
        } else {
            self.firstc
        }
    }

    /// Get the command type character for events.
    #[must_use]
    pub const fn cmdline_type(&self) -> i32 {
        let firstc = self.effective_firstc();
        if firstc > 0 {
            firstc
        } else {
            b'-' as i32
        }
    }

    /// Check if this is a search command.
    #[must_use]
    pub const fn is_search(&self) -> bool {
        let firstc = self.effective_firstc();
        firstc == b'/' as i32 || firstc == b'?' as i32
    }

    /// Check if this is an input() function call.
    #[must_use]
    pub const fn is_input_fn(&self) -> bool {
        self.effective_firstc() == b'@' as i32
    }

    /// Check if this should use langmap.
    #[must_use]
    pub const fn use_langmap(&self) -> bool {
        let firstc = self.effective_firstc();
        firstc == b'/' as i32 || firstc == b'?' as i32 || firstc == b'@' as i32
    }
}

// =============================================================================
// Saved State
// =============================================================================

/// State saved when entering command-line mode.
///
/// This is used to restore state when exiting, especially for recursive
/// command-line invocations.
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct SavedEntryState {
    /// Saved msg_scroll value.
    pub msg_scroll: bool,
    /// Saved State value.
    pub state: i32,
    /// Saved cmdpreview value.
    pub cmdpreview: bool,
    /// Previous cursor position (for change detection).
    pub prev_cmdpos: i32,
    /// Whether we saved ccline (recursive call).
    pub did_save_ccline: bool,
    /// Ignore mouse drag/release events initially.
    pub ignore_drag_release: bool,
}

impl SavedEntryState {
    /// Create a new saved state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            msg_scroll: false,
            state: 0,
            cmdpreview: false,
            prev_cmdpos: -1,
            did_save_ccline: false,
            ignore_drag_release: true,
        }
    }

    /// Capture current state for saving.
    pub fn capture(&mut self, msg_scroll: bool, state: i32, cmdpreview: bool) {
        self.msg_scroll = msg_scroll;
        self.state = state;
        self.cmdpreview = cmdpreview;
        self.prev_cmdpos = -1;
        self.ignore_drag_release = true;
    }
}

// =============================================================================
// Recursion Guard
// =============================================================================

/// Maximum cmdline recursion level.
pub const MAX_CMDLINE_LEVEL: i32 = 50;

/// Check if cmdline level is at maximum recursion depth.
#[must_use]
pub const fn is_at_max_level(level: i32) -> bool {
    level >= MAX_CMDLINE_LEVEL
}

/// Check if cmdline can be entered at the given level.
#[must_use]
pub const fn can_enter(level: i32) -> bool {
    level < MAX_CMDLINE_LEVEL
}

// =============================================================================
// Mode State
// =============================================================================

/// Mode value for cmdline mode.
pub const MODE_CMDLINE: i32 = 0x0010; // From state_defs.h

/// Mode value for langmap mode flag.
pub const MODE_LANGMAP: i32 = 0x8000; // From state_defs.h

/// Combine cmdline mode with langmap flag.
#[must_use]
pub const fn cmdline_mode_with_langmap(use_langmap: bool) -> i32 {
    if use_langmap {
        MODE_CMDLINE | MODE_LANGMAP
    } else {
        MODE_CMDLINE
    }
}

// =============================================================================
// History Type Mapping
// =============================================================================

/// History types for command-line.
pub mod hist {
    /// Command history (:)
    pub const CMD: i32 = 0;
    /// Search history (/ ?)
    pub const SEARCH: i32 = 1;
    /// Expression history (=)
    pub const EXPR: i32 = 2;
    /// Input history (@)
    pub const INPUT: i32 = 3;
    /// Debug history (>)
    pub const DEBUG: i32 = 4;
}

/// Map firstc to history type.
#[must_use]
pub const fn hist_char2type(firstc: i32) -> i32 {
    match firstc {
        c if c == b':' as i32 => hist::CMD,
        c if c == b'/' as i32 || c == b'?' as i32 => hist::SEARCH,
        c if c == b'=' as i32 => hist::EXPR,
        c if c == b'@' as i32 => hist::INPUT,
        c if c == b'>' as i32 => hist::DEBUG,
        _ => hist::CMD,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if entry result is success (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_result_is_success(result: c_int) -> c_int {
    let r = match result {
        0 => EntryResult::Success,
        1 => EntryResult::Aborted,
        2 => EntryResult::TooRecursive,
        _ => EntryResult::Error,
    };
    c_int::from(r.is_success())
}

/// Check if entry result is aborted (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_result_is_aborted(result: c_int) -> c_int {
    let r = match result {
        0 => EntryResult::Success,
        1 => EntryResult::Aborted,
        2 => EntryResult::TooRecursive,
        _ => EntryResult::Error,
    };
    c_int::from(r.is_aborted())
}

/// Check if cmdline level is at max recursion (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_is_at_max_level(level: c_int) -> c_int {
    c_int::from(is_at_max_level(level))
}

/// Map firstc to history type (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_hist_char2type(firstc: c_int) -> c_int {
    hist_char2type(firstc)
}

/// Get effective firstc (converts -1 to 0) (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_effective_firstc(firstc: c_int) -> c_int {
    if firstc == -1 {
        0
    } else {
        firstc
    }
}

/// Get cmdline type for events (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_cmdline_type(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    if effective > 0 {
        effective
    } else {
        c_int::from(b'-')
    }
}

/// Check if firstc indicates search command (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_is_search(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    c_int::from(effective == c_int::from(b'/') || effective == c_int::from(b'?'))
}

/// Check if firstc indicates input function (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_is_input_fn(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    c_int::from(effective == c_int::from(b'@'))
}

/// Check if firstc should use langmap (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_use_langmap(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    c_int::from(
        effective == c_int::from(b'/')
            || effective == c_int::from(b'?')
            || effective == c_int::from(b'@'),
    )
}

/// Get cmdline mode value with optional langmap (FFI).
#[no_mangle]
pub extern "C" fn rs_entry_cmdline_mode(use_langmap: c_int) -> c_int {
    cmdline_mode_with_langmap(use_langmap != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_result() {
        assert!(EntryResult::Success.is_success());
        assert!(!EntryResult::Aborted.is_success());
        assert!(!EntryResult::TooRecursive.is_success());
        assert!(!EntryResult::Error.is_success());

        assert!(!EntryResult::Success.is_aborted());
        assert!(EntryResult::Aborted.is_aborted());
    }

    #[test]
    fn test_entry_flags() {
        let default = EntryFlags::new();
        assert!(default.clear_ccline);
        assert!(!default.break_ctrl_c);

        let cmd_flags = EntryFlags::for_command();
        assert!(cmd_flags.clear_ccline);

        let prompt_flags = EntryFlags::for_prompt();
        assert!(!prompt_flags.clear_ccline);
    }

    #[test]
    fn test_entry_context() {
        let ctx = EntryContext::new(i32::from(b':'), 1, 0);
        assert_eq!(ctx.firstc, i32::from(b':'));
        assert_eq!(ctx.effective_firstc(), i32::from(b':'));
        assert!(!ctx.is_search());
        assert!(!ctx.is_input_fn());
        assert!(!ctx.use_langmap());

        let search = EntryContext::new(i32::from(b'/'), 1, 0);
        assert!(search.is_search());
        assert!(search.use_langmap());

        let special = EntryContext::new(-1, 1, 0);
        assert!(special.special_firstc);
        assert_eq!(special.effective_firstc(), 0);
        assert_eq!(special.cmdline_type(), i32::from(b'-'));
    }

    #[test]
    fn test_saved_state() {
        let mut state = SavedEntryState::new();
        assert_eq!(state.prev_cmdpos, -1);
        assert!(state.ignore_drag_release);

        state.capture(true, 42, false);
        assert!(state.msg_scroll);
        assert_eq!(state.state, 42);
        assert!(!state.cmdpreview);
    }

    #[test]
    fn test_recursion_guard() {
        assert!(!is_at_max_level(0));
        assert!(!is_at_max_level(49));
        assert!(is_at_max_level(50));
        assert!(is_at_max_level(100));

        assert!(can_enter(0));
        assert!(can_enter(49));
        assert!(!can_enter(50));
    }

    #[test]
    fn test_mode_values() {
        assert_eq!(MODE_CMDLINE, 0x0010);
        assert_eq!(MODE_LANGMAP, 0x8000);

        let mode = cmdline_mode_with_langmap(false);
        assert_eq!(mode, MODE_CMDLINE);

        let mode_lm = cmdline_mode_with_langmap(true);
        assert_eq!(mode_lm, MODE_CMDLINE | MODE_LANGMAP);
    }

    #[test]
    fn test_hist_char2type() {
        assert_eq!(hist_char2type(i32::from(b':')), hist::CMD);
        assert_eq!(hist_char2type(i32::from(b'/')), hist::SEARCH);
        assert_eq!(hist_char2type(i32::from(b'?')), hist::SEARCH);
        assert_eq!(hist_char2type(i32::from(b'=')), hist::EXPR);
        assert_eq!(hist_char2type(i32::from(b'@')), hist::INPUT);
        assert_eq!(hist_char2type(i32::from(b'>')), hist::DEBUG);
        assert_eq!(hist_char2type(0), hist::CMD);
    }
}
