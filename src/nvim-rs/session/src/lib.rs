//! Session persistence for Neovim
//!
//! This crate provides Rust implementations for session management functionality
//! from `src/nvim/ex_session.c`, supporting `:mksession`, `:mkview`, `:mkexrc`,
//! and `:mkvimrc` commands.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

mod buffers;
mod components;
mod layout;
mod serializer;

pub use buffers::*;
pub use components::*;
pub use layout::*;
pub use serializer::*;

// =============================================================================
// Session Option Flags (matches kOptSsopFlag* from option_vars.generated.h)
// =============================================================================

bitflags::bitflags! {
    /// Flags for 'sessionoptions' and 'viewoptions'
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SessionFlags: u32 {
        /// Save all buffers, not just those with windows
        const BUFFERS = 0x01;
        /// Save window position (gvim)
        const WINPOS = 0x02;
        /// Save screen size
        const RESIZE = 0x04;
        /// Save window sizes
        const WINSIZE = 0x08;
        /// Save local options/mappings
        const LOCALOPTIONS = 0x10;
        /// Save global options/mappings
        const OPTIONS = 0x20;
        /// Save help windows
        const HELP = 0x40;
        /// Save empty windows
        const BLANK = 0x80;
        /// Save global variables (g:*)
        const GLOBALS = 0x100;
        /// Use forward slashes in paths (always on)
        const SLASH = 0x200;
        /// Use Unix line endings (always on)
        const UNIX = 0x400;
        /// Change to session file directory
        const SESDIR = 0x800;
        /// Save current directory
        const CURDIR = 0x1000;
        /// Save folds
        const FOLDS = 0x2000;
        /// Save cursor position
        const CURSOR = 0x4000;
        /// Save all tab pages
        const TABPAGES = 0x8000;
        /// Save terminal windows
        const TERMINAL = 0x10000;
        /// Skip 'runtimepath' and 'packpath'
        const SKIPRTP = 0x20000;
    }
}

impl Default for SessionFlags {
    fn default() -> Self {
        // Default session options
        Self::CURDIR
            | Self::FOLDS
            | Self::HELP
            | Self::OPTIONS
            | Self::TABPAGES
            | Self::WINSIZE
            | Self::TERMINAL
    }
}

// =============================================================================
// Session Component Enumeration
// =============================================================================

/// Session components that can be saved
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionComponent {
    /// Buffer list and buffer-local options
    Buffers = 0,
    /// Window layout and window-local options
    Windows = 1,
    /// Tab pages
    TabPages = 2,
    /// Global options and mappings
    Options = 3,
    /// Global variables
    Globals = 4,
    /// Folds
    Folds = 5,
    /// Cursor positions
    Cursor = 6,
    /// Argument list
    ArgList = 7,
}

// =============================================================================
// Frame Layout Constants
// =============================================================================

/// Frame layout type: leaf (single window)
pub const FR_LEAF: c_int = 0;
/// Frame layout type: column (vertical split)
pub const FR_COL: c_int = 1;
/// Frame layout type: row (horizontal split)
pub const FR_ROW: c_int = 2;

// =============================================================================
// Session Error Codes
// =============================================================================

/// Success
pub const SESSION_OK: c_int = 0;
/// Write failure
pub const SESSION_FAIL: c_int = 1;
/// File open failure
pub const SESSION_FILE_ERROR: c_int = 2;
/// Invalid argument
pub const SESSION_INVALID_ARG: c_int = 3;

// =============================================================================
// Session Flag Helpers
// =============================================================================

/// Check if session should save all buffers
#[no_mangle]
pub extern "C" fn rs_session_has_buffers(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::BUFFERS)
}

/// Check if session should save window position
#[no_mangle]
pub extern "C" fn rs_session_has_winpos(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::WINPOS)
}

/// Check if session should save screen resize
#[no_mangle]
pub extern "C" fn rs_session_has_resize(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::RESIZE)
}

/// Check if session should save window sizes
#[no_mangle]
pub extern "C" fn rs_session_has_winsize(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::WINSIZE)
}

/// Check if session should save local options
#[no_mangle]
pub extern "C" fn rs_session_has_localoptions(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::LOCALOPTIONS)
}

