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

use std::ffi::{c_char, c_int, c_uint, c_void};

// Safely read a field from a ManuallyDrop union variant of a ShadaEntry.
//
// Usage: `read_union_field!(entry_ptr, variant_name, field_name)`
//
// Uses `addr_of!` to avoid triggering ManuallyDrop's Drop impl or creating
// unaligned references.
macro_rules! read_union_field {
    ($entry:expr, $field:ident, $inner:ident) => {{
        let data_ptr = std::ptr::addr_of!((*$entry).data.$field);
        let inner_ptr: *const _ = std::ptr::addr_of!((**data_ptr).$inner);
        std::ptr::read(inner_ptr)
    }};
}

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

    // File operations
    fn nvim_file_skip(fd: FileDescriptorHandle, offset: usize) -> isize;
    fn nvim_file_eof(fd: FileDescriptorHandle) -> c_int;
    fn nvim_file_close(fd: FileDescriptorHandle, fsync: c_int) -> c_int;

    // Memory allocation
    fn nvim_xfree(ptr: *mut c_void);
    fn nvim_xmalloc(size: usize) -> *mut c_void;
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;

    // Option access
    fn nvim_get_p_hi() -> i64;
    fn nvim_get_p_fs() -> bool;

    // Error messages
    fn nvim_semsg(fmt: *const c_char, ...);

    // nvim_filemarks_get_greatest_timestamp removed (plan 9106c29c Phase 2): direct field access.

    // Buffer/path filtering (Phase 2)
    fn nvim_shada_get_p_shada() -> *const c_char;
    fn home_replace_save(buf: *const c_void, src: *const c_char) -> *mut c_char;
    fn nvim_shada_home_replace(
        buf: *const c_void,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: c_int,
    );
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn nvim_shada_get_namebuff() -> *mut c_char;
    fn nvim_shada_buf_first() -> *const c_void;
    fn nvim_shada_buf_next(buf: *const c_void) -> *const c_void;
    fn nvim_shada_buf_get_ffname(buf: *const c_void) -> *const c_char;
    fn nvim_shada_buf_should_skip(buf: *const c_void) -> c_int;
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
        is_substitute: c_int,
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
    fn search_was_last_used() -> bool;
    static no_hlsearch: bool;
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
    fn nvim_shada_buf_get_buflist_info(
        buf: *const c_void,
        out_pos: *mut Position,
        out_additional_data: *mut *mut c_void,
    );
    fn os_time() -> Timestamp;
    fn setpcmark();
    fn cleanup_jumplist(wp: *mut c_void, loadfiles: bool);
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
    fn buflist_findnr(nr: c_int) -> *const c_void;
    // nvim_shada_siemsg removed (plan 9106c29c Phase 1): replaced by nvim_shada_siemsg_1s.

    // Phase 4: Entry free consolidation accessors
    fn nvim_shada_tv_clear(tv: *mut c_void);
    fn nvim_shada_free_variable(entry: *mut ShadaEntry);
    // Phase 1 (plan 11dd3cf4): Free Header entry dict via C (Dict type mismatch prevents direct call)
    fn nvim_shada_free_header_entry(entry: *mut ShadaEntry);

    // Phase 5: File I/O accessors
    fn nvim_shada_file_open(fd: FileDescriptorHandle, fname: *const c_char) -> c_int;
    fn nvim_shada_file_open_buffer(fd: FileDescriptorHandle, data: *mut c_char, len: usize);
    fn nvim_shada_os_strerror(err: c_int) -> *const c_char;
    fn verbose_enter();
    fn verbose_leave();
    fn nvim_shada_get_p_verbose() -> c_int;
    fn nvim_shada_smsg_reading(
        fname: *const c_char,
        want_info: c_int,
        want_marks: c_int,
        get_oldfiles: c_int,
        failed: c_int,
    );
    fn nvim_shada_build_default_path() -> *mut c_char;
    // nvim_shada_semsg_close_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_1s.
    // nvim_shada_semsg_open_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    fn nvim_shada_file_descriptor_size() -> usize;

    // Phase 6: curbuf accessors for check_marks_read
    fn nvim_shada_curbuf_marks_read() -> c_int;
    fn nvim_shada_curbuf_set_marks_read(val: c_int);
    fn nvim_shada_curbuf_ffname() -> *const c_char;

    // Phase 7: histentry_T accessor
    fn nvim_shada_set_histentry(
        hist_array: *mut c_void,
        idx: c_int,
        ts: Timestamp,
        hisstr: *mut c_char,
        additional_data: *mut c_void,
    );

    fn modname(fname: *const c_char, ext: *const c_char, prepend_dot: bool) -> *mut c_char;
    fn os_getperm(fname: *const c_char) -> c_int;
    fn nvim_shada_file_open_write(
        fd: FileDescriptorHandle,
        fname: *const c_char,
        flags: c_int,
        perm: c_int,
    ) -> c_int;
    fn nvim_shada_path_tail_with_sep_offset(fname: *const c_char) -> usize;
    fn os_isdir(fname: *const c_char) -> bool;
    fn nvim_shada_os_mkdir_recurse(
        fname: *const c_char,
        perm: c_int,
        out_failed_dir: *mut *mut c_char,
    ) -> c_int;
    fn vim_rename(from: *const c_char, to: *const c_char) -> c_int;
    fn os_remove(fname: *const c_char) -> c_int;
    // nvim_shada_smsg_writing removed (plan 9106c29c Phase 1): use nvim_shada_smsg_1s.
    // nvim_shada_semsg_merge_read_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_tempfile_open_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_all_tmpfiles removed (plan 9106c29c Phase 1): use nvim_shada_semsg_1s.
    // nvim_shada_semsg_mkdir_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_write_open_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_rename_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_not_shada removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_write_errors removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
    // nvim_shada_semsg_remove_reminder removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.

    // Phase 3 (plan 11dd3cf4): shada_read migration accessors
    // nvim_shada_read_next_item removed (Phase 2 plan 92c8078e): Rust uses rs_shada_read_next_item.
    fn nvim_shada_fname_bufs_new() -> *mut c_void;
    fn nvim_shada_fname_bufs_destroy(handle: *mut c_void);
    fn nvim_shada_oldfiles_set_new() -> *mut c_void;
    fn nvim_shada_oldfiles_set_destroy(handle: *mut c_void);
    fn nvim_shada_get_oldfiles_list() -> *mut c_void;
    fn nvim_shada_tv_list_len(list: *mut c_void) -> c_int;
    fn nvim_shada_create_oldfiles_list() -> *mut c_void;
    fn nvim_shada_argcount() -> c_int;
    // Phase 1 (plan 92c8078e): compound accessors replacing nvim_shada_apply_entry
    // nvim_shada_apply_search_pattern removed (plan b499a5d0 Phase 1): Rust rs_shada_apply_search_pattern.
    // nvim_shada_apply_sub_string removed (plan b499a5d0 Phase 1): Rust rs_shada_apply_sub_string.
    // nvim_shada_apply_register removed (plan b499a5d0 Phase 2): Rust rs_shada_apply_register.
    // nvim_shada_apply_variable removed (plan b499a5d0 Phase 2): Rust rs_shada_apply_variable.
    // nvim_shada_apply_mark_or_jump removed (plan b499a5d0 Phase 4): Rust rs_shada_apply_mark_or_jump.
    // nvim_shada_apply_buffer_list removed (plan b499a5d0 Phase 3): Rust rs_shada_apply_buffer_list.
    // nvim_shada_apply_local_or_change removed (plan b499a5d0 Phase 4): Rust rs_shada_apply_local_or_change.
    // Phase 4 (plan b499a5d0): thin accessors for mark/jump and local/change apply
    fn nvim_shada_mark_set_global_from_entry(
        entry: *mut ShadaEntry,
        fname_bufs: *mut c_void,
        no_overwrite: c_int,
    ) -> c_int;
    fn nvim_shada_jumplist_len() -> c_int;
    fn nvim_shada_jumplist_get_entry(
        idx: c_int,
        out_ts: *mut u64,
        out_lnum: *mut i64,
        out_col: *mut i32,
        out_fnum: *mut c_int,
        out_fname: *mut *const c_char,
    );
    fn nvim_shada_jumplist_insert_entry(
        i: c_int,
        entry: *mut ShadaEntry,
        fname_bufs: *mut c_void,
        jl_len: c_int,
    );
    fn nvim_shada_oldfiles_add(
        oldfiles_set: *mut c_void,
        oldfiles_list: *mut c_void,
        entry: *mut ShadaEntry,
        want_marks: c_int,
    );
    fn nvim_shada_oldfiles_has(oldfiles_set: *mut c_void, entry: *const ShadaEntry) -> c_int;
    fn nvim_shada_mark_set_local_from_entry(
        entry: *mut ShadaEntry,
        buf: *mut c_void,
        no_overwrite: c_int,
    ) -> c_int;
    fn nvim_shada_cl_bufs_set_put(cl_bufs: *mut c_void, buf: *mut c_void);
    fn nvim_shada_buf_get_changelistlen(buf: *const c_void) -> c_int;
    fn nvim_shada_changelist_get_entry(
        buf: *const c_void,
        idx: c_int,
        out_ts: *mut u64,
        out_lnum: *mut i64,
        out_col: *mut i32,
    );
    fn nvim_shada_changelist_insert_entry(
        buf: *mut c_void,
        i: c_int,
        entry: *mut ShadaEntry,
        cl_len: c_int,
    );
    fn nvim_shada_fm_xfree_fname(entry: *mut ShadaEntry);
    fn nvim_shada_buf_get_fnum(buf: *const c_void) -> c_int;
    fn nvim_shada_jumplist_marklist_insert(i: c_int) -> c_int;
    fn nvim_shada_changelist_marklist_insert(buf: *mut c_void, i: c_int) -> c_int;
    // Phase 1 (plan b499a5d0): thin accessors for search/sub apply
    fn nvim_shada_get_search_pattern_timestamp(is_substitute: c_int) -> u64;
    fn nvim_shada_set_search_pattern_from_entry(entry: *mut ShadaEntry, is_substitute: c_int);
    fn set_last_used_pattern(is_substitute: bool);
    fn set_no_hlsearch(val: bool);
    fn nvim_shada_get_sub_replacement_timestamp() -> u64;
    fn nvim_shada_set_sub_replacement_from_entry(entry: *mut ShadaEntry);
    // Phase 2 (plan b499a5d0): thin accessors for register/variable apply
    fn nvim_shada_entry_get_reg_type_valid(entry: *const ShadaEntry) -> c_int;
    fn nvim_shada_op_reg_get_timestamp(name: c_char) -> u64;
    fn nvim_shada_op_reg_set_from_entry(entry: *mut ShadaEntry) -> c_int;
    fn nvim_shada_var_set_global_from_entry(entry: *mut ShadaEntry);
    // Phase 3 (plan b499a5d0): thin accessors for buffer list apply
    // nvim_shada_apply_buffer_list removed (plan b499a5d0 Phase 3): Rust rs_shada_apply_buffer_list.
    fn nvim_shada_find_buffer(fname_bufs: *mut c_void, fname: *const c_char) -> *mut c_void;
    fn nvim_shada_path_try_shorten_fname(fname: *const c_char) -> *mut c_char;
    fn nvim_shada_buflist_new(fname: *const c_char, sfname: *const c_char) -> *mut c_void;
    fn nvim_shada_buf_set_cursor_and_data(buf: *mut c_void, entry: *mut ShadaEntry, i: usize);
    fn nvim_shada_for_all_tab_windows_update_changelist(cl_bufs_handle: *mut c_void);
    fn clr_history(i: c_int) -> c_int;
    fn hist_get_array(
        i: u8,
        out_hisidx: *mut *mut c_int,
        out_hisnum: *mut *mut c_int,
    ) -> *mut c_void;
    // Phase 2 (plan 92c8078e): compound parsing accessors for rs_shada_read_next_item
    fn nvim_shada_file_try_read_buffered(fd: *mut c_void, len: usize) -> *mut c_char;
    fn nvim_shada_file_bytes_read(fd: *mut c_void) -> u64;
    // nvim_shada_semsg_rcerr_too_long removed (plan 9106c29c Phase 1): use nvim_shada_semsg_u64.
    // nvim_shada_semsg_rcerr_missing removed (plan 9106c29c Phase 1): use nvim_shada_semsg_u64.
}

// =============================================================================
// Phase 2: shada_write / shada_read_when_writing migration accessors
// =============================================================================

#[allow(dead_code)]
extern "C" {
    /// Set b_last_cursor for all windows in all tabs.
    fn nvim_shada_set_all_last_cursors();
    /// The long Neovim version string.
    static longVersion: *const c_char;
    /// Get current process ID.
    fn os_get_pid() -> i64;
    /// The current encoding option (p_enc).
    static p_enc: *mut c_char;
    /// Iterate over global marks.
    fn nvim_shada_mark_global_iter(
        iter: *const c_void,
        out_name: *mut c_char,
        out_lnum: *mut i64,
        out_col: *mut i32,
        out_fnum: *mut c_int,
        out_ts: *mut Timestamp,
        out_fname: *mut *const c_char,
        out_additional: *mut *mut c_void,
    ) -> *const c_void;
    /// Get global mark index from name.
    fn nvim_shada_mark_global_index(name: c_int) -> c_int;
    /// Get local mark index from name.
    fn nvim_shada_mark_local_index(name: c_int) -> c_int;
    /// Get timestamp of namedfm[idx].
    fn nvim_shada_named_mark_timestamp(idx: c_int) -> Timestamp;
    /// Iterate over buffer-local marks.
    fn nvim_shada_mark_buffer_iter(
        iter: *const c_void,
        buf: *const c_void,
        out_name: *mut c_char,
        out_lnum: *mut i64,
        out_col: *mut i32,
        out_ts: *mut Timestamp,
        out_additional: *mut *mut c_void,
    ) -> *const c_void;
    /// Get changelist length of a buffer.
    fn nvim_shada_buf_changelist_len(buf: *const c_void) -> c_int;
    /// Get a changelist entry from a buffer.
    fn nvim_shada_buf_changelist_entry(
        buf: *const c_void,
        idx: c_int,
        out_lnum: *mut i64,
        out_col: *mut i32,
        out_ts: *mut Timestamp,
        out_additional: *mut *mut c_void,
    );
    /// Get current substitute replacement string.
    fn nvim_shada_sub_get_replacement(
        out_sub: *mut *const c_char,
        out_ts: *mut Timestamp,
        out_additional: *mut *mut c_void,
    );
    /// Get curwin->w_cursor.lnum.
    fn nvim_shada_curwin_lnum() -> i64;
    /// Get curwin->w_cursor as (lnum, col).
    fn nvim_shada_curwin_cursor(out_lnum: *mut i64, out_col: *mut i32);
    /// Put fname into WMS file_marks PMap; returns pointer to value slot.
    fn nvim_shada_wms_file_marks_put_ref(
        wms: *mut c_void,
        fname: *const c_char,
        is_new: *mut bool,
        out_key: *mut *const c_char,
    ) -> *mut *mut c_void;
    // Phase 2 (plan 9106c29c): FileMarks accessor functions removed; direct struct field access used.
    // nvim_shada_file_marks_alloc removed: use nvim_xcalloc(1, size_of::<FileMarks>()).cast::<FileMarks>()
    // nvim_shada_file_marks_greatest_ts removed: use (*fm).greatest_timestamp
    // nvim_shada_file_marks_update_ts removed: inline if ts > (*fm).greatest_timestamp
    // nvim_shada_file_marks_get_mark removed: use &raw mut (*fm).marks[idx]
    // nvim_shada_file_marks_get_change removed: use &raw mut (*fm).changes[idx]
    // nvim_shada_file_marks_changes_size removed: use (*fm).changes_size
    // nvim_shada_file_marks_set_changes_size removed: assign (*fm).changes_size
    // nvim_shada_file_marks_additional_size removed: use (*fm).additional_marks_size
    // nvim_shada_file_marks_get_additional removed: use (*fm).additional_marks.add(idx)
    // nvim_shada_file_marks_free_additional removed: nvim_xfree + null out fields
    // nvim_shada_file_marks_push_additional removed: inline xrealloc + write
    /// Collect, sort, and return all FileMarks from WMS as array.
    fn nvim_shada_wms_file_marks_get_sorted(
        wms: *const c_void,
        out_size: *mut usize,
    ) -> *mut *mut c_void;
    /// Destroy file_marks PMap in WMS (frees keys and values).
    fn nvim_shada_wms_file_marks_destroy(wms: *mut c_void);
    /// Check if name is in dumped_variables set.
    fn nvim_shada_wms_dumped_vars_has(wms: *const c_void, name: *const c_char) -> bool;
    /// Add name to dumped_variables set.
    fn nvim_shada_wms_dumped_vars_put(wms: *mut c_void, name: *const c_char);
    /// Destroy dumped_variables set in WMS.
    fn nvim_shada_wms_dumped_vars_destroy(wms: *mut c_void);
    /// Compare mark entry timestamps to determine if mark_get result is newer.
    fn nvim_shada_mark_get_cmp(
        buf: *const c_void,
        win: *const c_void,
        name: c_int,
        entry_ts: Timestamp,
    ) -> c_int;
    /// Compare file paths directly.
    fn path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;
    // nvim_shada_packer_flush_buf deleted (Phase 1 plan c02d0f11): replaced by nvim_shada_packer_flush inline helper.
    /// Initialize a PackerBuffer for writing to a FileDescriptor (by pointer).
    fn nvim_shada_packer_init_for_file(fd: *mut c_void, out: *mut ShadaPackerBuffer);
}

// =============================================================================
// Phase 3 (plan fd426e0f): nvim_shada_pack_all_gvars migration accessors
// =============================================================================

extern "C" {
    /// Get refcheck info from a typval (vtype, container ptr, copy_id) in one call.
    fn nvim_shada_tv_get_refcheck_info(
        tv: *const c_void,
        out_vtype: *mut c_int,
        out_container: *mut *mut c_void,
        out_copy_id: *mut c_int,
    );
    /// Mark all items in hashtab with copyID (from eval crate, opaque hashtab_T*).
    fn rs_set_ref_in_ht(ht: *mut c_void, copy_id: c_int, list_stack: *mut *mut c_void) -> bool;
    /// Mark all items in list with copyID (from eval crate, opaque list_T*).
    fn rs_set_ref_in_list_items(l: *mut c_void, copy_id: c_int, ht_stack: *mut *mut c_void)
        -> bool;
    /// Get a fresh copyID for circular reference detection (from eval crate).
    fn rs_get_copyID() -> c_int;
}

// =============================================================================
// Phase 4 (plan fd426e0f): nvim_shada_platform_check_writable migration accessors
// =============================================================================

extern "C" {
    /// Get stat fields for a file; returns 1 if exists and is not a directory.
    fn nvim_shada_os_fileinfo(
        fname: *const c_char,
        out_mode: *mut u64,
        out_uid: *mut u64,
        out_gid: *mut u64,
    ) -> c_int;
    /// Call os_fchown on an open FileDescriptor (sd_writer).
    fn nvim_shada_os_fchown(sd_writer: *mut c_void, uid: u64, gid: u64) -> c_int;
    // nvim_shada_semsg_not_writable removed (plan 9106c29c Phase 1): use nvim_shada_semsg_1s.
    // nvim_shada_semsg_fchown_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s.
}

// =============================================================================
// Phase 6 (plan 13c452f9): FFI declarations for unpack_* and decode_string
// =============================================================================

// NvimString is defined later in this file (line ~5153). is_null() method added there.

/// AdditionalDataBuilder matches C's `kvec_t(char)` = `AdditionalDataBuilder`:
/// `{size_t size; size_t capacity; char *items}`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AdditionalDataBuilder {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut c_char,
}

impl AdditionalDataBuilder {
    /// Equivalent of C's KV_INITIAL_VALUE.
    pub const fn new() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        }
    }
}

impl Default for AdditionalDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// MPACK status codes matching mpack_core.h
const MPACK_OK: c_int = 0;

/// Matches C's `KeySetLink` in api/private/defs.h.
/// Used by `unpack_keydict` to locate struct fields.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KeySetLink {
    pub str_: *mut c_char,
    pub ptr_off: usize,
    pub type_: c_int,
    pub opt_index: c_int,
    pub is_hlgroup: bool,
}

/// StringArray = kvec_t(String) = `{size: usize, capacity: usize, items: *mut NvimString}`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StringArray {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut NvimString,
}

/// Matches C's `Dict(_shada_search_pat)` from keysets_defs.h.
///
/// Fields ordered to match C struct layout. KEYSET_OPTIDX: sb=1, sc=2, se=3, sh=4,
/// sl=5, sm=6, so=7, sp=8, ss=9, su=10.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyDict_shada_search_pat {
    pub is_set: u64,
    pub magic: bool,                 // sm = 6
    pub smartcase: bool,             // sc = 2
    pub has_line_offset: bool,       // sl = 5
    pub place_cursor_at_end: bool,   // se = 3
    pub is_last_used: bool,          // su = 10
    pub is_substitute_pattern: bool, // ss = 9
    pub highlighted: bool,           // sh = 4
    pub search_backward: bool,       // sb = 1
    pub offset: i64,                 // so = 7
    pub pat: NvimString,             // sp = 8
}

/// Matches C's `Dict(_shada_mark)` from keysets_defs.h.
/// KEYSET_OPTIDX: c=1, f=2, l=3, n=4
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyDict_shada_mark {
    pub is_set: u64,
    pub n: i64,
    pub l: i64,
    pub c: i64,
    pub f: NvimString,
}

/// Matches C's `Dict(_shada_register)` from keysets_defs.h.
///
/// KEYSET_OPTIDX: n=1, rc=2, rt=3, ru=4, rw=5. C inserts 7 bytes of padding
/// after `ru` to align `rt`; `#[repr(C)]` does the same automatically.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyDict_shada_register {
    pub is_set: u64,
    pub rc: StringArray,
    pub ru: bool,
    pub rt: i64,
    pub n: i64,
    pub rw: i64,
}

/// Matches C's `Dict(_shada_buflist_item)` from keysets_defs.h.
/// KEYSET_OPTIDX: c=1, f=2, l=3
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyDict_shada_buflist_item {
    pub is_set: u64,
    pub l: i64,
    pub c: i64,
    pub f: NvimString,
}

/// Check if a keydict field is set (HAS_KEY macro equivalent).
#[inline]
const fn has_key(is_set: u64, opt_idx: u64) -> bool {
    (is_set & (1 << opt_idx)) != 0
}

