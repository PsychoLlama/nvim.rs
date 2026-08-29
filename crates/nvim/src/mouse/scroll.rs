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

use core::ffi::CStr;

use super::*;
use crate::buffer::buf_is_prompt;
use crate::drawscreen::redraw_statuslines;
use crate::edit::{set_can_cindent, start_arrow, undisplay_dollar};
use crate::keycodes::{K_MOUSEDOWN, K_MOUSELEFT, K_MOUSERIGHT, K_MOUSEUP};
use crate::main::{
    State, curbuf, curwin, mod_mask, mouse_col, mouse_row, p_mousem, p_mousescroll_hor,
    p_mousescroll_vert,
};
use crate::r#move::pagescroll;
use crate::normal::nv_scroll_line;
use crate::ops::clear_oparg;
use crate::popupmenu::pum_visible;
use crate::pos::equalpos;
use crate::search::{BACKWARD, FORWARD};
use crate::siemsg;
use crate::state::MODE_NORMAL;
use crate::types::{Direction, cmdarg_T, oparg_T};

/// A mouse click in Insert mode: place the cursor, then get Insert mode's own
/// bookkeeping back in order around the move.
///
/// # Safety
/// `c` must be a mouse key code.
pub(crate) unsafe fn ins_mouse(c: c_int) {
    // SAFETY: `curwin` is live from startup to exit.
    let old_curwin = unsafe { Win::current() };

    // SAFETY: both only touch the current window's Insert-mode state.
    unsafe { undisplay_dollar() };
    let mut tpos = old_curwin.w_cursor;

    // SAFETY: `do_mouse` accepts a null operator.
    if unsafe { do_mouse(ptr::null_mut(), c, BACKWARD as c_int, 1, false) } {
        // SAFETY: `curwin` is live.
        let new_curwin = unsafe { Win::current() };
        if new_curwin != old_curwin && old_curwin.is_valid() {
            // Mouse took us to another window.  We need to go back to the
            // previous one to stop insert there properly.
            curwin.set(old_curwin.raw());
            curbuf.set(old_curwin.buffer().raw());
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
            curwin.set(new_curwin.raw());
            curbuf.set(new_curwin.buffer().raw());
        }
        set_can_cindent(true);
    }

    // Redraw status lines (in case another window became active).
    // SAFETY: only schedules a redraw.
    unsafe { redraw_statuslines() };
}

/// Common mouse wheel scrolling, shared between Insert mode and NV modes.
///
/// Default action is to scroll `'mousescroll'` lines (or columns, depending on
/// the scroll direction) or one page when Shift or Ctrl is used.  Direction is
/// indicated by `cap->arg`: `K_MOUSEUP` is `MSCR_UP`, `K_MOUSEDOWN` is
/// `MSCR_DOWN`, `K_MOUSELEFT` is `MSCR_LEFT` and `K_MOUSERIGHT` is
/// `MSCR_RIGHT`.
///
/// `curwin` may have been changed to the window that should be scrolled and
/// differ from the window that actually has focus.
///
/// # Safety
/// `cap` must be a live command argument.
pub(crate) unsafe fn do_mousescroll(cap: *mut cmdarg_T) {
    let shift_or_ctrl = mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0;
    // SAFETY: `curwin` is live from startup to exit.
    let win = unsafe { Win::current() };
    // SAFETY: the caller's promise.
    let arg = unsafe { (*cap).arg };

    if arg == MSCR_UP || arg == MSCR_DOWN {
        // Vertical scrolling.
        if State.get() & MODE_NORMAL != 0 && shift_or_ctrl {
            // Whole page up or down.
            let dir = if arg != 0 { FORWARD } else { BACKWARD } as Direction;
            // SAFETY: scrolls the current window.
            unsafe { pagescroll(dir, 1, false) };
            return;
        }
        let count = if shift_or_ctrl {
            win.w_botline - win.w_topline
        } else {
            p_mousescroll_vert.get() as c_int
        };
        // The count is written even when it is zero, as the C is.
        // SAFETY: the caller's promise, and `nv_scroll_line` reads the counts
        // just written.
        unsafe { (*cap).count1 = count };
        if count > 0 {
            unsafe { (*cap).count0 = count };
            unsafe { nv_scroll_line(cap) };
        }
        return;
    }

    // Horizontal scrolling.
    let step = if shift_or_ctrl {
        win.w_view_width
    } else {
        p_mousescroll_hor.get() as c_int
    };
    do_mousescroll_horiz(win, wheel_leftcol(win.w_leftcol, step, arg));
}

/// Scrolling in Insert mode in direction `dir`, which is one of the `MSCR_`
/// values.
pub(crate) fn ins_mousescroll(dir: c_int) {
    // SAFETY: `cmdarg_T` and `oparg_T` are C aggregates of scalars and
    // pointers, which is what the C's `CLEAR_FIELD` zeroes; `clear_oparg`
    // then initialises the operator properly.
    let (mut cap, mut oa): (cmdarg_T, oparg_T) = unsafe { core::mem::zeroed() };
    // SAFETY: a live local operator.
    unsafe { clear_oparg(&raw mut oa) };
    cap.oap = &raw mut oa;
    cap.arg = dir;
    cap.cmdchar = match dir {
        MSCR_UP => K_MOUSEUP,
        MSCR_DOWN => K_MOUSEDOWN,
        MSCR_LEFT => K_MOUSELEFT,
        MSCR_RIGHT => K_MOUSERIGHT,
        _ => {
            siemsg!("Invalid ins_mousescroll() argument: {}", dir);
            0
        }
    };

    let old_curwin = curwin.get();
    if mouse_row.get() >= 0 && mouse_col.get() >= 0 {
        // Find the window at the mouse pointer coordinates.
        // NOTE: Must restore "curwin" to "old_curwin" before returning!
        let mut pos = MousePos::current();
        let Some(win) = find_win_inner(&mut pos) else {
            return;
        };
        curwin.set(win.raw());
        curbuf.set(win.buffer().raw());
    }

    // SAFETY: `curwin` is live from startup to exit.
    let mut win = unsafe { Win::current() };
    if win.raw() == old_curwin {
        // Don't scroll the current window if the popup menu is visible.
        if pum_visible() {
            return;
        }
        // SAFETY: only touches Insert mode's own state.
        unsafe { undisplay_dollar() };
    }

    let orig_cursor = win.w_cursor;

    // Call the common mouse scroll function shared with other modes.
    // SAFETY: `cap` is a live local command argument.
    unsafe { do_mousescroll(&raw mut cap) };

    // SAFETY: `curwin` may have moved under `do_mousescroll`.
    win = unsafe { Win::current() };
    win.w_redr_status = true;
    curwin.set(old_curwin);
    // SAFETY: `old_curwin` was live and nothing above closes a window.
    let restored = unsafe { Win::current() };
    curbuf.set(restored.buffer().raw());

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
    // SAFETY: an option string is NUL-terminated, never null.
    unsafe { *p_mousem.get() == 'p' as c_char }
}

/// Whether `'mousemodel'` is exactly "popup_setpos", which moves the cursor
/// before showing the menu.
pub(crate) fn mouse_model_popup_setpos() -> bool {
    // SAFETY: an option string is NUL-terminated, never null.
    unsafe { CStr::from_ptr(p_mousem.get()) == c"popup_setpos" }
}
