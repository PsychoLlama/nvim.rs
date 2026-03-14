//! Undo system utilities for Neovim
//!
//! This crate provides functions for Neovim's multi-level undo/redo system.
//!
//! ## Architecture
//!
//! The undo system uses an opaque handle pattern for interoperability with C:
//! - `BufHandle` - Handle to C `buf_T` buffer struct
//! - `UHeaderHandle` - Handle to C `u_header_T` undo header struct
//! - `UEntryHandle` - Handle to C `u_entry_T` undo entry struct
//!
//! ## Major Functions
//!
//! - `rs_u_write_undo` - Write undo tree to persistent file
//! - `rs_u_read_undo` - Read undo tree from persistent file
//! - `rs_ex_undolist` - `:undolist` Ex command implementation
//! - `rs_u_eval_tree` - Build VimL dict for `undotree()`
//! - `rs_f_undofile` - `undofile()` VimL function
//! - `rs_undo_time` - Navigate undo tree by time/sequence
//!
//! ## File I/O Infrastructure
//!
//! This module includes types and helpers for undo file persistence:
//! - Magic bytes and version constants for file format
//! - Serialization/deserialization helpers for undo headers and entries
//! - File I/O wrappers for reading/writing undo files

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr;

use nvim_encoding::sha256::Sha256Context;

/// Opaque handle to buf_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BufHandle(*mut c_void);

/// repr(C) struct matching pos_T (position in file or buffer).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PosT {
    /// Line number.
    pub lnum: LinenrT,
    /// Column number.
    pub col: c_int,
    /// Column addition (for virtual characters).
    pub coladd: c_int,
}

/// repr(C) struct matching fmarkv_T (view in which a mark was created).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FmarkV {
    /// Lines from mark lnum to top of window; MAXLNUM = no view.
    pub topline_offset: LinenrT,
}

/// repr(C) struct matching fmark_T (single local mark).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FmarkT {
    /// Cursor position.
    pub mark: PosT,
    /// File number.
    pub fnum: c_int,
    /// Timestamp when this mark was last set.
    pub timestamp: u64,
    /// View the mark was created on.
    pub view: FmarkV,
    // Padding: 4 bytes implicit between view and additional_data
    /// Additional data from ShaDa file (opaque).
    pub additional_data: *mut c_void,
}

/// repr(C) struct matching visualinfo_T (Visual area info).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VisualInfoT {
    /// Start position of last VIsual.
    pub vi_start: PosT,
    /// End position of last VIsual.
    pub vi_end: PosT,
    /// VIsual_mode of last VIsual.
    pub vi_mode: c_int,
    /// MAXCOL from w_curswant.
    pub vi_curswant: c_int,
}

/// repr(C) union matching the uh_next/prev/alt_next/alt_prev union in u_header_T.
///
/// At runtime, `.ptr` is used for linked-list traversal.
/// During file serialization/deserialization, `.seq` holds sequence numbers.
#[repr(C)]
#[derive(Clone, Copy)]
pub union UhLink {
    /// Pointer to linked header (runtime use).
    pub ptr: *mut UHeader,
    /// Sequence number (serialization use).
    pub seq: c_int,
}

/// repr(C) struct matching kvec_t(T) generic kvec layout.
///
/// The kvec macro expands to `struct { size_t size; size_t capacity; T *items; }`.
#[repr(C)]
pub struct KVec<T> {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut T,
}

/// C enum matching UndoObjectType.
pub type UndoObjectType = c_int;
pub const K_EXTMARK_SPLICE: UndoObjectType = 0; // kExtmarkSplice
pub const K_EXTMARK_MOVE: UndoObjectType = 1; // kExtmarkMove
pub const K_EXTMARK_UPDATE: UndoObjectType = 2; // kExtmarkUpdate
pub const K_EXTMARK_SAVEPOS: UndoObjectType = 3; // kExtmarkSavePos
pub const K_EXTMARK_CLEAR: UndoObjectType = 4; // kExtmarkClear

/// repr(C) struct matching ExtmarkSplice.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExtmarkSplice {
    pub start_row: c_int,
    pub start_col: c_int,
    pub old_row: c_int,
    pub old_col: c_int,
    pub new_row: c_int,
    pub new_col: c_int,
    pub start_byte: i64, // bcount_t = ptrdiff_t = i64 on 64-bit
    pub old_byte: i64,
    pub new_byte: i64,
}

/// repr(C) struct matching ExtmarkMove.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExtmarkMove {
    pub start_row: c_int,
    pub start_col: c_int,
    pub extent_row: c_int,
    pub extent_col: c_int,
    pub new_row: c_int,
    pub new_col: c_int,
    pub start_byte: i64,
    pub extent_byte: i64,
    pub new_byte: i64,
}

/// repr(C) struct matching ExtmarkSavePos.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExtmarkSavePos {
    pub mark: u64,
    pub old_row: c_int,
    pub old_col: c_int,
    pub invalidated: bool,
    // 7 bytes padding
}

/// repr(C) union matching the data union in ExtmarkUndoObject.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtmarkUndoObjectData {
    pub splice: ExtmarkSplice,
    pub mover: ExtmarkMove, // 'move' is a Rust keyword
    pub savepos: ExtmarkSavePos,
}

/// repr(C) struct matching ExtmarkUndoObject (struct undo_object in C).
#[repr(C)]
pub struct ExtmarkUndoObject {
    pub kind: UndoObjectType,
    // 4 bytes padding to align data union to 8
    pub data: ExtmarkUndoObjectData,
}

/// repr(C) struct matching u_header_T (undo header).
///
/// # Layout (64-bit)
///
/// Matches the C `u_header_T` struct layout exactly. The U_DEBUG `uh_magic`
/// field is excluded (only present when `U_DEBUG` is defined in C).
#[repr(C)]
pub struct UHeader {
    /// Next header in undo list (runtime: ptr; serialization: seq).
    pub uh_next: UhLink,
    /// Previous header in undo list.
    pub uh_prev: UhLink,
    /// Next header for alternate redo branch.
    pub uh_alt_next: UhLink,
    /// Previous header for alternate redo branch.
    pub uh_alt_prev: UhLink,
    /// Sequence number (higher = newer undo).
    pub uh_seq: c_int,
    /// Used by undo_time().
    pub uh_walk: c_int,
    /// Pointer to first entry.
    pub uh_entry: *mut UEntry,
    /// Pointer to where ue_bot must be set.
    pub uh_getbot_entry: *mut UEntry,
    /// Cursor position before saving.
    pub uh_cursor: PosT,
    /// Cursor virtual column.
    pub uh_cursor_vcol: c_int,
    /// Flags (UH_CHANGED, UH_EMPTYBUF, UH_RELOAD).
    pub uh_flags: c_int,
    /// Named marks before undo/after redo.
    pub uh_namedm: [FmarkT; 26], // NMARKS = 26
    /// Info to move extmarks.
    pub uh_extmark: KVec<ExtmarkUndoObject>,
    /// Visual areas before undo/after redo.
    pub uh_visual: VisualInfoT,
    /// Timestamp when the change was made.
    pub uh_time: TimeT,
    /// Set when the file was saved after changes in this block.
    pub uh_save_nr: c_int,
}

/// Type alias for u_header_T pointer (replaces opaque UHeaderHandle).
pub type UHeaderHandle = *mut UHeader;

/// Compile-time layout checks for new repr(C) types (64-bit only).
#[cfg(target_pointer_width = "64")]
const _: () = {
    // pos_T: linenr_T(4) + colnr_T(4) + colnr_T(4) = 12 bytes
    assert!(std::mem::size_of::<PosT>() == 12);
    assert!(std::mem::offset_of!(PosT, lnum) == 0);
    assert!(std::mem::offset_of!(PosT, col) == 4);
    assert!(std::mem::offset_of!(PosT, coladd) == 8);

    // fmarkv_T: linenr_T(4) = 4 bytes
    assert!(std::mem::size_of::<FmarkV>() == 4);

    // fmark_T: pos_T(12) + int(4) + u64(8) + fmarkv_T(4) + pad(4) + ptr(8) = 40 bytes
    assert!(std::mem::size_of::<FmarkT>() == 40);
    assert!(std::mem::offset_of!(FmarkT, mark) == 0);
    assert!(std::mem::offset_of!(FmarkT, fnum) == 12);
    assert!(std::mem::offset_of!(FmarkT, timestamp) == 16);
    assert!(std::mem::offset_of!(FmarkT, view) == 24);
    assert!(std::mem::offset_of!(FmarkT, additional_data) == 32);

    // visualinfo_T: pos_T(12) + pos_T(12) + int(4) + int(4) = 32 bytes
    assert!(std::mem::size_of::<VisualInfoT>() == 32);
    assert!(std::mem::offset_of!(VisualInfoT, vi_start) == 0);
    assert!(std::mem::offset_of!(VisualInfoT, vi_end) == 12);
    assert!(std::mem::offset_of!(VisualInfoT, vi_mode) == 24);
    assert!(std::mem::offset_of!(VisualInfoT, vi_curswant) == 28);

    // UhLink union: max(ptr=8, seq=4) = 8 bytes (ptr alignment dominates)
    assert!(std::mem::size_of::<UhLink>() == 8);

    // ExtmarkSplice: 6 ints(24) + 3 i64(24) = 48 bytes
    assert!(std::mem::size_of::<ExtmarkSplice>() == 48);

    // ExtmarkMove: same as ExtmarkSplice
    assert!(std::mem::size_of::<ExtmarkMove>() == 48);

    // ExtmarkSavePos: u64(8) + int(4) + int(4) + bool(1) + pad(7) = 24 bytes
    assert!(std::mem::size_of::<ExtmarkSavePos>() == 24);

    // ExtmarkUndoObject: int(4) + pad(4) + union(48) = 56 bytes
    assert!(std::mem::size_of::<ExtmarkUndoObject>() == 56);
    assert!(std::mem::offset_of!(ExtmarkUndoObject, kind) == 0);

    // u_header_T layout (64-bit):
    // 4 x UhLink(8) = 32, uh_seq+uh_walk = 8, 2 x ptr(8) = 16,
    // cursor(12)+cursor_vcol(4)+flags(4) = 20, pad(4),
    // namedm[26](40*26=1040), extmark(24), visual(32), time(8), save_nr(4) + pad(4)
    assert!(std::mem::offset_of!(UHeader, uh_next) == 0);
    assert!(std::mem::offset_of!(UHeader, uh_prev) == 8);
    assert!(std::mem::offset_of!(UHeader, uh_alt_next) == 16);
    assert!(std::mem::offset_of!(UHeader, uh_alt_prev) == 24);
    assert!(std::mem::offset_of!(UHeader, uh_seq) == 32);
    assert!(std::mem::offset_of!(UHeader, uh_walk) == 36);
    assert!(std::mem::offset_of!(UHeader, uh_entry) == 40);
    assert!(std::mem::offset_of!(UHeader, uh_getbot_entry) == 48);
    assert!(std::mem::offset_of!(UHeader, uh_cursor) == 56);
    assert!(std::mem::offset_of!(UHeader, uh_cursor_vcol) == 68);
    assert!(std::mem::offset_of!(UHeader, uh_flags) == 72);
    assert!(std::mem::offset_of!(UHeader, uh_namedm) == 80);
    assert!(std::mem::offset_of!(UHeader, uh_extmark) == 1120);
    assert!(std::mem::offset_of!(UHeader, uh_visual) == 1144);
    assert!(std::mem::offset_of!(UHeader, uh_time) == 1176);
    assert!(std::mem::offset_of!(UHeader, uh_save_nr) == 1184);
};

/// repr(C) struct matching u_entry_T (without U_DEBUG field).
///
/// # Layout
///
/// Matches the C `u_entry_T` struct layout exactly. The U_DEBUG `ue_magic` field
/// is excluded (it is only present when `U_DEBUG` is defined in C).
#[repr(C)]
pub struct UEntry {
    /// Pointer to next entry in list.
    pub ue_next: *mut UEntry,
    /// Number of line above undo block.
    pub ue_top: LinenrT,
    /// Number of line below undo block.
    pub ue_bot: LinenrT,
    /// Line count when u_save called.
    pub ue_lcount: LinenrT,
    /// Array of lines in undo block.
    pub ue_array: *mut *mut c_char,
    /// Number of lines in ue_array.
    pub ue_size: LinenrT,
}

/// Type alias for u_entry_T pointer (replaces opaque UEntryHandle).
pub type UEntryHandle = *mut UEntry;

/// Compile-time check that UEntry has the expected 64-bit layout.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(
        std::mem::size_of::<UEntry>() == 40,
        "UEntry size must be 40 bytes on 64-bit"
    );
    assert!(std::mem::offset_of!(UEntry, ue_next) == 0);
    assert!(std::mem::offset_of!(UEntry, ue_top) == 8);
    assert!(std::mem::offset_of!(UEntry, ue_bot) == 12);
    assert!(std::mem::offset_of!(UEntry, ue_lcount) == 16);
    assert!(std::mem::offset_of!(UEntry, ue_array) == 24);
    assert!(std::mem::offset_of!(UEntry, ue_size) == 32);
};

/// Opaque handle to win_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct WinHandle(*mut c_void);

/// Type alias for time_t (platform-dependent).
#[cfg(target_pointer_width = "64")]
pub type TimeT = i64;
#[cfg(target_pointer_width = "32")]
pub type TimeT = i32;

/// Type alias for linenr_T (line number type).
pub type LinenrT = i32;

/// Type alias for colnr_T (column number type).
pub type ColnrT = c_int;

/// Success return value (matches Neovim's OK).
const OK: c_int = 1;

/// Failure return value (matches Neovim's FAIL).
const FAIL: c_int = 0;

/// Maximum line number (invalid). Matches C `MAXLNUM`.
const MAXLNUM: LinenrT = 0x7fff_ffff;

/// Undo header flag: b_changed flag before undo/after redo.
const UH_CHANGED: c_int = 0x01;
/// Undo header flag: buffer was empty.
const UH_EMPTYBUF: c_int = 0x02;
/// Undo header flag: buffer was reloaded.
const UH_RELOAD: c_int = 0x04;

// =============================================================================
// Undo File Format Constants
// =============================================================================

/// Magic bytes at the start of undo file: "Vim\237UnDo\345"
pub const UF_START_MAGIC: &[u8; 9] = b"Vim\x9fUnDo\xe5";
/// Length of the start magic bytes.
pub const UF_START_MAGIC_LEN: usize = 9;

/// Magic at start of header.
pub const UF_HEADER_MAGIC: u16 = 0x5fd0;
/// Magic after last header.
pub const UF_HEADER_END_MAGIC: u16 = 0xe7aa;
/// Magic at start of entry.
pub const UF_ENTRY_MAGIC: u16 = 0xf518;
/// Magic after last entry.
pub const UF_ENTRY_END_MAGIC: u16 = 0x3581;

/// 2-byte undofile version number.
pub const UF_VERSION: u16 = 3;

/// Extra field identifier for last save number in header.
pub const UF_LAST_SAVE_NR: u8 = 1;

/// Extra field identifier for save number in uhp.
pub const UHP_SAVE_NR: u8 = 1;

/// Size of SHA-256 hash used in undo files.
pub const UNDO_HASH_SIZE: usize = 32;

// =============================================================================
// Undo File I/O Handle
// =============================================================================

/// Opaque handle to FILE* for undo file operations.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FileHandle(*mut c_void);

impl FileHandle {
    /// Create a null file handle.
    #[inline]
    pub fn null() -> Self {
        FileHandle(ptr::null_mut())
    }

