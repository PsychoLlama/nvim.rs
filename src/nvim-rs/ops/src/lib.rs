//! Operator functions for Neovim
//!
//! This crate provides Rust implementations of operator-related functions
//! from `src/nvim/ops.c`.
//!
//! ## Module Structure
//!
//! - [`types`]: Core type definitions (OpType, MotionType, BlockDef, Pos)
//! - [`oparg`]: Wrapper for operator arguments (oparg_T)
//! - [`tilde`]: Case swapping operations (g~, gU, gu, g?)
//! - [`shift`]: Indent shifting operations (< and >)
//! - [`addsub`]: Number/character increment/decrement (Ctrl-A, Ctrl-X)
//! - [`replace`]: Character replacement calculations (r)
//! - [`join`]: Line join calculations (J, gJ)
//! - [`delete`]: Delete operation calculations (d, x, D)
//! - [`yank`]: Yank operation calculations (y)
//! - [`insert`]: Insert/change operation calculations (I, A, c)
//! - [`put`]: Put/paste operation calculations (p, P)
//! - [`format`]: Text formatting operations (gq, gw, =)

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

pub mod addsub;
pub mod addsub_full;
pub mod cursor_pos_info;
pub mod delete;
pub mod delete_full;
pub mod format;
pub mod insert;
pub mod join;
pub mod oparg;
pub mod pending;
pub mod put;
pub mod replace;
pub mod replace_full;
pub mod shift;
pub mod tilde;
pub mod types;
pub mod yank;

pub use oparg::{OpArgHandle, OpArgMut, OpArgRef};
pub use types::{BlockDef, MotionType, OpType, Pos};

use std::ffi::{c_int, c_void};

/// Flags for operator properties
const OPF_LINES: u8 = 1; // operator always works on lines
const OPF_CHANGE: u8 = 2; // operator changes text

/// NUL character
const NUL: u8 = 0;

/// Ctrl+A character
const CTRL_A: u8 = 1;

/// Ctrl+X character
const CTRL_X: u8 = 24;

/// Operator type constants (must match ops.h)
const OP_NOP: c_int = 0;
const OP_YANK: c_int = 2;
const OP_TILDE: c_int = 7;
const OP_REPLACE: c_int = 16;
const OP_NR_ADD: c_int = 28;
const OP_NR_SUB: c_int = 29;

/// Operator character table.
/// Each entry is [char1, char2, flags].
/// Index must correspond with OP_* defines in ops.h!
static OPCHARS: [[u8; 3]; 30] = [
    [NUL, NUL, 0],                        // OP_NOP
    [b'd', NUL, OPF_CHANGE],              // OP_DELETE
    [b'y', NUL, 0],                       // OP_YANK
    [b'c', NUL, OPF_CHANGE],              // OP_CHANGE
    [b'<', NUL, OPF_LINES | OPF_CHANGE],  // OP_LSHIFT
    [b'>', NUL, OPF_LINES | OPF_CHANGE],  // OP_RSHIFT
    [b'!', NUL, OPF_LINES | OPF_CHANGE],  // OP_FILTER
    [b'g', b'~', OPF_CHANGE],             // OP_TILDE
    [b'=', NUL, OPF_LINES | OPF_CHANGE],  // OP_INDENT
    [b'g', b'q', OPF_LINES | OPF_CHANGE], // OP_FORMAT
    [b':', NUL, OPF_LINES],               // OP_COLON
    [b'g', b'U', OPF_CHANGE],             // OP_UPPER
    [b'g', b'u', OPF_CHANGE],             // OP_LOWER
    [b'J', NUL, OPF_LINES | OPF_CHANGE],  // OP_JOIN
    [b'g', b'J', OPF_LINES | OPF_CHANGE], // OP_JOIN_NS
    [b'g', b'?', OPF_CHANGE],             // OP_ROT13
    [b'r', NUL, OPF_CHANGE],              // OP_REPLACE
    [b'I', NUL, OPF_CHANGE],              // OP_INSERT
    [b'A', NUL, OPF_CHANGE],              // OP_APPEND
    [b'z', b'f', 0],                      // OP_FOLD
    [b'z', b'o', OPF_LINES],              // OP_FOLDOPEN
    [b'z', b'O', OPF_LINES],              // OP_FOLDOPENREC
    [b'z', b'c', OPF_LINES],              // OP_FOLDCLOSE
    [b'z', b'C', OPF_LINES],              // OP_FOLDCLOSEREC
    [b'z', b'd', OPF_LINES],              // OP_FOLDDEL
    [b'z', b'D', OPF_LINES],              // OP_FOLDDELREC
    [b'g', b'w', OPF_LINES | OPF_CHANGE], // OP_FORMAT2
    [b'g', b'@', OPF_CHANGE],             // OP_FUNCTION
    [CTRL_A, NUL, OPF_CHANGE],            // OP_NR_ADD
    [CTRL_X, NUL, OPF_CHANGE],            // OP_NR_SUB
];

/// Check if operator always works on whole lines.
///
/// Returns true if operator "op" always works on whole lines.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn op_on_lines_impl(op: c_int) -> bool {
    if op < 0 || op as usize >= OPCHARS.len() {
        return false;
    }
    (OPCHARS[op as usize][2] & OPF_LINES) != 0
}

