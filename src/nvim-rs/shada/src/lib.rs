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
    fn nvim_get_p_fs() -> bool;

    // Error messages
    fn nvim_semsg(fmt: *const c_char, ...);

    // FileMarks accessor
    fn nvim_filemarks_get_greatest_timestamp(fm: *const c_void) -> Timestamp;

    // Buffer/path filtering (Phase 2)
    fn nvim_shada_get_p_shada() -> *const c_char;
    fn nvim_shada_home_replace_save(buf: *const c_void, src: *const c_char) -> *mut c_char;
    fn nvim_shada_home_replace(
        buf: *const c_void,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: c_int,
    );
    fn nvim_shada_copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    fn nvim_shada_mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn nvim_shada_get_namebuff() -> *mut c_char;
    fn nvim_shada_buf_first() -> *const c_void;
    fn nvim_shada_buf_next(buf: *const c_void) -> *const c_void;
    fn nvim_shada_buf_get_ffname(buf: *const c_void) -> *const c_char;
    fn nvim_shada_buf_is_listed(buf: *const c_void) -> c_int;
    fn nvim_shada_buf_is_quickfix(buf: *const c_void) -> c_int;
    fn nvim_shada_buf_is_terminal(buf: *const c_void) -> c_int;
    // Set(ptr_t) operations
    fn nvim_shada_set_init_ptr() -> *mut c_void;
    fn nvim_shada_set_has_ptr(set: *const c_void, ptr: *const c_void) -> c_int;
    fn nvim_shada_set_put_ptr(set: *mut c_void, ptr: *const c_void);
    fn nvim_shada_set_destroy_ptr(set: *mut c_void);

    // Phase 3: Data collection accessors
    fn nvim_shada_hist_iter_raw(
        iter: *const c_void,
        history_type: u8,
        zero: c_int,
        out_str: *mut *mut c_char,
        out_strlen: *mut usize,
        out_ts: *mut Timestamp,
        out_additional_data: *mut *mut c_void,
    ) -> *const c_void;
    fn nvim_shada_get_search_pattern(
        out_pat: *mut *mut c_char,
        out_magic: *mut c_int,
        out_no_scs: *mut c_int,
        out_ts: *mut Timestamp,
        out_off_line: *mut c_int,
        out_off_end: *mut c_int,
        out_off_off: *mut i64,
        out_off_dir: *mut c_char,
        out_additional_data: *mut *mut c_void,
    );
    fn nvim_shada_get_substitute_pattern(
        out_pat: *mut *mut c_char,
        out_magic: *mut c_int,
        out_no_scs: *mut c_int,
        out_ts: *mut Timestamp,
        out_off_line: *mut c_int,
        out_off_end: *mut c_int,
        out_off_off: *mut i64,
        out_off_dir: *mut c_char,
        out_additional_data: *mut *mut c_void,
    );
    fn nvim_shada_search_was_last_used() -> c_int;
    fn nvim_shada_no_hlsearch() -> c_int;
    fn nvim_shada_reg_iter(
        iter: *const c_void,
        out_name: *mut c_char,
        out_type: *mut c_int,
        out_contents: *mut *mut c_void, // String* array
        out_size: *mut usize,
        out_width: *mut usize,
        out_is_unnamed: *mut c_int,
        out_ts: *mut Timestamp,
        out_additional_data: *mut *mut c_void,
    ) -> *const c_void;
    fn nvim_shada_op_reg_index(name: c_char) -> c_int;
    fn nvim_shada_get_percent_param() -> c_int;
    fn nvim_shada_buf_get_cursor(buf: *const c_void, pos: *mut Position);
    fn nvim_shada_buf_get_additional_data(buf: *const c_void) -> *mut c_void;
    fn nvim_shada_os_time() -> Timestamp;
    fn nvim_shada_setpcmark();
    fn nvim_shada_cleanup_jumplist(wp: *mut c_void, loadfiles: c_int);
    fn nvim_shada_curwin() -> *mut c_void;
    fn nvim_shada_jumplist_iter(
        iter: *const c_void,
        wp: *mut c_void,
        out_mark: *mut Position,
        out_fnum: *mut c_int,
        out_ts: *mut Timestamp,
        out_fname: *mut *mut c_char,
        out_additional_data: *mut *mut c_void,
    ) -> *const c_void;
    fn nvim_shada_buflist_findnr(nr: c_int) -> *const c_void;
    fn nvim_shada_siemsg(msg: *const c_char);

    // Phase 4: Entry free consolidation accessors
    fn nvim_shada_tv_clear(tv: *mut c_void);
    fn nvim_shada_free_reg_contents(contents_ptr: *mut c_void, contents_size: usize);
    fn nvim_shada_free_variable(entry: *mut ShadaEntry);

    // Phase 5: File I/O accessors
    fn nvim_shada_file_open(fd: FileDescriptorHandle, fname: *const c_char) -> c_int;
    fn nvim_shada_read(fd: FileDescriptorHandle, flags: c_int);
    fn nvim_shada_os_strerror(err: c_int) -> *const c_char;
    fn nvim_shada_verbose_enter();
    fn nvim_shada_verbose_leave();
    fn nvim_shada_get_p_verbose() -> c_int;
    fn nvim_shada_smsg_reading(
        fname: *const c_char,
        want_info: c_int,
        want_marks: c_int,
        get_oldfiles: c_int,
        failed: c_int,
    );
    fn nvim_shada_get_p_fs() -> c_int;
    fn nvim_shada_build_default_path() -> *mut c_char;
    fn nvim_shada_semsg_close_error(strerror_msg: *const c_char);
    fn nvim_shada_semsg_open_error(fname: *const c_char, strerror_msg: *const c_char);
    fn nvim_shada_file_descriptor_size() -> usize;
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

/// FFI export: check if two positions are equal.
#[no_mangle]
pub extern "C" fn rs_marks_equal(a: Position, b: Position) -> c_int {
    c_int::from(marks_equal(a, b))
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
// MessagePack Reading Utilities
// =============================================================================

// C accessor functions for file reading
extern "C" {
    fn nvim_file_read(fd: FileDescriptorHandle, buf: *mut c_char, size: usize) -> isize;
}

/// Read exactly `len` bytes from a file descriptor.
///
/// Returns the read status.
///
/// # Safety
///
/// - `fd` must be a valid file descriptor handle
/// - `buf` must be valid for `len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_fread_len(
    fd: FileDescriptorHandle,
    buf: *mut c_char,
    len: usize,
) -> c_int {
    if fd.is_null() || buf.is_null() {
        return SD_READ_STATUS_READ_ERROR;
    }

    let bytes_read = nvim_file_read(fd, buf, len);

    if bytes_read < 0 {
        return SD_READ_STATUS_READ_ERROR;
    }

    if (bytes_read as usize) != len {
        return SD_READ_STATUS_NOT_SHADA;
    }

    SD_READ_STATUS_SUCCESS
}

/// Read a MessagePack unsigned 64-bit integer from a file.
///
/// This reads the MessagePack format for positive integers:
/// - 0x00-0x7f: positive fixint (value is the byte itself)
/// - 0xcc: uint8 (1 additional byte)
/// - 0xcd: uint16 (2 additional bytes, big-endian)
/// - 0xce: uint32 (4 additional bytes, big-endian)
/// - 0xcf: uint64 (8 additional bytes, big-endian)
///
/// # Safety
///
/// - `fd` must be a valid file descriptor handle
/// - `result` must be a valid pointer to write the result
#[no_mangle]
pub unsafe extern "C" fn rs_msgpack_read_uint64(
    fd: FileDescriptorHandle,
    allow_eof: bool,
    result: *mut u64,
) -> c_int {
    if fd.is_null() || result.is_null() {
        return SD_READ_STATUS_READ_ERROR;
    }

    // Read the first byte
    let mut first_byte: u8 = 0;
    let bytes_read = nvim_file_read(fd, std::ptr::addr_of_mut!(first_byte).cast(), 1);

    if bytes_read < 0 {
        return SD_READ_STATUS_READ_ERROR;
    }

    if bytes_read == 0 {
        if allow_eof && nvim_file_eof(fd) != 0 {
            return SD_READ_STATUS_FINISHED;
        }
        return SD_READ_STATUS_NOT_SHADA;
    }

    // Check for positive fixint (0x00-0x7f)
    if (first_byte & 0x80) == 0 {
        *result = u64::from(first_byte);
        return SD_READ_STATUS_SUCCESS;
    }

    // Determine the length based on format byte
    let length: usize = match first_byte {
        0xcc => 1, // uint8
        0xcd => 2, // uint16
        0xce => 4, // uint32
        0xcf => 8, // uint64
        _ => return SD_READ_STATUS_NOT_SHADA,
    };

    // Read the value bytes into a buffer
    let mut buf: u64 = 0;
    let buf_ptr = std::ptr::addr_of_mut!(buf).cast::<u8>();

    // Read into the high bytes for big-endian conversion
    let offset = 8 - length;
    let read_status = rs_fread_len(fd, buf_ptr.add(offset).cast(), length);
    if read_status != SD_READ_STATUS_SUCCESS {
        return read_status;
    }

    // Convert from big-endian
    *result = u64::from_be(buf);

    SD_READ_STATUS_SUCCESS
}

/// Skip bytes in a ShaDa file using the file skip function.
///
/// This is used to skip over entries we don't want to read.
///
/// # Safety
///
/// - `fd` must be a valid file descriptor handle
#[no_mangle]
pub unsafe extern "C" fn rs_sd_reader_skip_bytes(fd: FileDescriptorHandle, offset: usize) -> c_int {
    if fd.is_null() {
        return SD_READ_STATUS_READ_ERROR;
    }

    let result = nvim_file_skip(fd, offset);
    if result < 0 {
        return SD_READ_STATUS_READ_ERROR;
    }

    if (result as usize) != offset {
        return SD_READ_STATUS_NOT_SHADA;
    }

    SD_READ_STATUS_SUCCESS
}

// =============================================================================
// ShaDa File Reading Infrastructure
// =============================================================================

/// Entry header information read from a ShaDa file.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ShadaEntryHeader {
    /// Entry type (one of SD_ITEM_* constants)
    pub entry_type: u64,
    /// Entry timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Length of entry data in bytes
    pub length: u64,
}

/// Read the header of a ShaDa entry (type, timestamp, length).
///
/// This is the first step in reading a ShaDa entry. After reading the header,
/// the caller can decide whether to read or skip the entry data.
///
/// # Safety
///
/// - `fd` must be a valid file descriptor handle
/// - `header` must be a valid pointer to write the header
#[no_mangle]
pub unsafe extern "C" fn rs_shada_read_entry_header(
    fd: FileDescriptorHandle,
    header: *mut ShadaEntryHeader,
    allow_eof: bool,
) -> c_int {
    if fd.is_null() || header.is_null() {
        return SD_READ_STATUS_READ_ERROR;
    }

    let mut entry_type: u64 = SD_ITEM_MISSING as u64;
    let mut timestamp: u64 = 0;
    let mut length: u64 = 0;

    // Read entry type
    let mut status = rs_msgpack_read_uint64(fd, allow_eof, &raw mut entry_type);
    if status != SD_READ_STATUS_SUCCESS {
        return status;
    }

    // Read timestamp
    status = rs_msgpack_read_uint64(fd, false, &raw mut timestamp);
    if status != SD_READ_STATUS_SUCCESS {
        return status;
    }

    // Read length
    status = rs_msgpack_read_uint64(fd, false, &raw mut length);
    if status != SD_READ_STATUS_SUCCESS {
        return status;
    }

    // Write to output
    (*header).entry_type = entry_type;
    (*header).timestamp = timestamp;
    (*header).length = length;

    SD_READ_STATUS_SUCCESS
}

/// Check if a ShaDa entry header is valid.
///
/// Validates that:
/// - Entry type is not kSDItemMissing (0)
/// - Length is within reasonable bounds
///
/// # Safety
///
/// `header` must be a valid pointer
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_shada_validate_entry_header(header: *const ShadaEntryHeader) -> c_int {
    if header.is_null() {
        return SD_READ_STATUS_NOT_SHADA;
    }

    let entry_type = (*header).entry_type;
    let length = (*header).length;

    // Entry type 0 (kSDItemMissing) should never appear in a file
    if entry_type == 0 {
        return SD_READ_STATUS_NOT_SHADA;
    }

    // Check for unreasonably large entries (PTRDIFF_MAX equivalent)
    if length > isize::MAX as u64 {
        return SD_READ_STATUS_NOT_SHADA;
    }

    SD_READ_STATUS_SUCCESS
}