    /// Check if the handle is null.
    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// FFI declarations for C accessor functions
#[allow(dead_code)]
extern "C" {
    // Buffer undo field accessors
    fn nvim_buf_get_b_u_oldhead(buf: BufHandle) -> UHeaderHandle;
    fn nvim_buf_get_b_u_newhead(buf: BufHandle) -> UHeaderHandle;
    fn nvim_buf_get_b_u_curhead(buf: BufHandle) -> UHeaderHandle;
    fn nvim_buf_get_b_u_numhead(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_u_synced(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_u_line_ptr(buf: BufHandle) -> *mut c_char;
    fn nvim_buf_get_b_u_line_lnum(buf: BufHandle) -> LinenrT;

    fn nvim_buf_set_b_u_oldhead(buf: BufHandle, val: UHeaderHandle);
    fn nvim_buf_set_b_u_newhead(buf: BufHandle, val: UHeaderHandle);
    fn nvim_buf_set_b_u_curhead(buf: BufHandle, val: UHeaderHandle);
    fn nvim_buf_set_b_u_numhead(buf: BufHandle, val: c_int);
    fn nvim_buf_set_b_u_synced(buf: BufHandle, val: bool);
    fn nvim_buf_set_b_u_line_ptr(buf: BufHandle, val: *mut c_char);
    fn nvim_buf_set_b_u_line_lnum(buf: BufHandle, val: LinenrT);

    // Buffer state accessors
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_bt_dontwrite(buf: BufHandle) -> bool;
    #[link_name = "rs_bt_prompt"]
    fn nvim_bt_prompt(buf: BufHandle) -> bool;
    #[link_name = "file_ff_differs"]
    fn nvim_file_ff_differs(buf: BufHandle, strict: bool) -> bool;

    // Global buffer iteration
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // Memory functions
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_void);

    // Allocation functions
    #[link_name = "xcalloc"]
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;

    // Buffer memline accessor
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;

    // Error message wrappers
    fn nvim_iemsg_undo_list_corrupt();
    fn nvim_iemsg_undo_line_missing();

    // Global state accessors
    fn nvim_get_no_u_sync() -> c_int;
    fn nvim_get_undolevel(buf: BufHandle) -> i64;

    // Buffer b_did_warn accessor
    fn nvim_buf_set_b_did_warn(buf: BufHandle, val: bool);

    // Buffer save_nr accessors
    fn nvim_buf_get_b_u_save_nr_last(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_save_nr_last(buf: BufHandle, val: c_int);
    fn nvim_buf_set_b_u_save_nr_cur(buf: BufHandle, val: c_int);

    // undo_allowed accessors
    fn nvim_buf_is_modifiable(buf: BufHandle) -> bool;
    fn nvim_get_sandbox() -> c_int;
    fn nvim_get_textlock() -> c_int;
    fn nvim_get_expr_map_lock() -> c_int;
    fn nvim_curbuf_is_dummy() -> c_int;

    // undo_allowed error message wrappers
    fn nvim_emsg_modifiable();
    fn nvim_emsg_sandbox();
    fn nvim_emsg_textlock();

    // ex_undojoin error message wrapper
    fn nvim_emsg_undojoin_after_undo();

    // u_undo/u_redo accessors
    fn nvim_has_cpo_undo() -> bool;
    fn nvim_get_undo_undoes() -> bool;
    fn nvim_set_undo_undoes(val: bool);

    // u_undo_and_forget accessors
    fn nvim_buf_get_b_u_seq_cur(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_seq_cur(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_seq_last(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_seq_last(buf: BufHandle, val: c_int);

    // u_doit accessors
    fn nvim_buf_ml_is_empty(buf: BufHandle) -> bool;
    fn nvim_get_u_newcount() -> c_int;
    fn nvim_set_u_newcount(val: c_int);
    fn nvim_get_u_oldcount() -> c_int;
    fn nvim_set_u_oldcount(val: c_int);
    fn nvim_msg_ext_set_kind_undo();
    fn nvim_change_warning_curbuf();
    fn nvim_beep_flush();
    fn nvim_msg_oldest_change();
    fn nvim_msg_newest_change();

    // Infrastructure for future migration (u_savecommon, etc.)
    fn nvim_ml_get_buf_copy(buf: BufHandle, lnum: LinenrT) -> *mut c_char;
    #[link_name = "fast_breakcheck"]
    fn nvim_fast_breakcheck();
    fn nvim_undo_got_int() -> bool;
    fn nvim_time_now() -> TimeT;
    fn nvim_get_curwin_cursor(lnum: *mut LinenrT, col: *mut ColnrT, coladd: *mut ColnrT);
    fn nvim_curwin_virtual_active() -> bool;
    #[link_name = "getviscol"]
    fn nvim_getviscol() -> ColnrT;

    // u_savecommon infrastructure
    fn nvim_buf_set_b_new_change(buf: BufHandle, val: bool);
    fn nvim_buf_set_b_u_time_cur(buf: BufHandle, val: TimeT);
    fn nvim_uhp_copy_marks_visual(buf: BufHandle, uhp: UHeaderHandle);
    fn nvim_emsg_line_count_changed();
    fn nvim_buf_is_curbuf(buf: BufHandle) -> bool;

    // u_find_first_changed infrastructure (cursor now accessed via direct field access)

    // u_undoline accessors
    fn nvim_buf_get_b_u_line_colnr(buf: BufHandle) -> ColnrT;
    fn nvim_buf_set_b_u_line_colnr(buf: BufHandle, val: ColnrT);
    fn nvim_undo_curwin_get_cursor_col() -> ColnrT;
    fn nvim_undo_curwin_set_cursor_col(col: ColnrT);
    fn nvim_undo_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_undo_curwin_set_cursor_lnum(lnum: LinenrT);
    fn nvim_check_cursor_col_curwin();
    fn nvim_u_undoline_replace_and_swap();

    // undo_time accessors
    fn nvim_buf_get_b_u_time_cur(buf: BufHandle) -> TimeT;
    fn nvim_buf_get_b_u_save_nr_cur(buf: BufHandle) -> c_int;
    fn nvim_text_locked() -> bool;
    fn nvim_text_locked_msg();
    fn nvim_undo_os_time() -> TimeT;
    fn nvim_inc_lastmark() -> c_int;
    fn nvim_internal_error_undo_time();
    fn nvim_semsg_undo_number_not_found(step: i64);

    // ==========================================================================
    // Undo File I/O FFI Functions
    // ==========================================================================

    // File operations
    #[link_name = "fclose"]
    fn nvim_undo_fclose(fp: FileHandle) -> c_int;
    #[link_name = "fwrite"]
    fn nvim_undo_fwrite(ptr: *const c_void, size: usize, count: usize, fp: FileHandle) -> usize;
    #[link_name = "fread"]
    fn nvim_undo_fread(ptr: *mut c_void, size: usize, count: usize, fp: FileHandle) -> usize;
    #[link_name = "fflush"]
    fn nvim_undo_fflush(fp: FileHandle) -> c_int;
    #[link_name = "getc"]
    fn nvim_undo_fgetc(fp: FileHandle) -> c_int;

    // File I/O helpers (reading from C file handle)
    #[link_name = "get2c"]
    fn nvim_undo_get2c(fp: FileHandle) -> c_int;
    #[link_name = "get4c"]
    fn nvim_undo_get4c(fp: FileHandle) -> c_int;
    #[link_name = "get8ctime"]
    fn nvim_undo_get8ctime(fp: FileHandle) -> TimeT;

    // Buffer file path accessors
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;

    // File system operations
    #[link_name = "os_path_exists"]
    fn nvim_os_path_exists(path: *const c_char) -> bool;
    #[link_name = "os_remove"]
    fn nvim_os_remove(path: *const c_char) -> c_int;
    #[link_name = "os_open"]
    fn nvim_os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int;
    #[link_name = "os_close"]
    fn nvim_os_close(fd: c_int) -> c_int;
    #[link_name = "os_getperm"]
    fn nvim_os_getperm(path: *const c_char) -> c_int;
    #[link_name = "os_setperm"]
    fn nvim_os_setperm(path: *const c_char, perm: c_int) -> c_int;
    #[link_name = "os_fsync"]
    fn nvim_os_fsync(fd: c_int) -> c_int;
    #[link_name = "fdopen"]
    fn nvim_fdopen(fd: c_int, mode: *const c_char) -> FileHandle;
    #[link_name = "fileno"]
    fn nvim_fileno(fp: FileHandle) -> c_int;

    // Message functions for undo file I/O
    #[link_name = "verbose_enter"]
    fn nvim_undo_verbose_enter();
    #[link_name = "verbose_leave"]
    fn nvim_undo_verbose_leave();
    fn nvim_undo_semsg(msg: *const c_char, arg: *const c_char);

    // Option accessors
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_get_p_fs() -> bool;

    // u_sync wrapper

    // Buffer line count and line accessors for hash computation
    fn nvim_buf_get_b_ml_line_count(buf: BufHandle) -> LinenrT;
    #[link_name = "ml_get_buf"]
    fn nvim_ml_get_buf_line(buf: BufHandle, lnum: LinenrT) -> *const c_char;

    // ACL operations (Unix)
    #[link_name = "os_get_acl"]
    fn nvim_os_get_acl(path: *const c_char) -> *mut c_void;
    #[link_name = "os_set_acl"]
    fn nvim_os_set_acl(path: *const c_char, acl: *mut c_void);
    #[link_name = "os_free_acl"]
    fn nvim_os_free_acl(acl: *mut c_void);

    // File info for Unix ownership checks
    fn nvim_undo_set_file_group(
        fd: c_int,
        orig_path: *const c_char,
        undo_path: *const c_char,
        perm: c_int,
    ) -> c_int;

    // Read helper for errno handling
    #[link_name = "read_eintr"]
    fn nvim_read_eintr(fd: c_int, buf: *mut c_void, count: usize) -> isize;

    // Global lastmark accessor

    // ==========================================================================
    // Phase 2: Core Undo Operations FFI (memline manipulation)
    // ==========================================================================

    /// Delete line 'lnum' in buffer. Returns OK/FAIL.
    fn nvim_ml_delete_lnum(lnum: LinenrT) -> c_int;

    /// Append line with flags. Returns OK/FAIL.
    fn nvim_ml_append_flags(lnum: LinenrT, line: *const c_char, len: ColnrT, flags: c_int)
        -> c_int;

    /// Replace line in current buffer. Returns OK/FAIL.
    fn nvim_ml_replace_lnum(lnum: LinenrT, line: *const c_char, copy: bool) -> c_int;

    /// Block/unblock autocommands
    #[link_name = "block_autocmds"]
    fn nvim_block_autocmds();
    #[link_name = "unblock_autocmds"]
    fn nvim_unblock_autocmds();

    /// Set pc mark for jump list
    #[link_name = "setpcmark"]
    fn nvim_undo_setpcmark();

    /// Check cursor line number validity and adjust if needed
    #[link_name = "check_cursor_lnum"]
    fn nvim_undo_check_cursor_lnum(win: WinHandle);

    /// Mark adjust for undo
    fn nvim_undo_mark_adjust(top: LinenrT, bot: LinenrT, amount: LinenrT, amount_after: LinenrT);

    /// Changed lines notification
    #[link_name = "changed_lines"]
    fn nvim_undo_changed_lines(
        buf: BufHandle,
        top: LinenrT,
        col: ColnrT,
        bot: LinenrT,
        xtra: LinenrT,
        do_buf_event: bool,
    );

    /// Mark buffer as changed
    #[link_name = "changed"]
    fn nvim_buf_changed(buf: BufHandle);

    /// Mark buffer as unchanged
    #[link_name = "unchanged"]
    fn nvim_buf_unchanged(buf: BufHandle, ff: bool, always_strstruc: bool);

    /// Check spell for window
    #[link_name = "spell_check_window"]
    fn nvim_spell_check_window(win: WinHandle) -> bool;

    /// Redraw window line
    #[link_name = "redrawWinline"]
    fn nvim_redrawWinline(win: WinHandle, lnum: LinenrT);

    /// Apply extmark undo
    fn nvim_extmark_apply_undo(uhp: UHeaderHandle, idx: usize, undo: bool);

    /// Buffer updates unload
    #[link_name = "buf_updates_unload"]
    fn nvim_buf_updates_unload(buf: BufHandle, force: bool);

    /// Current window handle accessor
    fn nvim_undo_get_curwin() -> WinHandle;

    /// Window buffer accessor
    fn nvim_undo_win_get_buffer(win: WinHandle) -> BufHandle;

    /// Set window cursor
    fn nvim_undo_win_set_cursor_pos(win: WinHandle, lnum: LinenrT, col: ColnrT, coladd: ColnrT);

    /// Get window cursor line
    fn nvim_undo_win_get_cursor_lnum(win: WinHandle) -> LinenrT;

    /// Save line for undo (returns allocated string)
    fn nvim_u_save_line_for_undo(buf: BufHandle, lnum: LinenrT) -> *mut c_char;

    /// Get global_busy flag
    fn nvim_get_global_busy() -> bool;

    /// Check if messaging is allowed
    fn nvim_messaging() -> bool;

    /// Get KeyTyped flag
    fn nvim_undo_get_key_typed() -> bool;

    /// Get fdo_flags for fold options
    fn nvim_undo_get_fdo_flags() -> c_int;

    /// Fold open cursor (calls Rust rs_foldOpenCursor directly)
    fn rs_foldOpenCursor();

    // ==========================================================================
    // Phase 3: u_undoredo FFI helpers
    // ==========================================================================

    /// Save named marks before undo/redo (zeros additional_data)
    fn nvim_undoredo_save_marks(buf: BufHandle, curhead: UHeaderHandle);

    /// Restore named marks from undo header to buffer and vice versa
    fn nvim_undoredo_restore_marks(
        buf: BufHandle,
        curhead: UHeaderHandle,
        saved_namedm: *const c_void,
    );

    /// Swap visual info between buffer and undo header
    fn nvim_undoredo_swap_visual(
        buf: BufHandle,
        curhead: UHeaderHandle,
        saved_visual: *const c_void,
    );

    /// Get saved namedm array and visual info from buffer
    fn nvim_undoredo_get_buf_marks(
        buf: BufHandle,
        out_namedm: *mut c_void,
        out_visual: *mut c_void,
    );

    /// Set b_op_start and b_op_end initial values
    fn nvim_undoredo_init_op_marks(buf: BufHandle);

    /// Get b_op_start.lnum
    fn nvim_buf_get_op_start_lnum(buf: BufHandle) -> LinenrT;

    /// Get b_op_end.lnum
    fn nvim_buf_get_op_end_lnum(buf: BufHandle) -> LinenrT;

    /// Set b_op_start.lnum
    fn nvim_buf_set_op_start_lnum(buf: BufHandle, lnum: LinenrT);

    /// Adjust b_op_start.lnum by delta
    fn nvim_buf_adjust_op_start_lnum(buf: BufHandle, delta: LinenrT);

    /// Set b_op_end.lnum
    fn nvim_buf_set_op_end_lnum(buf: BufHandle, lnum: LinenrT);

    /// Adjust b_op_end.lnum by delta
    fn nvim_buf_adjust_op_end_lnum(buf: BufHandle, delta: LinenrT);

    /// Clamp op marks to ml_line_count
    fn nvim_undoredo_clamp_op_marks(buf: BufHandle);

    /// Set ML_EMPTY flag if needed
    fn nvim_undoredo_set_ml_empty(buf: BufHandle, old_flags: c_int);

    /// Cursor adjustment for u_undoredo
    fn nvim_undoredo_adjust_cursor(curhead: UHeaderHandle);

    /// Get ml_get result as non-allocating pointer for strcmp
    #[link_name = "ml_get"]
    fn nvim_undoredo_ml_get(lnum: LinenrT) -> *const c_char;

    /// buf_updates_changedtick wrapper
    #[link_name = "buf_updates_changedtick"]
    fn nvim_undoredo_buf_updates_changedtick(buf: BufHandle);

    /// E438 error message wrapper
    fn nvim_iemsg_undo_line_numbers_wrong();

    /// xmalloc wrapper
    #[link_name = "xmalloc"]
    fn nvim_xmalloc(size: usize) -> *mut c_void;

    // ==========================================================================
    // Phase 4: u_undo_end + helpers FFI
    // ==========================================================================

    /// Redraw conceal for all windows showing buffer
    fn nvim_undo_end_redraw_conceal(buf: BufHandle);

    /// Check VIsual and call check_pos
    fn nvim_undo_end_check_visual(buf: BufHandle);

    /// Format and display the undo end message
    fn nvim_undo_end_smsg(
        count: i64,
        msgstr: *const c_char,
        did_undo: bool,
        seq: i64,
        timebuf: *const c_char,
    );

    // ==========================================================================
    // Phase 5: u_get_undo_file_name FFI helpers
    // ==========================================================================

    /// Resolve symlink, returns allocated copy
    fn nvim_undo_resolve_symlink(ffname: *const c_char) -> *mut c_char;

    /// Get p_udir option value
    fn nvim_undo_get_p_udir() -> *const c_char;

    /// copy_option_part wrapper
    fn nvim_undo_copy_option_part(
        dirp: *mut *const c_char,
        buf: *mut c_char,
        maxlen: usize,
    ) -> usize;

    /// Check if path is a directory
    #[link_name = "os_isdir"]
    fn nvim_undo_os_isdir(path: *const c_char) -> bool;

    /// Create directory recursively
    fn nvim_undo_mkdir_recurse(dir: *const c_char, failed_dir: *mut *mut c_char) -> c_int;

    /// E5003 error message
    fn nvim_undo_semsg_mkdir(failed_dir: *const c_char, err: c_int);

    /// path_tail offset
    fn nvim_undo_path_tail_offset(path: *const c_char) -> usize;

    /// vim_ispathsep check
    #[link_name = "vim_ispathsep"]
    fn nvim_undo_vim_ispathsep(c: c_int) -> bool;

    /// Multibyte pointer char length
    #[link_name = "utfc_ptr2len"]
    fn nvim_undo_mb_ptr_len(ptr: *const c_char) -> c_int;

    /// concat_fnames wrapper
    fn nvim_undo_concat_fnames(dir: *const c_char, fname: *const c_char) -> *mut c_char;

    /// Get MAXPATHL value
    fn nvim_undo_get_maxpathl() -> usize;
}

/// Check if the 'modified' flag is set, or 'ff' has changed.
/// "nofile" and "scratch" type buffers are considered to always be unchanged.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "bufIsChanged"]
pub unsafe extern "C" fn rs_buf_is_changed(buf: BufHandle) -> bool {
    // In a "prompt" buffer we do respect 'modified', so that we can control
    // closing the window by setting or resetting that option.
    (!nvim_bt_dontwrite(buf) || nvim_bt_prompt(buf))
        && (nvim_buf_get_b_changed(buf) || nvim_file_ff_differs(buf, true))
}

/// Return true if any buffer has changes. Also buffers that are not written.
///
/// # Safety
///
/// Accesses global buffer list via C FFI.
#[export_name = "anyBufIsChanged"]
pub unsafe extern "C" fn rs_any_buf_is_changed() -> bool {
    let mut buf = nvim_get_firstbuf();
    while !buf.0.is_null() {
        if rs_buf_is_changed(buf) {
            return true;
        }
        buf = nvim_buf_get_next(buf);
    }
    false
}

/// Return true if the current buffer has changed.
///
/// # Safety
///
/// Accesses global curbuf via C FFI.
#[export_name = "curbufIsChanged"]
pub unsafe extern "C" fn rs_curbuf_is_changed() -> bool {
    rs_buf_is_changed(nvim_get_curbuf())
}

/// Invalidate the undo buffer; called when storage has already been released.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "u_clearall"]
pub unsafe extern "C" fn rs_u_clearall(buf: BufHandle) {
    nvim_buf_set_b_u_newhead(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_oldhead(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_curhead(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_synced(buf, true);
    nvim_buf_set_b_u_numhead(buf, 0);
    nvim_buf_set_b_u_line_ptr(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_line_lnum(buf, 0);
}

/// Clear the line saved for the "U" command.
/// (this is used externally for crossing a line while in insert mode)
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "u_clearline"]
pub unsafe extern "C" fn rs_u_clearline(buf: BufHandle) {
    let line_ptr = nvim_buf_get_b_u_line_ptr(buf);
    if line_ptr.is_null() {
        return;
    }

    nvim_xfree(line_ptr as *mut c_void);
    nvim_buf_set_b_u_line_ptr(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_line_lnum(buf, 0);
}

/// Save a line for the "U" command.
/// This replaces the C `u_saveline` function.
///
/// # Safety
///
/// Must be called with valid buffer handle and line number.
unsafe fn u_saveline(buf: BufHandle, lnum: LinenrT) {
    if lnum == nvim_buf_get_b_u_line_lnum(buf) {
        // line is already saved
        return;
    }
    let line_count = nvim_buf_get_ml_line_count(buf);
    if lnum < 1 || lnum > line_count {
        // should never happen
        return;
    }
    rs_u_clearline(buf);
    nvim_buf_set_b_u_line_lnum(buf, lnum);
    let win = nvim_undo_get_curwin();
    let win_buf = nvim_undo_win_get_buffer(win);
    if win_buf.0 == buf.0 && nvim_undo_win_get_cursor_lnum(win) == lnum {
        nvim_buf_set_b_u_line_colnr(buf, nvim_undo_curwin_get_cursor_col());
    } else {
        nvim_buf_set_b_u_line_colnr(buf, 0);
    }
    nvim_buf_set_b_u_line_ptr(buf, nvim_ml_get_buf_copy(buf, lnum));
}

/// Free entry 'uep' and 'n' lines in uep->ue_array[].
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freeentry(uep: UEntryHandle, mut n: c_int) {
    // Free array elements from n-1 down to 0
    while n > 0 {
        n -= 1;
        let elem = *(*uep).ue_array.offset(n as isize);
        nvim_xfree(elem as *mut c_void);
    }

    // Free the array itself
    nvim_xfree((*uep).ue_array as *mut c_void);

    // Free the entry struct
    nvim_xfree(uep as *mut c_void);
}

/// Free all the undo entries for one header and the header itself.
/// This means that "uhp" is invalid when returning.
///
/// # Safety
///
/// All handles must be valid pointers. uhpp may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freeentries(
    buf: BufHandle,
    uhp: UHeaderHandle,
    uhpp: *mut UHeaderHandle,
) {
    // Check for pointers to the header that become invalid now.
    let curhead = nvim_buf_get_b_u_curhead(buf);
    if curhead == uhp {
        nvim_buf_set_b_u_curhead(buf, std::ptr::null_mut());
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead == uhp {
        nvim_buf_set_b_u_newhead(buf, std::ptr::null_mut());
    }

    if !uhpp.is_null() && *uhpp == uhp {
        *uhpp = std::ptr::null_mut();
    }

    // Free all entries in the list
    let mut uep = (*uhp).uh_entry;
    while !uep.is_null() {
        let nuep = (*uep).ue_next;
        let size = (*uep).ue_size;
        rs_u_freeentry(uep, size as c_int);
        uep = nuep;
    }

    // Destroy the extmark vector
    nvim_xfree((*uhp).uh_extmark.items as *mut c_void);
    (*uhp).uh_extmark.items = std::ptr::null_mut();
    (*uhp).uh_extmark.size = 0;
    (*uhp).uh_extmark.capacity = 0;

    // Free the header struct
    nvim_xfree(uhp as *mut c_void);

    // Decrement header count
    let numhead = nvim_buf_get_b_u_numhead(buf);
    nvim_buf_set_b_u_numhead(buf, numhead - 1);
}

/// Free one header "uhp" and its entry list and adjust the pointers.
///
/// # Safety
///
/// All handles must be valid pointers. uhpp may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freeheader(
    buf: BufHandle,
    uhp: UHeaderHandle,
    uhpp: *mut UHeaderHandle,
) {
    // When there is an alternate redo list free that branch completely,
    // because we can never go there.
    let alt_next = (*uhp).uh_alt_next.ptr;
    if !alt_next.is_null() {
        rs_u_freebranch(buf, alt_next, uhpp);
    }

    let alt_prev = (*uhp).uh_alt_prev.ptr;
    if !alt_prev.is_null() {
        (*alt_prev).uh_alt_next.ptr = std::ptr::null_mut();
    }

    // Update the links in the list to remove the header.
    let uh_next = (*uhp).uh_next.ptr;
    let uh_prev = (*uhp).uh_prev.ptr;

    if uh_next.is_null() {
        nvim_buf_set_b_u_oldhead(buf, uh_prev);
    } else {
        (*uh_next).uh_prev.ptr = uh_prev;
    }

    if uh_prev.is_null() {
        nvim_buf_set_b_u_newhead(buf, uh_next);
    } else {
        let mut uhap = uh_prev;
        while !uhap.is_null() {
            (*uhap).uh_next.ptr = uh_next;
            uhap = (*uhap).uh_alt_next.ptr;
        }
    }

    rs_u_freeentries(buf, uhp, uhpp);
}

/// Free an alternate branch and any following alternate branches.
///
/// # Safety
///
/// All handles must be valid pointers. uhpp may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freebranch(
    buf: BufHandle,
    uhp: UHeaderHandle,
    uhpp: *mut UHeaderHandle,
) {
    // If this is the top branch we may need to use u_freeheader() to update
    // all the pointers.
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    if uhp == oldhead {
        loop {
            let current_oldhead = nvim_buf_get_b_u_oldhead(buf);
            if current_oldhead.is_null() {
                break;
            }
            rs_u_freeheader(buf, current_oldhead, uhpp);
        }
        return;
    }

    let alt_prev = (*uhp).uh_alt_prev.ptr;
    if !alt_prev.is_null() {
        (*alt_prev).uh_alt_next.ptr = std::ptr::null_mut();
    }

    let mut next = uhp;
    while !next.is_null() {
        let tofree = next;
        let alt_next = (*tofree).uh_alt_next.ptr;
        if !alt_next.is_null() {
            rs_u_freebranch(buf, alt_next, uhpp); // recursive
        }
        next = (*tofree).uh_prev.ptr;
        rs_u_freeentries(buf, tofree, uhpp);
    }
}

/// Get the first entry in the undo header for the buffer.
/// Returns NULL if the undo list is corrupt.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_get_headentry(buf: BufHandle) -> UEntryHandle {
    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.is_null() {
        nvim_iemsg_undo_list_corrupt();
        return ptr::null_mut();
    }

    let entry = (*newhead).uh_entry;
    if entry.is_null() {
        nvim_iemsg_undo_list_corrupt();
        return ptr::null_mut();
    }

    entry
}

/// Compute the line number of the previous u_save.
/// It is called only when b_u_synced is false.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_getbot(buf: BufHandle) {
    // Check for corrupt undo list
    let check = rs_u_get_headentry(buf);
    if check.is_null() {
        return;
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let uep = (*newhead).uh_getbot_entry;
    if !uep.is_null() {
        // The new ue_bot is computed from the number of lines that has been
        // inserted (0 - deleted) since calling u_save. This is equal to the
        // old line count subtracted from the current line count.
        let ml_line_count = nvim_buf_get_ml_line_count(buf);
        let ue_lcount = (*uep).ue_lcount;
        let extra = ml_line_count - ue_lcount;

        let ue_top = (*uep).ue_top;
        let ue_size = (*uep).ue_size;
        let mut ue_bot = ue_top + ue_size + 1 + extra;

        if ue_bot < 1 || ue_bot > ml_line_count {
            nvim_iemsg_undo_line_missing();
            // Assume all lines deleted, will get all the old lines back
            // without deleting the current ones
            ue_bot = ue_top + 1;
        }

        (*uep).ue_bot = ue_bot;
        (*newhead).uh_getbot_entry = ptr::null_mut();
    }

    nvim_buf_set_b_u_synced(buf, true);
}

/// Free all undo headers and entries for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "u_blockfree"]
pub unsafe extern "C" fn rs_u_blockfree(buf: BufHandle) {
    loop {
        let oldhead = nvim_buf_get_b_u_oldhead(buf);
        if oldhead.is_null() {
            break;
        }
        rs_u_freeheader(buf, oldhead, std::ptr::null_mut());
    }

    // Free the line saved for "U" command
    let line_ptr = nvim_buf_get_b_u_line_ptr(buf);
    nvim_xfree(line_ptr as *mut c_void);
}

/// Stop adding to the current entry list.
///
/// # Safety
///
/// Accesses global curbuf via C FFI.
#[export_name = "u_sync"]
pub unsafe extern "C" fn rs_u_sync(force: bool) {
    let buf = nvim_get_curbuf();

    // Skip it when already synced or syncing is disabled.
    if nvim_buf_get_b_u_synced(buf) {
        return;
    }
    if !force && nvim_get_no_u_sync() > 0 {
        return;
    }

    if nvim_get_undolevel(buf) < 0 {
        // No entries, nothing to do
        nvim_buf_set_b_u_synced(buf, true);
    } else {
        // Compute ue_bot of previous u_save
        rs_u_getbot(buf);
        nvim_buf_set_b_u_curhead(buf, std::ptr::null_mut());
    }
}

/// Free all allocated memory blocks for the buffer and invalidate the undo buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "u_clearallandblockfree"]
pub unsafe extern "C" fn rs_u_clearallandblockfree(buf: BufHandle) {
    rs_u_blockfree(buf);
    rs_u_clearall(buf);
}

/// Mark all headers in the branch as changed (recursive).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T (or NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_u_unch_branch(uhp: UHeaderHandle) {
    let mut uh = uhp;
    while !uh.is_null() {
        // Set UH_CHANGED flag
        let flags = (*uh).uh_flags;
        (*uh).uh_flags = flags | UH_CHANGED;

        // Recurse into alternate branch if present
        let alt_next = (*uh).uh_alt_next.ptr;
        if !alt_next.is_null() {
            rs_u_unch_branch(alt_next);
        }

        // Move to previous header
        uh = (*uh).uh_prev.ptr;
    }
}

/// Called after writing or reloading the file and setting b_changed to false.
/// Now an undo means that the buffer is modified.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "u_unchanged"]
pub unsafe extern "C" fn rs_u_unchanged(buf: BufHandle) {
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    rs_u_unch_branch(oldhead);
    nvim_buf_set_b_did_warn(buf, false);
}

/// Increase the write count, store it in the last undo header.
/// This is what would be used for "u".
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "u_update_save_nr"]
pub unsafe extern "C" fn rs_u_update_save_nr(buf: BufHandle) {
    let save_nr_last = nvim_buf_get_b_u_save_nr_last(buf) + 1;
    nvim_buf_set_b_u_save_nr_last(buf, save_nr_last);
    nvim_buf_set_b_u_save_nr_cur(buf, save_nr_last);

    let curhead = nvim_buf_get_b_u_curhead(buf);
    let uhp = if !curhead.is_null() {
        (*curhead).uh_next.ptr
    } else {
        nvim_buf_get_b_u_newhead(buf)
    };

    if !uhp.is_null() {
        (*uhp).uh_save_nr = save_nr_last;
    }
}

/// Free a u_header_T and all its entries.
/// Used when reading an undo file fails.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_free_uhp(uhp: UHeaderHandle) {
    let mut uep = (*uhp).uh_entry;
    while !uep.is_null() {
        let nuep = (*uep).ue_next;
        let size = (*uep).ue_size;
        rs_u_freeentry(uep, size as c_int);
        uep = nuep;
    }
    nvim_xfree(uhp as *mut c_void);
}

/// Helper function to check if expression mapping is locked.
///
/// # Safety
///
/// Calls external C functions.
#[inline]
unsafe fn expr_map_locked() -> bool {
    let lock = nvim_get_expr_map_lock();
    let is_dummy = nvim_curbuf_is_dummy();
    lock > 0 && is_dummy == 0
}

/// Return true when undo is allowed. Otherwise print an error message and
/// return false.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[export_name = "undo_allowed"]
pub unsafe extern "C" fn rs_undo_allowed(buf: BufHandle) -> bool {
    // Don't allow changes when 'modifiable' is off.
    if !nvim_buf_is_modifiable(buf) {
        nvim_emsg_modifiable();
        return false;
    }

    // In the sandbox it's not allowed to change the text.
    if nvim_get_sandbox() != 0 {
        nvim_emsg_sandbox();
        return false;
    }

    // Don't allow changes in the buffer while editing the cmdline.
    // The caller of getcmdline() may get confused.
    if nvim_get_textlock() != 0 || expr_map_locked() {
        nvim_emsg_textlock();
        return false;
    }

    true
}

/// ":undojoin": continue adding to the last entry list
///
/// # Safety
///
/// Accesses global curbuf via C FFI.
#[export_name = "ex_undojoin"]
pub unsafe extern "C" fn rs_ex_undojoin(_eap: ExargHandle) {
    let buf = nvim_get_curbuf();

    // Nothing changed before
    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.is_null() {
        return;
    }

    // Not allowed after undo
    let curhead = nvim_buf_get_b_u_curhead(buf);
    if !curhead.is_null() {
        nvim_emsg_undojoin_after_undo();
        return;
    }

    // Already unsynced
    if !nvim_buf_get_b_u_synced(buf) {
        return;
    }

    // No entries, nothing to do
    if nvim_get_undolevel(buf) < 0 {
        return;
    }

    // Append next change to last entry
    nvim_buf_set_b_u_synced(buf, false);
}

/// If 'cpoptions' contains 'u': Undo the previous undo or redo (vi compatible).
/// If 'cpoptions' does not contain 'u': Always undo.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[export_name = "u_undo"]
pub unsafe extern "C" fn rs_u_undo(mut count: c_int) {
    let buf = nvim_get_curbuf();

    // If we get an undo command while executing a macro, we behave like the
    // original vi. If this happens twice in one macro the result will not
    // be compatible.
    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
        count = 1;
    }

    if !nvim_has_cpo_undo() {
        nvim_set_undo_undoes(true);
    } else {
        nvim_set_undo_undoes(!nvim_get_undo_undoes());
    }

    rs_u_doit(count, false, true);
}

/// If 'cpoptions' contains 'u': Repeat the previous undo or redo.
/// If 'cpoptions' does not contain 'u': Always redo.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[export_name = "u_redo"]
pub unsafe extern "C" fn rs_u_redo(count: c_int) {
    if !nvim_has_cpo_undo() {
        nvim_set_undo_undoes(false);
    }

    rs_u_doit(count, false, true);
}

/// Undo and remove the branch from the undo tree.
/// Also moves the cursor (as a "normal" undo would).
///
/// # Safety
///
/// Accesses global state via C FFI.
#[export_name = "u_undo_and_forget"]
pub unsafe extern "C" fn rs_u_undo_and_forget(mut count: c_int, do_buf_event: bool) -> bool {
    let buf = nvim_get_curbuf();

    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
        count = 1;
    }

    nvim_set_undo_undoes(true);
    rs_u_doit(count, true, do_buf_event);

    let curhead = nvim_buf_get_b_u_curhead(buf);
    if curhead.is_null() {
        // nothing was undone
        return false;
    }

    // Delete the current redo header
    // set the redo header to the next alternative branch (if any)
    // otherwise we will be in the leaf state
    let to_forget = curhead;
    let uh_next = (*to_forget).uh_next.ptr;
    nvim_buf_set_b_u_newhead(buf, uh_next);

    let alt_next = (*to_forget).uh_alt_next.ptr;
    nvim_buf_set_b_u_curhead(buf, alt_next);

    if !alt_next.is_null() {
        (*to_forget).uh_alt_next.ptr = std::ptr::null_mut();
        let alt_prev = (*to_forget).uh_alt_prev.ptr;
        (*alt_next).uh_alt_prev.ptr = alt_prev;

        let alt_next_next = (*alt_next).uh_next.ptr;
        if !alt_next_next.is_null() {
            nvim_buf_set_b_u_seq_cur(buf, (*alt_next_next).uh_seq);
        } else {
            nvim_buf_set_b_u_seq_cur(buf, 0);
        }
    } else {
        let newhead = nvim_buf_get_b_u_newhead(buf);
        if !newhead.is_null() {
            nvim_buf_set_b_u_seq_cur(buf, (*newhead).uh_seq);
        }
    }

    let alt_prev = (*to_forget).uh_alt_prev.ptr;
    if !alt_prev.is_null() {
        let new_curhead = nvim_buf_get_b_u_curhead(buf);
        (*alt_prev).uh_alt_next.ptr = new_curhead;
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    if !newhead.is_null() {
        let new_curhead = nvim_buf_get_b_u_curhead(buf);
        (*newhead).uh_prev.ptr = new_curhead;
    }

    let seq_last = nvim_buf_get_b_u_seq_last(buf);
    let to_forget_seq = (*to_forget).uh_seq;
    if seq_last == to_forget_seq {
        nvim_buf_set_b_u_seq_last(buf, seq_last - 1);
    }

    rs_u_freebranch(buf, to_forget, std::ptr::null_mut());
    true
}

/// Core undo/redo function.
/// The lines in the file are replaced by the lines in the entry list at
/// curbuf->b_u_curhead. The replaced lines in the file are saved in the entry
/// list for the next undo/redo.
///
/// # Safety
///
/// Must be called with valid global state (curbuf, curhead set correctly).
unsafe fn u_undoredo(undo: bool, do_buf_event: bool) {
    let buf = nvim_get_curbuf();
    let curhead = nvim_buf_get_b_u_curhead(buf);
    let win = nvim_undo_get_curwin();

    let mut newlnum: LinenrT = MAXLNUM;
    let mut new_curpos_lnum = nvim_undo_win_get_cursor_lnum(win);

    // Don't want autocommands using the undo structures here, they are
    // invalid till the end.
    nvim_block_autocmds();

    let old_flags = (*curhead).uh_flags;
    // Inline nvim_undoredo_compute_new_flags
    let new_flags: c_int = (if nvim_buf_get_b_changed(buf) {
        UH_CHANGED
    } else {
        0
    }) | (if nvim_buf_ml_is_empty(buf) {
        UH_EMPTYBUF
    } else {
        0
    }) | ((*curhead).uh_flags & UH_RELOAD);
    nvim_undo_setpcmark();

    // Save marks before undo/redo
    nvim_undoredo_save_marks(buf, curhead);
    // Allocate buffer for saved namedm + visualinfo
    const SAVED_MARKS_VISUAL_OFFSET: usize = std::mem::size_of::<FmarkT>() * NMARKS;
    let saved_marks = nvim_xmalloc(SAVED_MARKS_VISUAL_OFFSET + std::mem::size_of::<VisualInfoT>());
    // NMARKS fmark_T entries followed by one visualinfo_T
    nvim_undoredo_get_buf_marks(buf, saved_marks, saved_marks.add(SAVED_MARKS_VISUAL_OFFSET));

    nvim_undoredo_init_op_marks(buf);

    let mut newlist: *mut UEntry = ptr::null_mut();
    let mut uep = (*curhead).uh_entry;

    while !uep.is_null() {
        let top = (*uep).ue_top;
        let mut bot = (*uep).ue_bot;
        let line_count = nvim_buf_get_ml_line_count(buf);

        if bot == 0 {
            bot = line_count + 1;
        }

        if top > line_count || top >= bot || bot > line_count + 1 {
            nvim_unblock_autocmds();
            nvim_iemsg_undo_line_numbers_wrong();
            nvim_buf_changed(buf); // don't want UNCHANGED now
            nvim_xfree(saved_marks);
            return;
        }

        let oldsize = bot - top - 1; // number of lines before undo
        let newsize = (*uep).ue_size; // number of lines after undo

        // Decide about the cursor position, depending on what text changed.
        if top < newlnum {
            let cursor_lnum = (*curhead).uh_cursor.lnum;
            if cursor_lnum >= top && cursor_lnum <= top + newsize + 1 {
                new_curpos_lnum = cursor_lnum;
                newlnum = new_curpos_lnum - 1;
            } else {
                // Use the first line that actually changed.
                let mut i: LinenrT = 0;
                while i < newsize && i < oldsize {
                    let array_line = *(*uep).ue_array.offset(i as isize);
                    let buf_line = nvim_undoredo_ml_get(top + 1 + i);
                    if libc::strcmp(array_line, buf_line) != 0 {
                        break;
                    }
                    i += 1;
                }
                let next_uep = (*uep).ue_next;
                if i == newsize && newlnum == MAXLNUM && next_uep.is_null() {
                    newlnum = top;
                    new_curpos_lnum = newlnum + 1;
                } else if i < newsize {
                    newlnum = top + i;
                    new_curpos_lnum = newlnum + 1;
                }
            }
        }

        let mut empty_buffer = false;

        // Delete the lines between top and bot and save them in newarray.
        let newarray: *mut *mut c_char;
        if oldsize > 0 {
            newarray = nvim_xmalloc(std::mem::size_of::<*mut c_char>() * oldsize as usize)
                as *mut *mut c_char;
            // delete backwards, it goes faster in most cases
            let mut i = oldsize;
            let mut lnum = bot - 1;
            while {
                i -= 1;
                i >= 0
            } {
                *newarray.offset(i as isize) = nvim_u_save_line_for_undo(buf, lnum);
                if nvim_buf_get_ml_line_count(buf) == 1 {
                    empty_buffer = true;
                }
                nvim_ml_delete_lnum(lnum);
                lnum -= 1;
            }
        } else {
            newarray = ptr::null_mut();
        }

        // make sure the cursor is on a valid line after the deletions
        nvim_undo_check_cursor_lnum(win);

        // Insert the lines in u_array between top and bot.
        if newsize > 0 {
            let mut i: LinenrT = 0;
            let mut lnum = top;
            while i < newsize {
                let line = *(*uep).ue_array.offset(i as isize);
                if empty_buffer && lnum == 0 {
                    nvim_ml_replace_lnum(1, line, true);
                } else {
                    nvim_ml_append_flags(lnum, line, 0, 0);
                }
                nvim_xfree(*(*uep).ue_array.offset(i as isize) as *mut c_void);
                i += 1;
                lnum += 1;
            }
            nvim_xfree((*uep).ue_array as *mut c_void);
        }

        // Adjust marks
        if oldsize != newsize {
            // kExtmarkNOOP = 0
            nvim_undo_mark_adjust(top + 1, top + oldsize, MAXLNUM, newsize - oldsize);
            let op_start = nvim_buf_get_op_start_lnum(buf);
            if op_start > top + oldsize {
                nvim_buf_adjust_op_start_lnum(buf, newsize - oldsize);
            }
            let op_end = nvim_buf_get_op_end_lnum(buf);
            if op_end > top + oldsize {
                nvim_buf_adjust_op_end_lnum(buf, newsize - oldsize);
            }
        }

        if oldsize > 0 || newsize > 0 {
            nvim_undo_changed_lines(buf, top + 1, 0, bot, newsize - oldsize, do_buf_event);
            let line_count = nvim_buf_get_ml_line_count(buf);
            if nvim_spell_check_window(win) && bot <= line_count {
                nvim_redrawWinline(win, bot);
            }
        }

        // Set the '[ mark.
        let op_start = nvim_buf_get_op_start_lnum(buf);
        if top + 1 < op_start {
            nvim_buf_set_op_start_lnum(buf, top + 1);
        }
        // Set the '] mark.
        let op_end = nvim_buf_get_op_end_lnum(buf);
        if newsize == 0 && top + 1 > op_end {
            nvim_buf_set_op_end_lnum(buf, top + 1);
        } else if top + newsize > op_end {
            nvim_buf_set_op_end_lnum(buf, top + newsize);
        }

        nvim_set_u_newcount(nvim_get_u_newcount() + newsize as c_int);
        nvim_set_u_oldcount(nvim_get_u_oldcount() + oldsize as c_int);
        (*uep).ue_size = oldsize;
        (*uep).ue_array = newarray;
        (*uep).ue_bot = top + newsize + 1;

        // insert this entry in front of the new entry list
        let nuep = (*uep).ue_next;
        (*uep).ue_next = newlist;
        newlist = uep;
        uep = nuep;
    }

    // Ensure the '[ and '] marks are within bounds.
    nvim_undoredo_clamp_op_marks(buf);

    // Adjust Extmarks (inlined from nvim_undoredo_apply_extmarks)
    {
        let extmark_size = (*curhead).uh_extmark.size;
        if undo {
            let mut i = extmark_size as isize - 1;
            while i >= 0 {
                nvim_extmark_apply_undo(curhead, i as usize, undo);
                i -= 1;
            }
        } else {
            for i in 0..extmark_size {
                nvim_extmark_apply_undo(curhead, i, undo);
            }
        }
        if (*curhead).uh_flags & UH_RELOAD != 0 {
            nvim_buf_updates_unload(nvim_get_curbuf(), true);
        }
    }

    // Set the cursor to the desired position. Check that the line is valid.
    nvim_undo_win_set_cursor_pos(win, new_curpos_lnum, 0, 0);
    nvim_undo_check_cursor_lnum(win);

    (*curhead).uh_entry = newlist;
    (*curhead).uh_flags = new_flags;
    nvim_undoredo_set_ml_empty(buf, old_flags);

    if old_flags & UH_CHANGED != 0 {
        nvim_buf_changed(buf);
    } else {
        nvim_buf_unchanged(buf, false, true);
    }

    // because the calls to changed()/unchanged() above will bump changedtick
    // again, we need to send a nvim_buf_lines_event with just the new value of
    // b:changedtick
    if do_buf_event {
        nvim_undoredo_buf_updates_changedtick(buf);
    }

    // restore marks from before undo/redo
    nvim_undoredo_restore_marks(buf, curhead, saved_marks);
    nvim_undoredo_swap_visual(buf, curhead, saved_marks.add(SAVED_MARKS_VISUAL_OFFSET));
    nvim_xfree(saved_marks);

    // Adjust cursor position
    nvim_undoredo_adjust_cursor(curhead);

    // Remember where we are for "g-" and ":earlier 10s".
    // Inline nvim_undoredo_update_seq: update b_u_seq_cur, b_u_save_nr_cur, b_u_time_cur
    {
        let seq = (*curhead).uh_seq;
        let next_seq = if !(*curhead).uh_next.ptr.is_null() {
            (*(*curhead).uh_next.ptr).uh_seq
        } else {
            0
        };
        nvim_buf_set_b_u_seq_cur(buf, if undo { next_seq } else { seq });
        if (*curhead).uh_save_nr != 0 {
            nvim_buf_set_b_u_save_nr_cur(
                buf,
                if undo {
                    (*curhead).uh_save_nr - 1
                } else {
                    (*curhead).uh_save_nr
                },
            );
        }
        nvim_buf_set_b_u_time_cur(buf, (*curhead).uh_time);
    }

    nvim_unblock_autocmds();
}

/// Report the result of an undo/redo operation.
/// If we deleted or added lines, report the number of less/more lines.
/// Otherwise, report the number of changes.
unsafe fn u_undo_end(did_undo: bool, absolute: bool, quiet: bool) {
    let buf = nvim_get_curbuf();

    if (nvim_undo_get_fdo_flags() & K_OPT_FDO_FLAG_UNDO) != 0 && nvim_undo_get_key_typed() {
        rs_foldOpenCursor();
    }

    if quiet || nvim_get_global_busy() || !nvim_messaging() {
        return;
    }

    let mut u_newcount = nvim_get_u_newcount();
    let mut u_oldcount = nvim_get_u_oldcount();

    if nvim_buf_ml_is_empty(buf) {
        u_newcount -= 1;
    }

    u_oldcount -= u_newcount;
    let msgstr: &[u8] = if u_oldcount == -1 {
        b"more line\0"
    } else if u_oldcount < 0 {
        b"more lines\0"
    } else if u_oldcount == 1 {
        b"line less\0"
    } else if u_oldcount > 1 {
        b"fewer lines\0"
    } else {
        u_oldcount = u_newcount;
        if u_newcount == 1 {
            b"change\0"
        } else {
            b"changes\0"
        }
    };

    // Inline nvim_undo_end_get_uhp_seq: find the relevant undo header
    let mut adjusted_did_undo = did_undo;
    let uhp_for_seq: *mut UHeader = {
        let curhead = nvim_buf_get_b_u_curhead(buf);
        if !curhead.is_null() {
            if absolute && !(*curhead).uh_next.ptr.is_null() {
                adjusted_did_undo = false;
                (*curhead).uh_next.ptr
            } else if did_undo {
                curhead
            } else {
                (*curhead).uh_next.ptr
            }
        } else {
            nvim_buf_get_b_u_newhead(buf)
        }
    };
    let seq: c_int = if uhp_for_seq.is_null() {
        0
    } else {
        (*uhp_for_seq).uh_seq
    };

    // Inline nvim_undo_end_fmt_time: format time from the undo header
    let mut timebuf = [0u8; 80];
    {
        let curhead = nvim_buf_get_b_u_curhead(buf);
        let uhp_for_time: *mut UHeader = if !curhead.is_null() {
            if absolute && !(*curhead).uh_next.ptr.is_null() {
                (*curhead).uh_next.ptr
            } else if did_undo {
                curhead
            } else {
                (*curhead).uh_next.ptr
            }
        } else {
            nvim_buf_get_b_u_newhead(buf)
        };
        if uhp_for_time.is_null() {
            timebuf[0] = 0;
        } else {
            rs_undo_fmt_time(
                timebuf.as_mut_ptr() as *mut c_char,
                timebuf.len(),
                (*uhp_for_time).uh_time,
            );
        }
    }

    nvim_undo_end_redraw_conceal(buf);
    nvim_undo_end_check_visual(buf);

    let count = if u_oldcount < 0 {
        -u_oldcount as i64
    } else {
        u_oldcount as i64
    };

    nvim_undo_end_smsg(
        count,
        msgstr.as_ptr() as *const c_char,
        adjusted_did_undo,
        seq as i64,
        timebuf.as_ptr() as *const c_char,
    );
}

/// Undo fold flag constant. Verified by _Static_assert in undo.c.
const K_OPT_FDO_FLAG_UNDO: c_int = 0x200;

/// Core undo/redo loop.
/// Performs the actual undo or redo operations based on the current state.
///
/// # Safety
///
/// Must be called with valid global state (curbuf, undo_undoes set correctly).
#[no_mangle]
pub unsafe extern "C" fn rs_u_doit(startcount: c_int, quiet: bool, do_buf_event: bool) {
    let buf = nvim_get_curbuf();

    if !rs_undo_allowed(buf) {
        return;
    }

    nvim_set_u_newcount(0);
    nvim_set_u_oldcount(if nvim_buf_ml_is_empty(buf) { -1 } else { 0 });

    nvim_msg_ext_set_kind_undo();
    let mut count = startcount;

    while count > 0 {
        count -= 1;

        // Do the change warning now, so that it triggers FileChangedRO when
        // needed. This may cause the file to be reloaded, that must happen
        // before we do anything, because it may change curbuf->b_u_curhead
        // and more.
        nvim_change_warning_curbuf();

        let undo_undoes = nvim_get_undo_undoes();

        if undo_undoes {
            let curhead = nvim_buf_get_b_u_curhead(buf);
            if curhead.is_null() {
                // first undo
                let newhead = nvim_buf_get_b_u_newhead(buf);
                nvim_buf_set_b_u_curhead(buf, newhead);
            } else if nvim_get_undolevel(buf) > 0 {
                // multi level undo - get next undo
                let next = (*curhead).uh_next.ptr;
                nvim_buf_set_b_u_curhead(buf, next);
            }

            // nothing to undo
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let numhead = nvim_buf_get_b_u_numhead(buf);
            if numhead == 0 || curhead.is_null() {
                // stick curbuf->b_u_curhead at end
                let oldhead = nvim_buf_get_b_u_oldhead(buf);
                nvim_buf_set_b_u_curhead(buf, oldhead);
                nvim_beep_flush();
                if count == startcount - 1 {
                    nvim_msg_oldest_change();
                    return;
                }
                break;
            }

            u_undoredo(true, do_buf_event);
        } else {
            let curhead = nvim_buf_get_b_u_curhead(buf);
            if curhead.is_null() || nvim_get_undolevel(buf) <= 0 {
                // nothing to redo
                nvim_beep_flush();
                if count == startcount - 1 {
                    nvim_msg_newest_change();
                    return;
                }
                break;
            }

            u_undoredo(false, do_buf_event);

            // Advance for next redo. Set "newhead" when at the end of the
            // redoable changes.
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let prev = (*curhead).uh_prev.ptr;
            if prev.is_null() {
                nvim_buf_set_b_u_newhead(buf, curhead);
            }
            nvim_buf_set_b_u_curhead(buf, prev);
        }
    }

    let undo_undoes = nvim_get_undo_undoes();
    u_undo_end(undo_undoes, false, quiet);
}

/// Common code for various ways to save text before a change.
/// "top" is the line above the first changed line.
/// "bot" is the line below the last changed line.
/// "newbot" is the new bottom line. Use zero when not known.
/// "reload" is true when saving for a buffer reload.
///
/// # Safety
///
/// Must be called with valid buffer handle and line numbers.
#[export_name = "u_savecommon"]
pub unsafe extern "C" fn rs_u_savecommon(
    buf: BufHandle,
    top: LinenrT,
    bot: LinenrT,
    newbot: LinenrT,
    reload: bool,
) -> c_int {
    if !reload {
        // When making changes is not allowed return FAIL
        if !rs_undo_allowed(buf) {
            return FAIL;
        }

        // Saving text for undo means we are going to make a change
        if nvim_buf_is_curbuf(buf) {
            nvim_change_warning_curbuf();
        }

        let line_count = nvim_buf_get_ml_line_count(buf);
        if bot > line_count + 1 {
            nvim_emsg_line_count_changed();
            return FAIL;
        }
    }

    let size = bot - top - 1;

    // If curbuf->b_u_synced == true make a new header
    if nvim_buf_get_b_u_synced(buf) {
        // Need to create new entry in b_changelist
        nvim_buf_set_b_new_change(buf, true);

        let uhp: UHeaderHandle;
        if nvim_get_undolevel(buf) >= 0 {
            // Make a new header entry
            uhp = nvim_xcalloc(1, std::mem::size_of::<UHeader>()) as *mut UHeader;
            (*uhp).uh_extmark.items = std::ptr::null_mut();
            (*uhp).uh_extmark.size = 0;
            (*uhp).uh_extmark.capacity = 0;
        } else {
            uhp = std::ptr::null_mut();
        }

        // If we undid more than we redid, move the entry lists before and
        // including curbuf->b_u_curhead to an alternate branch
        let mut old_curhead = nvim_buf_get_b_u_curhead(buf);
        if !old_curhead.is_null() {
            let next = (*old_curhead).uh_next.ptr;
            nvim_buf_set_b_u_newhead(buf, next);
            nvim_buf_set_b_u_curhead(buf, std::ptr::null_mut());
        }

        // Free headers to keep the size right
        while nvim_buf_get_b_u_numhead(buf) as i64 > nvim_get_undolevel(buf) {
            let oldhead = nvim_buf_get_b_u_oldhead(buf);
            if oldhead.is_null() {
                break;
            }

            if oldhead == old_curhead {
                // Can't reconnect the branch, delete all of it
                rs_u_freebranch(buf, oldhead, &mut old_curhead as *mut UHeaderHandle);
            } else {
                let alt_next = (*oldhead).uh_alt_next.ptr;
                if alt_next.is_null() {
                    // There is no branch, only free one header
                    rs_u_freeheader(buf, oldhead, &mut old_curhead as *mut UHeaderHandle);
                } else {
                    // Free the oldest alternate branch as a whole
                    let mut uhfree = oldhead;
                    loop {
                        let next_alt = (*uhfree).uh_alt_next.ptr;
                        if next_alt.is_null() {
                            break;
                        }
                        uhfree = next_alt;
                    }
                    rs_u_freebranch(buf, uhfree, &mut old_curhead as *mut UHeaderHandle);
                }
            }
        }

        if uhp.is_null() {
            // No undo at all
            if !old_curhead.is_null() {
                rs_u_freebranch(buf, old_curhead, std::ptr::null_mut());
            }
            nvim_buf_set_b_u_synced(buf, false);
            return OK;
        }

        // Set up the new header
        (*uhp).uh_prev.ptr = std::ptr::null_mut();
        let newhead = nvim_buf_get_b_u_newhead(buf);
        (*uhp).uh_next.ptr = newhead;
        (*uhp).uh_alt_next.ptr = old_curhead;

        if !old_curhead.is_null() {
            let alt_prev = (*old_curhead).uh_alt_prev.ptr;
            (*uhp).uh_alt_prev.ptr = alt_prev;

            if !alt_prev.is_null() {
                (*alt_prev).uh_alt_next.ptr = uhp;
            }

            (*old_curhead).uh_alt_prev.ptr = uhp;

            let oldhead = nvim_buf_get_b_u_oldhead(buf);
            if oldhead == old_curhead {
                nvim_buf_set_b_u_oldhead(buf, uhp);
            }
        } else {
            (*uhp).uh_alt_prev.ptr = std::ptr::null_mut();
        }

        if !newhead.is_null() {
            (*newhead).uh_prev.ptr = uhp;
        }

        // Set sequence numbers and time
        let seq_last = nvim_buf_get_b_u_seq_last(buf);
        nvim_buf_set_b_u_seq_last(buf, seq_last + 1);
        (*uhp).uh_seq = seq_last + 1;
        nvim_buf_set_b_u_seq_cur(buf, seq_last + 1);

        let now = nvim_time_now();
        (*uhp).uh_time = now;
        (*uhp).uh_save_nr = 0;
        nvim_buf_set_b_u_time_cur(buf, now + 1);

        (*uhp).uh_walk = 0;
        (*uhp).uh_entry = ptr::null_mut();
        (*uhp).uh_getbot_entry = ptr::null_mut();

        // Save cursor position
        let mut lnum: LinenrT = 0;
        let mut col: ColnrT = 0;
        let mut coladd: ColnrT = 0;
        nvim_get_curwin_cursor(&mut lnum, &mut col, &mut coladd);
        (*uhp).uh_cursor.lnum = lnum;
        (*uhp).uh_cursor.col = col;
        (*uhp).uh_cursor.coladd = coladd;

        if nvim_curwin_virtual_active() && coladd > 0 {
            (*uhp).uh_cursor_vcol = nvim_getviscol();
        } else {
            (*uhp).uh_cursor_vcol = -1;
        }

        // Save changed and buffer empty flag
        let changed = nvim_buf_get_b_changed(buf);
        let ml_empty = nvim_buf_ml_is_empty(buf);
        let flags = (if changed { 1 } else { 0 }) + (if ml_empty { 2 } else { 0 });
        (*uhp).uh_flags = flags;

        // Save named marks and Visual marks (calls zero_fmark_additional_data and memmove)
        nvim_uhp_copy_marks_visual(buf, uhp);

        nvim_buf_set_b_u_newhead(buf, uhp);

        let oldhead = nvim_buf_get_b_u_oldhead(buf);
        if oldhead.is_null() {
            nvim_buf_set_b_u_oldhead(buf, uhp);
        }

        let numhead = nvim_buf_get_b_u_numhead(buf);
        nvim_buf_set_b_u_numhead(buf, numhead + 1);
    } else {
        if nvim_get_undolevel(buf) < 0 {
            // No undo at all
            return OK;
        }

        // When saving a single line, check if we can reuse existing entry
        if size == 1 {
            let mut uep = rs_u_get_headentry(buf);
            let mut prev_uep: *mut UEntry = ptr::null_mut();

            for _ in 0..10 {
                if uep.is_null() {
                    break;
                }

                let newhead = nvim_buf_get_b_u_newhead(buf);
                let getbot_entry = (*newhead).uh_getbot_entry;
                let ue_top = (*uep).ue_top;
                let ue_size = (*uep).ue_size;
                let ue_bot = (*uep).ue_bot;
                let ue_lcount = (*uep).ue_lcount;
                let line_count = nvim_buf_get_ml_line_count(buf);

                // Check if lines have been inserted/deleted
                let reuse_blocked = if getbot_entry != uep {
                    ue_top + ue_size + 1 != (if ue_bot == 0 { line_count + 1 } else { ue_bot })
                } else {
                    ue_lcount != line_count
                };

                if reuse_blocked
                    || (ue_size > 1 && top >= ue_top && top + 2 <= ue_top + ue_size + 1)
                {
                    break;
                }

                // If it's the same line we can skip saving it again
                if ue_size == 1 && ue_top == top {
                    if !prev_uep.is_null() {
                        // Move the found entry to become the last entry
                        rs_u_getbot(buf);
                        nvim_buf_set_b_u_synced(buf, false);

                        let uep_next = (*uep).ue_next;
                        (*prev_uep).ue_next = uep_next;

                        let newhead = nvim_buf_get_b_u_newhead(buf);
                        let entry = (*newhead).uh_entry;
                        (*uep).ue_next = entry;
                        (*newhead).uh_entry = uep;
                    }

                    // The executed command may change the line count
                    if newbot != 0 {
                        (*uep).ue_bot = newbot;
                    } else if bot > line_count {
                        (*uep).ue_bot = 0;
                    } else {
                        (*uep).ue_lcount = line_count;
                        let newhead = nvim_buf_get_b_u_newhead(buf);
                        (*newhead).uh_getbot_entry = uep;
                    }
                    return OK;
                }

                prev_uep = uep;
                uep = (*uep).ue_next;
            }
        }

        // Find line number for ue_bot for previous u_save()
        rs_u_getbot(buf);
    }

    // Add lines in front of entry list
    let uep = nvim_xcalloc(1, std::mem::size_of::<UEntry>()) as *mut UEntry;

    (*uep).ue_size = size;
    (*uep).ue_top = top;

    let line_count = nvim_buf_get_ml_line_count(buf);
    if newbot != 0 {
        (*uep).ue_bot = newbot;
    } else if bot > line_count {
        (*uep).ue_bot = 0;
    } else {
        (*uep).ue_lcount = line_count;
        let newhead = nvim_buf_get_b_u_newhead(buf);
        (*newhead).uh_getbot_entry = uep;
    }

    if size > 0 {
        (*uep).ue_array =
            nvim_xmalloc(std::mem::size_of::<*mut c_char>() * size as usize) as *mut *mut c_char;
        let mut lnum = top + 1;
        for i in 0..size {
            nvim_fast_breakcheck();
            if nvim_undo_got_int() {
                rs_u_freeentry(uep, i as c_int);
                return FAIL;
            }
            *(*uep).ue_array.offset(i as isize) = nvim_ml_get_buf_copy(buf, lnum);
            lnum += 1;
        }
    } else {
        (*uep).ue_array = ptr::null_mut();
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let entry = (*newhead).uh_entry;
    (*uep).ue_next = entry;
    (*newhead).uh_entry = uep;

    if reload {
        // Buffer was reloaded, notify text change subscribers
        let curbuf = nvim_get_curbuf();
        let curbuf_newhead = nvim_buf_get_b_u_newhead(curbuf);
        let flags = (*curbuf_newhead).uh_flags;
        (*curbuf_newhead).uh_flags = flags | UH_RELOAD;
    }

    nvim_buf_set_b_u_synced(buf, false);
    nvim_set_undo_undoes(false);

    OK
}

/// Save the line at cursor position for undo.
/// Rust implementation of u_save_cursor.
///
/// # Safety
///
/// Must be called from a valid Neovim context with curwin set.
#[export_name = "u_save_cursor"]
pub unsafe extern "C" fn rs_u_save_cursor() -> c_int {
    let mut lnum: LinenrT = 0;
    let mut col: ColnrT = 0;
    let mut coladd: ColnrT = 0;
    nvim_get_curwin_cursor(&mut lnum, &mut col, &mut coladd);

    let top = if lnum > 0 { lnum - 1 } else { 0 };
    let bot = lnum + 1;

    rs_u_save(top, bot)
}

/// Save lines between top and bot for both "u" and "U" command.
/// Rust implementation of u_save.
///
/// # Safety
///
/// Must be called with valid line numbers for curbuf.
#[export_name = "u_save"]
pub unsafe extern "C" fn rs_u_save(top: LinenrT, bot: LinenrT) -> c_int {
    rs_u_save_buf(nvim_get_curbuf(), top, bot)
}

/// Save lines between top and bot for the specified buffer.
/// Rust implementation of u_save_buf.
///
/// # Safety
///
/// Must be called with valid buffer handle and line numbers.
#[export_name = "u_save_buf"]
pub unsafe extern "C" fn rs_u_save_buf(buf: BufHandle, top: LinenrT, bot: LinenrT) -> c_int {
    let line_count = nvim_buf_get_ml_line_count(buf);

    if top >= bot || bot > (line_count + 1) {
        return FAIL;
    }

    if top + 2 == bot {
        u_saveline(buf, top + 1);
    }

    rs_u_savecommon(buf, top, bot, 0, false)
}

/// Save a line for substitution (used by ":s" and "~" command).
/// Rust implementation of u_savesub.
///
/// # Safety
///
/// Must be called with valid line number for curbuf.
#[export_name = "u_savesub"]
pub unsafe extern "C" fn rs_u_savesub(lnum: LinenrT) -> c_int {
    rs_u_savecommon(nvim_get_curbuf(), lnum - 1, lnum + 1, lnum + 1, false)
}

/// Save for line insertion (used by :s command).
/// Rust implementation of u_inssub.
///
/// # Safety
///
/// Must be called with valid line number for curbuf.
#[export_name = "u_inssub"]
pub unsafe extern "C" fn rs_u_inssub(lnum: LinenrT) -> c_int {
    rs_u_savecommon(nvim_get_curbuf(), lnum - 1, lnum, lnum + 1, false)
}

/// Save lines for deletion.
/// Rust implementation of u_savedel.
///
/// # Safety
///
/// Must be called with valid line numbers for curbuf.
#[export_name = "u_savedel"]
pub unsafe extern "C" fn rs_u_savedel(lnum: LinenrT, nlines: LinenrT) -> c_int {
    let buf = nvim_get_curbuf();
    let line_count = nvim_buf_get_ml_line_count(buf);
    let newbot = if nlines == line_count { 2 } else { lnum };

    rs_u_savecommon(buf, lnum - 1, lnum + nlines, newbot, false)
}

/// Find the first line that was changed and set uh_cursor to that line.
/// This is used after reloading a buffer.
/// Rust implementation of u_find_first_changed.
///
/// # Safety
///
/// Must be called from a valid Neovim context.
#[export_name = "u_find_first_changed"]
pub unsafe extern "C" fn rs_u_find_first_changed() {
    let curbuf = nvim_get_curbuf();
    let uhp = nvim_buf_get_b_u_newhead(curbuf);

    // If curhead is set or newhead is null, return early
    if !nvim_buf_get_b_u_curhead(curbuf).is_null() || uhp.is_null() {
        return; // undid something in an autocmd?
    }

    // Check that the last undo block was for the whole file
    let uep = (*uhp).uh_entry;
    if (*uep).ue_top != 0 || (*uep).ue_bot != 0 {
        return;
    }

    let line_count = nvim_buf_get_ml_line_count(curbuf);
    let ue_size = (*uep).ue_size;

    // Find the first line that differs
    let mut lnum: LinenrT = 1;
    while lnum < line_count && lnum <= ue_size {
        // Compare buffer line at lnum with ue_array[lnum - 1]
        let buf_line = nvim_ml_get_buf_line(curbuf, lnum);
        let arr_line = *(*uep).ue_array.offset((lnum - 1) as isize);
        if libc::strcmp(arr_line, buf_line) != 0 {
            (*uhp).uh_cursor.lnum = 0;
            (*uhp).uh_cursor.col = 0;
            (*uhp).uh_cursor.coladd = 0;
            (*uhp).uh_cursor.lnum = lnum;
            return;
        }
        lnum += 1;
    }

    // Lines added or deleted at the end, put cursor there
    if line_count != ue_size {
        (*uhp).uh_cursor.lnum = 0;
        (*uhp).uh_cursor.col = 0;
        (*uhp).uh_cursor.coladd = 0;
        (*uhp).uh_cursor.lnum = lnum;
    }
}

/// Restore the line saved for "U" command.
/// Rust implementation of u_undoline.
///
/// # Safety
///
/// Must be called from a valid Neovim context.
#[export_name = "u_undoline"]
pub unsafe extern "C" fn rs_u_undoline() {
    let curbuf = nvim_get_curbuf();
    let line_ptr = nvim_buf_get_b_u_line_ptr(curbuf);
    let line_lnum = nvim_buf_get_b_u_line_lnum(curbuf);
    let line_count = nvim_buf_get_ml_line_count(curbuf);

    // Check if line pointer is valid
    if line_ptr.is_null() || line_lnum > line_count {
        nvim_beep_flush();
        return;
    }

    // First save the line for the 'u' command
    if rs_u_savecommon(curbuf, line_lnum - 1, line_lnum + 1, 0, false) == FAIL {
        return;
    }

    // Do the replacement and swap
    nvim_u_undoline_replace_and_swap();

    // Handle column position
    let t = nvim_buf_get_b_u_line_colnr(curbuf);
    if nvim_undo_curwin_get_cursor_lnum() == line_lnum {
        nvim_buf_set_b_u_line_colnr(curbuf, nvim_undo_curwin_get_cursor_col());
    }
    nvim_undo_curwin_set_cursor_col(t);
    nvim_undo_curwin_set_cursor_lnum(line_lnum);
    nvim_check_cursor_col_curwin();
}

/// Given a buffer, return the undo header. If none is set, create one first.
/// NULL will be returned if e.g undolevels = -1 (undo disabled).
/// Rust implementation of u_force_get_undo_header.
///
/// # Safety
///
/// Must be called with a valid buffer handle.
#[export_name = "u_force_get_undo_header"]
pub unsafe extern "C" fn rs_u_force_get_undo_header(buf: BufHandle) -> UHeaderHandle {
    let mut uhp = nvim_buf_get_b_u_curhead(buf);
    if uhp.is_null() {
        uhp = nvim_buf_get_b_u_newhead(buf);
    }

    // Create the first undo header for the buffer
    if uhp.is_null() {
        // Args are tricky: this means replace empty range by empty range
        rs_u_savecommon(buf, 0, 1, 1, true);

        uhp = nvim_buf_get_b_u_curhead(buf);
        if uhp.is_null() {
            uhp = nvim_buf_get_b_u_newhead(buf);
            // If undolevel > 0 and still no header, abort
            // (This shouldn't happen in normal operation)
        }
    }
    uhp
}

/// Navigate the undo tree to a specific time, sequence number, or file save state.
///
/// This is the core implementation for `:earlier`, `:later`, and `:undo N` commands.
///
/// # Arguments
///
/// * `step` - Number of steps to go (negative for undo/earlier, positive for redo/later)
/// * `sec` - If true, step is in seconds
/// * `file` - If true, step is by file writes
/// * `absolute` - If true, step is an absolute sequence number (`:undo N`)
///
/// # Safety
///
/// Accesses global state via C FFI. Must be called with valid global state.
#[export_name = "undo_time"]
pub unsafe extern "C" fn rs_undo_time(step: c_int, sec: bool, file: bool, absolute: bool) {
    // Check text lock first
    if nvim_text_locked() {
        nvim_text_locked_msg();
        return;
    }

    let buf = nvim_get_curbuf();

    // First make sure the current undoable change is synced.
    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
    }

    nvim_set_u_newcount(0);
    nvim_set_u_oldcount(if nvim_buf_ml_is_empty(buf) { -1 } else { 0 });

    let mut dosec = sec;
    let mut dofile = file;
    let mut above = false;
    let mut did_undo = true;

    // "target" is the node below which we want to be.
    // Init "closest" to a value we can't reach.
    let (mut target, mut closest): (c_int, c_int) = if absolute {
        (step, -1)
    } else if dosec {
        (
            (nvim_buf_get_b_u_time_cur(buf) as c_int) + step,
            if step < 0 {
                -1
            } else {
                (nvim_undo_os_time() + 1) as c_int
            },
        )
    } else if dofile {
        let save_nr_cur = nvim_buf_get_b_u_save_nr_cur(buf);
        let mut t: c_int;

        if step < 0 {
            // Going back to a previous write. If there were changes after
            // the last write, count that as moving one file-write, so
            // that ":earlier 1f" undoes all changes since the last save.
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let uhp = if !curhead.is_null() {
                (*curhead).uh_next.ptr
            } else {
                nvim_buf_get_b_u_newhead(buf)
            };

            if !uhp.is_null() && (*uhp).uh_save_nr != 0 {
                // "uh_save_nr" was set in the last block, that means
                // there were no changes since the last write
                t = save_nr_cur + step;
            } else {
                // count the changes since the last write as one step
                t = save_nr_cur + step + 1;
            }

            if t <= 0 {
                // Go to before first write: before the oldest change. Use
                // the sequence number for that.
                dofile = false;
                t = 0; // Will be adjusted below
            }
            (
                t,
                if step < 0 && dofile {
                    -1
                } else if dofile {
                    nvim_buf_get_b_u_save_nr_last(buf) + 2
                } else {
                    nvim_buf_get_b_u_seq_last(buf) + 2
                },
            )
        } else {
            // Moving forward to a newer write.
            t = save_nr_cur + step;
            let save_nr_last = nvim_buf_get_b_u_save_nr_last(buf);
            if t > save_nr_last {
                // Go to after last write: after the latest change. Use
                // the sequence number for that.
                t = nvim_buf_get_b_u_seq_last(buf) + 1;
                dofile = false;
            }
            (t, save_nr_last + 2)
        }
    } else {
        (
            nvim_buf_get_b_u_seq_cur(buf) + step,
            if step < 0 {
                -1
            } else {
                nvim_buf_get_b_u_seq_last(buf) + 2
            },
        )
    };

    // Adjust target and closest for step direction
    if !absolute {
        if step < 0 {
            if target < 0 {
                target = 0;
            }
            closest = -1;
        } else {
            // Recalculate closest for positive step
            closest = if dosec {
                (nvim_undo_os_time() + 1) as c_int
            } else if dofile {
                nvim_buf_get_b_u_save_nr_last(buf) + 2
            } else {
                nvim_buf_get_b_u_seq_last(buf) + 2
            };
            if target >= closest {
                target = closest - 1;
            }
        }
    }

    let closest_start = closest;
    let mut closest_seq = nvim_buf_get_b_u_seq_cur(buf);

    // When "target" is 0; Back to origin.
    if target == 0 {
        undo_time_to_target(buf, target, 0, 0, above, &mut did_undo);
        u_undo_end(did_undo, absolute, false);
        return;
    }

    // May do this twice:
    // 1. Search for "target", update "closest" to the best match found.
    // 2. If "target" not found search for "closest".
    //
    // When using the closest time we use the sequence number in the second
    // round, because there may be several entries with the same time.
    for round in 1..=2 {
        // Find the path from the current state to where we want to go. The
        // desired state can be anywhere in the undo tree, need to go all over
        // it. We put "nomark" in uh_walk where we have been without success,
        // "mark" where it could possibly be.
        let mark = nvim_inc_lastmark();
        let nomark = nvim_inc_lastmark();

        let curhead = nvim_buf_get_b_u_curhead(buf);
        let mut uhp = if curhead.is_null() {
            // at leaf of the tree
            nvim_buf_get_b_u_newhead(buf)
        } else {
            curhead
        };

        while !uhp.is_null() {
            (*uhp).uh_walk = mark;
            let val = if dosec {
                (*uhp).uh_time as c_int
            } else if dofile {
                (*uhp).uh_save_nr
            } else {
                (*uhp).uh_seq
            };

            if round == 1 && !(dofile && val == 0) {
                // Remember the header that is closest to the target.
                // It must be at least in the right direction (checked with
                // "b_u_seq_cur"). When the timestamp is equal find the
                // highest/lowest sequence number.
                let uh_seq = (*uhp).uh_seq;
                let seq_cur = nvim_buf_get_b_u_seq_cur(buf);
                let in_right_direction = if step < 0 {
                    uh_seq <= seq_cur
                } else {
                    uh_seq > seq_cur
                };

                if in_right_direction {
                    let is_closer = if dosec && val == closest {
                        if step < 0 {
                            uh_seq < closest_seq
                        } else {
                            uh_seq > closest_seq
                        }
                    } else if closest == closest_start {
                        true
                    } else if val > target {
                        if closest > target {
                            val - target <= closest - target
                        } else {
                            val - target <= target - closest
                        }
                    } else {
                        // val <= target
                        if closest > target {
                            target - val <= closest - target
                        } else {
                            target - val <= target - closest
                        }
                    };

                    if is_closer {
                        closest = val;
                        closest_seq = uh_seq;
                    }
                }
            }

            // Quit searching when we found a match. But when searching for a
            // time we need to continue looking for the best uh_seq.
            if target == val && !dosec {
                target = (*uhp).uh_seq;
                break;
            }

            // go down in the tree if we haven't been there
            let prev = (*uhp).uh_prev.ptr;
            if !prev.is_null() && (*prev).uh_walk != nomark && (*prev).uh_walk != mark {
                uhp = prev;
            } else {
                let alt_next = (*uhp).uh_alt_next.ptr;
                if !alt_next.is_null()
                    && (*alt_next).uh_walk != nomark
                    && (*alt_next).uh_walk != mark
                {
                    // go to alternate branch if we haven't been there
                    uhp = alt_next;
                } else {
                    let next = (*uhp).uh_next.ptr;
                    let alt_prev = (*uhp).uh_alt_prev.ptr;
                    if !next.is_null()
                        && alt_prev.is_null()
                        && (*next).uh_walk != nomark
                        && (*next).uh_walk != mark
                    {
                        // go up in the tree if we haven't been there and we are at the
                        // start of alternate branches
                        // If still at the start we don't go through this change.
                        let curhead = nvim_buf_get_b_u_curhead(buf);
                        if uhp == curhead {
                            (*uhp).uh_walk = nomark;
                        }
                        uhp = next;
                    } else {
                        // need to backtrack; mark this node as useless
                        (*uhp).uh_walk = nomark;
                        if !alt_prev.is_null() {
                            uhp = alt_prev;
                        } else {
                            uhp = (*uhp).uh_next.ptr;
                        }
                    }
                }
            }
        }

        if !uhp.is_null() {
            // found it
            break;
        }

        if absolute {
            nvim_semsg_undo_number_not_found(i64::from(step));
            return;
        }

        if closest == closest_start {
            if step < 0 {
                nvim_msg_oldest_change();
            } else {
                nvim_msg_newest_change();
            }
            return;
        }

        target = closest_seq;
        dosec = false;
        dofile = false;
        if step < 0 {
            above = true; // stop above the header
        }
    }

    // If we found it: Follow the path to go to where we want to be.
    undo_time_to_target(
        buf,
        target,
        nvim_inc_lastmark() - 2,
        nvim_inc_lastmark() - 2,
        above,
        &mut did_undo,
    );
    u_undo_end(did_undo, absolute, false);
}

/// Helper function to walk to target in undo tree.
/// This follows the path from the current state to the target state.
///
/// # Safety
///
/// Must be called with valid buffer handle and mark values.
unsafe fn undo_time_to_target(
    buf: BufHandle,
    target: c_int,
    mark: c_int,
    nomark: c_int,
    above: bool,
    did_undo: &mut bool,
) {
    // First go up the tree as much as needed.
    while !nvim_undo_got_int() {
        // Do the change warning now, for the same reason as above.
        nvim_change_warning_curbuf();

        let curhead = nvim_buf_get_b_u_curhead(buf);
        let uhp = if curhead.is_null() {
            nvim_buf_get_b_u_newhead(buf)
        } else {
            (*curhead).uh_next.ptr
        };

        if uhp.is_null()
            || (target > 0 && (*uhp).uh_walk != mark)
            || ((*uhp).uh_seq == target && !above)
        {
            break;
        }

        nvim_buf_set_b_u_curhead(buf, uhp);
        u_undoredo(true, true);
        if target > 0 {
            (*uhp).uh_walk = nomark; // don't go back down here
        }
    }

    // When back to origin, redo is not needed.
    if target > 0 {
        // And now go down the tree (redo), branching off where needed.
        while !nvim_undo_got_int() {
            // Do the change warning now, for the same reason as above.
            nvim_change_warning_curbuf();

            let mut uhp = nvim_buf_get_b_u_curhead(buf);
            if uhp.is_null() {
                break;
            }

            // Go back to the first branch with a mark.
            let mut alt_prev = (*uhp).uh_alt_prev.ptr;
            while !alt_prev.is_null() && (*alt_prev).uh_walk == mark {
                uhp = alt_prev;
                alt_prev = (*uhp).uh_alt_prev.ptr;
            }

            // Find the last branch with a mark, that's the one.
            let mut last = uhp;
            let mut alt_next = (*last).uh_alt_next.ptr;
            while !alt_next.is_null() && (*alt_next).uh_walk == mark {
                last = alt_next;
                alt_next = (*last).uh_alt_next.ptr;
            }

            if last != uhp {
                // Make the used branch the first entry in the list of
                // alternatives to make "u" and CTRL-R take this branch.
                let mut first = uhp;
                let mut first_alt_prev = (*first).uh_alt_prev.ptr;
                while !first_alt_prev.is_null() {
                    first = first_alt_prev;
                    first_alt_prev = (*first).uh_alt_prev.ptr;
                }

                let last_alt_next = (*last).uh_alt_next.ptr;
                if !last_alt_next.is_null() {
                    let last_alt_prev = (*last).uh_alt_prev.ptr;
                    (*last_alt_next).uh_alt_prev.ptr = last_alt_prev;
                }

                let last_alt_prev = (*last).uh_alt_prev.ptr;
                (*last_alt_prev).uh_alt_next.ptr = (*last).uh_alt_next.ptr;
                (*last).uh_alt_prev.ptr = std::ptr::null_mut();
                (*last).uh_alt_next.ptr = first;
                (*first).uh_alt_prev.ptr = last;

                let oldhead = nvim_buf_get_b_u_oldhead(buf);
                if oldhead == first {
                    nvim_buf_set_b_u_oldhead(buf, last);
                }

                uhp = last;
                let next = (*uhp).uh_next.ptr;
                if !next.is_null() {
                    (*next).uh_prev.ptr = uhp;
                }
            }

            nvim_buf_set_b_u_curhead(buf, uhp);

            if (*uhp).uh_walk != mark {
                break; // must have reached the target
            }

            // Stop when going backwards in time and didn't find the exact
            // header we were looking for.
            if (*uhp).uh_seq == target && above {
                nvim_buf_set_b_u_seq_cur(buf, target - 1);
                break;
            }

            u_undoredo(false, true);

            // Advance "curhead" to below the header we last used. If it
            // becomes NULL then we need to set "newhead" to this leaf.
            let prev = (*uhp).uh_prev.ptr;
            if prev.is_null() {
                nvim_buf_set_b_u_newhead(buf, uhp);
            }
            nvim_buf_set_b_u_curhead(buf, prev);
            *did_undo = false;

            if (*uhp).uh_seq == target {
                // found it!
                break;
            }

            let prev = (*uhp).uh_prev.ptr;
            if prev.is_null() || (*prev).uh_walk != mark {
                // Need to redo more but can't find it...
                nvim_internal_error_undo_time();
                break;
            }
        }
    }
}

// =============================================================================
// Phase 349: Undo Time Formatting
// =============================================================================

extern "C" {
    fn nvim_undo_strftime(buf: *mut c_char, buflen: usize, fmt: *const c_char, tt: TimeT) -> usize;
    fn nvim_undo_time(seconds_count: i64) -> *const c_char;
}

/// Format a timestamp for display in undo messages.
///
/// This formats the given time relative to the current time:
/// - Within 100 seconds: "N second(s) ago"
/// - Within 12 hours: "HH:MM:SS"
/// - Older: "YYYY/MM/DD HH:MM:SS"
///
/// # Safety
///
/// `buf` must be a valid pointer to a buffer of at least `buflen` bytes.
#[export_name = "undo_fmt_time"]
pub unsafe extern "C" fn rs_undo_fmt_time(buf: *mut c_char, buflen: usize, tt: TimeT) {
    if buf.is_null() || buflen == 0 {
        return;
    }

    let now = nvim_undo_os_time();
    let seconds_diff = now - tt;

    if seconds_diff >= 100 {
        // Use strftime for longer times
        let n = if seconds_diff < 60 * 60 * 12 {
            // Within 12 hours - show time only
            nvim_undo_strftime(buf, buflen, c"%H:%M:%S".as_ptr(), tt)
        } else {
            // Longer ago - show full date and time
            nvim_undo_strftime(buf, buflen, c"%Y/%m/%d %H:%M:%S".as_ptr(), tt)
        };

        if n == 0 {
            // strftime failed, clear buffer
            *buf = 0;
        }
    } else {
        // Within 100 seconds - use "N second(s) ago" format
        // Call C's gettext for pluralization
        let msg = nvim_undo_time(seconds_diff);
        if !msg.is_null() {
            // Copy the message to the buffer
            let msg_len = libc::strlen(msg);
            let copy_len = if msg_len < buflen - 1 {
                msg_len
            } else {
                buflen - 1
            };
            ptr::copy_nonoverlapping(msg, buf, copy_len);
            *buf.add(copy_len) = 0;
        } else {
            *buf = 0;
        }
    }
}

/// Get the time elapsed since a timestamp in seconds.
///
/// # Safety
///
/// No specific safety requirements beyond normal FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_time_elapsed(tt: TimeT) -> i64 {
    let now = nvim_undo_os_time();
    now - tt
}

/// Check if a timestamp is within a certain number of seconds of now.
///
/// # Safety
///
/// No specific safety requirements beyond normal FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_time_within(tt: TimeT, seconds: i64) -> bool {
    let elapsed = nvim_undo_os_time() - tt;
    elapsed.abs() <= seconds
}

// =============================================================================
// Phase 1: Undo Tree Traversal Helpers
// =============================================================================

/// Walk the undo tree and count the total number of undo headers.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_tree_count(buf: BufHandle) -> c_int {
    let mut count: c_int = 0;
    let mut uhp = nvim_buf_get_b_u_oldhead(buf);

