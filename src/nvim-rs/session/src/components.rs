//! Session component handlers
//!
//! This module provides utilities for writing specific session components
//! like buffers, windows, tabs, and options.

use std::ffi::{c_char, c_int};

use crate::SessionFlags;

// =============================================================================
// Buffer Session Types
// =============================================================================

/// Buffer type classification for session saving
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferType {
    /// Normal file buffer
    Normal = 0,
    /// Help buffer
    Help = 1,
    /// Quickfix buffer
    Quickfix = 2,
    /// Terminal buffer
    Terminal = 3,
    /// No file name buffer (scratch)
    NoFile = 4,
    /// Prompt buffer
    Prompt = 5,
    /// Popup buffer
    Popup = 6,
}

impl BufferType {
    /// Create from C integer
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            1 => Self::Help,
            2 => Self::Quickfix,
            3 => Self::Terminal,
            4 => Self::NoFile,
            5 => Self::Prompt,
            6 => Self::Popup,
            _ => Self::Normal,
        }
    }
}

/// Check if buffer type should be saved based on session flags
#[no_mangle]
pub extern "C" fn rs_session_should_save_buftype(buftype: c_int, flags: u32) -> bool {
    let bt = BufferType::from_c(buftype);
    let f = SessionFlags::from_bits_truncate(flags);

    match bt {
        BufferType::Normal => true,
        BufferType::Help => f.contains(SessionFlags::HELP),
        BufferType::Terminal => f.contains(SessionFlags::TERMINAL),
        BufferType::NoFile => f.contains(SessionFlags::BLANK),
        // These are never saved in sessions
        BufferType::Quickfix | BufferType::Prompt | BufferType::Popup => false,
    }
}

// =============================================================================
// Window Session Predicates
// =============================================================================

/// Check if a window should be included in session based on its buffer
#[no_mangle]
pub extern "C" fn rs_session_should_save_win(
    is_help: bool,
    is_terminal: bool,
    is_blank: bool,
    has_file: bool,
    flags: u32,
) -> bool {
    let f = SessionFlags::from_bits_truncate(flags);

    // Help windows need HELP flag
    if is_help && !f.contains(SessionFlags::HELP) {
        return false;
    }

    // Terminal windows need TERMINAL flag
    if is_terminal && !f.contains(SessionFlags::TERMINAL) {
        return false;
    }

    // Blank windows need BLANK flag
    if is_blank && !has_file && !f.contains(SessionFlags::BLANK) {
        return false;
    }

    true
}

/// Check if a frame should be included in session
/// A frame is included if any of its windows should be saved
#[no_mangle]
pub extern "C" fn rs_session_frame_has_saveable_win(
    has_help_only: bool,
    has_terminal_only: bool,
    has_blank_only: bool,
    flags: u32,
) -> bool {
    let f = SessionFlags::from_bits_truncate(flags);

    // If all windows are help windows and no HELP flag
    if has_help_only && !f.contains(SessionFlags::HELP) {
        return false;
    }

    // If all windows are terminals and no TERMINAL flag
    if has_terminal_only && !f.contains(SessionFlags::TERMINAL) {
        return false;
    }

    // If all windows are blank and no BLANK flag
    if has_blank_only && !f.contains(SessionFlags::BLANK) {
        return false;
    }

    true
}

// =============================================================================
// Tab Page Session Types
// =============================================================================

/// Information about a tabpage for session saving
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TabpageInfo {
    /// Number of windows
    pub win_count: c_int,
    /// Has local directory
    pub has_localdir: bool,
    /// Is current tab
    pub is_current: bool,
}

/// Create a new tabpage info struct
#[no_mangle]
pub extern "C" fn rs_session_tabpage_info_new(
    win_count: c_int,
    has_localdir: bool,
    is_current: bool,
) -> TabpageInfo {
    TabpageInfo {
        win_count,
        has_localdir,
        is_current,
    }
}

// =============================================================================
// Alternate Buffer Handling
// =============================================================================

