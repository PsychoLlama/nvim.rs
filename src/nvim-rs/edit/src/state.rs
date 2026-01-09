//! Edit state management for insert mode.
//!
//! This module provides comprehensive state management for insert mode,
//! including tracking of cursor position, insertion points, and mode-specific
//! flags.

use std::ffi::c_int;

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
