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

use std::ffi::{c_char, c_int, c_uchar, c_void, CStr};
use std::ptr;

use nvim_encoding::sha256::Sha256Context;

/// Opaque handle to buf_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BufHandle(*mut c_void);

/// Opaque handle to u_header_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UHeaderHandle(*mut c_void);

/// Opaque handle to u_entry_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UEntryHandle(*mut c_void);

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
    fn nvim_bt_prompt(buf: BufHandle) -> bool;
    fn nvim_file_ff_differs(buf: BufHandle, strict: bool) -> bool;

    // Global buffer iteration
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // Memory functions
    fn nvim_xfree(ptr: *mut c_void);

    // u_header_T field accessors
    fn nvim_uhp_get_next(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_prev(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_alt_next(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_alt_prev(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_seq(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_walk(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_entry(uhp: UHeaderHandle) -> UEntryHandle;
    fn nvim_uhp_get_getbot_entry(uhp: UHeaderHandle) -> UEntryHandle;
    fn nvim_uhp_get_time(uhp: UHeaderHandle) -> TimeT;
    fn nvim_uhp_get_flags(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_save_nr(uhp: UHeaderHandle) -> c_int;

    fn nvim_uhp_set_next(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_prev(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_alt_next(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_alt_prev(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_seq(uhp: UHeaderHandle, val: c_int);
    fn nvim_uhp_set_walk(uhp: UHeaderHandle, val: c_int);
    fn nvim_uhp_set_entry(uhp: UHeaderHandle, val: UEntryHandle);
    fn nvim_uhp_set_getbot_entry(uhp: UHeaderHandle, val: UEntryHandle);
    fn nvim_uhp_set_time(uhp: UHeaderHandle, val: TimeT);
    fn nvim_uhp_set_flags(uhp: UHeaderHandle, val: c_int);
    fn nvim_uhp_set_save_nr(uhp: UHeaderHandle, val: c_int);

    // u_entry_T field accessors
    fn nvim_uep_get_next(uep: UEntryHandle) -> UEntryHandle;
    fn nvim_uep_get_top(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_bot(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_lcount(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_size(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_array(uep: UEntryHandle) -> *mut *mut c_char;

    fn nvim_uep_set_next(uep: UEntryHandle, val: UEntryHandle);
    fn nvim_uep_set_top(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_bot(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_lcount(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_size(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_array(uep: UEntryHandle, val: *mut *mut c_char);

    // u_entry_T array element accessors
    fn nvim_uep_get_array_element(uep: UEntryHandle, idx: LinenrT) -> *mut c_char;
    fn nvim_uep_set_array_element(uep: UEntryHandle, idx: LinenrT, val: *mut c_char);

    // Allocation functions
    fn nvim_alloc_u_entry() -> UEntryHandle;
    fn nvim_alloc_u_header() -> UHeaderHandle;

    // Extmark vector destruction
    fn nvim_uhp_destroy_extmark(uhp: UHeaderHandle);

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
    fn nvim_fast_breakcheck();
    fn nvim_undo_got_int() -> bool;
    fn nvim_time_now() -> TimeT;
    fn nvim_get_curwin_cursor(lnum: *mut LinenrT, col: *mut ColnrT, coladd: *mut ColnrT);
    fn nvim_curwin_virtual_active() -> bool;
    fn nvim_getviscol() -> ColnrT;

    // u_savecommon infrastructure
    fn nvim_buf_set_b_new_change(buf: BufHandle, val: bool);
    fn nvim_buf_set_b_u_time_cur(buf: BufHandle, val: TimeT);
    fn nvim_uhp_init_extmark(uhp: UHeaderHandle);
    fn nvim_uhp_copy_marks_visual(buf: BufHandle, uhp: UHeaderHandle);
    fn nvim_uhp_set_cursor(uhp: UHeaderHandle, lnum: LinenrT, col: ColnrT, coladd: ColnrT);
    fn nvim_uhp_set_cursor_vcol(uhp: UHeaderHandle, vcol: ColnrT);
    fn nvim_uep_alloc_array(uep: UEntryHandle, size: LinenrT);
    fn nvim_uep_set_array_from_buf(uep: UEntryHandle, idx: LinenrT, buf: BufHandle, lnum: LinenrT);
    fn nvim_emsg_line_count_changed();
    fn nvim_buf_is_curbuf(buf: BufHandle) -> bool;
    fn nvim_set_undo_undoes_false();

    // u_find_first_changed infrastructure
    fn nvim_uep_compare_line_with_array(
        uep: UEntryHandle,
        idx: LinenrT,
        buf: BufHandle,
        lnum: LinenrT,
    ) -> bool;
    fn nvim_uhp_clear_cursor(uhp: UHeaderHandle);
    fn nvim_uhp_set_cursor_lnum_only(uhp: UHeaderHandle, lnum: LinenrT);

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
    fn nvim_undo_fopen(path: *const c_char, mode: *const c_char) -> FileHandle;
    fn nvim_undo_fclose(fp: FileHandle) -> c_int;
    fn nvim_undo_fwrite(ptr: *const c_void, size: usize, count: usize, fp: FileHandle) -> usize;
    fn nvim_undo_fread(ptr: *mut c_void, size: usize, count: usize, fp: FileHandle) -> usize;
    fn nvim_undo_fflush(fp: FileHandle) -> c_int;
    fn nvim_undo_fgetc(fp: FileHandle) -> c_int;

    // File I/O helpers (reading from C file handle)
    fn nvim_undo_get2c(fp: FileHandle) -> c_int;
    fn nvim_undo_get4c(fp: FileHandle) -> c_int;
    fn nvim_undo_get8ctime(fp: FileHandle) -> TimeT;

    // Buffer file path accessors
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;

    // File system operations
    fn nvim_os_path_exists(path: *const c_char) -> bool;
    fn nvim_os_remove(path: *const c_char) -> c_int;
    fn nvim_os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int;
    fn nvim_os_close(fd: c_int) -> c_int;
    fn nvim_os_getperm(path: *const c_char) -> c_int;
    fn nvim_os_setperm(path: *const c_char, perm: c_int) -> c_int;
    fn nvim_os_fsync(fd: c_int) -> c_int;
    fn nvim_fdopen(fd: c_int, mode: *const c_char) -> FileHandle;
    fn nvim_fileno(fp: FileHandle) -> c_int;

    // Message functions for undo file I/O
    fn nvim_undo_verbose_enter();
    fn nvim_undo_verbose_leave();
    fn nvim_undo_smsg(msg: *const c_char, arg: *const c_char);
    fn nvim_undo_semsg(msg: *const c_char, arg: *const c_char);
    fn nvim_undo_give_warning(msg: *const c_char, serious: bool);
    fn nvim_undo_verb_msg(msg: *const c_char);

    // Option accessors
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_get_p_fs() -> bool;

    // u_sync wrapper
    fn nvim_u_sync(force: bool);

    // Buffer line count and line accessors for hash computation
    fn nvim_buf_get_b_ml_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_ml_get_buf_line(buf: BufHandle, lnum: LinenrT) -> *const c_char;

    // ACL operations (Unix)
    fn nvim_os_get_acl(path: *const c_char) -> *mut c_void;
    fn nvim_os_set_acl(path: *const c_char, acl: *mut c_void);
    fn nvim_os_free_acl(acl: *mut c_void);

    // Hash computation
    fn nvim_u_compute_hash(buf: BufHandle, hash: *mut c_uchar);

    // File info for Unix ownership checks
    fn nvim_undo_check_file_owner(orig_path: *const c_char, undo_path: *const c_char) -> bool;
    fn nvim_undo_set_file_group(
        fd: c_int,
        orig_path: *const c_char,
        undo_path: *const c_char,
        perm: c_int,
    ) -> c_int;

    // Read helper for errno handling
    fn nvim_read_eintr(fd: c_int, buf: *mut c_void, count: usize) -> isize;

    // Extmark serialization
    fn nvim_uhp_get_extmark_count(uhp: UHeaderHandle) -> usize;
    fn nvim_uhp_get_extmark_type(uhp: UHeaderHandle, idx: usize) -> c_int;
    fn nvim_uhp_get_extmark_data(uhp: UHeaderHandle, idx: usize, buf: *mut c_uchar, size: usize);

    // Named mark and visual info serialization
    fn nvim_uhp_get_namedm_lnum(uhp: UHeaderHandle, idx: c_int) -> LinenrT;
    fn nvim_uhp_get_namedm_col(uhp: UHeaderHandle, idx: c_int) -> ColnrT;
    fn nvim_uhp_get_namedm_coladd(uhp: UHeaderHandle, idx: c_int) -> ColnrT;
    fn nvim_uhp_get_visual_start_lnum(uhp: UHeaderHandle) -> LinenrT;
    fn nvim_uhp_get_visual_start_col(uhp: UHeaderHandle) -> ColnrT;
    fn nvim_uhp_get_visual_start_coladd(uhp: UHeaderHandle) -> ColnrT;
    fn nvim_uhp_get_visual_end_lnum(uhp: UHeaderHandle) -> LinenrT;
    fn nvim_uhp_get_visual_end_col(uhp: UHeaderHandle) -> ColnrT;
    fn nvim_uhp_get_visual_end_coladd(uhp: UHeaderHandle) -> ColnrT;
    fn nvim_uhp_get_visual_mode(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_visual_curswant(uhp: UHeaderHandle) -> ColnrT;

    // Cursor position from header
    fn nvim_uhp_get_cursor_lnum(uhp: UHeaderHandle) -> LinenrT;
    fn nvim_uhp_get_cursor_col(uhp: UHeaderHandle) -> ColnrT;
    fn nvim_uhp_get_cursor_coladd(uhp: UHeaderHandle) -> ColnrT;
    fn nvim_uhp_get_cursor_vcol(uhp: UHeaderHandle) -> ColnrT;

    // Sequence number accessors for serialization
    fn nvim_uhp_get_next_seq(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_prev_seq(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_alt_next_seq(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_alt_prev_seq(uhp: UHeaderHandle) -> c_int;

    // Global lastmark accessor
    fn nvim_get_lastmark() -> c_int;
    fn nvim_set_lastmark(val: c_int);

    // ==========================================================================
    // Phase 2: Core Undo Operations FFI (memline manipulation)
    // ==========================================================================

    /// Delete line 'lnum' in buffer. Returns OK/FAIL.
    fn nvim_ml_delete_lnum(lnum: LinenrT) -> c_int;

    /// Delete line 'lnum' in buffer with flags. Returns OK/FAIL.
    fn nvim_ml_delete_flags(lnum: LinenrT, flags: c_int) -> c_int;

    /// Append line after 'lnum' in current buffer. Returns OK/FAIL.
    fn nvim_ml_append_lnum(lnum: LinenrT, line: *const c_char, len: ColnrT, newfile: bool)
        -> c_int;

    /// Append line with flags. Returns OK/FAIL.
    fn nvim_ml_append_flags(lnum: LinenrT, line: *const c_char, len: ColnrT, flags: c_int)
        -> c_int;

    /// Replace line in current buffer. Returns OK/FAIL.
    fn nvim_ml_replace_lnum(lnum: LinenrT, line: *const c_char, copy: bool) -> c_int;

    /// Block/unblock autocommands
    fn nvim_block_autocmds();
    fn nvim_unblock_autocmds();

    /// Set pc mark for jump list
    fn nvim_undo_setpcmark();

    /// Check cursor line number validity and adjust if needed
    fn nvim_undo_check_cursor_lnum(win: WinHandle);

    /// Mark adjust for undo
    fn nvim_undo_mark_adjust(top: LinenrT, bot: LinenrT, amount: LinenrT, amount_after: LinenrT);

    /// Changed lines notification
    fn nvim_undo_changed_lines(
        buf: BufHandle,
        top: LinenrT,
        col: ColnrT,
        bot: LinenrT,
        xtra: LinenrT,
        do_buf_event: bool,
    );

    /// Mark buffer as changed
    fn nvim_buf_changed(buf: BufHandle);

    /// Mark buffer as unchanged
    fn nvim_buf_unchanged(buf: BufHandle, ff: bool, always_strstruc: bool);

    /// Check spell for window
    fn nvim_spell_check_window(win: WinHandle) -> bool;

    /// Redraw window line
    fn nvim_redrawWinline(win: WinHandle, lnum: LinenrT);

    /// Apply extmark undo
    fn nvim_extmark_apply_undo(uhp: UHeaderHandle, idx: usize, undo: bool);

    /// Buffer updates unload
    fn nvim_buf_updates_unload(buf: BufHandle, force: bool);

    /// Check position validity
    fn nvim_check_pos(buf: BufHandle, pos: *mut c_void);

    /// Buffer is empty check
    fn nvim_buf_is_empty(buf: BufHandle) -> bool;

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

    /// Fold open cursor
    fn nvim_undo_foldOpenCursor();

    /// Check VIsual_active
    fn nvim_undo_get_visual_active() -> bool;

    /// Get VIsual position
    fn nvim_undo_get_visual_pos(lnum: *mut LinenrT, col: *mut ColnrT, coladd: *mut ColnrT);

    // ==========================================================================
    // Phase 3: u_undoredo FFI helpers
    // ==========================================================================

    /// Compute new_flags for undo/redo based on current buffer state
    fn nvim_undoredo_compute_new_flags(buf: BufHandle, curhead: UHeaderHandle) -> c_int;

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

    /// Apply extmarks for undo/redo
    fn nvim_undoredo_apply_extmarks(curhead: UHeaderHandle, undo: bool);

    /// Set ML_EMPTY flag if needed
    fn nvim_undoredo_set_ml_empty(buf: BufHandle, old_flags: c_int);

    /// Cursor adjustment for u_undoredo
    fn nvim_undoredo_adjust_cursor(curhead: UHeaderHandle);

    /// Allocate saved marks buffer (fmark_T[NMARKS] + visualinfo_T)
    fn nvim_alloc_saved_marks() -> *mut c_void;

    /// Get ml_get result as non-allocating pointer for strcmp
    fn nvim_undoredo_ml_get(lnum: LinenrT) -> *const c_char;

    /// buf_updates_changedtick wrapper
    fn nvim_undoredo_buf_updates_changedtick(buf: BufHandle);

    /// Update sequence and time for undo/redo
    fn nvim_undoredo_update_seq(buf: BufHandle, curhead: UHeaderHandle, undo: bool);

    /// E438 error message wrapper
    fn nvim_iemsg_undo_line_numbers_wrong();

    /// xmalloc wrapper
    fn nvim_xmalloc(size: usize) -> *mut c_void;

    /// Return byte offset of visualinfo_T within saved marks buffer
    fn nvim_saved_marks_visual_offset() -> usize;

    // ==========================================================================
    // Phase 4: u_undo_end + helpers FFI
    // ==========================================================================

    /// Get uhp->uh_seq for the message, also adjusts did_undo
    fn nvim_undo_end_get_uhp_seq(
        buf: BufHandle,
        did_undo: bool,
        absolute: bool,
        did_undo_out: *mut bool,
    ) -> c_int;

    /// Format time for undo message
    fn nvim_undo_end_fmt_time(
        buf: BufHandle,
        did_undo: bool,
        absolute: bool,
        timebuf: *mut c_char,
        buflen: usize,
    );

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

    /// ML_EMPTY flag check
    fn nvim_undo_end_ml_empty(buf: BufHandle) -> bool;

    /// get_undolevel accessor
    fn nvim_get_undolevel_value(buf: BufHandle) -> i64;

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
    fn nvim_undo_os_isdir(path: *const c_char) -> bool;

    /// Create directory recursively
    fn nvim_undo_mkdir_recurse(dir: *const c_char, failed_dir: *mut *mut c_char) -> c_int;

    /// E5003 error message
    fn nvim_undo_semsg_mkdir(failed_dir: *const c_char, err: c_int);

    /// path_tail offset
    fn nvim_undo_path_tail_offset(path: *const c_char) -> usize;

    /// vim_ispathsep check
    fn nvim_undo_vim_ispathsep(c: c_int) -> bool;

    /// Multibyte pointer char length
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
#[no_mangle]
pub unsafe extern "C" fn rs_bufIsChanged(buf: BufHandle) -> bool {
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
#[no_mangle]
pub unsafe extern "C" fn rs_anyBufIsChanged() -> bool {
    let mut buf = nvim_get_firstbuf();
    while !buf.0.is_null() {
        if rs_bufIsChanged(buf) {
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
#[no_mangle]
pub unsafe extern "C" fn rs_curbufIsChanged() -> bool {
    rs_bufIsChanged(nvim_get_curbuf())
}

/// Invalidate the undo buffer; called when storage has already been released.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_clearall(buf: BufHandle) {
    nvim_buf_set_b_u_newhead(buf, UHeaderHandle(std::ptr::null_mut()));
    nvim_buf_set_b_u_oldhead(buf, UHeaderHandle(std::ptr::null_mut()));
    nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
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
#[no_mangle]
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
        let elem = nvim_uep_get_array_element(uep, LinenrT::from(n));
        nvim_xfree(elem as *mut c_void);
    }

    // Free the array itself
    let array = nvim_uep_get_array(uep);
    nvim_xfree(array as *mut c_void);

    // Free the entry struct
    nvim_xfree(uep.0);
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
    if curhead.0 == uhp.0 {
        nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.0 == uhp.0 {
        nvim_buf_set_b_u_newhead(buf, UHeaderHandle(std::ptr::null_mut()));
    }

    if !uhpp.is_null() && (*uhpp).0 == uhp.0 {
        *uhpp = UHeaderHandle(std::ptr::null_mut());
    }

    // Free all entries in the list
    let mut uep = nvim_uhp_get_entry(uhp);
    while !uep.0.is_null() {
        let nuep = nvim_uep_get_next(uep);
        let size = nvim_uep_get_size(uep);
        rs_u_freeentry(uep, size as c_int);
        uep = nuep;
    }

    // Destroy the extmark vector
    nvim_uhp_destroy_extmark(uhp);

    // Free the header struct
    nvim_xfree(uhp.0);

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
    let alt_next = nvim_uhp_get_alt_next(uhp);
    if !alt_next.0.is_null() {
        rs_u_freebranch(buf, alt_next, uhpp);
    }

    let alt_prev = nvim_uhp_get_alt_prev(uhp);
    if !alt_prev.0.is_null() {
        nvim_uhp_set_alt_next(alt_prev, UHeaderHandle(std::ptr::null_mut()));
    }

    // Update the links in the list to remove the header.
    let uh_next = nvim_uhp_get_next(uhp);
    let uh_prev = nvim_uhp_get_prev(uhp);

    if uh_next.0.is_null() {
        nvim_buf_set_b_u_oldhead(buf, uh_prev);
    } else {
        nvim_uhp_set_prev(uh_next, uh_prev);
    }

    if uh_prev.0.is_null() {
        nvim_buf_set_b_u_newhead(buf, uh_next);
    } else {
        let mut uhap = uh_prev;
        while !uhap.0.is_null() {
            nvim_uhp_set_next(uhap, uh_next);
            uhap = nvim_uhp_get_alt_next(uhap);
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
    if uhp.0 == oldhead.0 {
        loop {
            let current_oldhead = nvim_buf_get_b_u_oldhead(buf);
            if current_oldhead.0.is_null() {
                break;
            }
            rs_u_freeheader(buf, current_oldhead, uhpp);
        }
        return;
    }

    let alt_prev = nvim_uhp_get_alt_prev(uhp);
    if !alt_prev.0.is_null() {
        nvim_uhp_set_alt_next(alt_prev, UHeaderHandle(std::ptr::null_mut()));
    }

    let mut next = uhp;
    while !next.0.is_null() {
        let tofree = next;
        let alt_next = nvim_uhp_get_alt_next(tofree);
        if !alt_next.0.is_null() {
            rs_u_freebranch(buf, alt_next, uhpp); // recursive
        }
        next = nvim_uhp_get_prev(tofree);
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
    if newhead.0.is_null() {
        nvim_iemsg_undo_list_corrupt();
        return UEntryHandle(std::ptr::null_mut());
    }

    let entry = nvim_uhp_get_entry(newhead);
    if entry.0.is_null() {
        nvim_iemsg_undo_list_corrupt();
        return UEntryHandle(std::ptr::null_mut());
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
    if check.0.is_null() {
        return;
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let uep = nvim_uhp_get_getbot_entry(newhead);
    if !uep.0.is_null() {
        // The new ue_bot is computed from the number of lines that has been
        // inserted (0 - deleted) since calling u_save. This is equal to the
        // old line count subtracted from the current line count.
        let ml_line_count = nvim_buf_get_ml_line_count(buf);
        let ue_lcount = nvim_uep_get_lcount(uep);
        let extra = ml_line_count - ue_lcount;

        let ue_top = nvim_uep_get_top(uep);
        let ue_size = nvim_uep_get_size(uep);
        let mut ue_bot = ue_top + ue_size + 1 + extra;

        if ue_bot < 1 || ue_bot > ml_line_count {
            nvim_iemsg_undo_line_missing();
            // Assume all lines deleted, will get all the old lines back
            // without deleting the current ones
            ue_bot = ue_top + 1;
        }

        nvim_uep_set_bot(uep, ue_bot);
        nvim_uhp_set_getbot_entry(newhead, UEntryHandle(std::ptr::null_mut()));
    }

    nvim_buf_set_b_u_synced(buf, true);
}

/// Free all undo headers and entries for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_blockfree(buf: BufHandle) {
    loop {
        let oldhead = nvim_buf_get_b_u_oldhead(buf);
        if oldhead.0.is_null() {
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
#[no_mangle]
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
        nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
    }
}

/// Free all allocated memory blocks for the buffer and invalidate the undo buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
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
    while !uh.0.is_null() {
        // Set UH_CHANGED flag
        let flags = nvim_uhp_get_flags(uh);
        nvim_uhp_set_flags(uh, flags | UH_CHANGED);

        // Recurse into alternate branch if present
        let alt_next = nvim_uhp_get_alt_next(uh);
        if !alt_next.0.is_null() {
            rs_u_unch_branch(alt_next);
        }

        // Move to previous header
        uh = nvim_uhp_get_prev(uh);
    }
}

/// Called after writing or reloading the file and setting b_changed to false.
/// Now an undo means that the buffer is modified.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_u_update_save_nr(buf: BufHandle) {
    let save_nr_last = nvim_buf_get_b_u_save_nr_last(buf) + 1;
    nvim_buf_set_b_u_save_nr_last(buf, save_nr_last);
    nvim_buf_set_b_u_save_nr_cur(buf, save_nr_last);

    let curhead = nvim_buf_get_b_u_curhead(buf);
    let uhp = if !curhead.0.is_null() {
        nvim_uhp_get_next(curhead)
    } else {
        nvim_buf_get_b_u_newhead(buf)
    };

    if !uhp.0.is_null() {
        nvim_uhp_set_save_nr(uhp, save_nr_last);
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
    let mut uep = nvim_uhp_get_entry(uhp);
    while !uep.0.is_null() {
        let nuep = nvim_uep_get_next(uep);
        let size = nvim_uep_get_size(uep);
        rs_u_freeentry(uep, size as c_int);
        uep = nuep;
    }
    nvim_xfree(uhp.0);
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
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_ex_undojoin() {
    let buf = nvim_get_curbuf();

    // Nothing changed before
    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.0.is_null() {
        return;
    }

    // Not allowed after undo
    let curhead = nvim_buf_get_b_u_curhead(buf);
    if !curhead.0.is_null() {
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_u_undo_and_forget(mut count: c_int, do_buf_event: bool) -> bool {
    let buf = nvim_get_curbuf();

    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
        count = 1;
    }

    nvim_set_undo_undoes(true);
    rs_u_doit(count, true, do_buf_event);

    let curhead = nvim_buf_get_b_u_curhead(buf);
    if curhead.0.is_null() {
        // nothing was undone
        return false;
    }

    // Delete the current redo header
    // set the redo header to the next alternative branch (if any)
    // otherwise we will be in the leaf state
    let to_forget = curhead;
    let uh_next = nvim_uhp_get_next(to_forget);
    nvim_buf_set_b_u_newhead(buf, uh_next);

    let alt_next = nvim_uhp_get_alt_next(to_forget);
    nvim_buf_set_b_u_curhead(buf, alt_next);

    if !alt_next.0.is_null() {
        nvim_uhp_set_alt_next(to_forget, UHeaderHandle(std::ptr::null_mut()));
        let alt_prev = nvim_uhp_get_alt_prev(to_forget);
        nvim_uhp_set_alt_prev(alt_next, alt_prev);

        let alt_next_next = nvim_uhp_get_next(alt_next);
        if !alt_next_next.0.is_null() {
            nvim_buf_set_b_u_seq_cur(buf, nvim_uhp_get_seq(alt_next_next));
        } else {
            nvim_buf_set_b_u_seq_cur(buf, 0);
        }
    } else {
        let newhead = nvim_buf_get_b_u_newhead(buf);
        if !newhead.0.is_null() {
            nvim_buf_set_b_u_seq_cur(buf, nvim_uhp_get_seq(newhead));
        }
    }

    let alt_prev = nvim_uhp_get_alt_prev(to_forget);
    if !alt_prev.0.is_null() {
        let new_curhead = nvim_buf_get_b_u_curhead(buf);
        nvim_uhp_set_alt_next(alt_prev, new_curhead);
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    if !newhead.0.is_null() {
        let new_curhead = nvim_buf_get_b_u_curhead(buf);
        nvim_uhp_set_prev(newhead, new_curhead);
    }

    let seq_last = nvim_buf_get_b_u_seq_last(buf);
    let to_forget_seq = nvim_uhp_get_seq(to_forget);
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

    let old_flags = nvim_uhp_get_flags(curhead);
    let new_flags = nvim_undoredo_compute_new_flags(buf, curhead);
    nvim_undo_setpcmark();

    // Save marks before undo/redo
    nvim_undoredo_save_marks(buf, curhead);
    // Allocate buffer for saved namedm + visualinfo
    let saved_marks = nvim_alloc_saved_marks();
    // NMARKS fmark_T entries followed by one visualinfo_T
    nvim_undoredo_get_buf_marks(
        buf,
        saved_marks,
        saved_marks.add(nvim_saved_marks_visual_offset()),
    );

    nvim_undoredo_init_op_marks(buf);

    let mut newlist = UEntryHandle(ptr::null_mut());
    let mut uep = nvim_uhp_get_entry(curhead);

    while !uep.0.is_null() {
        let top = nvim_uep_get_top(uep);
        let mut bot = nvim_uep_get_bot(uep);
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
        let newsize = nvim_uep_get_size(uep); // number of lines after undo

        // Decide about the cursor position, depending on what text changed.
        if top < newlnum {
            let cursor_lnum = nvim_uhp_get_cursor_lnum(curhead);
            if cursor_lnum >= top && cursor_lnum <= top + newsize + 1 {
                new_curpos_lnum = cursor_lnum;
                newlnum = new_curpos_lnum - 1;
            } else {
                // Use the first line that actually changed.
                let mut i: LinenrT = 0;
                while i < newsize && i < oldsize {
                    let array_line = nvim_uep_get_array_element(uep, i);
                    let buf_line = nvim_undoredo_ml_get(top + 1 + i);
                    if libc::strcmp(array_line, buf_line) != 0 {
                        break;
                    }
                    i += 1;
                }
                let next_uep = nvim_uep_get_next(uep);
                if i == newsize && newlnum == MAXLNUM && next_uep.0.is_null() {
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
                if empty_buffer && lnum == 0 {
                    let line = nvim_uep_get_array_element(uep, i);
                    nvim_ml_replace_lnum(1, line, true);
                } else {
                    let line = nvim_uep_get_array_element(uep, i);
                    nvim_ml_append_flags(lnum, line, 0, 0);
                }
                nvim_xfree(nvim_uep_get_array_element(uep, i) as *mut c_void);
                i += 1;
                lnum += 1;
            }
            nvim_xfree(nvim_uep_get_array(uep) as *mut c_void);
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
        nvim_uep_set_size(uep, oldsize);
        nvim_uep_set_array(uep, newarray);
        nvim_uep_set_bot(uep, top + newsize + 1);

        // insert this entry in front of the new entry list
        let nuep = nvim_uep_get_next(uep);
        nvim_uep_set_next(uep, newlist);
        newlist = uep;
        uep = nuep;
    }

    // Ensure the '[ and '] marks are within bounds.
    nvim_undoredo_clamp_op_marks(buf);

    // Adjust Extmarks
    nvim_undoredo_apply_extmarks(curhead, undo);

    // Set the cursor to the desired position. Check that the line is valid.
    nvim_undo_win_set_cursor_pos(win, new_curpos_lnum, 0, 0);
    nvim_undo_check_cursor_lnum(win);

    nvim_uhp_set_entry(curhead, newlist);
    nvim_uhp_set_flags(curhead, new_flags);
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
    nvim_undoredo_swap_visual(
        buf,
        curhead,
        saved_marks.add(nvim_saved_marks_visual_offset()),
    );
    nvim_xfree(saved_marks);

    // Adjust cursor position
    nvim_undoredo_adjust_cursor(curhead);

    // Remember where we are for "g-" and ":earlier 10s".
    nvim_undoredo_update_seq(buf, curhead, undo);

    nvim_unblock_autocmds();
}

/// Report the result of an undo/redo operation.
/// If we deleted or added lines, report the number of less/more lines.
/// Otherwise, report the number of changes.
unsafe fn u_undo_end(did_undo: bool, absolute: bool, quiet: bool) {
    let buf = nvim_get_curbuf();

    if (nvim_undo_get_fdo_flags() & K_OPT_FDO_FLAG_UNDO) != 0 && nvim_undo_get_key_typed() {
        nvim_undo_foldOpenCursor();
    }

    if quiet || nvim_get_global_busy() || !nvim_messaging() {
        return;
    }

    let mut u_newcount = nvim_get_u_newcount();
    let mut u_oldcount = nvim_get_u_oldcount();

    if nvim_undo_end_ml_empty(buf) {
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

    let mut adjusted_did_undo = did_undo;
    let seq = nvim_undo_end_get_uhp_seq(buf, did_undo, absolute, &mut adjusted_did_undo);

    let mut timebuf = [0u8; 80];
    nvim_undo_end_fmt_time(
        buf,
        did_undo,
        absolute,
        timebuf.as_mut_ptr() as *mut c_char,
        timebuf.len(),
    );

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
            if curhead.0.is_null() {
                // first undo
                let newhead = nvim_buf_get_b_u_newhead(buf);
                nvim_buf_set_b_u_curhead(buf, newhead);
            } else if nvim_get_undolevel(buf) > 0 {
                // multi level undo - get next undo
                let next = nvim_uhp_get_next(curhead);
                nvim_buf_set_b_u_curhead(buf, next);
            }

            // nothing to undo
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let numhead = nvim_buf_get_b_u_numhead(buf);
            if numhead == 0 || curhead.0.is_null() {
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
            if curhead.0.is_null() || nvim_get_undolevel(buf) <= 0 {
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
            let prev = nvim_uhp_get_prev(curhead);
            if prev.0.is_null() {
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
#[no_mangle]
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
            uhp = nvim_alloc_u_header();
            nvim_uhp_init_extmark(uhp);
        } else {
            uhp = UHeaderHandle(std::ptr::null_mut());
        }

        // If we undid more than we redid, move the entry lists before and
        // including curbuf->b_u_curhead to an alternate branch
        let mut old_curhead = nvim_buf_get_b_u_curhead(buf);
        if !old_curhead.0.is_null() {
            let next = nvim_uhp_get_next(old_curhead);
            nvim_buf_set_b_u_newhead(buf, next);
            nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
        }

        // Free headers to keep the size right
        while nvim_buf_get_b_u_numhead(buf) as i64 > nvim_get_undolevel(buf) {
            let oldhead = nvim_buf_get_b_u_oldhead(buf);
            if oldhead.0.is_null() {
                break;
            }

            if oldhead.0 == old_curhead.0 {
                // Can't reconnect the branch, delete all of it
                rs_u_freebranch(buf, oldhead, &mut old_curhead as *mut UHeaderHandle);
            } else {
                let alt_next = nvim_uhp_get_alt_next(oldhead);
                if alt_next.0.is_null() {
                    // There is no branch, only free one header
                    rs_u_freeheader(buf, oldhead, &mut old_curhead as *mut UHeaderHandle);
                } else {
                    // Free the oldest alternate branch as a whole
                    let mut uhfree = oldhead;
                    loop {
                        let next_alt = nvim_uhp_get_alt_next(uhfree);
                        if next_alt.0.is_null() {
                            break;
                        }
                        uhfree = next_alt;
                    }
                    rs_u_freebranch(buf, uhfree, &mut old_curhead as *mut UHeaderHandle);
                }
            }
        }

        if uhp.0.is_null() {
            // No undo at all
            if !old_curhead.0.is_null() {
                rs_u_freebranch(buf, old_curhead, std::ptr::null_mut());
            }
            nvim_buf_set_b_u_synced(buf, false);
            return OK;
        }

        // Set up the new header
        nvim_uhp_set_prev(uhp, UHeaderHandle(std::ptr::null_mut()));
        let newhead = nvim_buf_get_b_u_newhead(buf);
        nvim_uhp_set_next(uhp, newhead);
        nvim_uhp_set_alt_next(uhp, old_curhead);

        if !old_curhead.0.is_null() {
            let alt_prev = nvim_uhp_get_alt_prev(old_curhead);
            nvim_uhp_set_alt_prev(uhp, alt_prev);

            if !alt_prev.0.is_null() {
                nvim_uhp_set_alt_next(alt_prev, uhp);
            }

            nvim_uhp_set_alt_prev(old_curhead, uhp);

            let oldhead = nvim_buf_get_b_u_oldhead(buf);
            if oldhead.0 == old_curhead.0 {
                nvim_buf_set_b_u_oldhead(buf, uhp);
            }
        } else {
            nvim_uhp_set_alt_prev(uhp, UHeaderHandle(std::ptr::null_mut()));
        }

        if !newhead.0.is_null() {
            nvim_uhp_set_prev(newhead, uhp);
        }

        // Set sequence numbers and time
        let seq_last = nvim_buf_get_b_u_seq_last(buf);
        nvim_buf_set_b_u_seq_last(buf, seq_last + 1);
        nvim_uhp_set_seq(uhp, seq_last + 1);
        nvim_buf_set_b_u_seq_cur(buf, seq_last + 1);

        let now = nvim_time_now();
        nvim_uhp_set_time(uhp, now);
        nvim_uhp_set_save_nr(uhp, 0);
        nvim_buf_set_b_u_time_cur(buf, now + 1);

        nvim_uhp_set_walk(uhp, 0);
        nvim_uhp_set_entry(uhp, UEntryHandle(std::ptr::null_mut()));
        nvim_uhp_set_getbot_entry(uhp, UEntryHandle(std::ptr::null_mut()));

        // Save cursor position
        let mut lnum: LinenrT = 0;
        let mut col: ColnrT = 0;
        let mut coladd: ColnrT = 0;
        nvim_get_curwin_cursor(&mut lnum, &mut col, &mut coladd);
        nvim_uhp_set_cursor(uhp, lnum, col, coladd);

        if nvim_curwin_virtual_active() && coladd > 0 {
            nvim_uhp_set_cursor_vcol(uhp, nvim_getviscol());
        } else {
            nvim_uhp_set_cursor_vcol(uhp, -1);
        }

        // Save changed and buffer empty flag
        let changed = nvim_buf_get_b_changed(buf);
        let ml_empty = nvim_buf_ml_is_empty(buf);
        let flags = (if changed { 1 } else { 0 }) + (if ml_empty { 2 } else { 0 });
        nvim_uhp_set_flags(uhp, flags);

        // Save named marks and Visual marks
        nvim_uhp_copy_marks_visual(buf, uhp);

        nvim_buf_set_b_u_newhead(buf, uhp);

        let oldhead = nvim_buf_get_b_u_oldhead(buf);
        if oldhead.0.is_null() {
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
            let mut prev_uep = UEntryHandle(std::ptr::null_mut());

            for _ in 0..10 {
                if uep.0.is_null() {
                    break;
                }

                let newhead = nvim_buf_get_b_u_newhead(buf);
                let getbot_entry = nvim_uhp_get_getbot_entry(newhead);
                let ue_top = nvim_uep_get_top(uep);
                let ue_size = nvim_uep_get_size(uep);
                let ue_bot = nvim_uep_get_bot(uep);
                let ue_lcount = nvim_uep_get_lcount(uep);
                let line_count = nvim_buf_get_ml_line_count(buf);

                // Check if lines have been inserted/deleted
                let reuse_blocked = if getbot_entry.0 != uep.0 {
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
                    if !prev_uep.0.is_null() {
                        // Move the found entry to become the last entry
                        rs_u_getbot(buf);
                        nvim_buf_set_b_u_synced(buf, false);

                        let uep_next = nvim_uep_get_next(uep);
                        nvim_uep_set_next(prev_uep, uep_next);

                        let newhead = nvim_buf_get_b_u_newhead(buf);
                        let entry = nvim_uhp_get_entry(newhead);
                        nvim_uep_set_next(uep, entry);
                        nvim_uhp_set_entry(newhead, uep);
                    }

                    // The executed command may change the line count
                    if newbot != 0 {
                        nvim_uep_set_bot(uep, newbot);
                    } else if bot > line_count {
                        nvim_uep_set_bot(uep, 0);
                    } else {
                        nvim_uep_set_lcount(uep, line_count);
                        let newhead = nvim_buf_get_b_u_newhead(buf);
                        nvim_uhp_set_getbot_entry(newhead, uep);
                    }
                    return OK;
                }

                prev_uep = uep;
                uep = nvim_uep_get_next(uep);
            }
        }

        // Find line number for ue_bot for previous u_save()
        rs_u_getbot(buf);
    }

    // Add lines in front of entry list
    let uep = nvim_alloc_u_entry();

    nvim_uep_set_size(uep, size);
    nvim_uep_set_top(uep, top);

    let line_count = nvim_buf_get_ml_line_count(buf);
    if newbot != 0 {
        nvim_uep_set_bot(uep, newbot);
    } else if bot > line_count {
        nvim_uep_set_bot(uep, 0);
    } else {
        nvim_uep_set_lcount(uep, line_count);
        let newhead = nvim_buf_get_b_u_newhead(buf);
        nvim_uhp_set_getbot_entry(newhead, uep);
    }

    if size > 0 {
        nvim_uep_alloc_array(uep, size);
        let mut lnum = top + 1;
        for i in 0..size {
            nvim_fast_breakcheck();
            if nvim_undo_got_int() {
                rs_u_freeentry(uep, i as c_int);
                return FAIL;
            }
            nvim_uep_set_array_from_buf(uep, i, buf, lnum);
            lnum += 1;
        }
    } else {
        nvim_uep_set_array(uep, std::ptr::null_mut());
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let entry = nvim_uhp_get_entry(newhead);
    nvim_uep_set_next(uep, entry);
    nvim_uhp_set_entry(newhead, uep);

    if reload {
        // Buffer was reloaded, notify text change subscribers
        let curbuf = nvim_get_curbuf();
        let curbuf_newhead = nvim_buf_get_b_u_newhead(curbuf);
        let flags = nvim_uhp_get_flags(curbuf_newhead);
        nvim_uhp_set_flags(curbuf_newhead, flags | 4); // UH_RELOAD = 4
    }

    nvim_buf_set_b_u_synced(buf, false);
    nvim_set_undo_undoes_false();

    OK
}

/// Save the line at cursor position for undo.
/// Rust implementation of u_save_cursor.
///
/// # Safety
///
/// Must be called from a valid Neovim context with curwin set.
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_u_save(top: LinenrT, bot: LinenrT) -> c_int {
    rs_u_save_buf(nvim_get_curbuf(), top, bot)
}

/// Save lines between top and bot for the specified buffer.
/// Rust implementation of u_save_buf.
///
/// # Safety
///
/// Must be called with valid buffer handle and line numbers.
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_u_savesub(lnum: LinenrT) -> c_int {
    rs_u_savecommon(nvim_get_curbuf(), lnum - 1, lnum + 1, lnum + 1, false)
}

/// Save for line insertion (used by :s command).
/// Rust implementation of u_inssub.
///
/// # Safety
///
/// Must be called with valid line number for curbuf.
#[no_mangle]
pub unsafe extern "C" fn rs_u_inssub(lnum: LinenrT) -> c_int {
    rs_u_savecommon(nvim_get_curbuf(), lnum - 1, lnum, lnum + 1, false)
}

/// Save lines for deletion.
/// Rust implementation of u_savedel.
///
/// # Safety
///
/// Must be called with valid line numbers for curbuf.
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_u_find_first_changed() {
    let curbuf = nvim_get_curbuf();
    let uhp = nvim_buf_get_b_u_newhead(curbuf);

    // If curhead is set or newhead is null, return early
    if !nvim_buf_get_b_u_curhead(curbuf).0.is_null() || uhp.0.is_null() {
        return; // undid something in an autocmd?
    }

    // Check that the last undo block was for the whole file
    let uep = nvim_uhp_get_entry(uhp);
    if nvim_uep_get_top(uep) != 0 || nvim_uep_get_bot(uep) != 0 {
        return;
    }

    let line_count = nvim_buf_get_ml_line_count(curbuf);
    let ue_size = nvim_uep_get_size(uep);

    // Find the first line that differs
    let mut lnum: LinenrT = 1;
    while lnum < line_count && lnum <= ue_size {
        // Compare buffer line at lnum with ue_array[lnum - 1]
        if nvim_uep_compare_line_with_array(uep, lnum - 1, curbuf, lnum) {
            nvim_uhp_clear_cursor(uhp);
            nvim_uhp_set_cursor_lnum_only(uhp, lnum);
            return;
        }
        lnum += 1;
    }

    // Lines added or deleted at the end, put cursor there
    if line_count != ue_size {
        nvim_uhp_clear_cursor(uhp);
        nvim_uhp_set_cursor_lnum_only(uhp, lnum);
    }
}

/// Restore the line saved for "U" command.
/// Rust implementation of u_undoline.
///
/// # Safety
///
/// Must be called from a valid Neovim context.
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_u_force_get_undo_header(buf: BufHandle) -> UHeaderHandle {
    let mut uhp = nvim_buf_get_b_u_curhead(buf);
    if uhp.0.is_null() {
        uhp = nvim_buf_get_b_u_newhead(buf);
    }

    // Create the first undo header for the buffer
    if uhp.0.is_null() {
        // Args are tricky: this means replace empty range by empty range
        rs_u_savecommon(buf, 0, 1, 1, true);

        uhp = nvim_buf_get_b_u_curhead(buf);
        if uhp.0.is_null() {
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
#[no_mangle]
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
            let uhp = if !curhead.0.is_null() {
                nvim_uhp_get_next(curhead)
            } else {
                nvim_buf_get_b_u_newhead(buf)
            };

            if !uhp.0.is_null() && nvim_uhp_get_save_nr(uhp) != 0 {
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
        let mut uhp = if curhead.0.is_null() {
            // at leaf of the tree
            nvim_buf_get_b_u_newhead(buf)
        } else {
            curhead
        };

        while !uhp.0.is_null() {
            nvim_uhp_set_walk(uhp, mark);
            let val = if dosec {
                nvim_uhp_get_time(uhp) as c_int
            } else if dofile {
                nvim_uhp_get_save_nr(uhp)
            } else {
                nvim_uhp_get_seq(uhp)
            };

            if round == 1 && !(dofile && val == 0) {
                // Remember the header that is closest to the target.
                // It must be at least in the right direction (checked with
                // "b_u_seq_cur"). When the timestamp is equal find the
                // highest/lowest sequence number.
                let uh_seq = nvim_uhp_get_seq(uhp);
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
                target = nvim_uhp_get_seq(uhp);
                break;
            }

            // go down in the tree if we haven't been there
            let prev = nvim_uhp_get_prev(uhp);
            if !prev.0.is_null()
                && nvim_uhp_get_walk(prev) != nomark
                && nvim_uhp_get_walk(prev) != mark
            {
                uhp = prev;
            } else {
                let alt_next = nvim_uhp_get_alt_next(uhp);
                if !alt_next.0.is_null()
                    && nvim_uhp_get_walk(alt_next) != nomark
                    && nvim_uhp_get_walk(alt_next) != mark
                {
                    // go to alternate branch if we haven't been there
                    uhp = alt_next;
                } else {
                    let next = nvim_uhp_get_next(uhp);
                    let alt_prev = nvim_uhp_get_alt_prev(uhp);
                    if !next.0.is_null()
                        && alt_prev.0.is_null()
                        && nvim_uhp_get_walk(next) != nomark
                        && nvim_uhp_get_walk(next) != mark
                    {
                        // go up in the tree if we haven't been there and we are at the
                        // start of alternate branches
                        // If still at the start we don't go through this change.
                        let curhead = nvim_buf_get_b_u_curhead(buf);
                        if uhp.0 == curhead.0 {
                            nvim_uhp_set_walk(uhp, nomark);
                        }
                        uhp = next;
                    } else {
                        // need to backtrack; mark this node as useless
                        nvim_uhp_set_walk(uhp, nomark);
                        if !alt_prev.0.is_null() {
                            uhp = alt_prev;
                        } else {
                            uhp = nvim_uhp_get_next(uhp);
                        }
                    }
                }
            }
        }

        if !uhp.0.is_null() {
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
        let uhp = if curhead.0.is_null() {
            nvim_buf_get_b_u_newhead(buf)
        } else {
            nvim_uhp_get_next(curhead)
        };

        if uhp.0.is_null()
            || (target > 0 && nvim_uhp_get_walk(uhp) != mark)
            || (nvim_uhp_get_seq(uhp) == target && !above)
        {
            break;
        }

        nvim_buf_set_b_u_curhead(buf, uhp);
        u_undoredo(true, true);
        if target > 0 {
            nvim_uhp_set_walk(uhp, nomark); // don't go back down here
        }
    }

    // When back to origin, redo is not needed.
    if target > 0 {
        // And now go down the tree (redo), branching off where needed.
        while !nvim_undo_got_int() {
            // Do the change warning now, for the same reason as above.
            nvim_change_warning_curbuf();

            let mut uhp = nvim_buf_get_b_u_curhead(buf);
            if uhp.0.is_null() {
                break;
            }

            // Go back to the first branch with a mark.
            let mut alt_prev = nvim_uhp_get_alt_prev(uhp);
            while !alt_prev.0.is_null() && nvim_uhp_get_walk(alt_prev) == mark {
                uhp = alt_prev;
                alt_prev = nvim_uhp_get_alt_prev(uhp);
            }

            // Find the last branch with a mark, that's the one.
            let mut last = uhp;
            let mut alt_next = nvim_uhp_get_alt_next(last);
            while !alt_next.0.is_null() && nvim_uhp_get_walk(alt_next) == mark {
                last = alt_next;
                alt_next = nvim_uhp_get_alt_next(last);
            }

            if last.0 != uhp.0 {
                // Make the used branch the first entry in the list of
                // alternatives to make "u" and CTRL-R take this branch.
                let mut first = uhp;
                let mut first_alt_prev = nvim_uhp_get_alt_prev(first);
                while !first_alt_prev.0.is_null() {
                    first = first_alt_prev;
                    first_alt_prev = nvim_uhp_get_alt_prev(first);
                }

                let last_alt_next = nvim_uhp_get_alt_next(last);
                if !last_alt_next.0.is_null() {
                    let last_alt_prev = nvim_uhp_get_alt_prev(last);
                    nvim_uhp_set_alt_prev(last_alt_next, last_alt_prev);
                }

                let last_alt_prev = nvim_uhp_get_alt_prev(last);
                nvim_uhp_set_alt_next(last_alt_prev, nvim_uhp_get_alt_next(last));
                nvim_uhp_set_alt_prev(last, UHeaderHandle(std::ptr::null_mut()));
                nvim_uhp_set_alt_next(last, first);
                nvim_uhp_set_alt_prev(first, last);

                let oldhead = nvim_buf_get_b_u_oldhead(buf);
                if oldhead.0 == first.0 {
                    nvim_buf_set_b_u_oldhead(buf, last);
                }

                uhp = last;
                let next = nvim_uhp_get_next(uhp);
                if !next.0.is_null() {
                    nvim_uhp_set_prev(next, uhp);
                }
            }

            nvim_buf_set_b_u_curhead(buf, uhp);

            if nvim_uhp_get_walk(uhp) != mark {
                break; // must have reached the target
            }

            // Stop when going backwards in time and didn't find the exact
            // header we were looking for.
            if nvim_uhp_get_seq(uhp) == target && above {
                nvim_buf_set_b_u_seq_cur(buf, target - 1);
                break;
            }

            u_undoredo(false, true);

            // Advance "curhead" to below the header we last used. If it
            // becomes NULL then we need to set "newhead" to this leaf.
            let prev = nvim_uhp_get_prev(uhp);
            if prev.0.is_null() {
                nvim_buf_set_b_u_newhead(buf, uhp);
            }
            nvim_buf_set_b_u_curhead(buf, prev);
            *did_undo = false;

            if nvim_uhp_get_seq(uhp) == target {
                // found it!
                break;
            }

            let prev = nvim_uhp_get_prev(uhp);
            if prev.0.is_null() || nvim_uhp_get_walk(prev) != mark {
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
#[no_mangle]
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

    while !uhp.0.is_null() {
        count += 1;
        // Count alternate branches
        let mut alt = nvim_uhp_get_alt_next(uhp);
        while !alt.0.is_null() {
            count += rs_undo_branch_count(alt);
            alt = nvim_uhp_get_alt_next(alt);
        }
        uhp = nvim_uhp_get_prev(uhp);
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
    if uhp.0.is_null() {
        return 0;
    }

    let mut count: c_int = 1;
    let mut current = nvim_uhp_get_prev(uhp);

    while !current.0.is_null() {
        count += 1;
        // Count alternate branches
        let mut alt = nvim_uhp_get_alt_next(current);
        while !alt.0.is_null() {
            count += rs_undo_branch_count(alt);
            alt = nvim_uhp_get_alt_next(alt);
        }
        current = nvim_uhp_get_prev(current);
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

    while !uhp.0.is_null() {
        if nvim_uhp_get_seq(uhp) == seq {
            return uhp;
        }

        // Check alternate branches
        let mut alt = nvim_uhp_get_alt_next(uhp);
        while !alt.0.is_null() {
            let found = rs_undo_find_seq_in_branch(alt, seq);
            if !found.0.is_null() {
                return found;
            }
            alt = nvim_uhp_get_alt_next(alt);
        }

        uhp = nvim_uhp_get_next(uhp);
    }

    UHeaderHandle(std::ptr::null_mut())
}

/// Find sequence number in a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
unsafe fn rs_undo_find_seq_in_branch(uhp: UHeaderHandle, seq: c_int) -> UHeaderHandle {
    if uhp.0.is_null() {
        return UHeaderHandle(std::ptr::null_mut());
    }

    if nvim_uhp_get_seq(uhp) == seq {
        return uhp;
    }

    let mut current = nvim_uhp_get_prev(uhp);
    while !current.0.is_null() {
        if nvim_uhp_get_seq(current) == seq {
            return current;
        }

        // Check alternate branches
        let mut alt = nvim_uhp_get_alt_next(current);
        while !alt.0.is_null() {
            let found = rs_undo_find_seq_in_branch(alt, seq);
            if !found.0.is_null() {
                return found;
            }
            alt = nvim_uhp_get_alt_next(alt);
        }

        current = nvim_uhp_get_prev(current);
    }

    UHeaderHandle(std::ptr::null_mut())
}

/// Count the number of undo entries in a header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_count_entries(uhp: UHeaderHandle) -> c_int {
    if uhp.0.is_null() {
        return 0;
    }

    let mut count: c_int = 0;
    let mut uep = nvim_uhp_get_entry(uhp);

    while !uep.0.is_null() {
        count += 1;
        uep = nvim_uep_get_next(uep);
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
    if uhp.0.is_null() {
        return 0;
    }

    let mut max_depth: c_int = 0;

    // Check this branch
    let mut current = uhp;
    let mut depth: c_int = 0;
    while !current.0.is_null() {
        depth += 1;

        // Check alternate branches
        let mut alt = nvim_uhp_get_alt_next(current);
        while !alt.0.is_null() {
            let alt_depth = rs_undo_get_branch_depth(alt);
            if depth + alt_depth > max_depth {
                max_depth = depth + alt_depth;
            }
            alt = nvim_uhp_get_alt_next(alt);
        }

        current = nvim_uhp_get_prev(current);
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
    if curhead.0.is_null() {
        !newhead.0.is_null()
    } else {
        !nvim_uhp_get_next(curhead).0.is_null()
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
    !curhead.0.is_null()
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
    nvim_uhp_get_seq(uhp)
}

/// Get the time from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_time(uhp: UHeaderHandle) -> TimeT {
    nvim_uhp_get_time(uhp)
}

/// Get the flags from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_flags(uhp: UHeaderHandle) -> c_int {
    nvim_uhp_get_flags(uhp)
}

/// Get the save number from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_save_nr(uhp: UHeaderHandle) -> c_int {
    nvim_uhp_get_save_nr(uhp)
}

/// Get the next header in the list.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_next(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_next(uhp)
}

/// Get the previous header in the list.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_prev(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_prev(uhp)
}

/// Get the next alternate header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_alt_next(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_alt_next(uhp)
}

/// Get the previous alternate header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_alt_prev(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_alt_prev(uhp)
}

/// Get the first entry in an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_entry(uhp: UHeaderHandle) -> UEntryHandle {
    nvim_uhp_get_entry(uhp)
}

/// Check if an undo header is NULL.
#[no_mangle]
pub extern "C" fn rs_uhp_is_null(uhp: UHeaderHandle) -> bool {
    uhp.0.is_null()
}

/// Check if an undo entry is NULL.
#[no_mangle]
pub extern "C" fn rs_uep_is_null(uep: UEntryHandle) -> bool {
    uep.0.is_null()
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
    nvim_uep_get_top(uep)
}

/// Get the bottom line number from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_bot(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_bot(uep)
}

/// Get the line count from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_lcount(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_lcount(uep)
}

/// Get the size (number of lines) from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_size(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_size(uep)
}

/// Get the next entry in the list.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_next(uep: UEntryHandle) -> UEntryHandle {
    nvim_uep_get_next(uep)
}

/// Get a line from the undo entry's array.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
/// The index must be valid (0 <= idx < size).
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_line(uep: UEntryHandle, idx: LinenrT) -> *const c_char {
    nvim_uep_get_array_element(uep, idx)
}

/// Get the number of lines affected by an undo entry.
/// This is the number of lines that will be replaced (bot - top - 1).
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_line_count(uep: UEntryHandle) -> LinenrT {
    let top = nvim_uep_get_top(uep);
    let bot = nvim_uep_get_bot(uep);
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
        nvim_uhp_get_visual_start_lnum(uhp),
        nvim_uhp_get_visual_start_col(uhp),
        nvim_uhp_get_visual_start_coladd(uhp),
    ) &&
    // vi_end
    serialize_pos(
        fp,
        nvim_uhp_get_visual_end_lnum(uhp),
        nvim_uhp_get_visual_end_col(uhp),
        nvim_uhp_get_visual_end_coladd(uhp),
    ) &&
    // vi_mode
    undo_write_4(fp, nvim_uhp_get_visual_mode(uhp)) &&
    // vi_curswant
    undo_write_4(fp, nvim_uhp_get_visual_curswant(uhp))
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
    let seq = if uhp.0.is_null() {
        0
    } else {
        nvim_uhp_get_seq(uhp) as u64
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
    if !undo_write_4(fp, nvim_uep_get_top(uep) as i32) {
        return false;
    }
    if !undo_write_4(fp, nvim_uep_get_bot(uep) as i32) {
        return false;
    }
    if !undo_write_4(fp, nvim_uep_get_lcount(uep) as i32) {
        return false;
    }

    let size = nvim_uep_get_size(uep);
    if !undo_write_4(fp, size as i32) {
        return false;
    }

    // Write each line in the array
    for i in 0..size {
        let line = nvim_uep_get_array_element(uep, i);
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
    if !put_header_ptr_by_seq(fp, nvim_uhp_get_next_seq(uhp)) {
        return false;
    }
    if !put_header_ptr_by_seq(fp, nvim_uhp_get_prev_seq(uhp)) {
        return false;
    }
    if !put_header_ptr_by_seq(fp, nvim_uhp_get_alt_next_seq(uhp)) {
        return false;
    }
    if !put_header_ptr_by_seq(fp, nvim_uhp_get_alt_prev_seq(uhp)) {
        return false;
    }

    // Write sequence number
    if !undo_write_4(fp, nvim_uhp_get_seq(uhp)) {
        return false;
    }

    // Write cursor position
    if !serialize_pos(
        fp,
        nvim_uhp_get_cursor_lnum(uhp),
        nvim_uhp_get_cursor_col(uhp),
        nvim_uhp_get_cursor_coladd(uhp),
    ) {
        return false;
    }

    // Write cursor vcol
    if !undo_write_4(fp, nvim_uhp_get_cursor_vcol(uhp)) {
        return false;
    }

    // Write flags (2 bytes)
    if !undo_write_2(fp, nvim_uhp_get_flags(uhp) as u16) {
        return false;
    }

    // Write named marks (NMARKS = 26)
    for i in 0..NMARKS as c_int {
        if !serialize_pos(
            fp,
            nvim_uhp_get_namedm_lnum(uhp, i),
            nvim_uhp_get_namedm_col(uhp, i),
            nvim_uhp_get_namedm_coladd(uhp, i),
        ) {
            return false;
        }
    }

    // Write visual info
    if !serialize_visualinfo(fp, uhp) {
        return false;
    }

    // Write time (8 bytes)
    if !undo_write_time(fp, nvim_uhp_get_time(uhp)) {
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
    if !undo_write_4(fp, nvim_uhp_get_save_nr(uhp)) {
        return false;
    }

    // Write end marker for optional fields
    if !undo_write_bytes(fp, 0, 1) {
        return false;
    }

    // Write all undo entries
    let mut uep = nvim_uhp_get_entry(uhp);
    while !uep.0.is_null() {
        if !undo_write_2(fp, UF_ENTRY_MAGIC) {
            return false;
        }
        if !serialize_uep(fp, uep) {
            return false;
        }
        uep = nvim_uep_get_next(uep);
    }

    // Write entry end magic
    if !undo_write_2(fp, UF_ENTRY_END_MAGIC) {
        return false;
    }

    // Write all extmark undo objects
    let extmark_count = nvim_uhp_get_extmark_count(uhp);
    for i in 0..extmark_count {
        let ext_type = nvim_uhp_get_extmark_type(uhp, i);
        // Only serialize splice and move types
        if ext_type == 1 || ext_type == 2 {
            // kExtmarkSplice = 1, kExtmarkMove = 2
            if !undo_write_2(fp, UF_ENTRY_MAGIC) {
                return false;
            }
            if !undo_write_4(fp, ext_type) {
                return false;
            }
            // Get the extmark data and write it
            let mut buf = [0u8; 128]; // Large enough for ExtmarkSplice or ExtmarkMove
            let size = if ext_type == 1 { 72 } else { 48 }; // Approximate sizes
            nvim_uhp_get_extmark_data(uhp, i, buf.as_mut_ptr(), size);
            if !undo_write(fp, buf.as_ptr(), size) {
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
#[no_mangle]
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
#[no_mangle]
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
    nvim_u_sync(true);

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
    while !uhp.0.is_null() {
        // Serialize current UHP if we haven't seen it
        if nvim_uhp_get_walk(uhp) != mark {
            nvim_uhp_set_walk(uhp, mark);
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
        let prev = nvim_uhp_get_prev(uhp);
        if !prev.0.is_null() && nvim_uhp_get_walk(prev) != mark {
            uhp = prev;
        } else {
            let alt_next = nvim_uhp_get_alt_next(uhp);
            if !alt_next.0.is_null() && nvim_uhp_get_walk(alt_next) != mark {
                uhp = alt_next;
            } else {
                let next = nvim_uhp_get_next(uhp);
                let alt_prev = nvim_uhp_get_alt_prev(uhp);
                if !next.0.is_null() && alt_prev.0.is_null() && nvim_uhp_get_walk(next) != mark {
                    uhp = next;
                } else if !alt_prev.0.is_null() {
                    uhp = alt_prev;
                } else {
                    uhp = nvim_uhp_get_next(uhp);
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
    fn nvim_os_fopen(path: *const c_char, mode: *const c_char) -> FileHandle;
    fn nvim_u_blockfree(buf: BufHandle);
    fn nvim_u_free_uhp(uhp: UHeaderHandle);
    fn nvim_undo_check_owner(orig_name: *const c_char, file_name: *const c_char) -> bool;

    // Get uh_seq from a header in seq mode (for pointer swizzling)
    fn nvim_uhp_get_next_seq_for_swizzle(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_prev_seq_for_swizzle(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_alt_next_seq_for_swizzle(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_alt_prev_seq_for_swizzle(uhp: UHeaderHandle) -> c_int;

    // Deserialization setters
    fn nvim_uhp_set_next_seq(uhp: UHeaderHandle, seq: c_int);
    fn nvim_uhp_set_prev_seq(uhp: UHeaderHandle, seq: c_int);
    fn nvim_uhp_set_alt_next_seq(uhp: UHeaderHandle, seq: c_int);
    fn nvim_uhp_set_alt_prev_seq(uhp: UHeaderHandle, seq: c_int);
    fn nvim_uhp_set_namedm(
        uhp: UHeaderHandle,
        idx: c_int,
        lnum: LinenrT,
        col: ColnrT,
        coladd: ColnrT,
        timestamp: u64,
        fnum: c_int,
    );
    fn nvim_uhp_set_visual(
        uhp: UHeaderHandle,
        start_lnum: LinenrT,
        start_col: ColnrT,
        start_coladd: ColnrT,
        end_lnum: LinenrT,
        end_col: ColnrT,
        end_coladd: ColnrT,
        mode: c_int,
        curswant: ColnrT,
    );
    fn nvim_uhp_push_extmark_splice(uhp: UHeaderHandle, data: *const u8, size: usize);
    fn nvim_uhp_push_extmark_move(uhp: UHeaderHandle, data: *const u8, size: usize);
    fn nvim_sizeof_extmark_splice() -> usize;
    fn nvim_sizeof_extmark_move() -> usize;
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
    nvim_uhp_set_visual(
        uhp,
        start_lnum,
        start_col,
        start_coladd,
        end_lnum,
        end_col,
        end_coladd,
        mode,
        curswant,
    );
}

/// Deserialize an extmark undo object from the undo file.
/// Returns true on success, false on error.
///
/// # Safety
///
/// - `fp` must be a valid file handle
/// - `uhp` must be a valid undo header handle
unsafe fn unserialize_extmark(fp: FileHandle, uhp: UHeaderHandle) -> bool {
    let ext_type = undo_read_4c(fp);

    // kExtmarkSplice = 0
    if ext_type == 0 {
        let size = nvim_sizeof_extmark_splice();
        let mut buf = vec![0u8; size];
        if !undo_read(fp, buf.as_mut_ptr(), size) {
            return false;
        }
        nvim_uhp_push_extmark_splice(uhp, buf.as_ptr(), size);
        return true;
    }

    // kExtmarkMove = 1
    if ext_type == 1 {
        let size = nvim_sizeof_extmark_move();
        let mut buf = vec![0u8; size];
        if !undo_read(fp, buf.as_mut_ptr(), size) {
            return false;
        }
        nvim_uhp_push_extmark_move(uhp, buf.as_ptr(), size);
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
    let uep = nvim_alloc_u_entry();
    // Fields are zero-initialized by xcalloc in nvim_alloc_u_entry

    nvim_uep_set_top(uep, undo_read_4c(fp));
    nvim_uep_set_bot(uep, undo_read_4c(fp));
    nvim_uep_set_lcount(uep, undo_read_4c(fp));
    let size = undo_read_4c(fp);
    nvim_uep_set_size(uep, size);

    if size > 0 {
        let ptr_size = std::mem::size_of::<*mut c_char>();
        if (size as usize) < usize::MAX / ptr_size {
            let array = nvim_xmallocz(ptr_size * size as usize) as *mut *mut c_char;
            // Zero out the array
            ptr::write_bytes(array, 0, size as usize);
            nvim_uep_set_array(uep, array);

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
    let uhp = nvim_alloc_u_header();
    // Fields are zero-initialized by xcalloc in nvim_alloc_u_header

    // Read sequence numbers for pointer swizzling
    nvim_uhp_set_next_seq(uhp, undo_read_4c(fp));
    nvim_uhp_set_prev_seq(uhp, undo_read_4c(fp));
    nvim_uhp_set_alt_next_seq(uhp, undo_read_4c(fp));
    nvim_uhp_set_alt_prev_seq(uhp, undo_read_4c(fp));

    let seq = undo_read_4c(fp);
    nvim_uhp_set_seq(uhp, seq);
    if seq <= 0 {
        nvim_undo_corruption_error(c"uh_seq".as_ptr(), file_name);
        nvim_xfree(uhp.0);
        return UHeaderHandle(ptr::null_mut());
    }

    // Read cursor position
    let (cursor_lnum, cursor_col, cursor_coladd) = unserialize_pos(fp);
    nvim_uhp_set_cursor(uhp, cursor_lnum, cursor_col, cursor_coladd);
    nvim_uhp_set_cursor_vcol(uhp, undo_read_4c(fp));
    nvim_uhp_set_flags(uhp, undo_read_2c(fp));

    // Read named marks
    let cur_timestamp = nvim_undo_os_time() as u64;
    for i in 0..NMARKS as c_int {
        let (lnum, col, coladd) = unserialize_pos(fp);
        nvim_uhp_set_namedm(uhp, i, lnum, col, coladd, cur_timestamp, 0);
    }

    // Read visual info
    unserialize_visualinfo(fp, uhp);

    // Read time
    nvim_uhp_set_time(uhp, undo_read_time(fp));

    // Read optional fields
    loop {
        let len = undo_read_byte(fp);
        if len == -1 {
            // EOF
            nvim_undo_corruption_error(c"truncated".as_ptr(), file_name);
            nvim_u_free_uhp(uhp);
            return UHeaderHandle(ptr::null_mut());
        }
        if len == 0 {
            break;
        }
        let what = undo_read_byte(fp);
        if what == UHP_SAVE_NR as c_int {
            nvim_uhp_set_save_nr(uhp, undo_read_4c(fp));
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
    let mut last_uep = UEntryHandle(ptr::null_mut());
    loop {
        let c = undo_read_2c(fp);
        if c != UF_ENTRY_MAGIC as c_int {
            if c != UF_ENTRY_END_MAGIC as c_int {
                nvim_undo_corruption_error(c"entry end".as_ptr(), file_name);
                nvim_u_free_uhp(uhp);
                return UHeaderHandle(ptr::null_mut());
            }
            break;
        }
        let (uep, error) = unserialize_uep(fp, file_name);
        if last_uep.0.is_null() {
            nvim_uhp_set_entry(uhp, uep);
        } else {
            nvim_uep_set_next(last_uep, uep);
        }
        last_uep = uep;
        if uep.0.is_null() || error {
            nvim_u_free_uhp(uhp);
            return UHeaderHandle(ptr::null_mut());
        }
    }

    // Read extmark undo information
    nvim_uhp_init_extmark(uhp);
    loop {
        let c = undo_read_2c(fp);
        if c != UF_ENTRY_MAGIC as c_int {
            if c != UF_ENTRY_END_MAGIC as c_int {
                nvim_undo_corruption_error(c"entry end".as_ptr(), file_name);
                nvim_uhp_destroy_extmark(uhp);
                nvim_u_free_uhp(uhp);
                return UHeaderHandle(ptr::null_mut());
            }
            break;
        }
        if !unserialize_extmark(fp, uhp) {
            nvim_uhp_destroy_extmark(uhp);
            nvim_u_free_uhp(uhp);
            return UHeaderHandle(ptr::null_mut());
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
#[no_mangle]
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
        if uhp.0.is_null() {
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
        if uhp.0.is_null() {
            continue;
        }

        // Check for duplicate sequence numbers
        let this_seq = nvim_uhp_get_seq(uhp);
        for j in 0..num_head {
            if i != j {
                let other = *uhp_table.add(j as usize);
                if !other.0.is_null() && nvim_uhp_get_seq(other) == this_seq {
                    nvim_undo_corruption_error(c"duplicate uh_seq".as_ptr(), file_name);
                    cleanup_read_error(name, file_name, fp, line_ptr, uhp_table, num_read_uhps);
                    return;
                }
            }
        }

        // Swizzle uh_next
        let next_seq = nvim_uhp_get_next_seq_for_swizzle(uhp);
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.0.is_null() && nvim_uhp_get_seq(candidate) == next_seq {
                nvim_uhp_set_next(uhp, candidate);
                break;
            }
        }

        // Swizzle uh_prev
        let prev_seq = nvim_uhp_get_prev_seq_for_swizzle(uhp);
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.0.is_null() && nvim_uhp_get_seq(candidate) == prev_seq {
                nvim_uhp_set_prev(uhp, candidate);
                break;
            }
        }

        // Swizzle uh_alt_next
        let alt_next_seq = nvim_uhp_get_alt_next_seq_for_swizzle(uhp);
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.0.is_null() && nvim_uhp_get_seq(candidate) == alt_next_seq {
                nvim_uhp_set_alt_next(uhp, candidate);
                break;
            }
        }

        // Swizzle uh_alt_prev
        let alt_prev_seq = nvim_uhp_get_alt_prev_seq_for_swizzle(uhp);
        for j in 0..num_head {
            let candidate = *uhp_table.add(j as usize);
            if !candidate.0.is_null() && nvim_uhp_get_seq(candidate) == alt_prev_seq {
                nvim_uhp_set_alt_prev(uhp, candidate);
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
    nvim_u_blockfree(buf);

    let oldhead = if old_idx < 0 {
        UHeaderHandle(ptr::null_mut())
    } else {
        *uhp_table.add(old_idx as usize)
    };
    let newhead = if new_idx < 0 {
        UHeaderHandle(ptr::null_mut())
    } else {
        *uhp_table.add(new_idx as usize)
    };
    let curhead = if cur_idx < 0 {
        UHeaderHandle(ptr::null_mut())
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
            if !uhp.0.is_null() {
                nvim_u_free_uhp(uhp);
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
    fn nvim_msg_start();
    fn nvim_msg_end();
    fn nvim_undo_msg_puts_hl_title(msg: *const c_char);
    fn nvim_undo_msg_putchar(c: c_int);
    fn nvim_undo_msg_puts(msg: *const c_char);
    fn nvim_undolist_format_entry(
        uhp: UHeaderHandle,
        changes: c_int,
        buf: *mut c_char,
        buf_size: usize,
    );
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
#[no_mangle]
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
    while !uhp.0.is_null() {
        let prev = nvim_uhp_get_prev(uhp);

        if prev.0.is_null() && nvim_uhp_get_walk(uhp) != nomark && nvim_uhp_get_walk(uhp) != mark {
            // Format the entry
            let entry_buf: [c_char; 256] = [0; 256];
            nvim_undolist_format_entry(uhp, changes, entry_buf.as_ptr() as *mut c_char, 256);
            let entry_str = nvim_undo_xstrdup(entry_buf.as_ptr());
            let seq = nvim_uhp_get_seq(uhp);
            entries.push((entry_str, seq));
        }

        nvim_uhp_set_walk(uhp, mark);

        // go down in the tree if we haven't been there
        if !prev.0.is_null() && nvim_uhp_get_walk(prev) != nomark && nvim_uhp_get_walk(prev) != mark
        {
            uhp = prev;
            changes += 1;
        } else {
            let alt_next = nvim_uhp_get_alt_next(uhp);
            if !alt_next.0.is_null()
                && nvim_uhp_get_walk(alt_next) != nomark
                && nvim_uhp_get_walk(alt_next) != mark
            {
                // go to alternate branch if we haven't been there
                uhp = alt_next;
            } else {
                let next = nvim_uhp_get_next(uhp);
                let alt_prev = nvim_uhp_get_alt_prev(uhp);
                if !next.0.is_null()
                    && alt_prev.0.is_null()
                    && nvim_uhp_get_walk(next) != nomark
                    && nvim_uhp_get_walk(next) != mark
                {
                    // go up in the tree if we haven't been there and we are at the
                    // start of alternate branches
                    uhp = next;
                    changes -= 1;
                } else {
                    // need to backtrack; mark this node as done
                    nvim_uhp_set_walk(uhp, nomark);
                    if !alt_prev.0.is_null() {
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
    fn nvim_tv_dict_alloc() -> DictHandle;
    fn nvim_tv_list_append_dict(list: ListHandle, dict: DictHandle);
    fn nvim_tv_dict_add_nr(dict: DictHandle, key: *const c_char, key_len: usize, nr: i64);
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
    while !uhp.0.is_null() {
        let dict = nvim_tv_dict_alloc();

        // Add seq
        let seq = nvim_uhp_get_seq(uhp) as i64;
        nvim_tv_dict_add_nr(dict, c"seq".as_ptr(), 3, seq);

        // Add time
        let time = nvim_uhp_get_time(uhp);
        nvim_tv_dict_add_nr(dict, c"time".as_ptr(), 4, time);

        // Add newhead marker if applicable
        if uhp.0 == newhead.0 {
            nvim_tv_dict_add_nr(dict, c"newhead".as_ptr(), 7, 1);
        }

        // Add curhead marker if applicable
        if uhp.0 == curhead.0 {
            nvim_tv_dict_add_nr(dict, c"curhead".as_ptr(), 7, 1);
        }

        // Add save number if > 0
        let save_nr = nvim_uhp_get_save_nr(uhp);
        if save_nr > 0 {
            nvim_tv_dict_add_nr(dict, c"save".as_ptr(), 4, save_nr as i64);
        }

        // Recurse for alternate branches
        let alt_next = nvim_uhp_get_alt_next(uhp);
        if !alt_next.0.is_null() {
            let alt_list = rs_u_eval_tree(buf, alt_next);
            nvim_tv_dict_add_list(dict, c"alt".as_ptr(), 3, alt_list);
        }

        nvim_tv_list_append_dict(list, dict);

        // Move to previous entry
        uhp = nvim_uhp_get_prev(uhp);
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
        assert!(rs_uhp_is_null(UHeaderHandle(std::ptr::null_mut())));
        assert!(rs_uep_is_null(UEntryHandle(std::ptr::null_mut())));
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
        // UHeaderHandle should be repr(transparent) over a pointer
        let ptr: *mut c_void = 0x5678 as *mut c_void;
        let handle = UHeaderHandle(ptr);
        assert_eq!(handle.0, ptr);
    }

    #[test]
    fn test_uentry_handle_repr() {
        // UEntryHandle should be repr(transparent) over a pointer
        let ptr: *mut c_void = 0xabcd as *mut c_void;
        let handle = UEntryHandle(ptr);
        assert_eq!(handle.0, ptr);
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
