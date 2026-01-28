//! Command line public API types
//!
//! This module provides types and utilities for the Vimscript command-line
//! functions like getcmdline(), setcmdline(), getcmdpos(), etc.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Command Line Type
// =============================================================================

/// Type of command line (from getcmdtype()).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CmdlineType {
    /// Not in command-line mode
    #[default]
    None = 0,
    /// Ex command (:)
    Ex = b':' as i32,
    /// Forward search (/)
    ForwardSearch = b'/' as i32,
    /// Backward search (?)
    BackwardSearch = b'?' as i32,
    /// Expression (=)
    Expression = b'=' as i32,
    /// Input function (@)
    Input = b'@' as i32,
    /// Debug mode (>)
    Debug = b'>' as i32,
    /// Substitute pattern (-)
    Substitute = b'-' as i32,
}

impl CmdlineType {
    /// Create from firstc character.
    #[must_use]
    pub const fn from_firstc(firstc: i32) -> Self {
        match firstc {
            c if c == b':' as i32 => Self::Ex,
            c if c == b'/' as i32 => Self::ForwardSearch,
            c if c == b'?' as i32 => Self::BackwardSearch,
            c if c == b'=' as i32 => Self::Expression,
            c if c == b'@' as i32 => Self::Input,
            c if c == b'>' as i32 => Self::Debug,
            c if c == b'-' as i32 => Self::Substitute,
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
            Self::Substitute => Some(b'-'),
        }
    }

    /// Check if this is a search type.
    #[must_use]
    pub const fn is_search(self) -> bool {
        matches!(self, Self::ForwardSearch | Self::BackwardSearch)
    }

    /// Get string representation for return value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Ex => ":",
            Self::ForwardSearch => "/",
            Self::BackwardSearch => "?",
            Self::Expression => "=",
            Self::Input => "@",
            Self::Debug => ">",
            Self::Substitute => "-",
        }
    }
}

// =============================================================================
// Command Line Window Type
// =============================================================================

/// Type of command-line window (from getcmdwintype()).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CmdwinType {
    /// Not in command-line window
    #[default]
    None = 0,
    /// Ex history (q:)
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
    /// Create from character.
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

    /// Get string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Ex => ":",
            Self::ForwardSearch => "/",
            Self::BackwardSearch => "?",
            Self::Expression => "=",
            Self::Input => "@",
            Self::Debug => ">",
        }
    }
}

// =============================================================================
// Screen Type
// =============================================================================

/// Screen type for command line (from getscreentype()).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenType {
    /// Normal screen
    #[default]
    Normal = 0,
    /// Not valid yet
    NotValid = 1,
    /// Scrolled
    Scrolled = 2,
}

// =============================================================================
// Position Info
// =============================================================================

/// Position information for command line.
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdlinePosition {
    /// Cursor position (0-based byte index).
    pub pos: i32,
    /// Length of command line in bytes.
    pub len: i32,
}

impl CmdlinePosition {
    /// Create a new position info.
    #[must_use]
    pub const fn new(pos: i32, len: i32) -> Self {
        Self { pos, len }
    }

    /// Check if cursor is at the start.
    #[must_use]
    pub const fn at_start(&self) -> bool {
        self.pos == 0
    }

    /// Check if cursor is at the end.
    #[must_use]
    pub const fn at_end(&self) -> bool {
        self.pos >= self.len
    }

    /// Get remaining length after cursor.
    #[must_use]
    pub const fn remaining(&self) -> i32 {
        if self.pos >= self.len {
            0
        } else {
            self.len - self.pos
        }
    }
}

// =============================================================================
// API Result
// =============================================================================

/// Result type for API operations.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApiResult {
    /// Operation succeeded
    #[default]
    Ok = 0,
    /// Invalid argument
    InvalidArg = 1,
    /// Not in command-line mode
    NotInCmdline = 2,
    /// Operation not allowed
    NotAllowed = 3,
    /// Buffer too small
    BufferTooSmall = 4,
}

impl ApiResult {
    /// Check if result is success.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if result is an error.
    #[must_use]
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }
}

// =============================================================================
// Completion Info
// =============================================================================

/// Completion information for getcmdcompltype().
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionType {
    /// No completion
    #[default]
    None = 0,
    /// Command names
    Command = 1,
    /// File names
    File = 2,
    /// Directory names
    Dir = 3,
    /// Buffer names
    Buffer = 4,
    /// Help topics
    Help = 5,
    /// Options
    Option = 6,
    /// Mappings
    Mapping = 7,
    /// User-defined
    User = 8,
    /// Custom function
    CustomFunc = 9,
    /// Custom list
    CustomList = 10,
}

