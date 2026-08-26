//! Moving a window within the layout -- exchange, rotate, and move to an
//! edge.
//!
//! [`exchange`] swaps two windows in place (CTRL-W x), [`rotate`] cycles a row
//! or column of them (CTRL-W r / CTRL-W R), [`win_splitmove`] takes a window
//! out of the tree and re-inserts it somewhere else (CTRL-W H/J/K/L and
//! `nvim_win_set_config`), and [`win_move_after`] reorders two windows in the
//! same frame.  [`make_windows`] answers how many windows will fit, and opens
//! that many, and [`max_wincount`] is the same question for one frame.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::drawscreen::UPD_NOT_VALID;
use crate::ex_getln::text_or_buf_locked;
use crate::getchar::beep_flush;
use crate::main::{curbuf, e_floatexchange, lastwin, p_ea, p_wh, p_wiw, p_wmh, p_wmw};
use crate::message::{emsg, iemsg};
use crate::normal::{reset_VIsual_and_resel, visual_active};
use crate::types::{FAIL, OK, OptInt, frame_T, win_T};
use crate::winlayer::{Frame, Win, frames};

pub unsafe fn make_windows(count: c_int, vertical: bool) -> c_int {
    let cur = cur_win();
    // Each window needs at least 'winminheight' lines and a status line, and
    // the current window wants 'winheight'.
    let maxcount = if vertical {
        ((cur.w_width + cur.w_vsep_width) as OptInt - (p_wiw.get() - p_wmw.get())) as c_int
            / (p_wmw.get() as c_int + 1)
    } else {
        ((cur.w_height + cur.w_hsep_height + cur.w_status_height) as OptInt
            - (p_wh.get() - p_wmh.get())) as c_int
            / (p_wmh.get() as c_int + STATUS_HEIGHT as c_int + global_winbar_rows())
    }
    .max(2);
    let count = count.min(maxcount);

    // add status line now, otherwise first window will be too big
    if count > 1 {
        last_status(true);
    }

    // Don't execute autocommands while creating the windows: `curwin` and
    // `curbuf` are not set up yet.
    // SAFETY: matched by the `unblock_autocmds` below.
    unsafe { block_autocmds() };
    let mut todo = count - 1;
    while todo > 0 {
        let cur = cur_win();
        let (size, flags) = if vertical {
            let width = cur.w_width;
            (
                width - (width - todo) / (todo + 1) - 1,
                WSP_VERT as c_int | WSP_ABOVE as c_int,
            )
        } else {
            let height = cur.w_height;
            let status = STATUS_HEIGHT as c_int;
            (
                height - (height - todo * status) / (todo + 1) - status,
                WSP_ABOVE as c_int,
            )
        };
        if win_split(size, flags) == FAIL {
            break;
        }
        todo -= 1;
    }
    // SAFETY: matches the `block_autocmds` above.
    unsafe { unblock_autocmds() };
    // return actual number of windows
    count - todo
}

/// Exchange the current window with the `prenum`th window of its row or
/// column, or with the next one when `prenum` is zero.
pub(crate) fn exchange(prenum: c_int) {
    let mut cur = cur_win();
    if cur.w_floating {
        // SAFETY: a static message.
        unsafe { emsg(&raw const e_floatexchange as *const c_char) };
        return;
    }
    // SAFETY: beeps; reads no argument of ours.
    if is_only_window(cur, None) || unsafe { text_or_buf_locked() } {
        // SAFETY: as above.
        beep_flush();
        return;
    }

    let frame = cur.frame();
    let parent = frame.parent().expect("not the only window");
    let frp = if prenum != 0 {
        let mut prenum = prenum;
        let mut frp = parent.child();
        while let Some(cur) = frp {
            prenum -= 1;
            if prenum <= 0 {
                break;
            }
            frp = cur.next();
        }
        frp
    } else {
        frame.next().or_else(|| frame.prev())
    };
    let Some(frp) = frp else {
        return;
    };
    let Some(mut wp) = frp.win().filter(|w| *w != cur) else {
        return;
    };

    // Remove `curwin` from the list, and put it in `wp`'s place; then do the
    // same the other way round.
    // SAFETY: a live window's `w_prev` is a live window or null.
    let wp2 = unsafe { Win::from_raw(cur.w_prev) };
    let frp2 = frame.prev();
    if wp.w_prev != cur.raw() {
        remove(cur, None);
        frame_remove(frame);
        // SAFETY: as above.
        append(unsafe { Win::from_raw(wp.w_prev) }, cur, None);
        frame_insert(frp, frame);
    }
    if Some(wp) != wp2 {
        remove(wp, None);
        frame_remove(wp.frame());
        append(wp2, wp, None);
        match frp2 {
            None => {
                let first = wp
                    .frame()
                    .parent()
                    .and_then(Frame::child)
                    .expect("a linked frame has a parent with children");
                frame_insert(first, wp.frame());
            }
            Some(frp2) => frame_append(frp2, wp.frame()),
        }
    }

    // Exchange the chrome, which belongs to the position and not to the
    // window.
    core::mem::swap(&mut cur.w_status_height, &mut wp.w_status_height);
    core::mem::swap(&mut cur.w_vsep_width, &mut wp.w_vsep_width);
    core::mem::swap(&mut cur.w_hsep_height, &mut wp.w_hsep_height);
    frame_fix_height(cur);
    frame_fix_height(wp);
    frame_fix_width(cur);
    frame_fix_width(wp);
    comp_positions();

    if wp.w_buffer != curbuf.get() {
        reset_VIsual_and_resel();
    } else if visual_active() {
        wp.w_cursor = cur.w_cursor;
    }
    // SAFETY: a live window; nothing derived from it is read afterwards.
    unsafe { win_enter(wp.raw(), true) };
    cur_win().redraw_later(UPD_NOT_VALID);
    wp.redraw_later(UPD_NOT_VALID);
}

