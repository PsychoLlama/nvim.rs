//! Command-line window functionality
//!
//! This module provides types and utilities for the command-line window (q:, q/, q?),
//! which allows editing command history in a regular window.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Command Window Type
// =============================================================================

/// Type of command-line window.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CmdwinType {
    /// Not in command-line window
    #[default]
    None = 0,
    /// Ex command history (q:)
    Ex = b':' as i32,
    /// Forward search history (q/)
    ForwardSearch = b'/' as i32,
    /// Backward search history (q?)
    BackwardSearch = b'?' as i32,
    /// Expression history (q=)
    Expression = b'=' as i32,
    /// Input history (q@)
    Input = b'@' as i32,
    /// Debug history (q>)
    Debug = b'>' as i32,
}

impl CmdwinType {
    /// Parse from character.
    #[must_use]
    pub const fn from_char(c: i32) -> Self {
        match c {
            c if c == b':' as i32 => Self::Ex,
            c if c == b'/' as i32 => Self::ForwardSearch,
            c if c == b'?' as i32 => Self::BackwardSearch,
            c if c == b'=' as i32 => Self::Expression,
            c if c == b'@' as i32 => Self::Input,
            c if c == b'>' as i32 => Self::Debug,
            _ => Self::None,
        }
    }

    /// Get character representation.
    #[must_use]
    pub const fn to_char(self) -> Option<u8> {
        match self {
            Self::None => None,
            Self::Ex => Some(b':'),
            Self::ForwardSearch => Some(b'/'),
            Self::BackwardSearch => Some(b'?'),
            Self::Expression => Some(b'='),
            Self::Input => Some(b'@'),
            Self::Debug => Some(b'>'),
        }
    }

    /// Check if this is a search type.
    #[must_use]
    pub const fn is_search(self) -> bool {
        matches!(self, Self::ForwardSearch | Self::BackwardSearch)
    }

    /// Check if command window is active.
    #[must_use]
    pub const fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Command Window State
// =============================================================================

/// State for command-line window.
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdwinState {
    /// Type of command-line window.
    pub win_type: CmdwinType,
    /// Command-line level when opened.
    pub level: i32,
    /// Result when closing (key code or 0).
    pub result: i32,
}

impl CmdwinState {
    /// Create a new command window state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            win_type: CmdwinType::None,
            level: 0,
            result: 0,
        }
    }

    /// Check if command window is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.win_type.is_active()
    }

    /// Initialize for opening command window.
    pub fn open(&mut self, win_type: CmdwinType, level: i32) {
        self.win_type = win_type;
        self.level = level;
        self.result = 0;
    }

    /// Close command window.
    pub fn close(&mut self, result: i32) {
        self.result = result;
        self.win_type = CmdwinType::None;
        self.level = 0;
    }

    /// Reset state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Result Codes
// =============================================================================

/// Result codes for command window closure.
pub mod result {
    use std::ffi::c_int;

    /// Closed normally with Enter.
    pub const ENTER: c_int = 13; // CR

    /// Closed with Ctrl-C (abort).
    pub const CTRL_C: c_int = 3;

    /// Closed with ESC (cancel).
    pub const ESC: c_int = 27;

    /// Closed with K_IGNORE (ignore).
    pub const IGNORE: c_int = -1;

    /// Check if result means execute the line.
    #[must_use]
    pub const fn should_execute(r: c_int) -> bool {
        r == ENTER
    }

    /// Check if result means cancel.
    #[must_use]
    pub const fn should_cancel(r: c_int) -> bool {
        r == CTRL_C || r == ESC
    }
}

// =============================================================================
// Open Restrictions
// =============================================================================

/// Reasons why command window cannot be opened.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdwinOpenError {
    /// Can be opened.
    Ok = 0,
    /// Already in command window.
    AlreadyInCmdwin = 1,
    /// Text or buffer is locked.
    TextLocked = 2,
    /// In secret mode (cmdline_star).
    SecretMode = 3,
    /// No room for window.
    NoRoom = 4,
}

