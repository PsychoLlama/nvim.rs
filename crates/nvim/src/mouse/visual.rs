//! What a click does to the Visual selection: the `jump_to_mouse()` flags the
//! button asks for, and the corner a right click inside a selection moves.
//!
//! Both are pure decisions over the editor's own state, so the module forbids
//! `unsafe` outright.
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
use core::cmp::Ordering;
use core::ffi::c_int;

use super::{
    MOUSE_FOCUS, MOUSE_LEFT, MOUSE_MAY_STOP_VIS, MOUSE_MAY_VIS, MOUSE_RIGHT, vcols_between,
};
use crate::main::{State, mod_mask};
use crate::normal::{
    VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor, visual_mode,
};
use crate::pos::lt;
use crate::state::{MODE_INSERT, MODE_NORMAL};
use crate::types::pos_T;
use crate::winlayer::Win;

/// The `jump_to_mouse()` flags the button and the mode ask for, and -- for a
/// right click in Visual mode -- the selection's corners before the cursor
/// moves.
pub(crate) fn visual_jump_flags(
    jump_flags: &mut c_int,
    is_click: bool,
    which_button: c_int,
    mouse_can_visual: bool,
    old_curwin: Win,
) -> Option<(pos_T, pos_T)> {
    if State.get() & (MODE_NORMAL | MODE_INSERT) == 0
        || mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL)
    {
        return None;
    }

    if which_button == MOUSE_LEFT && mouse_can_visual {
        if is_click {
            // Stop Visual mode for a left click in a window, but not when on
            // a status line.
            if visual_active() {
                *jump_flags |= MOUSE_MAY_STOP_VIS;
            }
        } else {
            *jump_flags |= MOUSE_MAY_VIS;
        }
        return None;
    }

    if which_button != MOUSE_RIGHT {
        return None;
    }
    if !mouse_can_visual {
        *jump_flags |= MOUSE_FOCUS;
        return None;
    }

    // Remember the start and end of visual before moving the cursor.
    let corners = (is_click && visual_active()).then(|| {
        let (cursor, visual) = (old_curwin.w_cursor, visual_anchor());
        if lt(cursor, visual) {
            (cursor, visual)
        } else {
            (visual, cursor)
        }
    });
    *jump_flags |= MOUSE_MAY_VIS;
    *jump_flags |= MOUSE_FOCUS;
    corners
}

/// A right click inside a Visual selection moves the nearest corner to the
/// pointer.  In Visual-block mode the area is divided in four and the corner
/// in the quarter the cursor is in is the one that moves.
pub(crate) fn extend_visual_block(mut win: Win, mut start_visual: pos_T, mut end_visual: pos_T) {
    // When ALT is pressed make Visual mode blockwise.
    if mod_mask.get().has(ModMask::ALT) {
        set_visual_mode(VisualMode::BLOCK);
    }

    if visual_mode().is_block() {
        let (leftcol, rightcol) = vcols_between(win, start_visual, end_visual);
        end_visual.col = if win.w_curswant > (leftcol + rightcol) / 2 {
            leftcol
        } else {
            rightcol
        };
        if win.w_cursor.lnum >= (start_visual.lnum + end_visual.lnum) / 2 {
            end_visual.lnum = start_visual.lnum;
        }

        // Move VIsual to the right column.
        start_visual = win.w_cursor; // save the cursor pos
        win.w_cursor = end_visual;
        win.coladvance(end_visual.col);
        set_visual_anchor(win.w_cursor);
        win.w_cursor = start_visual; // restore the cursor
        return;
    }

    // If the click is before the start of visual, change the start.  If the
    // click is after the end of visual, change the end.  If the click is
    // inside the visual, change the closest side.
    let cursor = win.w_cursor;
    set_visual_anchor(if lt(cursor, start_visual) {
        end_visual
    } else if lt(end_visual, cursor) {
        start_visual
    } else if end_visual.lnum == start_visual.lnum {
        // In the same line, compare column number.
        if cursor.col - start_visual.col > end_visual.col - cursor.col {
            start_visual
        } else {
            end_visual
        }
    } else {
        // In different lines, compare line number.
        let diff = (cursor.lnum - start_visual.lnum) - (end_visual.lnum - cursor.lnum);
        match diff.cmp(&0) {
            Ordering::Greater => start_visual, // closest to end
            Ordering::Less => end_visual,      // closest to start
            // In the middle line.
            Ordering::Equal => {
                if cursor.col < (start_visual.col + end_visual.col) / 2 {
                    end_visual
                } else {
                    start_visual
                }
            }
        }
    });
}