    while !uhp.is_null() {
        count += 1;
        // Count alternate branches
        let mut alt = (*uhp).uh_alt_next.ptr;
        while !alt.is_null() {
            count += rs_undo_branch_count(alt);
            alt = (*alt).uh_alt_next.ptr;
        }
        uhp = (*uhp).uh_prev.ptr;
    }

    count
}

/// Count headers in a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_branch_count(uhp: UHeaderHandle) -> c_int {
    if uhp.is_null() {
        return 0;
    }

    let mut count: c_int = 1;
    let mut current = (*uhp).uh_prev.ptr;

    while !current.is_null() {
        count += 1;
        // Count alternate branches
        let mut alt = (*current).uh_alt_next.ptr;
        while !alt.is_null() {
            count += rs_undo_branch_count(alt);
            alt = (*alt).uh_alt_next.ptr;
        }
        current = (*current).uh_prev.ptr;
    }

    count
}

/// Find an undo header by sequence number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_find_seq(buf: BufHandle, seq: c_int) -> UHeaderHandle {
    let mut uhp = nvim_buf_get_b_u_newhead(buf);

    while !uhp.is_null() {
        if (*uhp).uh_seq == seq {
            return uhp;
        }

        // Check alternate branches
        let mut alt = (*uhp).uh_alt_next.ptr;
        while !alt.is_null() {
            let found = rs_undo_find_seq_in_branch(alt, seq);
            if !found.is_null() {
                return found;
            }
            alt = (*alt).uh_alt_next.ptr;
        }

        uhp = (*uhp).uh_next.ptr;
    }

    std::ptr::null_mut()
}