/// FFI export for `op_on_lines`.
#[must_use]
#[unsafe(export_name = "op_on_lines")]
pub extern "C" fn rs_op_on_lines(op: c_int) -> c_int {
    c_int::from(op_on_lines_impl(op))
}

/// Check if operator changes text.
///
/// Returns true if operator "op" changes text.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn op_is_change_impl(op: c_int) -> bool {
    if op < 0 || op as usize >= OPCHARS.len() {
        return false;
    }
    (OPCHARS[op as usize][2] & OPF_CHANGE) != 0
}

/// FFI export for `op_is_change`.
#[must_use]
#[unsafe(export_name = "op_is_change")]
pub extern "C" fn rs_op_is_change(op: c_int) -> c_int {
    c_int::from(op_is_change_impl(op))
}

/// Get first operator command character.
///
/// Returns 'g' or 'z' if there is another command character.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn get_op_char_impl(optype: c_int) -> c_int {
    if optype < 0 || optype as usize >= OPCHARS.len() {
        return 0;
    }
    c_int::from(OPCHARS[optype as usize][0])
}

/// FFI export for `get_op_char`.
#[must_use]
#[unsafe(export_name = "get_op_char")]
pub extern "C" fn rs_get_op_char(optype: c_int) -> c_int {
    get_op_char_impl(optype)
}

/// Get second operator command character.
#[inline]
#[allow(clippy::cast_sign_loss)] // We check for negative values before casting
fn get_extra_op_char_impl(optype: c_int) -> c_int {
    if optype < 0 || optype as usize >= OPCHARS.len() {
        return 0;
    }
    c_int::from(OPCHARS[optype as usize][1])
}

/// FFI export for `get_extra_op_char`.
#[must_use]
#[unsafe(export_name = "get_extra_op_char")]
pub extern "C" fn rs_get_extra_op_char(optype: c_int) -> c_int {
    get_extra_op_char_impl(optype)
}

/// Translate a command name into an operator type.
///
/// Must only be called with a valid operator name!
/// Returns the operator ID matching the given char1/char2 pair.
/// Special cases are handled for 'r', '~', 'g'+Ctrl-A, 'g'+Ctrl-X, 'z'+'y'.
///
/// Returns `OP_NOP` (0) if no match is found (instead of calling `internal_error`).
#[inline]
#[allow(clippy::cast_possible_truncation)] // CTRL_A/CTRL_X are small values
fn get_op_type_impl(char1: c_int, char2: c_int) -> c_int {
    // Special case: 'r' ignores second character
    if char1 == c_int::from(b'r') {
        return OP_REPLACE;
    }
    // Special case: '~' when tilde is an operator
    if char1 == c_int::from(b'~') {
        return OP_TILDE;
    }
    // Special case: 'g' + Ctrl-A = add
    if char1 == c_int::from(b'g') && char2 == c_int::from(CTRL_A) {
        return OP_NR_ADD;
    }
    // Special case: 'g' + Ctrl-X = subtract
    if char1 == c_int::from(b'g') && char2 == c_int::from(CTRL_X) {
        return OP_NR_SUB;
    }
    // Special case: 'z' + 'y' = yank
    if char1 == c_int::from(b'z') && char2 == c_int::from(b'y') {
        return OP_YANK;
    }

    // Search in opchars table
    for (i, entry) in OPCHARS.iter().enumerate() {
        if c_int::from(entry[0]) == char1 && c_int::from(entry[1]) == char2 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            return i as c_int;
        }
    }

    // No match found - return OP_NOP instead of calling internal_error
    OP_NOP
}

/// FFI export for `get_op_type`.
#[must_use]
#[unsafe(export_name = "get_op_type")]
pub extern "C" fn rs_get_op_type(char1: c_int, char2: c_int) -> c_int {
    get_op_type_impl(char1, char2)
}

// =============================================================================
// Operator Pending State
// =============================================================================

// C accessor functions for operator state
extern "C" {
    /// Check if current_oap is NULL.
    fn nvim_oap_is_null() -> c_int;

    /// Get the finish_op global flag.
    fn nvim_get_finish_op() -> c_int;

    /// Get current_oap->prev_opcount (returns 0 if current_oap is NULL).
    fn nvim_oap_get_prev_opcount() -> c_int;

    /// Get current_oap->prev_count0 (returns 0 if current_oap is NULL).
    fn nvim_oap_get_prev_count0() -> c_int;

    /// Get current_oap->op_type (returns OP_NOP if current_oap is NULL).
    fn nvim_oap_get_op_type() -> c_int;

    /// Get current_oap->regname (returns NUL if current_oap is NULL).
    fn nvim_oap_get_regname() -> c_int;
}

/// Check if an operator was started but not finished yet.
///
/// Includes typing a count or a register name.
/// Returns true when an operator is pending, false otherwise.
#[inline]
fn op_pending_impl() -> bool {
    // SAFETY: These are safe accessors to C globals
    unsafe {
        let oap_null = nvim_oap_is_null() != 0;
        let finish_op = nvim_get_finish_op() != 0;
        let prev_opcount = nvim_oap_get_prev_opcount();
        let prev_count0 = nvim_oap_get_prev_count0();
        let op_type = nvim_oap_get_op_type();
        let regname = nvim_oap_get_regname();

        // Logic: !(current_oap != NULL && !finish_op && prev_opcount == 0
        //          && prev_count0 == 0 && op_type == OP_NOP && regname == NUL)
        !(!oap_null
            && !finish_op
            && prev_opcount == 0
            && prev_count0 == 0
            && op_type == OP_NOP
            && regname == 0)
    }
}

