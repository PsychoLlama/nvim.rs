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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::ModMask;
use crate::keycodes::SpecialKey;
use crate::types::Key;
use core::ffi::{CStr, c_int};

use super::{MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, MOUSE_X1, MOUSE_X2, MSCR_RIGHT};
use crate::mouse::state::{mouse_col, mouse_grid, mouse_row};
use crate::types::{ColNr, VarNumber};

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
pub(crate) fn is_mouse_key(c: Key) -> bool {
    matches!(
        c.special(),
        Some(
            SpecialKey::Leftmouse
                | SpecialKey::LeftmouseNm
                | SpecialKey::Leftdrag
                | SpecialKey::Leftrelease
                | SpecialKey::LeftreleaseNm
                | SpecialKey::Mousemove
                | SpecialKey::Middlemouse
                | SpecialKey::Middledrag
                | SpecialKey::Middlerelease
                | SpecialKey::Rightmouse
                | SpecialKey::Rightdrag
                | SpecialKey::Rightrelease
                | SpecialKey::Mousedown
                | SpecialKey::Mouseup
                | SpecialKey::Mouseleft
                | SpecialKey::Mouseright
                | SpecialKey::X1mouse
                | SpecialKey::X1drag
                | SpecialKey::X1release
                | SpecialKey::X2mouse
                | SpecialKey::X2drag
                | SpecialKey::X2release
        )
    )
}

/// How many clicks in a row `mod_mask` records.
///
/// The bits are cumulative -- `ModMask::FOUR_CLICK` *is* 2CLICK|3CLICK -- so the
/// wider counts have to be tested first.
pub(crate) fn click_count(mod_mask: ModMask) -> VarNumber {
    match mod_mask.masked(ModMask::MULTI_CLICK) {
        ModMask::FOUR_CLICK => 4,
        ModMask::THREE_CLICK => 3,
        ModMask::TWO_CLICK => 2,
        _ => 1,
    }
}

