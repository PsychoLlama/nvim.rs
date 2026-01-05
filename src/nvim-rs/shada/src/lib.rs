//! ShaDa (Session Data) persistence for Neovim
//!
//! This crate provides Rust implementations for ShaDa file handling,
//! including reading and writing session data like marks, registers,
//! history, search patterns, and variables.
//!
//! # ShaDa File Format
//!
//! ShaDa files use MessagePack encoding and contain entries of various types:
//! - Header: File metadata
//! - Search patterns: Last search and substitute patterns
//! - History entries: Command, search, expression, input, debug history
//! - Registers: Named and numbered registers
//! - Global marks: A-Z and 0-9 marks
//! - Local marks: Buffer-local marks
//! - Jump list: Global jump history
//! - Change list: Buffer change positions
//! - Variables: Global Vim variables
//! - Buffer list: List of open buffers

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// ShaDa Entry Type Constants
// =============================================================================

/// Unknown item type (used for unrecognized entries).
pub const SD_ITEM_UNKNOWN: i32 = -1;
/// Missing value. Should never appear in a file.
pub const SD_ITEM_MISSING: i32 = 0;
/// Header. Present for debugging purposes.
pub const SD_ITEM_HEADER: i32 = 1;
/// Last search pattern (not history item).
pub const SD_ITEM_SEARCH_PATTERN: i32 = 2;
/// Last substitute replacement string.
pub const SD_ITEM_SUB_STRING: i32 = 3;
/// History item.
pub const SD_ITEM_HISTORY_ENTRY: i32 = 4;
/// Register.
pub const SD_ITEM_REGISTER: i32 = 5;
/// Global variable.
pub const SD_ITEM_VARIABLE: i32 = 6;
/// Global mark definition.
pub const SD_ITEM_GLOBAL_MARK: i32 = 7;
/// Item from jump list.
pub const SD_ITEM_JUMP: i32 = 8;
/// Buffer list.
pub const SD_ITEM_BUFFER_LIST: i32 = 9;
/// Buffer-local mark.
pub const SD_ITEM_LOCAL_MARK: i32 = 10;
/// Item from buffer change list.
pub const SD_ITEM_CHANGE: i32 = 11;

/// Last valid entry type number.
pub const SHADA_LAST_ENTRY: u64 = SD_ITEM_CHANGE as u64;

// =============================================================================
// ShaDa Read Status Constants
// =============================================================================

/// Reading was successful.
pub const SD_READ_STATUS_SUCCESS: i32 = 0;
/// Nothing more to read.
pub const SD_READ_STATUS_FINISHED: i32 = 1;
/// Failed to read from file.
pub const SD_READ_STATUS_READ_ERROR: i32 = 2;
/// Input is most likely not a ShaDa file.
pub const SD_READ_STATUS_NOT_SHADA: i32 = 3;
/// Error in the currently read item.
pub const SD_READ_STATUS_MALFORMED: i32 = 4;

// =============================================================================
// ShaDa Write Result Constants
// =============================================================================

/// Writing was successful.
pub const SD_WRITE_SUCCESSFUL: i32 = 0;
/// Writing was successful, but when reading it attempted to read file
/// that did not look like a ShaDa file.
pub const SD_WRITE_READ_NOT_SHADA: i32 = 1;
/// Writing was not successful (e.g. because there was no space left on device).
pub const SD_WRITE_FAILED: i32 = 2;
/// Writing resulted in an error which can be ignored.
pub const SD_WRITE_IGN_ERROR: i32 = 3;

// =============================================================================
// ShaDa Read Flags
// =============================================================================

/// Read header (usually ignored).
pub const SD_READ_HEADER: u32 = 1 << SD_ITEM_HEADER;
/// Read search pattern.
pub const SD_READ_SEARCH_PATTERN: u32 = 1 << SD_ITEM_SEARCH_PATTERN;
/// Read substitute string.
pub const SD_READ_SUB_STRING: u32 = 1 << SD_ITEM_SUB_STRING;
/// Read jump list.
pub const SD_READ_JUMP: u32 = 1 << SD_ITEM_JUMP;
/// Data reading which cannot be disabled.
pub const SD_READ_UNDISABLEABLE_DATA: u32 =
    SD_READ_SEARCH_PATTERN | SD_READ_SUB_STRING | SD_READ_JUMP;