/// Find sequence number in a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
unsafe fn rs_undo_find_seq_in_branch(uhp: UHeaderHandle, seq: c_int) -> UHeaderHandle {
    if uhp.is_null() {
        return std::ptr::null_mut();
    }

    if (*uhp).uh_seq == seq {
        return uhp;
    }

    let mut current = (*uhp).uh_prev.ptr;
    while !current.is_null() {
        if (*current).uh_seq == seq {
            return current;
        }

        // Check alternate branches
        let mut alt = (*current).uh_alt_next.ptr;
        while !alt.is_null() {
            let found = rs_undo_find_seq_in_branch(alt, seq);
            if !found.is_null() {
                return found;
            }
            alt = (*alt).uh_alt_next.ptr;
        }

        current = (*current).uh_prev.ptr;
    }

    std::ptr::null_mut()
}

/// Count the number of undo entries in a header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_count_entries(uhp: UHeaderHandle) -> c_int {
    if uhp.is_null() {
        return 0;
    }

    let mut count: c_int = 0;
    let mut uep = (*uhp).uh_entry;

    while !uep.is_null() {
        count += 1;
        uep = (*uep).ue_next;
    }

    count
}

/// Get the depth of the undo tree (longest path from root to leaf).
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_tree_depth(buf: BufHandle) -> c_int {
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    rs_undo_get_branch_depth(oldhead)
}

