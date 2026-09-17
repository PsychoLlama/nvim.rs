//! The wheel, and the mouse in Insert mode -- `do_mousescroll()`,
//! `ins_mouse()` and `ins_mousescroll()`.
//!
//! [`do_mousescroll`] applies `'mousescroll'` to a wheel event, scrolling by
//! lines or by pages and honouring `'scrolloff'`; the `ins_*` pair is the
//! Insert-mode form, which has to leave and re-enter Insert mode around the
//! move so undo and `'backspace'` see a sane state.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::Key;
use crate::keycodes::ModMask;

use super::*;
use crate::buffer::buf_is_prompt;
use crate::drawscreen::redraw_statuslines;
use crate::edit::{set_can_cindent, start_arrow, undisplay_dollar};
use crate::getchar::state::mod_mask;
use crate::mouse::state::{mouse_col, mouse_row};
use crate::r#move::pagescroll;
use crate::narrow::number_as_int;
use crate::normal::nv_scroll_line;
use crate::ops::clear_oparg;
use crate::option::vars::{P_MOUSEM, p_mousem, p_mousescroll_hor, p_mousescroll_vert};
use crate::popupmenu::pum_visible;
use crate::pos::equalpos;
use crate::search::{BACKWARD, FORWARD};
use crate::siemsg;
use crate::state::MODE_NORMAL;
use crate::state::mode::State;
use crate::types::{CmdArg, Direction, OpArg};

/// A mouse click in Insert mode: place the cursor, then get Insert mode's own
/// bookkeeping back in order around the move.
pub(crate) fn ins_mouse(c: c_int) {
    let old_curwin = Win::current();

    undisplay_dollar();
    let mut tpos = old_curwin.w_cursor;

    if do_mouse(None, c, BACKWARD as c_int, 1, false) {
        let new_curwin = Win::current();
        if new_curwin != old_curwin && old_curwin.is_valid() {
            // Mouse took us to another window.  We need to go back to the
            // previous one to stop insert there properly.
            old_curwin.make_current();
            old_curwin.buffer().make_current();
            if buf_is_prompt(old_curwin.buffer_or_none()) {
                // Restart Insert mode when re-entering the prompt buffer.
                old_curwin.buffer().b_prompt_insert = 'A' as c_int;
            }
        }
        let end = if old_curwin.is_current() {
            &raw mut tpos
        } else {
            ptr::null_mut()
        };
        // SAFETY: `tpos` is a live local position; a null end means "the
        // cursor moved to another window".
        unsafe { start_arrow(end) };
        if !new_curwin.is_current() && new_curwin.is_valid() {
            new_curwin.make_current();
            new_curwin.buffer().make_current();
        }
        set_can_cindent(true);
    }

    // Redraw status lines (in case another window became active).
    redraw_statuslines();
}

/// Common mouse wheel scrolling, shared between Insert mode and NV modes.
///
/// Default action is to scroll `'mousescroll'` lines (or columns, depending on
/// the scroll direction) or one page when Shift or Ctrl is used.  Direction is
/// indicated by `cmd_arg->arg`: `K_MOUSEUP` is `MSCR_UP`, `K_MOUSEDOWN` is
/// `MSCR_DOWN`, `K_MOUSELEFT` is `MSCR_LEFT` and `K_MOUSERIGHT` is
/// `MSCR_RIGHT`.
///
/// `curwin` may have been changed to the window that should be scrolled and
/// differ from the window that actually has focus.
pub(crate) fn do_mousescroll(cmd_arg: &mut CmdArg) {
    let shift_or_ctrl = mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL);
    let win = Win::current();
    let arg = cmd_arg.arg;

    if arg == MSCR_UP || arg == MSCR_DOWN {
        // Vertical scrolling.
        if State.get() & MODE_NORMAL != 0 && shift_or_ctrl {
            // Whole page up or down.
            let dir = if arg != 0 { FORWARD } else { BACKWARD } as Direction;
            pagescroll(dir, 1, false);
            return;
        }
        let count = if shift_or_ctrl {
            win.w_botline - win.w_topline
        } else {
            number_as_int(p_mousescroll_vert.get())
        };
        // The count is written even when it is zero, as the C is, and
        // `nv_scroll_line` reads the counts just written.
        cmd_arg.count1 = count;
        if count > 0 {
            cmd_arg.count0 = count;
            nv_scroll_line(cmd_arg);
        }
        return;
    }

    // Horizontal scrolling.
    let step = if shift_or_ctrl {
        win.w_view_width
    } else {
        number_as_int(p_mousescroll_hor.get())
    };
    do_mousescroll_horiz(win, wheel_leftcol(win.w_leftcol, step, arg));
}

/// Scrolling in Insert mode in direction `dir`, which is one of the `MSCR_`
/// values.
pub(crate) fn ins_mousescroll(dir: c_int) {
    // SAFETY: `CmdArg` and `OpArg` are C aggregates of scalars and
    // pointers, which is what the C's `CLEAR_FIELD` zeroes; `clear_oparg`
    // then initialises the operator properly.
    let (mut cmd_arg, mut oa): (CmdArg, OpArg) = unsafe { core::mem::zeroed() };
    // SAFETY: a live local operator.
    unsafe { clear_oparg(&raw mut oa) };
    cmd_arg.oap = &raw mut oa;
    cmd_arg.arg = dir;
    cmd_arg.cmdchar = match dir {
        MSCR_UP => Key::Mouseup.code(),
        MSCR_DOWN => Key::Mousedown.code(),
        MSCR_LEFT => Key::Mouseleft.code(),
        MSCR_RIGHT => Key::Mouseright.code(),
        _ => {
            siemsg!("Invalid ins_mousescroll() argument: {}", dir);
            0
        }
    };

    let old_curwin = Win::current();
    if mouse_row.get() >= 0 && mouse_col.get() >= 0 {
        // Find the window at the mouse pointer coordinates.
        // NOTE: Must restore "curwin" to "old_curwin" before returning!
        let mut pos = MousePos::current();
        let Some(win) = find_win_inner(&mut pos) else {
            return;
        };
        win.make_current();
        win.buffer().make_current();
    }

    let mut win = Win::current();
    if win == old_curwin {
        // Don't scroll the current window if the popup menu is visible.
        if pum_visible() {
            return;
        }
        undisplay_dollar();
    }

    let orig_cursor = win.w_cursor;

    // Call the common mouse scroll function shared with other modes.
    do_mousescroll(&mut cmd_arg);

    win = Win::current();
    win.w_redr_status = true;
    // `old_curwin` was live when it was taken and nothing above closes a
    // window, so it is still the window to go back to.
    let restored = old_curwin;
    restored.make_current();
    restored.buffer().make_current();

    // Upstream compares the *restored* window's cursor against the cursor of
    // the window that was scrolled, which are two different windows whenever
    // the wheel was over another one.
    if !equalpos(restored.w_cursor, orig_cursor) {
        let mut orig_cursor = orig_cursor;
        // SAFETY: a live local position.
        unsafe { start_arrow(&raw mut orig_cursor) };
        set_can_cindent(true);
    }
}

/// Whether `'mousemodel'` is set to "popup" or "popup_setpos".
pub(crate) fn mouse_model_popup() -> bool {
    P_MOUSEM.first_byte() == b'p'
}

/// Whether `'mousemodel'` is exactly "popup_setpos", which moves the cursor
/// before showing the menu.
pub(crate) fn mouse_model_popup_setpos() -> bool {
    // SAFETY: an option string is NUL-terminated, never null.
    p_mousem(|value| value == c"popup_setpos")
}