/// Check if session should save global options
#[no_mangle]
pub extern "C" fn rs_session_has_options(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::OPTIONS)
}

/// Check if session should save help windows
#[no_mangle]
pub extern "C" fn rs_session_has_help(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::HELP)
}

/// Check if session should save blank windows
#[no_mangle]
pub extern "C" fn rs_session_has_blank(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::BLANK)
}

/// Check if session should save global variables
#[no_mangle]
pub extern "C" fn rs_session_has_globals(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::GLOBALS)
}

/// Check if session should change to session directory
#[no_mangle]
pub extern "C" fn rs_session_has_sesdir(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::SESDIR)
}

/// Check if session should save current directory
#[no_mangle]
pub extern "C" fn rs_session_has_curdir(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::CURDIR)
}

/// Check if session should save folds
#[no_mangle]
pub extern "C" fn rs_session_has_folds(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::FOLDS)
}

/// Check if session should save cursor positions
#[no_mangle]
pub extern "C" fn rs_session_has_cursor(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::CURSOR)
}

/// Check if session should save tab pages
#[no_mangle]
pub extern "C" fn rs_session_has_tabpages(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::TABPAGES)
}

/// Check if session should save terminal windows
#[no_mangle]
pub extern "C" fn rs_session_has_terminal(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::TERMINAL)
}

/// Check if session should skip 'runtimepath'
#[no_mangle]
pub extern "C" fn rs_session_has_skiprtp(flags: u32) -> bool {
    SessionFlags::from_bits_truncate(flags).contains(SessionFlags::SKIPRTP)
}

/// Get default session flags
#[no_mangle]
pub extern "C" fn rs_session_default_flags() -> u32 {
    SessionFlags::default().bits()
}

/// Validate session flags - check for conflicting options
#[no_mangle]
pub extern "C" fn rs_session_validate_flags(flags: u32) -> bool {
    let f = SessionFlags::from_bits_truncate(flags);
    // Cannot have both SESDIR and CURDIR
    !(f.contains(SessionFlags::SESDIR) && f.contains(SessionFlags::CURDIR))
}

// =============================================================================
// Session Component Helpers
// =============================================================================

/// Convert component enum to string name
#[no_mangle]
pub extern "C" fn rs_session_component_name(component: c_int) -> *const c_char {
    static BUFFERS: &[u8] = b"buffers\0";
    static WINDOWS: &[u8] = b"windows\0";
    static TABPAGES: &[u8] = b"tabpages\0";
    static OPTIONS: &[u8] = b"options\0";
    static GLOBALS: &[u8] = b"globals\0";
    static FOLDS: &[u8] = b"folds\0";
    static CURSOR: &[u8] = b"cursor\0";
    static ARGLIST: &[u8] = b"arglist\0";
    static UNKNOWN: &[u8] = b"unknown\0";

    let name = match component {
        0 => BUFFERS,
        1 => WINDOWS,
        2 => TABPAGES,
        3 => OPTIONS,
        4 => GLOBALS,
        5 => FOLDS,
        6 => CURSOR,
        7 => ARGLIST,
        _ => UNKNOWN,
    };
    name.as_ptr().cast::<c_char>()
}

// =============================================================================
// Frame Layout Helpers
// =============================================================================

/// Check if frame is a leaf (single window)
#[no_mangle]
pub extern "C" fn rs_session_frame_is_leaf(layout: c_int) -> bool {
    layout == FR_LEAF
}

/// Check if frame is a column (vertical split)
#[no_mangle]
pub extern "C" fn rs_session_frame_is_col(layout: c_int) -> bool {
    layout == FR_COL
}

/// Check if frame is a row (horizontal split)
#[no_mangle]
pub extern "C" fn rs_session_frame_is_row(layout: c_int) -> bool {
    layout == FR_ROW
}

/// Get split command for frame layout
#[no_mangle]
pub extern "C" fn rs_session_frame_split_cmd(layout: c_int) -> *const c_char {
    static SPLIT: &[u8] = b"split\n\0";
    static VSPLIT: &[u8] = b"vsplit\n\0";
    static EMPTY: &[u8] = b"\0";

    let cmd = match layout {
        1 => SPLIT,  // FR_COL
        2 => VSPLIT, // FR_ROW
        _ => EMPTY,
    };
    cmd.as_ptr().cast::<c_char>()
}

