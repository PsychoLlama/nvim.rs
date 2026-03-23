//! Change tracking for Neovim buffers.
//!
//! This crate provides Rust implementations of buffer modification tracking,
//! change events, and text editing primitives from `src/nvim/change.c`.
//!
//! ## Overview
//!
//! The change module handles:
//! - Buffer modification tracking (marking buffers as changed)
//! - Change event notifications (line/byte changes)
//! - Display invalidation after changes
//! - Text editing primitives (insert/delete bytes/chars)
//! - Line operations (truncate, delete lines)
//! - Comment leader parsing
//! - File format state tracking
//! - The complex `open_line()` function for creating new lines

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int, c_long, c_void};

pub mod comment;
pub mod editing;
pub mod events;
pub mod file_format;
pub mod flags;
pub mod invalidation;
pub mod lines;
pub mod open_line;
pub mod recording;

pub use comment::{rs_get_last_leader_offset, rs_get_leader_len};
pub use editing::{
    rs_del_bytes, rs_del_char, rs_del_chars, rs_ins_bytes, rs_ins_bytes_len, rs_ins_char,
    rs_ins_char_bytes, rs_ins_str,
};
pub use events::{
    rs_appended_lines, rs_appended_lines_buf, rs_appended_lines_mark, rs_changed_bytes,
    rs_changed_lines, rs_changed_lines_redraw_buf, rs_deleted_lines, rs_deleted_lines_buf,
    rs_deleted_lines_mark, rs_inserted_bytes,
};
pub use file_format::{rs_file_ff_differs, rs_save_file_ff, rs_unchanged};
pub use flags::OpenlineFlags;
pub use invalidation::rs_changed_lines_invalidate_buf;
pub use lines::{rs_del_lines, rs_truncate_line};
pub use open_line::{rs_open_line, rs_open_line_backward, rs_open_line_forward};
pub use recording::{rs_change_warning, rs_changed, rs_changed_internal};

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a Neovim buffer (`buf_T*`).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Create a null buffer handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a Neovim window (`win_T*`).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Create a null window handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

// =============================================================================
// Type Aliases
// =============================================================================

/// Type alias for linenr_T (line number type).
pub type LinenrT = i32;

/// Type alias for colnr_T (column number type).
pub type ColnrT = c_int;

/// Type alias for bcount_t (byte count type).
pub type BcountT = i64;

/// Success return value (matches Neovim's OK).
pub const OK: c_int = 1;

/// Failure return value (matches Neovim's FAIL).
pub const FAIL: c_int = 0;