/// Rotate the windows in the current row or column `count` places, upwards or
/// downwards.
pub(crate) fn rotate(upwards: bool, count: c_int) {
    if cur_win().w_floating {
        // SAFETY: a static message.
        unsafe { emsg(&raw const e_floatexchange as *const c_char) };
        return;
    }
    if count <= 0 || is_only_window(cur_win(), None) {
        // SAFETY: beeps.
        beep_flush();
        return;
    }
    let parent = cur_win().frame().parent().expect("not the only window");
    // Check that all frames in this row or column are leaves.
    if parent.children().any(|frp| frp.win().is_none()) {
        err(c"E443: Cannot rotate when another window is split".as_ptr());
        return;
    }

    let mut wp1 = None;
    let mut wp2 = None;
    for _ in 0..count {
        if upwards {
            // First window becomes last window.
            let frp = parent.child().expect("frp != NULL");
            let w1 = frp.win().expect("a leaf frame holds a window");
            remove(w1, None);
            frame_remove(frp);
            debug_assert!(parent.child().is_some(), "frp->fr_parent->fr_child");
            // Find the last frame and append the removed window after it.
            let last = frames(Some(frp)).last().expect("at least one");
            append(last.win(), w1, None);
            frame_append(last, w1.frame());
            wp1 = Some(w1);
            wp2 = last.win();
        } else {
            // Last window becomes first window.
            let frp = frames(Some(cur_win().frame()))
                .last()
                .expect("at least one");
            let w1 = frp.win().expect("a leaf frame holds a window");
            // SAFETY: a live window's `w_prev` is a live window or null.
            wp2 = unsafe { Win::from_raw(w1.w_prev) };
            remove(w1, None);
            frame_remove(frp);
            let first = parent.child().expect("frp->fr_parent->fr_child");
            let head = first.win().expect("a leaf frame holds a window");
            // SAFETY: as above.
            append(unsafe { Win::from_raw(head.w_prev) }, w1, None);
            frame_insert(first, frp);
            wp1 = Some(w1);
        }
        let (Some(mut w1), Some(mut w2)) = (wp1, wp2) else {
            continue;
        };
        // Exchange the chrome, which belongs to the position.
        core::mem::swap(&mut w2.w_status_height, &mut w1.w_status_height);
        core::mem::swap(&mut w2.w_hsep_height, &mut w1.w_hsep_height);
        frame_fix_height(w1);
        frame_fix_height(w2);
        core::mem::swap(&mut w2.w_vsep_width, &mut w1.w_vsep_width);
        frame_fix_width(w1);
        frame_fix_width(w2);
        comp_positions();
    }
    if let Some(mut w1) = wp1 {
        w1.w_pos_changed = true;
    }
    if let Some(mut w2) = wp2 {
        w2.w_pos_changed = true;
    }
    redraw_all(UPD_NOT_VALID);
}

pub unsafe fn win_splitmove(wp: *mut win_T, size: c_int, flags: c_int) -> c_int {
    // SAFETY: the caller's promise -- a live window.
    splitmove(unsafe { Win::new(wp) }, size, flags)
}