/// Get the depth of a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_branch_depth(uhp: UHeaderHandle) -> c_int {
    if uhp.is_null() {
        return 0;
    }

    let mut max_depth: c_int = 0;

    // Check this branch
    let mut current = uhp;
    let mut depth: c_int = 0;
    while !current.is_null() {
        depth += 1;

        // Check alternate branches
        let mut alt = (*current).uh_alt_next.ptr;
        while !alt.is_null() {
            let alt_depth = rs_undo_get_branch_depth(alt);
            if depth + alt_depth > max_depth {
                max_depth = depth + alt_depth;
            }
            alt = (*alt).uh_alt_next.ptr;
        }

        current = (*current).uh_prev.ptr;
    }

    if depth > max_depth {
        depth
    } else {
        max_depth
    }
}

/// Check if undo is available for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_can_undo(buf: BufHandle) -> bool {
    let curhead = nvim_buf_get_b_u_curhead(buf);
    let newhead = nvim_buf_get_b_u_newhead(buf);

    // Can undo if curhead is NULL (first undo) and newhead exists
    // or if curhead exists and has a next header
    if curhead.is_null() {
        !newhead.is_null()
    } else {
        !(*curhead).uh_next.ptr.is_null()
    }
}

/// Check if redo is available for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_can_redo(buf: BufHandle) -> bool {
    let curhead = nvim_buf_get_b_u_curhead(buf);
    // Can redo if curhead exists (there's something to redo)
    !curhead.is_null()
}

/// Get the current undo sequence number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_seq_cur(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_seq_cur(buf)
}

/// Get the last undo sequence number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_seq_last(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_seq_last(buf)
}

/// Get the number of undo headers in the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_numhead(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_numhead(buf)
}

/// Get the current undo time.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_time_cur(buf: BufHandle) -> TimeT {
    nvim_buf_get_b_u_time_cur(buf)
}

/// Get the save number of the current header.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_save_nr_cur(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_save_nr_cur(buf)
}

/// Get the last save number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_save_nr_last(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_save_nr_last(buf)
}

/// Check if the undo list is synced.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_is_synced(buf: BufHandle) -> bool {
    nvim_buf_get_b_u_synced(buf)
}

// =============================================================================
// Phase 2: Undo Header Accessors
// =============================================================================

/// Get the sequence number from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_seq(uhp: UHeaderHandle) -> c_int {
    (*uhp).uh_seq
}

/// Get the time from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_time(uhp: UHeaderHandle) -> TimeT {
    (*uhp).uh_time
}

/// Get the flags from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_flags(uhp: UHeaderHandle) -> c_int {
    (*uhp).uh_flags
}

/// Get the save number from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_save_nr(uhp: UHeaderHandle) -> c_int {
    (*uhp).uh_save_nr
}

/// Get the next header in the list.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_next(uhp: UHeaderHandle) -> UHeaderHandle {
    (*uhp).uh_next.ptr
}

/// Get the previous header in the list.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_prev(uhp: UHeaderHandle) -> UHeaderHandle {
    (*uhp).uh_prev.ptr
}

/// Get the next alternate header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_alt_next(uhp: UHeaderHandle) -> UHeaderHandle {
    (*uhp).uh_alt_next.ptr
}

/// Get the previous alternate header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_alt_prev(uhp: UHeaderHandle) -> UHeaderHandle {
    (*uhp).uh_alt_prev.ptr
}

/// Get the first entry in an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_entry(uhp: UHeaderHandle) -> UEntryHandle {
    (*uhp).uh_entry
}

/// Check if an undo header is NULL.
#[no_mangle]
pub extern "C" fn rs_uhp_is_null(uhp: UHeaderHandle) -> bool {
    uhp.is_null()
}

/// Check if an undo entry is NULL.
#[no_mangle]
pub extern "C" fn rs_uep_is_null(uep: UEntryHandle) -> bool {
    uep.is_null()
}

