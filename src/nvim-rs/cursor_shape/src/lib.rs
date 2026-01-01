//! Cursor shape handling for Neovim
//!
//! Provides Rust implementations of cursor shape functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_else)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(unsafe_code)]

use std::ffi::{c_char, c_int};

/// Cursor shape types matching `CursorShape` enum in C
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    Block = 0,
    Hor = 1,
    Ver = 2,
}

/// Mode shape indices matching `ModeShape` enum in C
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeShape {
    N = 0,     // Normal mode
    V = 1,     // Visual mode
    I = 2,     // Insert mode
    R = 3,     // Replace mode
    C = 4,     // Command line Normal mode
    Ci = 5,    // Command line Insert mode
    Cr = 6,    // Command line Replace mode
    O = 7,     // Operator-pending mode
    Ve = 8,    // Visual mode with 'selection' exclusive
    Cline = 9, // On command line
    Status = 10,
    Sdrag = 11,
    Vsep = 12,
    Vdrag = 13,
    More = 14,
    Morel = 15,
    Sm = 16,   // showing matching paren
    Term = 17, // Terminal mode
    Count = 18,
}

// State mode flags from state_defs.h
const MODE_CMDLINE: c_int = 0x08;
const MODE_INSERT: c_int = 0x10;
const MODE_TERMINAL: c_int = 0x80;
const REPLACE_FLAG: c_int = 0x100;
const VREPLACE_FLAG: c_int = 0x200;
const MODE_SHOWMATCH: c_int = 0x6000 | MODE_INSERT;

extern "C" {
    /// Get the cursor shape for a mode index
    fn nvim_get_shape_table_shape(idx: c_int) -> c_int;
    /// Get the blinkon value for a mode index
    fn nvim_get_shape_table_blinkon(idx: c_int) -> c_int;
    /// Get the highlight id for a mode index
    fn nvim_get_shape_table_id(idx: c_int) -> c_int;
    /// Get the langmap highlight id for a mode index
    fn nvim_get_shape_table_id_lm(idx: c_int) -> c_int;
    /// Check if guicursor option is empty
    fn nvim_is_guicursor_empty() -> c_int;
    /// Get current editor State
    fn nvim_get_state() -> c_int;
    /// Check if operator is pending
    fn nvim_get_finish_op() -> c_int;
    /// Check if Visual mode is active
    fn nvim_get_visual_active() -> c_int;
    /// Get first char of 'selection' option
    fn nvim_get_p_sel_first() -> c_char;
    /// Check if at end of command line
    fn rs_cmdline_at_end() -> c_int;
    /// Check if in overstrike mode on command line
    fn rs_cmdline_overstrike() -> c_int;
    /// Get the full name for a mode index
    fn nvim_get_shape_table_name(idx: c_int) -> *const c_char;
}

/// Returns true if the cursor is non-blinking "block" shape during
/// visual selection.
///
/// # Safety
/// Calls C accessor functions for `shape_table`.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_is_block_during_visual(exclusive: c_int) -> c_int {
    let mode_idx = if exclusive != 0 {
        ModeShape::Ve as c_int
    } else {
        ModeShape::V as c_int
    };

    let shape = nvim_get_shape_table_shape(mode_idx);
    let blinkon = nvim_get_shape_table_blinkon(mode_idx);

    c_int::from(shape == CursorShape::Block as c_int && blinkon == 0)
}

/// Check if a syntax id is used as a cursor style.
///
/// # Safety
/// Calls C accessor functions for `shape_table` and `guicursor`.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_mode_uses_syn_id(syn_id: c_int) -> c_int {
    if nvim_is_guicursor_empty() != 0 {
        return 0;
    }

    for mode_idx in 0..ModeShape::Count as c_int {
        let id = nvim_get_shape_table_id(mode_idx);
        let id_lm = nvim_get_shape_table_id_lm(mode_idx);
        if id == syn_id || id_lm == syn_id {
            return 1;
        }
    }

    0
}

/// Return the index into shape_table[] for the current mode.
///
/// # Safety
/// Calls C accessor functions for global state.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_get_mode_idx() -> c_int {
    let state = nvim_get_state();

    if state == MODE_SHOWMATCH {
        return ModeShape::Sm as c_int;
    }
    if state == MODE_TERMINAL {
        return ModeShape::Term as c_int;
    }
    if (state & VREPLACE_FLAG) != 0 {
        return ModeShape::R as c_int;
    }
    if (state & REPLACE_FLAG) != 0 {
        return ModeShape::R as c_int;
    }
    if (state & MODE_INSERT) != 0 {
        return ModeShape::I as c_int;
    }
    if (state & MODE_CMDLINE) != 0 {
        if rs_cmdline_at_end() != 0 {
            return ModeShape::C as c_int;
        } else if rs_cmdline_overstrike() != 0 {
            return ModeShape::Cr as c_int;
        } else {
            return ModeShape::Ci as c_int;
        }
    }
    if nvim_get_finish_op() != 0 {
        return ModeShape::O as c_int;
    }
    if nvim_get_visual_active() != 0 {
        if nvim_get_p_sel_first() == b'e' as c_char {
            return ModeShape::Ve as c_int;
        }
        return ModeShape::V as c_int;
    }

    ModeShape::N as c_int
}

