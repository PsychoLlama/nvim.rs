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
#![allow(unsafe_code)]

use crate::winlayer::Buf;
use core::ffi::c_int;

use super::*;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::drawscreen::UPD_NOT_VALID;
use crate::ex_getln::text_or_buf_locked;
use crate::getchar::beep_flush;
use crate::message::e_floatexchange;
use crate::message::{emsg, iemsg};
use crate::normal::{reset_visual_and_resel, visual_active};
use crate::option::vars::{p_ea, p_wh, p_wiw, p_wmh, p_wmw};
use crate::types::{FAIL, Failed, OptInt};
use crate::winlayer::graph::lastwin;
use crate::winlayer::{FrameId, FrameRef, Win, frames};

pub unsafe fn make_windows(count: c_int, vertical: bool) -> c_int {
    let cur = Win::current();
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
        let cur = Win::current();
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
        if win_split(size, flags).is_err() {
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
    let mut cur = Win::current();
    if cur.w_floating {
        emsg(e_floatexchange);
        return;
    }
    if is_only_window(cur, None) || text_or_buf_locked() {
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
    let wp2 = cur.prev();
    let frp2 = frame.prev();
    if wp.w_prev != Some(cur.id()) {
        win_remove(cur, None);
        frame_remove(frame);
        win_append(wp.prev(), cur, None);
        frame_insert(frp, frame);
    }
    if Some(wp) != wp2 {
        win_remove(wp, None);
        frame_remove(wp.frame());
        win_append(wp2, wp, None);
        match frp2 {
            None => {
                let first = wp
                    .frame()
                    .parent()
                    .and_then(FrameRef::child)
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

    if wp.w_buffer != Buf::current_raw() {
        reset_visual_and_resel();
    } else if visual_active() {
        wp.w_cursor = cur.w_cursor;
    }
    win_enter(wp, true);
    Win::current().redraw_later(UPD_NOT_VALID);
    wp.redraw_later(UPD_NOT_VALID);
}

/// Rotate the windows in the current row or column `count` places, upwards or
/// downwards.
pub(crate) fn rotate(upwards: bool, count: c_int) {
    if Win::current().w_floating {
        emsg(e_floatexchange);
        return;
    }
    if count <= 0 || is_only_window(Win::current(), None) {
        // SAFETY: beeps.
        beep_flush();
        return;
    }
    let parent = Win::current()
        .frame()
        .parent()
        .expect("not the only window");
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
            win_remove(w1, None);
            frame_remove(frp);
            debug_assert!(parent.child().is_some(), "frp->fr_parent->fr_child");
            // Find the last frame and append the removed window after it.
            let last = frames(Some(frp)).last().expect("at least one");
            win_append(last.win(), w1, None);
            frame_append(last, w1.frame());
            wp1 = Some(w1);
            wp2 = last.win();
        } else {
            // Last window becomes first window.
            let frp = frames(Some(Win::current().frame()))
                .last()
                .expect("at least one");
            let w1 = frp.win().expect("a leaf frame holds a window");
            wp2 = w1.prev();
            win_remove(w1, None);
            frame_remove(frp);
            let first = parent.child().expect("frp->fr_parent->fr_child");
            let head = first.win().expect("a leaf frame holds a window");
            win_append(head.prev(), w1, None);
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

pub fn win_splitmove(window: Win, size: c_int, flags: c_int) -> Result<(), Failed> {
    splitmove(window, size, flags)
}

/// Take `window` out of the layout and put it back in as a split given by `flags`,
/// from `win_splitmove()`. Restores the old layout on failure.
pub(crate) fn splitmove(window: Win, size: c_int, flags: c_int) -> Result<(), Failed> {
    let height = window.w_height;
    if is_only_window(window, None) {
        return Ok(());
    }
    // SAFETY: a live window.
    if is_autocmd_window(Some(window)) || unsafe { check_split_disallowed(window) } == FAIL {
        return Err(Failed);
    }

    let mut dir = 0;
    let mut unflat_altfr = None;
    if window.w_floating {
        win_remove(window, None);
    } else {
        // Remove the window and frame from the tree of frames, but leave the
        // altframe unflattened so a failure can be undone.
        let removed = winframe_remove(window, None, true);
        (dir, unflat_altfr) = (removed.dir, removed.unflat);
        debug_assert!(unflat_altfr.is_some(), "unflat_altfr != NULL");
        win_remove(window, None);
        last_status(false);
        comp_positions();
    }

    // The unflattened frame from above, resolved at each use rather than
    // held: `win_split_ins` flattens it on the way through, so the frame the
    // failure path wants back may be gone by the time it looks.
    let unflat = unflat_altfr.and_then(FrameId::get);
    if win_split_ins(size, flags, Some(window), dir, unflat).is_none() {
        // Restore the window to its original position.
        if !window.w_floating
            && let Some(unflat) = unflat_altfr.and_then(FrameId::get)
        {
            winframe_restore(window, dir, unflat);
        }
        win_append(window.prev(), window, None);
        return Err(Failed);
    }

    // Keep the window's height when it was moved horizontally. The identity
    // was taken before `win_split_ins` fired its autocommands.
    if size == 0 && flags & WSP_VERT as c_int == 0 && win_valid(window.id()) && !window.w_floating {
        setheight_win(height, window);
        if p_ea.get() != 0 {
            let cur = Win::current();
            equal(Some(cur), cur == window, 'v' as c_int);
        }
    }
    Ok(())
}

pub fn win_move_after(win1: Win, win2: Win) {
    move_after(win1, win2);
}

/// Move window `win1` to just after window `win2`, both in the same frame.
fn move_after(win1: Win, win2: Win) {
    let (mut win1, mut win2) = (win1, win2);
    // Can't move the first window.
    if win1 == win2 {
        return;
    }
    if win2.w_next != Some(win1.id()) {
        if win1.frame().fr_parent != win2.frame().fr_parent {
            iemsg(c"INTERNAL: trying to move a window into another frame");
            return;
        }
        // The last window has no separator or status line: exchange the chrome
        // with whichever window is about to become last.
        if lastwin.get() == Some(win1.id()) {
            let mut prev = win1.prev().expect("`win1` is not the first window");
            core::mem::swap(&mut prev.w_status_height, &mut win1.w_status_height);
            core::mem::swap(&mut prev.w_hsep_height, &mut win1.w_hsep_height);
            if prev.w_vsep_width == 1 {
                // The last window has no separator: give it to `win1`.
                prev.w_vsep_width = 0;
                prev.frame().fr_width -= 1;
                win1.w_vsep_width = 1;
                win1.frame().fr_width += 1;
            }
        } else if lastwin.get() == Some(win2.id()) {
            core::mem::swap(&mut win1.w_status_height, &mut win2.w_status_height);
            core::mem::swap(&mut win1.w_hsep_height, &mut win2.w_hsep_height);
            if win1.w_vsep_width == 1 {
                win2.w_vsep_width = 1;
                win2.frame().fr_width += 1;
                win1.w_vsep_width = 0;
                win1.frame().fr_width -= 1;
            }
        }
        win_remove(win1, None);
        frame_remove(win1.frame());
        win_append(Some(win2), win1, None);
        frame_append(win2.frame(), win1.frame());
        comp_positions(); // recompute window positions
        Win::current().redraw_later(UPD_NOT_VALID);
    }
    win1.w_pos_changed = true;
    win2.w_pos_changed = true;
    win_enter(win1, false);
}

/// How many windows would fit in `height` rows of frame `fr`: each costs
/// `'winminheight'` plus a status line,
/// plus its window bar where there is one.
pub(crate) fn max_wincount(fr: FrameRef, height: c_int) -> c_int {
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