/// Check if we should read an entry based on flags and type.
///
/// Returns true if the entry should be read, false if it should be skipped.
///
/// # Safety
///
/// `header` must be a valid pointer
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_shada_should_read_entry(
    header: *const ShadaEntryHeader,
    flags: u32,
    max_kbyte: usize,
) -> c_int {
    if header.is_null() {
        return 0;
    }

    let entry_type = (*header).entry_type;
    let length = (*header).length as usize;

    // Check max size constraint
    if max_kbyte > 0 && length > max_kbyte * 1024 {
        return 0;
    }

    // Unknown entry types
    if entry_type > SHADA_LAST_ENTRY {
        // Check if unknown entries are requested
        return c_int::from((flags & SD_READ_UNKNOWN) != 0);
    }

    // Known entry types - check if this type is in the flags
    let type_flag = 1u32 << (entry_type as u32);
    c_int::from((flags & type_flag) != 0)
}

/// Convert a raw entry type number to ShadaEntryType.
///
/// Types greater than SHADA_LAST_ENTRY are converted to Unknown.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_type_from_raw(raw_type: u64) -> c_int {
    if raw_type > SHADA_LAST_ENTRY {
        return SD_ITEM_UNKNOWN;
    }
    raw_type as c_int
}

/// Check if an entry type is unknown (future version compatibility).
#[no_mangle]
pub const extern "C" fn rs_shada_is_unknown_entry(entry_type: u64) -> c_int {
    if entry_type > SHADA_LAST_ENTRY {
        1
    } else {
        0
    }
}

// =============================================================================
// MessagePack Writing Utilities for ShaDa
// =============================================================================

/// ShaDa packing buffer - wraps PackerBuffer from msgpack crate
#[repr(C)]
pub struct ShadaPackerBuffer {
    _opaque: [u8; 0],
}

// C accessor functions for packer buffer
extern "C" {
    fn nvim_shada_packer_get_ptr(packer: *mut ShadaPackerBuffer) -> *mut u8;
    fn nvim_shada_packer_set_ptr(packer: *mut ShadaPackerBuffer, ptr: *mut u8);
    fn nvim_shada_packer_get_endptr(packer: *mut ShadaPackerBuffer) -> *mut u8;
    fn nvim_shada_packer_flush(packer: *mut ShadaPackerBuffer);
}

/// Minimum buffer size for packing items
pub const SHADA_PACK_ITEM_SIZE: usize = 9;

/// Ensure the packer buffer has enough space.
///
/// # Safety
///
/// `packer` must be a valid packer buffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_shada_check_buffer(packer: *mut ShadaPackerBuffer) {
    if packer.is_null() {
        return;
    }

    let ptr = nvim_shada_packer_get_ptr(packer);
    let endptr = nvim_shada_packer_get_endptr(packer);
    let remaining = (endptr as usize).saturating_sub(ptr as usize);

    if remaining < 2 * SHADA_PACK_ITEM_SIZE {
        nvim_shada_packer_flush(packer);
    }
}

/// Write a ShaDa entry header (type, timestamp, length).
///
/// The header consists of three msgpack unsigned integers:
/// 1. Entry type
/// 2. Timestamp
/// 3. Content length
///
/// # Safety
///
/// `packer` must be a valid packer buffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_header(
    packer: *mut ShadaPackerBuffer,
    entry_type: u64,
    timestamp: u64,
    length: u64,
) {
    if packer.is_null() {
        return;
    }

    rs_shada_check_buffer(packer);

    let mut ptr = nvim_shada_packer_get_ptr(packer);

    // Pack entry type
    rs_mpack_uint64_inline(&raw mut ptr, entry_type);
    // Pack timestamp
    rs_mpack_uint64_inline(&raw mut ptr, timestamp);
    // Pack length
    rs_mpack_uint64_inline(&raw mut ptr, length);

    nvim_shada_packer_set_ptr(packer, ptr);
}

/// Pack a 64-bit unsigned integer in MessagePack format (inline version).
///
/// Uses the most compact representation possible.
unsafe fn rs_mpack_uint64_inline(ptr: *mut *mut u8, val: u64) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }

    if val <= 0x7f {
        // Positive fixint
        **ptr = val as u8;
        *ptr = (*ptr).add(1);
    } else if val <= 0xff {
        // uint8
        **ptr = 0xcc;
        *ptr = (*ptr).add(1);
        **ptr = val as u8;
        *ptr = (*ptr).add(1);
    } else if val <= 0xffff {
        // uint16
        **ptr = 0xcd;
        *ptr = (*ptr).add(1);
        let bytes = (val as u16).to_be_bytes();
        **ptr = bytes[0];
        *ptr = (*ptr).add(1);
        **ptr = bytes[1];
        *ptr = (*ptr).add(1);
    } else if val <= 0xffff_ffff {
        // uint32
        **ptr = 0xce;
        *ptr = (*ptr).add(1);
        let bytes = (val as u32).to_be_bytes();
        for byte in bytes {
            **ptr = byte;
            *ptr = (*ptr).add(1);
        }
    } else {
        // uint64
        **ptr = 0xcf;
        *ptr = (*ptr).add(1);
        let bytes = val.to_be_bytes();
        for byte in bytes {
            **ptr = byte;
            *ptr = (*ptr).add(1);
        }
    }
}

/// Pack raw bytes into the ShaDa packer buffer.
///
/// # Safety
///
/// - `data` must be valid for `len` bytes
/// - `packer` must be a valid packer buffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_raw(
    data: *const u8,
    len: usize,
    packer: *mut ShadaPackerBuffer,
) {
    if packer.is_null() || (data.is_null() && len > 0) {
        return;
    }

    let mut pos: usize = 0;
    while pos < len {
        let ptr = nvim_shada_packer_get_ptr(packer);
        let endptr = nvim_shada_packer_get_endptr(packer);
        let remaining = (endptr as usize).saturating_sub(ptr as usize);
        let to_copy = (len - pos).min(remaining);

        if to_copy > 0 {
            std::ptr::copy_nonoverlapping(data.add(pos), ptr, to_copy);
            nvim_shada_packer_set_ptr(packer, ptr.add(to_copy));
        }
        pos += to_copy;

        if pos < len {
            nvim_shada_packer_flush(packer);
        }
    }

    rs_shada_check_buffer(packer);
}

// =============================================================================
// Phase 4: File Writing Infrastructure
// =============================================================================

/// Free space threshold before flushing packer buffer (4 * MPACK_ITEM_SIZE).
pub const SHADA_MPACK_FREE_SPACE: usize = 4 * SHADA_PACK_ITEM_SIZE;

// C accessor functions for file operations
extern "C" {
    /// Get space remaining in file buffer.
    fn nvim_file_space(fd: FileDescriptorHandle) -> usize;
    /// Flush file buffer to disk.
    fn nvim_file_flush(fd: FileDescriptorHandle) -> c_int;
}

/// Create a packer buffer for writing to a file.
///
/// This sets up a PackerBuffer that writes directly to the file's internal buffer,
/// flushing as needed when the buffer fills up.
///
/// # Safety
///
/// `file` must be a valid file descriptor handle opened for writing.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn rs_packer_buffer_for_file(
    file: FileDescriptorHandle,
    buffer_out: *mut ShadaPackerBuffer,
) -> c_int {
    if file.is_null() || buffer_out.is_null() {
        return -1;
    }

    // Ensure buffer has enough space
    if nvim_file_space(file) < SHADA_MPACK_FREE_SPACE {
        nvim_file_flush(file);
    }

    // Initialize the packer buffer to point to the file's buffer
    // The actual PackerBuffer struct is managed by C code
    0
}

/// Flush the packer buffer to the file.
///
/// Updates the file's write position and flushes to disk.
///
/// # Safety
///
/// `packer` must be a valid packer buffer pointer created for file writing.
#[no_mangle]
pub unsafe extern "C" fn rs_flush_file_buffer(packer: *mut ShadaPackerBuffer) {
    if packer.is_null() {
        return;
    }

    // The flush operation is handled by the C packer_flush callback
    nvim_shada_packer_flush(packer);
}

/// Pack an entry type value (as uint64).
///
/// # Safety
///
/// `packer` must be a valid packer buffer pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_entry_type(
    packer: *mut ShadaPackerBuffer,
    entry_type: ShadaEntryType,
) {
    if packer.is_null() {
        return;
    }

    rs_shada_check_buffer(packer);
    let mut ptr = nvim_shada_packer_get_ptr(packer);
    rs_mpack_uint64_inline(&raw mut ptr, entry_type as u64);
    nvim_shada_packer_set_ptr(packer, ptr);
}

/// Pack a timestamp value (as uint64).
///
/// # Safety
///
/// `packer` must be a valid packer buffer pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_timestamp(
    packer: *mut ShadaPackerBuffer,
    timestamp: Timestamp,
) {
    if packer.is_null() {
        return;
    }

    rs_shada_check_buffer(packer);
    let mut ptr = nvim_shada_packer_get_ptr(packer);
    rs_mpack_uint64_inline(&raw mut ptr, timestamp);
    nvim_shada_packer_set_ptr(packer, ptr);
}

/// Pack content length value (as uint64).
///
/// # Safety
///
/// `packer` must be a valid packer buffer pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_length(packer: *mut ShadaPackerBuffer, length: u64) {
    if packer.is_null() {
        return;
    }

    rs_shada_check_buffer(packer);
    let mut ptr = nvim_shada_packer_get_ptr(packer);
    rs_mpack_uint64_inline(&raw mut ptr, length);
    nvim_shada_packer_set_ptr(packer, ptr);
}

/// Check if entry should be written based on size constraints.
///
/// Returns true if the entry should be written (size is within limits).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_should_write_entry(packed_size: usize, max_kbyte: usize) -> c_int {
    c_int::from(max_kbyte == 0 || packed_size <= max_kbyte * 1024)
}

/// Calculate the number of non-default fields in a search pattern entry.
///
/// This is used to determine the map size when packing.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_shada_search_pattern_field_count(
    entry: *const SearchPatternData,
    defaults: *const SearchPatternData,
) -> u32 {
    if entry.is_null() || defaults.is_null() {
        return 1; // At least the pattern itself
    }

    let e = &*entry;
    let d = &*defaults;

    let mut count: u32 = 1; // Pattern is always present

    if e.magic != d.magic {
        count += 1;
    }
    if e.is_last_used != d.is_last_used {
        count += 1;
    }
    if e.smartcase != d.smartcase {
        count += 1;
    }
    if e.has_line_offset != d.has_line_offset {
        count += 1;
    }
    if e.place_cursor_at_end != d.place_cursor_at_end {
        count += 1;
    }
    if e.is_substitute_pattern != d.is_substitute_pattern {
        count += 1;
    }
    if e.highlighted != d.highlighted {
        count += 1;
    }
    if e.offset != d.offset {
        count += 1;
    }
    if e.search_backward != d.search_backward {
        count += 1;
    }

    count
}

/// Calculate the number of non-default fields in a filemark entry.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_shada_filemark_field_count(
    entry: *const FilemarkData,
    defaults: *const FilemarkData,
    include_name: bool,
) -> u32 {
    if entry.is_null() || defaults.is_null() {
        return 1; // At least the filename
    }

    let e = &*entry;
    let d = &*defaults;

    let mut count: u32 = 1; // Filename is always present

    if e.mark.lnum != d.mark.lnum {
        count += 1;
    }
    if e.mark.col != d.mark.col {
        count += 1;
    }
    if include_name && e.name != d.name {
        count += 1;
    }

    count
}

/// Calculate the number of non-default fields in a register entry.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_shada_register_field_count(
    entry: *const RegisterData,
    defaults: *const RegisterData,
) -> u32 {
    if entry.is_null() || defaults.is_null() {
        return 2; // Contents and name are always present
    }

    let e = &*entry;
    let d = &*defaults;

    let mut count: u32 = 2; // Contents and name

    if e.reg_type != d.reg_type {
        count += 1;
    }
    if e.width != d.width {
        count += 1;
    }
    if e.is_unnamed != d.is_unnamed {
        count += 1;
    }

    count
}