impl CmdwinOpenError {
    /// Check if opening is allowed.
    #[must_use]
    pub const fn can_open(self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// History Type Mapping
// =============================================================================

/// Map command window type to history type.
#[must_use]
pub const fn cmdwin_to_hist_type(win_type: CmdwinType) -> i32 {
    // History type constants from cmdhist.h
    const HIST_CMD: i32 = 0;
    const HIST_SEARCH: i32 = 1;
    const HIST_EXPR: i32 = 2;
    const HIST_INPUT: i32 = 3;
    const HIST_DEBUG: i32 = 4;

    match win_type {
        CmdwinType::ForwardSearch | CmdwinType::BackwardSearch => HIST_SEARCH,
        CmdwinType::Expression => HIST_EXPR,
        CmdwinType::Input => HIST_INPUT,
        CmdwinType::Debug => HIST_DEBUG,
        CmdwinType::Ex | CmdwinType::None => HIST_CMD,
    }
}

// =============================================================================
// Command Window Open Validation
// =============================================================================

/// Check if command window can be opened based on current state.
///
/// Returns an error code if it cannot be opened, or Ok if it can.
#[must_use]
pub const fn can_open_cmdwin(
    cmdwin_type_active: bool,
    text_locked: bool,
    cmdline_star: i32,
) -> CmdwinOpenError {
    if cmdwin_type_active {
        return CmdwinOpenError::AlreadyInCmdwin;
    }
    if text_locked {
        return CmdwinOpenError::TextLocked;
    }
    if cmdline_star > 0 {
        return CmdwinOpenError::SecretMode;
    }
    CmdwinOpenError::Ok
}

/// Check if window split validation failed.
///
/// After win_split(), check if autocommands messed with the old window.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub const fn cmdwin_split_invalid(
    old_curwin_valid: bool,
    curwin_is_old: bool,
    old_curbuf_valid: bool,
    buf_changed: bool,
) -> bool {
    !old_curwin_valid || curwin_is_old || !old_curbuf_valid || buf_changed
}

/// Check if buffer creation for cmdwin failed.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub const fn cmdwin_buffer_invalid(
    newbuf_status_ok: bool,
    cmdwin_valid: bool,
    curwin_is_cmdwin: bool,
    old_curwin_valid: bool,
    old_curbuf_valid: bool,
    buf_changed: bool,
) -> bool {
    !newbuf_status_ok
        || !cmdwin_valid
        || !curwin_is_cmdwin
        || !old_curwin_valid
        || !old_curbuf_valid
        || buf_changed
}

// =============================================================================
// Command Window Tab Mapping
// =============================================================================

/// Check if Tab key should be mapped for completion in cmdwin.
///
/// Tab completion mapping is added for Ex and Debug command windows.
#[must_use]
pub const fn cmdwin_needs_tab_mapping(histtype: i32, p_wc: i32) -> bool {
    // TAB = 9
    const TAB: i32 = 9;
    // HIST_CMD = 0, HIST_DEBUG = 4
    const HIST_CMD: i32 = 0;
    const HIST_DEBUG: i32 = 4;

    if p_wc != TAB {
        return false;
    }
    histtype == HIST_CMD || histtype == HIST_DEBUG
}

/// Check if cmdwin should set vim filetype.
///
/// Ex and Debug command windows get vim filetype for syntax highlighting.
#[must_use]
pub const fn cmdwin_needs_vim_filetype(histtype: i32) -> bool {
    const HIST_CMD: i32 = 0;
    const HIST_DEBUG: i32 = 4;
    histtype == HIST_CMD || histtype == HIST_DEBUG
}

// =============================================================================
// Command Window Cleanup Validation
// =============================================================================

