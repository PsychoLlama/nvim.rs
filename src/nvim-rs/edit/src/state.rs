//! Edit state management for insert mode.
//!
//! This module provides comprehensive state management for insert mode,
//! including tracking of cursor position, insertion points, and mode-specific
//! flags.

use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim).
pub type LinenrT = i32;

/// Column number type (matches `colnr_T` in Neovim).
pub type ColnrT = i32;

// C accessor functions for edit state.
extern "C" {
    // Insert position tracking
    fn nvim_get_Insstart_lnum() -> LinenrT;
    fn nvim_get_Insstart_col() -> ColnrT;
    fn nvim_set_Insstart(lnum: LinenrT, col: ColnrT);
    fn nvim_get_Insstart_orig_lnum() -> LinenrT;
    fn nvim_get_Insstart_orig_col() -> ColnrT;
    fn nvim_set_Insstart_orig(lnum: LinenrT, col: ColnrT);
    fn nvim_get_Insstart_textlen() -> ColnrT;
    fn nvim_set_Insstart_textlen(val: ColnrT);
    fn nvim_get_Insstart_blank_vcol() -> ColnrT;
    fn nvim_set_Insstart_blank_vcol(val: ColnrT);

    // Undo tracking
    fn nvim_get_ins_need_undo() -> c_int;
    fn nvim_set_ins_need_undo(val: c_int);
    fn nvim_get_update_Insstart_orig() -> c_int;
    fn nvim_set_update_Insstart_orig(val: c_int);

    // Cindent state
    fn nvim_get_can_cindent() -> c_int;
    fn nvim_set_can_cindent(val: c_int);

    // Reverse insert mode
    fn nvim_get_revins_on() -> c_int;
    fn nvim_get_revins_chars() -> c_int;
    fn nvim_set_revins_chars(val: c_int);
    fn nvim_get_revins_legal() -> c_int;
    fn nvim_set_revins_legal(val: c_int);
    fn nvim_get_revins_scol() -> c_int;
    fn nvim_set_revins_scol(val: c_int);

    // Restart edit tracking
    fn nvim_get_did_restart_edit() -> c_int;
    fn nvim_set_did_restart_edit(val: c_int);

    // Completion state
    fn nvim_get_compl_busy() -> c_int;

    // Insert tracking
    fn nvim_get_last_insert_skip() -> c_int;
    fn nvim_get_new_insert_skip() -> c_int;
    fn nvim_set_new_insert_skip(val: c_int);

    // Undo sync state
    fn nvim_get_dont_sync_undo() -> c_int;
    fn nvim_set_dont_sync_undo(val: c_int);

    // Line tracking
    fn nvim_get_o_lnum() -> LinenrT;
    fn nvim_set_o_lnum(val: LinenrT);

    // Arrow state
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_set_arrow_used(val: c_int);
    fn nvim_get_State() -> c_int;

    // Redo buffer ops
    fn nvim_edit_AppendToRedobuff(s: *const c_char);
    fn nvim_edit_append_char_to_redobuff(c: c_int);
    fn nvim_edit_ResetRedobuff();

    // stop_insert (stays C, pass end_insert_pos as opaque pointer)
    fn nvim_edit_stop_insert(end_insert_pos: *mut c_void, esc: c_int, nomove: c_int);

    // check_spell_redraw (already in Rust)
    fn check_spell_redraw();

    // stop_arrow dependencies
    fn nvim_edit_set_insstart_from_cursor();
    fn nvim_edit_insstart_col_gt_orig() -> c_int;
    fn nvim_edit_linetabsize_cursor_line() -> ColnrT;
    fn nvim_edit_u_save_cursor() -> c_int;
    fn nvim_edit_set_ai_col(val: ColnrT);
    fn nvim_edit_set_orig_line_count(val: LinenrT);
    fn nvim_edit_set_vr_lines_changed(val: c_int);
    fn nvim_edit_curbuf_line_count() -> LinenrT;
    fn nvim_edit_foldOpenCursor();
}

/// Position in a buffer (line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Position {
    /// Line number (1-based).
    pub lnum: LinenrT,
    /// Column number (0-based byte offset).
    pub col: ColnrT,
}

impl Position {
    /// Create a new position.
    #[inline]
    #[must_use]
    pub const fn new(lnum: LinenrT, col: ColnrT) -> Self {
        Self { lnum, col }
    }
}

/// Tri-state for undo sync control (matches C enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TriState {
    /// False state.
    False = 0,
    /// True state.
    True = 1,
    /// None/unset state.
    None = 2,
}

impl From<c_int> for TriState {
    fn from(val: c_int) -> Self {
        match val {
            0 => Self::False,
            1 => Self::True,
            _ => Self::None,
        }
    }
}