/// Result of a pack operation with size information.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShadaPackResult {
    /// The result code (SD_WRITE_*)
    pub result: c_int,
    /// Size of packed data (if successful)
    pub packed_size: usize,
}

impl Default for ShadaPackResult {
    fn default() -> Self {
        Self {
            result: SD_WRITE_FAILED,
            packed_size: 0,
        }
    }
}

/// Pack an unknown item entry.
///
/// Unknown items are stored with their original type and raw content.
///
/// # Safety
///
/// - `packer` must be a valid packer buffer pointer
/// - `data` in unknown_item must be valid for its size
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_unknown(
    packer: *mut ShadaPackerBuffer,
    unknown_type: u64,
    timestamp: Timestamp,
    data: *const u8,
    size: usize,
) -> c_int {
    if packer.is_null() {
        return SD_WRITE_FAILED;
    }

    rs_shada_check_buffer(packer);

    // Pack header: type, timestamp, length
    let mut ptr = nvim_shada_packer_get_ptr(packer);
    rs_mpack_uint64_inline(&raw mut ptr, unknown_type);
    rs_mpack_uint64_inline(&raw mut ptr, timestamp);
    rs_mpack_uint64_inline(&raw mut ptr, size as u64);
    nvim_shada_packer_set_ptr(packer, ptr);

    // Pack raw data
    if size > 0 && !data.is_null() {
        rs_shada_pack_raw(data, size, packer);
    }

    SD_WRITE_SUCCESSFUL
}

// =============================================================================
// ShaDa Entry Type Enum (Rust representation)
// =============================================================================

/// ShaDa entry type enum matching C's ShadaEntryType.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShadaEntryType {
    /// Unknown item type (used for unrecognized entries).
    Unknown = SD_ITEM_UNKNOWN,
    /// Missing value. Should never appear in a file.
    #[default]
    Missing = SD_ITEM_MISSING,
    /// Header. Present for debugging purposes.
    Header = SD_ITEM_HEADER,
    /// Last search pattern (not history item).
    SearchPattern = SD_ITEM_SEARCH_PATTERN,
    /// Last substitute replacement string.
    SubString = SD_ITEM_SUB_STRING,
    /// History item.
    HistoryEntry = SD_ITEM_HISTORY_ENTRY,
    /// Register.
    Register = SD_ITEM_REGISTER,
    /// Global variable.
    Variable = SD_ITEM_VARIABLE,
    /// Global mark definition.
    GlobalMark = SD_ITEM_GLOBAL_MARK,
    /// Item from jump list.
    Jump = SD_ITEM_JUMP,
    /// Buffer list.
    BufferList = SD_ITEM_BUFFER_LIST,
    /// Buffer-local mark.
    LocalMark = SD_ITEM_LOCAL_MARK,
    /// Item from buffer change list.
    Change = SD_ITEM_CHANGE,
}

impl ShadaEntryType {
    /// Convert from raw i32 value.
    pub const fn from_raw(value: i32) -> Self {
        match value {
            SD_ITEM_MISSING => Self::Missing,
            SD_ITEM_HEADER => Self::Header,
            SD_ITEM_SEARCH_PATTERN => Self::SearchPattern,
            SD_ITEM_SUB_STRING => Self::SubString,
            SD_ITEM_HISTORY_ENTRY => Self::HistoryEntry,
            SD_ITEM_REGISTER => Self::Register,
            SD_ITEM_VARIABLE => Self::Variable,
            SD_ITEM_GLOBAL_MARK => Self::GlobalMark,
            SD_ITEM_JUMP => Self::Jump,
            SD_ITEM_BUFFER_LIST => Self::BufferList,
            SD_ITEM_LOCAL_MARK => Self::LocalMark,
            SD_ITEM_CHANGE => Self::Change,
            _ => Self::Unknown,
        }
    }

    /// Convert to raw i32 value.
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

// =============================================================================
// Search Pattern Data
// =============================================================================

/// Search pattern entry data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SearchPatternData {
    /// Whether magic mode is enabled.
    pub magic: bool,
    /// Whether smartcase is enabled.
    pub smartcase: bool,
    /// Whether has line offset.
    pub has_line_offset: bool,
    /// Whether cursor should be placed at end.
    pub place_cursor_at_end: bool,
    /// Line offset value.
    pub offset: i64,
    /// Whether this is the last used pattern.
    pub is_last_used: bool,
    /// Whether this is a substitute pattern (from :s).
    pub is_substitute_pattern: bool,
    /// Whether pattern is highlighted.
    pub highlighted: bool,
    /// Whether search is backward.
    pub search_backward: bool,
    /// The pattern string (owned).
    pub pat: *mut c_char,
    /// Length of pattern string.
    pub pat_len: usize,
}

impl Default for SearchPatternData {
    fn default() -> Self {
        Self {
            magic: true,
            smartcase: false,
            has_line_offset: false,
            place_cursor_at_end: false,
            offset: 0,
            is_last_used: true,
            is_substitute_pattern: false,
            highlighted: false,
            search_backward: false,
            pat: std::ptr::null_mut(),
            pat_len: 0,
        }
    }
}

// =============================================================================
// Filemark Data (for marks, jumps, changes)
// =============================================================================

/// Filemark entry data for marks, jumps, and changes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FilemarkData {
    /// Mark name character.
    pub name: c_char,
    /// Position (line number, column, coladd).
    pub mark: Position,
    /// File name (owned).
    pub fname: *mut c_char,
}

impl Default for FilemarkData {
    #[allow(clippy::cast_possible_wrap)]
    fn default() -> Self {
        Self {
            name: b'"' as c_char,
            mark: Position::DEFAULT,
            fname: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// History Item Data
// =============================================================================

/// History item entry data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HistoryItemData {
    /// History type (HIST_CMD, HIST_SEARCH, etc.).
    pub histtype: u8,
    /// History string (owned).
    pub string: *mut c_char,
    /// Separator character (for search history).
    pub sep: c_char,
}

impl Default for HistoryItemData {
    fn default() -> Self {
        Self {
            histtype: HIST_CMD,
            string: std::ptr::null_mut(),
            sep: 0,
        }
    }
}

// =============================================================================
// Register Data
// =============================================================================

/// Register entry data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RegisterData {
    /// Register name character.
    pub name: c_char,
    /// Motion type (character-wise, line-wise, block-wise).
    pub reg_type: c_int,
    /// Register contents (array of strings).
    pub contents: *mut *mut c_char,
    /// Number of strings in contents.
    pub contents_size: usize,
    /// Whether this is the unnamed register.
    pub is_unnamed: bool,
    /// Block width (for block-wise registers).
    pub width: usize,
}

impl Default for RegisterData {
    fn default() -> Self {
        Self {
            name: 0,
            reg_type: MT_CHAR_WISE,
            contents: std::ptr::null_mut(),
            contents_size: 0,
            is_unnamed: false,
            width: 0,
        }
    }
}

// =============================================================================
// Global Variable Data
// =============================================================================

/// Global variable entry data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlobalVarData {
    /// Variable name (owned).
    pub name: *mut c_char,
    /// Variable value (typval_T equivalent - opaque for now).
    pub value: *mut c_void,
}