/// FFI wrapper for `op_pending`.
///
/// Returns true if an operator was started but not finished yet.
#[no_mangle]
pub extern "C" fn rs_op_pending() -> bool {
    op_pending_impl()
}

// =============================================================================
// Phase O7 API & Undo Integration Helpers
// =============================================================================

/// Check if undo should be saved before this operation.
#[must_use]
#[inline]
pub fn should_save_undo(op_type: c_int, empty: bool) -> bool {
    // Save undo for change operations that aren't empty
    op_is_change_impl(op_type) && !empty
}

/// Get undo message based on operator type.
#[must_use]
#[inline]
pub const fn get_undo_message_id(op_type: c_int) -> c_int {
    match op_type {
        1 => 1,           // OP_DELETE -> delete message
        3 => 2,           // OP_CHANGE -> change message
        2 => 3,           // OP_YANK -> yank message (no undo, but for yankreg)
        4 | 5 => 4,       // OP_LSHIFT/OP_RSHIFT -> shift message
        7 | 11 | 12 => 5, // OP_TILDE/OP_UPPER/OP_LOWER -> case change
        9 | 26 => 6,      // OP_FORMAT/OP_FORMAT2 -> format message
        13 | 14 => 7,     // OP_JOIN/OP_JOIN_NS -> join message
        16 => 8,          // OP_REPLACE -> replace message
        _ => 0,
    }
}

/// Check if API notification should be sent for this operation.
#[must_use]
#[inline]
pub fn should_notify_api(op_type: c_int) -> bool {
    // Notify for operations that change text
    op_is_change_impl(op_type)
}

/// Calculate the number of lines changed by operation.
#[must_use]
#[inline]
pub const fn calc_lines_changed(start_lnum: c_int, end_lnum: c_int, lines_added: c_int) -> c_int {
    let original_lines = if end_lnum >= start_lnum {
        end_lnum - start_lnum + 1
    } else {
        0
    };
    original_lines + lines_added
}

/// Check if marks need to be adjusted after this operation.
#[must_use]
#[inline]
pub fn needs_mark_adjust(op_type: c_int, line_count_changed: bool) -> bool {
    // Adjust marks for operations that delete or add lines
    op_is_change_impl(op_type) && line_count_changed
}

/// Calculate mark adjustment amount.
#[must_use]
#[inline]
pub const fn calc_mark_adjust(old_lines: c_int, new_lines: c_int) -> c_int {
    new_lines - old_lines
}

/// Check if extmarks need splice notification.
#[must_use]
#[inline]
pub fn needs_extmark_splice(op_type: c_int) -> bool {
    op_is_change_impl(op_type)
}

/// Check if cursor position needs adjustment after operation.
#[must_use]
#[inline]
pub fn needs_cursor_adjust(op_type: c_int, is_visual: bool) -> bool {
    op_is_change_impl(op_type) || is_visual
}

/// Calculate cursor line after delete.
#[must_use]
#[inline]
pub const fn calc_cursor_lnum_after_delete(
    delete_start: c_int,
    delete_end: c_int,
    cursor_lnum: c_int,
    total_lines: c_int,
) -> c_int {
    if cursor_lnum < delete_start {
        cursor_lnum
    } else if cursor_lnum > delete_end {
        cursor_lnum - (delete_end - delete_start + 1)
    } else {
        // Cursor was within deleted range
        let new_lnum = delete_start;
        if new_lnum > total_lines {
            if total_lines > 0 {
                total_lines
            } else {
                1
            }
        } else {
            new_lnum
        }
    }
}

/// Check if undo buffer is full.
#[must_use]
#[inline]
pub const fn is_undo_buffer_full(current_size: c_int, max_size: c_int) -> bool {
    max_size > 0 && current_size >= max_size
}

/// Check if undolevels allows this undo.
#[must_use]
#[inline]
pub const fn undolevels_allows(undolevels: c_int) -> bool {
    undolevels >= 0
}

/// Check if operation should trigger autocommand.
#[must_use]
#[inline]
pub fn should_trigger_autocmd(op_type: c_int, silent: bool) -> bool {
    op_is_change_impl(op_type) && !silent
}

/// Get operation byte count for undo.
#[must_use]
#[inline]
pub const fn calc_op_byte_count(line_count: c_int, avg_line_len: c_int) -> c_int {
    line_count * avg_line_len
}

// =============================================================================
// Phase O7 FFI Wrappers
// =============================================================================

/// FFI: Check if should save undo.
#[no_mangle]
pub extern "C" fn rs_should_save_undo(op_type: c_int, empty: c_int) -> c_int {
    c_int::from(should_save_undo(op_type, empty != 0))
}

/// FFI: Get undo message ID.
#[no_mangle]
pub extern "C" fn rs_get_undo_message_id(op_type: c_int) -> c_int {
    get_undo_message_id(op_type)
}

/// FFI: Check if should notify API.
#[no_mangle]
pub extern "C" fn rs_should_notify_api(op_type: c_int) -> c_int {
    c_int::from(should_notify_api(op_type))
}

/// FFI: Calculate lines changed.
#[no_mangle]
pub extern "C" fn rs_calc_lines_changed(
    start_lnum: c_int,
    end_lnum: c_int,
    lines_added: c_int,
) -> c_int {
    calc_lines_changed(start_lnum, end_lnum, lines_added)
}

