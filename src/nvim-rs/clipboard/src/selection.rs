//! Selection type handling
//!
//! This module provides types for X11/Wayland selection handling
//! and clipboard data representation.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::redundant_closure_for_method_calls)]

use std::ffi::c_int;

// =============================================================================
// Selection Type
// =============================================================================

/// X11/Wayland selection type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionType {
    /// PRIMARY selection (middle-click paste)
    #[default]
    Primary = 0,
    /// CLIPBOARD selection (Ctrl+C/Ctrl+V)
    Clipboard = 1,
    /// SECONDARY selection (rarely used)
    Secondary = 2,
}

impl SelectionType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Primary),
            1 => Some(Self::Clipboard),
            2 => Some(Self::Secondary),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Get selection name for X11
    pub const fn x11_name(self) -> &'static str {
        match self {
            Self::Primary => "PRIMARY",
            Self::Clipboard => "CLIPBOARD",
            Self::Secondary => "SECONDARY",
        }
    }

    /// Map to register name
    pub const fn to_register(self) -> c_int {
        match self {
            Self::Primary => b'*' as c_int,
            Self::Clipboard | Self::Secondary => b'+' as c_int,
        }
    }

    /// Create from register name
    pub const fn from_register(name: c_int) -> Self {
        match name {
            0x2A => Self::Primary,  // b'*'
            _ => Self::Clipboard,
        }
    }
}

// =============================================================================
// Motion Type
// =============================================================================

/// Motion type for clipboard data (from registers)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MotionType {
    /// Unknown motion type
    #[default]
    Unknown = 0,
    /// Characterwise motion
    CharWise = 1,
    /// Linewise motion
    LineWise = 2,
    /// Blockwise motion
    BlockWise = 3,
}

impl MotionType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::CharWise),
            2 => Some(Self::LineWise),
            3 => Some(Self::BlockWise),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Create from register type character
    pub const fn from_regtype_char(ch: u8) -> Self {
        match ch {
            b'v' | b'c' => Self::CharWise,
            b'V' | b'l' => Self::LineWise,
            b'b' | 0x16 => Self::BlockWise, // Ctrl-V
            _ => Self::Unknown,
        }
    }

    /// Convert to register type character
    pub const fn to_regtype_char(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::CharWise => b'v',
            Self::LineWise => b'V',
            Self::BlockWise => b'b',
        }
    }

    /// Check if motion adds trailing newline
    pub const fn has_trailing_newline(self) -> bool {
        matches!(self, Self::LineWise | Self::BlockWise)
    }
}

// =============================================================================
// Clipboard Data
// =============================================================================

/// Clipboard data representation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClipboardData {
    /// Number of lines
    pub line_count: usize,
    /// Motion type
    pub motion_type: MotionType,
    /// Block width (for blockwise)
    pub block_width: c_int,
    /// Whether data is valid
    pub valid: bool,
}

impl Default for ClipboardData {
    fn default() -> Self {
        Self {
            line_count: 0,
            motion_type: MotionType::Unknown,
            block_width: 0,
            valid: false,
        }
    }
}

impl ClipboardData {
    /// Create empty clipboard data
    pub const fn empty() -> Self {
        Self {
            line_count: 0,
            motion_type: MotionType::Unknown,
            block_width: 0,
            valid: false,
        }
    }

    /// Create with line count and motion type
    pub const fn new(line_count: usize, motion_type: MotionType) -> Self {
        Self {
            line_count,
            motion_type,
            block_width: 0,
            valid: true,
        }
    }

    /// Create blockwise data
    pub const fn blockwise(line_count: usize, width: c_int) -> Self {
        Self {
            line_count,
            motion_type: MotionType::BlockWise,
            block_width: width,
            valid: true,
        }
    }

    /// Check if empty
    pub const fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    /// Check if valid
    pub const fn is_valid(&self) -> bool {
        self.valid
    }

    /// Infer motion type from data
    pub fn infer_motion_type(&mut self, has_trailing_empty: bool) {
        if self.motion_type == MotionType::Unknown {
            if has_trailing_empty {
                self.motion_type = MotionType::LineWise;
            } else {
                self.motion_type = MotionType::CharWise;
            }
        }
    }
}

// =============================================================================
// Clipboard Content
// =============================================================================

/// Opaque handle to clipboard content (lines array)
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ClipboardContentHandle {
    ptr: *mut (),
}

impl Default for ClipboardContentHandle {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }
}