// =============================================================================
// Phase 2: Undo Entry Accessors
// =============================================================================

/// Get the top line number from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_top(uep: UEntryHandle) -> LinenrT {
    (*uep).ue_top
}

/// Get the bottom line number from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_bot(uep: UEntryHandle) -> LinenrT {
    (*uep).ue_bot
}

/// Get the line count from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_lcount(uep: UEntryHandle) -> LinenrT {
    (*uep).ue_lcount
}

/// Get the size (number of lines) from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_size(uep: UEntryHandle) -> LinenrT {
    (*uep).ue_size
}

/// Get the next entry in the list.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_next(uep: UEntryHandle) -> UEntryHandle {
    (*uep).ue_next
}

/// Get a line from the undo entry's array.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
/// The index must be valid (0 <= idx < size).
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_line(uep: UEntryHandle, idx: LinenrT) -> *const c_char {
    *(*uep).ue_array.offset(idx as isize)
}

/// Get the number of lines affected by an undo entry.
/// This is the number of lines that will be replaced (bot - top - 1).
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_line_count(uep: UEntryHandle) -> LinenrT {
    let top = (*uep).ue_top;
    let bot = (*uep).ue_bot;
    if bot == 0 {
        // Bot of 0 means end of buffer
        0
    } else {
        bot - top - 1
    }
}

// =============================================================================
// Phase 1: Undo File I/O Helper Functions
// =============================================================================
//
// These helper functions are infrastructure for Phase 2 (u_write_undo) and
// Phase 3 (u_read_undo). They are intentionally marked allow(dead_code) until
// those phases are implemented.

// Additional C functions needed for string allocation
extern "C" {
    #[link_name = "xmallocz"]
    fn nvim_xmallocz(size: usize) -> *mut c_void;
}

/// Number of named marks (NMARKS from mark_defs.h).
#[allow(dead_code)]
const NMARKS: usize = 26;

/// Write bytes to the undo file.
///
/// Wrapper around fwrite for undo file operations.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `ptr` must point to valid memory of at least `len` bytes
#[allow(dead_code)]
#[inline]
unsafe fn undo_write(fp: FileHandle, ptr: *const u8, len: usize) -> bool {
    if fp.is_null() || len == 0 {
        return len == 0;
    }
    nvim_undo_fwrite(ptr as *const c_void, len, 1, fp) == 1
}

/// Write a number in big-endian format with the specified number of bytes.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `len` must be between 1 and 8
#[allow(dead_code)]
#[inline]
unsafe fn undo_write_bytes(fp: FileHandle, nr: u64, len: usize) -> bool {
    debug_assert!(len > 0 && len <= 8);
    let mut buf = [0u8; 8];
    for (i, byte) in buf.iter_mut().enumerate().take(len) {
        *byte = ((nr >> ((len - 1 - i) * 8)) & 0xff) as u8;
    }
    undo_write(fp, buf.as_ptr(), len)
}

/// Write a 4-byte integer in big-endian format.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_write_4(fp: FileHandle, val: i32) -> bool {
    undo_write_bytes(fp, val as u64, 4)
}

/// Write a 2-byte integer in big-endian format.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_write_2(fp: FileHandle, val: u16) -> bool {
    undo_write_bytes(fp, u64::from(val), 2)
}

/// Write a time_t value (8 bytes).
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_write_time(fp: FileHandle, time: TimeT) -> bool {
    undo_write_bytes(fp, time as u64, 8)
}

/// Read bytes from the undo file.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `buf` must point to valid memory of at least `size` bytes
#[allow(dead_code)]
#[inline]
unsafe fn undo_read(fp: FileHandle, buf: *mut u8, size: usize) -> bool {
    if fp.is_null() || size == 0 {
        return size == 0;
    }
    let result = nvim_undo_fread(buf as *mut c_void, size, 1, fp) == 1;
    if !result {
        // Fill with zeros on error
        ptr::write_bytes(buf, 0, size);
    }
    result
}

/// Read a 4-byte integer in big-endian format.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_read_4c(fp: FileHandle) -> c_int {
    nvim_undo_get4c(fp)
}

/// Read a 2-byte integer in big-endian format.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_read_2c(fp: FileHandle) -> c_int {
    nvim_undo_get2c(fp)
}

/// Read a single byte from the undo file.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_read_byte(fp: FileHandle) -> c_int {
    nvim_undo_fgetc(fp)
}

/// Read a time_t value (8 bytes).
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn undo_read_time(fp: FileHandle) -> TimeT {
    nvim_undo_get8ctime(fp)
}

/// Read a string of specified length from the undo file.
///
/// Allocates memory for the string and appends a null terminator.
/// Returns null on error.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - Caller is responsible for freeing the returned memory
#[allow(dead_code)]
#[inline]
unsafe fn undo_read_string(fp: FileHandle, len: usize) -> *mut c_char {
    if len == 0 {
        // Allocate empty string
        let ptr = nvim_xmallocz(0);
        return ptr as *mut c_char;
    }

    let ptr = nvim_xmallocz(len);
    if !undo_read(fp, ptr as *mut u8, len) {
        nvim_xfree(ptr);
        return ptr::null_mut();
    }
    ptr as *mut c_char
}

/// Serialize a position (lnum, col, coladd) to the undo file.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn serialize_pos(fp: FileHandle, lnum: LinenrT, col: ColnrT, coladd: ColnrT) -> bool {
    undo_write_4(fp, lnum) && undo_write_4(fp, col) && undo_write_4(fp, coladd)
}

/// Serialize visual info to the undo file.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uhp` must be a valid undo header handle
#[allow(dead_code)]
#[inline]
unsafe fn serialize_visualinfo(fp: FileHandle, uhp: UHeaderHandle) -> bool {
    // vi_start
    serialize_pos(
        fp,
        (*uhp).uh_visual.vi_start.lnum,
        (*uhp).uh_visual.vi_start.col,
        (*uhp).uh_visual.vi_start.coladd,
    ) &&
    // vi_end
    serialize_pos(
        fp,
        (*uhp).uh_visual.vi_end.lnum,
        (*uhp).uh_visual.vi_end.col,
        (*uhp).uh_visual.vi_end.coladd,
    ) &&
    // vi_mode
    undo_write_4(fp, (*uhp).uh_visual.vi_mode) &&
    // vi_curswant
    undo_write_4(fp, (*uhp).uh_visual.vi_curswant)
}

/// Write the header pointer as a sequence number.
///
/// When writing pointers, we use the sequence number instead.
/// This is converted back to pointers when reading.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uhp` may be null (writes 0)
#[allow(dead_code)]
#[inline]
unsafe fn put_header_ptr(fp: FileHandle, uhp: UHeaderHandle) -> bool {
    let seq = if uhp.is_null() {
        0
    } else {
        (*uhp).uh_seq as u64
    };
    undo_write_bytes(fp, seq, 4)
}

/// Write the header pointer using the stored sequence number.
///
/// # Safety
///
/// - `fp` must be a valid file handle
#[allow(dead_code)]
#[inline]
unsafe fn put_header_ptr_by_seq(fp: FileHandle, seq: c_int) -> bool {
    let val = if seq < 0 { 0 } else { seq as u64 };
    undo_write_bytes(fp, val, 4)
}

// =============================================================================
// Phase 1: Undo File Serialization Functions
// =============================================================================

/// Serialize an undo entry (u_entry_T) to the file.
///
/// Writes: ue_top, ue_bot, ue_lcount, ue_size, then each line with its length.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uep` must be a valid undo entry handle
#[allow(dead_code)]
unsafe fn serialize_uep(fp: FileHandle, uep: UEntryHandle) -> bool {
    // Write entry header fields
    if !undo_write_4(fp, (*uep).ue_top) {
        return false;
    }
    if !undo_write_4(fp, (*uep).ue_bot) {
        return false;
    }
    if !undo_write_4(fp, (*uep).ue_lcount) {
        return false;
    }

    let size = (*uep).ue_size;
    if !undo_write_4(fp, size) {
        return false;
    }

    // Write each line in the array
    for i in 0..size {
        let line = *(*uep).ue_array.offset(i as isize);
        let len = if line.is_null() {
            0
        } else {
            libc::strlen(line)
        };

        // Write length first
        if !undo_write_bytes(fp, len as u64, 4) {
            return false;
        }

        // Write line content if non-empty
        if len > 0 && !undo_write(fp, line as *const u8, len) {
            return false;
        }
    }

    true
}

/// Serialize an undo header (u_header_T) to the file.
///
/// Writes the header magic, all header fields, then all entries and extmarks.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uhp` must be a valid undo header handle
#[allow(dead_code)]
unsafe fn serialize_uhp(fp: FileHandle, uhp: UHeaderHandle) -> bool {
    // Write header magic
    if !undo_write_2(fp, UF_HEADER_MAGIC) {
        return false;
    }

    // Write header pointers as sequence numbers
    let next_seq = if (*uhp).uh_next.ptr.is_null() {
        0
    } else {
        (*(*uhp).uh_next.ptr).uh_seq
    };
    if !put_header_ptr_by_seq(fp, next_seq) {
        return false;
    }
    let prev_seq = if (*uhp).uh_prev.ptr.is_null() {
        0
    } else {
        (*(*uhp).uh_prev.ptr).uh_seq
    };
    if !put_header_ptr_by_seq(fp, prev_seq) {
        return false;
    }
    let alt_next_seq = if (*uhp).uh_alt_next.ptr.is_null() {
        0
    } else {
        (*(*uhp).uh_alt_next.ptr).uh_seq
    };
    if !put_header_ptr_by_seq(fp, alt_next_seq) {
        return false;
    }
    let alt_prev_seq = if (*uhp).uh_alt_prev.ptr.is_null() {
        0
    } else {
        (*(*uhp).uh_alt_prev.ptr).uh_seq
    };
    if !put_header_ptr_by_seq(fp, alt_prev_seq) {
        return false;
    }

    // Write sequence number
    if !undo_write_4(fp, (*uhp).uh_seq) {
        return false;
    }

    // Write cursor position
    if !serialize_pos(
        fp,
        (*uhp).uh_cursor.lnum,
        (*uhp).uh_cursor.col,
        (*uhp).uh_cursor.coladd,
    ) {
        return false;
    }

    // Write cursor vcol
    if !undo_write_4(fp, (*uhp).uh_cursor_vcol) {
        return false;
    }

    // Write flags (2 bytes)
    if !undo_write_2(fp, (*uhp).uh_flags as u16) {
        return false;
    }

    // Write named marks (NMARKS = 26)
    for i in 0..NMARKS as c_int {
        if !serialize_pos(
            fp,
            (*uhp).uh_namedm[i as usize].mark.lnum,
            (*uhp).uh_namedm[i as usize].mark.col,
            (*uhp).uh_namedm[i as usize].mark.coladd,
        ) {
            return false;
        }
    }

    // Write visual info
    if !serialize_visualinfo(fp, uhp) {
        return false;
    }

    // Write time (8 bytes)
    if !undo_write_time(fp, (*uhp).uh_time) {
        return false;
    }

    // Write optional fields - save_nr
    if !undo_write_bytes(fp, 4, 1) {
        // length
        return false;
    }
    if !undo_write_bytes(fp, u64::from(UHP_SAVE_NR), 1) {
        // field id
        return false;
    }
    if !undo_write_4(fp, (*uhp).uh_save_nr) {
        return false;
    }

    // Write end marker for optional fields
    if !undo_write_bytes(fp, 0, 1) {
        return false;
    }

    // Write all undo entries
    let mut uep = (*uhp).uh_entry;
    while !uep.is_null() {
        if !undo_write_2(fp, UF_ENTRY_MAGIC) {
            return false;
        }
        if !serialize_uep(fp, uep) {
            return false;
        }
        uep = (*uep).ue_next;
    }

    // Write entry end magic
    if !undo_write_2(fp, UF_ENTRY_END_MAGIC) {
        return false;
    }

    // Write all extmark undo objects
    let extmark_count = (*uhp).uh_extmark.size;
    for i in 0..extmark_count {
        let obj = &*(*uhp).uh_extmark.items.add(i);
        let ext_type = obj.kind;
        // Only serialize splice and move types (kExtmarkSplice=0, kExtmarkMove=1)
        if ext_type == K_EXTMARK_SPLICE || ext_type == K_EXTMARK_MOVE {
            if !undo_write_2(fp, UF_ENTRY_MAGIC) {
                return false;
            }
            if !undo_write_4(fp, ext_type) {
                return false;
            }
            // Write the raw data bytes for splice or move
            let (ptr, size) = if ext_type == K_EXTMARK_SPLICE {
                (
                    std::ptr::addr_of!(obj.data.splice) as *const u8,
                    std::mem::size_of::<ExtmarkSplice>(),
                )
            } else {
                (
                    std::ptr::addr_of!(obj.data.mover) as *const u8,
                    std::mem::size_of::<ExtmarkMove>(),
                )
            };
            if !undo_write(fp, ptr, size) {
                return false;
            }
        }
    }

    // Write extmark end magic
    if !undo_write_2(fp, UF_ENTRY_END_MAGIC) {
        return false;
    }

    true
}

/// Serialize the undo file header.
///
/// Writes magic bytes, version, hash, buffer info, and undo tree header data.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `buf` must be a valid buffer handle
/// - `hash` must point to UNDO_HASH_SIZE bytes
#[allow(dead_code)]
unsafe fn serialize_header(fp: FileHandle, buf: BufHandle, hash: *const u8) -> bool {
    // Write start magic
    if !undo_write(fp, UF_START_MAGIC.as_ptr(), UF_START_MAGIC_LEN) {
        return false;
    }

    // Write version
    if !undo_write_2(fp, UF_VERSION) {
        return false;
    }

    // Write hash
    if !undo_write(fp, hash, UNDO_HASH_SIZE) {
        return false;
    }

    // Write buffer line count
    let line_count = nvim_buf_get_b_ml_line_count(buf);
    if !undo_write_4(fp, line_count as i32) {
        return false;
    }

    // Write b_u_line_ptr
    let line_ptr = nvim_buf_get_b_u_line_ptr(buf);
    let line_len = if line_ptr.is_null() {
        0
    } else {
        libc::strlen(line_ptr)
    };
    if !undo_write_bytes(fp, line_len as u64, 4) {
        return false;
    }
    if line_len > 0 && !undo_write(fp, line_ptr as *const u8, line_len) {
        return false;
    }

    // Write b_u_line_lnum and b_u_line_colnr
    if !undo_write_4(fp, nvim_buf_get_b_u_line_lnum(buf) as i32) {
        return false;
    }
    if !undo_write_4(fp, nvim_buf_get_b_u_line_colnr(buf)) {
        return false;
    }

    // Write undo tree header pointers
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    let newhead = nvim_buf_get_b_u_newhead(buf);
    let curhead = nvim_buf_get_b_u_curhead(buf);

    if !put_header_ptr(fp, oldhead) {
        return false;
    }
    if !put_header_ptr(fp, newhead) {
        return false;
    }
    if !put_header_ptr(fp, curhead) {
        return false;
    }

    // Write undo tree state
    if !undo_write_4(fp, nvim_buf_get_b_u_numhead(buf)) {
        return false;
    }
    if !undo_write_4(fp, nvim_buf_get_b_u_seq_last(buf)) {
        return false;
    }
    if !undo_write_4(fp, nvim_buf_get_b_u_seq_cur(buf)) {
        return false;
    }

    // Write time
    if !undo_write_time(fp, nvim_buf_get_b_u_time_cur(buf)) {
        return false;
    }

    // Write optional fields - last save nr
    if !undo_write_bytes(fp, 4, 1) {
        // length
        return false;
    }
    if !undo_write_bytes(fp, u64::from(UF_LAST_SAVE_NR), 1) {
        // field id
        return false;
    }
    if !undo_write_4(fp, nvim_buf_get_b_u_save_nr_last(buf)) {
        return false;
    }

    // Write end marker for optional fields
    if !undo_write_bytes(fp, 0, 1) {
        return false;
    }

    true
}

/// FFI export: Serialize undo entry.
///
/// # Safety
///
/// All handles must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_serialize_uep(fp: FileHandle, uep: UEntryHandle) -> bool {
    serialize_uep(fp, uep)
}

/// FFI export: Serialize undo header.
///
/// # Safety
///
/// All handles must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_serialize_uhp(fp: FileHandle, uhp: UHeaderHandle) -> bool {
    serialize_uhp(fp, uhp)
}

/// FFI export: Serialize file header.
///
/// # Safety
///
/// All handles must be valid, hash must point to UNDO_HASH_SIZE bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_serialize_header(
    fp: FileHandle,
    buf: BufHandle,
    hash: *const u8,
) -> bool {
    serialize_header(fp, buf, hash)
}

// =============================================================================
// Hash computation for undo files
// =============================================================================

/// Compute SHA-256 hash of all buffer lines.
///
/// This is the Rust implementation of `u_compute_hash`.
/// Each line is hashed including the null terminator.
///
/// # Safety
///
/// `buf` must be a valid buffer handle. `hash` must point to at least 32 bytes.
#[export_name = "u_compute_hash"]
pub unsafe extern "C" fn rs_u_compute_hash(buf: BufHandle, hash: *mut u8) {
    if hash.is_null() {
        return;
    }

    let mut ctx = Sha256Context::new();
    let line_count = nvim_buf_get_ml_line_count(buf);

    for lnum in 1..=line_count {
        let line = nvim_ml_get_buf_line(buf, lnum);
        if !line.is_null() {
            // Get line as a C string, then hash it including the null terminator
            let c_str = CStr::from_ptr(line);
            let bytes = c_str.to_bytes_with_nul();
            ctx.update(bytes);
        }
    }

    let digest = ctx.finish();
    std::ptr::copy_nonoverlapping(digest.as_ptr(), hash, 32);
}

// =============================================================================
// Phase 1: Complete Undo File Write Implementation
// =============================================================================

// Additional FFI declarations for u_write_undo
extern "C" {
    fn nvim_undo_cannot_write_no_dir();
    fn nvim_undo_will_not_overwrite_cannot_read(file_name: *const c_char);
    fn nvim_undo_will_not_overwrite_not_undo(file_name: *const c_char);
    fn nvim_undo_skip_write_nothing();
    fn nvim_undo_write_error(file_name: *const c_char);
    fn nvim_undo_writing(file_name: *const c_char);
}

/// O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW flags for open()
const O_CREAT: c_int = 0o100;
const O_WRONLY: c_int = 0o1;
const O_EXCL: c_int = 0o200;
const O_NOFOLLOW: c_int = 0o400000;
const O_RDONLY: c_int = 0o0;

