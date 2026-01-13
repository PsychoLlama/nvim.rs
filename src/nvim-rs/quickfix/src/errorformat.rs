//! Error format parsing for quickfix
//!
//! This module provides error format pattern matching and parsing for
//! processing compiler output and converting it to quickfix entries.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Error Format Types
// =============================================================================

/// Error format match type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfmMatchType {
    /// No match found
    NoMatch = 0,
    /// Message text
    Message = 1,
    /// File name
    FileName = 2,
    /// Line number
    LineNumber = 3,
    /// Column number
    ColumnNumber = 4,
    /// Error type (error, warning, info, note)
    ErrorType = 5,
    /// Error number
    ErrorNumber = 6,
    /// Module name
    Module = 7,
    /// End line number
    EndLine = 8,
    /// End column number
    EndColumn = 9,
    /// Virtual column flag
    VirtualColumn = 10,
    /// Pattern for searching
    Pattern = 11,
}

/// Error type classification
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfErrorType {
    /// No type / unknown
    Unknown = 0,
    /// Error
    Error = 1,
    /// Warning
    Warning = 2,
    /// Info/information
    Info = 3,
    /// Note
    Note = 4,
}

impl QfErrorType {
    /// Create from character type marker
    pub const fn from_char(c: u8) -> Self {
        match c {
            b'E' | b'e' => Self::Error,
            b'W' | b'w' => Self::Warning,
            b'I' | b'i' => Self::Info,
            b'N' | b'n' => Self::Note,
            _ => Self::Unknown,
        }
    }

    /// Convert to character type marker
    pub const fn to_char(self) -> u8 {
        match self {
            Self::Error => b'E',
            Self::Warning => b'W',
            Self::Info => b'I',
            Self::Note => b'N',
            Self::Unknown => b' ',
        }
    }

    /// Check if this is an error
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    /// Check if this is a warning
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }
}

// =============================================================================
// Error Format Flags
// =============================================================================

/// Error format flags for pattern matching
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EfmFlags {
    flags: u16,
}

/// Flag constants
pub const EFM_MULTILINE: u16 = 0x001;
pub const EFM_CONTINUATION: u16 = 0x002;
pub const EFM_GLOBAL: u16 = 0x004;
pub const EFM_FILE_ALLWAYS: u16 = 0x008;
pub const EFM_VALID: u16 = 0x010;
pub const EFM_IGNORE_CASE: u16 = 0x020;
pub const EFM_END_PATTERN: u16 = 0x040;
pub const EFM_INCLUDE: u16 = 0x080;
pub const EFM_DEFINE: u16 = 0x100;

impl EfmFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u16) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u16 {
        self.flags
    }

    /// Check if multiline flag is set
    pub const fn is_multiline(self) -> bool {
        (self.flags & EFM_MULTILINE) != 0
    }

    /// Check if continuation flag is set
    pub const fn is_continuation(self) -> bool {
        (self.flags & EFM_CONTINUATION) != 0
    }

    /// Check if global flag is set
    pub const fn is_global(self) -> bool {
        (self.flags & EFM_GLOBAL) != 0
    }

    /// Check if file_allways flag is set
    pub const fn file_always(self) -> bool {
        (self.flags & EFM_FILE_ALLWAYS) != 0
    }

    /// Check if valid flag is set
    pub const fn is_valid(self) -> bool {
        (self.flags & EFM_VALID) != 0
    }

    /// Check if ignore_case flag is set
    pub const fn ignore_case(self) -> bool {
        (self.flags & EFM_IGNORE_CASE) != 0
    }

    /// Check if end_pattern flag is set
    pub const fn is_end_pattern(self) -> bool {
        (self.flags & EFM_END_PATTERN) != 0
    }

    /// Check if include flag is set
    pub const fn is_include(self) -> bool {
        (self.flags & EFM_INCLUDE) != 0
    }

    /// Check if define flag is set
    pub const fn is_define(self) -> bool {
        (self.flags & EFM_DEFINE) != 0
    }

    /// Set a flag
    pub fn set(&mut self, flag: u16) {
        self.flags |= flag;
    }

    /// Clear a flag
    pub fn clear(&mut self, flag: u16) {
        self.flags &= !flag;
    }
}

// =============================================================================
// Parse Result
// =============================================================================

/// Result of parsing a single line with errorformat
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EfmParseResult {
    /// Whether the line matched a pattern
    pub matched: bool,
    /// File number if matched
    pub fnum: c_int,
    /// Line number if matched
    pub lnum: i32,
    /// End line number
    pub end_lnum: i32,
    /// Column number
    pub col: c_int,
    /// End column number
    pub end_col: c_int,
    /// Virtual column flag
    pub vcol: bool,
    /// Error type
    pub error_type: QfErrorType,
    /// Error number
    pub nr: c_int,
    /// Whether entry is valid
    pub valid: bool,
}

impl Default for EfmParseResult {
    fn default() -> Self {
        Self {
            matched: false,
            fnum: 0,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            vcol: false,
            error_type: QfErrorType::Unknown,
            nr: 0,
            valid: false,
        }
    }
}

