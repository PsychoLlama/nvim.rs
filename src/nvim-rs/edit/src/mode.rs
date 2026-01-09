//! Insert mode entry and exit handling.
//!
//! This module provides functions for managing insert mode transitions,
//! including mode entry, restart handling, and mode exit logic.

use std::ffi::c_int;

use crate::state::{
    can_cindent_set, did_restart_edit_get, did_restart_edit_set, ins_need_undo_set,
    insstart_blank_vcol_set, insstart_set, o_lnum_get, o_lnum_set, revins_chars_set,
    revins_legal_set, revins_on_get, revins_scol_set, update_insstart_orig_set, ColnrT, LinenrT,
    Position,
};

/// Neovim mode constants (from `vim_defs.h`).
mod consts {
    use std::ffi::c_int;

    pub const MODE_INSERT: c_int = 0x10;
    pub const MODE_REPLACE: c_int = 0x40;
    pub const MODE_VREPLACE: c_int = 0x80;
    pub const MODE_LANGMAP: c_int = 0x2000;

    /// MAXCOL value (maximum column number, used as sentinel).
    pub const MAXCOL: i32 = 0x7fff_ffff;
}

// C functions needed for mode transitions.
extern "C" {
    // Global state accessors (getters exist in window.c and cursor_shape.c)
    fn nvim_get_State() -> c_int;
    fn nvim_edit_set_State(val: c_int);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_edit_set_restart_edit(val: c_int);
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_set_arrow_used(val: c_int);
    fn nvim_get_p_ri() -> c_int;

    // Cursor position
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_curwin_get_cursor_col() -> ColnrT;
}

/// Command characters that start insert mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InsertCommand {
    /// Normal insert (i)
    Insert = b'i',
    /// Append (a)
    Append = b'a',
    /// Replace mode (R)
    Replace = b'R',
    /// Single CR insert (r<CR>)
    SingleCr = b'r',
    /// gI command (g)
    GotoInsert = b'g',
    /// Virtual replace (gR) (V)
    VirtualReplace = b'V',
    /// Single char virtual replace (gr) (v)
    SingleVirtualReplace = b'v',
}

impl InsertCommand {
    /// Create an `InsertCommand` from a command character.
    ///
    /// Returns `None` if the character is not a valid insert command.
    #[must_use]
    pub const fn from_char(c: u8) -> Option<Self> {
        match c {
            b'i' => Some(Self::Insert),
            b'a' => Some(Self::Append),
            b'R' => Some(Self::Replace),
            b'r' => Some(Self::SingleCr),
            b'g' => Some(Self::GotoInsert),
            b'V' => Some(Self::VirtualReplace),
            b'v' => Some(Self::SingleVirtualReplace),
            _ => None,
        }
    }

    /// Check if this command enters replace mode.
    #[inline]
    #[must_use]
    pub const fn is_replace(self) -> bool {
        matches!(self, Self::Replace)
    }

    /// Check if this command enters virtual replace mode.
    #[inline]
    #[must_use]
    pub const fn is_virtual_replace(self) -> bool {
        matches!(self, Self::VirtualReplace | Self::SingleVirtualReplace)
    }
}

/// Get the current editor state/mode.
#[inline]
#[must_use]
pub fn get_state() -> c_int {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_State() }
}

/// Set the current editor state/mode.
#[inline]
pub fn set_state(state: c_int) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_edit_set_State(state);
    }
}

/// Get the `restart_edit` flag.
#[inline]
#[must_use]
pub fn get_restart_edit() -> c_int {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_restart_edit() }
}

/// Set the `restart_edit` flag.
#[inline]
pub fn set_restart_edit(val: c_int) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_edit_set_restart_edit(val);
    }
}

/// Check if arrow keys have been used (cursor moved).
#[inline]
#[must_use]
pub fn get_arrow_used() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_arrow_used() != 0 }
}

/// Set the `arrow_used` flag.
#[inline]
pub fn set_arrow_used(val: bool) {
    // SAFETY: Simple global setter
    unsafe {
        nvim_set_arrow_used(c_int::from(val));
    }
}

