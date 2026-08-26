//! Whole-page movement and `'cursorbind'` -- `pagescroll()` and
//! `do_check_cursorbind()`.
//!
//! [`pagescroll`] is CTRL-F/CTRL-B and the `'smoothscroll'`-aware half-page
//! forms: a page is a window's worth of *screen* lines, so it walks folds and
//! wrapped lines rather than counting buffer lines.
//! [`do_check_cursorbind`] propagates the cursor to every other
//! `'cursorbind'` window, which is a movement decision made once per command
//! rather than per window.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::ptr;

use super::*;
use crate::cursor::check_cursor;
use crate::diff::diff_get_corresponding_line;
use crate::drawscreen::UPD_VALID;
use crate::edit::{BeginlineOpts, beginline, cursor_down_inner, cursor_up_inner};
use crate::getchar::beep_flush;
use crate::global_cell::GlobalCell;
use crate::main::{Rows, curbuf, curwin, firstwin, lastwin, p_sol, p_window, restart_edit};
use crate::mbyte::mb_adjust_cursor;
use crate::normal::{
    nv_g_home_m_cmd, nv_screengo, set_visual_active, set_visual_select, visual_active,
    visual_select,
};
use crate::pos::equalpos;
use crate::search::FORWARD;
use crate::types::{
    Direction, FAIL, OK, OptInt, cmdarg_T, colnr_T, linenr_T, oparg_T, pos_T, win_T,
};
use crate::winlayer::{Buf, Win};

/// A command with nothing set, as C's `cmdarg_T ca = { 0 }` leaves it.
const CMDARG_ZERO: cmdarg_T = cmdarg_T {
    oap: ptr::null_mut(),
    prechar: 0,
    cmdchar: 0,
    nchar: 0,
    nchar_composing: [0; 32],
    nchar_len: 0,
    extra_char: 0,
    opcount: 0,
    count0: 0,
    count1: 0,
    arg: 0,
    retval: 0,
    searchbuf: ptr::null_mut(),
};

/// What `pagescroll` measured before it scrolled: the buffer's length, and
/// the cursor position a half-page scroll puts back before moving it itself.
#[derive(Clone, Copy)]
struct Saved {
    buflen: linenr_T,
    cursor: pos_T,
    curswant: colnr_T,
}

/// Move the screen `count` (half) pages backwards (`dir` is `BACKWARD`) or
/// forwards (`FORWARD`) and update the screen, moving the cursor with it and
/// -- for the half-page CTRL-D/CTRL-U -- not revealing lines past the end of
/// the buffer. Answers `FAIL` when neither the viewport nor the cursor moved.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn pagescroll(dir: Direction, count: c_int, half: bool) -> c_int {
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    let saved = Saved {
        buflen: win.buffer().line_count(),
        cursor: win.w_cursor,
        curswant: win.w_curswant,
    };
    // One operator and one command, shared by both arms as upstream shares
    // them: `nv_screengo()` fills in the operator that `nv_g_home_m_cmd()`
    // reads back through `ca`.
    let mut oa = oparg_T::ZERO;
    let mut ca = CMDARG_ZERO;
    ca.oap = &raw mut oa;

    let mut did_move = if half {
        // SAFETY: the caller's promise; `oa` is an operator of this frame.
        unsafe { half_page(win, dir, count, saved, &raw mut oa) }
    } else {
        whole_page(win, dir, count)
    };

    if win.scrolloff() > 0 {
        win.cursor_correct();
    }
    // Move the cursor to the first line of a closed fold.
    win.fold_adjust_cursor();

    did_move =
        did_move || saved.cursor.col != win.w_cursor.col || saved.cursor.lnum != win.w_cursor.lnum;

    // An error when neither the viewport nor the cursor changed.
    if !did_move {
        // SAFETY: beeping reads editor state, not a pointer of ours.
        beep_flush();
    } else if win.w_onebuf_opt.wo_sms == 0 {
        // SAFETY: the caller's promise -- this moves `curwin`'s cursor.
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    } else if p_sol.get() != 0 {
        // SAFETY: the caller's promise; `ca` is a command of this frame.
        unsafe { nv_g_home_m_cmd(&raw mut ca) };
    }

    if did_move { OK } else { FAIL }
}

/// CTRL-D/CTRL-U: scroll 'scroll' screen lines without revealing lines past
/// the end of the buffer, and move the cursor by as many screen lines.
///
/// # Safety
/// `win` must be the current window and `oap` an operator the caller owns.
unsafe fn half_page(
    mut win: Win,
    dir: Direction,
    count: c_int,
    saved: Saved,
    oap: *mut oparg_T,
) -> bool {
    // Scroll [count], 'scroll', or the window height in lines.
    let mut count = count;
    if count != 0 {
        win.w_onebuf_opt.wo_scr = win.w_view_height.min(count) as OptInt;
    }
    count = win.w_view_height.min(win.w_onebuf_opt.wo_scr as c_int);

    let mut curscount = count;
    // Adjust the count so as not to reveal lines past the end of the buffer.
    if dir == FORWARD
        && (win.w_topline + win.w_view_height as linenr_T + count as linenr_T > saved.buflen
            || win.lines_concealed())
    {
        let mut n = win.corrected_plines(win.w_topline, false).0;
        if n - count < win.w_view_height && win.w_topline < saved.buflen {
            n += win.plines_range(win.w_topline + 1, saved.buflen, win.w_view_height + count);
        }
        if n < win.w_view_height + count {
            count = n - win.w_view_height;
        }
    }

    // (Try to) scroll the window unless already at the end of the buffer.
    let mut did_move = false;
    if count > 0 {
        did_move = scroll_with_sms(win, dir, count, &mut curscount);
        win.w_cursor.lnum = saved.cursor.lnum;
        win.w_cursor.col = saved.cursor.col;
        win.w_curswant = saved.curswant;
    }

    // Move the cursor by the same number of screen lines, skipping over
    // concealed lines as those were not counted in `curscount` either.
    if win.w_onebuf_opt.wo_wrap != 0 {
        // SAFETY: the caller's promise.
        unsafe { nv_screengo(oap, dir, curscount, true) };
    } else if dir == FORWARD {
        // SAFETY: a live window.
        unsafe { cursor_down_inner(win.raw(), curscount, true) };
    } else {
        // SAFETY: a live window.
        unsafe { cursor_up_inner(win.raw(), curscount as linenr_T, true) };
    }
    did_move
}