/// Convert a mode name string to its index in shape_table.
///
/// # Safety
/// - `mode` must be a valid, NUL-terminated C string.
/// - Calls C accessor function for `shape_table`.
///
/// # Returns
/// The mode index (0-17) if found, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_mode_str2int(mode: *const c_char) -> c_int {
    use std::ffi::CStr;

    if mode.is_null() {
        return -1;
    }

    let mode_str = CStr::from_ptr(mode);

    for mode_idx in 0..ModeShape::Count as c_int {
        let name_ptr = nvim_get_shape_table_name(mode_idx);
        if !name_ptr.is_null() {
            let name = CStr::from_ptr(name_ptr);
            if mode_str == name {
                return mode_idx;
            }
        }
    }

    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_shape_values() {
        // Verify CursorShape enum values match C definitions
        assert_eq!(CursorShape::Block as c_int, 0);
        assert_eq!(CursorShape::Hor as c_int, 1);
        assert_eq!(CursorShape::Ver as c_int, 2);
    }

    #[test]
    fn test_mode_shape_values() {
        // Verify key ModeShape enum values match C definitions
        assert_eq!(ModeShape::N as c_int, 0); // Normal
        assert_eq!(ModeShape::V as c_int, 1); // Visual
        assert_eq!(ModeShape::I as c_int, 2); // Insert
        assert_eq!(ModeShape::R as c_int, 3); // Replace
        assert_eq!(ModeShape::C as c_int, 4); // Command line Normal
        assert_eq!(ModeShape::Term as c_int, 17); // Terminal
        assert_eq!(ModeShape::Count as c_int, 18); // Total modes
    }

    #[test]
    fn test_mode_shape_all_values() {
        // Verify all ModeShape enum values are sequential
        assert_eq!(ModeShape::N as c_int, 0);
        assert_eq!(ModeShape::V as c_int, 1);
        assert_eq!(ModeShape::I as c_int, 2);
        assert_eq!(ModeShape::R as c_int, 3);
        assert_eq!(ModeShape::C as c_int, 4);
        assert_eq!(ModeShape::Ci as c_int, 5);
        assert_eq!(ModeShape::Cr as c_int, 6);
        assert_eq!(ModeShape::O as c_int, 7);
        assert_eq!(ModeShape::Ve as c_int, 8);
        assert_eq!(ModeShape::Cline as c_int, 9);
        assert_eq!(ModeShape::Status as c_int, 10);
        assert_eq!(ModeShape::Sdrag as c_int, 11);
        assert_eq!(ModeShape::Vsep as c_int, 12);
        assert_eq!(ModeShape::Vdrag as c_int, 13);
        assert_eq!(ModeShape::More as c_int, 14);
        assert_eq!(ModeShape::Morel as c_int, 15);
        assert_eq!(ModeShape::Sm as c_int, 16);
        assert_eq!(ModeShape::Term as c_int, 17);
        assert_eq!(ModeShape::Count as c_int, 18);
    }

    #[test]
    fn test_mode_flags() {
        // Verify mode flag constants match C definitions
        assert_eq!(MODE_CMDLINE, 0x08);
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(MODE_TERMINAL, 0x80);
        assert_eq!(REPLACE_FLAG, 0x100);
        assert_eq!(VREPLACE_FLAG, 0x200);
        // MODE_SHOWMATCH should combine flags
        assert_eq!(MODE_SHOWMATCH, 0x6000 | MODE_INSERT);
    }

    #[test]
    fn test_mode_flags_distinct() {
        // Mode flags should be distinct bit patterns
        assert_eq!(MODE_CMDLINE & MODE_INSERT, 0);
        assert_eq!(MODE_INSERT & MODE_TERMINAL, 0);
        assert_eq!(MODE_TERMINAL & REPLACE_FLAG, 0);
        assert_eq!(REPLACE_FLAG & VREPLACE_FLAG, 0);
    }

    #[test]
    fn test_enum_sizes() {
        // CursorShape and ModeShape should be C-compatible
        assert_eq!(
            std::mem::size_of::<CursorShape>(),
            std::mem::size_of::<c_int>()
        );
        assert_eq!(
            std::mem::size_of::<ModeShape>(),
            std::mem::size_of::<c_int>()
        );
    }
}