/// Get the 'revins' option value.
#[inline]
#[must_use]
pub fn get_p_ri() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_p_ri() != 0 }
}

/// Initialize reverse insert mode state.
///
/// Sets up the state for reverse insert mode if enabled.
pub fn init_revins_state() {
    // There is no reverse replace mode
    let revins_on = get_state() == consts::MODE_INSERT && get_p_ri();

    if revins_on {
        // Reset reverse insert state
        revins_chars_set(0);
        revins_legal_set(0);
        revins_scol_set(-1);
    }
}

/// Initialize insert start position.
///
/// Sets up `Insstart` based on cursor position and start-of-line flag.
pub fn init_insstart(startln: bool) {
    // SAFETY: Accessing cursor position globals
    let lnum = unsafe { nvim_curwin_get_cursor_lnum() };
    let col = if startln {
        0
    } else {
        unsafe { nvim_curwin_get_cursor_col() }
    };

    insstart_set(Position::new(lnum, col));
}

/// Initialize insert mode state for entry.
///
/// This sets up all the state needed when entering insert mode,
/// including undo tracking, cindent state, and insert position.
pub fn init_insert_state(startln: bool, _cmdchar: c_int) {
    // Set Insstart_orig to Insstart at the start
    update_insstart_orig_set(true);

    // Initialize insert start position
    init_insstart(startln);

    // Initialize blank column tracking
    insstart_blank_vcol_set(consts::MAXCOL);

    // Need to save line for undo before first insert
    ins_need_undo_set(true);

    // Allow cindenting
    can_cindent_set(true);

    // Initialize reverse insert mode
    init_revins_state();
}

/// Handle restart edit setup.
///
/// Called when insert mode is being restarted after CTRL-O.
#[must_use]
pub fn handle_restart_edit() -> bool {
    let restart = get_restart_edit();
    if restart != 0 {
        // Remember the restart_edit value
        did_restart_edit_set(restart);
        return true;
    }
    false
}

/// Check if we're in insert mode.
#[inline]
#[must_use]
pub fn in_insert_mode() -> bool {
    (get_state() & consts::MODE_INSERT) != 0
}

/// Check if we're in replace mode.
#[inline]
#[must_use]
pub fn in_replace_mode() -> bool {
    (get_state() & consts::MODE_REPLACE) != 0
}

/// Check if we're in virtual replace mode.
#[inline]
#[must_use]
pub fn in_vreplace_mode() -> bool {
    (get_state() & consts::MODE_VREPLACE) != 0
}

/// Set the mode for entering insert/replace/vreplace.
///
/// Updates the State global based on the command character.
pub fn set_insert_mode(cmdchar: c_int) {
    let state = if cmdchar == c_int::from(b'R') {
        consts::MODE_REPLACE
    } else if cmdchar == c_int::from(b'V') || cmdchar == c_int::from(b'v') {
        consts::MODE_VREPLACE
    } else {
        consts::MODE_INSERT
    };
    set_state(state);
}

/// Update `o_lnum` for CTRL-O handling.
///
/// Called when exiting insert mode to track the line for cursor restoration.
pub fn update_o_lnum_on_exit() {
    // SAFETY: Accessing cursor position
    let cursor_lnum = unsafe { nvim_curwin_get_cursor_lnum() };
    o_lnum_set(cursor_lnum);
}

/// Check if langmap mode is active.
#[inline]
#[must_use]
pub fn has_langmap() -> bool {
    (get_state() & consts::MODE_LANGMAP) != 0
}

/// Enable langmap mode.
pub fn enable_langmap() {
    let state = get_state();
    set_state(state | consts::MODE_LANGMAP);
}

/// Disable langmap mode.
pub fn disable_langmap() {
    let state = get_state();
    set_state(state & !consts::MODE_LANGMAP);
}

// FFI exports

/// FFI: Initialize insert mode state.
#[no_mangle]
pub extern "C" fn rs_init_insert_state(startln: c_int, cmdchar: c_int) {
    init_insert_state(startln != 0, cmdchar);
}

