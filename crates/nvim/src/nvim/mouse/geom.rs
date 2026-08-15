//! The mouse's arithmetic: the screen position an event names, the key codes
//! that carry one, and the small sums the other five children work in.
//!
//! Nothing here touches the editor's memory, so the module forbids `unsafe`
//! outright.  [`MousePos`] is the value the C passes as three `int *` -- a
//! grid handle plus a row and column that `find_win_inner`/`find_win_outer`
//! rewrite from screen coordinates into window-relative ones.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_char, c_int};

use super::{
    MOD_MASK_2CLICK, MOD_MASK_3CLICK, MOD_MASK_4CLICK, MOD_MASK_ALT, MOD_MASK_CTRL, MOD_MASK_META,
    MOD_MASK_MULTI_CLICK, MOD_MASK_SHIFT, MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, MOUSE_X1,
    MOUSE_X2, MSCR_RIGHT,
};
use crate::src::nvim::keycodes::{
    K_LEFTDRAG, K_LEFTMOUSE, K_LEFTMOUSE_NM, K_LEFTRELEASE, K_LEFTRELEASE_NM, K_MIDDLEDRAG,
    K_MIDDLEMOUSE, K_MIDDLERELEASE, K_MOUSEDOWN, K_MOUSELEFT, K_MOUSEMOVE, K_MOUSERIGHT, K_MOUSEUP,
    K_RIGHTDRAG, K_RIGHTMOUSE, K_RIGHTRELEASE, K_X1DRAG, K_X1MOUSE, K_X1RELEASE, K_X2DRAG,
    K_X2MOUSE, K_X2RELEASE,
};
use crate::src::nvim::main::{mouse_col, mouse_grid, mouse_row};
use crate::src::nvim::types::{colnr_T, varnumber_T};

/// Where a mouse event landed: a grid handle, and a row and column within it.
///
/// The C carries these as three `int *` because [`super::find_win_inner`] and
/// friends rewrite them in place, from screen coordinates into coordinates
/// relative to the window they found.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MousePos {
    /// Grid handle: `DEFAULT_GRID_HANDLE` for the screen, 0 for "ask the
    /// compositor", a window's own handle under `ext_multigrid`.
    pub grid: c_int,
    pub row: c_int,
    pub col: c_int,
}

impl MousePos {
    /// Where the last mouse event landed.
    pub fn current() -> Self {
        Self {
            grid: mouse_grid.get(),
            row: mouse_row.get(),
            col: mouse_col.get(),
        }
    }
}

/// Whether `c` is a mouse key.
pub fn is_mouse_key(c: c_int) -> bool {
    matches!(
        c,
        K_LEFTMOUSE
            | K_LEFTMOUSE_NM
            | K_LEFTDRAG
            | K_LEFTRELEASE
            | K_LEFTRELEASE_NM
            | K_MOUSEMOVE
            | K_MIDDLEMOUSE
            | K_MIDDLEDRAG
            | K_MIDDLERELEASE
            | K_RIGHTMOUSE
            | K_RIGHTDRAG
            | K_RIGHTRELEASE
            | K_MOUSEDOWN
            | K_MOUSEUP
            | K_MOUSELEFT
            | K_MOUSERIGHT
            | K_X1MOUSE
            | K_X1DRAG
            | K_X1RELEASE
            | K_X2MOUSE
            | K_X2DRAG
            | K_X2RELEASE
    )
}

/// The `KE_*` half of a key code, as C's `KEY2TERMCAP1` reads it: the low byte
/// of `-c` shifted down.  Nothing checks that `c` is a key code.
pub fn key_extra(c: c_int) -> c_int {
    ((-c as u32 >> 8) & 0xff) as c_int
}

/// How many clicks in a row `mod_mask` records.
///
/// The bits are cumulative -- `MOD_MASK_4CLICK` *is* 2CLICK|3CLICK -- so the
/// wider counts have to be tested first.
pub fn click_count(mod_mask: c_int) -> varnumber_T {
    match mod_mask & MOD_MASK_MULTI_CLICK {
        MOD_MASK_4CLICK => 4,
        MOD_MASK_3CLICK => 3,
        MOD_MASK_2CLICK => 2,
        _ => 1,
    }
}

/// The four-byte modifier string a `%@Func@` handler receives, one letter per
/// modifier held down and a space where it was not.
pub fn modifier_letters(mod_mask: c_int) -> [c_char; 5] {
    let held = |bit: c_int, letter: u8| (if mod_mask & bit != 0 { letter } else { b' ' }) as c_char;
    [
        held(MOD_MASK_SHIFT, b's'),
        held(MOD_MASK_CTRL, b'c'),
        held(MOD_MASK_ALT, b'a'),
        held(MOD_MASK_META, b'm'),
        0,
    ]
}

/// The name a `%@Func@` handler receives for the button that was pressed.
pub fn button_name(which_button: c_int) -> &'static CStr {
    match which_button {
        MOUSE_LEFT => c"l",
        MOUSE_RIGHT => c"r",
        MOUSE_MIDDLE => c"m",
        MOUSE_X1 => c"x1",
        MOUSE_X2 => c"x2",
        _ => c"?",
    }
}

/// Screen lines of the top line that `'smoothscroll'` has clipped away, which
/// [`super::comp_pos`] must not count when walking down the window.
///
/// A similar formula is used in `curs_columns()`; see `move/arith.rs`.
pub fn skipped_top_lines(skipcol: colnr_T, width1: c_int, width2: c_int) -> c_int {
    if skipcol > width1 {
        (skipcol - width1) / width2 + 1
    } else if skipcol > 0 {
        1
    } else {
        0
    }
}

/// The column a horizontal wheel event scrolls to: `'mousescroll'` columns (or
/// a window's width) left or right of `leftcol`, never past the left margin.
pub fn wheel_leftcol(leftcol: colnr_T, step: c_int, direction: c_int) -> colnr_T {
    let moved = leftcol + if direction == MSCR_RIGHT { -step } else { step };
    moved.max(0)
}