/// Check if an alternate buffer should be saved in session
#[no_mangle]
pub extern "C" fn rs_session_should_save_altbuf(
    has_file: bool,
    is_help: bool,
    is_terminal: bool,
    flags: u32,
) -> bool {
    // Alternate buffer needs to have a file
    if !has_file {
        return false;
    }

    let f = SessionFlags::from_bits_truncate(flags);

    // Skip help buffers unless HELP is set
    if is_help && !f.contains(SessionFlags::HELP) {
        return false;
    }

    // Skip terminal buffers unless TERMINAL is set
    if is_terminal && !f.contains(SessionFlags::TERMINAL) {
        return false;
    }

    true
}

// =============================================================================
// Global Variable Session Helpers
// =============================================================================

/// VimL types that can be saved in sessions
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionVarType {
    /// Cannot be saved
    Unsupported = 0,
    /// Number
    Number = 1,
    /// String
    String = 2,
    /// Float
    Float = 3,
    /// Boolean (v:true/v:false)
    Bool = 4,
}

impl SessionVarType {
    /// Create from typval type constant
    #[must_use]
    pub fn from_typval_type(tv_type: c_int) -> Self {
        // VAR_NUMBER = 0, VAR_STRING = 1, VAR_FLOAT = 5, VAR_BOOL = 6
        match tv_type {
            0 => Self::Number,
            1 => Self::String,
            5 => Self::Float,
            6 => Self::Bool,
            _ => Self::Unsupported,
        }
    }
}

/// Check if a typval type can be saved in session
#[no_mangle]
pub extern "C" fn rs_session_can_save_var_type(tv_type: c_int) -> bool {
    SessionVarType::from_typval_type(tv_type) != SessionVarType::Unsupported
}

/// Get the session-saveable variable type
#[no_mangle]
pub extern "C" fn rs_session_var_type(tv_type: c_int) -> c_int {
    SessionVarType::from_typval_type(tv_type) as c_int
}

// =============================================================================
// Fold Session Helpers
// =============================================================================

/// Check if fold method allows manual fold saving
#[no_mangle]
pub extern "C" fn rs_session_fold_method_saves_manual(method: c_int) -> bool {
    // Only manual and marker methods can have manually created folds saved
    matches!(method, 0 | 3) // Manual or Marker
}

/// Check if folds should be saved for this method
#[no_mangle]
pub extern "C" fn rs_session_should_save_folds(_method: c_int, flags: u32) -> bool {
    let f = SessionFlags::from_bits_truncate(flags);
    if !f.contains(SessionFlags::FOLDS) {
        return false;
    }

    // For manual and marker, we save the actual folds
    // For other methods, we just save the foldmethod option
    true
}

// =============================================================================
// Options Session Helpers
// =============================================================================

/// Check if options should be saved based on scope and flags
#[no_mangle]
pub extern "C" fn rs_session_should_save_option(scope: c_int, flags: u32) -> bool {
    let f = SessionFlags::from_bits_truncate(flags);

    match scope {
        0 => f.contains(SessionFlags::OPTIONS), // Global
        1 | 2 => {
            // Buffer/Window local
            f.contains(SessionFlags::LOCALOPTIONS) || f.contains(SessionFlags::OPTIONS)
        }
        _ => false,
    }
}

/// Check if 'runtimepath' and 'packpath' should be skipped
#[no_mangle]
pub extern "C" fn rs_session_should_skip_rtp(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::SKIPRTP)
}

// =============================================================================
// View Component Types
// =============================================================================

/// View components (subset of session for single buffer/window)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewFlags {
    bits: u32,
}

impl ViewFlags {
    /// Cursor position
    pub const CURSOR: u32 = 0x01;
    /// Folds
    pub const FOLDS: u32 = 0x02;
    /// Local options
    pub const OPTIONS: u32 = 0x04;
    /// Cursor column (current directory)
    pub const CURDIR: u32 = 0x08;
}

/// Create view flags from raw bits
#[no_mangle]
pub extern "C" fn rs_view_flags_from_bits(bits: u32) -> ViewFlags {
    ViewFlags { bits }
}

/// Check if view should save cursor
#[no_mangle]
pub extern "C" fn rs_view_has_cursor(flags: ViewFlags) -> bool {
    (flags.bits & ViewFlags::CURSOR) != 0
}

