//! Cursor shape handling for Neovim
//!
//! Provides Rust implementations of cursor shape functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]

use std::ffi::c_int;

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
    N = 0,      // Normal mode
    V = 1,      // Visual mode
    I = 2,      // Insert mode
    R = 3,      // Replace mode
    C = 4,      // Command line Normal mode
    Ci = 5,     // Command line Insert mode
    Cr = 6,     // Command line Replace mode
    O = 7,      // Operator-pending mode
    Ve = 8,     // Visual mode with 'selection' exclusive
    Cline = 9,  // On command line
    Status = 10,
    Sdrag = 11,
    Vsep = 12,
    Vdrag = 13,
    More = 14,
    Morel = 15,
    Sm = 16,    // showing matching paren
    Term = 17,  // Terminal mode
    Count = 18,
}

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