/// Check if command window cleanup detected an error (window/buffer changed).
#[must_use]
pub const fn cmdwin_cleanup_had_error(
    old_curwin_valid: bool,
    old_curbuf_valid: bool,
    buf_changed: bool,
) -> bool {
    !old_curwin_valid || !old_curbuf_valid || buf_changed
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if command window can be opened (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_can_open(
    cmdwin_type_active: c_int,
    text_locked: c_int,
    cmdline_star: c_int,
) -> c_int {
    can_open_cmdwin(cmdwin_type_active != 0, text_locked != 0, cmdline_star) as c_int
}

/// Check if split validation failed (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_split_invalid(
    old_curwin_valid: c_int,
    curwin_is_old: c_int,
    old_curbuf_valid: c_int,
    buf_changed: c_int,
) -> c_int {
    c_int::from(cmdwin_split_invalid(
        old_curwin_valid != 0,
        curwin_is_old != 0,
        old_curbuf_valid != 0,
        buf_changed != 0,
    ))
}

/// Check if buffer creation validation failed (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_buffer_invalid(
    newbuf_status_ok: c_int,
    cmdwin_valid: c_int,
    curwin_is_cmdwin: c_int,
    old_curwin_valid: c_int,
    old_curbuf_valid: c_int,
    buf_changed: c_int,
) -> c_int {
    c_int::from(cmdwin_buffer_invalid(
        newbuf_status_ok != 0,
        cmdwin_valid != 0,
        curwin_is_cmdwin != 0,
        old_curwin_valid != 0,
        old_curbuf_valid != 0,
        buf_changed != 0,
    ))
}

/// Check if Tab mapping is needed for cmdwin (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_needs_tab_mapping(histtype: c_int, p_wc: c_int) -> c_int {
    c_int::from(cmdwin_needs_tab_mapping(histtype, p_wc))
}

/// Check if vim filetype is needed for cmdwin (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_needs_vim_filetype(histtype: c_int) -> c_int {
    c_int::from(cmdwin_needs_vim_filetype(histtype))
}

/// Check if cleanup detected an error (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_cleanup_had_error(
    old_curwin_valid: c_int,
    old_curbuf_valid: c_int,
    buf_changed: c_int,
) -> c_int {
    c_int::from(cmdwin_cleanup_had_error(
        old_curwin_valid != 0,
        old_curbuf_valid != 0,
        buf_changed != 0,
    ))
}

/// Get command window type from char (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_type_from_char_v2(c: c_int) -> c_int {
    CmdwinType::from_char(c) as c_int
}

/// Check if command window type is active (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_is_active(win_type: c_int) -> c_int {
    c_int::from(CmdwinType::from_char(win_type).is_active())
}

/// Check if command window type is search (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_is_search(win_type: c_int) -> c_int {
    c_int::from(CmdwinType::from_char(win_type).is_search())
}

/// Get history type for command window type (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_to_hist_type(win_type: c_int) -> c_int {
    cmdwin_to_hist_type(CmdwinType::from_char(win_type))
}

/// Check if result means execute (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_result_should_execute(result: c_int) -> c_int {
    c_int::from(result::should_execute(result))
}