impl EfmParseResult {
    /// Create an empty (no match) result
    pub const fn no_match() -> Self {
        Self {
            matched: false,
            fnum: 0,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            vcol: false,
            error_type: QfErrorType::Unknown,
            nr: 0,
            valid: false,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get error type from character
#[no_mangle]
pub extern "C" fn rs_qf_error_type_from_char(c: u8) -> QfErrorType {
    QfErrorType::from_char(c)
}

/// FFI export: Get character from error type
#[no_mangle]
pub extern "C" fn rs_qf_error_type_to_char(t: QfErrorType) -> u8 {
    t.to_char()
}

/// FFI export: Check if error type is error
#[no_mangle]
pub extern "C" fn rs_qf_error_type_is_error(t: QfErrorType) -> c_int {
    c_int::from(t.is_error())
}

/// FFI export: Check if error type is warning
#[no_mangle]
pub extern "C" fn rs_qf_error_type_is_warning(t: QfErrorType) -> c_int {
    c_int::from(t.is_warning())
}

/// FFI export: Check efm flags multiline
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_multiline(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_multiline())
}

/// FFI export: Check efm flags continuation
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_continuation(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_continuation())
}

/// FFI export: Check efm flags valid
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_valid(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_valid())
}

/// FFI export: Check efm flags global
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_global(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_global())
}

/// FFI export: Check efm flags file_always
#[no_mangle]
pub extern "C" fn rs_efm_flags_file_always(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).file_always())
}

/// FFI export: Check efm flags ignore_case
#[no_mangle]
pub extern "C" fn rs_efm_flags_ignore_case(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).ignore_case())
}

/// FFI export: Check efm flags end_pattern
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_end_pattern(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_end_pattern())
}

/// FFI export: Check efm flags include
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_include(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_include())
}

/// FFI export: Check efm flags define
#[no_mangle]
pub extern "C" fn rs_efm_flags_is_define(flags: u16) -> c_int {
    c_int::from(EfmFlags::from_raw(flags).is_define())
}

/// FFI export: Get EFM_MULTILINE constant
#[no_mangle]
pub extern "C" fn rs_efm_multiline() -> c_int {
    c_int::from(EFM_MULTILINE)
}

/// FFI export: Get EFM_CONTINUATION constant
#[no_mangle]
pub extern "C" fn rs_efm_continuation() -> c_int {
    c_int::from(EFM_CONTINUATION)
}

/// FFI export: Get EFM_GLOBAL constant
#[no_mangle]
pub extern "C" fn rs_efm_global() -> c_int {
    c_int::from(EFM_GLOBAL)
}

/// FFI export: Get EFM_VALID constant
#[no_mangle]
pub extern "C" fn rs_efm_valid() -> c_int {
    c_int::from(EFM_VALID)
}

/// FFI export: Get EfmMatchType::Message value
#[no_mangle]
pub extern "C" fn rs_efm_match_message() -> c_int {
    EfmMatchType::Message as c_int
}

/// FFI export: Get EfmMatchType::FileName value
#[no_mangle]
pub extern "C" fn rs_efm_match_filename() -> c_int {
    EfmMatchType::FileName as c_int
}

/// FFI export: Get EfmMatchType::LineNumber value
#[no_mangle]
pub extern "C" fn rs_efm_match_linenumber() -> c_int {
    EfmMatchType::LineNumber as c_int
}

/// FFI export: Get EfmMatchType::ColumnNumber value
#[no_mangle]
pub extern "C" fn rs_efm_match_columnnumber() -> c_int {
    EfmMatchType::ColumnNumber as c_int
}

/// FFI export: Get EfmMatchType::ErrorType value
#[no_mangle]
pub extern "C" fn rs_efm_match_errortype() -> c_int {
    EfmMatchType::ErrorType as c_int
}

/// FFI export: Get EfmMatchType::Pattern value
#[no_mangle]
pub extern "C" fn rs_efm_match_pattern() -> c_int {
    EfmMatchType::Pattern as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_from_char() {
        assert_eq!(QfErrorType::from_char(b'E'), QfErrorType::Error);
        assert_eq!(QfErrorType::from_char(b'e'), QfErrorType::Error);
        assert_eq!(QfErrorType::from_char(b'W'), QfErrorType::Warning);
        assert_eq!(QfErrorType::from_char(b'I'), QfErrorType::Info);
        assert_eq!(QfErrorType::from_char(b'N'), QfErrorType::Note);
        assert_eq!(QfErrorType::from_char(b'X'), QfErrorType::Unknown);
    }

    #[test]
    fn test_error_type_to_char() {
        assert_eq!(QfErrorType::Error.to_char(), b'E');
        assert_eq!(QfErrorType::Warning.to_char(), b'W');
        assert_eq!(QfErrorType::Info.to_char(), b'I');
        assert_eq!(QfErrorType::Note.to_char(), b'N');
        assert_eq!(QfErrorType::Unknown.to_char(), b' ');
    }

    #[test]
    fn test_error_type_checks() {
        assert!(QfErrorType::Error.is_error());
        assert!(!QfErrorType::Warning.is_error());
        assert!(QfErrorType::Warning.is_warning());
        assert!(!QfErrorType::Error.is_warning());
    }

    #[test]
    fn test_efm_flags() {
        let mut flags = EfmFlags::none();
        assert!(!flags.is_multiline());

        flags.set(EFM_MULTILINE);
        assert!(flags.is_multiline());

        flags.set(EFM_CONTINUATION);
        assert!(flags.is_continuation());
        assert!(flags.is_multiline());

        flags.clear(EFM_MULTILINE);
        assert!(!flags.is_multiline());
        assert!(flags.is_continuation());
    }

    #[test]
    fn test_efm_parse_result_default() {
        let result = EfmParseResult::default();
        assert!(!result.matched);
        assert!(!result.valid);
        assert_eq!(result.error_type, QfErrorType::Unknown);
    }

    #[test]
    fn test_efm_match_type_values() {
        assert_eq!(EfmMatchType::NoMatch as c_int, 0);
        assert_eq!(EfmMatchType::Message as c_int, 1);
        assert_eq!(EfmMatchType::FileName as c_int, 2);
        assert_eq!(EfmMatchType::LineNumber as c_int, 3);
    }
}