/// Take `wp` out of the layout and put it back in as a split given by `flags`,
/// from `win_splitmove()`. Restores the old layout on failure.
pub(crate) fn splitmove(wp: Win, size: c_int, flags: c_int) -> c_int {
    let height = wp.w_height;
    if is_only_window(wp, None) {
        return OK;
    }
    // SAFETY: a live window.
    if is_autocmd_window(Some(wp)) || unsafe { check_split_disallowed(wp.raw()) } == FAIL {
        return FAIL;
    }

    let mut dir = 0;
    let mut unflat_altfr = ptr::null_mut::<frame_T>();
    if wp.w_floating {
        remove(wp, None);
    } else {
        // Remove the window and frame from the tree of frames, but leave the
        // altframe unflattened so a failure can be undone.
        let (d, alt) = (&raw mut dir, &raw mut unflat_altfr);
        // SAFETY: a live window, and two out-parameters we own.
        unsafe { winframe_remove(wp.raw(), d, ptr::null_mut(), alt) };
        debug_assert!(!unflat_altfr.is_null(), "unflat_altfr != NULL");
        remove(wp, None);
        last_status(false);
        comp_positions();
    }

    // SAFETY: a live window and the unflattened frame from above.
    if unsafe { win_split_ins(size, flags, wp.raw(), dir, unflat_altfr) }.is_null() {
        // Restore the window to its original position.
        if !wp.w_floating {
            debug_assert!(!unflat_altfr.is_null(), "unflat_altfr != NULL");
            // SAFETY: as above.
            unsafe { winframe_restore(wp.raw(), dir, unflat_altfr) };
        }
        // SAFETY: a live window's `w_prev` is a live window or null.
        append(unsafe { Win::from_raw(wp.w_prev) }, wp, None);
        return FAIL;
    }

    // Keep the window's height when it was moved horizontally.
    // SAFETY: only compares the pointer against the window list.
    if size == 0
        && flags & WSP_VERT as c_int == 0
        && unsafe { win_valid(wp.raw()) }
        && !wp.w_floating
    {
        setheight_win(height, wp);
        if p_ea.get() != 0 {
            let cur = cur_win();
            equal(Some(cur), cur == wp, 'v' as c_int);
        }
    }
    OK
}

pub unsafe fn win_move_after(win1: *mut win_T, win2: *mut win_T) {
    // SAFETY: the caller's promise -- two live windows.
    unsafe { move_after(Win::new(win1), Win::new(win2)) };
}

/// Move window `win1` to just after window `win2`, both in the same frame.
fn move_after(win1: Win, win2: Win) {
    let (mut win1, mut win2) = (win1, win2);
    // Can't move the first window.
    if win1 == win2 {
        return;
    }
    if win2.w_next != win1.raw() {
        if win1.frame().fr_parent != win2.frame().fr_parent {
            // SAFETY: a static message.
            unsafe { iemsg(c"INTERNAL: trying to move a window into another frame".as_ptr()) };
            return;
        }
        // The last window has no separator or status line: exchange the chrome
        // with whichever window is about to become last.
        if win1.raw() == lastwin.get() {
            // SAFETY: `win1` is not first, so `w_prev` is a live window.
            let mut prev = unsafe { Win::new(win1.w_prev) };
            core::mem::swap(&mut prev.w_status_height, &mut win1.w_status_height);
            core::mem::swap(&mut prev.w_hsep_height, &mut win1.w_hsep_height);
            if prev.w_vsep_width == 1 {
                // The last window has no separator: give it to `win1`.
                prev.w_vsep_width = 0;
                prev.frame().fr_width -= 1;
                win1.w_vsep_width = 1;
                win1.frame().fr_width += 1;
            }
        } else if win2.raw() == lastwin.get() {
            core::mem::swap(&mut win1.w_status_height, &mut win2.w_status_height);
            core::mem::swap(&mut win1.w_hsep_height, &mut win2.w_hsep_height);
            if win1.w_vsep_width == 1 {
                win2.w_vsep_width = 1;
                win2.frame().fr_width += 1;
                win1.w_vsep_width = 0;
                win1.frame().fr_width -= 1;
            }
        }
        remove(win1, None);
        frame_remove(win1.frame());
        append(Some(win2), win1, None);
        frame_append(win2.frame(), win1.frame());
        comp_positions(); // recompute window positions
        cur_win().redraw_later(UPD_NOT_VALID);
    }
    win1.w_pos_changed = true;
    win2.w_pos_changed = true;
    // SAFETY: a live window; nothing derived from it is read afterwards.
    unsafe { win_enter(win1.raw(), false) };
}

/// How many windows would fit in `height` rows of frame `fr`: each costs
/// `'winminheight'` plus a status line,
/// plus its window bar where there is one.
pub(crate) fn max_wincount(fr: Frame, height: c_int) -> c_int {
    let per_win = p_wmh.get() as c_int + STATUS_HEIGHT as c_int;
    if fr.fr_layout as c_int != FR_COL {
        return height / (per_win + frame2window(fr).w_winbar_height);
    }
    if global_winbar_rows() != 0 {
        // If a window bar is globally enabled, no need to check each window.
        return height / (per_win + 1);
    }

    // First, try to fit all child frames of "fr" into "height".
    let mut height = height;
    let mut total = 0;
    for frp in fr.children() {
        let cost = per_win + frame2window(frp).w_winbar_height;
        if (height as OptInt) < cost as OptInt {
            break;
        }
        height -= cost;
        total += 1;
    }
    // With room left over, use the default window-bar height (which is zero)
    // for however many more would fit.
    total + height / per_win
}