/// NUL character constant.
pub const NUL: c_char = 0;

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Buffer field accessors
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_changed(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_changed_invalid(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_changed_invalid(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_did_warn(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_did_warn(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_ro_locked(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_ro_locked(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_may_swap(buf: BufHandle) -> bool;
    fn nvim_bt_dontwrite(buf: BufHandle) -> bool;

    // Buffer modification tracking
    fn nvim_buf_get_b_mod_set(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_mod_set(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_mod_top(buf: BufHandle) -> LinenrT;
    fn nvim_buf_set_b_mod_top(buf: BufHandle, val: LinenrT);
    fn nvim_buf_get_b_mod_bot(buf: BufHandle) -> LinenrT;
    fn nvim_buf_set_b_mod_bot(buf: BufHandle, val: LinenrT);
    fn nvim_buf_get_b_mod_xlines(buf: BufHandle) -> LinenrT;
    fn nvim_buf_set_b_mod_xlines(buf: BufHandle, val: LinenrT);

    // File format accessors
    fn nvim_buf_get_b_start_ffc(buf: BufHandle) -> c_char;
    fn nvim_buf_set_b_start_ffc(buf: BufHandle, val: c_char);
    fn nvim_buf_get_b_start_eof(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_start_eof(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_start_eol(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_start_eol(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_start_bomb(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_start_bomb(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_start_fenc(buf: BufHandle) -> *mut c_char;
    fn nvim_buf_set_b_start_fenc(buf: BufHandle, val: *mut c_char);
    fn nvim_buf_get_b_p_ff_first_char(buf: BufHandle) -> c_char;
    fn nvim_buf_get_b_p_eof(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_eol(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_bomb(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_bin(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_fixeol(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_fenc(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_ml_ml_line_count(buf: BufHandle) -> LinenrT;

    // Global state accessors
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_autocmd_busy() -> bool;
    fn nvim_get_highlight_match() -> c_int;
    fn nvim_set_highlight_match(val: c_int);
    fn nvim_curbufIsChanged() -> c_int;

    // Message functions
    fn nvim_msg_start();
    fn nvim_msg_source(attr: c_int);
    fn nvim_msg_ext_set_kind(kind: *const c_char);
    fn nvim_msg_puts_hl(msg: *const c_char, attr: c_int, right: bool);
    fn nvim_msg_clr_eos();
    fn nvim_msg_end();
    fn nvim_msg_silent() -> c_int;
    fn nvim_silent_mode() -> bool;
    fn nvim_ui_active() -> bool;
    fn nvim_ui_has_messages() -> c_int;
    fn nvim_set_vim_var_string(idx: c_int, val: *const c_char, len: c_int);

    // Redraw functions
    fn nvim_redraw_buf_status_later(buf: BufHandle);
    fn nvim_set_redraw_cmdline(val: bool);

    // Other functions
    fn nvim_ml_setflags(buf: BufHandle);
    fn nvim_ml_open_file(buf: BufHandle);
    fn nvim_buf_inc_changedtick(buf: BufHandle);
    fn nvim_apply_autocmds_filechangedro(buf: BufHandle);
    fn nvim_showmode();
    fn nvim_ui_flush();
    fn nvim_os_delay(ms: c_long, allow_input: bool);
    fn nvim_wait_return(redraw: bool);
    fn nvim_get_msg_scroll() -> c_int;
    fn nvim_set_msg_scroll(val: c_int);
    static mut need_wait_return: bool;
    static mut emsg_silent: c_int;
    fn nvim_in_assert_fails() -> bool;
    static mut msg_row: c_int;
    static mut msg_col: c_int;

    // Buffer updates
    fn nvim_buf_updates_send_changes(buf: BufHandle, lnum: LinenrT, added: i64, removed: i64);

    // Extmark operations
    fn nvim_extmark_splice_cols(
        buf: BufHandle,
        lnum: c_int,
        col: ColnrT,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );

    // Window accessors
    fn nvim_win_get_buffer(win: WinHandle) -> BufHandle;
    fn nvim_win_get_cursor_lnum(win: WinHandle) -> LinenrT;
    fn nvim_win_get_cursor_col(win: WinHandle) -> ColnrT;
    fn nvim_win_set_cursor_col(win: WinHandle, col: ColnrT);

    // Memory allocation
    fn nvim_xfree(ptr: *mut c_void);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;

    // String comparison
    fn nvim_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Marktree accessors
    fn nvim_buf_marktree_n_keys(buf: BufHandle) -> c_int;
    fn nvim_buf_meta_total(buf: BufHandle, meta_type: c_int) -> c_int;
}

// =============================================================================
// Constants for FFI
// =============================================================================

/// Highlight attribute for warnings.
pub const HLF_W: c_int = 26;

/// VimL variable index for v:warningmsg.
pub const VV_WARNINGMSG: c_int = 4;

/// Buffer flag: buffer never loaded.
pub const BF_NEVERLOADED: c_int = 0x04;

/// Buffer flag: new buffer.
pub const BF_NEW: c_int = 0x10;

/// Extmark undo operation type.
pub const KEXTMARK_UNDO: c_int = 0;

/// Meta type for inline virtual text.
pub const KMT_META_INLINE: c_int = 0;

/// Meta type for virtual lines.
pub const KMT_META_LINES: c_int = 1;

// Re-export the flags module contents
pub use flags::*;