/// FFI: Check if needs mark adjust.
#[no_mangle]
pub extern "C" fn rs_needs_mark_adjust(op_type: c_int, line_count_changed: c_int) -> c_int {
    c_int::from(needs_mark_adjust(op_type, line_count_changed != 0))
}

/// FFI: Calculate mark adjustment.
#[no_mangle]
pub extern "C" fn rs_calc_mark_adjust(old_lines: c_int, new_lines: c_int) -> c_int {
    calc_mark_adjust(old_lines, new_lines)
}

/// FFI: Check if needs extmark splice.
#[no_mangle]
pub extern "C" fn rs_needs_extmark_splice(op_type: c_int) -> c_int {
    c_int::from(needs_extmark_splice(op_type))
}

/// FFI: Check if needs cursor adjust.
#[no_mangle]
pub extern "C" fn rs_needs_cursor_adjust(op_type: c_int, is_visual: c_int) -> c_int {
    c_int::from(needs_cursor_adjust(op_type, is_visual != 0))
}

/// FFI: Calculate cursor line after delete.
#[no_mangle]
pub extern "C" fn rs_calc_cursor_lnum_after_delete(
    delete_start: c_int,
    delete_end: c_int,
    cursor_lnum: c_int,
    total_lines: c_int,
) -> c_int {
    calc_cursor_lnum_after_delete(delete_start, delete_end, cursor_lnum, total_lines)
}

/// FFI: Check if undo buffer is full.
#[no_mangle]
pub extern "C" fn rs_is_undo_buffer_full(current_size: c_int, max_size: c_int) -> c_int {
    c_int::from(is_undo_buffer_full(current_size, max_size))
}

/// FFI: Check if undolevels allows.
#[no_mangle]
pub extern "C" fn rs_undolevels_allows(undolevels: c_int) -> c_int {
    c_int::from(undolevels_allows(undolevels))
}

/// FFI: Check if should trigger autocmd.
#[no_mangle]
pub extern "C" fn rs_should_trigger_autocmd(op_type: c_int, silent: c_int) -> c_int {
    c_int::from(should_trigger_autocmd(op_type, silent != 0))
}

/// FFI: Calculate op byte count.
#[no_mangle]
pub extern "C" fn rs_calc_op_byte_count(line_count: c_int, avg_line_len: c_int) -> c_int {
    calc_op_byte_count(line_count, avg_line_len)
}

// =============================================================================
// Phase 6: adjust_cursor_eol
// =============================================================================

const K_OPT_VE_FLAG_ONEMORE: c_int = 0x08;
const K_OPT_VE_FLAG_ALL: c_int = 0x04;
const MODE_INSERT: c_int = 0x10;

extern "C" {
    // Cursor position accessors
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_gchar_cursor() -> c_int;
    fn nvim_get_ve_flags() -> c_int;
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_get_State() -> c_int;
    fn nvim_dec_cursor() -> c_int;
    // getvcol for cursor (returns scol and ecol via out-params)
    fn nvim_getvcol_cursor(scol: *mut c_int, ecol: *mut c_int);
    fn nvim_set_cursor_coladd(val: c_int);
}

/// Adjust the cursor column if we're at EOL and not in 'virtualedit' onemore.
///
/// Ported from `adjust_cursor_eol()` in ops.c.
///
/// # Safety
/// Accesses global cursor state via C accessors.
#[unsafe(export_name = "adjust_cursor_eol")]
pub unsafe extern "C" fn rs_adjust_cursor_eol() {
    let cur_ve_flags = nvim_get_ve_flags();

    let col = nvim_get_cursor_col();
    let ch = nvim_gchar_cursor();
    let onemore = (cur_ve_flags & K_OPT_VE_FLAG_ONEMORE) == 0;
    let restart = nvim_get_restart_edit() != 0;
    let state = nvim_get_State();
    let in_insert = (state & MODE_INSERT) != 0;

    let adj_cursor = col > 0 && ch == 0 && onemore && !restart && !in_insert;
    if !adj_cursor {
        return;
    }

    // Put the cursor on the last character in the line.
    nvim_dec_cursor();

    if cur_ve_flags == K_OPT_VE_FLAG_ALL {
        let mut scol: c_int = 0;
        let mut ecol: c_int = 0;
        nvim_getvcol_cursor(&raw mut scol, &raw mut ecol);
        nvim_set_cursor_coladd(ecol - scol + 1);
    }
}

// =============================================================================
// Phase 3: get_region_bytecount
// =============================================================================

extern "C" {
    /// Get `buf->b_ml.ml_line_count` for an arbitrary buffer.
    fn nvim_buf_get_ml_line_count(buf: *mut c_void) -> c_int;
    /// Get line length for arbitrary buffer (already exported from memline crate).
    fn ml_get_buf_len(buf: *mut c_void, lnum: c_int) -> c_int;
}