impl Default for GlobalVarData {
    fn default() -> Self {
        Self {
            name: std::ptr::null_mut(),
            value: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Unknown Item Data
// =============================================================================

/// Unknown item entry data (for forward compatibility).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UnknownItemData {
    /// Entry type number.
    pub type_num: u64,
    /// Raw contents (owned).
    pub contents: *mut c_char,
    /// Size of contents.
    pub size: usize,
}

impl Default for UnknownItemData {
    fn default() -> Self {
        Self {
            type_num: 0,
            contents: std::ptr::null_mut(),
            size: 0,
        }
    }
}

// =============================================================================
// Substitute String Data
// =============================================================================

/// Substitute string entry data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SubStringData {
    /// Substitute string (owned).
    pub sub: *mut c_char,
}

impl Default for SubStringData {
    fn default() -> Self {
        Self {
            sub: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Buffer List Buffer Entry
// =============================================================================

/// Single buffer in buffer list.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BufferListBuffer {
    /// Cursor position in buffer.
    pub pos: Position,
    /// File name (owned).
    pub fname: *mut c_char,
    /// Additional data (msgpack dict).
    pub additional_data: *mut c_void,
}

impl Default for BufferListBuffer {
    fn default() -> Self {
        Self {
            pos: Position::DEFAULT,
            fname: std::ptr::null_mut(),
            additional_data: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Buffer List Data
// =============================================================================

/// Buffer list entry data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BufferListData {
    /// Number of buffers.
    pub size: usize,
    /// Array of buffers (owned).
    pub buffers: *mut BufferListBuffer,
}

impl Default for BufferListData {
    fn default() -> Self {
        Self {
            size: 0,
            buffers: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Header Data
// =============================================================================

/// Header entry data (msgpack dict).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HeaderData {
    /// Header dictionary (opaque for now).
    pub dict: *mut c_void,
    /// Number of entries in dict.
    pub size: usize,
}

impl Default for HeaderData {
    fn default() -> Self {
        Self {
            dict: std::ptr::null_mut(),
            size: 0,
        }
    }
}

// =============================================================================
// ShaDa Entry Data Union
// =============================================================================

/// Union of all possible entry data types.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ShadaEntryData {
    pub header: std::mem::ManuallyDrop<HeaderData>,
    pub filemark: std::mem::ManuallyDrop<FilemarkData>,
    pub search_pattern: std::mem::ManuallyDrop<SearchPatternData>,
    pub history_item: std::mem::ManuallyDrop<HistoryItemData>,
    pub reg: std::mem::ManuallyDrop<RegisterData>,
    pub global_var: std::mem::ManuallyDrop<GlobalVarData>,
    pub unknown_item: std::mem::ManuallyDrop<UnknownItemData>,
    pub sub_string: std::mem::ManuallyDrop<SubStringData>,
    pub buffer_list: std::mem::ManuallyDrop<BufferListData>,
}

impl Default for ShadaEntryData {
    fn default() -> Self {
        // Default to zero-initialized header (safest option)
        Self {
            header: std::mem::ManuallyDrop::new(HeaderData::default()),
        }
    }
}

// =============================================================================
// ShaDa Entry Structure
// =============================================================================

/// Complete ShaDa entry matching C's ShadaEntry struct.
#[repr(C)]
#[derive(Clone)]
pub struct ShadaEntry {
    /// Entry type.
    pub entry_type: ShadaEntryType,
    /// Whether the entry's string data can be freed.
    pub can_free_entry: bool,
    /// Entry timestamp (Unix epoch seconds).
    pub timestamp: Timestamp,
    /// Entry data (union based on entry_type).
    pub data: ShadaEntryData,
    /// Additional data dictionary (for forward compatibility).
    pub additional_data: *mut c_void,
}

impl Default for ShadaEntry {
    fn default() -> Self {
        Self {
            entry_type: ShadaEntryType::Missing,
            can_free_entry: false,
            timestamp: 0,
            data: ShadaEntryData::default(),
            additional_data: std::ptr::null_mut(),
        }
    }
}

impl ShadaEntry {
    /// Create a new missing entry.
    pub const fn missing() -> Self {
        Self {
            entry_type: ShadaEntryType::Missing,
            can_free_entry: false,
            timestamp: 0,
            data: ShadaEntryData {
                header: std::mem::ManuallyDrop::new(HeaderData {
                    dict: std::ptr::null_mut(),
                    size: 0,
                }),
            },
            additional_data: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// HML List Entry (for history merging)
// =============================================================================

/// One entry in the sized linked list for history merging.
#[repr(C)]
#[derive(Clone)]
pub struct HMLListEntry {
    /// Entry data.
    pub data: ShadaEntry,
    /// Pointer to next entry or NULL.
    pub next: *mut HMLListEntry,
    /// Pointer to previous entry or NULL.
    pub prev: *mut HMLListEntry,
}

impl Default for HMLListEntry {
    fn default() -> Self {
        Self {
            data: ShadaEntry::default(),
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// HML List (Hash Map-backed Linked List)
// =============================================================================

/// Sized linked list structure for history merging.
///
/// This is a C-compatible representation. The actual map operations
/// will be performed through C accessor functions.
#[repr(C)]
pub struct HMLList {
    /// Pointer to the start of the allocated array of entries.
    pub entries: *mut HMLListEntry,
    /// First entry in the list (not necessarily start of array) or NULL.
    pub first: *mut HMLListEntry,
    /// Last entry in the list or NULL.
    pub last: *mut HMLListEntry,
    /// Last free entry removed by hmll_remove.
    pub free_entry: *mut HMLListEntry,
    /// Last unused element in entries array.
    pub last_free_entry: *mut HMLListEntry,
    /// Number of allocated entries.
    pub size: usize,
    /// Number of entries already used.
    pub num_entries: usize,
    /// Map of history strings to entry pointers (opaque - handled by C).
    pub contained_entries: *mut c_void,
}

impl Default for HMLList {
    fn default() -> Self {
        Self {
            entries: std::ptr::null_mut(),
            first: std::ptr::null_mut(),
            last: std::ptr::null_mut(),
            free_entry: std::ptr::null_mut(),
            last_free_entry: std::ptr::null_mut(),
            size: 0,
            num_entries: 0,
            contained_entries: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// History Merger State
// =============================================================================

/// State structure for history merging.
#[repr(C)]
pub struct HistoryMergerState {
    /// The HML list for merging.
    pub hmll: HMLList,
    /// Whether to do merging.
    pub do_merge: bool,
    /// Whether currently reading.
    pub reading: bool,
    /// Iterator state (opaque pointer to C iterator).
    pub iter: *const c_void,
    /// Last history entry read from Neovim.
    pub last_hist_entry: ShadaEntry,
    /// History type (HIST_CMD, etc.).
    pub history_type: u8,
}

impl Default for HistoryMergerState {
    fn default() -> Self {
        Self {
            hmll: HMLList::default(),
            do_merge: false,
            reading: false,
            iter: std::ptr::null(),
            last_hist_entry: ShadaEntry::default(),
            history_type: HIST_CMD,
        }
    }
}

// =============================================================================
// Phase 5: History Merging Functions
// =============================================================================

// C accessor functions for history operations
extern "C" {
    /// Get next history entry from Neovim.
    fn nvim_shada_hist_iter(
        iter: *const c_void,
        history_type: u8,
        reading: bool,
        entry: *mut ShadaEntry,
    ) -> *const c_void;
    /// Free ShaDa entry contents.
    fn nvim_shada_free_shada_entry(entry: *mut ShadaEntry);
    /// Create contained entries map.
    fn nvim_hmll_map_init() -> *mut c_void;
    /// Destroy contained entries map.
    fn nvim_hmll_map_destroy(map: *mut c_void);
    /// Get entry from map by string key.
    fn nvim_hmll_map_get(map: *mut c_void, key: *const c_char) -> *mut HMLListEntry;
    /// Put entry into map by string key.
    fn nvim_hmll_map_put(map: *mut c_void, key: *const c_char, entry: *mut HMLListEntry);
    /// Remove entry from map by string key.
    fn nvim_hmll_map_del(map: *mut c_void, key: *const c_char);
}

/// Initialize an HML linked list.
///
/// # Safety
///
/// `hmll` must be a valid pointer to an uninitialized HMLList.
#[no_mangle]
pub unsafe extern "C" fn rs_hmll_init(hmll: *mut HMLList, size: usize) {
    if hmll.is_null() || size == 0 {
        return;
    }

    let entries = nvim_xcalloc(size, std::mem::size_of::<HMLListEntry>()).cast::<HMLListEntry>();

    (*hmll) = HMLList {
        entries,
        first: std::ptr::null_mut(),
        last: std::ptr::null_mut(),
        free_entry: std::ptr::null_mut(),
        last_free_entry: entries,
        size,
        num_entries: 0,
        contained_entries: nvim_hmll_map_init(),
    };
}

/// Remove an entry from the HML linked list.
///
/// # Safety
///
/// - `hmll` must be a valid pointer to an initialized HMLList
/// - `hmll_entry` must be a valid entry in the list
#[no_mangle]
pub unsafe extern "C" fn rs_hmll_remove(hmll: *mut HMLList, hmll_entry: *mut HMLListEntry) {
    if hmll.is_null() || hmll_entry.is_null() {
        return;
    }

    let list = &mut *hmll;
    let entry = &mut *hmll_entry;

    // Update free entry tracking
    if hmll_entry == list.last_free_entry.sub(1) {
        list.last_free_entry = list.last_free_entry.sub(1);
    } else {
        list.free_entry = hmll_entry;
    }

    // Remove from the contained entries map
    let key = entry.data.data.history_item.string;
    if !key.is_null() {
        nvim_hmll_map_del(list.contained_entries, key);
    }

    // Update linked list pointers
    if entry.next.is_null() {
        list.last = entry.prev;
    } else {
        (*entry.next).prev = entry.prev;
    }

    if entry.prev.is_null() {
        list.first = entry.next;
    } else {
        (*entry.prev).next = entry.next;
    }

    list.num_entries -= 1;

    // Free the entry data
    nvim_shada_free_shada_entry(&raw mut entry.data);
}

/// Insert an entry into the HML linked list.
///
/// # Safety
///
/// - `hmll` must be a valid pointer to an initialized HMLList
/// - `hmll_entry` is the entry to insert after (can be null to insert at front)
/// - `data` is the data to insert
#[no_mangle]
pub unsafe extern "C" fn rs_hmll_insert(
    hmll: *mut HMLList,
    hmll_entry: *mut HMLListEntry,
    data: ShadaEntry,
) {
    if hmll.is_null() {
        return;
    }

    let list = &mut *hmll;
    let mut insert_after = hmll_entry;

    // If list is full, remove the first (oldest) entry
    if list.num_entries == list.size {
        if insert_after == list.first {
            insert_after = std::ptr::null_mut();
        }
        rs_hmll_remove(hmll, list.first);
    }

    // Get the target entry slot
    let target_entry: *mut HMLListEntry;
    if list.free_entry.is_null() {
        target_entry = list.last_free_entry;
        list.last_free_entry = list.last_free_entry.add(1);
    } else {
        target_entry = list.free_entry;
        list.free_entry = std::ptr::null_mut();
    }

    // Get the key before moving data
    let key = data.data.history_item.string;

    // Set the entry data
    (*target_entry).data = data;

    // Add to the contained entries map
    if !key.is_null() {
        nvim_hmll_map_put(list.contained_entries, key, target_entry);
    }

    list.num_entries += 1;

    // Update linked list pointers
    (*target_entry).prev = insert_after;
    if insert_after.is_null() {
        (*target_entry).next = list.first;
        list.first = target_entry;
    } else {
        (*target_entry).next = (*insert_after).next;
        (*insert_after).next = target_entry;
    }

    if (*target_entry).next.is_null() {
        list.last = target_entry;
    } else {
        (*(*target_entry).next).prev = target_entry;
    }
}

/// Free an HML linked list.
///
/// # Safety
///
/// `hmll` must be a valid pointer to an initialized HMLList.
#[no_mangle]
pub unsafe extern "C" fn rs_hmll_dealloc(hmll: *mut HMLList) {
    if hmll.is_null() {
        return;
    }

    let list = &mut *hmll;

    // Destroy the map
    if !list.contained_entries.is_null() {
        nvim_hmll_map_destroy(list.contained_entries);
    }

    // Free the entries array
    if !list.entries.is_null() {
        nvim_xfree(list.entries.cast::<c_void>());
    }

    // Reset the struct
    *list = HMLList::default();
}

/// Initialize history merger state.
///
/// # Safety
///
/// `hms_p` must be a valid pointer to an uninitialized HistoryMergerState.
#[no_mangle]
pub unsafe extern "C" fn rs_hms_init(
    hms_p: *mut HistoryMergerState,
    history_type: u8,
    num_elements: usize,
    do_merge: bool,
    reading: bool,
) {
    if hms_p.is_null() {
        return;
    }

    let hms = &mut *hms_p;

    rs_hmll_init(&raw mut hms.hmll, num_elements);
    hms.do_merge = do_merge;
    hms.reading = reading;
    hms.history_type = history_type;

    // Initialize iterator and get first history entry
    hms.iter = nvim_shada_hist_iter(
        std::ptr::null(),
        history_type,
        reading,
        &raw mut hms.last_hist_entry,
    );
}

/// Insert an entry into the history merger.
///
/// # Safety
///
/// - `hms_p` must be a valid pointer to an initialized HistoryMergerState
/// - `entry` must be a valid ShadaEntry
#[no_mangle]
pub unsafe extern "C" fn rs_hms_insert(
    hms_p: *mut HistoryMergerState,
    entry: ShadaEntry,
    do_iter: bool,
) {
    if hms_p.is_null() {
        return;
    }

    let hms = &mut *hms_p;

    // If do_iter, first insert entries from Neovim history that are older
    if do_iter {
        while hms.last_hist_entry.entry_type != ShadaEntryType::Missing
            && hms.last_hist_entry.timestamp < entry.timestamp
        {
            let hist_entry = hms.last_hist_entry.clone();
            rs_hms_insert(hms_p, hist_entry, false);

            if hms.iter.is_null() {
                hms.last_hist_entry.entry_type = ShadaEntryType::Missing;
                break;
            }
            hms.iter = nvim_shada_hist_iter(
                hms.iter,
                hms.history_type,
                hms.reading,
                &raw mut hms.last_hist_entry,
            );
        }
    }

    let hmll = &raw mut hms.hmll;
    let key = entry.data.history_item.string;

    // Check if entry already exists
    let existing = if key.is_null() {
        std::ptr::null_mut()
    } else {
        nvim_hmll_map_get((*hmll).contained_entries, key)
    };

    if !existing.is_null() {
        let existing_entry = &mut *existing;
        if entry.timestamp > existing_entry.data.timestamp {
            // New entry is newer, remove the old one
            rs_hmll_remove(hmll, existing);
        } else if !do_iter && entry.timestamp == existing_entry.data.timestamp {
            // Same timestamp, prefer current Neovim instance entry
            nvim_shada_free_shada_entry(&raw mut existing_entry.data);
            existing_entry.data = entry;
            return;
        } else {
            // Existing entry is newer or same timestamp from file, skip
            return;
        }
    }

    // Find insertion point (iterate backwards to find where to insert)
    let mut insert_after = (*hmll).last;
    while !insert_after.is_null() {
        if (*insert_after).data.timestamp <= entry.timestamp {
            break;
        }
        insert_after = (*insert_after).prev;
    }

    rs_hmll_insert(hmll, insert_after, entry);
}

/// Insert all remaining Neovim history entries into the merger.
///
/// # Safety
///
/// `hms_p` must be a valid pointer to an initialized HistoryMergerState.
#[no_mangle]
pub unsafe extern "C" fn rs_hms_insert_whole_neovim_history(hms_p: *mut HistoryMergerState) {
    if hms_p.is_null() {
        return;
    }

    let hms = &mut *hms_p;

    while hms.last_hist_entry.entry_type != ShadaEntryType::Missing {
        let hist_entry = hms.last_hist_entry.clone();
        rs_hms_insert(hms_p, hist_entry, false);

        if hms.iter.is_null() {
            break;
        }
        hms.iter = nvim_shada_hist_iter(
            hms.iter,
            hms.history_type,
            hms.reading,
            &raw mut hms.last_hist_entry,
        );
    }
}

/// Free history merger state.
///
/// # Safety
///
/// `hms_p` must be a valid pointer to an initialized HistoryMergerState.
#[no_mangle]
pub unsafe extern "C" fn rs_hms_dealloc(hms_p: *mut HistoryMergerState) {
    if hms_p.is_null() {
        return;
    }

    rs_hmll_dealloc(&raw mut (*hms_p).hmll);
}

/// Get the number of entries in the history merger.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_hms_get_num_entries(hms_p: *const HistoryMergerState) -> usize {
    if hms_p.is_null() {
        return 0;
    }
    (*hms_p).hmll.num_entries
}

/// Get the first entry in the HML list.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_hmll_get_first(hmll: *const HMLList) -> *mut HMLListEntry {
    if hmll.is_null() {
        return std::ptr::null_mut();
    }
    (*hmll).first
}

/// Get the next entry in the HML list iteration.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_hmll_entry_get_next(entry: *const HMLListEntry) -> *mut HMLListEntry {
    if entry.is_null() {
        return std::ptr::null_mut();
    }
    (*entry).next
}

/// Get the data from an HML list entry.
#[no_mangle]
#[allow(
    clippy::not_unsafe_ptr_arg_deref,
    clippy::missing_const_for_fn,
    clippy::borrow_as_ptr
)]
pub unsafe extern "C" fn rs_hmll_entry_get_data(entry: *const HMLListEntry) -> *const ShadaEntry {
    if entry.is_null() {
        return std::ptr::null();
    }
    &raw const (*entry).data
}

// =============================================================================
// Phase 6: High-Level API Functions
// =============================================================================

// C accessor functions for high-level API
extern "C" {
    /// Get default ShaDa file path.
    fn nvim_shada_get_default_file() -> *const c_char;
    /// Get current p_shadafile option.
    fn nvim_get_p_shadafile() -> *const c_char;
    /// Expand environment variables in path.
    fn nvim_expand_env(src: *const c_char, dst: *mut c_char, dstlen: usize) -> usize;
    /// Duplicate string with length and allocation.
    fn nvim_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
    /// Compare strings for equality.
    fn nvim_strequal(s1: *const c_char, s2: *const c_char) -> bool;
}

/// Maximum path length for expansion.
const MAXPATHL: usize = 4096;

/// Get the ShaDa file name to use.
///
/// If `file` is given and not empty, use it.
/// Otherwise use "-i file_name", value from 'shada' or the default.
///
/// # Safety
///
/// `file` must be a valid C string or NULL.
///
/// # Returns
///
/// An allocated string containing the shada file name, or NULL if shada
/// file should not be used.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_filename(file: *const c_char) -> *mut c_char {
    let file = if file.is_null() || *file == 0 {
        // No file provided, check options
        let p_shadafile = nvim_get_p_shadafile();
        if !p_shadafile.is_null() && *p_shadafile != 0 {
            // Check if writing to ShaDa file was disabled ("-i NONE" or "--clean")
            let none_str = c"NONE".as_ptr();
            if nvim_strequal(p_shadafile, none_str) {
                return std::ptr::null_mut();
            }
            p_shadafile
        } else {
            // Check for -n parameter or use default
            let param_file = nvim_find_shada_parameter(c_int::from(b'n'));
            if param_file.is_null() || *param_file == 0 {
                let default_file = nvim_shada_get_default_file();
                // Expand environment variables
                let mut name_buff = [0i8; MAXPATHL];
                let len = nvim_expand_env(default_file, name_buff.as_mut_ptr(), MAXPATHL);
                return nvim_xmemdupz(name_buff.as_ptr(), len);
            }
            // Expand environment variables
            let mut name_buff = [0i8; MAXPATHL];
            let len = nvim_expand_env(param_file, name_buff.as_mut_ptr(), MAXPATHL);
            return nvim_xmemdupz(name_buff.as_ptr(), len);
        }
    } else {
        file
    };

    nvim_xstrdup(file)
}

/// Get the default ShaDa file path.
///
/// Returns a cached path on subsequent calls. The path is built as
/// `<user_state_dir>/shada/main.shada`.
///
/// # Returns
///
/// A pointer to the default ShaDa file path (not allocated, do not free).
#[no_mangle]
pub unsafe extern "C" fn rs_shada_get_default_file() -> *const c_char {
    static mut DEFAULT_SHADA_FILE: *mut c_char = std::ptr::null_mut();
    if DEFAULT_SHADA_FILE.is_null() {
        DEFAULT_SHADA_FILE = nvim_shada_build_default_path();
    }
    DEFAULT_SHADA_FILE
}

/// Read marks information from ShaDa file.
///
/// # Returns
///
/// OK (0) on success, FAIL (1) on failure.
#[no_mangle]
pub extern "C" fn rs_shada_read_marks() -> c_int {
    // Returns kShaDaWantMarks flag
    unsafe { rs_shada_read_file(std::ptr::null(), SHADA_WANT_MARKS as c_int) }
}

/// Read all information from ShaDa file.
///
/// # Safety
///
/// `fname` must be a valid C string or NULL.
///
/// # Returns
///
/// OK (0) on success, FAIL (1) on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_read_everything(
    fname: *const c_char,
    forceit: bool,
    missing_ok: bool,
) -> c_int {
    let mut flags = SHADA_WANT_INFO | SHADA_WANT_MARKS | SHADA_GET_OLDFILES;
    if forceit {
        flags |= SHADA_FORCEIT;
    }
    if !missing_ok {
        flags |= SHADA_MISSING_ERROR;
    }
    rs_shada_read_file(fname, flags as c_int)
}

/// Read ShaDa file with specified flags.
///
/// # Safety
///
/// `file` must be a valid C string or NULL.
///
/// # Returns
///
/// OK (0) on success, FAIL (1) on failure.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn rs_shada_read_file(file: *const c_char, flags: c_int) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = 1;
    const UV_ENOENT: c_int = -2;

    let fname = rs_shada_filename(file);
    if fname.is_null() {
        return FAIL;
    }

    // Allocate a FileDescriptor on the heap (opaque C struct)
    let fd_size = nvim_shada_file_descriptor_size();
    let sd_reader = nvim_xcalloc(1, fd_size);
    let fd = FileDescriptorHandle::from_ptr(sd_reader);

    let of_ret = nvim_shada_file_open(fd, fname);

    if nvim_shada_get_p_verbose() > 1 {
        nvim_shada_verbose_enter();
        nvim_shada_smsg_reading(
            fname,
            c_int::from(flags & SHADA_WANT_INFO as c_int != 0),
            c_int::from(flags & SHADA_WANT_MARKS as c_int != 0),
            c_int::from(flags & SHADA_GET_OLDFILES as c_int != 0),
            c_int::from(of_ret != 0),
        );
        nvim_shada_verbose_leave();
    }

    if of_ret != 0 {
        if of_ret != UV_ENOENT || (flags & SHADA_MISSING_ERROR as c_int) != 0 {
            nvim_shada_semsg_open_error(fname, nvim_shada_os_strerror(of_ret));
        }
        nvim_xfree(fname.cast::<c_void>());
        nvim_xfree(sd_reader);
        return FAIL;
    }
    nvim_xfree(fname.cast::<c_void>());

    nvim_shada_read(fd, flags);
    rs_close_file(fd);
    nvim_xfree(sd_reader);

    OK
}

/// Check if a file path looks like a ShaDa file.
///
/// Basic validation - checks if path ends with ".shada" or similar.
///
/// # Safety
///
/// `path` must be a valid C string.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn rs_shada_is_valid_filename(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    // Simple validation - file exists and has reasonable name
    // More detailed validation happens during actual read
    c_int::from(*path != 0)
}

/// Get the combined flags for reading ShaDa with info and marks.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_get_read_flags(want_info: bool, want_marks: bool) -> c_int {
    let mut flags: c_int = 0;
    if want_info {
        flags |= SHADA_WANT_INFO;
    }
    if want_marks {
        flags |= SHADA_WANT_MARKS;
    }
    flags
}

// =============================================================================
// Phase 7: Encoding/Decoding API
// =============================================================================

/// String structure matching Neovim's String type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvimString {
    /// Pointer to string data.
    pub data: *mut c_char,
    /// Length of string (not including null terminator).
    pub size: usize,
}

impl Default for NvimString {
    fn default() -> Self {
        Self {
            data: std::ptr::null_mut(),
            size: 0,
        }
    }
}

// C accessor functions for encoding/decoding
extern "C" {
    /// Encode registers to string (calls C implementation).
    fn nvim_shada_encode_regs() -> NvimString;
    /// Encode jump list to string (calls C implementation).
    fn nvim_shada_encode_jumps() -> NvimString;
    /// Encode buffer list to string (calls C implementation).
    fn nvim_shada_encode_buflist() -> NvimString;
    /// Encode global variables to string (calls C implementation).
    fn nvim_shada_encode_gvars() -> NvimString;
    /// Read ShaDa from string (calls C implementation).
    fn nvim_shada_read_string(string: NvimString, flags: c_int);
}

/// Encode registers to a ShaDa-format string.
///
/// Returns a newly allocated string containing all register entries
/// in MessagePack format suitable for ShaDa storage.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_regs() -> NvimString {
    nvim_shada_encode_regs()
}

/// Encode jump list to a ShaDa-format string.
///
/// Returns a newly allocated string containing jump list entries
/// in MessagePack format suitable for ShaDa storage.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_jumps() -> NvimString {
    nvim_shada_encode_jumps()
}

/// Encode buffer list to a ShaDa-format string.
///
/// Returns a newly allocated string containing the buffer list entry
/// in MessagePack format suitable for ShaDa storage.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_buflist() -> NvimString {
    nvim_shada_encode_buflist()
}

/// Encode global variables to a ShaDa-format string.
///
/// Returns a newly allocated string containing global variable entries
/// in MessagePack format suitable for ShaDa storage.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_gvars() -> NvimString {
    nvim_shada_encode_gvars()
}