/// CTRL-F/CTRL-B: scroll `count` times 'window' or the window height in
/// lines, and put the cursor at the top or bottom of the new view.
fn whole_page(mut win: Win, dir: Direction, count: c_int) -> bool {
    // With a single window and a 'window' smaller than the screen, that is
    // the page; otherwise a page is the window less its overlap.
    let page = if firstwin.get() == lastwin.get()
        && p_window.get() > 0
        && p_window.get() < (Rows.get() - 1) as OptInt
    {
        (p_window.get() as c_int - 2).max(1)
    } else {
        get_scroll_overlap(win, dir)
    };
    // `scroll_with_sms` corrects the count it was given, which upstream
    // aliases with the count itself here.
    let mut count = count * page;
    let did_move = scroll_with_sms(win, dir, count, &mut count);

    if did_move {
        // Place the cursor at the top or bottom of the window.
        win.validate_botline();
        let lnum = if dir == FORWARD {
            win.w_topline
        } else {
            win.w_botline - 1
        };
        // In silent Ex mode `w_botline - 1` may be 0, but the cursor's line
        // number has to be at least 1.
        win.w_cursor.lnum = lnum.max(1);
    }
    did_move
}

/// Give every 'cursorbind' window in this tab page the current window's cursor
/// position, adjusted for 'diff' where the two buffers disagree.
///
/// # Safety
/// The editor's window list must be valid.
pub unsafe fn do_check_cursorbind() {
    static prev_curwin: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut::<win_T>());
    static prev_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });

    // SAFETY: `curwin` is set from startup to exit.
    let old_curwin = unsafe { Win::current() };
    if old_curwin.raw() == prev_curwin.get() && equalpos(old_curwin.w_cursor, prev_cursor.get()) {
        return;
    }
    prev_curwin.set(old_curwin.raw());
    prev_cursor.set(old_curwin.w_cursor);

    let cursor = old_curwin.w_cursor;
    let curswant = old_curwin.w_curswant;
    let set_curswant = old_curwin.w_set_curswant;
    // SAFETY: `curbuf` is set from startup to exit.
    let old_curbuf = unsafe { Buf::current() };
    let old_visual_select = visual_select();
    let old_visual_active = visual_active();

    // Loop through the cursorbound windows.
    set_visual_active(false);
    set_visual_select(false);
    // Upstream asks the tab page for its window list, but the *current* tab
    // page's windows always hang off `firstwin` -- `tp_firstwin` is only
    // filled in when a tab page is left.
    // SAFETY: the editor's window list holds live windows.
    let mut next = (!firstwin.get().is_null()).then(|| unsafe { Win::new(firstwin.get()) });
    while let Some(mut win) = next {
        curwin.set(win.raw());
        curbuf.set(win.buffer().raw());
        // Skip the original window, and the ones with 'nocursorbind'.
        if win != old_curwin && win.w_onebuf_opt.wo_crb != 0 {
            win.w_cursor.lnum = if win.w_onebuf_opt.wo_diff != 0 {
                // SAFETY: a live buffer.
                unsafe { diff_get_corresponding_line(old_curbuf.raw(), cursor.lnum) }
            } else {
                cursor.lnum
            };
            win.w_cursor.col = cursor.col;
            win.w_cursor.coladd = cursor.coladd;
            win.w_curswant = curswant;
            win.w_set_curswant = set_curswant;

            // Make sure the cursor is in a valid position. `restart_edit` is
            // set for the duration so that it may sit beyond the end of line.
            let restart_edit_save = restart_edit.get();
            restart_edit.set(1);
            // SAFETY: a live window.
            unsafe { check_cursor(win.raw()) };
            // Avoid a scroll here for the cursor position: 'scrollbind' is
            // more important.
            if win.w_onebuf_opt.wo_scb == 0 {
                win.validate_cursor();
            }
            restart_edit.set(restart_edit_save);

            // Correct the cursor for a multi-byte character.
            // SAFETY: `curwin` is the window this loop just switched to.
            unsafe { mb_adjust_cursor() };
            win.redraw_later(UPD_VALID);

            // Only scroll when 'scrollbind' has not done it already.
            if win.w_onebuf_opt.wo_scb == 0 {
                win.update_topline();
            }
            win.w_redr_status = true;
        }
        next = win.next();
    }

    set_visual_select(old_visual_select);
    set_visual_active(old_visual_active);
    curwin.set(old_curwin.raw());
    curbuf.set(old_curbuf.raw());
}