/// Read registers.
pub const SD_READ_REGISTERS: u32 = 1 << SD_ITEM_REGISTER;
/// Read history.
pub const SD_READ_HISTORY: u32 = 1 << SD_ITEM_HISTORY_ENTRY;
/// Read variables.
pub const SD_READ_VARIABLES: u32 = 1 << SD_ITEM_VARIABLE;
/// Read buffer list.
pub const SD_READ_BUFFER_LIST: u32 = 1 << SD_ITEM_BUFFER_LIST;
/// Read unknown items.
pub const SD_READ_UNKNOWN: u32 = 1 << (SHADA_LAST_ENTRY as u32 + 1);
/// Read global marks.
pub const SD_READ_GLOBAL_MARKS: u32 = 1 << SD_ITEM_GLOBAL_MARK;
/// Read local marks.
pub const SD_READ_LOCAL_MARKS: u32 = 1 << SD_ITEM_LOCAL_MARK;
/// Read changes.
pub const SD_READ_CHANGES: u32 = 1 << SD_ITEM_CHANGE;

// =============================================================================
// ShaDa File Flags
// =============================================================================

/// Load non-mark information.
pub const SHADA_WANT_INFO: i32 = 1;
/// Load local file marks and change list.
pub const SHADA_WANT_MARKS: i32 = 2;
/// Overwrite info already read.
pub const SHADA_FORCEIT: i32 = 4;
/// Load v:oldfiles.
pub const SHADA_GET_OLDFILES: i32 = 8;
/// Error out when os_open returns -ENOENT.
pub const SHADA_MISSING_ERROR: i32 = 16;

// =============================================================================
// History Type Constants (from cmdhist_defs.h)
// =============================================================================

/// Command-line history.
pub const HIST_CMD: u8 = 0;
/// Search history.
pub const HIST_SEARCH: u8 = 1;
/// Expression history.
pub const HIST_EXPR: u8 = 2;
/// Input history.
pub const HIST_INPUT: u8 = 3;
/// Debug history.
pub const HIST_DEBUG: u8 = 4;
/// Number of history types.
pub const HIST_COUNT: usize = 5;

// =============================================================================
// Search Pattern Key Names (msgpack map keys)
// =============================================================================

/// Key for magic flag.
pub const SEARCH_KEY_MAGIC: &[u8] = b"sm";
/// Key for smartcase flag.
pub const SEARCH_KEY_SMARTCASE: &[u8] = b"sc";
/// Key for has_line_offset flag.
pub const SEARCH_KEY_HAS_LINE_OFFSET: &[u8] = b"sl";
/// Key for place_cursor_at_end flag.
pub const SEARCH_KEY_PLACE_CURSOR_AT_END: &[u8] = b"se";
/// Key for is_last_used flag.
pub const SEARCH_KEY_IS_LAST_USED: &[u8] = b"su";
/// Key for is_substitute_pattern flag.
pub const SEARCH_KEY_IS_SUBSTITUTE_PATTERN: &[u8] = b"ss";
/// Key for highlighted flag.
pub const SEARCH_KEY_HIGHLIGHTED: &[u8] = b"sh";
/// Key for offset.
pub const SEARCH_KEY_OFFSET: &[u8] = b"so";
/// Key for pattern.
pub const SEARCH_KEY_PAT: &[u8] = b"sp";
/// Key for search_backward flag.
pub const SEARCH_KEY_BACKWARD: &[u8] = b"sb";

// =============================================================================
// Register Key Names (msgpack map keys)
// =============================================================================

/// Key for register type.
pub const REG_KEY_TYPE: &[u8] = b"rt";
/// Key for register width.
pub const REG_KEY_WIDTH: &[u8] = b"rw";
/// Key for register contents.
pub const REG_KEY_CONTENTS: &[u8] = b"rc";
/// Key for unnamed flag.
pub const REG_KEY_UNNAMED: &[u8] = b"ru";

// =============================================================================
// Common Key Names (msgpack map keys)
// =============================================================================