impl From<TriState> for c_int {
    fn from(val: TriState) -> Self {
        val as Self
    }
}

/// Get the position where insert mode started.
///
/// Returns the cursor position when insert mode was entered.
#[inline]
#[must_use]
pub fn insstart_get() -> Position {
    // SAFETY: These are simple global accessors
    unsafe {
        Position {
            lnum: nvim_get_Insstart_lnum(),
            col: nvim_get_Insstart_col(),
        }
    }
}

/// Set the position where insert mode started.
#[inline]
pub fn insstart_set(pos: Position) {
    // SAFETY: These are simple global setters
    unsafe {
        nvim_set_Insstart(pos.lnum, pos.col);
    }
}

/// Get the original insert start position (before any adjustments).
#[inline]
#[must_use]
pub fn insstart_orig_get() -> Position {
    // SAFETY: These are simple global accessors
    unsafe {
        Position {
            lnum: nvim_get_Insstart_orig_lnum(),
            col: nvim_get_Insstart_orig_col(),
        }
    }
}

/// Set the original insert start position.
#[inline]
pub fn insstart_orig_set(pos: Position) {
    // SAFETY: These are simple global setters
    unsafe {
        nvim_set_Insstart_orig(pos.lnum, pos.col);
    }
}

/// Get the text length when insert started.
#[inline]
#[must_use]
pub fn insstart_textlen_get() -> ColnrT {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_Insstart_textlen() }
}

/// Set the text length when insert started.
#[inline]
pub fn insstart_textlen_set(val: ColnrT) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_Insstart_textlen(val);
    }
}

/// Get the virtual column for first inserted blank.
#[inline]
#[must_use]
pub fn insstart_blank_vcol_get() -> ColnrT {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_Insstart_blank_vcol() }
}

/// Set the virtual column for first inserted blank.
#[inline]
pub fn insstart_blank_vcol_set(val: ColnrT) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_Insstart_blank_vcol(val);
    }
}

/// Check if undo needs to be saved before the next insert.
#[inline]
#[must_use]
pub fn ins_need_undo_get() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_ins_need_undo() != 0 }
}

/// Set whether undo needs to be saved before the next insert.
#[inline]
pub fn ins_need_undo_set(val: bool) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_ins_need_undo(c_int::from(val));
    }
}

/// Check if `Insstart_orig` should be updated to match `Insstart`.
#[inline]
#[must_use]
pub fn update_insstart_orig_get() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_update_Insstart_orig() != 0 }
}

/// Set whether `Insstart_orig` should be updated to match `Insstart`.
#[inline]
pub fn update_insstart_orig_set(val: bool) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_update_Insstart_orig(c_int::from(val));
    }
}

/// Check if cindenting may be done on this line.
#[inline]
#[must_use]
pub fn can_cindent_get() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_can_cindent() != 0 }
}

/// Set whether cindenting may be done on this line.
#[inline]
pub fn can_cindent_set(val: bool) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_can_cindent(c_int::from(val));
    }
}

/// Check if reverse insert mode is on.
#[inline]
#[must_use]
pub fn revins_on_get() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_revins_on() != 0 }
}

/// Get the number of characters to skip after edit in reverse insert mode.
#[inline]
#[must_use]
pub fn revins_chars_get() -> i32 {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_revins_chars() }
}

/// Set the number of characters to skip after edit in reverse insert mode.
#[inline]
pub fn revins_chars_set(val: i32) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_revins_chars(val);
    }
}

/// Get whether the last character was legal in reverse insert mode.
#[inline]
#[must_use]
pub fn revins_legal_get() -> i32 {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_revins_legal() }
}

/// Set whether the last character was legal in reverse insert mode.
#[inline]
pub fn revins_legal_set(val: i32) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_revins_legal(val);
    }
}

/// Get the start column of reverse insert session.
#[inline]
#[must_use]
pub fn revins_scol_get() -> i32 {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_revins_scol() }
}

/// Set the start column of reverse insert session.
#[inline]
pub fn revins_scol_set(val: i32) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_revins_scol(val);
    }
}

/// Get the `restart_edit` value when `edit()` was called.
#[inline]
#[must_use]
pub fn did_restart_edit_get() -> i32 {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_did_restart_edit() }
}

/// Set the `restart_edit` value when `edit()` was called.
#[inline]
pub fn did_restart_edit_set(val: i32) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_did_restart_edit(val);
    }
}

/// Check if we're busy with completion (to prevent recursion).
#[inline]
#[must_use]
pub fn compl_busy_get() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_compl_busy() != 0 }
}