impl ClipboardContentHandle {
    /// Create null handle
    pub const fn null() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.ptr.is_null()
    }

    /// Create from raw pointer
    pub const fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr }
    }

    /// Get raw pointer
    pub const fn as_ptr(self) -> *mut () {
        self.ptr
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if selection type is valid
#[no_mangle]
pub extern "C" fn rs_clipboard_selection_valid(selection: c_int) -> c_int {
    c_int::from(SelectionType::from_raw(selection).is_some())
}

/// FFI export: Get selection register
#[no_mangle]
pub extern "C" fn rs_clipboard_selection_to_register(selection: c_int) -> c_int {
    SelectionType::from_raw(selection).map_or(0, |s| s.to_register())
}

/// FFI export: Get selection from register
#[no_mangle]
pub extern "C" fn rs_clipboard_selection_from_register(name: c_int) -> SelectionType {
    SelectionType::from_register(name)
}

/// FFI export: Check if motion type is valid
#[no_mangle]
pub extern "C" fn rs_clipboard_motion_valid(motion: c_int) -> c_int {
    c_int::from(MotionType::from_raw(motion).is_some())
}

/// FFI export: Get motion from regtype char
#[no_mangle]
pub extern "C" fn rs_clipboard_motion_from_char(ch: u8) -> MotionType {
    MotionType::from_regtype_char(ch)
}

/// FFI export: Get regtype char from motion
#[no_mangle]
pub extern "C" fn rs_clipboard_motion_to_char(motion: c_int) -> u8 {
    MotionType::from_raw(motion).map_or(0, |m| m.to_regtype_char())
}

/// FFI export: Check if motion has trailing newline
#[no_mangle]
pub extern "C" fn rs_clipboard_motion_has_newline(motion: c_int) -> c_int {
    MotionType::from_raw(motion).map_or(0, |m| c_int::from(m.has_trailing_newline()))
}

/// FFI export: Create empty clipboard data
#[no_mangle]
pub extern "C" fn rs_clipboard_data_empty() -> ClipboardData {
    ClipboardData::empty()
}

/// FFI export: Create clipboard data
#[no_mangle]
pub extern "C" fn rs_clipboard_data_new(line_count: usize, motion: c_int) -> ClipboardData {
    let motion_type = MotionType::from_raw(motion).unwrap_or(MotionType::Unknown);
    ClipboardData::new(line_count, motion_type)
}

/// FFI export: Check if clipboard data is empty
#[no_mangle]
pub extern "C" fn rs_clipboard_data_is_empty(data: *const ClipboardData) -> c_int {
    if data.is_null() {
        return 1;
    }
    c_int::from(unsafe { (*data).is_empty() })
}

/// FFI export: Check if clipboard data is valid
#[no_mangle]
pub extern "C" fn rs_clipboard_data_is_valid(data: *const ClipboardData) -> c_int {
    if data.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*data).is_valid() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_type() {
        assert_eq!(
            SelectionType::from_raw(0),
            Some(SelectionType::Primary)
        );
        assert_eq!(
            SelectionType::from_raw(1),
            Some(SelectionType::Clipboard)
        );
        assert_eq!(SelectionType::from_raw(100), None);

        assert_eq!(SelectionType::Primary.x11_name(), "PRIMARY");
        assert_eq!(SelectionType::Clipboard.x11_name(), "CLIPBOARD");
    }

    #[test]
    fn test_selection_register() {
        assert_eq!(SelectionType::Primary.to_register(), b'*' as c_int);
        assert_eq!(SelectionType::Clipboard.to_register(), b'+' as c_int);

        assert_eq!(
            SelectionType::from_register(b'*' as c_int),
            SelectionType::Primary
        );
        assert_eq!(
            SelectionType::from_register(b'+' as c_int),
            SelectionType::Clipboard
        );
    }

    #[test]
    fn test_motion_type() {
        assert_eq!(MotionType::from_raw(0), Some(MotionType::Unknown));
        assert_eq!(MotionType::from_raw(1), Some(MotionType::CharWise));
        assert_eq!(MotionType::from_raw(100), None);

        assert_eq!(MotionType::from_regtype_char(b'v'), MotionType::CharWise);
        assert_eq!(MotionType::from_regtype_char(b'V'), MotionType::LineWise);
        assert_eq!(MotionType::from_regtype_char(b'b'), MotionType::BlockWise);
    }

    #[test]
    fn test_motion_regtype_char() {
        assert_eq!(MotionType::CharWise.to_regtype_char(), b'v');
        assert_eq!(MotionType::LineWise.to_regtype_char(), b'V');
        assert_eq!(MotionType::BlockWise.to_regtype_char(), b'b');
        assert_eq!(MotionType::Unknown.to_regtype_char(), 0);
    }

    #[test]
    fn test_motion_trailing_newline() {
        assert!(!MotionType::CharWise.has_trailing_newline());
        assert!(MotionType::LineWise.has_trailing_newline());
        assert!(MotionType::BlockWise.has_trailing_newline());
    }

    #[test]
    fn test_clipboard_data() {
        let empty = ClipboardData::empty();
        assert!(empty.is_empty());
        assert!(!empty.is_valid());

        let data = ClipboardData::new(5, MotionType::LineWise);
        assert!(!data.is_empty());
        assert!(data.is_valid());
        assert_eq!(data.line_count, 5);

        let block = ClipboardData::blockwise(3, 10);
        assert_eq!(block.motion_type, MotionType::BlockWise);
        assert_eq!(block.block_width, 10);
    }

    #[test]
    fn test_clipboard_data_infer() {
        let mut data = ClipboardData::new(3, MotionType::Unknown);
        data.infer_motion_type(true);
        assert_eq!(data.motion_type, MotionType::LineWise);

        let mut data = ClipboardData::new(3, MotionType::Unknown);
        data.infer_motion_type(false);
        assert_eq!(data.motion_type, MotionType::CharWise);
    }

    #[test]
    fn test_content_handle() {
        let handle = ClipboardContentHandle::null();
        assert!(handle.is_null());

        let ptr = 0x1234 as *mut ();
        let handle = ClipboardContentHandle::from_ptr(ptr);
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_clipboard_selection_valid(0), 1);
        assert_eq!(rs_clipboard_selection_valid(100), 0);

        assert_eq!(
            rs_clipboard_selection_to_register(0),
            b'*' as c_int
        );
        assert_eq!(rs_clipboard_motion_from_char(b'v'), MotionType::CharWise);
        assert_eq!(rs_clipboard_motion_to_char(1), b'v');
    }
}