/// Key for line number.
pub const KEY_LNUM: &[u8] = b"l";
/// Key for column.
pub const KEY_COL: &[u8] = b"c";
/// Key for file name.
pub const KEY_FILE: &[u8] = b"f";
/// Key for mark name character.
pub const KEY_NAME_CHAR: &[u8] = b"n";

// =============================================================================
// Error Message Prefixes
// =============================================================================

/// Common prefix for all errors inside ShaDa file (parsing errors).
pub const RERR: &str = "E575: ";
/// Common prefix for critical read errors.
pub const RCERR: &str = "E576: ";
/// Common prefix for all "system" errors.
pub const SERR: &str = "E886: ";
/// Common prefix for all "rename" errors.
pub const RNERR: &str = "E136: ";
/// Common prefix for all ignorable "write" errors.
pub const WERR: &str = "E574: ";

// =============================================================================
// Mark Constants
// =============================================================================

/// Number of local marks (a-z, plus extra marks).
pub const NLOCALMARKS: usize = 26;
/// Size of jump list.
pub const JUMPLISTSIZE: usize = 100;
/// Number of named marks (A-Z).
pub const NMARKS: usize = 26;
/// Number of extra marks ('0-'9).
pub const EXTRA_MARKS: usize = 10;
/// Number of registers saved to ShaDa.
pub const NUM_SAVED_REGISTERS: usize = 37;

// =============================================================================
// MotionType Constants (from normal_defs.h)
// =============================================================================

/// Character-wise motion.
pub const MT_CHAR_WISE: c_int = 0;
/// Line-wise motion.
pub const MT_LINE_WISE: c_int = 1;
/// Block-wise motion.
pub const MT_BLOCK_WISE: c_int = 2;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to FileDescriptor for file operations.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FileDescriptorHandle(*mut c_void);

impl FileDescriptorHandle {
    /// Create a handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to PackerBuffer for msgpack packing.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PackerBufferHandle(*mut c_void);

impl PackerBufferHandle {
    /// Create a handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Position Type
// =============================================================================

/// Position in a buffer (line number, column, column add).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-based).
    pub lnum: i64,
    /// Column number (0-based).
    pub col: i32,
    /// Additional column offset for virtual columns.
    pub coladd: i32,
}

impl Position {
    /// Create a new position.
    #[inline]
    pub const fn new(lnum: i64, col: i32, coladd: i32) -> Self {
        Self { lnum, col, coladd }
    }