/// The four-byte modifier string a `%@Func@` handler receives, one letter per
/// modifier held down and a space where it was not.
pub(crate) fn modifier_letters(mod_mask: ModMask) -> [u8; 5] {
    let held = |bit: ModMask, letter: u8| if mod_mask.has(bit) { letter } else { b' ' };
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
pub(crate) fn skipped_top_lines(skipcol: ColNr, width1: c_int, width2: c_int) -> c_int {
    if skipcol > ColNr::new(width1) {
        (skipcol - width1).get() / width2 + 1
    } else if skipcol > ColNr::ZERO {
        1
    } else {
        0
    }
}

/// The column a horizontal wheel event scrolls to: `'mousescroll'` columns (or
/// a window's width) left or right of `leftcol`, never past the left margin.
pub(crate) fn wheel_leftcol(leftcol: ColNr, step: c_int, direction: c_int) -> ColNr {
    let moved = leftcol + if direction == MSCR_RIGHT { -step } else { step };
    moved.max(ColNr::ZERO)
}

#[cfg(test)]
mod tests {
    //! The key-side decoding a mouse event goes through before `do_mouse`
    //! reads the buffer: which keys are mouse keys, which button and phase
    //! each names, the click count and modifier letters `%@Func@` handlers
    //! receive, and the two pieces of scroll arithmetic.

    use super::*;
    use crate::keycodes::mouse_button;

    const MOUSE_KEYS: [(SpecialKey, c_int, bool, bool); 18] = [
        (SpecialKey::Leftmouse, MOUSE_LEFT, true, false),
        (SpecialKey::Leftdrag, MOUSE_LEFT, false, true),
        (SpecialKey::Leftrelease, MOUSE_LEFT, false, false),
        (SpecialKey::Middlemouse, MOUSE_MIDDLE, true, false),
        (SpecialKey::Middledrag, MOUSE_MIDDLE, false, true),
        (SpecialKey::Middlerelease, MOUSE_MIDDLE, false, false),
        (SpecialKey::Rightmouse, MOUSE_RIGHT, true, false),
        (SpecialKey::Rightdrag, MOUSE_RIGHT, false, true),
        (SpecialKey::Rightrelease, MOUSE_RIGHT, false, false),
        (SpecialKey::X1mouse, MOUSE_X1, true, false),
        (SpecialKey::X1drag, MOUSE_X1, false, true),
        (SpecialKey::X1release, MOUSE_X1, false, false),
        (SpecialKey::X2mouse, MOUSE_X2, true, false),
        (SpecialKey::X2drag, MOUSE_X2, false, true),
        (SpecialKey::X2release, MOUSE_X2, false, false),
        // A move is a drag with no button down.
        (SpecialKey::Mousemove, 3, false, true),
        // The wheel names no button.
        (SpecialKey::Mousedown, 0, false, false),
        (SpecialKey::Mouseleft, 0, false, false),
    ];

    #[test]
    fn a_mouse_key_names_its_button_and_phase() {
        for (key, button, click, drag) in MOUSE_KEYS {
            assert!(is_mouse_key(Key::from(key)), "{key:?}");
            assert_eq!(
                mouse_button(Key::from(key)),
                (button, click, drag),
                "{key:?}"
            );
        }
        for key in [
            SpecialKey::Mouseup,
            SpecialKey::Mouseright,
            SpecialKey::LeftmouseNm,
            SpecialKey::LeftreleaseNm,
        ] {
            assert!(is_mouse_key(Key::from(key)), "{key:?}");
        }
        let mouse = SpecialKey::ALL
            .iter()
            .filter(|key| is_mouse_key(Key::from(**key)))
            .count();
        assert_eq!(mouse, 22);
        for key in [Key::ascii(b'a'), Key::NUL, Key::from(SpecialKey::Up)] {
            assert!(!is_mouse_key(key));
            assert_eq!(mouse_button(key), (0, false, false));
        }
        // `<Ignore>` is not a mouse key, but decodes as a release with no
        // button: what a stray release is turned into.
        assert!(!is_mouse_key(Key::from(SpecialKey::Ignore)));
        assert_eq!(
            mouse_button(Key::from(SpecialKey::Ignore)),
            (3, false, false)
        );
    }

    #[test]
    fn the_click_count_reads_the_widest_bit() {
        assert_eq!(click_count(ModMask::NONE), 1);
        assert_eq!(click_count(ModMask::SHIFT), 1);
        assert_eq!(click_count(ModMask::TWO_CLICK), 2);
        assert_eq!(click_count(ModMask::THREE_CLICK | ModMask::CTRL), 3);
        assert_eq!(click_count(ModMask::FOUR_CLICK), 4);
    }

    #[test]
    fn a_click_handler_gets_letters_and_a_button_name() {
        assert_eq!(&modifier_letters(ModMask::NONE), b"    \0");
        assert_eq!(
            &modifier_letters(ModMask::SHIFT | ModMask::ALT | ModMask::TWO_CLICK),
            b"s a \0"
        );
        assert_eq!(&modifier_letters(ModMask::CTRL | ModMask::META), b" c m\0");
        for (button, name) in [
            (MOUSE_LEFT, c"l"),
            (MOUSE_RIGHT, c"r"),
            (MOUSE_MIDDLE, c"m"),
            (MOUSE_X1, c"x1"),
            (MOUSE_X2, c"x2"),
            (3, c"?"),
        ] {
            assert_eq!(button_name(button), name);
        }
    }

    #[test]
    fn the_scroll_arithmetic() {
        // 'smoothscroll': the first screen line is the first width, every
        // later one the second.
        assert_eq!(skipped_top_lines(ColNr::ZERO, 80, 72), 0);
        assert_eq!(skipped_top_lines(ColNr::new(1), 80, 72), 1);
        assert_eq!(skipped_top_lines(ColNr::new(80), 80, 72), 1);
        assert_eq!(skipped_top_lines(ColNr::new(81), 80, 72), 1);
        assert_eq!(skipped_top_lines(ColNr::new(152), 80, 72), 2);
        assert_eq!(skipped_top_lines(ColNr::new(153), 80, 72), 2);
        assert_eq!(skipped_top_lines(ColNr::new(224), 80, 72), 3);
        // The horizontal wheel never scrolls past the left margin.
        assert_eq!(wheel_leftcol(ColNr::new(10), 6, MSCR_RIGHT), ColNr::new(4));
        assert_eq!(wheel_leftcol(ColNr::new(4), 6, MSCR_RIGHT), ColNr::ZERO);
        assert_eq!(
            wheel_leftcol(ColNr::new(4), 6, super::super::MSCR_LEFT),
            ColNr::new(10)
        );
    }
}
