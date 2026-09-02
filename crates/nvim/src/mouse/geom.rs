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

use crate::keycodes::Key;
use crate::keycodes::ModMask;
use core::ffi::{CStr, c_char, c_int};

use super::{MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, MOUSE_X1, MOUSE_X2, MSCR_RIGHT};
use crate::main::{mouse_col, mouse_grid, mouse_row};
use crate::types::{colnr_T, varnumber_T};

/// Where a mouse event landed: a grid handle, and a row and column within it.
///
/// The C carries these as three `int *` because [`super::find_win_inner`] and
/// friends rewrite them in place, from screen coordinates into coordinates
/// relative to the window they found.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct MousePos {
    /// Grid handle: `DEFAULT_GRID_HANDLE` for the screen, 0 for "ask the
    /// compositor", a window's own handle under `ext_multigrid`.
    pub grid: c_int,
    pub row: c_int,
    pub col: c_int,
}

impl MousePos {
    /// Where the last mouse event landed.
    pub(crate) fn current() -> Self {
        Self {
            grid: mouse_grid.get(),
            row: mouse_row.get(),
            col: mouse_col.get(),
        }
    }
}

/// Whether `c` is a mouse key.
pub(crate) fn is_mouse_key(c: c_int) -> bool {
    matches!(
        Key::try_from(c),
        Ok(Key::Leftmouse
            | Key::LeftmouseNm
            | Key::Leftdrag
            | Key::Leftrelease
            | Key::LeftreleaseNm
            | Key::Mousemove
            | Key::Middlemouse
            | Key::Middledrag
            | Key::Middlerelease
            | Key::Rightmouse
            | Key::Rightdrag
            | Key::Rightrelease
            | Key::Mousedown
            | Key::Mouseup
            | Key::Mouseleft
            | Key::Mouseright
            | Key::X1mouse
            | Key::X1drag
            | Key::X1release
            | Key::X2mouse
            | Key::X2drag
            | Key::X2release)
    )
}

/// The `KE_*` half of a key code, as C's `KEY2TERMCAP1` reads it: the low byte
/// of `-c` shifted down.  Nothing checks that `c` is a key code.
pub(crate) fn key_extra(c: c_int) -> c_int {
    ((-c as u32 >> 8) & 0xff) as c_int
}

/// How many clicks in a row `mod_mask` records.
///
/// The bits are cumulative -- `ModMask::FOUR_CLICK` *is* 2CLICK|3CLICK -- so the
/// wider counts have to be tested first.
pub(crate) fn click_count(mod_mask: ModMask) -> varnumber_T {
    match mod_mask.masked(ModMask::MULTI_CLICK) {
        ModMask::FOUR_CLICK => 4,
        ModMask::THREE_CLICK => 3,
        ModMask::TWO_CLICK => 2,
        _ => 1,
    }
}

/// The four-byte modifier string a `%@Func@` handler receives, one letter per
/// modifier held down and a space where it was not.
pub(crate) fn modifier_letters(mod_mask: ModMask) -> [c_char; 5] {
    let held = |bit: ModMask, letter: u8| (if mod_mask.has(bit) { letter } else { b' ' }) as c_char;
    [
        held(ModMask::SHIFT, b's'),
        held(ModMask::CTRL, b'c'),
        held(ModMask::ALT, b'a'),
        held(ModMask::META, b'm'),
        0,
    ]
}

/// The name a `%@Func@` handler receives for the button that was pressed.
pub(crate) fn button_name(which_button: c_int) -> &'static CStr {
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
pub(crate) fn skipped_top_lines(skipcol: colnr_T, width1: c_int, width2: c_int) -> c_int {
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
pub(crate) fn wheel_leftcol(leftcol: colnr_T, step: c_int, direction: c_int) -> colnr_T {
    let moved = leftcol + if direction == MSCR_RIGHT { -step } else { step };
    moved.max(0)
}