/// Get the number of chars in front of previous insert.
#[inline]
#[must_use]
pub fn last_insert_skip_get() -> i32 {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_last_insert_skip() }
}

/// Get the number of chars in front of current insert.
#[inline]
#[must_use]
pub fn new_insert_skip_get() -> i32 {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_new_insert_skip() }
}

/// Set the number of chars in front of current insert.
#[inline]
pub fn new_insert_skip_set(val: i32) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_new_insert_skip(val);
    }
}

/// Get the undo sync state.
///
/// This controls whether undo is synced for the next left/right cursor key.
#[inline]
#[must_use]
pub fn dont_sync_undo_get() -> TriState {
    // SAFETY: Simple global accessor
    unsafe { TriState::from(nvim_get_dont_sync_undo()) }
}

/// Set the undo sync state.
#[inline]
pub fn dont_sync_undo_set(val: TriState) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_dont_sync_undo(val.into());
    }
}

/// Get the saved line number for CTRL-O.
#[inline]
#[must_use]
pub fn o_lnum_get() -> LinenrT {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_o_lnum() }
}

/// Set the saved line number for CTRL-O.
#[inline]
pub fn o_lnum_set(val: LinenrT) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_o_lnum(val);
    }
}

// FFI exports for state queries

/// FFI: Check if undo is needed for insert mode.
#[no_mangle]
pub extern "C" fn rs_state_ins_need_undo() -> c_int {
    c_int::from(ins_need_undo_get())
}

/// FFI: Check if cindenting may be done.
#[no_mangle]
pub extern "C" fn rs_state_can_cindent() -> c_int {
    c_int::from(can_cindent_get())
}

/// FFI: Set whether cindenting may be done.
#[no_mangle]
pub extern "C" fn rs_state_set_can_cindent(val: c_int) {
    can_cindent_set(val != 0);
}

/// FFI: Check if reverse insert mode is on.
#[no_mangle]
pub extern "C" fn rs_state_revins_on() -> c_int {
    c_int::from(revins_on_get())
}

/// FFI: Get `did_restart_edit` value.
#[no_mangle]
pub extern "C" fn rs_state_did_restart_edit() -> c_int {
    did_restart_edit_get()
}

/// FFI: Check if completion is busy.
#[no_mangle]
pub extern "C" fn rs_state_compl_busy() -> c_int {
    c_int::from(compl_busy_get())
}

/// FFI: Get the insert start line number.
#[no_mangle]
pub extern "C" fn rs_state_insstart_lnum() -> LinenrT {
    insstart_get().lnum
}

/// FFI: Get the insert start column.
#[no_mangle]
pub extern "C" fn rs_state_insstart_col() -> ColnrT {
    insstart_get().col
}

/// FFI: Get the original insert start line number.
#[no_mangle]
pub extern "C" fn rs_state_insstart_orig_lnum() -> LinenrT {
    insstart_orig_get().lnum
}

/// FFI: Get the original insert start column.
#[no_mangle]
pub extern "C" fn rs_state_insstart_orig_col() -> ColnrT {
    insstart_orig_get().col
}

/// FFI: Get the text length when insert started.
#[no_mangle]
pub extern "C" fn rs_state_insstart_textlen() -> ColnrT {
    insstart_textlen_get()
}

/// FFI: Get the virtual column for first inserted blank.
#[no_mangle]
pub extern "C" fn rs_state_insstart_blank_vcol() -> ColnrT {
    insstart_blank_vcol_get()
}

/// FFI: Get the undo sync state.
#[no_mangle]
pub extern "C" fn rs_state_dont_sync_undo() -> c_int {
    dont_sync_undo_get().into()
}

/// FFI: Set the undo sync state.
#[no_mangle]
pub extern "C" fn rs_state_set_dont_sync_undo(val: c_int) {
    dont_sync_undo_set(TriState::from(val));
}

/// FFI: Get `o_lnum` (saved line for CTRL-O).
#[no_mangle]
pub extern "C" fn rs_state_o_lnum() -> LinenrT {
    o_lnum_get()
}

/// FFI: Set `o_lnum` (saved line for CTRL-O).
#[no_mangle]
pub extern "C" fn rs_state_set_o_lnum(val: LinenrT) {
    o_lnum_set(val);
}

// ============================================================================
// Constants (verified with _Static_assert in edit.c)
// ============================================================================

/// `OK` from `vim_defs.h`
const OK: c_int = 1;

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;

/// `VREPLACE_FLAG` from `state_defs.h`
const VREPLACE_FLAG: c_int = 0x200;

/// `Ctrl_G` from `ascii_defs.h`
const CTRL_G: c_int = 7;

