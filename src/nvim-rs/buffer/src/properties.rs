//! Buffer property accessors and modifiers
//!
//! This module provides comprehensive access to buffer properties through
//! a unified interface, abstracting the opaque handle pattern.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(dead_code)]

use std::ffi::c_int;

use crate::{buf_struct::buf_ref, BufHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // State accessors (complex - not pure field reads)
    fn nvim_buf_get_changedtick(buf: BufHandle) -> c_int;
}

// =============================================================================
// Buffer Type Classification
// =============================================================================

/// Buffer type enumeration based on 'buftype' option
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferType {
    /// Normal buffer (empty buftype)
    #[default]
    Normal = 0,
    /// Help buffer
    Help = 1,
    /// Quickfix buffer
    Quickfix = 2,
    /// Terminal buffer
    Terminal = 3,
    /// Prompt buffer
    Prompt = 4,
    /// Nofile buffer
    Nofile = 5,
    /// Nowrite buffer
    Nowrite = 6,
    /// Acwrite buffer
    Acwrite = 7,
}

impl BufferType {
    /// Check if this buffer type should not be written
    #[must_use]
    pub const fn is_nowrite(self) -> bool {
        matches!(
            self,
            Self::Nofile | Self::Nowrite | Self::Terminal | Self::Prompt
        )
    }

    /// Check if this buffer type should not be read from file
    #[must_use]
    pub const fn is_noread(self) -> bool {
        matches!(
            self,
            Self::Nofile | Self::Quickfix | Self::Terminal | Self::Prompt
        )
    }

    /// Check if this buffer name might not be a file path
    #[must_use]
    pub const fn is_nofilename(self) -> bool {
        matches!(
            self,
            Self::Nofile | Self::Acwrite | Self::Terminal | Self::Prompt
        )
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Help,
            2 => Self::Quickfix,
            3 => Self::Terminal,
            4 => Self::Prompt,
            5 => Self::Nofile,
            6 => Self::Nowrite,
            7 => Self::Acwrite,
            _ => Self::Normal,
        }
    }
}

/// Get the buffer type classification.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_buffer_type(buf: BufHandle) -> BufferType {
    if buf.is_null() {
        return BufferType::Normal;
    }

    // Check help buffer first
    let b = buf_ref(buf);
    if b.b_help != 0 {
        return BufferType::Help;
    }

    // Check terminal
    if !b.terminal.is_null() {
        return BufferType::Terminal;
    }

    let bt0 = b.buftype_char0();
    match bt0 {
        b'q' => BufferType::Quickfix,
        b't' => BufferType::Terminal,
        b'p' => BufferType::Prompt,
        b'a' => BufferType::Acwrite,
        b'n' => {
            // Check second char for nofile vs nowrite
            let bt2 = b.buftype_char2();
            if bt2 == b'f' {
                BufferType::Nofile
            } else {
                BufferType::Nowrite
            }
        }
        _ => BufferType::Normal,
    }
}

// =============================================================================
// Buffer Hidden Action
// =============================================================================

/// Action to take when buffer becomes hidden ('bufhidden' option)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HiddenAction {
    /// Use global hidden option
    #[default]
    Default = 0,
    /// Hide the buffer
    Hide = 1,
    /// Unload the buffer
    Unload = 2,
    /// Delete the buffer
    Delete = 3,
    /// Wipe the buffer
    Wipe = 4,
}

impl HiddenAction {
    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Hide,
            2 => Self::Unload,
            3 => Self::Delete,
            4 => Self::Wipe,
            _ => Self::Default,
        }
    }
}

/// Get the hidden action for a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_hidden_action(buf: BufHandle) -> HiddenAction {
    if buf.is_null() {
        return HiddenAction::Default;
    }

    let bh = buf_ref(buf).bufhidden_char0();
    match bh {
        b'h' => HiddenAction::Hide,
        b'u' => HiddenAction::Unload,
        b'd' => HiddenAction::Delete,
        b'w' => HiddenAction::Wipe,
        _ => HiddenAction::Default,
    }
}