/// Write the undo tree to an undo file.
///
/// This is the Rust implementation of `u_write_undo`.
///
/// # Arguments
///
/// * `name` - Name of the undo file or NULL to generate from buffer name
/// * `forceit` - True for `:wundo!`, false otherwise
/// * `buf` - Buffer for which undo file is written
/// * `hash` - Hash value of the buffer text (UNDO_HASH_SIZE bytes)
///
/// # Safety
///
/// All handles must be valid. `hash` must point to UNDO_HASH_SIZE bytes.
#[export_name = "u_write_undo"]
pub unsafe extern "C" fn rs_u_write_undo(
    name: *const c_char,
    forceit: bool,
    buf: BufHandle,
    hash: *const u8,
) {
    let file_name: *mut c_char;
    let mut write_ok = false;

    // Get the undo file name
    if name.is_null() {
        let ffname = nvim_buf_get_b_ffname(buf);
        file_name = u_get_undo_file_name(ffname, false);
        if file_name.is_null() {
            if nvim_get_p_verbose() > 0 {
                nvim_undo_verbose_enter();
                nvim_undo_cannot_write_no_dir();
                nvim_undo_verbose_leave();
            }
            return;
        }
    } else {
        file_name = name as *mut c_char;
    }

    // Decide about the permission to use for the undo file. If the buffer
    // has a name use the permission of the original file. Otherwise only
    // allow the user to access the undo file.
    let ffname = nvim_buf_get_b_ffname(buf);
    let mut perm: c_int = 0o600;
    if !ffname.is_null() {
        perm = nvim_os_getperm(ffname);
        if perm < 0 {
            perm = 0o600;
        }
    }

    // Strip any sticky and executable bits.
    perm &= 0o666;

    // If the undo file already exists, verify that it actually is an undo
    // file, and delete it.
    if nvim_os_path_exists(file_name) {
        if name.is_null() || !forceit {
            // Check we can read it and it's an undo file.
            let fd = nvim_os_open(file_name, O_RDONLY, 0);
            if fd < 0 {
                if !name.is_null() || nvim_get_p_verbose() > 0 {
                    if name.is_null() {
                        nvim_undo_verbose_enter();
                    }
                    nvim_undo_will_not_overwrite_cannot_read(file_name);
                    if name.is_null() {
                        nvim_undo_verbose_leave();
                    }
                }
                if name.is_null() {
                    nvim_xfree(file_name as *mut c_void);
                }
                return;
            }

            let mut mbuf = [0u8; UF_START_MAGIC_LEN];
            let len = nvim_read_eintr(fd, mbuf.as_mut_ptr() as *mut c_void, UF_START_MAGIC_LEN);
            nvim_os_close(fd);

            if len < UF_START_MAGIC_LEN as isize || mbuf != *UF_START_MAGIC {
                if !name.is_null() || nvim_get_p_verbose() > 0 {
                    if name.is_null() {
                        nvim_undo_verbose_enter();
                    }
                    nvim_undo_will_not_overwrite_not_undo(file_name);
                    if name.is_null() {
                        nvim_undo_verbose_leave();
                    }
                }
                if name.is_null() {
                    nvim_xfree(file_name as *mut c_void);
                }
                return;
            }
        }
        nvim_os_remove(file_name);
    }

    // If there is no undo information at all, quit here after deleting any
    // existing undo file.
    let numhead = nvim_buf_get_b_u_numhead(buf);
    let line_ptr = nvim_buf_get_b_u_line_ptr(buf);
    if numhead == 0 && line_ptr.is_null() {
        if nvim_get_p_verbose() > 0 {
            nvim_undo_verbose_enter();
            nvim_undo_skip_write_nothing();
            nvim_undo_verbose_leave();
        }
        if name.is_null() {
            nvim_xfree(file_name as *mut c_void);
        }
        return;
    }

    // Create the undo file
    let fd = nvim_os_open(file_name, O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW, perm);
    if fd < 0 {
        nvim_undo_semsg(
            c"E828: Cannot open undo file for writing: %s".as_ptr(),
            file_name,
        );
        if name.is_null() {
            nvim_xfree(file_name as *mut c_void);
        }
        return;
    }
    nvim_os_setperm(file_name, perm);

    if nvim_get_p_verbose() > 0 {
        nvim_undo_verbose_enter();
        nvim_undo_writing(file_name);
        nvim_undo_verbose_leave();
    }

    // Try to set the group of the undo file same as the original file
    if !ffname.is_null() {
        let new_perm = nvim_undo_set_file_group(fd, ffname, file_name, perm);
        if new_perm != perm {
            nvim_os_setperm(file_name, new_perm);
        }
    }

    let fp = nvim_fdopen(fd, c"w".as_ptr());
    if fp.is_null() {
        nvim_undo_semsg(
            c"E828: Cannot open undo file for writing: %s".as_ptr(),
            file_name,
        );
        nvim_os_close(fd);
        nvim_os_remove(file_name);
        if name.is_null() {
            nvim_xfree(file_name as *mut c_void);
        }
        return;
    }

    // Undo must be synced.
    rs_u_sync(true);

    // Write the header.
    if !serialize_header(fp, buf, hash) {
        nvim_undo_fclose(fp);
        nvim_undo_write_error(file_name);
        if name.is_null() {
            nvim_xfree(file_name as *mut c_void);
        }
        return;
    }

    // Iteratively serialize UHPs and their UEPs from the top down.
    let mark = nvim_inc_lastmark();
    let mut uhp = nvim_buf_get_b_u_oldhead(buf);
    while !uhp.is_null() {
        // Serialize current UHP if we haven't seen it
        if (*uhp).uh_walk != mark {
            (*uhp).uh_walk = mark;
            if !serialize_uhp(fp, uhp) {
                nvim_undo_fclose(fp);
                nvim_undo_write_error(file_name);
                if name.is_null() {
                    nvim_xfree(file_name as *mut c_void);
                }
                return;
            }
        }

        // Now walk through the tree - algorithm from undo_time().
        let prev = (*uhp).uh_prev.ptr;
        if !prev.is_null() && (*prev).uh_walk != mark {
            uhp = prev;
        } else {
            let alt_next = (*uhp).uh_alt_next.ptr;
            if !alt_next.is_null() && (*alt_next).uh_walk != mark {
                uhp = alt_next;
            } else {
                let next = (*uhp).uh_next.ptr;
                let alt_prev = (*uhp).uh_alt_prev.ptr;
                if !next.is_null() && alt_prev.is_null() && (*next).uh_walk != mark {
                    uhp = next;
                } else if !alt_prev.is_null() {
                    uhp = alt_prev;
                } else {
                    uhp = (*uhp).uh_next.ptr;
                }
            }
        }
    }

    // Write end magic
    if undo_write_2(fp, UF_HEADER_END_MAGIC) {
        write_ok = true;
    }

    // Fsync if p_fs is set
    if nvim_get_p_fs() && nvim_undo_fflush(fp) == 0 && nvim_os_fsync(nvim_fileno(fp)) != 0 {
        write_ok = false;
    }

    nvim_undo_fclose(fp);

    if !write_ok {
        nvim_undo_write_error(file_name);
    }

    // Copy ACL from original file
    if !ffname.is_null() {
        let acl = nvim_os_get_acl(ffname);
        nvim_os_set_acl(file_name, acl);
        nvim_os_free_acl(acl);
    }

    if name.is_null() {
        nvim_xfree(file_name as *mut c_void);
    }
}

// =============================================================================
// Phase 2: Complete Undo File Read Implementation
// =============================================================================

// Additional FFI declarations for u_read_undo
extern "C" {
    fn nvim_undo_reading(file_name: *const c_char);
    fn nvim_undo_not_reading_owner_differs(file_name: *const c_char);
    fn nvim_undo_cannot_open_for_reading(file_name: *const c_char);
    fn nvim_undo_not_undo_file(file_name: *const c_char);
    fn nvim_undo_incompatible_version(file_name: *const c_char);
    fn nvim_undo_corruption_error(what: *const c_char, file_name: *const c_char);
    fn nvim_undo_file_changed_warning();
    fn nvim_undo_finished_reading(file_name: *const c_char);
    #[link_name = "os_fopen"]
    fn nvim_os_fopen(path: *const c_char, mode: *const c_char) -> FileHandle;
    fn nvim_undo_check_owner(orig_name: *const c_char, file_name: *const c_char) -> bool;

}

// =============================================================================
// Deserialization Functions (migrated from C unserialize_*)
// =============================================================================

/// Deserialize a pos_T from the undo file.
/// Returns (lnum, col, coladd), each clamped to >= 0.
///
/// # Safety
///
/// - `fp` must be a valid file handle
unsafe fn unserialize_pos(fp: FileHandle) -> (LinenrT, ColnrT, ColnrT) {
    let lnum = undo_read_4c(fp).max(0);
    let col = undo_read_4c(fp).max(0);
    let coladd = undo_read_4c(fp).max(0);
    (lnum, col, coladd)
}

/// Deserialize visual info from the undo file into the undo header.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uhp` must be a valid undo header handle
unsafe fn unserialize_visualinfo(fp: FileHandle, uhp: UHeaderHandle) {
    let (start_lnum, start_col, start_coladd) = unserialize_pos(fp);
    let (end_lnum, end_col, end_coladd) = unserialize_pos(fp);
    let mode = undo_read_4c(fp);
    let curswant = undo_read_4c(fp);
    (*uhp).uh_visual.vi_start.lnum = start_lnum;
    (*uhp).uh_visual.vi_start.col = start_col;
    (*uhp).uh_visual.vi_start.coladd = start_coladd;
    (*uhp).uh_visual.vi_end.lnum = end_lnum;
    (*uhp).uh_visual.vi_end.col = end_col;
    (*uhp).uh_visual.vi_end.coladd = end_coladd;
    (*uhp).uh_visual.vi_mode = mode;
    (*uhp).uh_visual.vi_curswant = curswant;
}

/// Deserialize an extmark undo object from the undo file.
/// Returns true on success, false on error.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uhp` must be a valid undo header handle
unsafe fn kv_push_extmark(uhp: *mut UHeader, obj: ExtmarkUndoObject) {
    let kv = &mut (*uhp).uh_extmark;
    if kv.size == kv.capacity {
        // kv_resize_full: double capacity (or start with 8)
        let new_cap = if kv.capacity == 0 { 8 } else { kv.capacity * 2 };
        kv.items = xrealloc(
            kv.items as *mut c_void,
            new_cap * std::mem::size_of::<ExtmarkUndoObject>(),
        ) as *mut ExtmarkUndoObject;
        kv.capacity = new_cap;
    }
    *kv.items.add(kv.size) = obj;
    kv.size += 1;
}

unsafe fn unserialize_extmark(fp: FileHandle, uhp: UHeaderHandle) -> bool {
    let ext_type = undo_read_4c(fp);

    // kExtmarkSplice = 0
    if ext_type == K_EXTMARK_SPLICE {
        let size = std::mem::size_of::<ExtmarkSplice>();
        let mut splice = std::mem::MaybeUninit::<ExtmarkSplice>::uninit();
        if !undo_read(fp, splice.as_mut_ptr() as *mut u8, size) {
            return false;
        }
        kv_push_extmark(
            uhp,
            ExtmarkUndoObject {
                kind: K_EXTMARK_SPLICE,
                data: ExtmarkUndoObjectData {
                    splice: splice.assume_init(),
                },
            },
        );
        return true;
    }

    // kExtmarkMove = 1
    if ext_type == K_EXTMARK_MOVE {
        let size = std::mem::size_of::<ExtmarkMove>();
        let mut mover = std::mem::MaybeUninit::<ExtmarkMove>::uninit();
        if !undo_read(fp, mover.as_mut_ptr() as *mut u8, size) {
            return false;
        }
        kv_push_extmark(
            uhp,
            ExtmarkUndoObject {
                kind: K_EXTMARK_MOVE,
                data: ExtmarkUndoObjectData {
                    mover: mover.assume_init(),
                },
            },
        );
        return true;
    }

    // Unknown type
    false
}

/// Deserialize a u_entry_T from the undo file.
/// Returns the new entry handle and sets `error` to true on failure.
///
/// # Safety
///
/// - `fp` must be a valid file handle
unsafe fn unserialize_uep(fp: FileHandle, file_name: *const c_char) -> (UEntryHandle, bool) {
    let uep = nvim_xcalloc(1, std::mem::size_of::<UEntry>()) as *mut UEntry;
    // Fields are zero-initialized by xcalloc

    (*uep).ue_top = undo_read_4c(fp);
    (*uep).ue_bot = undo_read_4c(fp);
    (*uep).ue_lcount = undo_read_4c(fp);
    let size = undo_read_4c(fp);
    (*uep).ue_size = size;

    if size > 0 {
        let ptr_size = std::mem::size_of::<*mut c_char>();
        if (size as usize) < usize::MAX / ptr_size {
            let array = nvim_xmallocz(ptr_size * size as usize) as *mut *mut c_char;
            // Zero out the array
            ptr::write_bytes(array, 0, size as usize);
            (*uep).ue_array = array;

            for i in 0..size as usize {
                let line_len = undo_read_4c(fp);
                if line_len >= 0 {
                    let line = undo_read_string(fp, line_len as usize);
                    if line.is_null() {
                        return (uep, true); // error
                    }
                    *array.add(i) = line;
                } else {
                    nvim_undo_corruption_error(c"line length".as_ptr(), file_name);
                    return (uep, true); // error
                }
            }
        }
    }

    (uep, false)
}

/// Deserialize a u_header_T from the undo file.
/// Returns null handle on error.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `file_name` must be a valid C string
unsafe fn unserialize_uhp(fp: FileHandle, file_name: *const c_char) -> UHeaderHandle {
    let uhp = nvim_xcalloc(1, std::mem::size_of::<UHeader>()) as *mut UHeader;
    // Fields are zero-initialized by xcalloc

    // Read sequence numbers for pointer swizzling
    (*uhp).uh_next.seq = undo_read_4c(fp);
    (*uhp).uh_prev.seq = undo_read_4c(fp);
    (*uhp).uh_alt_next.seq = undo_read_4c(fp);
    (*uhp).uh_alt_prev.seq = undo_read_4c(fp);

    let seq = undo_read_4c(fp);
    (*uhp).uh_seq = seq;
    if seq <= 0 {
        nvim_undo_corruption_error(c"uh_seq".as_ptr(), file_name);
        nvim_xfree(uhp as *mut c_void);
        return ptr::null_mut();
    }

    // Read cursor position
    let (cursor_lnum, cursor_col, cursor_coladd) = unserialize_pos(fp);
    (*uhp).uh_cursor.lnum = cursor_lnum;
    (*uhp).uh_cursor.col = cursor_col;
    (*uhp).uh_cursor.coladd = cursor_coladd;
    (*uhp).uh_cursor_vcol = undo_read_4c(fp);
    (*uhp).uh_flags = undo_read_2c(fp);

    // Read named marks
    let cur_timestamp = nvim_undo_os_time() as u64;
    for i in 0..NMARKS as c_int {
        let (lnum, col, coladd) = unserialize_pos(fp);
        let idx = i as usize;
        (*uhp).uh_namedm[idx].mark.lnum = lnum;
        (*uhp).uh_namedm[idx].mark.col = col;
        (*uhp).uh_namedm[idx].mark.coladd = coladd;
        (*uhp).uh_namedm[idx].timestamp = cur_timestamp;
        (*uhp).uh_namedm[idx].fnum = 0;
    }

    // Read visual info
    unserialize_visualinfo(fp, uhp);

    // Read time
    (*uhp).uh_time = undo_read_time(fp);

    // Read optional fields
    loop {
        let len = undo_read_byte(fp);
        if len == -1 {
            // EOF
            nvim_undo_corruption_error(c"truncated".as_ptr(), file_name);
            rs_u_free_uhp(uhp);
            return ptr::null_mut();
        }
        if len == 0 {
            break;
        }
        let what = undo_read_byte(fp);
        if what == UHP_SAVE_NR as c_int {
            (*uhp).uh_save_nr = undo_read_4c(fp);
        } else {
            // Skip unknown field
            let mut remaining = len;
            while remaining > 0 {
                remaining -= 1;
                undo_read_byte(fp);
            }
        }
    }

    // Read the uep list
    let mut last_uep: *mut UEntry = ptr::null_mut();
    loop {
        let c = undo_read_2c(fp);
        if c != UF_ENTRY_MAGIC as c_int {
            if c != UF_ENTRY_END_MAGIC as c_int {
                nvim_undo_corruption_error(c"entry end".as_ptr(), file_name);
                rs_u_free_uhp(uhp);
                return ptr::null_mut();
            }
            break;
        }
        let (uep, error) = unserialize_uep(fp, file_name);
        if last_uep.is_null() {
            (*uhp).uh_entry = uep;
        } else {
            (*last_uep).ue_next = uep;
        }
        last_uep = uep;
        if uep.is_null() || error {
            rs_u_free_uhp(uhp);
            return ptr::null_mut();
        }
    }

    // Read extmark undo information
    (*uhp).uh_extmark.items = std::ptr::null_mut();
    (*uhp).uh_extmark.size = 0;
    (*uhp).uh_extmark.capacity = 0;
    loop {
        let c = undo_read_2c(fp);
        if c != UF_ENTRY_MAGIC as c_int {
            if c != UF_ENTRY_END_MAGIC as c_int {
                nvim_undo_corruption_error(c"entry end".as_ptr(), file_name);
                nvim_xfree((*uhp).uh_extmark.items as *mut c_void);
                (*uhp).uh_extmark.items = std::ptr::null_mut();
                (*uhp).uh_extmark.size = 0;
                (*uhp).uh_extmark.capacity = 0;
                rs_u_free_uhp(uhp);
                return ptr::null_mut();
            }
            break;
        }
        if !unserialize_extmark(fp, uhp) {
            nvim_xfree((*uhp).uh_extmark.items as *mut c_void);
            (*uhp).uh_extmark.items = std::ptr::null_mut();
            (*uhp).uh_extmark.size = 0;
            (*uhp).uh_extmark.capacity = 0;
            rs_u_free_uhp(uhp);
            return ptr::null_mut();
        }
    }

    uhp
}