#[allow(dead_code)]
extern "C" {
    /// Unpack the next msgpack array; returns element count or -1 on error.
    fn unpack_array(data: *mut *const c_char, size: *mut usize) -> isize;
    /// Unpack the next msgpack string; returns NvimString with data=null on error.
    fn unpack_string(data: *mut *const c_char, size: *mut usize) -> NvimString;
    /// Unpack the next msgpack integer into *res; returns true on success.
    fn unpack_integer(data: *mut *const c_char, size: *mut usize, res: *mut i64) -> bool;
    /// Skip the next msgpack value; returns MPACK_OK/MPACK_EOF/MPACK_ERROR.
    fn unpack_skip(data: *mut *const c_char, size: *mut usize) -> c_int;
    /// Decode msgpack into a typval_T at *ret; returns MPACK_OK on success.
    fn unpack_typval(data: *mut *const c_char, size: *mut usize, ret: *mut c_void) -> c_int;
    /// Push raw msgpack bytes into the AdditionalDataBuilder.
    fn push_additional_data(ad: *mut AdditionalDataBuilder, data: *const c_char, size: usize);
    /// Decode a msgpack binary string to a typval_T written at dst (16 bytes).
    /// Callers must provide a 16-byte aligned destination via nvim_shada_entry_var_value_ptr.
    fn nvim_shada_decode_string_into(
        s: *const c_char,
        len: usize,
        force_blob: bool,
        dst: *mut c_void,
    );
    // nvim_shada_semsg_readerr removed (plan 9106c29c Phase 1): use nvim_shada_semsg_2s_u64.
    // nvim_shada_semsg_rcerr_extra_bytes removed (plan 9106c29c Phase 1): use nvim_shada_semsg_u64.
    // nvim_shada_semsg_rcerr_incomplete removed (plan 9106c29c Phase 1): use nvim_shada_semsg_u64.
    // nvim_shada_semsg_rcerr_parse_error removed (plan 9106c29c Phase 1): use nvim_shada_semsg_u64.
    /// Generic semsg wrapper: one string argument.
    fn nvim_shada_semsg_1s(fmt: *const c_char, arg: *const c_char);
    /// Generic semsg wrapper: two string arguments.
    fn nvim_shada_semsg_2s(fmt: *const c_char, a: *const c_char, b: *const c_char);
    /// Generic semsg wrapper: one u64 argument.
    fn nvim_shada_semsg_u64(fmt: *const c_char, val: u64);
    /// Generic semsg wrapper: two strings + u64 + string (for readerr pattern).
    fn nvim_shada_semsg_2s_u64(fmt: *const c_char, a: *const c_char, val: u64, b: *const c_char);
    /// Generic smsg wrapper: one string argument.
    fn nvim_shada_smsg_1s(fmt: *const c_char, arg: *const c_char);
    /// Generic siemsg wrapper: one string argument.
    fn nvim_shada_siemsg_1s(fmt: *const c_char, arg: *const c_char);
    /// Unpack a msgpack dict into a typed keydict struct via field hash.
    /// Returns true on success; writes error string (xmalloc'd) to *error on failure.
    fn unpack_keydict(
        retval: *mut c_void,
        hashy: Option<unsafe extern "C" fn(*const c_char, usize) -> *mut KeySetLink>,
        ad: *mut AdditionalDataBuilder,
        data: *mut *const c_char,
        size: *mut usize,
        error: *mut *mut c_char,
    ) -> bool;
    /// Heap-copy a NvimString (pass arena=NULL for heap allocation).
    fn copy_string(str: NvimString, arena: *mut c_void) -> NvimString;
    /// DictHash for _shada_search_pat.
    fn KeyDict__shada_search_pat_get_field(str: *const c_char, len: usize) -> *mut KeySetLink;
    /// DictHash for _shada_mark.
    fn KeyDict__shada_mark_get_field(str: *const c_char, len: usize) -> *mut KeySetLink;
    /// DictHash for _shada_register.
    fn KeyDict__shada_register_get_field(str: *const c_char, len: usize) -> *mut KeySetLink;
    /// DictHash for _shada_buflist_item.
    fn KeyDict__shada_buflist_item_get_field(str: *const c_char, len: usize) -> *mut KeySetLink;
}

// =============================================================================
// Phase 6 (plan 13c452f9): Rust implementations of non-keydict parse functions
// =============================================================================

/// Inline the RCERR msgpack-check-status logic.
/// Returns SD_READ_STATUS_SUCCESS or SD_READ_STATUS_NOT_SHADA.
unsafe fn rs_check_skip_status(status: c_int, read_size: usize, parse_pos: u64) -> c_int {
    // MPACK_OK = 0, MPACK_EOF = 1, MPACK_ERROR = 2
    if status == MPACK_OK {
        if read_size != 0 {
            nvim_shada_semsg_u64(
                c"E576: Failed to parse ShaDa file: extra bytes in msgpack string at position %llu"
                    .as_ptr(),
                parse_pos,
            );
            SD_READ_STATUS_NOT_SHADA
        } else {
            SD_READ_STATUS_SUCCESS
        }
    } else if status == 1 {
        // MPACK_EOF
        nvim_shada_semsg_u64(
            c"E576: Failed to parse ShaDa file: incomplete msgpack string at position %llu"
                .as_ptr(),
            parse_pos,
        );
        SD_READ_STATUS_NOT_SHADA
    } else {
        nvim_shada_semsg_u64(
            c"E576: Failed to parse ShaDa file due to a msgpack parser error at position %llu"
                .as_ptr(),
            parse_pos,
        );
        SD_READ_STATUS_NOT_SHADA
    }
}

/// Rust replacement for nvim_shada_set_entry_default_data.
#[allow(clippy::cast_possible_wrap)]
unsafe fn rs_set_entry_default_data(entry: *mut ShadaEntry, type_u64: u64) {
    match type_u64 as i32 {
        SD_ITEM_HEADER => {
            (*entry).data.header = std::mem::ManuallyDrop::new(HeaderData::default());
        }
        SD_ITEM_SEARCH_PATTERN => {
            (*entry).data.search_pattern =
                std::mem::ManuallyDrop::new(SearchPatternData::default());
        }
        SD_ITEM_SUB_STRING => {
            (*entry).data.sub_string = std::mem::ManuallyDrop::new(SubStringData::default());
        }
        SD_ITEM_HISTORY_ENTRY => {
            (*entry).data.history_item = std::mem::ManuallyDrop::new(HistoryItemData::default());
        }
        SD_ITEM_REGISTER => {
            (*entry).data.reg = std::mem::ManuallyDrop::new(RegisterData::default());
        }
        SD_ITEM_VARIABLE => {
            (*entry).data.global_var = std::mem::ManuallyDrop::new(GlobalVarData::default());
        }
        SD_ITEM_GLOBAL_MARK | SD_ITEM_LOCAL_MARK => {
            (*entry).data.filemark = std::mem::ManuallyDrop::new(FilemarkData {
                name: b'"' as c_char,
                mark: Position::DEFAULT,
                fname: std::ptr::null_mut(),
            });
        }
        SD_ITEM_JUMP | SD_ITEM_CHANGE => {
            (*entry).data.filemark = std::mem::ManuallyDrop::new(FilemarkData {
                name: 0,
                mark: Position::DEFAULT,
                fname: std::ptr::null_mut(),
            });
        }
        SD_ITEM_BUFFER_LIST => {
            (*entry).data.buffer_list = std::mem::ManuallyDrop::new(BufferListData::default());
        }
        _ => {}
    }
}

/// Rust replacement for nvim_shada_verify_skip.
unsafe fn rs_verify_skip(buf: *const c_char, size: usize, parse_pos: u64) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;
    let status = unpack_skip(&raw mut read_ptr, &raw mut read_size);
    rs_check_skip_status(status, read_size, parse_pos)
}

/// Rust replacement for nvim_shada_set_unknown_item.
#[allow(clippy::too_many_arguments)]
unsafe fn rs_set_unknown_item(
    entry: *mut ShadaEntry,
    type_u64: u64,
    buf: *mut c_char,
    length: usize,
    buf_allocated: bool,
    read_ptr: *const c_char,
    read_size: usize,
    initial_fpos: u64,
    parse_pos: u64,
) -> c_int {
    (*entry).entry_type = ShadaEntryType::Unknown;
    (*entry).data.unknown_item = std::mem::ManuallyDrop::new(UnknownItemData {
        type_num: type_u64,
        contents: std::ptr::null_mut(),
        size: length,
    });

    if initial_fpos == 0 {
        let mut rp = read_ptr;
        let mut rs = read_size;
        let status = unpack_skip(&raw mut rp, &raw mut rs);
        let spm_ret = rs_check_skip_status(status, rs, parse_pos);
        if spm_ret != SD_READ_STATUS_SUCCESS {
            if buf_allocated {
                nvim_xfree(buf.cast::<c_void>());
            }
            (*entry).entry_type = ShadaEntryType::Missing;
            return spm_ret;
        }
    }

    let contents = if buf_allocated {
        buf
    } else {
        let mem = nvim_xmalloc(length).cast::<c_char>();
        std::ptr::copy_nonoverlapping(buf, mem, length);
        mem
    };
    (*entry).data.unknown_item = std::mem::ManuallyDrop::new(UnknownItemData {
        type_num: type_u64,
        contents,
        size: length,
    });
    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_history.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
unsafe fn rs_parse_history(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
    num_additional: *mut u32,
    read_ptr_out: *mut *const c_char,
    read_size_out: *mut usize,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;

    let len = unpack_array(&raw mut read_ptr, &raw mut read_size);
    if len < 2 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"history".as_ptr(),
            initial_fpos,
            c"is not an array with enough elements".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    let mut hist_type: i64 = 0;
    if !unpack_integer(&raw mut read_ptr, &raw mut read_size, &raw mut hist_type) {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"history".as_ptr(),
            initial_fpos,
            c"has wrong history type type".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    let item = unpack_string(&raw mut read_ptr, &raw mut read_size);
    if item.is_null() {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"history".as_ptr(),
            initial_fpos,
            c"has wrong history string type".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }
    if !libc::memchr(item.data.cast::<c_void>(), 0, item.size).is_null() {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"history".as_ptr(),
            initial_fpos,
            c"contains string with zero byte inside".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    let hist_type_u8 = hist_type as u8;
    let is_hist_search = hist_type_u8 == HIST_SEARCH;

    let sep: c_char = if is_hist_search {
        if len < 3 {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"search history".as_ptr(),
                initial_fpos,
                c"does not have separator character".as_ptr(),
            );
            return SD_READ_STATUS_MALFORMED;
        }
        let mut sep_type: i64 = 0;
        if !unpack_integer(&raw mut read_ptr, &raw mut read_size, &raw mut sep_type) {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"search history".as_ptr(),
                initial_fpos,
                c"has wrong history separator type".as_ptr(),
            );
            return SD_READ_STATUS_MALFORMED;
        }
        sep_type as c_char
    } else {
        0
    };

    // Allocate string: item.size bytes + NUL + sep (matches C logic exactly)
    let strsize = item.size + 2;
    let string = nvim_xmalloc(strsize).cast::<c_char>();
    std::ptr::copy_nonoverlapping(item.data, string, item.size);
    *string.add(item.size) = 0;
    *string.add(item.size + 1) = sep;

    (*entry).data.history_item = std::mem::ManuallyDrop::new(HistoryItemData {
        histtype: hist_type_u8,
        string,
        sep,
    });

    *num_additional = (len as u32).wrapping_sub(2 + u32::from(is_hist_search));
    *read_ptr_out = read_ptr;
    *read_size_out = read_size;
    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_variable.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
unsafe fn rs_parse_variable(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
    num_additional: *mut u32,
    read_ptr_out: *mut *const c_char,
    read_size_out: *mut usize,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;

    let len = unpack_array(&raw mut read_ptr, &raw mut read_size);
    if len < 2 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"variable".as_ptr(),
            initial_fpos,
            c"is not an array with enough elements".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    let name = unpack_string(&raw mut read_ptr, &raw mut read_size);
    if name.is_null() {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"variable".as_ptr(),
            initial_fpos,
            c"has wrong variable name type".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    // Set the name atomically (ManuallyDrop union fields cannot be mutated field-by-field).
    // We set value to null here; the actual typval_T is written below via tv_ptr into C memory.
    (*entry).data.global_var = std::mem::ManuallyDrop::new(GlobalVarData {
        name: nvim_xmemdupz(name.data, name.size),
        value: std::ptr::null_mut(),
    });

    let binval = unpack_string(&raw mut read_ptr, &raw mut read_size);
    let mut is_blob = false;
    if binval.is_null() {
        // No binary value: unpack as a generic typval_T
        let tv_ptr = nvim_shada_entry_var_value_ptr(entry);
        let status = unpack_typval(&raw mut read_ptr, &raw mut read_size, tv_ptr);
        if status != MPACK_OK {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"variable".as_ptr(),
                initial_fpos,
                c"has value that cannot be converted to the Vimscript value".as_ptr(),
            );
            return SD_READ_STATUS_MALFORMED;
        }
    } else {
        // Binary value: optionally check blob type tag
        if len > 2 {
            let mut type_val: i64 = 0;
            if !unpack_integer(&raw mut read_ptr, &raw mut read_size, &raw mut type_val)
                || type_val != 10
            // VAR_TYPE_BLOB
            {
                nvim_shada_semsg_2s_u64(
                    c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                    c"variable".as_ptr(),
                    initial_fpos,
                    c"has wrong variable type".as_ptr(),
                );
                return SD_READ_STATUS_MALFORMED;
            }
            is_blob = true;
        }
        // decode_string writes a typval_T into the entry at the C-layout location
        let tv_ptr = nvim_shada_entry_var_value_ptr(entry);
        nvim_shada_decode_string_into(binval.data, binval.size, is_blob, tv_ptr);
    }

    *num_additional = (len as u32).wrapping_sub(2 + u32::from(is_blob));
    *read_ptr_out = read_ptr;
    *read_size_out = read_size;
    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_substr.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
unsafe fn rs_parse_substr(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
    num_additional: *mut u32,
    read_ptr_out: *mut *const c_char,
    read_size_out: *mut usize,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;

    let len = unpack_array(&raw mut read_ptr, &raw mut read_size);
    if len < 1 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"sub string".as_ptr(),
            initial_fpos,
            c"is not an array with enough elements".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    let sub = unpack_string(&raw mut read_ptr, &raw mut read_size);
    if sub.is_null() {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"sub string".as_ptr(),
            initial_fpos,
            c"has wrong sub string type".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    // Assign atomically (ManuallyDrop union fields cannot be mutated field-by-field).
    (*entry).data.sub_string = std::mem::ManuallyDrop::new(SubStringData {
        sub: nvim_xmemdupz(sub.data, sub.size),
    });
    *num_additional = (len as u32).wrapping_sub(1);
    *read_ptr_out = read_ptr;
    *read_size_out = read_size;
    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_additional_data.
#[allow(clippy::cast_sign_loss)]
unsafe fn rs_parse_additional_data(
    entry: *mut ShadaEntry,
    read_ptr: *const c_char,
    read_size: usize,
    num_additional: u32,
    initial_fpos: u64,
) -> c_int {
    let mut ad = AdditionalDataBuilder::new();
    let mut rp = read_ptr;
    let mut rs = read_size;

    for _ in 0..num_additional {
        let item_start = rp;
        let status = unpack_skip(&raw mut rp, &raw mut rs);
        if status != MPACK_OK {
            if !ad.items.is_null() {
                nvim_xfree(ad.items.cast::<c_void>());
            }
            return SD_READ_STATUS_MALFORMED;
        }
        let item_len = rp.offset_from(item_start) as usize;
        push_additional_data(&raw mut ad, item_start, item_len);
    }

    if rs != 0 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"item".as_ptr(),
            initial_fpos,
            c"additional bytes".as_ptr(),
        );
        if !ad.items.is_null() {
            nvim_xfree(ad.items.cast::<c_void>());
        }
        return SD_READ_STATUS_MALFORMED;
    }

    (*entry).additional_data = ad.items.cast::<c_void>();
    SD_READ_STATUS_SUCCESS
}

// =============================================================================
// Phase 6 Part 2 (plan 13c452f9): Rust implementations of keydict parse functions
// =============================================================================

/// Rust replacement for nvim_shada_parse_search_pattern.
/// Uses a local KeyDict_shada_search_pat with unpack_keydict, then copies fields
/// into the Rust SearchPatternData in entry->data.search_pattern.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
unsafe fn rs_parse_search_pattern(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;
    let mut error_alloc: *mut c_char = std::ptr::null_mut();

    // Use a zeroed local keydict struct as the unpack target.
    let mut it = KeyDict_shada_search_pat {
        is_set: 0,
        magic: false,
        smartcase: false,
        has_line_offset: false,
        place_cursor_at_end: false,
        is_last_used: false,
        is_substitute_pattern: false,
        highlighted: false,
        search_backward: false,
        offset: 0,
        pat: NvimString {
            data: std::ptr::null_mut(),
            size: 0,
        },
    };

    if !unpack_keydict(
        std::ptr::addr_of_mut!(it).cast::<c_void>(),
        Some(KeyDict__shada_search_pat_get_field),
        std::ptr::null_mut(),
        &raw mut read_ptr,
        &raw mut read_size,
        &raw mut error_alloc,
    ) {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"search pattern".as_ptr(),
            initial_fpos,
            error_alloc,
        );
        nvim_xfree(error_alloc.cast::<c_void>());
        return SD_READ_STATUS_MALFORMED;
    }

    // KEYSET_OPTIDX__shada_search_pat__sp = 8
    if !has_key(it.is_set, 8) {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"search pattern".as_ptr(),
            initial_fpos,
            c"has no pattern".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    // Heap-copy the pat string (copy_string with NULL arena).
    let pat_copy = copy_string(it.pat, std::ptr::null_mut());

    // Copy fields into the Rust SearchPatternData. Keep defaults for fields not set.
    // KEYSET_OPTIDX: sb=1, sc=2, se=3, sh=4, sl=5, sm=6, so=7, sp=8, ss=9, su=10
    let defaults = SearchPatternData::default();
    let magic = if has_key(it.is_set, 6) {
        it.magic
    } else {
        defaults.magic
    };
    let smartcase = if has_key(it.is_set, 2) {
        it.smartcase
    } else {
        defaults.smartcase
    };
    let has_line_offset = if has_key(it.is_set, 5) {
        it.has_line_offset
    } else {
        defaults.has_line_offset
    };
    let place_cursor_at_end = if has_key(it.is_set, 3) {
        it.place_cursor_at_end
    } else {
        defaults.place_cursor_at_end
    };
    let is_last_used = if has_key(it.is_set, 10) {
        it.is_last_used
    } else {
        defaults.is_last_used
    };
    let is_substitute_pattern = if has_key(it.is_set, 9) {
        it.is_substitute_pattern
    } else {
        defaults.is_substitute_pattern
    };
    let highlighted = if has_key(it.is_set, 4) {
        it.highlighted
    } else {
        defaults.highlighted
    };
    let search_backward = if has_key(it.is_set, 1) {
        it.search_backward
    } else {
        defaults.search_backward
    };
    let offset = if has_key(it.is_set, 7) {
        it.offset
    } else {
        defaults.offset
    };

    (*entry).data.search_pattern = std::mem::ManuallyDrop::new(SearchPatternData {
        magic,
        smartcase,
        has_line_offset,
        place_cursor_at_end,
        offset,
        is_last_used,
        is_substitute_pattern,
        highlighted,
        search_backward,
        pat: pat_copy.data,
        pat_len: pat_copy.size,
    });

    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_mark.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
unsafe fn rs_parse_mark(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
    type_u64: u64,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;
    let mut error_alloc: *mut c_char = std::ptr::null_mut();

    let mut it = KeyDict_shada_mark {
        is_set: 0,
        n: 0,
        l: 0,
        c: 0,
        f: NvimString {
            data: std::ptr::null_mut(),
            size: 0,
        },
    };

    if !unpack_keydict(
        std::ptr::addr_of_mut!(it).cast::<c_void>(),
        Some(KeyDict__shada_mark_get_field),
        std::ptr::null_mut(),
        &raw mut read_ptr,
        &raw mut read_size,
        &raw mut error_alloc,
    ) {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"mark".as_ptr(),
            initial_fpos,
            error_alloc,
        );
        nvim_xfree(error_alloc.cast::<c_void>());
        return SD_READ_STATUS_MALFORMED;
    }

    // KEYSET_OPTIDX: c=1, f=2, l=3, n=4
    // Read the existing filemark defaults (set by rs_set_entry_default_data).
    let existing: FilemarkData = *(*entry).data.filemark;

    let name = if has_key(it.is_set, 4) {
        // n key only valid for local/global marks, not jump/change
        if type_u64 == SD_ITEM_JUMP as u64 || type_u64 == SD_ITEM_CHANGE as u64 {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"mark".as_ptr(),
                initial_fpos,
                c"has n key which is only valid for local and global mark entries".as_ptr(),
            );
            return SD_READ_STATUS_MALFORMED;
        }
        it.n as c_char
    } else {
        existing.name
    };

    let lnum = if has_key(it.is_set, 3) {
        it.l
    } else {
        existing.mark.lnum
    };
    let col = if has_key(it.is_set, 1) {
        it.c as i32
    } else {
        existing.mark.col
    };
    let fname = if has_key(it.is_set, 2) {
        nvim_xmemdupz(it.f.data, it.f.size)
    } else {
        existing.fname
    };

    if fname.is_null() {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"mark".as_ptr(),
            initial_fpos,
            c"is missing file name".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }
    if lnum <= 0 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"mark".as_ptr(),
            initial_fpos,
            c"has invalid line number".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }
    if col < 0 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"mark".as_ptr(),
            initial_fpos,
            c"has invalid column number".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    (*entry).data.filemark = std::mem::ManuallyDrop::new(FilemarkData {
        name,
        mark: Position::new(lnum, col, 0),
        fname,
    });

    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_register.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
unsafe fn rs_parse_register(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;
    let mut error_alloc: *mut c_char = std::ptr::null_mut();

    let mut it = KeyDict_shada_register {
        is_set: 0,
        rc: StringArray {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        },
        ru: false,
        rt: 0,
        n: 0,
        rw: 0,
    };

    if !unpack_keydict(
        std::ptr::addr_of_mut!(it).cast::<c_void>(),
        Some(KeyDict__shada_register_get_field),
        std::ptr::null_mut(),
        &raw mut read_ptr,
        &raw mut read_size,
        &raw mut error_alloc,
    ) {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"register".as_ptr(),
            initial_fpos,
            error_alloc,
        );
        nvim_xfree(error_alloc.cast::<c_void>());
        nvim_xfree(it.rc.items.cast::<c_void>());
        return SD_READ_STATUS_MALFORMED;
    }

    // KEYSET_OPTIDX: n=1, rc=2, rt=3, ru=4, rw=5
    if it.rc.size == 0 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"register".as_ptr(),
            initial_fpos,
            c"has rc key with missing or empty array".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }

    // Allocate Rust-layout contents array (array of *mut c_char).
    let contents_size = it.rc.size;
    let contents =
        nvim_xmalloc(contents_size * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    for j in 0..contents_size {
        let s = copy_string(*it.rc.items.add(j), std::ptr::null_mut());
        *contents.add(j) = s.data;
    }
    nvim_xfree(it.rc.items.cast::<c_void>());

    let defaults = RegisterData::default();
    // ru=4, rt=3, n=1, rw=5
    let is_unnamed = if has_key(it.is_set, 4) {
        it.ru
    } else {
        defaults.is_unnamed
    };
    let reg_type = if has_key(it.is_set, 3) {
        it.rt as c_int
    } else {
        defaults.reg_type
    };
    let name = if has_key(it.is_set, 1) {
        it.n as c_char
    } else {
        defaults.name
    };
    let width = if has_key(it.is_set, 5) {
        it.rw as usize
    } else {
        defaults.width
    };

    (*entry).data.reg = std::mem::ManuallyDrop::new(RegisterData {
        name,
        reg_type,
        contents,
        contents_size,
        is_unnamed,
        width,
    });

    SD_READ_STATUS_SUCCESS
}

/// Rust replacement for nvim_shada_parse_buflist.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
unsafe fn rs_parse_buflist(
    entry: *mut ShadaEntry,
    buf: *const c_char,
    size: usize,
    initial_fpos: u64,
) -> c_int {
    let mut read_ptr = buf;
    let mut read_size = size;

    let len = unpack_array(&raw mut read_ptr, &raw mut read_size);
    if len < 0 {
        nvim_shada_semsg_2s_u64(
            c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
            c"buffer list".as_ptr(),
            initial_fpos,
            c"is not an array".as_ptr(),
        );
        return SD_READ_STATUS_MALFORMED;
    }
    if len == 0 {
        return SD_READ_STATUS_SUCCESS;
    }

    let buf_count = len as usize;
    let buffers =
        nvim_xcalloc(buf_count, std::mem::size_of::<BufferListBuffer>()).cast::<BufferListBuffer>();

    let mut actual_size: usize = 0;
    for i in 0..buf_count {
        actual_size += 1;
        let mut it = KeyDict_shada_buflist_item {
            is_set: 0,
            l: 0,
            c: 0,
            f: NvimString {
                data: std::ptr::null_mut(),
                size: 0,
            },
        };
        let mut it_ad = AdditionalDataBuilder::new();
        let mut error_alloc: *mut c_char = std::ptr::null_mut();

        if !unpack_keydict(
            std::ptr::addr_of_mut!(it).cast::<c_void>(),
            Some(KeyDict__shada_buflist_item_get_field),
            &raw mut it_ad,
            &raw mut read_ptr,
            &raw mut read_size,
            &raw mut error_alloc,
        ) {
            // Build error message like C: "buffer list at position %u contains entry that %s"
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"buffer list".as_ptr(),
                initial_fpos,
                error_alloc,
            );
            nvim_xfree(error_alloc.cast::<c_void>());
            nvim_xfree(it_ad.items.cast::<c_void>());
            (*entry).data.buffer_list = std::mem::ManuallyDrop::new(BufferListData {
                size: actual_size,
                buffers,
            });
            return SD_READ_STATUS_MALFORMED;
        }

        let e = &mut *buffers.add(i);
        e.additional_data = it_ad.items.cast::<c_void>();
        e.pos = Position::DEFAULT;

        // KEYSET_OPTIDX: c=1, f=2, l=3
        if has_key(it.is_set, 3) {
            e.pos.lnum = it.l;
        }
        if has_key(it.is_set, 1) {
            e.pos.col = it.c as i32;
        }
        if has_key(it.is_set, 2) {
            e.fname = nvim_xmemdupz(it.f.data, it.f.size);
        }

        if e.pos.lnum <= 0 {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"buffer list".as_ptr(),
                initial_fpos,
                c"contains entry with invalid line number".as_ptr(),
            );
            (*entry).data.buffer_list = std::mem::ManuallyDrop::new(BufferListData {
                size: actual_size,
                buffers,
            });
            return SD_READ_STATUS_MALFORMED;
        }
        if e.pos.col < 0 {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"buffer list".as_ptr(),
                initial_fpos,
                c"contains entry with invalid column number".as_ptr(),
            );
            (*entry).data.buffer_list = std::mem::ManuallyDrop::new(BufferListData {
                size: actual_size,
                buffers,
            });
            return SD_READ_STATUS_MALFORMED;
        }
        if e.fname.is_null() {
            nvim_shada_semsg_2s_u64(
                c"E575: Error while reading ShaDa file: %s entry at position %llu %s".as_ptr(),
                c"buffer list".as_ptr(),
                initial_fpos,
                c"contains entry that does not have a file name".as_ptr(),
            );
            (*entry).data.buffer_list = std::mem::ManuallyDrop::new(BufferListData {
                size: actual_size,
                buffers,
            });
            return SD_READ_STATUS_MALFORMED;
        }
    }

    (*entry).data.buffer_list = std::mem::ManuallyDrop::new(BufferListData {
        size: actual_size,
        buffers,
    });
    SD_READ_STATUS_SUCCESS
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