// =============================================================================
// Position Clamping
// =============================================================================

/// Clamp a cursor position to be within valid range.
///
/// Position is clamped to [0, cmdlen].
#[must_use]
pub const fn clamp_cmdpos(pos: i32, cmdlen: i32) -> i32 {
    if pos < 0 {
        cmdlen
    } else if pos > cmdlen {
        cmdlen
    } else {
        pos
    }
}

/// Convert 1-based Vim position to 0-based internal position.
///
/// Vim uses 1-based positions, internal code uses 0-based.
#[must_use]
pub const fn vim_pos_to_internal(vim_pos: i32) -> i32 {
    if vim_pos <= 0 {
        0
    } else {
        vim_pos - 1
    }
}

/// Convert 0-based internal position to 1-based Vim position.
///
/// For getcmdpos() return value.
#[must_use]
pub const fn internal_pos_to_vim(internal_pos: i32) -> i32 {
    internal_pos + 1
}

// =============================================================================
// Validation Helpers
// =============================================================================

/// Check if cmdline pointer is valid for API operations.
#[must_use]
pub const fn is_valid_cmdline_ptr(ptr_is_null: bool) -> bool {
    !ptr_is_null
}

/// Calculate the new position after setcmdline.
///
/// If pos < 0 or pos > len, clamp to len.
#[must_use]
pub const fn calculate_new_cmdpos(requested_pos: i32, cmdlen: i32) -> i32 {
    clamp_cmdpos(requested_pos, cmdlen)
}

/// Check if setcmdpos argument is valid.
///
/// The position must be positive (Vim uses 1-based).
#[must_use]
pub const fn is_valid_setcmdpos_arg(pos: i32) -> bool {
    pos >= 1
}

// =============================================================================
// Return Value Helpers
// =============================================================================

/// Get return value for getcmdpos when not in cmdline.
#[must_use]
pub const fn getcmdpos_not_in_cmdline() -> i32 {
    0
}

/// Get return value for getcmdscreenpos when not in cmdline.
#[must_use]
pub const fn getcmdscreenpos_not_in_cmdline() -> i32 {
    0
}

/// Get return value for setcmdline/setcmdpos on failure.
#[must_use]
pub const fn setcmd_failure() -> i32 {
    1
}

/// Get return value for setcmdline/setcmdpos on success.
#[must_use]
pub const fn setcmd_success() -> i32 {
    0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Clamp cursor position (FFI).
#[no_mangle]
pub extern "C" fn rs_clamp_cmdpos(pos: c_int, cmdlen: c_int) -> c_int {
    clamp_cmdpos(pos, cmdlen)
}

/// Convert Vim position to internal (FFI).
#[no_mangle]
pub extern "C" fn rs_vim_pos_to_internal(vim_pos: c_int) -> c_int {
    vim_pos_to_internal(vim_pos)
}

/// Convert internal position to Vim (FFI).
#[no_mangle]
pub extern "C" fn rs_internal_pos_to_vim(internal_pos: c_int) -> c_int {
    internal_pos_to_vim(internal_pos)
}

/// Check if cmdline ptr is valid (FFI).
#[no_mangle]
pub extern "C" fn rs_is_valid_cmdline_ptr(ptr_is_null: c_int) -> c_int {
    c_int::from(is_valid_cmdline_ptr(ptr_is_null != 0))
}

/// Calculate new cmdpos for setcmdline (FFI).
#[no_mangle]
pub extern "C" fn rs_calculate_new_cmdpos(requested_pos: c_int, cmdlen: c_int) -> c_int {
    calculate_new_cmdpos(requested_pos, cmdlen)
}

/// Check if setcmdpos arg is valid (FFI).
#[no_mangle]
pub extern "C" fn rs_is_valid_setcmdpos_arg(pos: c_int) -> c_int {
    c_int::from(is_valid_setcmdpos_arg(pos))
}

/// Get cmdline type from firstc (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdline_type_from_firstc(firstc: c_int) -> c_int {
    CmdlineType::from_firstc(firstc) as c_int
}

/// Check if cmdline type is search (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdline_type_is_search(ctype: c_int) -> c_int {
    c_int::from(CmdlineType::from_firstc(ctype).is_search())
}

/// Get cmdwin type from char (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_type_from_char(c: c_int) -> c_int {
    CmdwinType::from_char(c) as c_int
}

/// Check if position is at end (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdline_pos_at_end(pos: c_int, len: c_int) -> c_int {
    let posinfo = CmdlinePosition::new(pos, len);
    c_int::from(posinfo.at_end())
}