// =============================================================================
// File Format
// =============================================================================

/// End-of-line format
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileFormat {
    /// Unix format (LF)
    #[default]
    Unix = 0,
    /// DOS format (CRLF)
    Dos = 1,
    /// Mac format (CR)
    Mac = 2,
}

impl FileFormat {
    /// Get the line ending string
    #[must_use]
    pub const fn line_ending(self) -> &'static str {
        match self {
            Self::Unix => "\n",
            Self::Dos => "\r\n",
            Self::Mac => "\r",
        }
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Dos,
            2 => Self::Mac,
            _ => Self::Unix,
        }
    }
}

/// Get the file format for a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_file_format(buf: BufHandle) -> FileFormat {
    if buf.is_null() {
        return FileFormat::Unix;
    }

    // Binary mode always uses Unix
    let b = buf_ref(buf);
    if b.b_p_bin != 0 {
        return FileFormat::Unix;
    }

    let ff = b.fileformat_char0();
    match ff {
        b'd' => FileFormat::Dos,
        b'm' => FileFormat::Mac,
        _ => FileFormat::Unix,
    }
}

// =============================================================================
// Comprehensive Buffer Properties
// =============================================================================

/// Comprehensive buffer properties structure
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferProperties {
    /// File number (unique identifier)
    pub fnum: c_int,
    /// Change tick counter
    pub changedtick: c_int,
    /// Number of lines
    pub line_count: c_int,
    /// Number of windows displaying this buffer
    pub nwindows: c_int,
    /// Buffer type classification
    pub buftype: BufferType,
    /// Hidden action
    pub bufhidden: HiddenAction,
    /// File format
    pub fileformat: FileFormat,
    /// Whether buffer has a filename
    pub has_filename: bool,
    /// Whether buffer is modified
    pub is_modified: bool,
    /// Whether buffer is read-only
    pub is_readonly: bool,
    /// Whether buffer is modifiable
    pub is_modifiable: bool,
    /// Whether buffer is listed
    pub is_listed: bool,
    /// Whether buffer is help
    pub is_help: bool,
    /// Whether buffer is terminal
    pub is_terminal: bool,
    /// Whether buffer is in binary mode
    pub is_binary: bool,
    /// Whether buffer is locked
    pub is_locked: bool,
    /// Whether memory is loaded
    pub is_loaded: bool,
}

/// Get comprehensive properties for a buffer.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn get_buffer_properties(buf: BufHandle) -> BufferProperties {
    if buf.is_null() {
        return BufferProperties::default();
    }

    let b = buf_ref(buf);
    BufferProperties {
        fnum: b.handle,
        changedtick: nvim_buf_get_changedtick(buf),
        line_count: b.ml_line_count,
        nwindows: b.b_nwindows,
        buftype: get_buffer_type(buf),
        bufhidden: get_hidden_action(buf),
        fileformat: get_file_format(buf),
        has_filename: !b.b_ffname.is_null(),
        is_modified: b.b_changed != 0,
        is_readonly: b.b_p_ro != 0,
        is_modifiable: b.b_p_ma != 0,
        is_listed: b.b_p_bl != 0,
        is_help: b.b_help != 0,
        is_terminal: !b.terminal.is_null(),
        is_binary: b.b_p_bin != 0,
        is_locked: b.b_locked > 0,
        is_loaded: !b.ml_mfp_is_null(),
    }
}

// =============================================================================
// Individual Property Accessors
// =============================================================================

/// Get buffer fnum.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn get_fnum(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    buf_ref(buf).handle
}

/// Get buffer changedtick.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn get_changedtick(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_changedtick(buf)
}

/// Get buffer line count.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn get_line_count(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    buf_ref(buf).ml_line_count
}

/// Check if buffer is readonly.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn is_readonly(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf_ref(buf).b_p_ro != 0
}

/// Check if buffer is modifiable.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn is_modifiable(buf: BufHandle) -> bool {
    if buf.is_null() {
        return true; // Default is modifiable
    }
    buf_ref(buf).b_p_ma != 0
}