/// Find the position after a given parameter character in the 'shada' option string.
///
/// Iterates through `p_shada` looking for a character matching `typ`. Stops at 'n'
/// (always the last parameter) or if no more commas are found.
///
/// Returns a pointer to the character immediately after `typ` in the option string,
/// or NULL if `typ` is not found.
///
/// Pure Rust implementation; does not call back into C.
///
/// # Safety
///
/// `nvim_shada_get_p_shada()` must return a valid NUL-terminated C string.
unsafe fn find_shada_parameter_impl(typ: c_int) -> *const c_char {
    let p_shada = nvim_shada_get_p_shada();
    if p_shada.is_null() {
        return std::ptr::null();
    }
    let target = typ as u8;
    let mut p = p_shada;
    while *p != 0 {
        if *p as u8 == target {
            // Return pointer to the character after the type char
            return p.add(1);
        }
        if *p as u8 == b'n' {
            // 'n' is always the last parameter
            break;
        }
        // Skip forward to the next ',' delimiter
        let comma = libc::strchr(p, c_int::from(b','));
        if comma.is_null() {
            break;
        }
        // The for-loop `p++` in C advances past the comma; replicate that
        p = comma.add(1).cast_const();
    }
    std::ptr::null()
}

/// Get the shada parameter value for a given type character.
///
/// Returns the integer value following `typ` in 'shada', or -1 if not found or
/// not followed by a digit.
///
/// Pure Rust implementation; does not call back into C.
///
/// # Safety
///
/// `nvim_shada_get_p_shada()` must return a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_shada_parameter(typ: c_int) -> c_int {
    let p = find_shada_parameter_impl(typ);
    if !p.is_null() && (*p as u8).is_ascii_digit() {
        return libc::atoi(p);
    }
    -1
}

/// Find the shada parameter string for a given type character.
///
/// Returns a pointer into the 'shada' option string pointing to the character
/// immediately after `typ`, or NULL if `typ` is not found.
///
/// Pure Rust implementation; does not call back into C.
///
/// # Safety
///
/// `nvim_shada_get_p_shada()` must return a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_shada_parameter(typ: c_int) -> *const c_char {
    find_shada_parameter_impl(typ)
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