/// Get movement command for frame layout (to go back to first window)
#[no_mangle]
pub extern "C" fn rs_session_frame_move_cmd(layout: c_int) -> *const c_char {
    static WINCMD_K: &[u8] = b"wincmd k\n\0";
    static WINCMD_H: &[u8] = b"wincmd h\n\0";
    static EMPTY: &[u8] = b"\0";

    let cmd = match layout {
        1 => WINCMD_K, // FR_COL - move up
        2 => WINCMD_H, // FR_ROW - move left
        _ => EMPTY,
    };
    cmd.as_ptr().cast::<c_char>()
}

// =============================================================================
// Path Escaping for Session Files
// =============================================================================

/// Escape a character for use in a session file path.
/// Returns the escape sequence length (0 if no escaping needed).
///
/// # Safety
/// The `out` pointer must be valid for writing at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_session_escape_char(c: u8, out: *mut u8) -> c_int {
    if out.is_null() {
        return 0;
    }

    match c {
        b' ' => {
            *out = b'\\';
            *out.add(1) = b' ';
            2
        }
        b'\\' => {
            *out = b'\\';
            *out.add(1) = b'\\';
            2
        }
        b'\n' => {
            *out = b'\\';
            *out.add(1) = b'n';
            2
        }
        b'\r' => {
            *out = b'\\';
            *out.add(1) = b'r';
            2
        }
        b'|' | b'"' | b'<' | b'>' | b'#' | b'%' | b'!' => {
            *out = b'\\';
            *out.add(1) = c;
            2
        }
        _ => {
            *out = c;
            1
        }
    }
}

/// Check if a character needs escaping in session file paths
#[no_mangle]
pub extern "C" fn rs_session_char_needs_escape(c: u8) -> bool {
    matches!(
        c,
        b' ' | b'\\' | b'\n' | b'\r' | b'|' | b'"' | b'<' | b'>' | b'#' | b'%' | b'!'
    )
}

/// Convert backslashes to forward slashes in a path (for session file portability)
/// Modifies the buffer in place.
///
/// # Safety
/// The `path` pointer must be valid for reading and writing `len` bytes.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_session_slash_path(path: *mut u8, len: c_int) {
    if path.is_null() || len <= 0 {
        return;
    }

    let slice = std::slice::from_raw_parts_mut(path, len as usize);
    for c in slice.iter_mut() {
        if *c == b'\\' {
            *c = b'/';
        }
    }
}

// =============================================================================
// View File Name Generation
// =============================================================================

/// Maximum length for a view file name hash component
pub const VIEW_FILE_HASH_LEN: usize = 10;

/// Generate a simple hash for view file naming
/// Uses djb2 hash algorithm
///
/// # Safety
/// The `path` pointer must be valid for reading `len` bytes.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_session_view_hash(path: *const u8, len: c_int) -> u32 {
    if path.is_null() || len <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(path, len as usize);
    let mut hash: u32 = 5381;
    for &c in slice {
        hash = hash.wrapping_mul(33).wrapping_add(u32::from(c));
    }
    hash
}

// =============================================================================
// Session Variable Name Validation
// =============================================================================

/// Check if a global variable name is valid for session saving.
/// Must start with uppercase letter and contain only word characters.
///
/// # Safety
/// The `name` pointer must be valid for reading `len` bytes.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_session_is_valid_global_var(name: *const u8, len: c_int) -> bool {
    if name.is_null() || len <= 0 {
        return false;
    }

    let slice = std::slice::from_raw_parts(name, len as usize);

    // Must start with uppercase letter
    if slice.is_empty() || !slice[0].is_ascii_uppercase() {
        return false;
    }

    // Rest must be alphanumeric or underscore
    slice
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

// =============================================================================
// Window Size Calculations for Sessions
// =============================================================================

/// Calculate proportional window height for session restoration
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_session_calc_height(
    win_height: c_int,
    total_rows: c_int,
    target_rows: c_int,
) -> c_int {
    if total_rows == 0 {
        return win_height;
    }
    // Formula: (win_height * target_rows + total_rows/2) / total_rows
    // The +total_rows/2 is for rounding
    ((i64::from(win_height) * i64::from(target_rows) + i64::from(total_rows) / 2)
        / i64::from(total_rows)) as c_int
}