/// Port of `get_region_bytecount` -- byte count of buffer region, end-exclusive.
///
/// # Safety
/// `buf` must be a valid `buf_T *`.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn get_region_bytecount(
    buf: *mut c_void,
    start_lnum: c_int,
    end_lnum: c_int,
    start_col: c_int,
    end_col: c_int,
) -> isize {
    let max_lnum = nvim_buf_get_ml_line_count(buf);
    if start_lnum > max_lnum {
        return 0;
    }
    if start_lnum == end_lnum {
        return (end_col - start_col) as isize;
    }
    let mut deleted_bytes: isize = (ml_get_buf_len(buf, start_lnum) - start_col + 1) as isize;
    let mut i: c_int = 1;
    while i < end_lnum - start_lnum {
        if start_lnum + i > max_lnum {
            return deleted_bytes;
        }
        deleted_bytes += (ml_get_buf_len(buf, start_lnum + i) + 1) as isize;
        i += 1;
    }
    if end_lnum > max_lnum {
        return deleted_bytes;
    }
    deleted_bytes + end_col as isize
}

// =============================================================================
// Phase 3: skip_comment
// =============================================================================

use std::ffi::c_char;

extern "C" {
    /// Get offset of last comment leader in line (wrapper around get_last_leader_offset).
    fn nvim_get_last_leader_offset(line: *const c_char, flags: *mut *mut c_char) -> c_int;
    /// Get leader length for line (wrapper around get_leader_len).
    fn nvim_get_leader_len(
        line: *const c_char,
        flags: *mut *mut c_char,
        backward: bool,
        include_space: bool,
    ) -> c_int;
}

/// COM_END: flag meaning "end of comment", matches C's `COM_END` in option_vars.h
const COM_END: c_int = b'e' as c_int;