    /// Default position (line 1, column 0).
    pub const DEFAULT: Self = Self {
        lnum: 1,
        col: 0,
        coladd: 0,
    };
}

// =============================================================================
// Timestamp Type
// =============================================================================

/// Timestamp for ShaDa entries (Unix epoch seconds).
pub type Timestamp = u64;

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

// These are FFI declarations for C functions that will be called from Rust.
// Some are not yet used but will be needed for future migrations.
#[allow(dead_code)]
extern "C" {
    // History iteration
    fn nvim_hist_iter(
        iter: *const c_void,
        history_type: u8,
        zero: c_int,
        out_string: *mut *mut c_char,
        out_timestamp: *mut Timestamp,
        out_sep: *mut c_char,
    ) -> *const c_void;

    // History type to char conversion (C version)
    fn rs_hist_type2char_c(hist_type: c_int) -> c_int;

    // ShaDa parameter functions
    fn nvim_get_shada_parameter(typ: c_int) -> c_int;
    fn nvim_find_shada_parameter(typ: c_int) -> *const c_char;

    // File operations
    fn nvim_file_skip(fd: FileDescriptorHandle, offset: usize) -> isize;
    fn nvim_file_eof(fd: FileDescriptorHandle) -> c_int;
    fn nvim_file_close(fd: FileDescriptorHandle, fsync: c_int) -> c_int;

    // Memory allocation
    fn nvim_xfree(ptr: *mut c_void);
    fn nvim_xmalloc(size: usize) -> *mut c_void;
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;

    // Option access
    fn nvim_get_p_hi() -> i64;
    fn nvim_get_p_fs() -> c_int;

    // Error messages
    fn nvim_semsg(fmt: *const c_char, ...);
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Convert a history type to its character representation.
///
/// Returns the character for the history type:
/// - HIST_CMD -> ':'
/// - HIST_SEARCH -> '/'
/// - HIST_EXPR -> '='
/// - HIST_INPUT -> '@'
/// - HIST_DEBUG -> '>'
#[no_mangle]
pub extern "C" fn rs_shada_hist_type2char(hist_type: c_int) -> c_int {
    match hist_type {
        0 => c_int::from(b':'), // HIST_CMD
        1 => c_int::from(b'/'), // HIST_SEARCH
        2 => c_int::from(b'='), // HIST_EXPR
        3 => c_int::from(b'@'), // HIST_INPUT
        4 => c_int::from(b'>'), // HIST_DEBUG
        _ => 0,
    }
}

/// Convert a history character back to its type.
///
/// Returns the history type for the character, or -1 if invalid.
#[no_mangle]
pub extern "C" fn rs_shada_hist_char2type(c: c_int) -> c_int {
    let Ok(ch) = u8::try_from(c) else {
        return -1;
    };
    match ch {
        b':' => 0, // HIST_CMD
        b'/' => 1, // HIST_SEARCH
        b'=' => 2, // HIST_EXPR
        b'@' => 3, // HIST_INPUT
        b'>' => 4, // HIST_DEBUG
        _ => -1,
    }
}

/// Check if two positions are equal.
#[inline]
pub const fn marks_equal(a: Position, b: Position) -> bool {
    a.lnum == b.lnum && a.col == b.col
}

/// Get the shada parameter value for a given type character.
///
/// This wraps the C function `get_shada_parameter`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_shada_parameter(typ: c_int) -> c_int {
    nvim_get_shada_parameter(typ)
}

/// Find the shada parameter string for a given type character.
///
/// This wraps the C function `find_shada_parameter`.
/// Returns NULL if the parameter is not found.
#[no_mangle]
pub unsafe extern "C" fn rs_find_shada_parameter(typ: c_int) -> *const c_char {
    nvim_find_shada_parameter(typ)
}

// =============================================================================
// File Skip Helper
// =============================================================================

/// Skip bytes in a file descriptor.
///
/// Returns the read status code.
#[no_mangle]
pub unsafe extern "C" fn rs_sd_reader_skip(
    sd_reader: FileDescriptorHandle,
    offset: usize,
) -> c_int {
    if sd_reader.is_null() {
        return SD_READ_STATUS_READ_ERROR;
    }

    let skip_bytes = nvim_file_skip(sd_reader, offset);
    if skip_bytes < 0 {
        // System error while skipping
        return SD_READ_STATUS_READ_ERROR;
    }
    if (skip_bytes as usize) != offset {
        // Not enough bytes read
        if nvim_file_eof(sd_reader) != 0 {
            // EOF reached before expected
            return SD_READ_STATUS_NOT_SHADA;
        }
        return SD_READ_STATUS_NOT_SHADA;
    }
    SD_READ_STATUS_SUCCESS
}

// =============================================================================
// Entry Type Helpers
// =============================================================================

/// Check if an entry type is valid (known type).
#[no_mangle]
pub extern "C" fn rs_shada_entry_type_valid(entry_type: i32) -> c_int {
    c_int::from((SD_ITEM_MISSING..=SD_ITEM_CHANGE).contains(&entry_type))
}

/// Get the name of an entry type for debugging.
///
/// Returns a static string describing the entry type.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_shada_entry_type_name(entry_type: i32) -> *const c_char {
    match entry_type {
        SD_ITEM_UNKNOWN => c"Unknown".as_ptr(),
        SD_ITEM_MISSING => c"Missing".as_ptr(),
        SD_ITEM_HEADER => c"Header".as_ptr(),
        SD_ITEM_SEARCH_PATTERN => c"SearchPattern".as_ptr(),
        SD_ITEM_SUB_STRING => c"SubString".as_ptr(),
        SD_ITEM_HISTORY_ENTRY => c"HistoryEntry".as_ptr(),
        SD_ITEM_REGISTER => c"Register".as_ptr(),
        SD_ITEM_VARIABLE => c"Variable".as_ptr(),
        SD_ITEM_GLOBAL_MARK => c"GlobalMark".as_ptr(),
        SD_ITEM_JUMP => c"Jump".as_ptr(),
        SD_ITEM_BUFFER_LIST => c"BufferList".as_ptr(),
        SD_ITEM_LOCAL_MARK => c"LocalMark".as_ptr(),
        SD_ITEM_CHANGE => c"Change".as_ptr(),
        _ => c"Invalid".as_ptr(),
    }
}