/// Check if view should save folds
#[no_mangle]
pub extern "C" fn rs_view_has_folds(flags: ViewFlags) -> bool {
    (flags.bits & ViewFlags::FOLDS) != 0
}

/// Check if view should save options
#[no_mangle]
pub extern "C" fn rs_view_has_options(flags: ViewFlags) -> bool {
    (flags.bits & ViewFlags::OPTIONS) != 0
}

/// Check if view should save current directory
#[no_mangle]
pub extern "C" fn rs_view_has_curdir(flags: ViewFlags) -> bool {
    (flags.bits & ViewFlags::CURDIR) != 0
}

// =============================================================================
// Session File Type Detection
// =============================================================================

/// Session file type
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionFileType {
    /// Full session file (:mksession)
    Session = 0,
    /// View file (:mkview)
    View = 1,
    /// Vimrc file (:mkvimrc)
    Vimrc = 2,
    /// Exrc file (:mkexrc)
    Exrc = 3,
}

/// Determine session file type from command index
#[no_mangle]
pub extern "C" fn rs_session_file_type(cmd_idx: c_int) -> c_int {
    // CMD_mksession = 0, CMD_mkview = 1, CMD_mkvimrc = 2, CMD_mkexrc = 3
    match cmd_idx {
        1 => SessionFileType::View as c_int,
        2 => SessionFileType::Vimrc as c_int,
        3 => SessionFileType::Exrc as c_int,
        // 0 and anything else defaults to Session
        _ => SessionFileType::Session as c_int,
    }
}

/// Check if file type is a session (not view/vimrc/exrc)
#[no_mangle]
pub extern "C" fn rs_session_is_session_type(file_type: c_int) -> bool {
    file_type == SessionFileType::Session as c_int
}

/// Check if file type is a view
#[no_mangle]
pub extern "C" fn rs_session_is_view_type(file_type: c_int) -> bool {
    file_type == SessionFileType::View as c_int
}

/// Get default file extension for session type
#[no_mangle]
pub extern "C" fn rs_session_file_extension(file_type: c_int) -> *const c_char {
    static VIM: &[u8] = b".vim\0";
    static EMPTY: &[u8] = b"\0";

    let ext = match file_type {
        0..=3 => VIM,
        _ => EMPTY,
    };
    ext.as_ptr().cast::<c_char>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_type() {
        assert_eq!(BufferType::from_c(0), BufferType::Normal);
        assert_eq!(BufferType::from_c(1), BufferType::Help);
        assert_eq!(BufferType::from_c(3), BufferType::Terminal);
    }

    #[test]
    fn test_should_save_buftype() {
        // Normal buffers always saved
        assert!(rs_session_should_save_buftype(0, 0));

        // Help needs HELP flag
        assert!(!rs_session_should_save_buftype(1, 0));
        assert!(rs_session_should_save_buftype(1, SessionFlags::HELP.bits()));

        // Terminal needs TERMINAL flag
        assert!(!rs_session_should_save_buftype(3, 0));
        assert!(rs_session_should_save_buftype(
            3,
            SessionFlags::TERMINAL.bits()
        ));
    }

    #[test]
    fn test_var_type() {
        assert!(rs_session_can_save_var_type(0)); // Number
        assert!(rs_session_can_save_var_type(1)); // String
        assert!(rs_session_can_save_var_type(5)); // Float
        assert!(rs_session_can_save_var_type(6)); // Bool
        assert!(!rs_session_can_save_var_type(2)); // List - not supported
    }

    #[test]
    fn test_view_flags() {
        let flags = rs_view_flags_from_bits(ViewFlags::CURSOR | ViewFlags::FOLDS);
        assert!(rs_view_has_cursor(flags));
        assert!(rs_view_has_folds(flags));
        assert!(!rs_view_has_options(flags));
    }

    #[test]
    fn test_session_file_type() {
        assert!(rs_session_is_session_type(0));
        assert!(rs_session_is_view_type(1));
        assert!(!rs_session_is_session_type(1));
    }
}