/// ESC string (0x1b + NUL)
const ESC_STR: &[u8; 2] = b"\x1b\0";

// ============================================================================
// start_arrow_common — core logic for arrow key state transition
// ============================================================================

/// Core logic for arrow key state transitions.
/// If `end_change` is true and something was inserted, appends ESC to redo
/// buffer and calls `stop_insert`, then sets `arrow_used = true`.
/// Always calls `check_spell_redraw` at end.
unsafe fn start_arrow_common_impl(end_insert_pos: *mut c_void, end_change: c_int) {
    if nvim_get_arrow_used() == 0 && end_change != 0 {
        // something has been inserted
        nvim_edit_AppendToRedobuff(ESC_STR.as_ptr().cast());
        nvim_edit_stop_insert(end_insert_pos, 0, 0);
        nvim_set_arrow_used(1); // This means we stopped the current insert.
    }
    check_spell_redraw();
}

/// # Safety
/// Called from C FFI only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_start_arrow_common(end_insert_pos: *mut c_void, end_change: c_int) {
    start_arrow_common_impl(end_insert_pos, end_change);
}

// ============================================================================
// start_arrow — called when an arrow key is used in insert mode
// ============================================================================

/// Called when an arrow key is used in insert mode.
/// For undo/redo it resembles hitting the <ESC> key.
///
/// # Safety
/// Called from C FFI only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_start_arrow(end_insert_pos: *mut c_void) {
    start_arrow_common_impl(end_insert_pos, 1);
}

// ============================================================================
// start_arrow_with_change — like start_arrow() with end_change argument
// ============================================================================

/// Like `start_arrow()` but with `end_change` argument.
/// Will prepare for redo of CTRL-G U if `end_change` is false.
///
/// # Safety
/// Called from C FFI only.
#[unsafe(export_name = "start_arrow_with_change")]
pub unsafe extern "C" fn rs_start_arrow_with_change(
    end_insert_pos: *mut c_void,
    end_change: c_int,
) {
    start_arrow_common_impl(end_insert_pos, end_change);
    if end_change == 0 {
        nvim_edit_append_char_to_redobuff(CTRL_G);
        nvim_edit_append_char_to_redobuff(c_int::from(b'U'));
    }
}

// ============================================================================
// stop_arrow — reset state after arrow key movement
// ============================================================================

/// Called before a change is made in insert mode.
/// If an arrow key has been used, start a new insertion.
/// Returns FAIL if undo is impossible, shouldn't insert then.
unsafe fn stop_arrow_impl() -> c_int {
    if nvim_get_arrow_used() != 0 {
        nvim_edit_set_insstart_from_cursor(); // new insertion starts here
        if nvim_edit_insstart_col_gt_orig() != 0 && !ins_need_undo_get() {
            // Don't update the original insert position when moved to the
            // right, except when nothing was inserted yet.
            update_insstart_orig_set(false);
        }
        insstart_textlen_set(nvim_edit_linetabsize_cursor_line());

        if nvim_edit_u_save_cursor() == OK {
            nvim_set_arrow_used(0);
            ins_need_undo_set(false);
        }
        nvim_edit_set_ai_col(0);
        if nvim_get_State() & VREPLACE_FLAG != 0 {
            nvim_edit_set_orig_line_count(nvim_edit_curbuf_line_count());
            nvim_edit_set_vr_lines_changed(1);
        }
        nvim_edit_ResetRedobuff();
        nvim_edit_AppendToRedobuff(c"1i".as_ptr()); // Pretend we start an insertion.
        new_insert_skip_set(2);
    } else if ins_need_undo_get() && nvim_edit_u_save_cursor() == OK {
        ins_need_undo_set(false);
    }

    // Always open fold at the cursor line when inserting something.
    nvim_edit_foldOpenCursor();

    if nvim_get_arrow_used() != 0 || ins_need_undo_get() {
        FAIL
    } else {
        OK
    }
}

/// # Safety
/// Called from C FFI only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_stop_arrow() -> c_int {
    stop_arrow_impl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(1, 2);
        let pos2 = Position::new(1, 2);
        let pos3 = Position::new(1, 3);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_tristate_from_int() {
        assert_eq!(TriState::from(0), TriState::False);
        assert_eq!(TriState::from(1), TriState::True);
        assert_eq!(TriState::from(2), TriState::None);
        assert_eq!(TriState::from(99), TriState::None);
    }

    #[test]
    fn test_tristate_to_int() {
        assert_eq!(c_int::from(TriState::False), 0);
        assert_eq!(c_int::from(TriState::True), 1);
        assert_eq!(c_int::from(TriState::None), 2);
    }
}