// =============================================================================
// Read Flag Helpers
// =============================================================================

/// Build the read flags for a given set of options.
///
/// This is a helper to compute the `srni_flags` value based on the
/// ShaDa read file flags and options.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_build_read_flags(flags: c_int, local_marks_param: c_int) -> u32 {
    let mut srni_flags: u32 = 0;

    // Check if we want info
    if (flags & SHADA_WANT_INFO) != 0 {
        srni_flags |= SD_READ_UNDISABLEABLE_DATA;
        srni_flags |= SD_READ_REGISTERS;
        srni_flags |= SD_READ_GLOBAL_MARKS;

        // Check p_hi (history option)
        if nvim_get_p_hi() > 0 {
            srni_flags |= SD_READ_HISTORY;
        }

        // Check for '!' in shada option
        if !nvim_find_shada_parameter(c_int::from(b'!')).is_null() {
            srni_flags |= SD_READ_VARIABLES;
        }

        // Check for '%' in shada option
        if !nvim_find_shada_parameter(c_int::from(b'%')).is_null() {
            srni_flags |= SD_READ_BUFFER_LIST;
        }
    }

    // Check if we want marks
    if (flags & SHADA_WANT_MARKS) != 0 && local_marks_param > 0 {
        srni_flags |= SD_READ_LOCAL_MARKS | SD_READ_CHANGES;
    }

    // Check if we want oldfiles
    if (flags & SHADA_GET_OLDFILES) != 0 {
        srni_flags |= SD_READ_LOCAL_MARKS;
    }

    srni_flags
}

// =============================================================================
// Byte Order Conversion
// =============================================================================

/// Convert a 64-bit value from big-endian to host byte order.
///
/// This is used for reading ShaDa timestamps and other 64-bit values
/// from the msgpack format.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_vim_be64toh(big_endian_64_bits: u64) -> u64 {
    u64::from_be(big_endian_64_bits)
}

/// Convert a 64-bit value from host byte order to big-endian.
///
/// This is used for writing ShaDa timestamps and other 64-bit values
/// to the msgpack format.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_vim_htobe64(host_64_bits: u64) -> u64 {
    host_64_bits.to_be()
}