/// FFI: Handle restart edit setup.
#[no_mangle]
pub extern "C" fn rs_handle_restart_edit() -> c_int {
    c_int::from(handle_restart_edit())
}

/// FFI: Check if in insert mode.
#[no_mangle]
pub extern "C" fn rs_in_insert_mode() -> c_int {
    c_int::from(in_insert_mode())
}

/// FFI: Check if in replace mode.
#[no_mangle]
pub extern "C" fn rs_in_replace_mode() -> c_int {
    c_int::from(in_replace_mode())
}

/// FFI: Check if in virtual replace mode.
#[no_mangle]
pub extern "C" fn rs_in_vreplace_mode() -> c_int {
    c_int::from(in_vreplace_mode())
}

/// FFI: Set insert mode based on command character.
#[no_mangle]
pub extern "C" fn rs_set_insert_mode(cmdchar: c_int) {
    set_insert_mode(cmdchar);
}

/// FFI: Update `o_lnum` on exit.
#[no_mangle]
pub extern "C" fn rs_update_o_lnum_on_exit() {
    update_o_lnum_on_exit();
}

/// FFI: Check if langmap mode is active.
#[no_mangle]
pub extern "C" fn rs_has_langmap() -> c_int {
    c_int::from(has_langmap())
}

/// FFI: Enable langmap mode.
#[no_mangle]
pub extern "C" fn rs_enable_langmap() {
    enable_langmap();
}

/// FFI: Disable langmap mode.
#[no_mangle]
pub extern "C" fn rs_disable_langmap() {
    disable_langmap();
}

/// FFI: Check if reverse insert mode is on.
#[no_mangle]
pub extern "C" fn rs_revins_on() -> c_int {
    c_int::from(revins_on_get())
}

/// FFI: Get the `did_restart_edit` value.
#[no_mangle]
pub extern "C" fn rs_did_restart_edit() -> c_int {
    did_restart_edit_get()
}

/// FFI: Get the `arrow_used` flag.
#[no_mangle]
pub extern "C" fn rs_get_arrow_used() -> c_int {
    c_int::from(get_arrow_used())
}

/// FFI: Set the `arrow_used` flag.
#[no_mangle]
pub extern "C" fn rs_set_arrow_used(val: c_int) {
    set_arrow_used(val != 0);
}

/// FFI: Get `o_lnum`.
#[no_mangle]
pub extern "C" fn rs_get_o_lnum() -> LinenrT {
    o_lnum_get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_command_from_char() {
        assert_eq!(InsertCommand::from_char(b'i'), Some(InsertCommand::Insert));
        assert_eq!(InsertCommand::from_char(b'a'), Some(InsertCommand::Append));
        assert_eq!(InsertCommand::from_char(b'R'), Some(InsertCommand::Replace));
        assert_eq!(
            InsertCommand::from_char(b'r'),
            Some(InsertCommand::SingleCr)
        );
        assert_eq!(
            InsertCommand::from_char(b'g'),
            Some(InsertCommand::GotoInsert)
        );
        assert_eq!(
            InsertCommand::from_char(b'V'),
            Some(InsertCommand::VirtualReplace)
        );
        assert_eq!(
            InsertCommand::from_char(b'v'),
            Some(InsertCommand::SingleVirtualReplace)
        );
        assert_eq!(InsertCommand::from_char(b'x'), None);
    }

    #[test]
    fn test_insert_command_is_replace() {
        assert!(InsertCommand::Replace.is_replace());
        assert!(!InsertCommand::Insert.is_replace());
        assert!(!InsertCommand::VirtualReplace.is_replace());
    }

    #[test]
    fn test_insert_command_is_virtual_replace() {
        assert!(InsertCommand::VirtualReplace.is_virtual_replace());
        assert!(InsertCommand::SingleVirtualReplace.is_virtual_replace());
        assert!(!InsertCommand::Replace.is_virtual_replace());
        assert!(!InsertCommand::Insert.is_virtual_replace());
    }
}