/// Check if result means cancel (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_result_should_cancel(result: c_int) -> c_int {
    c_int::from(result::should_cancel(result))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdwin_type() {
        assert_eq!(CmdwinType::from_char(i32::from(b':')), CmdwinType::Ex);
        assert_eq!(
            CmdwinType::from_char(i32::from(b'/')),
            CmdwinType::ForwardSearch
        );
        assert_eq!(CmdwinType::from_char(0), CmdwinType::None);

        assert!(CmdwinType::Ex.is_active());
        assert!(!CmdwinType::None.is_active());

        assert!(CmdwinType::ForwardSearch.is_search());
        assert!(!CmdwinType::Ex.is_search());

        assert_eq!(CmdwinType::Ex.to_char(), Some(b':'));
        assert_eq!(CmdwinType::None.to_char(), None);
    }

    #[test]
    fn test_cmdwin_state() {
        let mut state = CmdwinState::new();
        assert!(!state.is_active());

        state.open(CmdwinType::Ex, 1);
        assert!(state.is_active());
        assert_eq!(state.level, 1);

        state.close(result::ENTER);
        assert!(!state.is_active());
        assert_eq!(state.result, result::ENTER);
    }

    #[test]
    fn test_result_codes() {
        assert!(result::should_execute(result::ENTER));
        assert!(!result::should_execute(result::ESC));

        assert!(result::should_cancel(result::CTRL_C));
        assert!(result::should_cancel(result::ESC));
        assert!(!result::should_cancel(result::ENTER));
    }

    #[test]
    fn test_history_mapping() {
        assert_eq!(cmdwin_to_hist_type(CmdwinType::Ex), 0); // HIST_CMD
        assert_eq!(cmdwin_to_hist_type(CmdwinType::ForwardSearch), 1); // HIST_SEARCH
        assert_eq!(cmdwin_to_hist_type(CmdwinType::Expression), 2); // HIST_EXPR
    }

    #[test]
    fn test_open_error() {
        assert!(CmdwinOpenError::Ok.can_open());
        assert!(!CmdwinOpenError::AlreadyInCmdwin.can_open());
        assert!(!CmdwinOpenError::TextLocked.can_open());
    }

    #[test]
    fn test_can_open_cmdwin() {
        // Normal case - can open
        assert_eq!(can_open_cmdwin(false, false, 0), CmdwinOpenError::Ok);

        // Already in cmdwin
        assert_eq!(
            can_open_cmdwin(true, false, 0),
            CmdwinOpenError::AlreadyInCmdwin
        );

        // Text locked
        assert_eq!(can_open_cmdwin(false, true, 0), CmdwinOpenError::TextLocked);

        // Secret mode (password)
        assert_eq!(
            can_open_cmdwin(false, false, 1),
            CmdwinOpenError::SecretMode
        );
    }

    #[test]
    fn test_cmdwin_split_invalid() {
        // All valid
        assert!(!cmdwin_split_invalid(true, false, true, false));

        // Old curwin not valid
        assert!(cmdwin_split_invalid(false, false, true, false));

        // Curwin is old (didn't create new window)
        assert!(cmdwin_split_invalid(true, true, true, false));

        // Old curbuf not valid
        assert!(cmdwin_split_invalid(true, false, false, false));

        // Buffer changed
        assert!(cmdwin_split_invalid(true, false, true, true));
    }

    #[test]
    fn test_cmdwin_needs_tab_mapping() {
        const TAB: i32 = 9;
        const HIST_CMD: i32 = 0;
        const HIST_DEBUG: i32 = 4;
        const HIST_SEARCH: i32 = 1;

        // Tab wildchar with Ex history
        assert!(cmdwin_needs_tab_mapping(HIST_CMD, TAB));

        // Tab wildchar with Debug history
        assert!(cmdwin_needs_tab_mapping(HIST_DEBUG, TAB));

        // Tab wildchar with Search history - no mapping
        assert!(!cmdwin_needs_tab_mapping(HIST_SEARCH, TAB));

        // Non-tab wildchar - no mapping
        assert!(!cmdwin_needs_tab_mapping(HIST_CMD, b'%' as i32));
    }

    #[test]
    fn test_cmdwin_needs_vim_filetype() {
        const HIST_CMD: i32 = 0;
        const HIST_DEBUG: i32 = 4;
        const HIST_SEARCH: i32 = 1;

        assert!(cmdwin_needs_vim_filetype(HIST_CMD));
        assert!(cmdwin_needs_vim_filetype(HIST_DEBUG));
        assert!(!cmdwin_needs_vim_filetype(HIST_SEARCH));
    }

    #[test]
    fn test_cmdwin_cleanup_had_error() {
        // All valid - no error
        assert!(!cmdwin_cleanup_had_error(true, true, false));

        // Window invalid
        assert!(cmdwin_cleanup_had_error(false, true, false));

        // Buffer invalid
        assert!(cmdwin_cleanup_had_error(true, false, false));

        // Buffer changed
        assert!(cmdwin_cleanup_had_error(true, true, true));
    }
}