/// Calculate proportional window width for session restoration
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_session_calc_width(
    win_width: c_int,
    total_cols: c_int,
    target_cols: c_int,
) -> c_int {
    if total_cols == 0 {
        return win_width;
    }
    // Formula: (win_width * target_cols + total_cols/2) / total_cols
    ((i64::from(win_width) * i64::from(target_cols) + i64::from(total_cols) / 2)
        / i64::from(total_cols)) as c_int
}

// =============================================================================
// Cursor Position Formatting
// =============================================================================

/// Maximum cursor column value (MAXCOL in vim)
pub const MAXCOL: c_int = 0x7FFF_FFFF;

/// Check if cursor is at end of line (curswant == MAXCOL)
#[no_mangle]
pub extern "C" fn rs_session_cursor_at_eol(curswant: c_int) -> bool {
    curswant == MAXCOL
}

/// Format cursor normal command for session file
/// Returns the command type: 0 = "$", 1 = "0{col}|"
#[no_mangle]
pub extern "C" fn rs_session_cursor_cmd_type(curswant: c_int) -> c_int {
    c_int::from(curswant != MAXCOL)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_flags() {
        let flags = SessionFlags::default();
        assert!(flags.contains(SessionFlags::CURDIR));
        assert!(flags.contains(SessionFlags::FOLDS));
        assert!(flags.contains(SessionFlags::TABPAGES));
        assert!(!flags.contains(SessionFlags::BUFFERS));
    }

    #[test]
    fn test_flag_validation() {
        // SESDIR and CURDIR cannot both be set
        let both = SessionFlags::SESDIR | SessionFlags::CURDIR;
        assert!(!rs_session_validate_flags(both.bits()));

        // Either one alone is fine
        assert!(rs_session_validate_flags(SessionFlags::SESDIR.bits()));
        assert!(rs_session_validate_flags(SessionFlags::CURDIR.bits()));
    }

    #[test]
    fn test_escape_char() {
        let mut buf = [0u8; 4];

        // Space needs escaping
        unsafe {
            assert_eq!(rs_session_escape_char(b' ', buf.as_mut_ptr()), 2);
        }
        assert_eq!(&buf[..2], b"\\ ");

        // Regular char doesn't need escaping
        unsafe {
            assert_eq!(rs_session_escape_char(b'a', buf.as_mut_ptr()), 1);
        }
        assert_eq!(buf[0], b'a');
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn test_view_hash() {
        let path = b"test/path.txt";
        unsafe {
            let hash1 = rs_session_view_hash(path.as_ptr(), path.len() as c_int);
            let hash2 = rs_session_view_hash(path.as_ptr(), path.len() as c_int);
            assert_eq!(hash1, hash2);
            assert_ne!(hash1, 0);
        }
    }

    #[test]
    fn test_global_var_validation() {
        unsafe {
            // Valid: starts with uppercase
            assert!(rs_session_is_valid_global_var(b"Foo".as_ptr(), 3));
            assert!(rs_session_is_valid_global_var(b"MY_VAR".as_ptr(), 6));
            assert!(rs_session_is_valid_global_var(b"Test123".as_ptr(), 7));

            // Invalid: starts with lowercase
            assert!(!rs_session_is_valid_global_var(b"foo".as_ptr(), 3));

            // Invalid: empty
            assert!(!rs_session_is_valid_global_var(b"".as_ptr(), 0));
        }
    }

    #[test]
    fn test_calc_height() {
        // 50% of 100 rows at 80 rows target
        assert_eq!(rs_session_calc_height(50, 100, 80), 40);

        // Handle zero total
        assert_eq!(rs_session_calc_height(50, 0, 80), 50);
    }

    #[test]
    fn test_cursor_at_eol() {
        assert!(rs_session_cursor_at_eol(MAXCOL));
        assert!(!rs_session_cursor_at_eol(10));
    }

    #[test]
    fn test_frame_layout() {
        assert!(rs_session_frame_is_leaf(FR_LEAF));
        assert!(rs_session_frame_is_col(FR_COL));
        assert!(rs_session_frame_is_row(FR_ROW));
    }
}
