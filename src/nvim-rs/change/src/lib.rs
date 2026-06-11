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

use std::ffi::{c_char, c_int, c_void};

use nvim_buffer::buf_struct::BufStruct;
use nvim_window::win_struct::WinStruct;

/// Convert a local WinHandle to nvim_window::WinHandle for WinStruct access.
#[inline]
unsafe fn win_ref<'a>(wp: WinHandle) -> &'a WinStruct {
    nvim_window::win_struct::win_ref(std::mem::transmute::<WinHandle, nvim_window::WinHandle>(wp))
}

/// Convert a local WinHandle to nvim_window::WinHandle for mutable WinStruct access.
#[inline]
unsafe fn win_mut<'a>(wp: WinHandle) -> &'a mut WinStruct {
    nvim_window::win_struct::win_mut(std::mem::transmute::<WinHandle, nvim_window::WinHandle>(wp))
}

/// Convert a local BufHandle to a reference to BufStruct for direct field access.
#[inline]
unsafe fn buf_ref<'a>(bp: BufHandle) -> &'a BufStruct {
    nvim_buffer::buf_struct::buf_ref(std::mem::transmute::<BufHandle, nvim_buffer::BufHandle>(bp))
}

/// Convert a local BufHandle to a mutable reference to BufStruct for direct field access.
#[inline]
unsafe fn buf_mut<'a>(bp: BufHandle) -> &'a mut BufStruct {
    nvim_buffer::buf_struct::buf_mut(std::mem::transmute::<BufHandle, nvim_buffer::BufHandle>(bp))
}

pub mod comment;
pub mod common;
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

/// Extmark undo operation type (kExtmarkUndo=1 per extmark_defs.h).
pub const KEXTMARK_UNDO: c_int = 1;
// Guard: must match kExtmarkUndo in src/nvim/extmark_defs.h
const _: () = assert!(
    KEXTMARK_UNDO == 1,
    "KEXTMARK_UNDO must equal kExtmarkUndo (1) per extmark_defs.h"
);

/// Meta type for inline virtual text.
pub const KMT_META_INLINE: c_int = 0;

/// Meta type for virtual lines.
pub const KMT_META_LINES: c_int = 1;

// Re-export the flags module contents
pub use flags::*;