/// Port of `skip_comment` -- skip comment leaders on a line.
///
/// Returns a pointer into `line` past the comment leader, or `line` unchanged.
/// Sets `*is_comment` to indicate whether the line starts an unclosed comment.
///
/// # Safety
/// - `line` must be a valid NUL-terminated C string
/// - `is_comment` must be a valid pointer to a bool
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn skip_comment(
    line: *mut c_char,
    process: bool,
    include_space: bool,
    is_comment: *mut bool,
) -> *mut c_char {
    let mut comment_flags: *mut c_char = std::ptr::null_mut();
    let leader_offset = nvim_get_last_leader_offset(line, &raw mut comment_flags);

    *is_comment = false;
    if leader_offset != -1 {
        // Check whether the line ends with an unclosed comment.
        // If the last comment leader has COM_END in flags, there's no comment.
        while !comment_flags.is_null() && *comment_flags != 0 {
            let ch = c_int::from(*comment_flags as u8);
            if ch == COM_END || ch == c_int::from(b':') {
                break;
            }
            comment_flags = comment_flags.add(1);
        }
        if !comment_flags.is_null() && c_int::from(*comment_flags as u8) != COM_END {
            *is_comment = true;
        }
    }

    if !process {
        return line;
    }

    comment_flags = std::ptr::null_mut();
    let lead_len = nvim_get_leader_len(line, &raw mut comment_flags, false, include_space);

    if lead_len == 0 {
        return line;
    }

    // Find COM_END or colon, whichever comes first.
    while !comment_flags.is_null() && *comment_flags != 0 {
        let ch = c_int::from(*comment_flags as u8);
        if ch == COM_END || ch == c_int::from(b':') {
            break;
        }
        comment_flags = comment_flags.add(1);
    }

    // If we found a colon (or NUL), advance line past the leader.
    // (COM_END means closing part of 3-part comment -- don't remove it.)
    if comment_flags.is_null()
        || *comment_flags == 0
        || c_int::from(*comment_flags as u8) == c_int::from(b':')
    {
        line.add(lead_len as usize)
    } else {
        line
    }
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    // Operator IDs from ops.h
    const OP_NOP: c_int = 0;
    const OP_DELETE: c_int = 1;
    const OP_YANK: c_int = 2;
    const OP_CHANGE: c_int = 3;
    const OP_LSHIFT: c_int = 4;
    const OP_RSHIFT: c_int = 5;
    const OP_FILTER: c_int = 6;
    const OP_TILDE: c_int = 7;
    const OP_INDENT: c_int = 8;
    const OP_FORMAT: c_int = 9;
    const OP_COLON: c_int = 10;
    const OP_UPPER: c_int = 11;
    const OP_LOWER: c_int = 12;
    const OP_JOIN: c_int = 13;
    const OP_FOLD: c_int = 19;
    const OP_FOLDOPEN: c_int = 20;
    const OP_REPLACE: c_int = 16;
    const OP_NR_ADD: c_int = 28;
    const OP_NR_SUB: c_int = 29;

    #[test]
    fn test_op_on_lines() {
        // Operators that work on lines
        assert!(op_on_lines_impl(OP_LSHIFT));
        assert!(op_on_lines_impl(OP_RSHIFT));
        assert!(op_on_lines_impl(OP_FILTER));
        assert!(op_on_lines_impl(OP_INDENT));
        assert!(op_on_lines_impl(OP_FORMAT));
        assert!(op_on_lines_impl(OP_COLON));
        assert!(op_on_lines_impl(OP_JOIN));
        assert!(op_on_lines_impl(OP_FOLDOPEN));

        // Operators that don't work on lines
        assert!(!op_on_lines_impl(OP_NOP));
        assert!(!op_on_lines_impl(OP_DELETE));
        assert!(!op_on_lines_impl(OP_YANK));
        assert!(!op_on_lines_impl(OP_CHANGE));
        assert!(!op_on_lines_impl(OP_TILDE));
        assert!(!op_on_lines_impl(OP_UPPER));
        assert!(!op_on_lines_impl(OP_LOWER));
        assert!(!op_on_lines_impl(OP_FOLD));

        // Edge cases
        assert!(!op_on_lines_impl(-1));
        assert!(!op_on_lines_impl(100));
    }

    #[test]
    fn test_op_is_change() {
        // Operators that change text
        assert!(op_is_change_impl(OP_DELETE));
        assert!(op_is_change_impl(OP_CHANGE));
        assert!(op_is_change_impl(OP_LSHIFT));
        assert!(op_is_change_impl(OP_RSHIFT));
        assert!(op_is_change_impl(OP_FILTER));
        assert!(op_is_change_impl(OP_TILDE));
        assert!(op_is_change_impl(OP_INDENT));
        assert!(op_is_change_impl(OP_FORMAT));
        assert!(op_is_change_impl(OP_UPPER));
        assert!(op_is_change_impl(OP_LOWER));
        assert!(op_is_change_impl(OP_NR_ADD));
        assert!(op_is_change_impl(OP_NR_SUB));

        // Operators that don't change text
        assert!(!op_is_change_impl(OP_NOP));
        assert!(!op_is_change_impl(OP_YANK));
        assert!(!op_is_change_impl(OP_COLON));
        assert!(!op_is_change_impl(OP_FOLD));
        assert!(!op_is_change_impl(OP_FOLDOPEN));

        // Edge cases
        assert!(!op_is_change_impl(-1));
        assert!(!op_is_change_impl(100));
    }

    #[test]
    fn test_get_op_char() {
        assert_eq!(get_op_char_impl(OP_NOP), 0);
        assert_eq!(get_op_char_impl(OP_DELETE), b'd' as c_int);
        assert_eq!(get_op_char_impl(OP_YANK), b'y' as c_int);
        assert_eq!(get_op_char_impl(OP_CHANGE), b'c' as c_int);
        assert_eq!(get_op_char_impl(OP_LSHIFT), b'<' as c_int);
        assert_eq!(get_op_char_impl(OP_RSHIFT), b'>' as c_int);
        assert_eq!(get_op_char_impl(OP_FILTER), b'!' as c_int);
        assert_eq!(get_op_char_impl(OP_TILDE), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_INDENT), b'=' as c_int);
        assert_eq!(get_op_char_impl(OP_FORMAT), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_COLON), b':' as c_int);
        assert_eq!(get_op_char_impl(OP_UPPER), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_LOWER), b'g' as c_int);
        assert_eq!(get_op_char_impl(OP_JOIN), b'J' as c_int);
        assert_eq!(get_op_char_impl(OP_NR_ADD), 1); // Ctrl+A
        assert_eq!(get_op_char_impl(OP_NR_SUB), 24); // Ctrl+X

        // Edge cases
        assert_eq!(get_op_char_impl(-1), 0);
        assert_eq!(get_op_char_impl(100), 0);
    }

    #[test]
    fn test_get_extra_op_char() {
        // Operators with no extra char (NUL)
        assert_eq!(get_extra_op_char_impl(OP_NOP), 0);
        assert_eq!(get_extra_op_char_impl(OP_DELETE), 0);
        assert_eq!(get_extra_op_char_impl(OP_YANK), 0);
        assert_eq!(get_extra_op_char_impl(OP_CHANGE), 0);
        assert_eq!(get_extra_op_char_impl(OP_JOIN), 0);

        // Operators with extra char
        assert_eq!(get_extra_op_char_impl(OP_TILDE), b'~' as c_int);
        assert_eq!(get_extra_op_char_impl(OP_FORMAT), b'q' as c_int);
        assert_eq!(get_extra_op_char_impl(OP_UPPER), b'U' as c_int);
        assert_eq!(get_extra_op_char_impl(OP_LOWER), b'u' as c_int);

        // Edge cases
        assert_eq!(get_extra_op_char_impl(-1), 0);
        assert_eq!(get_extra_op_char_impl(100), 0);
    }

    #[test]
    fn test_ffi_wrappers() {
        // Verify FFI wrappers return same values as impl functions
        assert_eq!(rs_op_on_lines(OP_LSHIFT), 1);
        assert_eq!(rs_op_on_lines(OP_NOP), 0);
        assert_eq!(rs_op_is_change(OP_DELETE), 1);
        assert_eq!(rs_op_is_change(OP_YANK), 0);
        assert_eq!(rs_get_op_char(OP_DELETE), b'd' as c_int);
        assert_eq!(rs_get_extra_op_char(OP_TILDE), b'~' as c_int);
    }

    #[test]
    fn test_get_op_type() {
        // Single-char operators
        assert_eq!(get_op_type_impl(b'd' as c_int, 0), OP_DELETE);
        assert_eq!(get_op_type_impl(b'y' as c_int, 0), OP_YANK);
        assert_eq!(get_op_type_impl(b'c' as c_int, 0), OP_CHANGE);
        assert_eq!(get_op_type_impl(b'<' as c_int, 0), OP_LSHIFT);
        assert_eq!(get_op_type_impl(b'>' as c_int, 0), OP_RSHIFT);
        assert_eq!(get_op_type_impl(b'!' as c_int, 0), OP_FILTER);
        assert_eq!(get_op_type_impl(b'=' as c_int, 0), OP_INDENT);
        assert_eq!(get_op_type_impl(b':' as c_int, 0), OP_COLON);
        assert_eq!(get_op_type_impl(b'J' as c_int, 0), OP_JOIN);

        // Two-char operators (g prefix)
        assert_eq!(get_op_type_impl(b'g' as c_int, b'~' as c_int), OP_TILDE);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'q' as c_int), OP_FORMAT);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'U' as c_int), OP_UPPER);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'u' as c_int), OP_LOWER);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'J' as c_int), 14); // OP_JOIN_NS
        assert_eq!(get_op_type_impl(b'g' as c_int, b'?' as c_int), 15); // OP_ROT13
        assert_eq!(get_op_type_impl(b'g' as c_int, b'w' as c_int), 26); // OP_FORMAT2
        assert_eq!(get_op_type_impl(b'g' as c_int, b'@' as c_int), 27); // OP_FUNCTION

        // Two-char operators (z prefix)
        assert_eq!(get_op_type_impl(b'z' as c_int, b'f' as c_int), 19); // OP_FOLD
        assert_eq!(get_op_type_impl(b'z' as c_int, b'o' as c_int), 20); // OP_FOLDOPEN
        assert_eq!(get_op_type_impl(b'z' as c_int, b'O' as c_int), 21); // OP_FOLDOPENREC
        assert_eq!(get_op_type_impl(b'z' as c_int, b'c' as c_int), 22); // OP_FOLDCLOSE
        assert_eq!(get_op_type_impl(b'z' as c_int, b'd' as c_int), 24); // OP_FOLDDEL
        assert_eq!(get_op_type_impl(b'z' as c_int, b'D' as c_int), 25); // OP_FOLDDELREC

        // Special cases
        assert_eq!(get_op_type_impl(b'r' as c_int, 0), OP_REPLACE);
        assert_eq!(get_op_type_impl(b'r' as c_int, b'x' as c_int), OP_REPLACE); // ignores second char
        assert_eq!(get_op_type_impl(b'~' as c_int, 0), OP_TILDE);
        assert_eq!(get_op_type_impl(b'g' as c_int, 1), OP_NR_ADD); // Ctrl+A
        assert_eq!(get_op_type_impl(b'g' as c_int, 24), OP_NR_SUB); // Ctrl+X
        assert_eq!(get_op_type_impl(b'z' as c_int, b'y' as c_int), OP_YANK);

        // Invalid - should return OP_NOP
        assert_eq!(get_op_type_impl(b'x' as c_int, 0), OP_NOP);
        assert_eq!(get_op_type_impl(b'g' as c_int, b'x' as c_int), OP_NOP);
    }

    #[test]
    fn test_ffi_get_op_type() {
        assert_eq!(rs_get_op_type(b'd' as c_int, 0), OP_DELETE);
        assert_eq!(rs_get_op_type(b'g' as c_int, b'~' as c_int), OP_TILDE);
        assert_eq!(rs_get_op_type(b'r' as c_int, 0), OP_REPLACE);
    }

    #[test]
    fn test_operator_flags_constants() {
        // Verify operator flag constants match C definitions
        assert_eq!(OPF_LINES, 1);
        assert_eq!(OPF_CHANGE, 2);
    }

    #[test]
    fn test_ctrl_char_constants() {
        // Verify control character constants
        assert_eq!(NUL, 0);
        assert_eq!(CTRL_A, 1);
        assert_eq!(CTRL_X, 24);
    }

    #[test]
    fn test_opchars_table_size() {
        // Verify OPCHARS table has expected size (30 operators)
        assert_eq!(OPCHARS.len(), 30);
    }

    #[test]
    fn test_op_pending_logic() {
        // Test the op_pending logic without FFI
        // When oap is null, result should be true (operator pending)
        // When all conditions are false, result should be false (no pending)
        // This tests the boolean logic of the function

        // Simulate: oap is NULL -> operator is pending
        let oap_null = true;
        let finish_op = false;
        let prev_opcount = 0;
        let prev_count0 = 0;
        let op_type = OP_NOP;
        let regname = 0; // NUL

        // op_pending logic: !(oap != NULL && !finish_op && prev_opcount == 0
        //                    && prev_count0 == 0 && op_type == OP_NOP && regname == NUL)
        // When oap is NULL: !(false && ...) = !(false) = true
        let result = !(!oap_null
            && !finish_op
            && prev_opcount == 0
            && prev_count0 == 0
            && op_type == OP_NOP
            && regname == 0);
        assert!(result); // oap_null means operator pending

        // Simulate: oap is not null, all conditions met -> no operator pending
        let oap_null = false;
        let result = !(!oap_null
            && !finish_op
            && prev_opcount == 0
            && prev_count0 == 0
            && op_type == OP_NOP
            && regname == 0);
        assert!(!result); // all conditions met means no pending

        // Simulate: oap not null, but finish_op is true -> operator pending
        let finish_op = true;
        let result = !(!oap_null
            && !finish_op
            && prev_opcount == 0
            && prev_count0 == 0
            && op_type == OP_NOP
            && regname == 0);
        assert!(result); // finish_op means operator pending
    }

    // =========================================================================
    // Phase O7 API & Undo Integration Tests
    // =========================================================================

    #[test]
    fn test_should_save_undo() {
        // Change operations that aren't empty
        assert!(should_save_undo(OP_DELETE, false));
        assert!(should_save_undo(OP_CHANGE, false));
        // Yank doesn't change
        assert!(!should_save_undo(OP_YANK, false));
        // Empty operations
        assert!(!should_save_undo(OP_DELETE, true));
    }

    #[test]
    fn test_get_undo_message_id() {
        assert_eq!(get_undo_message_id(OP_DELETE), 1);
        assert_eq!(get_undo_message_id(OP_CHANGE), 2);
        assert_eq!(get_undo_message_id(OP_YANK), 3);
        assert_eq!(get_undo_message_id(OP_LSHIFT), 4);
        assert_eq!(get_undo_message_id(OP_RSHIFT), 4);
        assert_eq!(get_undo_message_id(OP_TILDE), 5);
        assert_eq!(get_undo_message_id(OP_FORMAT), 6);
        assert_eq!(get_undo_message_id(OP_JOIN), 7);
        assert_eq!(get_undo_message_id(OP_REPLACE), 8);
        assert_eq!(get_undo_message_id(OP_NOP), 0);
    }

    #[test]
    fn test_should_notify_api() {
        // Change operations should notify
        assert!(should_notify_api(OP_DELETE));
        assert!(should_notify_api(OP_CHANGE));
        // Non-change operations
        assert!(!should_notify_api(OP_YANK));
    }

    #[test]
    fn test_calc_lines_changed() {
        assert_eq!(calc_lines_changed(1, 10, 0), 10);
        assert_eq!(calc_lines_changed(1, 10, -5), 5);
        assert_eq!(calc_lines_changed(1, 10, 5), 15);
    }

    #[test]
    fn test_needs_mark_adjust() {
        // Change operation with line count changed
        assert!(needs_mark_adjust(OP_DELETE, true));
        // Change operation without line count change
        assert!(!needs_mark_adjust(OP_DELETE, false));
        // Non-change operation
        assert!(!needs_mark_adjust(OP_YANK, true));
    }

    #[test]
    fn test_calc_mark_adjust() {
        assert_eq!(calc_mark_adjust(10, 15), 5);
        assert_eq!(calc_mark_adjust(10, 5), -5);
        assert_eq!(calc_mark_adjust(10, 10), 0);
    }

    #[test]
    fn test_needs_extmark_splice() {
        assert!(needs_extmark_splice(OP_DELETE));
        assert!(needs_extmark_splice(OP_CHANGE));
        assert!(!needs_extmark_splice(OP_YANK));
    }

    #[test]
    fn test_needs_cursor_adjust() {
        // Change operations
        assert!(needs_cursor_adjust(OP_DELETE, false));
        // Non-change with visual
        assert!(needs_cursor_adjust(OP_YANK, true));
        // Non-change without visual
        assert!(!needs_cursor_adjust(OP_YANK, false));
    }

    #[test]
    fn test_calc_cursor_lnum_after_delete() {
        // Cursor before deleted range
        assert_eq!(calc_cursor_lnum_after_delete(5, 10, 3, 100), 3);
        // Cursor after deleted range
        assert_eq!(calc_cursor_lnum_after_delete(5, 10, 15, 100), 9);
        // Cursor within deleted range
        assert_eq!(calc_cursor_lnum_after_delete(5, 10, 7, 100), 5);
        // Cursor at new line past total
        assert_eq!(calc_cursor_lnum_after_delete(5, 10, 7, 5), 5);
    }

    #[test]
    fn test_is_undo_buffer_full() {
        assert!(is_undo_buffer_full(100, 100));
        assert!(is_undo_buffer_full(101, 100));
        assert!(!is_undo_buffer_full(50, 100));
        assert!(!is_undo_buffer_full(100, 0)); // max_size=0 means unlimited
    }

    #[test]
    fn test_undolevels_allows() {
        assert!(undolevels_allows(0));
        assert!(undolevels_allows(100));
        assert!(!undolevels_allows(-1));
    }

    #[test]
    fn test_should_trigger_autocmd() {
        // Change operation, not silent
        assert!(should_trigger_autocmd(OP_DELETE, false));
        // Change operation, silent
        assert!(!should_trigger_autocmd(OP_DELETE, true));
        // Non-change operation
        assert!(!should_trigger_autocmd(OP_YANK, false));
    }

    #[test]
    fn test_calc_op_byte_count() {
        assert_eq!(calc_op_byte_count(10, 80), 800);
        assert_eq!(calc_op_byte_count(5, 100), 500);
    }

    #[test]
    fn test_phase_o7_ffi_wrappers() {
        assert_eq!(rs_should_save_undo(OP_DELETE, 0), 1);
        assert_eq!(rs_should_save_undo(OP_DELETE, 1), 0);
        assert_eq!(rs_get_undo_message_id(OP_DELETE), 1);
        assert_eq!(rs_should_notify_api(OP_DELETE), 1);
        assert_eq!(rs_calc_lines_changed(1, 10, 0), 10);
        assert_eq!(rs_needs_mark_adjust(OP_DELETE, 1), 1);
        assert_eq!(rs_calc_mark_adjust(10, 15), 5);
        assert_eq!(rs_needs_extmark_splice(OP_DELETE), 1);
        assert_eq!(rs_needs_cursor_adjust(OP_DELETE, 0), 1);
        assert_eq!(rs_calc_cursor_lnum_after_delete(5, 10, 15, 100), 9);
        assert_eq!(rs_is_undo_buffer_full(100, 100), 1);
        assert_eq!(rs_undolevels_allows(0), 1);
        assert_eq!(rs_should_trigger_autocmd(OP_DELETE, 0), 1);
        assert_eq!(rs_calc_op_byte_count(10, 80), 800);
    }
}