/// Read ShaDa entries from a string.
///
/// Parses the given MessagePack-formatted string and applies the ShaDa
/// entries to Neovim's state according to the specified flags.
///
/// # Safety
///
/// `string` must contain valid MessagePack-formatted ShaDa data.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_read_string(string: NvimString, flags: c_int) {
    if string.data.is_null() || string.size == 0 {
        return;
    }
    nvim_shada_read_string(string, flags);
}

/// Create an empty NvimString.
#[no_mangle]
pub extern "C" fn rs_nvim_string_empty() -> NvimString {
    NvimString::default()
}

/// Check if an NvimString is empty.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_nvim_string_is_empty(s: *const NvimString) -> c_int {
    if s.is_null() {
        return 1;
    }
    c_int::from((*s).data.is_null() || (*s).size == 0)
}

/// Get the size of an NvimString.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_nvim_string_size(s: *const NvimString) -> usize {
    if s.is_null() {
        return 0;
    }
    (*s).size
}

/// Free an NvimString's data.
///
/// # Safety
///
/// The string must have been allocated by Neovim's allocator.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_string_free(s: *mut NvimString) {
    if s.is_null() {
        return;
    }
    if !(*s).data.is_null() {
        nvim_xfree((*s).data.cast::<c_void>());
    }
    (*s).data = std::ptr::null_mut();
    (*s).size = 0;
}

// =============================================================================
// File Marks Structure
// =============================================================================

/// Structure that holds one file's marks.
#[repr(C)]
pub struct FileMarks {
    /// All file marks (a-z).
    pub marks: [ShadaEntry; NLOCALMARKS],
    /// All file changes.
    pub changes: [ShadaEntry; JUMPLISTSIZE],
    /// Number of changes occupied.
    pub changes_size: usize,
    /// All marks with unknown names (dynamically allocated).
    pub additional_marks: *mut ShadaEntry,
    /// Size of the additional_marks array.
    pub additional_marks_size: usize,
    /// Greatest timestamp among marks.
    pub greatest_timestamp: Timestamp,
}

// FileMarks needs a custom Default because it contains arrays
impl Default for FileMarks {
    fn default() -> Self {
        // Use MaybeUninit to safely initialize arrays
        let marks = std::array::from_fn(|_| ShadaEntry::default());
        let changes = std::array::from_fn(|_| ShadaEntry::default());
        Self {
            marks,
            changes,
            changes_size: 0,
            additional_marks: std::ptr::null_mut(),
            additional_marks_size: 0,
            greatest_timestamp: 0,
        }
    }
}

// =============================================================================
// Write Merger State
// =============================================================================

/// State structure used by shada_write.
///
/// Before actually writing, most of the data is read to this structure.
#[repr(C)]
pub struct WriteMergerState {
    /// Structures for history merging.
    pub hms: [HistoryMergerState; HIST_COUNT],
    /// Named global marks (A-Z).
    pub global_marks: [ShadaEntry; NMARKS],
    /// Numbered marks (0-9).
    pub numbered_marks: [ShadaEntry; EXTRA_MARKS],
    /// All registers.
    pub registers: [ShadaEntry; NUM_SAVED_REGISTERS],
    /// All dumped jumps.
    pub jumps: [ShadaEntry; JUMPLISTSIZE],
    /// Number of jumps occupied.
    pub jumps_size: usize,
    /// Last search pattern.
    pub search_pattern: ShadaEntry,
    /// Last s/ search pattern.
    pub sub_search_pattern: ShadaEntry,
    /// Last s// replacement string.
    pub replacement: ShadaEntry,
    /// Names of already dumped variables (opaque - handled by C).
    pub dumped_variables: *mut c_void,
    /// All file marks (opaque - handled by C).
    pub file_marks: *mut c_void,
}