/// Check if position is at start (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdline_pos_at_start(pos: c_int) -> c_int {
    c_int::from(pos == 0)
}

/// Check if API result is OK (FFI).
#[no_mangle]
pub extern "C" fn rs_api_result_is_ok(result: c_int) -> c_int {
    let r = match result {
        0 => ApiResult::Ok,
        2 => ApiResult::NotInCmdline,
        3 => ApiResult::NotAllowed,
        4 => ApiResult::BufferTooSmall,
        _ => ApiResult::InvalidArg,
    };
    c_int::from(r.is_ok())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdline_type() {
        assert_eq!(CmdlineType::from_firstc(i32::from(b':')), CmdlineType::Ex);
        assert_eq!(
            CmdlineType::from_firstc(i32::from(b'/')),
            CmdlineType::ForwardSearch
        );
        assert_eq!(
            CmdlineType::from_firstc(i32::from(b'?')),
            CmdlineType::BackwardSearch
        );
        assert_eq!(CmdlineType::from_firstc(0), CmdlineType::None);

        assert!(CmdlineType::ForwardSearch.is_search());
        assert!(CmdlineType::BackwardSearch.is_search());
        assert!(!CmdlineType::Ex.is_search());

        assert_eq!(CmdlineType::Ex.as_str(), ":");
        assert_eq!(CmdlineType::None.as_str(), "");
    }

    #[test]
    fn test_cmdwin_type() {
        assert_eq!(CmdwinType::from_char(i32::from(b':')), CmdwinType::Ex);
        assert_eq!(CmdwinType::from_char(0), CmdwinType::None);

        assert_eq!(CmdwinType::Ex.as_str(), ":");
        assert_eq!(CmdwinType::None.as_str(), "");
    }

    #[test]
    fn test_cmdline_position() {
        let pos = CmdlinePosition::new(0, 10);
        assert!(pos.at_start());
        assert!(!pos.at_end());
        assert_eq!(pos.remaining(), 10);

        let pos = CmdlinePosition::new(10, 10);
        assert!(!pos.at_start());
        assert!(pos.at_end());
        assert_eq!(pos.remaining(), 0);

        let pos = CmdlinePosition::new(5, 10);
        assert!(!pos.at_start());
        assert!(!pos.at_end());
        assert_eq!(pos.remaining(), 5);
    }

    #[test]
    fn test_api_result() {
        assert!(ApiResult::Ok.is_ok());
        assert!(!ApiResult::Ok.is_err());

        assert!(!ApiResult::InvalidArg.is_ok());
        assert!(ApiResult::InvalidArg.is_err());
    }

    #[test]
    fn test_clamp_cmdpos() {
        // Normal case
        assert_eq!(clamp_cmdpos(5, 10), 5);

        // At boundary
        assert_eq!(clamp_cmdpos(10, 10), 10);
        assert_eq!(clamp_cmdpos(0, 10), 0);

        // Out of bounds
        assert_eq!(clamp_cmdpos(15, 10), 10);
        assert_eq!(clamp_cmdpos(-1, 10), 10);
        assert_eq!(clamp_cmdpos(-5, 10), 10);
    }

    #[test]
    fn test_vim_pos_conversion() {
        // Vim uses 1-based, internal uses 0-based
        assert_eq!(vim_pos_to_internal(1), 0);
        assert_eq!(vim_pos_to_internal(5), 4);
        assert_eq!(vim_pos_to_internal(0), 0);

        assert_eq!(internal_pos_to_vim(0), 1);
        assert_eq!(internal_pos_to_vim(4), 5);
    }

    #[test]
    fn test_is_valid_cmdline_ptr() {
        assert!(is_valid_cmdline_ptr(false));
        assert!(!is_valid_cmdline_ptr(true));
    }

    #[test]
    fn test_calculate_new_cmdpos() {
        assert_eq!(calculate_new_cmdpos(5, 10), 5);
        assert_eq!(calculate_new_cmdpos(15, 10), 10);
        assert_eq!(calculate_new_cmdpos(-1, 10), 10);
    }

    #[test]
    fn test_is_valid_setcmdpos_arg() {
        assert!(is_valid_setcmdpos_arg(1));
        assert!(is_valid_setcmdpos_arg(10));
        assert!(!is_valid_setcmdpos_arg(0));
        assert!(!is_valid_setcmdpos_arg(-1));
    }

    #[test]
    fn test_return_values() {
        assert_eq!(getcmdpos_not_in_cmdline(), 0);
        assert_eq!(getcmdscreenpos_not_in_cmdline(), 0);
        assert_eq!(setcmd_failure(), 1);
        assert_eq!(setcmd_success(), 0);
    }
}