// Note: Mark index helpers (rs_mark_global_index, rs_mark_local_index, etc.)
// are provided by the nvim-mark crate.

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_constants() {
        assert_eq!(SD_ITEM_UNKNOWN, -1);
        assert_eq!(SD_ITEM_MISSING, 0);
        assert_eq!(SD_ITEM_HEADER, 1);
        assert_eq!(SD_ITEM_SEARCH_PATTERN, 2);
        assert_eq!(SD_ITEM_SUB_STRING, 3);
        assert_eq!(SD_ITEM_HISTORY_ENTRY, 4);
        assert_eq!(SD_ITEM_REGISTER, 5);
        assert_eq!(SD_ITEM_VARIABLE, 6);
        assert_eq!(SD_ITEM_GLOBAL_MARK, 7);
        assert_eq!(SD_ITEM_JUMP, 8);
        assert_eq!(SD_ITEM_BUFFER_LIST, 9);
        assert_eq!(SD_ITEM_LOCAL_MARK, 10);
        assert_eq!(SD_ITEM_CHANGE, 11);
    }

    #[test]
    fn test_read_status_constants() {
        assert_eq!(SD_READ_STATUS_SUCCESS, 0);
        assert_eq!(SD_READ_STATUS_FINISHED, 1);
        assert_eq!(SD_READ_STATUS_READ_ERROR, 2);
        assert_eq!(SD_READ_STATUS_NOT_SHADA, 3);
        assert_eq!(SD_READ_STATUS_MALFORMED, 4);
    }

    #[test]
    fn test_write_result_constants() {
        assert_eq!(SD_WRITE_SUCCESSFUL, 0);
        assert_eq!(SD_WRITE_READ_NOT_SHADA, 1);
        assert_eq!(SD_WRITE_FAILED, 2);
        assert_eq!(SD_WRITE_IGN_ERROR, 3);
    }

    #[test]
    fn test_history_type_constants() {
        assert_eq!(HIST_CMD, 0);
        assert_eq!(HIST_SEARCH, 1);
        assert_eq!(HIST_EXPR, 2);
        assert_eq!(HIST_INPUT, 3);
        assert_eq!(HIST_DEBUG, 4);
        assert_eq!(HIST_COUNT, 5);
    }

    #[test]
    fn test_hist_type2char() {
        assert_eq!(rs_shada_hist_type2char(0), c_int::from(b':'));
        assert_eq!(rs_shada_hist_type2char(1), c_int::from(b'/'));
        assert_eq!(rs_shada_hist_type2char(2), c_int::from(b'='));
        assert_eq!(rs_shada_hist_type2char(3), c_int::from(b'@'));
        assert_eq!(rs_shada_hist_type2char(4), c_int::from(b'>'));
        assert_eq!(rs_shada_hist_type2char(5), 0);
        assert_eq!(rs_shada_hist_type2char(-1), 0);
    }

    #[test]
    fn test_hist_char2type() {
        assert_eq!(rs_shada_hist_char2type(c_int::from(b':')), 0);
        assert_eq!(rs_shada_hist_char2type(c_int::from(b'/')), 1);
        assert_eq!(rs_shada_hist_char2type(c_int::from(b'=')), 2);
        assert_eq!(rs_shada_hist_char2type(c_int::from(b'@')), 3);
        assert_eq!(rs_shada_hist_char2type(c_int::from(b'>')), 4);
        assert_eq!(rs_shada_hist_char2type(c_int::from(b'!')), -1);
        assert_eq!(rs_shada_hist_char2type(-1), -1);
    }

    #[test]
    fn test_entry_type_valid() {
        assert_eq!(rs_shada_entry_type_valid(SD_ITEM_MISSING), 1);
        assert_eq!(rs_shada_entry_type_valid(SD_ITEM_HEADER), 1);
        assert_eq!(rs_shada_entry_type_valid(SD_ITEM_CHANGE), 1);
        assert_eq!(rs_shada_entry_type_valid(SD_ITEM_UNKNOWN), 0);
        assert_eq!(rs_shada_entry_type_valid(12), 0);
        assert_eq!(rs_shada_entry_type_valid(-2), 0);
    }

    #[test]
    fn test_marks_equal() {
        let a = Position::new(10, 5, 0);
        let b = Position::new(10, 5, 0);
        let c = Position::new(10, 6, 0);
        let d = Position::new(11, 5, 0);

        assert!(marks_equal(a, b));
        assert!(!marks_equal(a, c));
        assert!(!marks_equal(a, d));
    }

    #[test]
    fn test_position_default() {
        let pos = Position::DEFAULT;
        assert_eq!(pos.lnum, 1);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
    }

    #[test]
    fn test_be64_conversion() {
        let value: u64 = 0x0102_0304_0506_0708;
        let be = rs_vim_htobe64(value);
        let back = rs_vim_be64toh(be);
        assert_eq!(back, value);
    }

    // Note: Mark index tests are in the nvim-mark crate

    #[test]
    fn test_read_flags() {
        // Verify the bit patterns
        assert_eq!(SD_READ_HEADER, 1 << 1);
        assert_eq!(SD_READ_SEARCH_PATTERN, 1 << 2);
        assert_eq!(SD_READ_REGISTERS, 1 << 5);
        assert_eq!(SD_READ_HISTORY, 1 << 4);
        assert_eq!(SD_READ_GLOBAL_MARKS, 1 << 7);
        assert_eq!(SD_READ_LOCAL_MARKS, 1 << 10);
        assert_eq!(SD_READ_CHANGES, 1 << 11);
    }

    #[test]
    fn test_motion_type_constants() {
        assert_eq!(MT_CHAR_WISE, 0);
        assert_eq!(MT_LINE_WISE, 1);
        assert_eq!(MT_BLOCK_WISE, 2);
    }
}