/// Check if buffer is listed.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn is_listed(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf_ref(buf).b_p_bl != 0
}

/// Check if buffer is in binary mode.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn is_binary(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    buf_ref(buf).b_p_bin != 0
}

/// Check if buffer memory is loaded.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn is_loaded(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    !buf_ref(buf).ml_mfp_is_null()
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get buffer type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_type(buf: BufHandle) -> c_int {
    get_buffer_type(buf).to_raw()
}

/// Get buffer hidden action.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_hidden_action(buf: BufHandle) -> c_int {
    get_hidden_action(buf).to_raw()
}

/// Get buffer file format.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_file_format(buf: BufHandle) -> c_int {
    get_file_format(buf).to_raw()
}

/// Get buffer fnum.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_fnum(buf: BufHandle) -> c_int {
    get_fnum(buf)
}

/// Check if buffer is readonly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_readonly(buf: BufHandle) -> c_int {
    c_int::from(is_readonly(buf))
}

/// Check if buffer is modifiable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_modifiable(buf: BufHandle) -> c_int {
    c_int::from(is_modifiable(buf))
}

/// Check if buffer is listed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_listed(buf: BufHandle) -> c_int {
    c_int::from(is_listed(buf))
}

/// Check if buffer is binary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_binary(buf: BufHandle) -> c_int {
    c_int::from(is_binary(buf))
}

/// Check if buffer is loaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_is_loaded(buf: BufHandle) -> c_int {
    c_int::from(is_loaded(buf))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_type_classification() {
        assert!(BufferType::Nofile.is_nowrite());
        assert!(BufferType::Nowrite.is_nowrite());
        assert!(BufferType::Terminal.is_nowrite());
        assert!(BufferType::Prompt.is_nowrite());
        assert!(!BufferType::Normal.is_nowrite());
        assert!(!BufferType::Help.is_nowrite());
    }

    #[test]
    fn test_buffer_type_noread() {
        assert!(BufferType::Nofile.is_noread());
        assert!(BufferType::Quickfix.is_noread());
        assert!(BufferType::Terminal.is_noread());
        assert!(BufferType::Prompt.is_noread());
        assert!(!BufferType::Normal.is_noread());
        assert!(!BufferType::Nowrite.is_noread());
    }

    #[test]
    fn test_buffer_type_nofilename() {
        assert!(BufferType::Nofile.is_nofilename());
        assert!(BufferType::Acwrite.is_nofilename());
        assert!(BufferType::Terminal.is_nofilename());
        assert!(BufferType::Prompt.is_nofilename());
        assert!(!BufferType::Normal.is_nofilename());
        assert!(!BufferType::Quickfix.is_nofilename());
    }

    #[test]
    fn test_file_format_endings() {
        assert_eq!(FileFormat::Unix.line_ending(), "\n");
        assert_eq!(FileFormat::Dos.line_ending(), "\r\n");
        assert_eq!(FileFormat::Mac.line_ending(), "\r");
    }

    #[test]
    fn test_buffer_type_roundtrip() {
        for i in 0..8 {
            let bt = BufferType::from_raw(i);
            assert_eq!(bt.to_raw(), i);
        }
    }

    #[test]
    fn test_hidden_action_roundtrip() {
        for i in 0..5 {
            let ha = HiddenAction::from_raw(i);
            assert_eq!(ha.to_raw(), i);
        }
    }

    #[test]
    fn test_file_format_roundtrip() {
        for i in 0..3 {
            let ff = FileFormat::from_raw(i);
            assert_eq!(ff.to_raw(), i);
        }
    }

    #[test]
    fn test_buffer_properties_default() {
        let props = BufferProperties::default();
        assert_eq!(props.fnum, 0);
        assert_eq!(props.changedtick, 0);
        assert_eq!(props.line_count, 0);
        assert!(!props.has_filename);
        assert!(!props.is_modified);
    }
}