/// Read the undo tree from an undo file.
///
/// This is the Rust implementation of `u_read_undo`.
///
/// # Arguments
///
/// * `name` - Name of the undo file or NULL to generate from curbuf
/// * `hash` - Hash value of the buffer text (UNDO_HASH_SIZE bytes)
/// * `orig_name` - Original file name (for owner check on Unix)
///
/// # Safety
///
/// All handles must be valid. `hash` must point to UNDO_HASH_SIZE bytes.
#[export_name = "u_read_undo"]
pub unsafe extern "C" fn rs_u_read_undo(
    name: *const c_char,
    hash: *const u8,
    orig_name: *const c_char,
) {
    let buf = nvim_get_curbuf();
    let file_name: *mut c_char;
    let mut uhp_table: *mut UHeaderHandle = ptr::null_mut();
    let mut line_ptr: *mut c_char = ptr::null_mut();
    let mut num_read_uhps: c_int = 0;

    // Get the undo file name
    if name.is_null() {
        let ffname = nvim_buf_get_b_ffname(buf);
        file_name = u_get_undo_file_name(ffname, true);
        if file_name.is_null() {
            return;
        }

        // For safety we only read an undo file if the owner is equal to the
        // owner of the text file or equal to the current user.
        if !nvim_undo_check_owner(orig_name, file_name) {
            if nvim_get_p_verbose() > 0 {
                nvim_undo_verbose_enter();
                nvim_undo_not_reading_owner_differs(file_name);
                nvim_undo_verbose_leave();
            }
            nvim_xfree(file_name as *mut c_void);
            return;
        }
    } else {
        file_name = name as *mut c_char;
    }

    if nvim_get_p_verbose() > 0 {
        nvim_undo_verbose_enter();
        nvim_undo_reading(file_name);
        nvim_undo_verbose_leave();
    }

    let fp = nvim_os_fopen(file_name, c"r".as_ptr());
    if fp.is_null() {
        if !name.is_null() || nvim_get_p_verbose() > 0 {
            nvim_undo_cannot_open_for_reading(file_name);
        }
        // goto error
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    // Read the undo file header.
    let mut magic_buf = [0u8; UF_START_MAGIC_LEN];
    if nvim_undo_fread(
        magic_buf.as_mut_ptr() as *mut c_void,
        UF_START_MAGIC_LEN,
        1,
        fp,
    ) != 1
        || magic_buf != *UF_START_MAGIC
    {
        nvim_undo_not_undo_file(file_name);
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    let version = undo_read_2c(fp);
    if version != UF_VERSION as c_int {
        nvim_undo_incompatible_version(file_name);
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    // Read hash
    let mut read_hash = [0u8; UNDO_HASH_SIZE];
    if !undo_read(fp, read_hash.as_mut_ptr(), UNDO_HASH_SIZE) {
        nvim_undo_corruption_error(c"hash".as_ptr(), file_name);
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    let line_count = undo_read_4c(fp) as LinenrT;
    let buf_line_count = nvim_buf_get_ml_line_count(buf);

    // Compare hashes
    let hash_slice = std::slice::from_raw_parts(hash, UNDO_HASH_SIZE);
    if read_hash != *hash_slice || line_count != buf_line_count {
        if nvim_get_p_verbose() > 0 || !name.is_null() {
            if name.is_null() {
                nvim_undo_verbose_enter();
            }
            nvim_undo_file_changed_warning();
            if name.is_null() {
                nvim_undo_verbose_leave();
            }
        }
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    // Read undo data for "U" command.
    let str_len = undo_read_4c(fp);
    if str_len < 0 {
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    if str_len > 0 {
        line_ptr = undo_read_string(fp, str_len as usize);
    }

    let line_lnum = undo_read_4c(fp) as LinenrT;
    let line_colnr = undo_read_4c(fp) as ColnrT;
    if line_lnum < 0 || line_colnr < 0 {
        nvim_undo_corruption_error(c"line lnum/col".as_ptr(), file_name);
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    // Begin general undo data
    let old_header_seq = undo_read_4c(fp);
    let new_header_seq = undo_read_4c(fp);
    let cur_header_seq = undo_read_4c(fp);
    let num_head = undo_read_4c(fp);
    let seq_last = undo_read_4c(fp);
    let seq_cur = undo_read_4c(fp);
    let seq_time = undo_read_time(fp);

    // Optional header fields.
    let mut last_save_nr: c_int = 0;
    loop {
        let len = undo_read_byte(fp);
        if len == 0 || len == -1 {
            break;
        }
        let what = undo_read_byte(fp);
        match what as u8 {
            UF_LAST_SAVE_NR => {
                last_save_nr = undo_read_4c(fp);
            }
            _ => {
                // field not supported, skip
                for _ in 0..len {
                    undo_read_byte(fp);
                }
            }
        }
    }

    // Allocate uhp_table to store the freshly created undo headers
    if num_head > 0 {
        let size = (num_head as usize).saturating_mul(std::mem::size_of::<UHeaderHandle>());
        if size < usize::MAX {
            uhp_table = nvim_xmallocz(size) as *mut UHeaderHandle;
        }
    }

    // Read all undo headers
    loop {
        let c = undo_read_2c(fp);
        if c != UF_HEADER_MAGIC as c_int {
            if c != UF_HEADER_END_MAGIC as c_int {
                nvim_undo_corruption_error(c"end marker".as_ptr(), file_name);
                cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
                return;
            }
            break;
        }

        if num_read_uhps >= num_head {
            nvim_undo_corruption_error(c"num_head too small".as_ptr(), file_name);
            cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
            return;
        }

        let uhp = unserialize_uhp(fp, file_name);
        if uhp.is_null() {
            cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
            return;
        }
        *uhp_table.add(num_read_uhps as usize) = uhp;
        num_read_uhps += 1;
    }

    if num_read_uhps != num_head {
        nvim_undo_corruption_error(c"num_head".as_ptr(), file_name);
        cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
        return;
    }

    // Swizzle sequence numbers into pointers
    let mut old_idx: i16 = -1;
    let mut new_idx: i16 = -1;
    let mut cur_idx: i16 = -1;

    for i in 0..num_head {
        let uhp = *uhp_table.add(i as usize);
        if uhp.is_null() {
            continue;
        }

        // Check for duplicate sequence numbers
        let this_seq = (*uhp).uh_seq;
        for j in 0..num_head {
            if i != j {
                let other = *uhp_table.add(j as usize);
                if !other.is_null() && (*other).uh_seq == this_seq {
                    nvim_undo_corruption_error(c"duplicate uh_seq".as_ptr(), file_name);
                    cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
                    return;
                }
            }
        }

        // Swizzle uh_next
        let next_seq = (*uhp).uh_next.seq;
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.is_null() && (*candidate).uh_seq == next_seq {
                (*uhp).uh_next.ptr = candidate;
                break;
            }
        }

        // Swizzle uh_prev
        let prev_seq = (*uhp).uh_prev.seq;
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.is_null() && (*candidate).uh_seq == prev_seq {
                (*uhp).uh_prev.ptr = candidate;
                break;
            }
        }

        // Swizzle uh_alt_next
        let alt_next_seq = (*uhp).uh_alt_next.seq;
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.is_null() && (*candidate).uh_seq == alt_next_seq {
                (*uhp).uh_alt_next.ptr = candidate;
                break;
            }
        }

        // Swizzle uh_alt_prev
        let alt_prev_seq = (*uhp).uh_alt_prev.seq;
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.is_null() && (*candidate).uh_seq == alt_prev_seq {
                (*uhp).uh_alt_prev.ptr = candidate;
                break;
            }
        }

        // Find indices for old, new, cur headers
        if old_header_seq > 0 && old_idx < 0 && this_seq == old_header_seq {
            old_idx = i as i16;
        }
        if new_header_seq > 0 && new_idx < 0 && this_seq == new_header_seq {
            new_idx = i as i16;
        }
        if cur_header_seq > 0 && cur_idx < 0 && this_seq == cur_header_seq {
            cur_idx = i as i16;
        }
    }

    // Now that we have read the undo info successfully, free the current undo
    // info and use the info from the file.
    rs_u_blockfree(buf);

    let oldhead = if old_idx < 0 {
        ptr::null_mut()
    } else {
        *uhp_table.add(old_idx as usize)
    };
    let newhead = if new_idx < 0 {
        ptr::null_mut()
    } else {
        *uhp_table.add(new_idx as usize)
    };
    let curhead = if cur_idx < 0 {
        ptr::null_mut()
    } else {
        *uhp_table.add(cur_idx as usize)
    };

    nvim_buf_set_b_u_oldhead(buf, oldhead);
    nvim_buf_set_b_u_newhead(buf, newhead);
    nvim_buf_set_b_u_curhead(buf, curhead);
    nvim_buf_set_b_u_line_ptr(buf, line_ptr);
    nvim_buf_set_b_u_line_lnum(buf, line_lnum);
    nvim_buf_set_b_u_line_colnr(buf, line_colnr);
    nvim_buf_set_b_u_numhead(buf, num_head);
    nvim_buf_set_b_u_seq_last(buf, seq_last);
    nvim_buf_set_b_u_seq_cur(buf, seq_cur);
    nvim_buf_set_b_u_time_cur(buf, seq_time);
    nvim_buf_set_b_u_save_nr_last(buf, last_save_nr);
    nvim_buf_set_b_u_save_nr_cur(buf, last_save_nr);
    nvim_buf_set_b_u_synced(buf, true);

    nvim_xfree(uhp_table as *mut c_void);

    if !name.is_null() {
        nvim_undo_finished_reading(file_name);
    }

    nvim_undo_fclose(fp);
    if name.is_null() {
        nvim_xfree(file_name as *mut c_void);
    }
}

/// Cleanup helper for read_undo error paths.
unsafe fn cleanup_read_error(
    name: *const c_char,
    file_name: *mut c_char,
    fp: FileHandle,
    line_ptr: *mut c_char,
    uhp_table: *mut UHeaderHandle,
    num_read_uhps: c_int,
) {
    nvim_xfree(line_ptr as *mut c_void);
    if !uhp_table.is_null() {
        for i in 0..num_read_uhps {
            let uhp = *uhp_table.add(i as usize);
            if !uhp.is_null() {
                rs_u_free_uhp(uhp);
            }
        }
        nvim_xfree(uhp_table as *mut c_void);
    }
    if !fp.is_null() {
        nvim_undo_fclose(fp);
    }
    if name.is_null() && !file_name.is_null() {
        nvim_xfree(file_name as *mut c_void);
    }
}

// =============================================================================
// Phase 4: Ex Command Migration - ex_undolist
// =============================================================================

// GArray handle for Rust FFI
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct GArrayHandle(*mut c_void);

// Additional FFI declarations for ex_undolist
extern "C" {
    fn nvim_undo_msg_simple(msg: *const c_char);
    #[link_name = "msg_start"]
    fn nvim_msg_start();
    fn nvim_msg_end();
    fn nvim_undo_msg_puts_hl_title(msg: *const c_char);
    #[link_name = "msg_putchar"]
    fn nvim_undo_msg_putchar(c: c_int);
    #[link_name = "msg_puts"]
    fn nvim_undo_msg_puts(msg: *const c_char);
    #[link_name = "xstrdup"]
    fn nvim_undo_xstrdup(s: *const c_char) -> *mut c_char;
}

/// opaque handle for exarg_T
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ExargHandle(*mut c_void);

/// ":undolist": List the leafs of the undo tree
///
/// # Safety
///
/// `eap` must be a valid ExargT handle.
#[export_name = "ex_undolist"]
pub unsafe extern "C" fn rs_ex_undolist(_eap: ExargHandle) {
    let buf = nvim_get_curbuf();
    let mut changes: c_int = 1;

    // 1: walk the tree to find all leafs, put the info in "ga".
    // 2: sort the lines
    // 3: display the list
    let mark = nvim_inc_lastmark();
    let nomark = nvim_inc_lastmark();

    // We'll collect strings directly in a Vec and then sort and display
    let mut entries: Vec<(*mut c_char, c_int)> = Vec::new();

    let mut uhp = nvim_buf_get_b_u_oldhead(buf);
    while !uhp.is_null() {
        let prev = (*uhp).uh_prev.ptr;

        if prev.is_null() && (*uhp).uh_walk != nomark && (*uhp).uh_walk != mark {
            // Format the entry (inlined from nvim_undolist_format_entry)
            let mut entry_buf: [c_char; 256] = [0; 256];
            let buf_ptr = entry_buf.as_mut_ptr();
            let buf_size: usize = 256;
            libc::snprintf(
                buf_ptr,
                buf_size,
                c"%6d %7d  ".as_ptr(),
                (*uhp).uh_seq,
                changes,
            );
            let cur_len = libc::strlen(buf_ptr);
            rs_undo_fmt_time(buf_ptr.add(cur_len), buf_size - cur_len, (*uhp).uh_time);
            if (*uhp).uh_save_nr > 0 {
                // Pad to length 33
                while libc::strlen(buf_ptr) < 33 {
                    let pos = libc::strlen(buf_ptr);
                    if pos + 1 >= buf_size {
                        break;
                    }
                    *buf_ptr.add(pos) = b' ' as c_char;
                    *buf_ptr.add(pos + 1) = 0;
                }
                let cur_len = libc::strlen(buf_ptr);
                libc::snprintf(
                    buf_ptr.add(cur_len),
                    buf_size - cur_len,
                    c"  %3d".as_ptr(),
                    (*uhp).uh_save_nr,
                );
            }
            let entry_str = nvim_undo_xstrdup(entry_buf.as_ptr());
            let seq = (*uhp).uh_seq;
            entries.push((entry_str, seq));
        }

        (*uhp).uh_walk = mark;

        // go down in the tree if we haven't been there
        if !prev.is_null() && (*prev).uh_walk != nomark && (*prev).uh_walk != mark {
            uhp = prev;
            changes += 1;
        } else {
            let alt_next = (*uhp).uh_alt_next.ptr;
            if !alt_next.is_null() && (*alt_next).uh_walk != nomark && (*alt_next).uh_walk != mark {
                // go to alternate branch if we haven't been there
                uhp = alt_next;
            } else {
                let next = (*uhp).uh_next.ptr;
                let alt_prev = (*uhp).uh_alt_prev.ptr;
                if !next.is_null()
                    && alt_prev.is_null()
                    && (*next).uh_walk != nomark
                    && (*next).uh_walk != mark
                {
                    // go up in the tree if we haven't been there and we are at the
                    // start of alternate branches
                    uhp = next;
                    changes -= 1;
                } else {
                    // need to backtrack; mark this node as done
                    (*uhp).uh_walk = nomark;
                    if !alt_prev.is_null() {
                        uhp = alt_prev;
                    } else {
                        uhp = next;
                        changes -= 1;
                    }
                }
            }
        }
    }

    if entries.is_empty() {
        nvim_undo_msg_simple(c"Nothing to undo".as_ptr());
    } else {
        // Sort by sequence number
        entries.sort_by_key(|e| e.1);

        nvim_msg_start();
        nvim_undo_msg_puts_hl_title(c"number changes  when               saved".as_ptr());
        for (entry, _seq) in &entries {
            if nvim_undo_got_int() {
                break;
            }
            nvim_undo_msg_putchar(b'\n' as c_int);
            nvim_undo_msg_puts(*entry);
        }
        nvim_msg_end();

        // Free all the strings
        for (entry, _) in entries {
            nvim_xfree(entry as *mut c_void);
        }
    }
}

// ============================================================================
// Phase 5: VimL Function Migration (undofile, undotree)
// ============================================================================

/// Opaque handle for list_T
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ListHandle(*mut c_void);

impl ListHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        ListHandle(std::ptr::null_mut())
    }

    /// Check if this handle is null.
    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle for dict_T
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct DictHandle(*mut c_void);

impl DictHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        DictHandle(std::ptr::null_mut())
    }
}

// FFI declarations for typval construction
extern "C" {
    fn nvim_tv_list_alloc() -> ListHandle;
    #[link_name = "tv_dict_alloc"]
    fn nvim_tv_dict_alloc() -> DictHandle;
    #[link_name = "tv_list_append_dict"]
    fn nvim_tv_list_append_dict(list: ListHandle, dict: DictHandle);
    #[link_name = "tv_dict_add_nr"]
    fn nvim_tv_dict_add_nr(dict: DictHandle, key: *const c_char, key_len: usize, nr: i64);
    #[link_name = "tv_dict_add_list"]
    fn nvim_tv_dict_add_list(
        dict: DictHandle,
        key: *const c_char,
        key_len: usize,
        list: ListHandle,
    );
    fn nvim_FullName_save(fname: *const c_char, force: bool) -> *mut c_char;
}

/// Build the undo tree as a VimL list for undotree().
///
/// This is called recursively to build alternate branches.
///
/// # Safety
///
/// All handles must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_u_eval_tree(buf: BufHandle, first_uhp: UHeaderHandle) -> ListHandle {
    let list = nvim_tv_list_alloc();

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let curhead = nvim_buf_get_b_u_curhead(buf);

    let mut uhp = first_uhp;
    while !uhp.is_null() {
        let dict = nvim_tv_dict_alloc();

        // Add seq
        let seq = (*uhp).uh_seq as i64;
        nvim_tv_dict_add_nr(dict, c"seq".as_ptr(), 3, seq);

        // Add time
        let time = (*uhp).uh_time;
        nvim_tv_dict_add_nr(dict, c"time".as_ptr(), 4, time);

        // Add newhead marker if applicable
        if uhp == newhead {
            nvim_tv_dict_add_nr(dict, c"newhead".as_ptr(), 7, 1);
        }

        // Add curhead marker if applicable
        if uhp == curhead {
            nvim_tv_dict_add_nr(dict, c"curhead".as_ptr(), 7, 1);
        }

        // Add save number if > 0
        let save_nr = (*uhp).uh_save_nr;
        if save_nr > 0 {
            nvim_tv_dict_add_nr(dict, c"save".as_ptr(), 4, save_nr as i64);
        }

        // Recurse for alternate branches
        let alt_next = (*uhp).uh_alt_next.ptr;
        if !alt_next.is_null() {
            let alt_list = rs_u_eval_tree(buf, alt_next);
            nvim_tv_dict_add_list(dict, c"alt".as_ptr(), 3, alt_list);
        }

        nvim_tv_list_append_dict(list, dict);

        // Move to previous entry
        uhp = (*uhp).uh_prev.ptr;
    }

    list
}

/// Get the name of the undo file for a buffer's file name.
/// When reading, find the first file that exists.
/// When writing, use the first directory that exists or ".".
///
/// Returns an allocated string or null.
///
/// # Safety
///
/// `buf_ffname` must be a valid C string or null.
unsafe fn u_get_undo_file_name(buf_ffname: *const c_char, reading: bool) -> *mut c_char {
    if buf_ffname.is_null() {
        return ptr::null_mut();
    }

    // Resolve symlink
    let ffname = nvim_undo_resolve_symlink(buf_ffname);

    let maxpathl = nvim_undo_get_maxpathl();
    let mut dir_name = vec![0u8; maxpathl + 1];
    let mut munged_name: *mut c_char = ptr::null_mut();
    let mut undo_file_name: *mut c_char = ptr::null_mut();

    // Loop over 'undodir'
    let p_udir = nvim_undo_get_p_udir();
    let mut dirp = p_udir;

    while *dirp != 0 {
        let dir_len =
            nvim_undo_copy_option_part(&mut dirp, dir_name.as_mut_ptr() as *mut c_char, maxpathl);

        if dir_len == 1 && dir_name[0] == b'.' {
            // Use same directory as the ffname: "dir/name" -> "dir/.name.un~"
            let ffname_len = libc::strlen(ffname);
            undo_file_name = nvim_xmalloc(ffname_len + 6) as *mut c_char;
            libc::memcpy(
                undo_file_name as *mut c_void,
                ffname as *const c_void,
                ffname_len + 1,
            );
            let tail_off = nvim_undo_path_tail_offset(undo_file_name);
            let tail = undo_file_name.add(tail_off);
            let tail_len = libc::strlen(tail);
            // Shift tail right by 1 to make room for '.'
            libc::memmove(
                tail.add(1) as *mut c_void,
                tail as *const c_void,
                tail_len + 1,
            );
            *tail = b'.' as c_char;
            // Append ".un~"
            libc::memcpy(
                tail.add(tail_len + 1) as *mut c_void,
                c".un~".as_ptr() as *const c_void,
                5,
            );
        } else {
            dir_name[dir_len] = 0;

            // Remove trailing path separators
            let mut p = dir_len;
            while p > 0 && nvim_undo_vim_ispathsep(dir_name[p - 1] as c_int) {
                p -= 1;
                dir_name[p] = 0;
            }

            let mut has_directory = nvim_undo_os_isdir(dir_name.as_ptr() as *const c_char);
            if !has_directory && *dirp == 0 && !reading {
                // Last directory in the list does not exist, create it.
                let mut failed_dir: *mut c_char = ptr::null_mut();
                let ret =
                    nvim_undo_mkdir_recurse(dir_name.as_ptr() as *const c_char, &mut failed_dir);
                if ret != 0 {
                    nvim_undo_semsg_mkdir(failed_dir, ret);
                    nvim_xfree(failed_dir as *mut c_void);
                } else {
                    has_directory = true;
                }
            }
            if has_directory {
                if munged_name.is_null() {
                    munged_name = nvim_undo_xstrdup(ffname);
                    let mut c = munged_name;
                    while *c != 0 {
                        if nvim_undo_vim_ispathsep(*c as c_int) {
                            *c = b'%' as c_char;
                        }
                        c = c.add(nvim_undo_mb_ptr_len(c) as usize);
                    }
                }
                undo_file_name =
                    nvim_undo_concat_fnames(dir_name.as_ptr() as *const c_char, munged_name);
            }
        }

        // When reading check if the file exists.
        if !undo_file_name.is_null() && (!reading || nvim_os_path_exists(undo_file_name)) {
            break;
        }
        if !undo_file_name.is_null() {
            nvim_xfree(undo_file_name as *mut c_void);
            undo_file_name = ptr::null_mut();
        }
    }

    nvim_xfree(munged_name as *mut c_void);
    nvim_xfree(ffname as *mut c_void);
    undo_file_name
}

/// undofile(name) - return the undo file path for a file name.
///
/// This is a Rust implementation that returns the path string.
/// The C wrapper handles typval conversion.
///
/// # Safety
///
/// `fname` must be a valid C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_undofile(fname: *const c_char) -> *mut c_char {
    if fname.is_null() {
        return std::ptr::null_mut();
    }

    // Check if fname is empty
    if *fname == 0 {
        return std::ptr::null_mut();
    }

    // Get full path
    let ffname = nvim_FullName_save(fname, true);
    if ffname.is_null() {
        return std::ptr::null_mut();
    }

    // Get undo file name
    let result = u_get_undo_file_name(ffname, false);
    nvim_xfree(ffname as *mut c_void);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Handle Size and Null Tests
    // =========================================================================

    #[test]
    fn test_handle_sizes() {
        // Verify handle sizes match pointer size
        assert_eq!(
            std::mem::size_of::<BufHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
        assert_eq!(
            std::mem::size_of::<UHeaderHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
        assert_eq!(
            std::mem::size_of::<UEntryHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }

    #[test]
    fn test_null_handle_checks() {
        assert!(rs_uhp_is_null(std::ptr::null_mut()));
        assert!(rs_uep_is_null(ptr::null_mut()));
    }

    #[test]
    fn test_file_handle_null() {
        let handle = FileHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_file_handle_size() {
        assert_eq!(
            std::mem::size_of::<FileHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }

    // =========================================================================
    // Undo File Format Constants Tests
    // =========================================================================

    #[test]
    fn test_undo_magic_bytes() {
        // Verify the magic bytes are correct
        assert_eq!(UF_START_MAGIC, b"Vim\x9fUnDo\xe5");
        assert_eq!(UF_START_MAGIC_LEN, 9);
    }

    #[test]
    fn test_undo_magic_values() {
        // Verify magic values for file format
        assert_eq!(UF_HEADER_MAGIC, 0x5fd0);
        assert_eq!(UF_HEADER_END_MAGIC, 0xe7aa);
        assert_eq!(UF_ENTRY_MAGIC, 0xf518);
        assert_eq!(UF_ENTRY_END_MAGIC, 0x3581);
    }

    #[test]
    fn test_undo_version() {
        // Version should be 3 for current format
        assert_eq!(UF_VERSION, 3);
    }

    #[test]
    fn test_undo_extra_field_ids() {
        // Extra field identifiers
        assert_eq!(UF_LAST_SAVE_NR, 1);
        assert_eq!(UHP_SAVE_NR, 1);
    }

    #[test]
    fn test_undo_hash_size() {
        // SHA-256 produces 32 bytes
        assert_eq!(UNDO_HASH_SIZE, 32);
    }

    // =========================================================================
    // Result Constants Tests
    // =========================================================================

    #[test]
    fn test_ok_fail_constants() {
        // Verify OK and FAIL match Neovim conventions
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_ne!(OK, FAIL);
    }

    // =========================================================================
    // Type Alias Tests
    // =========================================================================

    #[test]
    fn test_time_t_size() {
        // TimeT should be pointer-sized (i64 on 64-bit, i32 on 32-bit)
        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<TimeT>(), 8);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(std::mem::size_of::<TimeT>(), 4);
    }

    #[test]
    fn test_linenr_t_size() {
        // LinenrT is i32 (matching C int32_t linenr_T)
        assert_eq!(std::mem::size_of::<LinenrT>(), 4);
    }

    #[test]
    fn test_colnr_t_size() {
        // ColnrT is c_int
        assert_eq!(std::mem::size_of::<ColnrT>(), std::mem::size_of::<c_int>());
    }

    // =========================================================================
    // Handle Representation Tests
    // =========================================================================

    #[test]
    fn test_buf_handle_repr() {
        // BufHandle should be repr(transparent) over a pointer
        let ptr: *mut c_void = 0x1234 as *mut c_void;
        let handle = BufHandle(ptr);
        assert_eq!(handle.0, ptr);
    }

    #[test]
    fn test_uheader_handle_repr() {
        // UHeaderHandle is now *mut UHeader - a raw pointer type
        let ptr: *mut UHeader = 0x5678 as *mut UHeader;
        let handle: UHeaderHandle = ptr;
        assert_eq!(handle, ptr);
    }

    #[test]
    fn test_uentry_handle_repr() {
        // UEntryHandle is now *mut UEntry - a raw pointer type
        let ptr: *mut UEntry = 0xabcd as *mut UEntry;
        let handle: UEntryHandle = ptr;
        assert_eq!(handle, ptr);
    }

    // =========================================================================
    // Magic Number Uniqueness Tests
    // =========================================================================

    #[test]
    fn test_magic_numbers_are_unique() {
        // All magic numbers should be distinct
        let magics = [
            UF_HEADER_MAGIC,
            UF_HEADER_END_MAGIC,
            UF_ENTRY_MAGIC,
            UF_ENTRY_END_MAGIC,
        ];

        for i in 0..magics.len() {
            for j in (i + 1)..magics.len() {
                assert_ne!(
                    magics[i], magics[j],
                    "Magic numbers at indices {i} and {j} should be different"
                );
            }
        }
    }

    #[test]
    fn test_magic_numbers_nonzero() {
        // Magic numbers should be non-zero
        assert_ne!(UF_HEADER_MAGIC, 0);
        assert_ne!(UF_HEADER_END_MAGIC, 0);
        assert_ne!(UF_ENTRY_MAGIC, 0);
        assert_ne!(UF_ENTRY_END_MAGIC, 0);
    }

    // =========================================================================
    // Start Magic Tests
    // =========================================================================

    #[test]
    fn test_start_magic_contains_vim() {
        // The start magic should begin with "Vim"
        assert!(UF_START_MAGIC.starts_with(b"Vim"));
    }

    #[test]
    fn test_start_magic_contains_undo() {
        // The start magic should contain "UnDo" (case sensitive)
        let magic_str = std::str::from_utf8(&UF_START_MAGIC[4..8]).unwrap_or("");
        assert_eq!(magic_str, "UnDo");
    }

    #[test]
    fn test_start_magic_special_bytes() {
        // Verify the special bytes (0x9f after Vim, 0xe5 at end)
        assert_eq!(UF_START_MAGIC[3], 0x9f);
        assert_eq!(UF_START_MAGIC[8], 0xe5);
    }
}