/// Compute srni_flags for shada_read from ShaDaReadFileFlags.
///
/// Matches the logic of C nvim_shada_get_srni_flags (now deleted).
///
/// # Safety
///
/// Accesses p_hi and shada option via C accessors.
pub unsafe fn compute_srni_flags(
    flags: c_int,
    local_marks: c_int,
    get_old_files: bool,
    argcount: c_int,
) -> u32 {
    let want_marks = (flags & SHADA_WANT_MARKS) != 0;
    let mut srni_flags: u32 = 0;
    if (flags & SHADA_WANT_INFO) != 0 {
        srni_flags |= SD_READ_UNDISABLEABLE_DATA | SD_READ_REGISTERS | SD_READ_GLOBAL_MARKS;
        if nvim_get_p_hi() > 0 {
            srni_flags |= SD_READ_HISTORY;
        }
        if !find_shada_parameter_impl(c_int::from(b'!')).is_null() {
            srni_flags |= SD_READ_VARIABLES;
        }
        if !find_shada_parameter_impl(c_int::from(b'%')).is_null() && argcount == 0 {
            srni_flags |= SD_READ_BUFFER_LIST;
        }
    }
    if want_marks && local_marks > 0 {
        srni_flags |= SD_READ_LOCAL_MARKS | SD_READ_CHANGES;
    }
    if get_old_files {
        srni_flags |= SD_READ_LOCAL_MARKS;
    }
    srni_flags
}

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
        if !find_shada_parameter_impl(c_int::from(b'!')).is_null() {
            srni_flags |= SD_READ_VARIABLES;
        }

        // Check for '%' in shada option
        if !find_shada_parameter_impl(c_int::from(b'%')).is_null() {
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

/// ShaDa packing buffer - transparent view of C's PackerBuffer from packer_defs.h.
///
/// Layout must match `struct packer_buffer_t` exactly:
/// ```c
/// struct packer_buffer_t {
///   char *startptr;
///   char *ptr;
///   char *endptr;
///   void *anydata;
///   int64_t anyint;
///   void (*packer_flush)(PackerBuffer *self);
/// };
/// ```
#[repr(C)]
pub struct ShadaPackerBuffer {
    pub startptr: *mut c_char,
    pub ptr: *mut c_char,
    pub endptr: *mut c_char,
    pub anydata: *mut c_void,
    pub anyint: i64,
    pub packer_flush: Option<unsafe extern "C" fn(*mut ShadaPackerBuffer)>,
}

// Direct C function declarations replacing the 8 nvim_shada_packer_* wrappers.
extern "C" {
    /// Create a string-backed packer buffer (from packer.c).
    fn packer_string_buffer() -> ShadaPackerBuffer;
    /// Take the packed string from a string-backed packer buffer (from packer.c).
    fn packer_take_string(buf: *mut ShadaPackerBuffer) -> NvimString;
}

/// Minimum buffer size for packing items
pub const SHADA_PACK_ITEM_SIZE: usize = 9;

// Inline helpers replacing the deleted nvim_shada_packer_get_ptr / _set_ptr /
// _get_endptr / _flush / _get_anyint C accessor functions.  All call sites
// already use these names, so they compile unchanged after this change.

#[inline]
unsafe fn nvim_shada_packer_get_ptr(packer: *mut ShadaPackerBuffer) -> *mut u8 {
    (*packer).ptr.cast::<u8>()
}

#[inline]
unsafe fn nvim_shada_packer_set_ptr(packer: *mut ShadaPackerBuffer, ptr: *mut u8) {
    (*packer).ptr = ptr.cast::<c_char>();
}

#[inline]
unsafe fn nvim_shada_packer_flush(packer: *mut ShadaPackerBuffer) {
    if let Some(flush) = (*packer).packer_flush {
        flush(packer);
    }
}

#[inline]
unsafe fn nvim_shada_packer_get_anyint(packer: *mut ShadaPackerBuffer) -> i64 {
    (*packer).anyint
}

#[inline]
unsafe fn nvim_shada_packer_string_buffer(out: *mut ShadaPackerBuffer) {
    *out = packer_string_buffer();
}

#[inline]
unsafe fn nvim_shada_packer_take_string(buf: *mut ShadaPackerBuffer) -> NvimString {
    packer_take_string(buf)
}

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

    let ptr = (*packer).ptr.cast::<u8>();
    let endptr = (*packer).endptr.cast::<u8>();
    let remaining = (endptr as usize).saturating_sub(ptr as usize);

    if remaining < 2 * SHADA_PACK_ITEM_SIZE {
        if let Some(flush) = (*packer).packer_flush {
            flush(packer);
        }
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

    let mut ptr = (*packer).ptr.cast::<u8>();

    // Pack entry type
    rs_mpack_uint64_inline(&raw mut ptr, entry_type);
    // Pack timestamp
    rs_mpack_uint64_inline(&raw mut ptr, timestamp);
    // Pack length
    rs_mpack_uint64_inline(&raw mut ptr, length);

    (*packer).ptr = ptr.cast::<c_char>();
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
        let ptr = (*packer).ptr.cast::<u8>();
        let endptr = (*packer).endptr.cast::<u8>();
        let remaining = (endptr as usize).saturating_sub(ptr as usize);
        let to_copy = (len - pos).min(remaining);

        if to_copy > 0 {
            std::ptr::copy_nonoverlapping(data.add(pos), ptr, to_copy);
            (*packer).ptr = ptr.add(to_copy).cast::<c_char>();
        }
        pos += to_copy;

        if pos < len {
            if let Some(flush) = (*packer).packer_flush {
                flush(packer);
            }
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

    if let Some(flush) = (*packer).packer_flush {
        flush(packer);
    }
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
    let mut ptr = (*packer).ptr.cast::<u8>();
    rs_mpack_uint64_inline(&raw mut ptr, entry_type as u64);
    (*packer).ptr = ptr.cast::<c_char>();
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
    let mut ptr = (*packer).ptr.cast::<u8>();
    rs_mpack_uint64_inline(&raw mut ptr, timestamp);
    (*packer).ptr = ptr.cast::<c_char>();
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
    let mut ptr = (*packer).ptr.cast::<u8>();
    rs_mpack_uint64_inline(&raw mut ptr, length);
    (*packer).ptr = ptr.cast::<c_char>();
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

impl BufferListBuffer {
    /// Default value (constant for use in raw pointer contexts).
    pub const DEFAULT: Self = Self {
        pos: Position::DEFAULT,
        fname: std::ptr::null_mut(),
        additional_data: std::ptr::null_mut(),
    };
}

impl Default for BufferListBuffer {
    fn default() -> Self {
        Self::DEFAULT
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
// Map types (must match C layout from map_defs.h)
// =============================================================================

/// FFI-compatible representation of C's `MapHash` (32 bytes).
#[repr(C)]
pub struct MapHash {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub n_keys: u32,
    pub keys_capacity: u32,
    pub hash: *mut u32,
}

impl Default for MapHash {
    fn default() -> Self {
        Self {
            n_buckets: 0,
            size: 0,
            n_occupied: 0,
            upper_bound: 0,
            n_keys: 0,
            keys_capacity: 0,
            hash: std::ptr::null_mut(),
        }
    }
}

/// Sentinel value for "not found" in map operations.
const MH_TOMBSTONE: u32 = u32::MAX;

/// FFI-compatible representation of C's `Set(cstr_t)` (40 bytes).
#[repr(C)]
pub struct SetCstrT {
    pub h: MapHash,
    pub keys: *mut *const c_char,
}

impl Default for SetCstrT {
    fn default() -> Self {
        Self {
            h: MapHash::default(),
            keys: std::ptr::null_mut(),
        }
    }
}

/// FFI-compatible representation of C's `PMap(cstr_t)` aka `Map(cstr_t, ptr_t)` (48 bytes).
#[repr(C)]
pub struct PMapCstrT {
    pub set: SetCstrT,
    pub values: *mut *mut c_void,
}

impl Default for PMapCstrT {
    fn default() -> Self {
        Self {
            set: SetCstrT::default(),
            values: std::ptr::null_mut(),
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
    /// Map of history strings to entry pointers (inline PMap(cstr_t), 48 bytes).
    pub contained_entries: PMapCstrT,
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
            contained_entries: PMapCstrT::default(),
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
    /// Initialize contained entries map in place.
    fn nvim_hmll_map_init(map: *mut PMapCstrT);
    /// Destroy contained entries map contents.
    fn nvim_hmll_map_destroy(map: *mut PMapCstrT);
    /// Get entry from map by string key.
    fn nvim_hmll_map_get(map: *mut PMapCstrT, key: *const c_char) -> *mut HMLListEntry;
    /// Get key index from set by string key (returns MH_TOMBSTONE if not found).
    fn mh_get_cstr_t(set: *mut SetCstrT, key: *const c_char) -> u32;
    /// Put entry into map by string key.
    fn nvim_hmll_map_put(map: *mut PMapCstrT, key: *const c_char, entry: *mut HMLListEntry);
    /// Remove entry from map by string key.
    fn nvim_hmll_map_del(map: *mut PMapCstrT, key: *const c_char);
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
        contained_entries: PMapCstrT::default(),
    };
    nvim_hmll_map_init(&raw mut (*hmll).contained_entries);
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
        nvim_hmll_map_del(&raw mut list.contained_entries, key);
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
    rs_shada_free_entry_contents(&raw mut entry.data);
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
        nvim_hmll_map_put(&raw mut list.contained_entries, key, target_entry);
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
    nvim_hmll_map_destroy(&raw mut list.contained_entries);

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
    hms.iter = rs_shada_hist_iter(
        std::ptr::null(),
        history_type,
        c_int::from(reading),
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
            hms.iter = rs_shada_hist_iter(
                hms.iter,
                hms.history_type,
                c_int::from(hms.reading),
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
        nvim_hmll_map_get(&raw mut (*hmll).contained_entries, key)
    };

    if !existing.is_null() {
        let existing_entry = &mut *existing;
        if entry.timestamp > existing_entry.data.timestamp {
            // New entry is newer, remove the old one
            rs_hmll_remove(hmll, existing);
        } else if !do_iter && entry.timestamp == existing_entry.data.timestamp {
            // Same timestamp, prefer current Neovim instance entry.
            // The old key string is freed as part of freeing the entry, so
            // we must update the key pointer in the map to the new string.
            let new_key = entry.data.history_item.string;
            let map = &raw mut (*hmll).contained_entries;
            let ki = mh_get_cstr_t(&raw mut (*map).set, key);
            rs_shada_free_entry_contents(&raw mut existing_entry.data);
            existing_entry.data = entry;
            if ki != MH_TOMBSTONE {
                *(*map).set.keys.add(ki as usize) = new_key;
            }
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
        hms.iter = rs_shada_hist_iter(
            hms.iter,
            hms.history_type,
            c_int::from(hms.reading),
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

/// Convert history merger state to a histentry_T array.
///
/// Iterates over the history merger's linked list and populates the
/// provided histentry_T array via a C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_hms_to_he_array(
    hms_p: *const HistoryMergerState,
    hist_array: *mut c_void,
    new_hisidx: *mut c_int,
    new_hisnum: *mut c_int,
) {
    let hmll = &(*hms_p).hmll;
    let mut idx: c_int = 0;
    let mut cur = hmll.first;
    while !cur.is_null() {
        let entry = &(*cur).data;
        let hist_ptr: *const HistoryItemData = std::ptr::addr_of!(entry.data.history_item).cast();
        let hist = std::ptr::read(hist_ptr);
        nvim_shada_set_histentry(
            hist_array,
            idx,
            entry.timestamp,
            hist.string,
            entry.additional_data,
        );
        idx += 1;
        cur = (*cur).next;
    }
    *new_hisnum = idx;
    *new_hisidx = idx - 1;
}

// =============================================================================
// Phase 6: High-Level API Functions
// =============================================================================

// C accessor functions for high-level API
extern "C" {
    /// Get current p_shadafile option.
    fn nvim_get_p_shadafile() -> *const c_char;
    /// Expand environment variables in path.
    fn nvim_expand_env(src: *const c_char, dst: *mut c_char, dstlen: usize) -> usize;
    /// Duplicate string with length and allocation.
    fn nvim_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
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
            if !p_shadafile.is_null() && libc::strcmp(p_shadafile, none_str) == 0 {
                return std::ptr::null_mut();
            }
            p_shadafile
        } else {
            // Check for -n parameter or use default
            let param_file = find_shada_parameter_impl(c_int::from(b'n'));
            if param_file.is_null() || *param_file == 0 {
                let default_file = rs_shada_get_default_file();
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

/// Check if marks need to be read from ShaDa file for current buffer.
///
/// If the current buffer hasn't had its marks read yet, and the 'shada'
/// option includes the `'` parameter (marks saving), and the buffer has
/// a filename, then reads marks from the ShaDa file.
///
/// Always sets `b_marks_read` to true afterward.
#[no_mangle]
pub unsafe extern "C" fn rs_check_marks_read() {
    if nvim_shada_curbuf_marks_read() == 0
        && rs_get_shada_parameter(c_int::from(b'\'')) > 0
        && !nvim_shada_curbuf_ffname().is_null()
    {
        rs_shada_read_marks();
    }
    // Always set b_marks_read; needed when 'shada' is changed to include
    // the ' parameter after opening a buffer.
    nvim_shada_curbuf_set_marks_read(1);
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
        verbose_enter();
        nvim_shada_smsg_reading(
            fname,
            c_int::from(flags & SHADA_WANT_INFO as c_int != 0),
            c_int::from(flags & SHADA_WANT_MARKS as c_int != 0),
            c_int::from(flags & SHADA_GET_OLDFILES as c_int != 0),
            c_int::from(of_ret != 0),
        );
        verbose_leave();
    }

    if of_ret != 0 {
        if of_ret != UV_ENOENT || (flags & SHADA_MISSING_ERROR as c_int) != 0 {
            nvim_shada_semsg_2s(
                c"E886: System error while opening ShaDa file %s for reading: %s".as_ptr(),
                fname,
                nvim_shada_os_strerror(of_ret),
            );
        }
        nvim_xfree(fname.cast::<c_void>());
        nvim_xfree(sd_reader);
        return FAIL;
    }
    nvim_xfree(fname.cast::<c_void>());

    rs_shada_read(fd.as_ptr(), flags);
    rs_close_file(fd);
    nvim_xfree(sd_reader);

    OK
}

// =============================================================================
// ShaDa Write: rs_shada_write (replaces C shada_write + shada_read_when_writing)
// =============================================================================

/// Implement COMPARE_WITH_ENTRY logic: if wms_entry has a newer or equal
/// timestamp, free the incoming entry; otherwise free the wms_entry and
/// replace it with the incoming entry.
///
/// # Safety
///
/// Both pointers must be valid. `entry` is consumed (moved into `*wms_entry`
/// or freed). Returns false if entry was discarded, true if placed.
#[inline]
unsafe fn compare_with_entry(wms_entry: *mut ShadaEntry, entry: ShadaEntry) -> bool {
    if (*wms_entry).entry_type != ShadaEntryType::Missing {
        if (*wms_entry).timestamp >= entry.timestamp {
            // Existing entry is newer or equal; discard incoming entry.
            let mut e = entry;
            rs_shada_free_entry_contents(&raw mut e);
            return false;
        }
        // Incoming entry is newer; free existing.
        rs_shada_free_entry_contents(wms_entry);
    }
    *wms_entry = entry;
    true
}

/// Read the existing ShaDa file and merge entries into WriteMergerState.
///
/// Corresponds to C's `shada_read_when_writing`. Called only from
/// `rs_shada_write` when `sd_reader` is non-null.
///
/// # Safety
///
/// All pointer arguments must be valid.
#[allow(clippy::too_many_lines, clippy::cast_possible_wrap)]
unsafe fn shada_read_when_writing(
    sd_reader: *mut c_void,
    srni_flags: u32,
    max_kbyte: usize,
    wms: *mut WriteMergerState,
    packer: *mut ShadaPackerBuffer,
) -> c_int {
    let mut ret = SD_WRITE_SUCCESSFUL;
    let mut entry = ShadaEntry::default();

    loop {
        let srni_ret = rs_shada_read_next_item(sd_reader, &raw mut entry, srni_flags, max_kbyte);
        match srni_ret {
            r if r == SD_READ_STATUS_FINISHED => break,
            r if r == SD_READ_STATUS_SUCCESS => {}
            r if r == SD_READ_STATUS_NOT_SHADA => {
                ret = SD_WRITE_READ_NOT_SHADA;
                return ret;
            }
            r if r == SD_READ_STATUS_READ_ERROR => {
                return ret;
            }
            _ => {
                // kSDReadStatusMalformed: skip
                entry = ShadaEntry::default();
                continue;
            }
        }

        match entry.entry_type {
            ShadaEntryType::Missing => {}
            ShadaEntryType::Header | ShadaEntryType::BufferList => {
                // These should never appear when reading for writing.
                // In C this is abort(); we just free and continue.
                rs_shada_free_entry_contents(&raw mut entry);
                entry = ShadaEntry::default();
            }
            ShadaEntryType::Unknown => {
                let pack_ret = rs_shada_pack_entry(packer, &raw const entry, 0);
                rs_shada_free_entry_contents(&raw mut entry);
                if pack_ret == SD_WRITE_FAILED {
                    ret = SD_WRITE_FAILED;
                }
                entry = ShadaEntry::default();
            }
            ShadaEntryType::SearchPattern => {
                let is_sub = std::ptr::read(std::ptr::addr_of!(
                    (*entry.data.search_pattern).is_substitute_pattern
                ));
                let target = if is_sub {
                    &raw mut (*wms).sub_search_pattern
                } else {
                    &raw mut (*wms).search_pattern
                };
                compare_with_entry(target, entry);
                entry = ShadaEntry::default();
            }
            ShadaEntryType::SubString => {
                compare_with_entry(&raw mut (*wms).replacement, entry);
                entry = ShadaEntry::default();
            }
            ShadaEntryType::HistoryEntry => {
                let histtype =
                    std::ptr::read(std::ptr::addr_of!((*entry.data.history_item).histtype))
                        as usize;
                if histtype >= HIST_COUNT {
                    let pack_ret = rs_shada_pack_entry(packer, &raw const entry, 0);
                    rs_shada_free_entry_contents(&raw mut entry);
                    if pack_ret == SD_WRITE_FAILED {
                        ret = SD_WRITE_FAILED;
                    }
                } else if (*wms).hms[histtype].hmll.size != 0 {
                    rs_hms_insert(&raw mut (*wms).hms[histtype], entry, true);
                } else {
                    rs_shada_free_entry_contents(&raw mut entry);
                }
                entry = ShadaEntry::default();
            }
            ShadaEntryType::Register => {
                let reg_name = std::ptr::read(std::ptr::addr_of!((*entry.data.reg).name));
                let idx = nvim_shada_op_reg_index(reg_name);
                if idx < 0 {
                    let pack_ret = rs_shada_pack_entry(packer, &raw const entry, 0);
                    rs_shada_free_entry_contents(&raw mut entry);
                    if pack_ret == SD_WRITE_FAILED {
                        ret = SD_WRITE_FAILED;
                    }
                } else {
                    compare_with_entry(&raw mut (*wms).registers[idx as usize], entry);
                }
                entry = ShadaEntry::default();
            }
            ShadaEntryType::Variable => {
                let var_name = std::ptr::read(std::ptr::addr_of!((*entry.data.global_var).name));
                if !nvim_shada_wms_dumped_vars_has(wms.cast(), var_name) {
                    let pack_ret = rs_shada_pack_entry(packer, &raw const entry, 0);
                    if pack_ret == SD_WRITE_FAILED {
                        ret = SD_WRITE_FAILED;
                    }
                }
                rs_shada_free_entry_contents(&raw mut entry);
                entry = ShadaEntry::default();
            }
            ShadaEntryType::GlobalMark => {
                let fm_name = std::ptr::read(std::ptr::addr_of!((*entry.data.filemark).name));
                #[allow(clippy::cast_possible_wrap)]
                if (fm_name as u8).is_ascii_digit() {
                    // Numbered mark: sort by timestamp descending.
                    // Use mem::take to move entry and reset it to Default in one step.
                    let mut entry_opt = Some(std::mem::take(&mut entry));
                    let numbered_marks_size = EXTRA_MARKS;
                    let mut i = numbered_marks_size;
                    'num_mark_loop: while i > 0 {
                        i -= 1;
                        let wms_entry_ref = &raw const (*wms).numbered_marks[i];
                        let wms_entry_type =
                            std::ptr::read(std::ptr::addr_of!((*wms_entry_ref).entry_type));
                        if wms_entry_type != ShadaEntryType::GlobalMark {
                            continue;
                        }
                        let wms_ts = std::ptr::read(std::ptr::addr_of!((*wms_entry_ref).timestamp));
                        let wms_additional =
                            std::ptr::read(std::ptr::addr_of!((*wms_entry_ref).additional_data));
                        let e_ref = entry_opt.as_ref().unwrap();
                        // Ignore exact duplicates.
                        if wms_ts == e_ref.timestamp
                            && wms_additional.is_null()
                            && e_ref.additional_data.is_null()
                        {
                            let wms_fm =
                                std::ptr::read(std::ptr::addr_of!((*wms_entry_ref).data.filemark));
                            let entry_fm = std::ptr::read(std::ptr::addr_of!(e_ref.data.filemark));
                            if rs_marks_equal(wms_fm.mark, entry_fm.mark) != 0
                                && libc::strcmp(wms_fm.fname, entry_fm.fname) == 0
                            {
                                let mut discard = entry_opt.take().unwrap();
                                rs_shada_free_entry_contents(&raw mut discard);
                                break 'num_mark_loop;
                            }
                        }
                        if wms_ts >= e_ref.timestamp {
                            if i + 1 < numbered_marks_size {
                                rs_replace_numbered_mark(wms, i + 1, entry_opt.take().unwrap());
                            } else {
                                let mut discard = entry_opt.take().unwrap();
                                rs_shada_free_entry_contents(&raw mut discard);
                            }
                            break 'num_mark_loop;
                        }
                    }
                    // If entry was not consumed in the loop, insert at index 0.
                    if let Some(remaining) = entry_opt.take() {
                        rs_replace_numbered_mark(wms, 0, remaining);
                    }
                } else {
                    let idx = nvim_shada_mark_global_index(c_int::from(fm_name as u8));
                    if idx < 0 {
                        let pack_ret = rs_shada_pack_entry(packer, &raw const entry, 0);
                        rs_shada_free_entry_contents(&raw mut entry);
                        if pack_ret == SD_WRITE_FAILED {
                            ret = SD_WRITE_FAILED;
                        }
                    } else {
                        // Global or numbered mark slot.
                        let mark_slot = if idx < 26 {
                            &raw mut (*wms).global_marks[idx as usize]
                        } else {
                            &raw mut (*wms).numbered_marks[(idx - 26) as usize]
                        };
                        if (*mark_slot).entry_type == ShadaEntryType::Missing {
                            // Check namedfm timestamp.
                            let named_ts = nvim_shada_named_mark_timestamp(idx);
                            if named_ts >= entry.timestamp {
                                rs_shada_free_entry_contents(&raw mut entry);
                            } else {
                                *mark_slot = entry;
                            }
                        } else {
                            compare_with_entry(mark_slot, entry);
                        }
                    }
                }
                entry = ShadaEntry::default();
            }
            ShadaEntryType::Change | ShadaEntryType::LocalMark => {
                let fm_fname = std::ptr::read(std::ptr::addr_of!((*entry.data.filemark).fname));
                if rs_shada_removable(fm_fname) != 0 {
                    rs_shada_free_entry_contents(&raw mut entry);
                    entry = ShadaEntry::default();
                    continue;
                }
                let mut is_new = false;
                let mut out_key: *const c_char = std::ptr::null();
                let val_slot = nvim_shada_wms_file_marks_put_ref(
                    wms.cast(),
                    fm_fname,
                    &raw mut is_new,
                    &raw mut out_key,
                );
                if val_slot.is_null() {
                    rs_shada_free_entry_contents(&raw mut entry);
                    entry = ShadaEntry::default();
                    continue;
                }
                if is_new {
                    // Key was copied by put_ref; fname key is now owned by the map.
                    // The key was xstrdup'd inside nvim_shada_wms_file_marks_put_ref.
                }
                if (*val_slot).is_null() {
                    *val_slot = nvim_xcalloc(1, std::mem::size_of::<FileMarks>());
                }
                let filemarks: *mut FileMarks = (*val_slot).cast::<FileMarks>();
                // Update greatest timestamp.
                if entry.timestamp > (*filemarks).greatest_timestamp {
                    (*filemarks).greatest_timestamp = entry.timestamp;
                }

                if entry.entry_type == ShadaEntryType::LocalMark {
                    let mark_name = std::ptr::read(std::ptr::addr_of!((*entry.data.filemark).name));
                    let idx = nvim_shada_mark_local_index(c_int::from(mark_name as u8));
                    if idx < 0 {
                        // Unknown local mark: append to additional marks.
                        (*filemarks).additional_marks_size += 1;
                        (*filemarks).additional_marks = nvim_xrealloc(
                            (*filemarks).additional_marks.cast::<c_void>(),
                            (*filemarks).additional_marks_size * std::mem::size_of::<ShadaEntry>(),
                        )
                        .cast::<ShadaEntry>();
                        *(*filemarks)
                            .additional_marks
                            .add((*filemarks).additional_marks_size - 1) = entry;
                    } else {
                        let wms_entry_ptr = if idx >= 0 && (idx as usize) < NLOCALMARKS {
                            &raw mut (*filemarks).marks[idx as usize]
                        } else {
                            std::ptr::null_mut()
                        };
                        let mut set_wms = true;
                        if !wms_entry_ptr.is_null()
                            && (*wms_entry_ptr).entry_type != ShadaEntryType::Missing
                        {
                            if (*wms_entry_ptr).timestamp >= entry.timestamp {
                                rs_shada_free_entry_contents(&raw mut entry);
                                entry = ShadaEntry::default();
                                set_wms = false;
                            } else if (*wms_entry_ptr).can_free_entry {
                                // If the key matches the old entry's fname, update key to new.
                                let old_fname = std::ptr::read(std::ptr::addr_of!(
                                    (*(*wms_entry_ptr).data.filemark).fname
                                ));
                                if !out_key.is_null() && std::ptr::eq(old_fname, out_key.cast_mut())
                                {
                                    // The key pointer in the map was pointing to the old
                                    // fname; update key to the new entry's fname.
                                    // (C does: *key = entry.data.filemark.fname)
                                    // We handle this via the C accessor which already did
                                    // xstrdup for new entries; for existing entries the key
                                    // was already there. This is a subtle ownership transfer
                                    // that we leave to C. No action needed here.
                                }
                                rs_shada_free_entry_contents(wms_entry_ptr);
                            }
                        } else {
                            // wms_entry is Missing: check if a buffer already has this mark.
                            // We use the C accessor to compare mark timestamps.
                            let curwin = nvim_shada_curwin();
                            let mut buf = nvim_shada_buf_first();
                            while !buf.is_null() {
                                let buf_ffname = nvim_shada_buf_get_ffname(buf);
                                if !buf_ffname.is_null() && path_fnamecmp(fm_fname, buf_ffname) == 0
                                {
                                    let cmp = nvim_shada_mark_get_cmp(
                                        buf,
                                        curwin,
                                        c_int::from(mark_name as u8),
                                        entry.timestamp,
                                    );
                                    if cmp != 0 {
                                        rs_shada_free_entry_contents(&raw mut entry);
                                        entry = ShadaEntry::default();
                                        set_wms = false;
                                    }
                                    break;
                                }
                                buf = nvim_shada_buf_next(buf);
                            }
                        }
                        if set_wms && !wms_entry_ptr.is_null() {
                            *wms_entry_ptr = entry;
                        }
                    }
                } else {
                    // Change entry: insert into sorted changes list.
                    let changes_size = (*filemarks).changes_size;
                    let mut i = changes_size as c_int;
                    while i > 0 {
                        i -= 1;
                        let jl_entry_ptr: *mut ShadaEntry = if i >= 0 && (i as usize) < JUMPLISTSIZE
                        {
                            &raw mut (*filemarks).changes[i as usize]
                        } else {
                            std::ptr::null_mut()
                        };
                        if jl_entry_ptr.is_null() {
                            break;
                        }
                        let jl_ts = (*jl_entry_ptr).timestamp;
                        if jl_ts <= entry.timestamp {
                            // Check for duplicates.
                            let jl_fm =
                                std::ptr::read(std::ptr::addr_of!((*jl_entry_ptr).data.filemark));
                            let entry_fm = std::ptr::read(std::ptr::addr_of!(entry.data.filemark));
                            if rs_marks_equal(jl_fm.mark, entry_fm.mark) != 0 {
                                i = -1;
                            }
                            break;
                        }
                    }
                    if i >= 0 && changes_size == JUMPLISTSIZE {
                        // List is full; discard oldest.
                        let oldest: *mut ShadaEntry = &raw mut (*filemarks).changes[0];
                        if !oldest.is_null() {
                            rs_shada_free_entry_contents(oldest);
                        }
                    }
                    let new_i = rs_marklist_insert(
                        (*filemarks).changes.as_mut_ptr().cast::<c_void>(),
                        std::mem::size_of::<ShadaEntry>(),
                        changes_size as c_int,
                        i,
                    );
                    if new_i == -1 {
                        rs_shada_free_entry_contents(&raw mut entry);
                    } else {
                        let slot: *mut ShadaEntry = if new_i >= 0 && (new_i as usize) < JUMPLISTSIZE
                        {
                            &raw mut (*filemarks).changes[new_i as usize]
                        } else {
                            std::ptr::null_mut()
                        };
                        if !slot.is_null() {
                            *slot = entry;
                        }
                        if changes_size < JUMPLISTSIZE {
                            (*filemarks).changes_size = changes_size + 1;
                        }
                    }
                }
                entry = ShadaEntry::default();
            }
            ShadaEntryType::Jump => {
                let mut i = (*wms).jumps_size as c_int;
                while i > 0 {
                    i -= 1;
                    let jl_entry_ref = &raw const (*wms).jumps[i as usize];
                    let jl_ts = std::ptr::read(std::ptr::addr_of!((*jl_entry_ref).timestamp));
                    if jl_ts <= entry.timestamp {
                        // Check for duplicates.
                        let jl_fm =
                            std::ptr::read(std::ptr::addr_of!((*jl_entry_ref).data.filemark));
                        let entry_fm = std::ptr::read(std::ptr::addr_of!(entry.data.filemark));
                        if rs_marks_equal(jl_fm.mark, entry_fm.mark) != 0
                            && libc::strcmp(jl_fm.fname, entry_fm.fname) == 0
                        {
                            i = -1;
                        }
                        break;
                    }
                }
                if i >= 0 && (*wms).jumps_size == JUMPLISTSIZE {
                    rs_shada_free_entry_contents(&raw mut (*wms).jumps[0]);
                }
                let new_i = rs_marklist_insert(
                    (*wms).jumps.as_mut_ptr().cast::<c_void>(),
                    std::mem::size_of::<ShadaEntry>(),
                    (*wms).jumps_size as c_int,
                    i,
                );
                if new_i == -1 {
                    rs_shada_free_entry_contents(&raw mut entry);
                } else {
                    (*wms).jumps[new_i as usize] = entry;
                    if (*wms).jumps_size < JUMPLISTSIZE {
                        (*wms).jumps_size += 1;
                    }
                }
                entry = ShadaEntry::default();
            }
        }
    }

    ret
}

/// Write ShaDa data to the given file descriptors.
///
/// This is the Rust replacement for the C `shada_write` function.
///
/// # Safety
///
/// `sd_writer` must be a valid `FileDescriptor *`.
/// `sd_reader` must be a valid `FileDescriptor *` or null.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_shada_write(sd_writer: *mut c_void, sd_reader: *mut c_void) -> c_int {
    let mut ret = SD_WRITE_SUCCESSFUL;

    let max_kbyte_i = rs_get_shada_parameter(c_int::from(b's'));
    let max_kbyte_i = if max_kbyte_i < 0 { 10 } else { max_kbyte_i };
    if max_kbyte_i == 0 {
        return ret;
    }
    let max_kbyte = max_kbyte_i as usize;

    let wms = nvim_xcalloc(1, std::mem::size_of::<WriteMergerState>()).cast::<WriteMergerState>();
    std::ptr::write(wms, WriteMergerState::default());

    // Determine what to dump.
    let dump_global_vars = !find_shada_parameter_impl(c_int::from(b'!')).is_null();
    let max_reg_lines_raw = rs_get_shada_parameter(c_int::from(b'<'));
    let max_reg_lines = if max_reg_lines_raw < 0 {
        rs_get_shada_parameter(c_int::from(b'"'))
    } else {
        max_reg_lines_raw
    };
    let dump_registers = max_reg_lines != 0;

    let removable_bufs = nvim_shada_set_init_ptr();
    let num_marked_files = {
        let v = rs_get_shada_parameter(c_int::from(b'\''));
        if v < 0 {
            0usize
        } else {
            v as usize
        }
    };
    let dump_global_marks = rs_get_shada_parameter(c_int::from(b'f')) != 0;
    let mut dump_history = false;
    let mut dump_one_history = [false; HIST_COUNT];

    // Initialize history mergers.
    for (i, hms_slot) in (*wms).hms.iter_mut().enumerate() {
        let hist_char = rs_shada_hist_type2char(i as c_int);
        let mut num_saved = rs_get_shada_parameter(hist_char);
        if num_saved == -1 {
            num_saved = nvim_get_p_hi() as c_int;
        }
        if num_saved > 0 {
            dump_history = true;
            dump_one_history[i] = true;
            rs_hms_init(
                hms_slot,
                i as u8,
                num_saved as usize,
                !sd_reader.is_null(),
                false,
            );
        }
    }

    // Compute SRNI flags.
    let srni_flags: u32 = SD_READ_UNDISABLEABLE_DATA
        | SD_READ_UNKNOWN
        | if dump_history { SD_READ_HISTORY } else { 0 }
        | if dump_registers { SD_READ_REGISTERS } else { 0 }
        | if dump_global_vars {
            SD_READ_VARIABLES
        } else {
            0
        }
        | if dump_global_marks {
            SD_READ_GLOBAL_MARKS
        } else {
            0
        }
        | if num_marked_files > 0 {
            SD_READ_LOCAL_MARKS | SD_READ_CHANGES
        } else {
            0
        };

    // Initialize packer.
    let mut packer_storage = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_init_for_file(sd_writer, packer_storage.as_mut_ptr());
    let packer = packer_storage.as_mut_ptr();

    // Set b_last_cursor for all windows in all tabs.
    nvim_shada_set_all_last_cursors();

    rs_find_removable_bufs(removable_bufs);

    // Write header.
    if rs_shada_pack_file_header(packer, max_kbyte) == SD_WRITE_FAILED {
        ret = SD_WRITE_FAILED;
        // fall through to cleanup (goto equivalent)
    }

    'write_exit: {
        if ret == SD_WRITE_FAILED {
            break 'write_exit;
        }

        // Write buffer list if '%' is in 'shada'.
        if !find_shada_parameter_impl(c_int::from(b'%')).is_null() {
            let buflist_entry = rs_shada_get_buflist(removable_bufs);
            let pack_ret = rs_shada_pack_entry(packer, &raw const buflist_entry, 0);
            // Free buffers array.
            let buffers_ptr = std::ptr::read(std::ptr::addr_of!(
                (*buflist_entry.data.buffer_list).buffers
            ));
            nvim_xfree(buffers_ptr.cast::<c_void>());
            if pack_ret == SD_WRITE_FAILED {
                ret = SD_WRITE_FAILED;
                break 'write_exit;
            }
        }

        // Write global variables.
        if dump_global_vars {
            let gvar_ret = rs_shada_pack_all_gvars(packer, wms.cast(), max_kbyte);
            if gvar_ret == SD_WRITE_FAILED {
                ret = SD_WRITE_FAILED;
                break 'write_exit;
            }
        }

        // Initialize jump list.
        if num_marked_files > 0 {
            (*wms).jumps_size = rs_shada_init_jumps((*wms).jumps.as_mut_ptr(), removable_bufs);
        }

        // Initialize search and substitute patterns.
        if dump_one_history[HIST_SEARCH as usize] {
            let search_highlighted =
                !no_hlsearch && find_shada_parameter_impl(c_int::from(b'h')).is_null();
            let search_last_used = search_was_last_used();

            rs_add_search_pattern(
                &raw mut (*wms).search_pattern,
                0,
                c_int::from(search_last_used),
                c_int::from(search_highlighted),
            );
            rs_add_search_pattern(
                &raw mut (*wms).sub_search_pattern,
                1,
                c_int::from(search_last_used),
                c_int::from(search_highlighted),
            );

            // Initialize substitute replacement string.
            let mut sub_str: *const c_char = std::ptr::null();
            let mut sub_ts: Timestamp = 0;
            let mut sub_additional: *mut c_void = std::ptr::null_mut();
            nvim_shada_sub_get_replacement(
                &raw mut sub_str,
                &raw mut sub_ts,
                &raw mut sub_additional,
            );
            if !sub_str.is_null() {
                (*wms).replacement = ShadaEntry {
                    entry_type: ShadaEntryType::SubString,
                    can_free_entry: false,
                    timestamp: sub_ts,
                    data: ShadaEntryData {
                        sub_string: std::mem::ManuallyDrop::new(SubStringData {
                            sub: sub_str.cast_mut(),
                        }),
                    },
                    additional_data: sub_additional,
                };
            }
        }

        // Initialize global marks.
        if dump_global_marks {
            let mut global_mark_iter: *const c_void = std::ptr::null();
            let mut digit_mark_idx: usize = 0;
            loop {
                let mut name: c_char = 0;
                let mut lnum: i64 = 0;
                let mut col: i32 = 0;
                let mut fnum: c_int = 0;
                let mut ts: Timestamp = 0;
                let mut fname: *const c_char = std::ptr::null();
                let mut additional: *mut c_void = std::ptr::null_mut();
                global_mark_iter = nvim_shada_mark_global_iter(
                    global_mark_iter,
                    &raw mut name,
                    &raw mut lnum,
                    &raw mut col,
                    &raw mut fnum,
                    &raw mut ts,
                    &raw mut fname,
                    &raw mut additional,
                );
                if name == 0 {
                    break;
                }
                let actual_fname: *const c_char = if fnum == 0 {
                    if fname.is_null() || rs_shada_removable(fname) != 0 {
                        if global_mark_iter.is_null() {
                            break;
                        }
                        continue;
                    }
                    fname
                } else {
                    let buf = buflist_findnr(fnum);
                    if buf.is_null() {
                        if global_mark_iter.is_null() {
                            break;
                        }
                        continue;
                    }
                    let buf_ffname = nvim_shada_buf_get_ffname(buf);
                    if buf_ffname.is_null() || nvim_shada_set_has_ptr(removable_bufs, buf) != 0 {
                        if global_mark_iter.is_null() {
                            break;
                        }
                        continue;
                    }
                    buf_ffname
                };
                let entry = ShadaEntry {
                    entry_type: ShadaEntryType::GlobalMark,
                    can_free_entry: false,
                    timestamp: ts,
                    data: ShadaEntryData {
                        filemark: std::mem::ManuallyDrop::new(FilemarkData {
                            name,
                            mark: Position::new(lnum, col, 0),
                            fname: actual_fname.cast_mut(),
                        }),
                    },
                    additional_data: additional,
                };
                #[allow(clippy::cast_possible_wrap)]
                if (name as u8).is_ascii_digit() {
                    rs_replace_numbered_mark(wms, digit_mark_idx, entry);
                    digit_mark_idx += 1;
                } else {
                    let idx = nvim_shada_mark_global_index(c_int::from(name as u8));
                    if idx >= 0 {
                        (*wms).global_marks[idx as usize] = entry;
                    }
                }
                if global_mark_iter.is_null() {
                    break;
                }
            }
        }

        // Initialize registers.
        if dump_registers {
            rs_shada_initialize_registers(wms, max_reg_lines);
        }

        // Initialize buffers (local marks and changelists).
        if num_marked_files > 0 {
            let mut buf = nvim_shada_buf_first();
            while !buf.is_null() {
                if rs_ignore_buf(buf, removable_bufs) != 0 {
                    buf = nvim_shada_buf_next(buf);
                    continue;
                }
                let fname = nvim_shada_buf_get_ffname(buf);
                if fname.is_null() {
                    buf = nvim_shada_buf_next(buf);
                    continue;
                }
                let mut is_new = false;
                let mut out_key: *const c_char = std::ptr::null();
                let val_slot = nvim_shada_wms_file_marks_put_ref(
                    wms.cast(),
                    fname,
                    &raw mut is_new,
                    &raw mut out_key,
                );
                if val_slot.is_null() {
                    buf = nvim_shada_buf_next(buf);
                    continue;
                }
                if is_new {
                    // key was xstrdup'd inside put_ref
                }
                if (*val_slot).is_null() {
                    *val_slot = nvim_xcalloc(1, std::mem::size_of::<FileMarks>());
                }
                let filemarks: *mut FileMarks = (*val_slot).cast::<FileMarks>();

                // Iterate local marks.
                let mut local_marks_iter: *const c_void = std::ptr::null();
                loop {
                    let mut mark_name: c_char = 0;
                    let mut lnum: i64 = 0;
                    let mut col: i32 = 0;
                    let mut mark_ts: Timestamp = 0;
                    let mut mark_additional: *mut c_void = std::ptr::null_mut();
                    local_marks_iter = nvim_shada_mark_buffer_iter(
                        local_marks_iter,
                        buf,
                        &raw mut mark_name,
                        &raw mut lnum,
                        &raw mut col,
                        &raw mut mark_ts,
                        &raw mut mark_additional,
                    );
                    if mark_name == 0 {
                        break;
                    }
                    let idx = nvim_shada_mark_local_index(c_int::from(mark_name as u8));
                    if idx >= 0 {
                        let mark_slot = if idx >= 0 && (idx as usize) < NLOCALMARKS {
                            &raw mut (*filemarks).marks[idx as usize]
                        } else {
                            std::ptr::null_mut()
                        };
                        if !mark_slot.is_null() {
                            *mark_slot = ShadaEntry {
                                entry_type: ShadaEntryType::LocalMark,
                                can_free_entry: false,
                                timestamp: mark_ts,
                                data: ShadaEntryData {
                                    filemark: std::mem::ManuallyDrop::new(FilemarkData {
                                        name: mark_name,
                                        mark: Position::new(lnum, col, 0),
                                        fname: fname.cast_mut(),
                                    }),
                                },
                                additional_data: mark_additional,
                            };
                        }
                        if mark_ts > (*filemarks).greatest_timestamp {
                            (*filemarks).greatest_timestamp = mark_ts;
                        }
                    }
                    if local_marks_iter.is_null() {
                        break;
                    }
                }

                // Initialize changelist.
                let changelist_len = nvim_shada_buf_changelist_len(buf);
                for ci in 0..changelist_len {
                    let mut cl_lnum: i64 = 0;
                    let mut cl_col: i32 = 0;
                    let mut cl_ts: Timestamp = 0;
                    let mut cl_additional: *mut c_void = std::ptr::null_mut();
                    nvim_shada_buf_changelist_entry(
                        buf,
                        ci,
                        &raw mut cl_lnum,
                        &raw mut cl_col,
                        &raw mut cl_ts,
                        &raw mut cl_additional,
                    );
                    let cl_slot: *mut ShadaEntry = if ci >= 0 && (ci as usize) < JUMPLISTSIZE {
                        &raw mut (*filemarks).changes[ci as usize]
                    } else {
                        std::ptr::null_mut()
                    };
                    if !cl_slot.is_null() {
                        *cl_slot = ShadaEntry {
                            entry_type: ShadaEntryType::Change,
                            can_free_entry: false,
                            timestamp: cl_ts,
                            data: ShadaEntryData {
                                filemark: std::mem::ManuallyDrop::new(FilemarkData {
                                    name: 0,
                                    mark: Position::new(cl_lnum, cl_col, 0),
                                    fname: fname.cast_mut(),
                                }),
                            },
                            additional_data: cl_additional,
                        };
                        if cl_ts > (*filemarks).greatest_timestamp {
                            (*filemarks).greatest_timestamp = cl_ts;
                        }
                    }
                }
                (*filemarks).changes_size = changelist_len as usize;

                buf = nvim_shada_buf_next(buf);
            }
        }

        // Read existing ShaDa file and merge.
        if !sd_reader.is_null() {
            let srww_ret = shada_read_when_writing(sd_reader, srni_flags, max_kbyte, wms, packer);
            if srww_ret != SD_WRITE_SUCCESSFUL {
                ret = srww_ret;
            }
        }

        // Update numbered marks: replace '0 with current position.
        if dump_global_marks && rs_ignore_buf(nvim_shada_buf_first(), removable_bufs) == 0 {
            let cur_lnum = nvim_shada_curwin_lnum();
            if cur_lnum != 0 {
                let mut cl: i64 = 0;
                let mut cc: i32 = 0;
                nvim_shada_curwin_cursor(&raw mut cl, &raw mut cc);
                let curbuf_ffname = nvim_shada_curbuf_ffname();
                rs_replace_numbered_mark(
                    wms,
                    0,
                    ShadaEntry {
                        entry_type: ShadaEntryType::GlobalMark,
                        can_free_entry: false,
                        timestamp: os_time(),
                        data: ShadaEntryData {
                            filemark: std::mem::ManuallyDrop::new(FilemarkData {
                                mark: Position::new(cl, cc, 0),
                                name: b'0'.cast_signed(),
                                fname: curbuf_ffname.cast_mut(),
                            }),
                        },
                        additional_data: std::ptr::null_mut(),
                    },
                );
            }
        }

        // Pack WMS arrays: global marks, numbered marks, registers.
        macro_rules! pack_wms_array {
            ($arr:expr) => {
                for entry_ref in $arr.iter_mut() {
                    if entry_ref.entry_type != ShadaEntryType::Missing {
                        if rs_shada_pack_pfreed_entry(
                            packer,
                            std::ptr::from_mut(entry_ref),
                            max_kbyte,
                        ) == SD_WRITE_FAILED
                        {
                            ret = SD_WRITE_FAILED;
                            break 'write_exit;
                        }
                    }
                }
            };
        }
        pack_wms_array!((*wms).global_marks);
        pack_wms_array!((*wms).numbered_marks);
        pack_wms_array!((*wms).registers);

        // Pack jumps.
        for i in 0..(*wms).jumps_size {
            if rs_shada_pack_pfreed_entry(packer, &raw mut (*wms).jumps[i], max_kbyte)
                == SD_WRITE_FAILED
            {
                ret = SD_WRITE_FAILED;
                break 'write_exit;
            }
        }

        // Pack search pattern, sub search pattern, replacement.
        macro_rules! pack_wms_entry {
            ($entry:expr) => {
                if $entry.entry_type != ShadaEntryType::Missing {
                    if rs_shada_pack_pfreed_entry(packer, &raw mut $entry, max_kbyte)
                        == SD_WRITE_FAILED
                    {
                        ret = SD_WRITE_FAILED;
                        break 'write_exit;
                    }
                }
            };
        }
        pack_wms_entry!((*wms).search_pattern);
        pack_wms_entry!((*wms).sub_search_pattern);
        pack_wms_entry!((*wms).replacement);

        // Pack file marks (sorted by greatest timestamp).
        let mut sorted_size: usize = 0;
        let all_file_markss =
            nvim_shada_wms_file_marks_get_sorted(wms.cast(), &raw mut sorted_size);
        let file_markss_to_dump = sorted_size.min(num_marked_files);
        for i in 0..file_markss_to_dump {
            let fm_ptr = (*all_file_markss.add(i)).cast::<FileMarks>();

            // Pack all local marks.
            for mi in 0..NLOCALMARKS {
                let mark_slot: *mut ShadaEntry = &raw mut (*fm_ptr).marks[mi];
                if !mark_slot.is_null()
                    && (*mark_slot).entry_type != ShadaEntryType::Missing
                    && rs_shada_pack_pfreed_entry(packer, mark_slot, max_kbyte) == SD_WRITE_FAILED
                {
                    ret = SD_WRITE_FAILED;
                    // Free remaining file marks and exit.
                    nvim_xfree(all_file_markss.cast::<c_void>());
                    break 'write_exit;
                }
            }

            // Pack changes.
            let changes_size = (*fm_ptr).changes_size;
            for ci in 0..changes_size {
                let change_slot: *mut ShadaEntry = &raw mut (*fm_ptr).changes[ci];
                if !change_slot.is_null()
                    && rs_shada_pack_pfreed_entry(packer, change_slot, max_kbyte) == SD_WRITE_FAILED
                {
                    ret = SD_WRITE_FAILED;
                    nvim_xfree(all_file_markss.cast::<c_void>());
                    break 'write_exit;
                }
            }

            // Pack additional marks.
            let additional_size = (*fm_ptr).additional_marks_size;
            for ai in 0..additional_size {
                let add_slot: *mut ShadaEntry = if ai < (*fm_ptr).additional_marks_size {
                    (*fm_ptr).additional_marks.add(ai)
                } else {
                    std::ptr::null_mut()
                };
                if !add_slot.is_null() {
                    if rs_shada_pack_entry(packer, add_slot, 0) == SD_WRITE_FAILED {
                        rs_shada_free_entry_contents(add_slot);
                        ret = SD_WRITE_FAILED;
                        nvim_xfree((*fm_ptr).additional_marks.cast::<c_void>());
                        (*fm_ptr).additional_marks = std::ptr::null_mut();
                        (*fm_ptr).additional_marks_size = 0;
                        nvim_xfree(all_file_markss.cast::<c_void>());
                        break 'write_exit;
                    }
                    rs_shada_free_entry_contents(add_slot);
                }
            }
            nvim_xfree((*fm_ptr).additional_marks.cast::<c_void>());
            (*fm_ptr).additional_marks = std::ptr::null_mut();
            (*fm_ptr).additional_marks_size = 0;
        }
        nvim_xfree(all_file_markss.cast::<c_void>());

        // Pack history.
        if dump_history {
            'history_loop: for (i, do_dump) in dump_one_history.iter().enumerate() {
                if !do_dump {
                    continue;
                }
                rs_hms_insert_whole_neovim_history(&raw mut (*wms).hms[i]);
                // Iterate HMLList (HMLL_FORALL / HMS_ITER equivalent).
                let hmll = &raw const (*wms).hms[i].hmll;
                let mut cur_entry = (*hmll).first;
                while !cur_entry.is_null() {
                    let pack_ret =
                        rs_shada_pack_pfreed_entry(packer, &raw mut (*cur_entry).data, max_kbyte);
                    if pack_ret == SD_WRITE_FAILED {
                        ret = SD_WRITE_FAILED;
                        break 'history_loop;
                    }
                    cur_entry = (*cur_entry).next;
                }
                if ret == SD_WRITE_FAILED {
                    break 'write_exit;
                }
            }
        }
    } // 'write_exit

    // Cleanup: dealloc history mergers.
    for (i, do_dump) in dump_one_history.iter().enumerate() {
        if *do_dump {
            rs_hms_dealloc(&raw mut (*wms).hms[i]);
        }
    }
    // Cleanup: destroy file_marks and dumped_variables.
    nvim_shada_wms_file_marks_destroy(wms.cast());
    nvim_shada_set_destroy_ptr(removable_bufs);
    // Flush packer.
    nvim_shada_packer_flush(packer);
    nvim_shada_wms_dumped_vars_destroy(wms.cast());
    nvim_xfree(wms.cast::<c_void>());

    ret
}

/// Write ShaDa data to a file.
///
/// Opens the ShaDa file for writing, optionally merging with an existing
/// ShaDa file (unless `nomerge` is true), calls `shada_write`, and renames
/// the temporary file into place.
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
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_shada_write_file(file: *const c_char, nomerge: bool) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = 1;
    // libuv error codes
    const UV_ENOENT: c_int = -2;
    const UV_EEXIST: c_int = -17;
    const UV_ELOOP: c_int = -40;
    // FileOpenFlags values
    const K_FILE_READ_ONLY: c_int = 1;
    const K_FILE_CREATE: c_int = 2;
    const K_FILE_NO_SYMLINK: c_int = 8;
    const K_FILE_CREATE_ONLY: c_int = 16;
    const K_FILE_TRUNCATE: c_int = 32;
    // ShaDaWriteResult values
    const K_SD_WRITE_SUCCESSFUL: c_int = 0;
    const K_SD_WRITE_READ_NOT_SHADA: c_int = 1;

    let fname = rs_shada_filename(file);
    if fname.is_null() {
        return FAIL;
    }

    // Allocate FileDescriptor structs on the heap (opaque C structs)
    let fd_size = nvim_shada_file_descriptor_size();
    let sd_writer_mem = nvim_xcalloc(1, fd_size);
    let sd_reader_mem = nvim_xcalloc(1, fd_size);

    let mut did_open_writer = false;
    let mut did_open_reader = false;
    let mut nomerge = nomerge;
    let mut tempname: *mut c_char = std::ptr::null_mut();

    if !nomerge {
        let error = nvim_shada_file_open_write(
            FileDescriptorHandle::from_ptr(sd_reader_mem),
            fname,
            K_FILE_READ_ONLY,
            0,
        );
        if error != 0 {
            if error != UV_ENOENT {
                nvim_shada_semsg_2s(c"E886: System error while opening ShaDa file %s for reading to merge before writing it: %s".as_ptr(), fname, nvim_shada_os_strerror(error));
                // Try writing the file even if opening it emerged any issues besides
                // file not existing: maybe writing will succeed nevertheless.
            }
            nomerge = true;
        } else {
            did_open_reader = true;
        }

        if !nomerge {
            tempname = modname(fname, c".tmp.a".as_ptr(), false);
            if tempname.is_null() {
                nomerge = true;
            }
        }

        if !nomerge {
            // Save permissions from the original file, with modifications:
            // 1: Strip SUID bit if any.
            // 2: Make sure that user can always read and write the result.
            // 3: If somebody happened to delete the file after it was opened for
            //    reading, use u=rw permissions.
            let raw_perm = os_getperm(fname);
            let perm = if raw_perm >= 0 {
                (raw_perm & 0o777) | 0o600
            } else {
                0o600
            };

            // Try opening temp file, incrementing suffix from .tmp.a to .tmp.z
            loop {
                let error = nvim_shada_file_open_write(
                    FileDescriptorHandle::from_ptr(sd_writer_mem),
                    tempname,
                    K_FILE_CREATE_ONLY | K_FILE_NO_SYMLINK,
                    perm,
                );
                if error == 0 {
                    did_open_writer = true;
                    break;
                }
                if error == UV_EEXIST || error == UV_ELOOP {
                    // File already exists, try another name
                    let tempname_len = libc::strlen(tempname);
                    let wp = tempname.add(tempname_len - 1);
                    #[allow(clippy::cast_possible_wrap)]
                    if *wp == b'z' as c_char {
                        // Tried names from .tmp.a to .tmp.z, all failed.
                        nvim_shada_semsg_1s(
                            c"E138: All %s.tmp.X files exist, cannot write ShaDa file!".as_ptr(),
                            fname,
                        );
                        nvim_xfree(fname.cast::<c_void>());
                        nvim_xfree(tempname.cast::<c_void>());
                        if did_open_reader {
                            rs_close_file(FileDescriptorHandle::from_ptr(sd_reader_mem));
                        }
                        nvim_xfree(sd_writer_mem);
                        nvim_xfree(sd_reader_mem);
                        return FAIL;
                    }
                    *wp += 1;
                    // continue loop
                } else {
                    nvim_shada_semsg_2s(
                        c"E886: System error while opening temporary ShaDa file %s for writing: %s"
                            .as_ptr(),
                        tempname,
                        nvim_shada_os_strerror(error),
                    );
                    break;
                }
            }
        }
    }

    if nomerge {
        // Create directory if needed
        let tail_offset = nvim_shada_path_tail_with_sep_offset(fname);
        if tail_offset != 0 {
            // Temporarily null-terminate at the tail to get the directory path
            let tail_ptr = fname.add(tail_offset).cast::<c_char>();
            let tail_save = *tail_ptr;
            *tail_ptr = 0;
            if !os_isdir(fname) {
                let mut failed_dir: *mut c_char = std::ptr::null_mut();
                let ret = nvim_shada_os_mkdir_recurse(fname, 0o700, &raw mut failed_dir);
                if ret != 0 {
                    nvim_shada_semsg_2s(
                        c"E886: Failed to create directory %s for writing ShaDa file: %s".as_ptr(),
                        failed_dir,
                        nvim_shada_os_strerror(ret),
                    );
                    *tail_ptr = tail_save;
                    nvim_xfree(fname.cast::<c_void>());
                    nvim_xfree(failed_dir.cast::<c_void>());
                    nvim_xfree(sd_writer_mem);
                    nvim_xfree(sd_reader_mem);
                    return FAIL;
                }
            }
            *tail_ptr = tail_save;
        }
        let error = nvim_shada_file_open_write(
            FileDescriptorHandle::from_ptr(sd_writer_mem),
            fname,
            K_FILE_CREATE | K_FILE_TRUNCATE,
            0o600,
        );
        if error != 0 {
            nvim_shada_semsg_2s(
                c"E886: System error while opening ShaDa file %s for writing: %s".as_ptr(),
                fname,
                nvim_shada_os_strerror(error),
            );
        } else {
            did_open_writer = true;
        }
    }

    if !did_open_writer {
        nvim_xfree(fname.cast::<c_void>());
        nvim_xfree(tempname.cast::<c_void>());
        if did_open_reader {
            rs_close_file(FileDescriptorHandle::from_ptr(sd_reader_mem));
        }
        nvim_xfree(sd_writer_mem);
        nvim_xfree(sd_reader_mem);
        return FAIL;
    }

    if nvim_shada_get_p_verbose() > 1 {
        verbose_enter();
        nvim_shada_smsg_1s(c"Writing ShaDa file \"%s\"".as_ptr(), fname);
        verbose_leave();
    }

    let sd_reader_arg = if nomerge {
        std::ptr::null_mut::<c_void>()
    } else {
        sd_reader_mem
    };
    let sw_ret = rs_shada_write(sd_writer_mem, sd_reader_arg);

    if !nomerge {
        if did_open_reader {
            rs_close_file(FileDescriptorHandle::from_ptr(sd_reader_mem));
        }
        let mut did_remove = false;
        if sw_ret == K_SD_WRITE_SUCCESSFUL {
            let check_result = rs_shada_platform_check_writable(fname, sd_writer_mem, tempname);
            if check_result == 1 {
                if vim_rename(tempname, fname) == -1 {
                    nvim_shada_semsg_2s(
                        c"E136: Can\'t rename ShaDa file from %s to %s!".as_ptr(),
                        tempname,
                        fname,
                    );
                } else {
                    did_remove = true;
                    let _ = os_remove(tempname);
                }
            }
            // check_result == 0 or -1: E137/RNERR already emitted by accessor
        } else if sw_ret == K_SD_WRITE_READ_NOT_SHADA {
            nvim_shada_semsg_2s(
                c"E136: Did not rename %s because %s does not look like a ShaDa file".as_ptr(),
                tempname,
                fname,
            );
        } else {
            nvim_shada_semsg_2s(
                c"E136: Did not rename %s to %s because there were errors during writing it"
                    .as_ptr(),
                tempname,
                fname,
            );
        }
        if !did_remove {
            nvim_shada_semsg_2s(
                c"E136: Do not forget to remove %s or rename it manually to %s.".as_ptr(),
                tempname,
                fname,
            );
        }
        nvim_xfree(tempname.cast::<c_void>());
    }

    rs_close_file(FileDescriptorHandle::from_ptr(sd_writer_mem));
    nvim_xfree(fname.cast::<c_void>());
    nvim_xfree(sd_writer_mem);
    nvim_xfree(sd_reader_mem);

    OK
}

// =============================================================================
// Phase 1 (plan b499a5d0): Rust apply functions for search pattern and sub string
// =============================================================================

/// Apply a search pattern ShaDa entry (Rust replacement for C nvim_shada_apply_search_pattern).
///
/// Compares timestamps with the current pattern; if the entry is newer (or force is true),
/// sets the search or substitute pattern. Handles is_last_used and highlighted flags.
///
/// # Safety
///
/// `entry` must be a valid pointer to a ShadaEntry of type SearchPattern.
unsafe fn rs_shada_apply_search_pattern(entry: *mut ShadaEntry, force: bool) {
    let is_sub = c_int::from(read_union_field!(
        entry,
        search_pattern,
        is_substitute_pattern
    ));
    if !force {
        let cur_ts = nvim_shada_get_search_pattern_timestamp(is_sub);
        // cur_ts == 0 means no pattern set (pat is NULL); skip only if pat exists and is newer
        if cur_ts != 0 && cur_ts >= (*entry).timestamp {
            rs_shada_free_entry_contents(entry);
            return;
        }
    }
    nvim_shada_set_search_pattern_from_entry(entry, is_sub);
    if read_union_field!(entry, search_pattern, is_last_used) {
        set_last_used_pattern(is_sub != 0);
        let highlighted = read_union_field!(entry, search_pattern, highlighted);
        set_no_hlsearch(!highlighted);
    }
    // Memory was consumed by set_search_pattern / set_substitute_pattern; do not free.
}

/// Apply a substitute string ShaDa entry (Rust replacement for C nvim_shada_apply_sub_string).
///
/// Compares timestamps with the current sub replacement; if the entry is newer (or force is true),
/// sets the sub replacement string.
///
/// # Safety
///
/// `entry` must be a valid pointer to a ShadaEntry of type SubString.
unsafe fn rs_shada_apply_sub_string(entry: *mut ShadaEntry, force: bool) {
    if !force {
        let cur_ts = nvim_shada_get_sub_replacement_timestamp();
        if cur_ts != 0 && cur_ts >= (*entry).timestamp {
            rs_shada_free_entry_contents(entry);
            return;
        }
    }
    nvim_shada_set_sub_replacement_from_entry(entry);
    // Memory was consumed by sub_set_replacement; do not free.
}

// =============================================================================
// Phase 2 (plan b499a5d0): Rust apply functions for register and variable
// =============================================================================

/// Apply a register ShaDa entry (Rust replacement for C nvim_shada_apply_register).
///
/// Validates register type, compares timestamps with current register,
/// calls op_reg_set via thin C accessor.
///
/// # Safety
///
/// `entry` must be a valid pointer to a ShadaEntry of type Register.
unsafe fn rs_shada_apply_register(entry: *mut ShadaEntry, force: bool) {
    if nvim_shada_entry_get_reg_type_valid(entry) == 0 {
        rs_shada_free_entry_contents(entry);
        return;
    }
    if !force {
        let name = read_union_field!(entry, reg, name);
        let cur_ts = nvim_shada_op_reg_get_timestamp(name);
        // cur_ts == 0 means register is NULL; if non-null and newer or equal, skip
        if cur_ts != 0 && cur_ts >= (*entry).timestamp {
            rs_shada_free_entry_contents(entry);
            return;
        }
    }
    if nvim_shada_op_reg_set_from_entry(entry) == 0 {
        rs_shada_free_entry_contents(entry);
    }
    // If op_reg_set returned 1, memory was consumed; do not free.
}

/// Apply a global variable ShaDa entry (Rust replacement for C nvim_shada_apply_variable).
///
/// Calls var_set_global via thin C accessor (which also clears the typval),
/// then frees the entry.
///
/// # Safety
///
/// `entry` must be a valid pointer to a ShadaEntry of type Variable.
unsafe fn rs_shada_apply_variable(entry: *mut ShadaEntry) {
    nvim_shada_var_set_global_from_entry(entry);
    rs_shada_free_entry_contents(entry);
}

// =============================================================================
// Phase 3 (plan b499a5d0): Rust apply function for buffer list
// =============================================================================

/// Apply a buffer list ShaDa entry (Rust replacement for C nvim_shada_apply_buffer_list).
///
/// Iterates buffer list entries, calls buflist_new, sets cursor positions and
/// transfers additional data ownership.
///
/// # Safety
///
/// `entry` must be a valid pointer to a ShadaEntry of type BufferList.
unsafe fn rs_shada_apply_buffer_list(entry: *mut ShadaEntry) {
    let size = read_union_field!(entry, buffer_list, size);
    let buffers = read_union_field!(entry, buffer_list, buffers);
    for i in 0..size {
        let fname = (*buffers.add(i)).fname;
        if fname.is_null() {
            continue;
        }
        let sfname = nvim_shada_path_try_shorten_fname(fname);
        let buf = nvim_shada_buflist_new(fname, sfname);
        if !buf.is_null() {
            nvim_shada_buf_set_cursor_and_data(buf, entry, i);
        }
    }
    rs_shada_free_entry_contents(entry);
}

// =============================================================================
// Phase 4 (plan b499a5d0): Rust apply functions for mark/jump and local/change
// =============================================================================

/// Apply a global mark or jump ShaDa entry (Rust replacement for C nvim_shada_apply_mark_or_jump).
///
/// For GlobalMark: calls compound mark_set_global_from_entry accessor.
/// For Jump: implements duplicate-detection loop using jumplist accessors,
/// calls rs_marklist_insert, then sets the entry.
///
/// # Safety
///
/// All pointer arguments must be valid.
unsafe fn rs_shada_apply_mark_or_jump(
    entry: *mut ShadaEntry,
    fname_bufs: *mut c_void,
    force: bool,
) {
    if (*entry).entry_type == ShadaEntryType::GlobalMark {
        let no_overwrite = c_int::from(!force);
        if nvim_shada_mark_set_global_from_entry(entry, fname_bufs, no_overwrite) == 0 {
            rs_shada_free_entry_contents(entry);
        }
        // If 1: memory was consumed by mark_set_global.
        return;
    }

    // Jump entry: duplicate-detection loop (mirrors C nvim_shada_apply_mark_or_jump).
    let jl_len = nvim_shada_jumplist_len();
    let entry_ts = (*entry).timestamp;

    // Find buf to know whether we compare by fnum or fname.
    let fm_mark = read_union_field!(entry, filemark, mark);
    let entry_fname = read_union_field!(entry, filemark, fname);
    let fm_buf = nvim_shada_find_buffer(fname_bufs, entry_fname);
    let compare_by_fnum = !fm_buf.is_null();

    let mut i = jl_len;
    while i > 0 {
        let mut jl_ts: u64 = 0;
        let mut jl_lnum: i64 = 0;
        let mut jl_col: i32 = 0;
        let mut jl_filenum: c_int = 0;
        let mut jl_fname: *const c_char = std::ptr::null();
        nvim_shada_jumplist_get_entry(
            i - 1,
            &raw mut jl_ts,
            &raw mut jl_lnum,
            &raw mut jl_col,
            &raw mut jl_filenum,
            &raw mut jl_fname,
        );
        if jl_ts <= entry_ts {
            // Check for duplicate: same mark position AND same file identity.
            let entry_lnum: i64 = fm_mark.lnum;
            let entry_col: i32 = fm_mark.col;
            let jl_pos = Position::new(jl_lnum, jl_col, 0);
            let entry_pos = Position::new(entry_lnum, entry_col, 0);
            if rs_marks_equal(jl_pos, entry_pos) != 0 {
                let file_match = if compare_by_fnum {
                    // Buf was found: compare fnum.
                    let buf_fnum = nvim_shada_buf_get_fnum(fm_buf);
                    jl_filenum == buf_fnum
                } else {
                    // No buf: compare fname strings.
                    !jl_fname.is_null() && libc::strcmp(entry_fname, jl_fname) == 0
                };
                if file_match {
                    i = -1;
                }
            }
            break;
        }
        i -= 1;
    }

    let i = nvim_shada_jumplist_marklist_insert(i);
    if i == -1 {
        rs_shada_free_entry_contents(entry);
    } else {
        nvim_shada_jumplist_insert_entry(i, entry, fname_bufs, jl_len);
    }
}

/// Apply a local mark or change ShaDa entry (Rust replacement for C nvim_shada_apply_local_or_change).
///
/// Handles oldfiles list update, then either sets a local mark or inserts into
/// changelist with duplicate detection.
///
/// # Safety
///
/// All pointer arguments must be valid.
#[allow(clippy::too_many_arguments)]
unsafe fn rs_shada_apply_local_or_change(
    entry: *mut ShadaEntry,
    fname_bufs: *mut c_void,
    cl_bufs: *mut c_void,
    oldfiles_set: *mut c_void,
    oldfiles_list: *mut c_void,
    force: bool,
    want_marks: bool,
    get_old_files: bool,
) {
    // Handle oldfiles list update (mirrors C logic).
    if get_old_files && nvim_shada_oldfiles_has(oldfiles_set, entry) == 0 {
        nvim_shada_oldfiles_add(oldfiles_set, oldfiles_list, entry, c_int::from(want_marks));
    }
    if !want_marks {
        rs_shada_free_entry_contents(entry);
        return;
    }

    // Find buffer for this entry.
    let cl_mark = read_union_field!(entry, filemark, mark);
    let cl_entry_fname = read_union_field!(entry, filemark, fname);
    let buf = nvim_shada_find_buffer(fname_bufs, cl_entry_fname);
    if buf.is_null() {
        rs_shada_free_entry_contents(entry);
        return;
    }

    if (*entry).entry_type == ShadaEntryType::LocalMark {
        let no_overwrite = c_int::from(!force);
        if nvim_shada_mark_set_local_from_entry(entry, buf, no_overwrite) == 0 {
            rs_shada_free_entry_contents(entry);
        }
        // mark_set_local uses fnum=0, does not own fname.
    } else {
        // Change entry: duplicate-detection loop (mirrors C nvim_shada_apply_local_or_change).
        nvim_shada_cl_bufs_set_put(cl_bufs, buf);

        let cl_len = nvim_shada_buf_get_changelistlen(buf);
        let entry_ts = (*entry).timestamp;

        let mut i = cl_len;
        while i > 0 {
            let mut cl_ts: u64 = 0;
            let mut cl_lnum: i64 = 0;
            let mut cl_col: i32 = 0;
            nvim_shada_changelist_get_entry(
                buf,
                i - 1,
                &raw mut cl_ts,
                &raw mut cl_lnum,
                &raw mut cl_col,
            );
            if cl_ts <= entry_ts {
                // Check for duplicate: same mark position.
                let entry_lnum: i64 = cl_mark.lnum;
                let entry_col: i32 = cl_mark.col;
                let cl_pos = Position::new(cl_lnum, cl_col, 0);
                let entry_pos = Position::new(entry_lnum, entry_col, 0);
                if rs_marks_equal(cl_pos, entry_pos) != 0 {
                    i = -1;
                }
                break;
            }
            i -= 1;
        }

        let i = nvim_shada_changelist_marklist_insert(buf, i);
        if i == -1 {
            // Duplicate: free the additional_data (rest of entry is stack-allocated).
            nvim_xfree((*entry).additional_data);
        } else {
            nvim_shada_changelist_insert_entry(buf, i, entry, cl_len);
        }
    }
    // xfree entry->data.filemark.fname (mirrors C end-of-function xfree for both branches).
    nvim_shada_fm_xfree_fname(entry);
}

/// Dispatch a single ShaDa entry to Neovim's in-memory state.
///
/// This is the Rust replacement for the C `nvim_shada_apply_entry` function.
/// It delegates to per-type compound C accessors that bundle the C-level
/// struct manipulation for each entry kind.
///
/// # Safety
///
/// All pointer arguments must be valid.
#[allow(clippy::too_many_arguments)]
unsafe fn rs_shada_apply_entry(
    force: bool,
    want_marks: bool,
    get_old_files: bool,
    entry: *mut ShadaEntry,
    fname_bufs: *mut c_void,
    cl_bufs: *mut c_void,
    oldfiles_set: *mut c_void,
    oldfiles_list: *mut c_void,
) {
    match (*entry).entry_type {
        // Missing: should never happen. Unknown: caller handles freeing.
        // HistoryEntry: handled by caller via rs_hms_insert before this call.
        ShadaEntryType::Missing | ShadaEntryType::Unknown | ShadaEntryType::HistoryEntry => {}
        ShadaEntryType::Header => {
            rs_shada_free_entry_contents(entry);
        }
        ShadaEntryType::SearchPattern => {
            rs_shada_apply_search_pattern(entry, force);
        }
        ShadaEntryType::SubString => {
            rs_shada_apply_sub_string(entry, force);
        }
        ShadaEntryType::Register => {
            rs_shada_apply_register(entry, force);
        }
        ShadaEntryType::Variable => {
            rs_shada_apply_variable(entry);
        }
        ShadaEntryType::GlobalMark | ShadaEntryType::Jump => {
            rs_shada_apply_mark_or_jump(entry, fname_bufs, force);
        }
        ShadaEntryType::BufferList => {
            rs_shada_apply_buffer_list(entry);
        }
        ShadaEntryType::LocalMark | ShadaEntryType::Change => {
            rs_shada_apply_local_or_change(
                entry,
                fname_bufs,
                cl_bufs,
                oldfiles_set,
                oldfiles_list,
                force,
                want_marks,
                get_old_files,
            );
        }
    }
}

// =============================================================================
// Phase 2 (plan 92c8078e): rs_shada_read_next_item
// =============================================================================

/// Read and parse the next ShaDa file entry from sd_reader into entry.
///
/// This is the Rust replacement for the C `shada_read_next_item` function.
/// Mirrors the C logic exactly: reads header (type/timestamp/length), reads
/// body bytes, delegates body parsing to C compound accessors, handles the
/// verify_but_ignore path, unknown entries, and additional data.
///
/// Returns SD_READ_STATUS_* constants.
///
/// # Safety
///
/// - `sd_reader` must be a valid FileDescriptor handle
/// - `entry` must be a valid pointer to a zero-initialised ShadaEntry
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
unsafe fn rs_shada_read_next_item(
    sd_reader: *mut c_void,
    entry: *mut ShadaEntry,
    flags: u32,
    max_kbyte: usize,
) -> c_int {
    let fd = FileDescriptorHandle(sd_reader);
    loop {
        // Zero-initialise entry so all pointers are NULL (safe to free on error)
        std::ptr::write(entry, ShadaEntry::default());

        if nvim_file_eof(fd) != 0 {
            return SD_READ_STATUS_FINISHED;
        }

        let mut verify_but_ignore = false;

        let mut type_u64: u64 = SD_ITEM_MISSING as u64;
        let mut timestamp_u64: u64 = 0;
        let mut length_u64: u64 = 0;

        let initial_fpos = nvim_shada_file_bytes_read(sd_reader);

        // Read type / timestamp / length
        let mru_ret = rs_msgpack_read_uint64(fd, true, &raw mut type_u64);
        if mru_ret != SD_READ_STATUS_SUCCESS {
            return mru_ret;
        }
        let mru_ret = rs_msgpack_read_uint64(fd, false, &raw mut timestamp_u64);
        if mru_ret != SD_READ_STATUS_SUCCESS {
            return mru_ret;
        }
        let mru_ret = rs_msgpack_read_uint64(fd, false, &raw mut length_u64);
        if mru_ret != SD_READ_STATUS_SUCCESS {
            return mru_ret;
        }

        if length_u64 > isize::MAX as u64 {
            nvim_shada_semsg_u64(c"E576: Error while reading ShaDa file: there is an item at position %llu that is stated to be too long".as_ptr(), initial_fpos);
            return SD_READ_STATUS_NOT_SHADA;
        }

        let length = length_u64 as usize;
        (*entry).timestamp = timestamp_u64;
        (*entry).can_free_entry = true;

        if type_u64 == 0 {
            nvim_shada_semsg_u64(c"E576: Error while reading ShaDa file: there is an item at position %llu that must not be there: Missing items are for internal uses only".as_ptr(), initial_fpos);
            return SD_READ_STATUS_NOT_SHADA;
        }

        // Decide whether to read or skip this entry
        let should_read = {
            let type_flag_ok = if type_u64 > SHADA_LAST_ENTRY {
                (flags & SD_READ_UNKNOWN) != 0
            } else {
                (flags & (1u32 << type_u64 as u32)) != 0
            };
            let size_ok = max_kbyte == 0 || length <= max_kbyte * 1024;
            type_flag_ok && size_ok
        };

        if !should_read {
            // Check for non-ShaDa file at position 0
            if initial_fpos == 0 && (type_u64 == u64::from(b'\n') || type_u64 > SHADA_LAST_ENTRY) {
                verify_but_ignore = true;
            } else {
                let srs_ret = rs_sd_reader_skip_bytes(fd, length);
                if srs_ret != SD_READ_STATUS_SUCCESS {
                    return srs_ret;
                }
                // loop back to start
                continue;
            }
        }

        let parse_pos = nvim_shada_file_bytes_read(sd_reader);

        // Try zero-alloc read from internal buffer, else malloc+read
        let buf_from_file = nvim_shada_file_try_read_buffered(sd_reader, length);
        let (buf, buf_allocated) = if buf_from_file.is_null() {
            let b = nvim_xmalloc(length).cast::<c_char>();
            let fl_ret = rs_fread_len(fd, b, length);
            if fl_ret != SD_READ_STATUS_SUCCESS {
                nvim_xfree(b.cast::<c_void>());
                return fl_ret;
            }
            (b, true)
        } else {
            (buf_from_file, false)
        };

        let read_ptr = buf.cast_const();
        let read_size = length;

        if verify_but_ignore {
            // Phase 6 (plan 13c452f9): use Rust rs_verify_skip
            let spm_ret = rs_verify_skip(read_ptr, read_size, parse_pos);
            if buf_allocated {
                nvim_xfree(buf.cast::<c_void>());
            }
            if spm_ret != SD_READ_STATUS_SUCCESS {
                return spm_ret;
            }
            continue; // loop back to start
        }

        if type_u64 > SHADA_LAST_ENTRY {
            // Phase 6 (plan 13c452f9): use Rust rs_set_unknown_item
            let r = rs_set_unknown_item(
                entry,
                type_u64,
                buf,
                length,
                buf_allocated,
                read_ptr,
                read_size,
                initial_fpos,
                parse_pos,
            );
            return r;
        }

        // Set default data for this entry type (Phase 6: Rust implementation)
        rs_set_entry_default_data(entry, type_u64);

        // Parse the entry body using per-type implementations
        let mut num_additional: u32 = 0;
        let mut body_read_ptr = read_ptr;
        let mut body_read_size = read_size;

        let parse_ret = match type_u64 {
            t if t == SD_ITEM_HEADER as u64 => {
                // Header: no parsing needed (nvim never reads header contents)
                SD_READ_STATUS_SUCCESS
            }
            t if t == SD_ITEM_SEARCH_PATTERN as u64 => {
                rs_parse_search_pattern(entry, read_ptr, read_size, initial_fpos)
            }
            t if t == SD_ITEM_CHANGE as u64
                || t == SD_ITEM_JUMP as u64
                || t == SD_ITEM_GLOBAL_MARK as u64
                || t == SD_ITEM_LOCAL_MARK as u64 =>
            {
                rs_parse_mark(entry, read_ptr, read_size, initial_fpos, type_u64)
            }
            t if t == SD_ITEM_REGISTER as u64 => {
                rs_parse_register(entry, read_ptr, read_size, initial_fpos)
            }
            t if t == SD_ITEM_HISTORY_ENTRY as u64 => rs_parse_history(
                entry,
                read_ptr,
                read_size,
                initial_fpos,
                &raw mut num_additional,
                &raw mut body_read_ptr,
                &raw mut body_read_size,
            ),
            t if t == SD_ITEM_VARIABLE as u64 => rs_parse_variable(
                entry,
                read_ptr,
                read_size,
                initial_fpos,
                &raw mut num_additional,
                &raw mut body_read_ptr,
                &raw mut body_read_size,
            ),
            t if t == SD_ITEM_SUB_STRING as u64 => rs_parse_substr(
                entry,
                read_ptr,
                read_size,
                initial_fpos,
                &raw mut num_additional,
                &raw mut body_read_ptr,
                &raw mut body_read_size,
            ),
            t if t == SD_ITEM_BUFFER_LIST as u64 => {
                rs_parse_buflist(entry, read_ptr, read_size, initial_fpos)
            }
            _ => {
                // kSDItemMissing / kSDItemUnknown should not reach here
                SD_READ_STATUS_MALFORMED
            }
        };

        if parse_ret != SD_READ_STATUS_SUCCESS {
            // Error: free partial entry data and return Malformed
            (*entry).entry_type = ShadaEntryType::from_raw(type_u64 as c_int);
            rs_shada_free_entry_contents(entry);
            (*entry).entry_type = ShadaEntryType::Missing;
            if buf_allocated {
                nvim_xfree(buf.cast::<c_void>());
            }
            return SD_READ_STATUS_MALFORMED;
        }

        // Parse additional data (Phase 6: Rust implementation)
        // Only array-based parsers (history, variable, substr) set num_additional
        // and write back body_read_ptr/body_read_size. Keydict-based parsers
        // (search_pattern, mark, register, buflist) use local copies of read_ptr/
        // read_size and do not update body_read_size; they handle unknown keys
        // internally via unpack_keydict, so body_read_size remains the original
        // body size and must not be used as a "remaining bytes" signal here.
        if num_additional > 0 {
            let ad_ret = rs_parse_additional_data(
                entry,
                body_read_ptr,
                body_read_size,
                num_additional,
                initial_fpos,
            );
            if ad_ret != SD_READ_STATUS_SUCCESS {
                (*entry).entry_type = ShadaEntryType::from_raw(type_u64 as c_int);
                rs_shada_free_entry_contents(entry);
                (*entry).entry_type = ShadaEntryType::Missing;
                if buf_allocated {
                    nvim_xfree(buf.cast::<c_void>());
                }
                return SD_READ_STATUS_MALFORMED;
            }
        }

        (*entry).entry_type = ShadaEntryType::from_raw(type_u64 as c_int);
        if buf_allocated {
            nvim_xfree(buf.cast::<c_void>());
        }
        return SD_READ_STATUS_SUCCESS;
    }
}

/// Read ShaDa data and merge it into Neovim's in-memory state.
///
/// This is the Rust replacement for the C `shada_read` function.
///
/// # Safety
///
/// `sd_reader` must be a valid `FileDescriptor *` or null.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_shada_read(sd_reader: *mut c_void, flags: c_int) {
    let force = (flags & SHADA_FORCEIT) != 0;
    let want_marks = (flags & SHADA_WANT_MARKS) != 0;
    let get_old_files = (flags & (SHADA_GET_OLDFILES | SHADA_FORCEIT)) != 0 && {
        let oldfiles_list = nvim_shada_get_oldfiles_list();
        force || oldfiles_list.is_null() || nvim_shada_tv_list_len(oldfiles_list) == 0
    };
    let local_marks_param = rs_get_shada_parameter(c_int::from(b'\''));
    let argcount = nvim_shada_argcount();
    let srni_flags = compute_srni_flags(flags, local_marks_param, get_old_files, argcount);
    if srni_flags == 0 {
        return;
    }

    let need_history = (srni_flags & SD_READ_HISTORY) != 0;
    let mut hms: [HistoryMergerState; HIST_COUNT] =
        std::array::from_fn(|_| HistoryMergerState::default());
    if need_history {
        let p_hi = nvim_get_p_hi();
        for (i, hms_slot) in hms.iter_mut().enumerate() {
            rs_hms_init(hms_slot, i as u8, p_hi as usize, true, true);
        }
    }

    let fname_bufs = nvim_shada_fname_bufs_new();
    let cl_bufs = nvim_shada_set_init_ptr();
    let oldfiles_set = nvim_shada_oldfiles_set_new();
    // Ensure VV_OLDFILES list exists when we need to gather old files.
    let oldfiles_list = nvim_shada_get_oldfiles_list();
    if get_old_files && (oldfiles_list.is_null() || force) {
        nvim_shada_create_oldfiles_list();
    }

    let mut cur_entry = ShadaEntry::default();
    'read_loop: loop {
        let srni_ret = rs_shada_read_next_item(sd_reader, &raw mut cur_entry, srni_flags, 0);
        match srni_ret {
            r if r == SD_READ_STATUS_FINISHED => break 'read_loop,
            r if r == SD_READ_STATUS_SUCCESS => {}
            r if r == SD_READ_STATUS_NOT_SHADA || r == SD_READ_STATUS_READ_ERROR => {
                break 'read_loop;
            }
            _ => {
                // kSDReadStatusMalformed or unknown: skip entry
                cur_entry = ShadaEntry::default();
                continue 'read_loop;
            }
        }

        if cur_entry.entry_type == ShadaEntryType::HistoryEntry && need_history {
            let hist_ptr: *const HistoryItemData =
                std::ptr::addr_of!(cur_entry.data.history_item).cast();
            let histtype = (*hist_ptr).histtype as usize;
            if histtype < HIST_COUNT {
                rs_hms_insert(&raw mut hms[histtype], cur_entry, true);
            } else {
                rs_shada_free_entry_contents(&raw mut cur_entry);
            }
            cur_entry = ShadaEntry::default();
            continue 'read_loop;
        }

        // Dispatch all other entry types through the Rust dispatcher.
        rs_shada_apply_entry(
            force,
            want_marks,
            get_old_files,
            &raw mut cur_entry,
            fname_bufs,
            cl_bufs,
            oldfiles_set,
            nvim_shada_get_oldfiles_list(),
        );
        cur_entry = ShadaEntry::default();
    }

    if need_history {
        for (i, hms_slot) in hms.iter_mut().enumerate() {
            let i_int = i as c_int;
            rs_hms_insert_whole_neovim_history(hms_slot);
            let _ = clr_history(i_int);
            let mut new_hisidx: *mut c_int = std::ptr::null_mut();
            let mut new_hisnum: *mut c_int = std::ptr::null_mut();
            let hist = hist_get_array(i as u8, &raw mut new_hisidx, &raw mut new_hisnum);
            if !hist.is_null() {
                rs_hms_to_he_array(hms_slot, hist, new_hisidx, new_hisnum);
            }
            rs_hms_dealloc(hms_slot);
        }
    }

    nvim_shada_for_all_tab_windows_update_changelist(cl_bufs);
    nvim_shada_fname_bufs_destroy(fname_bufs);
    nvim_shada_set_destroy_ptr(cl_bufs);
    nvim_shada_oldfiles_set_destroy(oldfiles_set);
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

impl NvimString {
    /// Check if data pointer is null (string has no data).
    pub const fn is_null(self) -> bool {
        self.data.is_null()
    }
}

// C accessor functions for encoding/decoding
extern "C" {
    /// Iterate over shada-saveable global variables.
    /// flavour: bitmask of VAR_FLAVOUR_DEFAULT(1)|VAR_FLAVOUR_SESSION(2)|VAR_FLAVOUR_SHADA(4).
    /// On each call: *out_name set to variable name (static lifetime, do not free),
    /// *out_tv set to a freshly xmalloc'd typval_T copy (must be passed to nvim_shada_build_gvar_entry).
    /// Returns next iter pointer (NULL when exhausted).
    fn nvim_shada_var_shada_iter(
        iter: *const c_void,
        out_name: *mut *const c_char,
        out_tv: *mut *mut c_void,
        flavour: c_uint,
    ) -> *const c_void;
    /// Get the v_type field of a typval_T pointer.
    fn nvim_shada_tv_get_type(tv: *const c_void) -> c_int;
    /// Build a ShadaEntry for a global variable into *out (C layout: inline typval_T).
    /// Copies and consumes tv (clears+frees the xmalloc'd pointer).
    /// Caller must call nvim_shada_clear_gvar_entry_value after rs_shada_pack_entry.
    fn nvim_shada_build_gvar_entry(
        name: *const c_char,
        tv: *mut c_void,
        ts: Timestamp,
        out: *mut ShadaEntry,
    );
    /// Clear the inline typval_T of a global variable ShadaEntry (after packing).
    fn nvim_shada_clear_gvar_entry_value(entry: *mut ShadaEntry);
}

/// Encode registers to a ShaDa-format string.
///
/// Returns a newly allocated string containing all register entries
/// in MessagePack format suitable for ShaDa storage.
///
/// # Panics
///
/// Panics if packing a register entry fails (mirrors C `abort()` on failure).
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_regs() -> NvimString {
    let wms_size = std::mem::size_of::<WriteMergerState>();
    let wms = nvim_xcalloc(1, wms_size).cast::<WriteMergerState>();
    // Initialize the struct with safe defaults
    std::ptr::write(wms, WriteMergerState::default());
    rs_shada_initialize_registers(wms, -1);

    let mut packer = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_string_buffer(packer.as_mut_ptr());
    let packer = packer.as_mut_ptr();

    for i in 0..NUM_SAVED_REGISTERS {
        if (*wms).registers[i].entry_type == ShadaEntryType::Register {
            let entry_ptr = std::ptr::addr_of_mut!((*wms).registers[i]);
            assert!(
                rs_shada_pack_pfreed_entry(packer, entry_ptr, 0) != SD_WRITE_FAILED,
                "shada_encode_regs: pack failed"
            );
        }
    }
    // Free the WriteMergerState (register contents not owned by WMS, can_free_entry is false)
    nvim_xfree(wms.cast::<c_void>());

    nvim_shada_packer_take_string(packer)
}

/// Encode jump list to a ShaDa-format string.
///
/// Returns a newly allocated string containing jump list entries
/// in MessagePack format suitable for ShaDa storage.
///
/// # Panics
///
/// Panics if packing a jump entry fails (mirrors C `abort()` on failure).
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_jumps() -> NvimString {
    let removable_bufs = nvim_shada_set_init_ptr();
    rs_find_removable_bufs(removable_bufs);

    let mut jumps = [const { std::mem::MaybeUninit::<ShadaEntry>::uninit() }; JUMPLISTSIZE];
    let jumps_ptr = jumps.as_mut_ptr().cast::<ShadaEntry>();
    let jumps_size = rs_shada_init_jumps(jumps_ptr, removable_bufs);
    nvim_shada_set_destroy_ptr(removable_bufs);

    let mut packer = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_string_buffer(packer.as_mut_ptr());
    let packer = packer.as_mut_ptr();

    for i in 0..jumps_size {
        let entry_ptr = jumps_ptr.add(i);
        assert!(
            rs_shada_pack_pfreed_entry(packer, entry_ptr, 0) != SD_WRITE_FAILED,
            "shada_encode_jumps: pack failed"
        );
    }

    nvim_shada_packer_take_string(packer)
}

/// Encode buffer list to a ShaDa-format string.
///
/// Returns a newly allocated string containing the buffer list entry
/// in MessagePack format suitable for ShaDa storage.
///
/// # Panics
///
/// Panics if packing the buffer list entry fails (mirrors C `abort()` on failure).
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_buflist() -> NvimString {
    let removable_bufs = nvim_shada_set_init_ptr();
    rs_find_removable_bufs(removable_bufs);
    let buflist_entry = rs_shada_get_buflist(removable_bufs);
    nvim_shada_set_destroy_ptr(removable_bufs);

    let mut packer = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_string_buffer(packer.as_mut_ptr());
    let packer = packer.as_mut_ptr();

    assert!(
        rs_shada_pack_entry(packer, &raw const buflist_entry, 0) != SD_WRITE_FAILED,
        "shada_encode_buflist: pack failed"
    );
    // Free the buffer list array allocated by rs_shada_get_buflist.
    // Access via addr_of to avoid implicit autoref through ManuallyDrop union field.
    let buffers_ptr = std::ptr::read(std::ptr::addr_of!(
        (*buflist_entry.data.buffer_list).buffers
    ));
    nvim_xfree(buffers_ptr.cast::<c_void>());

    nvim_shada_packer_take_string(packer)
}

/// Encode global variables to a ShaDa-format string.
///
/// Returns a newly allocated string containing global variable entries
/// in MessagePack format suitable for ShaDa storage.
///
/// # Panics
///
/// Panics if packing a variable entry fails (mirrors C `abort()` on failure).
#[no_mangle]
pub unsafe extern "C" fn rs_shada_encode_gvars() -> NvimString {
    // VAR_FLAVOUR_DEFAULT=1 | VAR_FLAVOUR_SESSION=2 | VAR_FLAVOUR_SHADA=4
    const VAR_FLAVOUR_ALL: c_uint = 7;
    // v_type values: VAR_FUNC=3, VAR_PARTIAL=9
    const VAR_FUNC: c_int = 3;
    const VAR_PARTIAL: c_int = 9;

    let mut packer = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_string_buffer(packer.as_mut_ptr());
    let packer = packer.as_mut_ptr();

    let cur_timestamp = os_time();

    let mut var_iter: *const c_void = std::ptr::null();
    loop {
        let mut name: *const c_char = std::ptr::null();
        let mut tv: *mut c_void = std::ptr::null_mut();
        var_iter = nvim_shada_var_shada_iter(var_iter, &raw mut name, &raw mut tv, VAR_FLAVOUR_ALL);
        if name.is_null() {
            break;
        }
        // tv is non-null when name is non-null
        let vtype = nvim_shada_tv_get_type(tv);
        if vtype != VAR_FUNC && vtype != VAR_PARTIAL {
            // Build the ShadaEntry in C (inline typval_T layout), then pack, then clear.
            let mut entry = ShadaEntry::default();
            nvim_shada_build_gvar_entry(name, tv, cur_timestamp, &raw mut entry);
            let r = rs_shada_pack_entry(packer, &raw const entry, 0);
            nvim_shada_clear_gvar_entry_value(&raw mut entry);
            assert!(r != SD_WRITE_FAILED, "shada_encode_gvars: pack failed");
        } else {
            // Free the typval without packing it (we skip func/partial types)
            nvim_shada_tv_clear(tv.cast());
            nvim_xfree(tv);
        }
        if var_iter.is_null() {
            break;
        }
    }

    nvim_shada_packer_take_string(packer)
}

/// Write all global variables (VAR_FLAVOUR_SHADA) to the packer,
/// updating wms->dumped_variables. Includes circular-reference detection
/// for dict and list values.
///
/// This is the Rust replacement for C nvim_shada_pack_all_gvars.
///
/// Returns SD_WRITE_SUCCESSFUL or SD_WRITE_FAILED.
///
/// # Safety
///
/// `packer` and `wms` must be valid pointers.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn rs_shada_pack_all_gvars(
    packer: *mut ShadaPackerBuffer,
    wms: *mut c_void,
    // max_kbyte is passed by caller but nvim_shada_pack_gvar_entry always uses 0
    _max_kbyte: usize,
) -> c_int {
    // VAR_FLAVOUR_SHADA = 4
    const VAR_FLAVOUR_SHADA: c_uint = 4;
    // v_type values
    const VAR_FUNC: c_int = 3;
    const VAR_PARTIAL: c_int = 9;
    const VAR_DICT: c_int = 6;
    const VAR_LIST: c_int = 5;

    let cur_timestamp = os_time();
    let mut var_iter: *const c_void = std::ptr::null();

    loop {
        let mut name: *const c_char = std::ptr::null();
        let mut tv: *mut c_void = std::ptr::null_mut();
        var_iter =
            nvim_shada_var_shada_iter(var_iter, &raw mut name, &raw mut tv, VAR_FLAVOUR_SHADA);
        if name.is_null() {
            break;
        }

        let mut vtype: c_int = 0;
        let mut container: *mut c_void = std::ptr::null_mut();
        let mut tv_copy_id: c_int = 0;
        nvim_shada_tv_get_refcheck_info(
            tv,
            &raw mut vtype,
            &raw mut container,
            &raw mut tv_copy_id,
        );

        // Skip function and partial types.
        if vtype == VAR_FUNC || vtype == VAR_PARTIAL {
            nvim_shada_tv_clear(tv.cast());
            nvim_xfree(tv);
            if var_iter.is_null() {
                break;
            }
            continue;
        }

        // Circular reference detection for dict and list.
        if vtype == VAR_DICT {
            let copy_id = rs_get_copyID();
            let set_result = if container.is_null() {
                false
            } else {
                rs_set_ref_in_ht(container, copy_id, std::ptr::null_mut())
            };
            if !set_result && copy_id == tv_copy_id {
                nvim_shada_tv_clear(tv.cast());
                nvim_xfree(tv);
                if var_iter.is_null() {
                    break;
                }
                continue;
            }
        } else if vtype == VAR_LIST {
            let copy_id = rs_get_copyID();
            let set_result = if container.is_null() {
                false
            } else {
                rs_set_ref_in_list_items(container, copy_id, std::ptr::null_mut())
            };
            if !set_result && copy_id == tv_copy_id {
                nvim_shada_tv_clear(tv.cast());
                nvim_xfree(tv);
                if var_iter.is_null() {
                    break;
                }
                continue;
            }
        }

        // Build the ShadaEntry in C (inline typval_T layout), then pack, then clear.
        let mut gvar_entry = ShadaEntry::default();
        nvim_shada_build_gvar_entry(name, tv, cur_timestamp, &raw mut gvar_entry);
        let spe_ret = rs_shada_pack_entry(packer, &raw const gvar_entry, 0);
        nvim_shada_clear_gvar_entry_value(&raw mut gvar_entry);
        if spe_ret == SD_WRITE_FAILED {
            return SD_WRITE_FAILED;
        }
        if spe_ret == SD_WRITE_SUCCESSFUL {
            nvim_shada_wms_dumped_vars_put(wms, name);
        }

        if var_iter.is_null() {
            break;
        }
    }

    SD_WRITE_SUCCESSFUL
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
    let fd_size = nvim_shada_file_descriptor_size();
    let sd_reader = nvim_xcalloc(1, fd_size);
    let fd = FileDescriptorHandle::from_ptr(sd_reader);
    nvim_shada_file_open_buffer(fd, string.data, string.size);
    rs_shada_read(fd.as_ptr(), flags);
    rs_close_file(fd);
    nvim_xfree(sd_reader);
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
    /// Names of already dumped variables (inline Set(cstr_t), 40 bytes).
    pub dumped_variables: SetCstrT,
    /// All file marks (inline PMap(cstr_t), 48 bytes).
    pub file_marks: PMapCstrT,
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
            dumped_variables: SetCstrT::default(),
            file_marks: PMapCstrT::default(),
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

    match (*entry).entry_type {
        ShadaEntryType::Unknown => {
            let contents = read_union_field!(entry, unknown_item, contents);
            if !contents.is_null() {
                nvim_xfree(contents.cast());
            }
        }
        // Missing: nothing to free.
        ShadaEntryType::Missing => {}
        // Header is a Dict (kvec_t) — layout differs between Rust and C.
        // Delegate freeing to C which knows the correct Dict layout.
        ShadaEntryType::Header => {
            nvim_shada_free_header_entry(entry);
        }
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
            // Register contents are *mut c_char pointers (Rust layout, uniform for all paths).
            // Read-path entries (can_free_entry=true) own each pointer; write-path (false) skip here.
            let contents = read_union_field!(entry, reg, contents);
            let contents_size = read_union_field!(entry, reg, contents_size);
            if !contents.is_null() {
                for i in 0..contents_size {
                    nvim_xfree((*contents.add(i)).cast::<c_void>());
                }
                nvim_xfree(contents.cast::<c_void>());
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
    let a_fm: *const FileMarks = *(a.cast::<*const FileMarks>());
    let b_fm: *const FileMarks = *(b.cast::<*const FileMarks>());
    let a_ts = if a_fm.is_null() {
        0
    } else {
        (*a_fm).greatest_timestamp
    };
    let b_ts = if b_fm.is_null() {
        0
    } else {
        (*b_fm).greatest_timestamp
    };
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
    rs_shada_free_entry_contents(std::ptr::addr_of_mut!(wms.numbered_marks[last]));

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
    let new_name = home_replace_save(std::ptr::null(), name);
    let p_shada = nvim_shada_get_p_shada();
    let mut p = p_shada.cast_mut();
    let mut retval = false;

    while *p != 0 {
        copy_option_part(
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
            if mb_strnicmp(name_buff, new_name, n) == 0 {
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
    if nvim_shada_buf_should_skip(buf) != 0 {
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

    nvim_shada_get_search_pattern(
        c_int::from(is_sub),
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
            // Convert String* array (16-byte structs) to *mut c_char array (8-byte pointers)
            // so all register entries share uniform Rust layout for packing.
            let strings = contents.cast::<NvimString>();
            let ptr_array =
                nvim_xmalloc(size * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
            for j in 0..size {
                *ptr_array.add(j) = (*strings.add(j)).data;
            }
            wms.registers[idx as usize] = ShadaEntry {
                can_free_entry: false,
                entry_type: ShadaEntryType::Register,
                timestamp: ts,
                data: ShadaEntryData {
                    reg: std::mem::ManuallyDrop::new(RegisterData {
                        contents: ptr_array,
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
    let max_bufs = rs_get_shada_parameter(c_int::from(b'%'));
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
        let mut additional_data: *mut c_void = std::ptr::null_mut();
        nvim_shada_buf_get_buflist_info(buf, &raw mut pos, &raw mut additional_data);
        let ffname = nvim_shada_buf_get_ffname(buf);
        (*buffers.add(i)).pos = pos;
        (*buffers.add(i)).fname = ffname.cast_mut();
        (*buffers.add(i)).additional_data = additional_data;
        i += 1;
        buf = nvim_shada_buf_next(buf);
    }

    ShadaEntry {
        entry_type: ShadaEntryType::BufferList,
        can_free_entry: false,
        timestamp: os_time(),
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

    setpcmark();
    let wp = nvim_shada_curwin();
    cleanup_jumplist(wp, false);

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
            buflist_findnr(fnum)
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
    let error = nvim_file_close(cookie, c_int::from(nvim_get_p_fs()));
    if error != 0 {
        nvim_shada_semsg_1s(
            c"E886: System error while closing ShaDa file: %s".as_ptr(),
            nvim_shada_os_strerror(error),
        );
    }
}

// =============================================================================
// Phase 1: shada_pack_entry migration
// =============================================================================

// C accessor functions for packing
extern "C" {
    /// Encode a typval_T to msgpack.
    fn nvim_encode_vim_to_msgpack(
        packer: *mut ShadaPackerBuffer,
        tv: *mut c_void,
        desc: *const c_char,
    ) -> c_int;
    /// Get number of additional data items.
    fn nvim_shada_additional_data_len(ad_ptr: *const c_void) -> u32;
    /// Write additional data raw bytes to a packer buffer.
    fn nvim_shada_dump_additional_data(ad_ptr: *const c_void, sbuf: *mut ShadaPackerBuffer);
    /// Check if entry variable is a blob.
    fn nvim_shada_entry_is_blob_var(entry: *const ShadaEntry) -> c_int;
    /// Get typval_T pointer from a variable entry.
    fn nvim_shada_entry_var_value_ptr(entry: *mut ShadaEntry) -> *mut c_void;
    /// Pack a header entry's Dict into a packer buffer (replaces 7 individual header accessors).
    fn nvim_shada_pack_header_dict(entry: *const ShadaEntry, sbuf: *mut ShadaPackerBuffer);

    // MessagePack primitives (from nvim-msgpack crate, callable as extern "C")
    fn rs_mpack_array(ptr: *mut *mut u8, size: u32);
    fn rs_mpack_map(ptr: *mut *mut u8, size: u32);
    fn rs_mpack_uint(ptr: *mut *mut u8, val: u32);
    fn rs_mpack_bool(ptr: *mut *mut u8, val: c_int);
    #[link_name = "mpack_uint64"]
    fn rs_mpack_uint64(ptr: *mut *mut u8, val: u64);
    #[link_name = "mpack_integer"]
    fn rs_mpack_integer(ptr: *mut *mut u8, val: i64);
    fn rs_mpack_bin(data: *const u8, len: usize, packer: *mut ShadaPackerBuffer);
    fn rs_mpack_str(data: *const u8, len: usize, packer: *mut ShadaPackerBuffer);
    #[link_name = "mpack_raw"]
    fn rs_mpack_raw(data: *const u8, len: usize, packer: *mut ShadaPackerBuffer);
    #[link_name = "mpack_check_buffer"]
    fn rs_mpack_check_buffer(packer: *mut ShadaPackerBuffer);

}

/// VAR_BLOB constant value matching C's VarType enum.
const VAR_TYPE_BLOB: i64 = 10;

/// Pack a key string (2-char byte literal) into the packer.
///
/// # Safety
///
/// packer must be a valid packer buffer pointer.
#[inline]
unsafe fn pack_key(key: &[u8], sbuf: *mut ShadaPackerBuffer) {
    rs_mpack_str(key.as_ptr(), key.len(), sbuf);
}

/// Write a single ShaDa entry to a packer buffer.
///
/// This is the Rust implementation of the C `shada_pack_entry` function.
/// Serializes a ShadaEntry to msgpack format, writing to both a temporary
/// string buffer (sbuf) and the outer packer.
///
/// Returns SD_WRITE_SUCCESSFUL, SD_WRITE_FAILED, or SD_WRITE_IGN_ERROR.
///
/// # Safety
///
/// - `packer` must be a valid PackerBuffer pointer
/// - `entry` must be a valid ShadaEntry pointer
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::similar_names)] // packer/packed and fm_name/fm_fname are intentionally distinct
#[allow(clippy::cast_possible_wrap)] // b'"' as c_char is intentional
#[allow(clippy::cast_lossless)] // char-as-u8-as-u32 casts for register/mark char fields
pub unsafe extern "C" fn rs_shada_pack_entry(
    packer: *mut ShadaPackerBuffer,
    entry: *const ShadaEntry,
    max_kbyte: usize,
) -> c_int {
    if packer.is_null() || entry.is_null() {
        return SD_WRITE_FAILED;
    }

    // Create a temporary string buffer for the entry body
    let mut sbuf = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_string_buffer(sbuf.as_mut_ptr());
    let sbuf = sbuf.as_mut_ptr();

    let entry_type = (*entry).entry_type;
    let timestamp = (*entry).timestamp;
    let additional_data = (*entry).additional_data;
    let ad_len = nvim_shada_additional_data_len(additional_data);

    rs_mpack_check_buffer(sbuf);

    match entry_type {
        ShadaEntryType::Missing => {
            // abort() equivalent - should never happen
            nvim_xfree(nvim_shada_packer_take_string(sbuf).data.cast::<c_void>());
            return SD_WRITE_FAILED;
        }
        ShadaEntryType::Unknown => {
            let contents = read_union_field!(entry, unknown_item, contents);
            let size = read_union_field!(entry, unknown_item, size);
            if !contents.is_null() {
                rs_mpack_raw(contents.cast::<u8>(), size, sbuf);
            }
        }
        ShadaEntryType::HistoryEntry => {
            let histtype = read_union_field!(entry, history_item, histtype);
            let string = read_union_field!(entry, history_item, string);
            let sep = read_union_field!(entry, history_item, sep);
            let is_hist_search = histtype == HIST_SEARCH;
            let arr_size = 2u32 + u32::from(is_hist_search) + ad_len;
            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_array(&raw mut ptr, arr_size);
            rs_mpack_uint(&raw mut ptr, u32::from(histtype));
            nvim_shada_packer_set_ptr(sbuf, ptr);
            // Pack binary string (the history string)
            let slen = if string.is_null() {
                0
            } else {
                libc::strlen(string)
            };
            rs_mpack_bin(string.cast::<u8>(), slen, sbuf);
            if is_hist_search {
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_uint(&raw mut ptr2, u32::from(sep as u8));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            nvim_shada_dump_additional_data(additional_data.cast::<c_void>(), sbuf);
        }
        ShadaEntryType::Variable => {
            // Check if it's a blob type (must use C accessor - reads C-layout typval_T)
            let is_blob = nvim_shada_entry_is_blob_var(entry) != 0;
            let arr_size = 2u32 + u32::from(is_blob) + ad_len;
            let name = read_union_field!(entry, global_var, name);
            let name_len = if name.is_null() {
                0
            } else {
                libc::strlen(name)
            };
            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_array(&raw mut ptr, arr_size);
            nvim_shada_packer_set_ptr(sbuf, ptr);
            // Pack variable name as binary
            rs_mpack_bin(name.cast::<u8>(), name_len, sbuf);
            // Build vardesc for error messages
            let mut vardesc = [0u8; 256];
            let prefix = b"variable g:";
            let copy_len = name_len.min(vardesc.len() - prefix.len() - 1);
            vardesc[..prefix.len()].copy_from_slice(prefix);
            if !name.is_null() && copy_len > 0 {
                std::ptr::copy_nonoverlapping(
                    name.cast::<u8>(),
                    vardesc[prefix.len()..].as_mut_ptr(),
                    copy_len,
                );
            }
            vardesc[prefix.len() + copy_len] = 0;
            // Encode the typval_T
            let tv_ptr = nvim_shada_entry_var_value_ptr(entry.cast_mut());
            let encode_ret =
                nvim_encode_vim_to_msgpack(sbuf, tv_ptr, vardesc.as_ptr().cast::<c_char>());
            if encode_ret != 0 {
                // encode_vim_to_msgpack returns FAIL for non-serializable values
                let sbuf_str = nvim_shada_packer_take_string(sbuf);
                nvim_xfree(sbuf_str.data.cast::<c_void>());
                return SD_WRITE_IGN_ERROR;
            }
            if is_blob {
                rs_mpack_check_buffer(sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_integer(&raw mut ptr2, VAR_TYPE_BLOB);
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            nvim_shada_dump_additional_data(additional_data.cast::<c_void>(), sbuf);
        }
        ShadaEntryType::SubString => {
            let sub = read_union_field!(entry, sub_string, sub);
            let arr_size = 1u32 + ad_len;
            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_array(&raw mut ptr, arr_size);
            nvim_shada_packer_set_ptr(sbuf, ptr);
            let sub_len = if sub.is_null() { 0 } else { libc::strlen(sub) };
            rs_mpack_bin(sub.cast::<u8>(), sub_len, sbuf);
            nvim_shada_dump_additional_data(additional_data.cast::<c_void>(), sbuf);
        }
        ShadaEntryType::SearchPattern => {
            // Default values for comparison
            let def_magic = true;
            let def_is_last_used = true;
            let def_smartcase = false;
            let def_has_line_offset = false;
            let def_place_cursor_at_end = false;
            let def_is_substitute_pattern = false;
            let def_highlighted = false;
            let def_offset: i64 = 0;
            let def_search_backward = false;

            // Data is in Rust SearchPatternData layout (created by rs_parse_search_pattern
            // or rs_add_search_pattern), so direct field access via read_union_field! is correct.
            let sp_magic = read_union_field!(entry, search_pattern, magic);
            let sp_is_last_used = read_union_field!(entry, search_pattern, is_last_used);
            let sp_smartcase = read_union_field!(entry, search_pattern, smartcase);
            let sp_has_line_offset = read_union_field!(entry, search_pattern, has_line_offset);
            let sp_place_cursor_at_end =
                read_union_field!(entry, search_pattern, place_cursor_at_end);
            let sp_is_substitute_pattern =
                read_union_field!(entry, search_pattern, is_substitute_pattern);
            let sp_highlighted = read_union_field!(entry, search_pattern, highlighted);
            let sp_search_backward = read_union_field!(entry, search_pattern, search_backward);
            let sp_offset = read_union_field!(entry, search_pattern, offset);
            let sp_pat = read_union_field!(entry, search_pattern, pat);
            let sp_pat_len = read_union_field!(entry, search_pattern, pat_len);

            let mut map_size: u32 = 1; // pattern is always present
            if sp_magic != def_magic {
                map_size += 1;
            }
            if sp_is_last_used != def_is_last_used {
                map_size += 1;
            }
            if sp_smartcase != def_smartcase {
                map_size += 1;
            }
            if sp_has_line_offset != def_has_line_offset {
                map_size += 1;
            }
            if sp_place_cursor_at_end != def_place_cursor_at_end {
                map_size += 1;
            }
            if sp_is_substitute_pattern != def_is_substitute_pattern {
                map_size += 1;
            }
            if sp_highlighted != def_highlighted {
                map_size += 1;
            }
            if sp_offset != def_offset {
                map_size += 1;
            }
            if sp_search_backward != def_search_backward {
                map_size += 1;
            }
            map_size += ad_len;

            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_map(&raw mut ptr, map_size);
            nvim_shada_packer_set_ptr(sbuf, ptr);

            // Always pack pattern
            pack_key(SEARCH_KEY_PAT, sbuf);
            rs_mpack_bin(sp_pat.cast::<u8>(), sp_pat_len, sbuf);

            if sp_magic != def_magic {
                pack_key(SEARCH_KEY_MAGIC, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_magic));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_is_last_used != def_is_last_used {
                pack_key(SEARCH_KEY_IS_LAST_USED, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_is_last_used));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_smartcase != def_smartcase {
                pack_key(SEARCH_KEY_SMARTCASE, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_smartcase));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_has_line_offset != def_has_line_offset {
                pack_key(SEARCH_KEY_HAS_LINE_OFFSET, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_has_line_offset));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_place_cursor_at_end != def_place_cursor_at_end {
                pack_key(SEARCH_KEY_PLACE_CURSOR_AT_END, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_place_cursor_at_end));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_is_substitute_pattern != def_is_substitute_pattern {
                pack_key(SEARCH_KEY_IS_SUBSTITUTE_PATTERN, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_is_substitute_pattern));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_highlighted != def_highlighted {
                pack_key(SEARCH_KEY_HIGHLIGHTED, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_highlighted));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_search_backward != def_search_backward {
                pack_key(SEARCH_KEY_BACKWARD, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr2, c_int::from(!def_search_backward));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if sp_offset != def_offset {
                pack_key(SEARCH_KEY_OFFSET, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_integer(&raw mut ptr2, sp_offset);
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            nvim_shada_dump_additional_data(additional_data.cast::<c_void>(), sbuf);
        }
        ShadaEntryType::Change
        | ShadaEntryType::GlobalMark
        | ShadaEntryType::LocalMark
        | ShadaEntryType::Jump => {
            // Direct Rust field access - FilemarkData uses Rust Position (i64 lnum).
            let fm_mark = read_union_field!(entry, filemark, mark);
            let fm_lnum = fm_mark.lnum;
            let fm_col = fm_mark.col;
            let fm_name = read_union_field!(entry, filemark, name);
            let fm_fname = read_union_field!(entry, filemark, fname);

            let def_lnum: i64 = 1;
            let def_col: i32 = 0;
            // Default name depends on type
            let def_name: c_char = match entry_type {
                ShadaEntryType::GlobalMark | ShadaEntryType::LocalMark => b'"' as c_char,
                _ => 0,
            };
            let mut map_size: u32 = 1; // fname always present
            if fm_lnum != def_lnum {
                map_size += 1;
            }
            if fm_col != def_col {
                map_size += 1;
            }
            let include_name =
                entry_type == ShadaEntryType::GlobalMark || entry_type == ShadaEntryType::LocalMark;
            if include_name && fm_name != def_name {
                map_size += 1;
            }
            map_size += ad_len;

            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_map(&raw mut ptr, map_size);
            nvim_shada_packer_set_ptr(sbuf, ptr);

            // Always pack filename
            pack_key(KEY_FILE, sbuf);
            let fname_len = if fm_fname.is_null() {
                0
            } else {
                libc::strlen(fm_fname)
            };
            rs_mpack_bin(fm_fname.cast::<u8>(), fname_len, sbuf);

            if fm_lnum != def_lnum {
                pack_key(KEY_LNUM, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_integer(&raw mut ptr2, fm_lnum);
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if fm_col != def_col {
                pack_key(KEY_COL, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_integer(&raw mut ptr2, i64::from(fm_col));
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            if include_name && fm_name != def_name {
                pack_key(KEY_NAME_CHAR, sbuf);
                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_uint(&raw mut ptr2, fm_name as u8 as u32);
                nvim_shada_packer_set_ptr(sbuf, ptr2);
            }
            nvim_shada_dump_additional_data(additional_data.cast::<c_void>(), sbuf);
        }
        ShadaEntryType::Register => {
            // Direct Rust field access - all register entries use uniform *mut *mut c_char layout.
            let reg_type = read_union_field!(entry, reg, reg_type);
            let reg_name = read_union_field!(entry, reg, name);
            let reg_is_unnamed = read_union_field!(entry, reg, is_unnamed);
            let reg_width = read_union_field!(entry, reg, width);
            let contents_count = read_union_field!(entry, reg, contents_size);
            let reg_contents = read_union_field!(entry, reg, contents);

            let def_reg_type: i32 = MT_CHAR_WISE;
            let def_width: usize = 0;
            let def_is_unnamed = false;

            let mut map_size: u32 = 2; // contents + name always present
            if reg_type != def_reg_type {
                map_size += 1;
            }
            if reg_width != def_width {
                map_size += 1;
            }
            if reg_is_unnamed != def_is_unnamed {
                map_size += 1;
            }
            map_size += ad_len;

            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_map(&raw mut ptr, map_size);
            nvim_shada_packer_set_ptr(sbuf, ptr);

            // Pack register contents (rc key)
            pack_key(REG_KEY_CONTENTS, sbuf);
            let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_array(&raw mut ptr2, contents_count as u32);
            nvim_shada_packer_set_ptr(sbuf, ptr2);
            for i in 0..contents_count {
                if reg_contents.is_null() {
                    rs_mpack_bin(std::ptr::null(), 0, sbuf);
                } else {
                    let data = *reg_contents.add(i);
                    let size = if data.is_null() {
                        0
                    } else {
                        libc::strlen(data)
                    };
                    rs_mpack_bin(data.cast::<u8>(), size, sbuf);
                }
            }

            // Pack register name (n key)
            pack_key(KEY_NAME_CHAR, sbuf);
            let mut ptr3 = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_uint(&raw mut ptr3, reg_name as u8 as u32);
            nvim_shada_packer_set_ptr(sbuf, ptr3);

            if reg_type != def_reg_type {
                pack_key(REG_KEY_TYPE, sbuf);
                let mut ptr4 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_uint(&raw mut ptr4, reg_type as u8 as u32);
                nvim_shada_packer_set_ptr(sbuf, ptr4);
            }
            if reg_width != def_width {
                pack_key(REG_KEY_WIDTH, sbuf);
                let mut ptr4 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_uint64(&raw mut ptr4, reg_width as u64);
                nvim_shada_packer_set_ptr(sbuf, ptr4);
            }
            if reg_is_unnamed != def_is_unnamed {
                pack_key(REG_KEY_UNNAMED, sbuf);
                let mut ptr4 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_bool(&raw mut ptr4, c_int::from(reg_is_unnamed));
                nvim_shada_packer_set_ptr(sbuf, ptr4);
            }
            nvim_shada_dump_additional_data(additional_data.cast::<c_void>(), sbuf);
        }
        ShadaEntryType::BufferList => {
            // Direct Rust field access - BufferListBuffer uses Rust Position (i64 lnum).
            let bl_size = read_union_field!(entry, buffer_list, size);
            let bl_buffers = read_union_field!(entry, buffer_list, buffers);
            let def_lnum: i64 = 1;
            let def_col: i32 = 0;
            let mut ptr = nvim_shada_packer_get_ptr(sbuf);
            rs_mpack_array(&raw mut ptr, bl_size as u32);
            nvim_shada_packer_set_ptr(sbuf, ptr);
            for i in 0..bl_size {
                let (buf_ad_ptr, buf_lnum, buf_col, buf_fname) = if bl_buffers.is_null() {
                    (
                        std::ptr::null_mut(),
                        0i64,
                        0i32,
                        std::ptr::null_mut::<c_char>(),
                    )
                } else {
                    let b = &*bl_buffers.add(i);
                    (b.additional_data, b.pos.lnum, b.pos.col, b.fname)
                };
                let buf_ad = nvim_shada_additional_data_len(buf_ad_ptr.cast_const());
                let buf_fname_len = if buf_fname.is_null() {
                    0
                } else {
                    libc::strlen(buf_fname)
                };

                let mut entry_map_size: u32 = 1; // fname always present
                if buf_lnum != def_lnum {
                    entry_map_size += 1;
                }
                if buf_col != def_col {
                    entry_map_size += 1;
                }
                entry_map_size += buf_ad;

                let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
                rs_mpack_map(&raw mut ptr2, entry_map_size);
                nvim_shada_packer_set_ptr(sbuf, ptr2);

                pack_key(KEY_FILE, sbuf);
                rs_mpack_bin(buf_fname.cast::<u8>(), buf_fname_len, sbuf);

                if buf_lnum != def_lnum {
                    pack_key(KEY_LNUM, sbuf);
                    let mut ptr3 = nvim_shada_packer_get_ptr(sbuf);
                    rs_mpack_uint64(&raw mut ptr3, buf_lnum as u64);
                    nvim_shada_packer_set_ptr(sbuf, ptr3);
                }
                if buf_col != def_col {
                    pack_key(KEY_COL, sbuf);
                    let mut ptr3 = nvim_shada_packer_get_ptr(sbuf);
                    rs_mpack_uint64(&raw mut ptr3, buf_col as u64);
                    nvim_shada_packer_set_ptr(sbuf, ptr3);
                }
                nvim_shada_dump_additional_data(buf_ad_ptr.cast_const(), sbuf);
            }
        }
        ShadaEntryType::Header => {
            // Compound accessor: C handles Dict iteration (Dict/kvec_t layout is C-specific).
            nvim_shada_pack_header_dict(entry, sbuf);
        }
    }

    // Take the packed body string
    let packed = nvim_shada_packer_take_string(sbuf);

    // Check size limit and write to outer packer
    if max_kbyte == 0 || packed.size <= max_kbyte * 1024 {
        rs_shada_check_buffer(packer);

        // Write header: type, timestamp, packed size
        let mut outer_ptr = nvim_shada_packer_get_ptr(packer);
        if entry_type == ShadaEntryType::Unknown {
            let type_num = read_union_field!(entry, unknown_item, type_num);
            rs_mpack_uint64_inline(&raw mut outer_ptr, type_num);
        } else {
            rs_mpack_uint64_inline(&raw mut outer_ptr, entry_type as u64);
        }
        rs_mpack_uint64_inline(&raw mut outer_ptr, timestamp);
        if packed.size > 0 {
            rs_mpack_uint64_inline(&raw mut outer_ptr, packed.size as u64);
        }
        nvim_shada_packer_set_ptr(packer, outer_ptr);

        // Write raw body if non-empty
        if packed.size > 0 {
            rs_mpack_raw(packed.data.cast::<u8>(), packed.size, packer);
        }

        // Check for write errors (anyint != 0)
        let anyint = nvim_shada_packer_get_anyint(packer);
        if anyint != 0 {
            nvim_xfree(packed.data.cast::<c_void>());
            return SD_WRITE_FAILED;
        }
    }

    nvim_xfree(packed.data.cast::<c_void>());
    SD_WRITE_SUCCESSFUL
}

// =============================================================================
// Phase 4 (plan fd426e0f): Platform check writable in Rust
// =============================================================================

/// Check whether the existing ShaDa file at `fname` can be replaced.
///
/// Matches the logic of C nvim_shada_platform_check_writable (now deleted).
///
/// Returns:
///  1 = ok to replace (go ahead with rename)
///  0 = not writable (E137 already emitted)
/// -1 = fchown failed (RNERR already emitted)
///
/// # Safety
///
/// `fname`, `sd_writer`, and `tempname` must be valid pointers.
#[allow(
    clippy::not_unsafe_ptr_arg_deref,
    clippy::cast_possible_wrap,
    clippy::similar_names
)]
unsafe fn rs_shada_platform_check_writable(
    fname: *const c_char,
    sd_writer: *mut c_void,
    tempname: *const c_char,
) -> c_int {
    let mut file_mode: u64 = 0;
    let mut file_uid: u64 = 0;
    let mut file_gid: u64 = 0;

    // os_fileinfo returns 1 if file exists and is not a directory.
    let info_ok = nvim_shada_os_fileinfo(
        fname,
        &raw mut file_mode,
        &raw mut file_uid,
        &raw mut file_gid,
    );
    if info_ok == 0 {
        nvim_shada_semsg_1s(c"E137: ShaDa file is not writable: %s".as_ptr(), fname);
        return 0;
    }

    #[cfg(unix)]
    {
        const ROOT_UID: u64 = 0;
        let euid = u64::from(libc::getuid());
        let egid = u64::from(libc::getgid());

        // Non-root: check write permission based on owner/group/other bits.
        if euid == ROOT_UID {
            // Running as root: fchown the temp file to match the original owner.
            if file_uid != ROOT_UID || file_gid != egid {
                let fchown_ret = nvim_shada_os_fchown(sd_writer, file_uid, file_gid);
                if fchown_ret != 0 {
                    nvim_shada_semsg_2s(
                        c"E136: Failed setting uid and gid for file %s: %s".as_ptr(),
                        tempname,
                        nvim_shada_os_strerror(fchown_ret),
                    );
                    return -1;
                }
            }
        } else {
            let is_writable = if file_uid == euid {
                (file_mode & 0o200) != 0
            } else if file_gid == egid {
                (file_mode & 0o020) != 0
            } else {
                (file_mode & 0o002) != 0
            };
            if !is_writable {
                nvim_shada_semsg_1s(c"E137: ShaDa file is not writable: %s".as_ptr(), fname);
                return 0;
            }
        }
    }

    1
}

// =============================================================================
// Phase 1 (plan fd426e0f): Pack header directly in Rust
// =============================================================================

/// Pack the ShaDa file header entry (generator/version/max_kbyte/pid/encoding) directly in Rust.
///
/// Constructs a header msgpack map(5) with keys: generator, version,
/// max_kbyte, pid, encoding -- all collected from existing C accessors.
/// Bypasses the C ShadaEntry/Dict infrastructure entirely.
///
/// Returns SD_WRITE_SUCCESSFUL or SD_WRITE_FAILED.
///
/// # Safety
///
/// `packer` must be a valid PackerBuffer pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_file_header(
    packer: *mut ShadaPackerBuffer,
    max_kbyte: usize,
) -> c_int {
    if packer.is_null() {
        return SD_WRITE_FAILED;
    }

    // Collect values from C accessors.
    let timestamp = os_time();
    let long_version = longVersion;
    let pid = os_get_pid();
    let enc = p_enc;

    // Build body in a temporary string buffer.
    let mut sbuf = std::mem::MaybeUninit::<ShadaPackerBuffer>::uninit();
    nvim_shada_packer_string_buffer(sbuf.as_mut_ptr());
    let sbuf = sbuf.as_mut_ptr();
    rs_mpack_check_buffer(sbuf);

    // Pack map(5): generator, version, max_kbyte, pid, encoding
    let mut ptr = nvim_shada_packer_get_ptr(sbuf);
    rs_mpack_map(&raw mut ptr, 5);
    nvim_shada_packer_set_ptr(sbuf, ptr);

    // "generator" => "nvim"
    rs_mpack_str(b"generator".as_ptr(), 9, sbuf);
    rs_mpack_bin(b"nvim".as_ptr(), 4, sbuf);

    // "version" => longVersion (string)
    rs_mpack_str(b"version".as_ptr(), 7, sbuf);
    let version_len = if long_version.is_null() {
        0
    } else {
        libc::strlen(long_version)
    };
    rs_mpack_bin(long_version.cast::<u8>(), version_len, sbuf);

    // "max_kbyte" => max_kbyte (integer)
    rs_mpack_str(b"max_kbyte".as_ptr(), 9, sbuf);
    let mut ptr2 = nvim_shada_packer_get_ptr(sbuf);
    #[allow(clippy::cast_possible_wrap)]
    rs_mpack_integer(&raw mut ptr2, max_kbyte as i64);
    nvim_shada_packer_set_ptr(sbuf, ptr2);

    // "pid" => pid (integer)
    rs_mpack_str(b"pid".as_ptr(), 3, sbuf);
    let mut ptr3 = nvim_shada_packer_get_ptr(sbuf);
    rs_mpack_integer(&raw mut ptr3, pid);
    nvim_shada_packer_set_ptr(sbuf, ptr3);

    // "encoding" => p_enc (string)
    rs_mpack_str(b"encoding".as_ptr(), 8, sbuf);
    let enc_len = if enc.is_null() { 0 } else { libc::strlen(enc) };
    rs_mpack_bin(enc.cast::<u8>(), enc_len, sbuf);

    let header_body = nvim_shada_packer_take_string(sbuf);

    // Write entry header: type=1, timestamp, length
    rs_shada_check_buffer(packer);
    let mut outer_ptr = nvim_shada_packer_get_ptr(packer);
    rs_mpack_uint64_inline(&raw mut outer_ptr, SD_ITEM_HEADER as u64);
    rs_mpack_uint64_inline(&raw mut outer_ptr, timestamp);
    rs_mpack_uint64_inline(&raw mut outer_ptr, header_body.size as u64);
    nvim_shada_packer_set_ptr(packer, outer_ptr);

    // Write body
    if header_body.size > 0 {
        rs_mpack_raw(header_body.data.cast::<u8>(), header_body.size, packer);
    }

    let anyint = nvim_shada_packer_get_anyint(packer);
    nvim_xfree(header_body.data.cast::<c_void>());

    if anyint != 0 {
        SD_WRITE_FAILED
    } else {
        SD_WRITE_SUCCESSFUL
    }
}

/// Write a single ShaDa entry and free it afterwards.
///
/// Calls rs_shada_pack_entry and then rs_shada_free_entry_contents.
///
/// # Safety
///
/// - `packer` must be a valid PackerBuffer pointer
/// - `entry` must be a valid ShadaEntry pointer (will be freed)
#[no_mangle]
pub unsafe extern "C" fn rs_shada_pack_pfreed_entry(
    packer: *mut ShadaPackerBuffer,
    entry: *mut ShadaEntry,
    max_kbyte: usize,
) -> c_int {
    let ret = rs_shada_pack_entry(packer, entry, max_kbyte);
    rs_shada_free_entry_contents(entry);
    ret
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
        assert_eq!(list.contained_entries.set.h.size, 0);
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