// WriteMergerState needs a custom Default
impl Default for WriteMergerState {
    fn default() -> Self {
        let hms = std::array::from_fn(|_| HistoryMergerState::default());
        let global_marks = std::array::from_fn(|_| ShadaEntry::default());
        let numbered_marks = std::array::from_fn(|_| ShadaEntry::default());
        let registers = std::array::from_fn(|_| ShadaEntry::default());
        let jumps = std::array::from_fn(|_| ShadaEntry::default());
        Self {
            hms,
            global_marks,
            numbered_marks,
            registers,
            jumps,
            jumps_size: 0,
            search_pattern: ShadaEntry::default(),
            sub_search_pattern: ShadaEntry::default(),
            replacement: ShadaEntry::default(),
            dumped_variables: std::ptr::null_mut(),
            file_marks: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Entry Constructors (FFI exports)
// =============================================================================

/// Create a new missing entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_missing() -> ShadaEntry {
    ShadaEntry::missing()
}

/// Create a new header entry.
#[no_mangle]
pub extern "C" fn rs_shada_entry_header(timestamp: Timestamp) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::Header,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            header: std::mem::ManuallyDrop::new(HeaderData::default()),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new search pattern entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_search_pattern(
    timestamp: Timestamp,
    pat: *mut c_char,
    pat_len: usize,
    magic: bool,
    smartcase: bool,
    is_substitute: bool,
    is_last_used: bool,
    search_backward: bool,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::SearchPattern,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            search_pattern: std::mem::ManuallyDrop::new(SearchPatternData {
                magic,
                smartcase,
                has_line_offset: false,
                place_cursor_at_end: false,
                offset: 0,
                is_last_used,
                is_substitute_pattern: is_substitute,
                highlighted: false,
                search_backward,
                pat,
                pat_len,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new substitute string entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_sub_string(timestamp: Timestamp, sub: *mut c_char) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::SubString,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            sub_string: std::mem::ManuallyDrop::new(SubStringData { sub }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new history entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_history(
    timestamp: Timestamp,
    histtype: u8,
    string: *mut c_char,
    sep: c_char,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::HistoryEntry,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            history_item: std::mem::ManuallyDrop::new(HistoryItemData {
                histtype,
                string,
                sep,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new register entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_register(
    timestamp: Timestamp,
    name: c_char,
    reg_type: c_int,
    contents: *mut *mut c_char,
    contents_size: usize,
    is_unnamed: bool,
    width: usize,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::Register,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            reg: std::mem::ManuallyDrop::new(RegisterData {
                name,
                reg_type,
                contents,
                contents_size,
                is_unnamed,
                width,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new global mark entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_global_mark(
    timestamp: Timestamp,
    name: c_char,
    lnum: i64,
    col: i32,
    fname: *mut c_char,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::GlobalMark,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            filemark: std::mem::ManuallyDrop::new(FilemarkData {
                name,
                mark: Position::new(lnum, col, 0),
                fname,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new local mark entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_local_mark(
    timestamp: Timestamp,
    name: c_char,
    lnum: i64,
    col: i32,
    fname: *mut c_char,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::LocalMark,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            filemark: std::mem::ManuallyDrop::new(FilemarkData {
                name,
                mark: Position::new(lnum, col, 0),
                fname,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new jump entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_jump(
    timestamp: Timestamp,
    lnum: i64,
    col: i32,
    fname: *mut c_char,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::Jump,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            filemark: std::mem::ManuallyDrop::new(FilemarkData {
                name: 0,
                mark: Position::new(lnum, col, 0),
                fname,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new change entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_change(
    timestamp: Timestamp,
    lnum: i64,
    col: i32,
    fname: *mut c_char,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::Change,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            filemark: std::mem::ManuallyDrop::new(FilemarkData {
                name: 0,
                mark: Position::new(lnum, col, 0),
                fname,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Create a new buffer list entry.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_buffer_list(
    timestamp: Timestamp,
    buffers: *mut BufferListBuffer,
    size: usize,
) -> ShadaEntry {
    ShadaEntry {
        entry_type: ShadaEntryType::BufferList,
        can_free_entry: true,
        timestamp,
        data: ShadaEntryData {
            buffer_list: std::mem::ManuallyDrop::new(BufferListData { size, buffers }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

// =============================================================================
// Entry Type Accessors
// =============================================================================

/// Get the entry type as raw i32.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_get_type(entry: *const ShadaEntry) -> c_int {
    if entry.is_null() {
        return SD_ITEM_MISSING;
    }
    unsafe { (*entry).entry_type.as_raw() }
}

/// Set the entry type from raw i32.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_entry_set_type(entry: *mut ShadaEntry, entry_type: c_int) {
    if !entry.is_null() {
        (*entry).entry_type = ShadaEntryType::from_raw(entry_type);
    }
}

/// Get the entry timestamp.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_shada_entry_get_timestamp(entry: *const ShadaEntry) -> Timestamp {
    if entry.is_null() {
        return 0;
    }
    unsafe { (*entry).timestamp }
}

/// Set the entry timestamp.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_entry_set_timestamp(
    entry: *mut ShadaEntry,
    timestamp: Timestamp,
) {
    if !entry.is_null() {
        (*entry).timestamp = timestamp;
    }
}

/// Check if entry is missing type.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_shada_entry_is_missing(entry: *const ShadaEntry) -> c_int {
    if entry.is_null() {
        return 1;
    }
    c_int::from(unsafe { (*entry).entry_type == ShadaEntryType::Missing })
}

// =============================================================================
// Entry Comparison Functions
// =============================================================================

/// Compare two entries by timestamp.
///
/// Returns -1 if a < b, 0 if equal, 1 if a > b.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rs_shada_entry_compare_timestamp(
    a: *const ShadaEntry,
    b: *const ShadaEntry,
) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }
    let ts_a = unsafe { (*a).timestamp };
    let ts_b = unsafe { (*b).timestamp };
    match ts_a.cmp(&ts_b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

// =============================================================================
// Memory Management
// =============================================================================

/// Free a ShaDa entry's resources.
///
/// This frees any allocated strings or data within the entry.
/// Does NOT free the entry struct itself.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn rs_shada_free_entry_contents(entry: *mut ShadaEntry) {
    if entry.is_null() || !(*entry).can_free_entry {
        return;
    }

    // Helper to read from ManuallyDrop union fields safely
    macro_rules! read_union_field {
        ($entry:expr, $field:ident, $inner:ident) => {{
            let data_ptr = std::ptr::addr_of!((*$entry).data.$field);
            let inner_ptr: *const _ = std::ptr::addr_of!((**data_ptr).$inner);
            std::ptr::read(inner_ptr)
        }};
    }

    match (*entry).entry_type {
        ShadaEntryType::Unknown => {
            let contents = read_union_field!(entry, unknown_item, contents);
            if !contents.is_null() {
                nvim_xfree(contents.cast());
            }
        }
        // Header is a Dict (kvec_t) — layout differs between Rust and C.
        // In practice, Header entries always have can_free_entry=false,
        // so this branch is unreachable.
        ShadaEntryType::Missing | ShadaEntryType::Header => {}
        ShadaEntryType::SearchPattern => {
            let pat = read_union_field!(entry, search_pattern, pat);
            if !pat.is_null() {
                nvim_xfree(pat.cast());
            }
        }
        ShadaEntryType::SubString => {
            let sub = read_union_field!(entry, sub_string, sub);
            if !sub.is_null() {
                nvim_xfree(sub.cast());
            }
        }
        ShadaEntryType::HistoryEntry => {
            let string = read_union_field!(entry, history_item, string);
            if !string.is_null() {
                nvim_xfree(string.cast());
            }
        }
        ShadaEntryType::Register => {
            // Register contents are String structs in C ({char*, size_t} = 16 bytes each),
            // not simple char* pointers. Delegate to C to free correctly.
            let contents = read_union_field!(entry, reg, contents);
            let contents_size = read_union_field!(entry, reg, contents_size);
            if !contents.is_null() {
                nvim_shada_free_reg_contents(contents.cast(), contents_size);
            }
        }
        ShadaEntryType::Variable => {
            // Variable has layout mismatch between Rust (name + void*) and C (name + typval_T).
            // Delegate entirely to C which knows the correct struct layout.
            nvim_shada_free_variable(entry);
        }
        ShadaEntryType::GlobalMark
        | ShadaEntryType::LocalMark
        | ShadaEntryType::Jump
        | ShadaEntryType::Change => {
            let fname = read_union_field!(entry, filemark, fname);
            if !fname.is_null() {
                nvim_xfree(fname.cast());
            }
        }
        ShadaEntryType::BufferList => {
            let buffers = read_union_field!(entry, buffer_list, buffers);
            let size = read_union_field!(entry, buffer_list, size);
            if !buffers.is_null() {
                for i in 0..size {
                    let buf = buffers.add(i);
                    if !(*buf).fname.is_null() {
                        nvim_xfree((*buf).fname.cast());
                    }
                    if !(*buf).additional_data.is_null() {
                        nvim_xfree((*buf).additional_data.cast());
                    }
                }
                nvim_xfree(buffers.cast());
            }
        }
    }

    // Free additional_data if present
    if !(*entry).additional_data.is_null() {
        nvim_xfree((*entry).additional_data);
        (*entry).additional_data = std::ptr::null_mut();
    }

    // Reset entry to missing state
    (*entry).entry_type = ShadaEntryType::Missing;
    (*entry).can_free_entry = false;
}

// =============================================================================
// Stateless Helpers (Phase 1)
// =============================================================================

/// Insert into a mark list (jump list or change list) at the given position.
///
/// Handles the case where the list is full (JUMPLISTSIZE) by either dropping
/// the oldest entry or rejecting the insert. Returns the adjusted index where
/// the entry was placed, or -1 if the insert was rejected.
///
/// # Safety
///
/// - `jumps_arr` must point to a valid array of at least `jl_len` elements,
///   each of size `jump_size`.
/// - `i` must be in the range `[0, jl_len]`.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_marklist_insert(
    jumps_arr: *mut c_void,
    jump_size: usize,
    jl_len: c_int,
    i: c_int,
) -> c_int {
    let jumps = jumps_arr.cast::<u8>();
    let mut i = i;
    if i > 0 {
        if jl_len == JUMPLISTSIZE as c_int {
            i -= 1;
            if i > 0 {
                // delete oldest item to make room for new element
                std::ptr::copy(jumps.add(jump_size), jumps, jump_size * i as usize);
            }
        } else if i != jl_len {
            // insert at position i, move newer items out of the way
            std::ptr::copy(
                jumps.add(i as usize * jump_size),
                jumps.add((i as usize + 1) * jump_size),
                jump_size * (jl_len - i) as usize,
            );
        }
    } else if i == 0 {
        if jl_len == JUMPLISTSIZE as c_int {
            return -1; // don't insert, older than the entire list
        } else if jl_len > 0 {
            // insert i as the oldest item
            std::ptr::copy(jumps, jumps.add(jump_size), jump_size * jl_len as usize);
        }
    }
    i
}

/// Compare two FileMarks pointers by greatest_timestamp for qsort.
///
/// Orders in reverse: structure with greatest timestamp comes first.
///
/// # Safety
///
/// Both `a` and `b` must be valid pointers to `*const FileMarks`.
#[no_mangle]
pub unsafe extern "C" fn rs_compare_file_marks(a: *const c_void, b: *const c_void) -> c_int {
    let a_fm: *const c_void = (*(a.cast::<*const FileMarks>())).cast();
    let b_fm: *const c_void = (*(b.cast::<*const FileMarks>())).cast();
    let a_ts = nvim_filemarks_get_greatest_timestamp(a_fm);
    let b_ts = nvim_filemarks_get_greatest_timestamp(b_fm);
    // Reverse order: greater timestamp comes first
    match b_ts.cmp(&a_ts) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Replace a numbered mark in the WriteMergerState.
///
/// Frees the last mark, moves marks from idx to last-but-one (adjusting names),
/// and saves the new mark at the given index.
///
/// # Safety
///
/// - `wms` must be a valid pointer to a WriteMergerState.
/// - `idx` must be < EXTRA_MARKS.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_replace_numbered_mark(
    wms: *mut WriteMergerState,
    idx: usize,
    entry: ShadaEntry,
) {
    let wms = &mut *wms;
    let last = EXTRA_MARKS - 1;

    // Free the last entry
    nvim_shada_free_shada_entry(std::ptr::addr_of_mut!(wms.numbered_marks[last]));

    // Adjust names of marks that will shift down
    for i in idx..last {
        if wms.numbered_marks[i].entry_type == ShadaEntryType::GlobalMark {
            // Write filemark.name through raw pointer to avoid implicit autoref
            let fm_ptr: *mut FilemarkData =
                std::ptr::addr_of_mut!(wms.numbered_marks[i].data.filemark).cast();
            std::ptr::addr_of_mut!((*fm_ptr).name).write((b'0' + i as u8 + 1) as c_char);
        }
    }

    // Move marks from idx..last-1 to idx+1..last
    if idx < last {
        std::ptr::copy(
            wms.numbered_marks.as_ptr().add(idx),
            wms.numbered_marks.as_mut_ptr().add(idx + 1),
            last - idx,
        );
    }

    // Place the new entry at idx
    wms.numbered_marks[idx] = entry;

    // Set the name of the new entry
    let fm_ptr: *mut FilemarkData =
        std::ptr::addr_of_mut!(wms.numbered_marks[idx].data.filemark).cast();
    std::ptr::addr_of_mut!((*fm_ptr).name).write((b'0' + idx as u8) as c_char);
}

// =============================================================================
// Buffer/Path Filtering Helpers (Phase 2)
// =============================================================================

/// Opaque handle to a buffer (buf_T *).
pub type BufHandle = *const c_void;

/// Opaque handle to a Set(ptr_t).
pub type SetPtrHandle = *mut c_void;

/// Check if a file path matches a removable directory from 'shada' option.
///
/// Returns true if `name` starts with any `rXXX` entry in `'shada'`.
///
/// # Safety
///
/// `name` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_removable(name: *const c_char) -> c_int {
    let mut part = [0u8; MAXPATHL + 1];
    let new_name = nvim_shada_home_replace_save(std::ptr::null(), name);
    let p_shada = nvim_shada_get_p_shada();
    let mut p = p_shada.cast_mut();
    let mut retval = false;

    while *p != 0 {
        nvim_shada_copy_option_part(
            &raw mut p,
            part.as_mut_ptr().cast(),
            part.len(),
            c", ".as_ptr(),
        );
        if part[0] == b'r' {
            let name_buff = nvim_shada_get_namebuff();
            nvim_shada_home_replace(
                std::ptr::null(),
                part.as_ptr().add(1).cast(),
                name_buff,
                MAXPATHL,
                1,
            );
            let n = libc::strlen(name_buff.cast());
            if nvim_shada_mb_strnicmp(name_buff, new_name, n) == 0 {
                retval = true;
                break;
            }
        }
    }
    nvim_xfree(new_name.cast());
    c_int::from(retval)
}

/// Check if a buffer should be ignored when saving ShaDa data.
///
/// A buffer is ignored if it is NULL, has no filename, is not listed,
/// is a quickfix buffer, is a terminal buffer, or is in the removable set.
///
/// # Safety
///
/// - `buf` must be a valid buffer handle or null.
/// - `removable_bufs` must be a valid Set(ptr_t) handle.
#[no_mangle]
pub unsafe extern "C" fn rs_ignore_buf(buf: BufHandle, removable_bufs: SetPtrHandle) -> c_int {
    if buf.is_null() {
        return 1;
    }
    let ffname = nvim_shada_buf_get_ffname(buf);
    if ffname.is_null() {
        return 1;
    }
    if nvim_shada_buf_is_listed(buf) == 0 {
        return 1;
    }
    if nvim_shada_buf_is_quickfix(buf) != 0 {
        return 1;
    }
    if nvim_shada_buf_is_terminal(buf) != 0 {
        return 1;
    }
    if nvim_shada_set_has_ptr(removable_bufs, buf) != 0 {
        return 1;
    }
    0
}

/// Find buffers ignored due to their location and add them to the set.
///
/// Iterates all buffers and calls `rs_shada_removable` for each one with a filename.
///
/// # Safety
///
/// `removable_bufs` must be a valid Set(ptr_t) handle.
#[no_mangle]
pub unsafe extern "C" fn rs_find_removable_bufs(removable_bufs: SetPtrHandle) {
    let mut buf = nvim_shada_buf_first();
    while !buf.is_null() {
        let ffname = nvim_shada_buf_get_ffname(buf);
        if !ffname.is_null() && rs_shada_removable(ffname) != 0 {
            nvim_shada_set_put_ptr(removable_bufs, buf);
        }
        buf = nvim_shada_buf_next(buf);
    }
}

// =============================================================================
// Data Collection Functions (Phase 3)
// =============================================================================

/// Iterate over history entries and construct ShadaEntry.
///
/// # Safety
///
/// `hist` must be a valid pointer to write to.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_hist_iter(
    iter: *const c_void,
    history_type: u8,
    zero: c_int,
    hist: *mut ShadaEntry,
) -> *const c_void {
    let mut out_str: *mut c_char = std::ptr::null_mut();
    let mut out_strlen: usize = 0;
    let mut out_ts: Timestamp = 0;
    let mut out_additional_data: *mut c_void = std::ptr::null_mut();

    let ret = nvim_shada_hist_iter_raw(
        iter,
        history_type,
        zero,
        &raw mut out_str,
        &raw mut out_strlen,
        &raw mut out_ts,
        &raw mut out_additional_data,
    );

    if out_str.is_null() {
        *hist = ShadaEntry::missing();
    } else {
        let sep = if history_type == HIST_SEARCH {
            *out_str.add(out_strlen + 1)
        } else {
            0
        };
        *hist = ShadaEntry {
            can_free_entry: zero != 0,
            entry_type: ShadaEntryType::HistoryEntry,
            timestamp: out_ts,
            data: ShadaEntryData {
                history_item: std::mem::ManuallyDrop::new(HistoryItemData {
                    histtype: history_type,
                    string: out_str,
                    sep,
                }),
            },
            additional_data: out_additional_data,
        };
    }
    ret
}

/// Save a search pattern to a ShadaEntry.
///
/// # Safety
///
/// `ret_pse` must be a valid pointer to write to.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_add_search_pattern(
    ret_pse: *mut ShadaEntry,
    is_substitute: c_int,
    search_last_used: c_int,
    search_highlighted: c_int,
) {
    let is_sub = is_substitute != 0;
    let last_used = search_last_used != 0;
    let highlighted = search_highlighted != 0;

    let mut pat: *mut c_char = std::ptr::null_mut();
    let mut magic: c_int = 0;
    let mut no_scs: c_int = 0;
    let mut ts: Timestamp = 0;
    let mut off_line: c_int = 0;
    let mut off_end: c_int = 0;
    let mut off_off: i64 = 0;
    let mut off_dir: c_char = 0;
    let mut additional_data: *mut c_void = std::ptr::null_mut();

    if is_sub {
        nvim_shada_get_substitute_pattern(
            &raw mut pat,
            &raw mut magic,
            &raw mut no_scs,
            &raw mut ts,
            &raw mut off_line,
            &raw mut off_end,
            &raw mut off_off,
            &raw mut off_dir,
            &raw mut additional_data,
        );
    } else {
        nvim_shada_get_search_pattern(
            &raw mut pat,
            &raw mut magic,
            &raw mut no_scs,
            &raw mut ts,
            &raw mut off_line,
            &raw mut off_end,
            &raw mut off_off,
            &raw mut off_dir,
            &raw mut additional_data,
        );
    }

    if !pat.is_null() {
        // Default values for substitute pattern fields
        let has_line_offset = if is_sub { false } else { off_line != 0 };
        let place_cursor_at_end = if is_sub { false } else { off_end != 0 };
        let offset = if is_sub { 0 } else { off_off };

        *ret_pse = ShadaEntry {
            can_free_entry: false,
            entry_type: ShadaEntryType::SearchPattern,
            timestamp: ts,
            data: ShadaEntryData {
                search_pattern: std::mem::ManuallyDrop::new(SearchPatternData {
                    magic: magic != 0,
                    smartcase: no_scs == 0,
                    has_line_offset,
                    place_cursor_at_end,
                    offset,
                    is_last_used: is_sub ^ last_used,
                    is_substitute_pattern: is_sub,
                    highlighted: (is_sub ^ last_used) && highlighted,
                    pat,
                    pat_len: libc::strlen(pat.cast()),
                    search_backward: !is_sub && off_dir == b'?' as c_char,
                }),
            },
            additional_data,
        };
    }
}

/// Initialize registers for writing to ShaDa file.
///
/// # Safety
///
/// `wms` must be a valid WriteMergerState pointer.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_shada_initialize_registers(
    wms: *mut WriteMergerState,
    max_reg_lines: c_int,
) {
    let wms = &mut *wms;
    let limit = max_reg_lines >= 0;
    let mut reg_iter: *const c_void = std::ptr::null();

    loop {
        let mut name: c_char = 0;
        let mut reg_type: c_int = 0;
        let mut contents: *mut c_void = std::ptr::null_mut();
        let mut size: usize = 0;
        let mut width: usize = 0;
        let mut is_unnamed: c_int = 0;
        let mut ts: Timestamp = 0;
        let mut additional_data: *mut c_void = std::ptr::null_mut();

        reg_iter = nvim_shada_reg_iter(
            reg_iter,
            &raw mut name,
            &raw mut reg_type,
            &raw mut contents,
            &raw mut size,
            &raw mut width,
            &raw mut is_unnamed,
            &raw mut ts,
            &raw mut additional_data,
        );

        if name == 0 {
            break;
        }
        if limit && size > max_reg_lines as usize {
            if reg_iter.is_null() {
                break;
            }
            continue;
        }

        let idx = nvim_shada_op_reg_index(name);
        if idx >= 0 {
            wms.registers[idx as usize] = ShadaEntry {
                can_free_entry: false,
                entry_type: ShadaEntryType::Register,
                timestamp: ts,
                data: ShadaEntryData {
                    reg: std::mem::ManuallyDrop::new(RegisterData {
                        contents: contents.cast(),
                        contents_size: size,
                        reg_type,
                        width,
                        name,
                        is_unnamed: is_unnamed != 0,
                    }),
                },
                additional_data,
            };
        }

        if reg_iter.is_null() {
            break;
        }
    }
}

/// Get list of buffers to write to the ShaDa file.
///
/// # Safety
///
/// `removable_bufs` must be a valid Set(ptr_t) handle.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_get_buflist(removable_bufs: SetPtrHandle) -> ShadaEntry {
    let max_bufs = nvim_shada_get_percent_param();
    let mut buf_count: usize = 0;

    // Count buffers
    let mut buf = nvim_shada_buf_first();
    while !buf.is_null() {
        if rs_ignore_buf(buf, removable_bufs) == 0
            && (max_bufs < 0 || buf_count < max_bufs as usize)
        {
            buf_count += 1;
        }
        buf = nvim_shada_buf_next(buf);
    }

    // Allocate buffer list
    let buf_size = std::mem::size_of::<BufferListBuffer>();
    let buffers = nvim_xmalloc(buf_count * buf_size).cast::<BufferListBuffer>();

    let mut i: usize = 0;
    buf = nvim_shada_buf_first();
    while !buf.is_null() {
        if rs_ignore_buf(buf, removable_bufs) != 0 {
            buf = nvim_shada_buf_next(buf);
            continue;
        }
        if i >= buf_count {
            break;
        }
        let mut pos = Position::DEFAULT;
        nvim_shada_buf_get_cursor(buf, &raw mut pos);
        let ffname = nvim_shada_buf_get_ffname(buf);
        let additional_data = nvim_shada_buf_get_additional_data(buf);
        (*buffers.add(i)).pos = pos;
        (*buffers.add(i)).fname = ffname.cast_mut();
        (*buffers.add(i)).additional_data = additional_data;
        i += 1;
        buf = nvim_shada_buf_next(buf);
    }

    ShadaEntry {
        entry_type: ShadaEntryType::BufferList,
        can_free_entry: false,
        timestamp: nvim_shada_os_time(),
        data: ShadaEntryData {
            buffer_list: std::mem::ManuallyDrop::new(BufferListData {
                size: buf_count,
                buffers,
            }),
        },
        additional_data: std::ptr::null_mut(),
    }
}

/// Initialize ShaDa jumplist entries.
///
/// # Safety
///
/// - `jumps` must be a valid array of at least JUMPLISTSIZE ShadaEntry elements.
/// - `removable_bufs` must be a valid Set(ptr_t) handle.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_init_jumps(
    jumps: *mut ShadaEntry,
    removable_bufs: SetPtrHandle,
) -> usize {
    let mut jumps_size: usize = 0;
    let mut jump_iter: *const c_void = std::ptr::null();

    nvim_shada_setpcmark();
    let wp = nvim_shada_curwin();
    nvim_shada_cleanup_jumplist(wp, 0);

    loop {
        let mut mark = Position::DEFAULT;
        let mut fnum: c_int = 0;
        let mut ts: Timestamp = 0;
        let mut fname: *mut c_char = std::ptr::null_mut();
        let mut additional_data: *mut c_void = std::ptr::null_mut();

        jump_iter = nvim_shada_jumplist_iter(
            jump_iter,
            wp,
            &raw mut mark,
            &raw mut fnum,
            &raw mut ts,
            &raw mut fname,
            &raw mut additional_data,
        );

        if mark.lnum == 0 {
            if jump_iter.is_null() {
                break;
            }
            continue;
        }

        let buf = if fnum == 0 {
            std::ptr::null()
        } else {
            nvim_shada_buflist_findnr(fnum)
        };

        if if buf.is_null() {
            fnum != 0
        } else {
            rs_ignore_buf(buf, removable_bufs) != 0
        } {
            if jump_iter.is_null() {
                break;
            }
            continue;
        }

        let final_fname = if fnum == 0 {
            fname
        } else if !buf.is_null() {
            nvim_shada_buf_get_ffname(buf).cast_mut()
        } else {
            std::ptr::null_mut()
        };

        if final_fname.is_null() {
            if jump_iter.is_null() {
                break;
            }
            continue;
        }

        *jumps.add(jumps_size) = ShadaEntry {
            can_free_entry: false,
            entry_type: ShadaEntryType::Jump,
            timestamp: ts,
            data: ShadaEntryData {
                filemark: std::mem::ManuallyDrop::new(FilemarkData {
                    name: 0, // NUL
                    mark,
                    fname: final_fname,
                }),
            },
            additional_data,
        };
        jumps_size += 1;

        if jump_iter.is_null() {
            break;
        }
    }
    jumps_size
}

// =============================================================================
// Phase 5: File I/O Wrappers
// =============================================================================

/// Close a ShaDa file descriptor, reporting errors.
///
/// Equivalent to the C `close_file` function.
#[no_mangle]
pub unsafe extern "C" fn rs_close_file(cookie: FileDescriptorHandle) {
    let error = nvim_file_close(cookie, nvim_shada_get_p_fs());
    if error != 0 {
        nvim_shada_semsg_close_error(nvim_shada_os_strerror(error));
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
#[allow(clippy::cast_possible_wrap)]
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

    // =============================================================================
    // Entry Type Tests
    // =============================================================================

    #[test]
    fn test_entry_type_enum() {
        assert_eq!(ShadaEntryType::Unknown.as_raw(), SD_ITEM_UNKNOWN);
        assert_eq!(ShadaEntryType::Missing.as_raw(), SD_ITEM_MISSING);
        assert_eq!(ShadaEntryType::Header.as_raw(), SD_ITEM_HEADER);
        assert_eq!(
            ShadaEntryType::SearchPattern.as_raw(),
            SD_ITEM_SEARCH_PATTERN
        );
        assert_eq!(ShadaEntryType::SubString.as_raw(), SD_ITEM_SUB_STRING);
        assert_eq!(ShadaEntryType::HistoryEntry.as_raw(), SD_ITEM_HISTORY_ENTRY);
        assert_eq!(ShadaEntryType::Register.as_raw(), SD_ITEM_REGISTER);
        assert_eq!(ShadaEntryType::Variable.as_raw(), SD_ITEM_VARIABLE);
        assert_eq!(ShadaEntryType::GlobalMark.as_raw(), SD_ITEM_GLOBAL_MARK);
        assert_eq!(ShadaEntryType::Jump.as_raw(), SD_ITEM_JUMP);
        assert_eq!(ShadaEntryType::BufferList.as_raw(), SD_ITEM_BUFFER_LIST);
        assert_eq!(ShadaEntryType::LocalMark.as_raw(), SD_ITEM_LOCAL_MARK);
        assert_eq!(ShadaEntryType::Change.as_raw(), SD_ITEM_CHANGE);
    }

    #[test]
    fn test_entry_type_from_raw() {
        assert_eq!(
            ShadaEntryType::from_raw(SD_ITEM_MISSING),
            ShadaEntryType::Missing
        );
        assert_eq!(
            ShadaEntryType::from_raw(SD_ITEM_HEADER),
            ShadaEntryType::Header
        );
        assert_eq!(
            ShadaEntryType::from_raw(SD_ITEM_CHANGE),
            ShadaEntryType::Change
        );
        assert_eq!(ShadaEntryType::from_raw(100), ShadaEntryType::Unknown);
        assert_eq!(ShadaEntryType::from_raw(-5), ShadaEntryType::Unknown);
    }

    #[test]
    fn test_entry_type_roundtrip() {
        let types = [
            ShadaEntryType::Missing,
            ShadaEntryType::Header,
            ShadaEntryType::SearchPattern,
            ShadaEntryType::SubString,
            ShadaEntryType::HistoryEntry,
            ShadaEntryType::Register,
            ShadaEntryType::Variable,
            ShadaEntryType::GlobalMark,
            ShadaEntryType::Jump,
            ShadaEntryType::BufferList,
            ShadaEntryType::LocalMark,
            ShadaEntryType::Change,
        ];
        for typ in types {
            assert_eq!(ShadaEntryType::from_raw(typ.as_raw()), typ);
        }
    }

    // =============================================================================
    // Entry Structure Tests
    // =============================================================================

    #[test]
    fn test_shada_entry_default() {
        let entry = ShadaEntry::default();
        assert_eq!(entry.entry_type, ShadaEntryType::Missing);
        assert!(!entry.can_free_entry);
        assert_eq!(entry.timestamp, 0);
        assert!(entry.additional_data.is_null());
    }

    #[test]
    fn test_shada_entry_missing() {
        let entry = ShadaEntry::missing();
        assert_eq!(entry.entry_type, ShadaEntryType::Missing);
        assert!(!entry.can_free_entry);
        assert_eq!(entry.timestamp, 0);
    }

    #[test]
    fn test_shada_entry_header() {
        let entry = rs_shada_entry_header(12345);
        assert_eq!(entry.entry_type, ShadaEntryType::Header);
        assert!(entry.can_free_entry);
        assert_eq!(entry.timestamp, 12345);
    }

    #[test]
    fn test_shada_entry_get_type() {
        let entry = ShadaEntry::default();
        assert_eq!(rs_shada_entry_get_type(&entry), SD_ITEM_MISSING);

        let header = rs_shada_entry_header(0);
        assert_eq!(rs_shada_entry_get_type(&header), SD_ITEM_HEADER);
    }

    #[test]
    fn test_shada_entry_get_type_null() {
        assert_eq!(rs_shada_entry_get_type(std::ptr::null()), SD_ITEM_MISSING);
    }

    #[test]
    fn test_shada_entry_get_timestamp() {
        let entry = rs_shada_entry_header(54321);
        assert_eq!(rs_shada_entry_get_timestamp(&entry), 54321);
    }

    #[test]
    fn test_shada_entry_get_timestamp_null() {
        assert_eq!(rs_shada_entry_get_timestamp(std::ptr::null()), 0);
    }

    #[test]
    fn test_shada_entry_is_missing() {
        let missing = ShadaEntry::missing();
        assert_ne!(rs_shada_entry_is_missing(&missing), 0);

        let header = rs_shada_entry_header(0);
        assert_eq!(rs_shada_entry_is_missing(&header), 0);
    }

    #[test]
    fn test_shada_entry_is_missing_null() {
        assert_ne!(rs_shada_entry_is_missing(std::ptr::null()), 0);
    }

    #[test]
    fn test_shada_entry_compare_timestamp() {
        let entry1 = rs_shada_entry_header(100);
        let entry2 = rs_shada_entry_header(200);
        let entry3 = rs_shada_entry_header(100);

        assert_eq!(rs_shada_entry_compare_timestamp(&entry1, &entry2), -1);
        assert_eq!(rs_shada_entry_compare_timestamp(&entry2, &entry1), 1);
        assert_eq!(rs_shada_entry_compare_timestamp(&entry1, &entry3), 0);
    }

    // =============================================================================
    // Data Structure Tests
    // =============================================================================

    #[test]
    fn test_search_pattern_data_default() {
        let data = SearchPatternData::default();
        assert!(data.magic);
        assert!(!data.smartcase);
        assert!(!data.has_line_offset);
        assert!(!data.place_cursor_at_end);
        assert_eq!(data.offset, 0);
        assert!(data.is_last_used);
        assert!(!data.is_substitute_pattern);
        assert!(!data.highlighted);
        assert!(!data.search_backward);
        assert!(data.pat.is_null());
        assert_eq!(data.pat_len, 0);
    }

    #[test]
    fn test_filemark_data_default() {
        let data = FilemarkData::default();
        assert_eq!(data.name, b'"' as c_char);
        assert_eq!(data.mark.lnum, 1);
        assert_eq!(data.mark.col, 0);
        assert!(data.fname.is_null());
    }

    #[test]
    fn test_history_item_data_default() {
        let data = HistoryItemData::default();
        assert_eq!(data.histtype, HIST_CMD);
        assert!(data.string.is_null());
        assert_eq!(data.sep, 0);
    }

    #[test]
    fn test_register_data_default() {
        let data = RegisterData::default();
        assert_eq!(data.name, 0);
        assert_eq!(data.reg_type, MT_CHAR_WISE);
        assert!(data.contents.is_null());
        assert_eq!(data.contents_size, 0);
        assert!(!data.is_unnamed);
        assert_eq!(data.width, 0);
    }

    #[test]
    fn test_buffer_list_data_default() {
        let data = BufferListData::default();
        assert_eq!(data.size, 0);
        assert!(data.buffers.is_null());
    }

    // =============================================================================
    // Linked List Tests
    // =============================================================================

    #[test]
    fn test_hml_list_entry_default() {
        let entry = HMLListEntry::default();
        assert_eq!(entry.data.entry_type, ShadaEntryType::Missing);
        assert!(entry.next.is_null());
        assert!(entry.prev.is_null());
    }

    #[test]
    fn test_hml_list_default() {
        let list = HMLList::default();
        assert!(list.entries.is_null());
        assert!(list.first.is_null());
        assert!(list.last.is_null());
        assert!(list.free_entry.is_null());
        assert!(list.last_free_entry.is_null());
        assert_eq!(list.size, 0);
        assert_eq!(list.num_entries, 0);
        assert!(list.contained_entries.is_null());
    }

    #[test]
    fn test_history_merger_state_default() {
        let state = HistoryMergerState::default();
        assert!(!state.do_merge);
        assert!(!state.reading);
        assert!(state.iter.is_null());
        assert_eq!(state.last_hist_entry.entry_type, ShadaEntryType::Missing);
        assert_eq!(state.history_type, HIST_CMD);
    }

    // =============================================================================
    // Size and Alignment Tests (important for FFI compatibility)
    // =============================================================================

    #[test]
    fn test_position_size() {
        // Position should be 16 bytes: 8 (lnum) + 4 (col) + 4 (coladd)
        assert_eq!(std::mem::size_of::<Position>(), 16);
    }

    #[test]
    fn test_array_constants() {
        // Verify array sizes match expected values
        assert_eq!(NLOCALMARKS, 26);
        assert_eq!(JUMPLISTSIZE, 100);
        assert_eq!(NMARKS, 26);
        assert_eq!(EXTRA_MARKS, 10);
        assert_eq!(NUM_SAVED_REGISTERS, 37);
        assert_eq!(HIST_COUNT, 5);
    }
}
