//! Moving a window within the layout -- exchange, rotate, and move to an
//! edge.
//!
//! [`win_exchange`] swaps two windows in place (CTRL-W x), [`win_rotate`]
//! cycles a row or column of them (CTRL-W r / CTRL-W R), [`win_splitmove`]
//! takes a window out of the tree and re-inserts it somewhere else (CTRL-W
//! H/J/K/L and `nvim_win_set_config`), and [`win_move_after`] reorders two
//! windows in the same frame.  [`make_windows`] answers how many windows will
//! fit, and opens that many.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later, redraw_later};
use crate::src::nvim::ex_getln::text_or_buf_locked;
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::main::{
    VIsual_active, curbuf, curwin, e_floatexchange, lastwin, p_ea, p_wh, p_wiw, p_wmh, p_wmw,
};
use crate::src::nvim::message::{emsg, iemsg};
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::{OptInt, frame_T, tabpage_T, win_T};

pub unsafe extern "C" fn make_windows(
    mut count: ::core::ffi::c_int,
    mut vertical: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut maxcount: ::core::ffi::c_int = 0;
        if vertical {
            maxcount = (((*curwin.get()).w_width + (*curwin.get()).w_vsep_width) as OptInt
                - (p_wiw.get() - p_wmw.get())) as ::core::ffi::c_int
                / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
        } else {
            maxcount = (((*curwin.get()).w_height
                + (*curwin.get()).w_hsep_height
                + (*curwin.get()).w_status_height) as OptInt
                - (p_wh.get() - p_wmh.get())) as ::core::ffi::c_int
                / (p_wmh.get() as ::core::ffi::c_int
                    + STATUS_HEIGHT as ::core::ffi::c_int
                    + global_winbar_height());
        }
        maxcount = if maxcount > 2 as ::core::ffi::c_int {
            maxcount
        } else {
            2 as ::core::ffi::c_int
        };
        count = if count < maxcount { count } else { maxcount };
        if count > 1 as ::core::ffi::c_int {
            last_status(true_0 != 0);
        }
        block_autocmds();
        let mut todo: ::core::ffi::c_int = 0;
        todo = count - 1 as ::core::ffi::c_int;
        while todo > 0 as ::core::ffi::c_int {
            if vertical {
                if win_split(
                    (*curwin.get()).w_width
                        - ((*curwin.get()).w_width - todo) / (todo + 1 as ::core::ffi::c_int)
                        - 1 as ::core::ffi::c_int,
                    WSP_VERT as ::core::ffi::c_int | WSP_ABOVE as ::core::ffi::c_int,
                ) == FAIL
                {
                    break;
                }
            } else if win_split(
                (*curwin.get()).w_height
                    - ((*curwin.get()).w_height - todo * STATUS_HEIGHT as ::core::ffi::c_int)
                        / (todo + 1 as ::core::ffi::c_int)
                    - STATUS_HEIGHT as ::core::ffi::c_int,
                WSP_ABOVE as ::core::ffi::c_int,
            ) == FAIL
            {
                break;
            }
            todo -= 1;
        }
        unblock_autocmds();
        return count - todo;
    }
}

pub(crate) unsafe extern "C" fn win_exchange(mut Prenum: ::core::ffi::c_int) {
    unsafe {
        if (*curwin.get()).w_floating {
            emsg(&raw const e_floatexchange as *const ::core::ffi::c_char);
            return;
        }
        if one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) {
            beep_flush();
            return;
        }
        if text_or_buf_locked() {
            beep_flush();
            return;
        }
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        if Prenum != 0 {
            frp = (*(*(*curwin.get()).w_frame).fr_parent).fr_child;
            while !frp.is_null() && {
                Prenum -= 1;
                Prenum > 0 as ::core::ffi::c_int
            } {
                frp = (*frp).fr_next;
            }
        } else if !(*(*curwin.get()).w_frame).fr_next.is_null() {
            frp = (*(*curwin.get()).w_frame).fr_next;
        } else {
            frp = (*(*curwin.get()).w_frame).fr_prev;
        }
        if frp.is_null() || (*frp).fr_win.is_null() || (*frp).fr_win == curwin.get() {
            return;
        }
        let mut wp: *mut win_T = (*frp).fr_win;
        let mut wp2: *mut win_T = (*curwin.get()).w_prev;
        let mut frp2: *mut frame_T = (*(*curwin.get()).w_frame).fr_prev;
        if (*wp).w_prev != curwin.get() {
            win_remove(curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
            frame_remove((*curwin.get()).w_frame);
            win_append(
                (*wp).w_prev,
                curwin.get(),
                ::core::ptr::null_mut::<tabpage_T>(),
            );
            frame_insert(frp, (*curwin.get()).w_frame);
        }
        if wp != wp2 {
            win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
            frame_remove((*wp).w_frame);
            win_append(wp2, wp, ::core::ptr::null_mut::<tabpage_T>());
            if frp2.is_null() {
                frame_insert((*(*(*wp).w_frame).fr_parent).fr_child, (*wp).w_frame);
            } else {
                frame_append(frp2, (*wp).w_frame);
            }
        }
        let mut temp: ::core::ffi::c_int = (*curwin.get()).w_status_height;
        (*curwin.get()).w_status_height = (*wp).w_status_height;
        (*wp).w_status_height = temp;
        temp = (*curwin.get()).w_vsep_width;
        (*curwin.get()).w_vsep_width = (*wp).w_vsep_width;
        (*wp).w_vsep_width = temp;
        temp = (*curwin.get()).w_hsep_height;
        (*curwin.get()).w_hsep_height = (*wp).w_hsep_height;
        (*wp).w_hsep_height = temp;
        frame_fix_height(curwin.get());
        frame_fix_height(wp);
        frame_fix_width(curwin.get());
        frame_fix_width(wp);
        win_comp_pos();
        if (*wp).w_buffer != curbuf.get() {
            reset_VIsual_and_resel();
        } else if VIsual_active.get() {
            (*wp).w_cursor = (*curwin.get()).w_cursor;
        }
        win_enter(wp, true_0 != 0);
        redraw_later(curwin.get(), UPD_NOT_VALID);
        redraw_later(wp, UPD_NOT_VALID);
    }
}

pub(crate) unsafe extern "C" fn win_rotate(mut upwards: bool, mut count: ::core::ffi::c_int) {
    unsafe {
        if (*curwin.get()).w_floating {
            emsg(&raw const e_floatexchange as *const ::core::ffi::c_char);
            return;
        }
        if count <= 0 as ::core::ffi::c_int
            || one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int
                != 0
        {
            beep_flush();
            return;
        }
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp = (*(*(*curwin.get()).w_frame).fr_parent).fr_child;
        while !frp.is_null() {
            if (*frp).fr_win.is_null() {
                emsg(gettext(
                    c"E443: Cannot rotate when another window is split".as_ptr(),
                ));
                return;
            }
            frp = (*frp).fr_next;
        }
        let mut wp1: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wp2: *mut win_T = ::core::ptr::null_mut::<win_T>();
        loop {
            let c2rust_fresh0 = count;
            count = count - 1;
            if c2rust_fresh0 == 0 {
                break;
            }
            if upwards {
                frp = (*(*(*curwin.get()).w_frame).fr_parent).fr_child;
                debug_assert!(!frp.is_null(), "frp != NULL");
                wp1 = (*frp).fr_win;
                win_remove(wp1, ::core::ptr::null_mut::<tabpage_T>());
                frame_remove(frp);
                debug_assert!(
                    !(*(*frp).fr_parent).fr_child.is_null(),
                    "frp->fr_parent->fr_child"
                );
                while !(*frp).fr_next.is_null() {
                    frp = (*frp).fr_next;
                }
                win_append((*frp).fr_win, wp1, ::core::ptr::null_mut::<tabpage_T>());
                frame_append(frp, (*wp1).w_frame);
                wp2 = (*frp).fr_win;
            } else {
                frp = (*curwin.get()).w_frame;
                while !(*frp).fr_next.is_null() {
                    frp = (*frp).fr_next;
                }
                wp1 = (*frp).fr_win;
                wp2 = (*wp1).w_prev;
                win_remove(wp1, ::core::ptr::null_mut::<tabpage_T>());
                frame_remove(frp);
                debug_assert!(
                    !(*(*frp).fr_parent).fr_child.is_null(),
                    "frp->fr_parent->fr_child"
                );
                win_append(
                    (*(*(*(*frp).fr_parent).fr_child).fr_win).w_prev,
                    wp1,
                    ::core::ptr::null_mut::<tabpage_T>(),
                );
                frame_insert((*(*frp).fr_parent).fr_child, frp);
            }
            let mut n: ::core::ffi::c_int = (*wp2).w_status_height;
            (*wp2).w_status_height = (*wp1).w_status_height;
            (*wp1).w_status_height = n;
            n = (*wp2).w_hsep_height;
            (*wp2).w_hsep_height = (*wp1).w_hsep_height;
            (*wp1).w_hsep_height = n;
            frame_fix_height(wp1);
            frame_fix_height(wp2);
            n = (*wp2).w_vsep_width;
            (*wp2).w_vsep_width = (*wp1).w_vsep_width;
            (*wp1).w_vsep_width = n;
            frame_fix_width(wp1);
            frame_fix_width(wp2);
            win_comp_pos();
        }
        (*wp1).w_pos_changed = true_0 != 0;
        (*wp2).w_pos_changed = true_0 != 0;
        redraw_all_later(UPD_NOT_VALID);
    }
}

pub unsafe extern "C" fn win_splitmove(
    mut wp: *mut win_T,
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut height: ::core::ffi::c_int = (*wp).w_height;
        if one_window(wp, ::core::ptr::null_mut::<tabpage_T>()) {
            return OK;
        }
        if is_aucmd_win(wp) as ::core::ffi::c_int != 0 || check_split_disallowed(wp) == FAIL {
            return FAIL;
        }
        let mut unflat_altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        if (*wp).w_floating {
            win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
        } else {
            winframe_remove(
                wp,
                &raw mut dir,
                ::core::ptr::null_mut::<tabpage_T>(),
                &raw mut unflat_altfr,
            );
            debug_assert!(!unflat_altfr.is_null(), "unflat_altfr != NULL");
            win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
            last_status(false_0 != 0);
            win_comp_pos();
        }
        if win_split_ins(size, flags, wp, dir, unflat_altfr).is_null() {
            if !(*wp).w_floating {
                debug_assert!(!unflat_altfr.is_null(), "unflat_altfr != NULL");
                winframe_restore(wp, dir, unflat_altfr);
            }
            win_append((*wp).w_prev, wp, ::core::ptr::null_mut::<tabpage_T>());
            return FAIL;
        }
        if size == 0 as ::core::ffi::c_int
            && flags & WSP_VERT as ::core::ffi::c_int == 0
            && win_valid(wp) as ::core::ffi::c_int != 0
            && !(*wp).w_floating
        {
            win_setheight_win(height, wp);
            if p_ea.get() != 0 {
                win_equal(curwin.get(), curwin.get() == wp, 'v' as ::core::ffi::c_int);
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn win_move_after(mut win1: *mut win_T, mut win2: *mut win_T) {
    unsafe {
        if win1 == win2 {
            return;
        }
        if (*win2).w_next != win1 {
            if (*(*win1).w_frame).fr_parent != (*(*win2).w_frame).fr_parent {
                iemsg(c"INTERNAL: trying to move a window into another frame".as_ptr());
                return;
            }
            if win1 == lastwin.get() {
                let mut height: ::core::ffi::c_int = (*(*win1).w_prev).w_status_height;
                (*(*win1).w_prev).w_status_height = (*win1).w_status_height;
                (*win1).w_status_height = height;
                height = (*(*win1).w_prev).w_hsep_height;
                (*(*win1).w_prev).w_hsep_height = (*win1).w_hsep_height;
                (*win1).w_hsep_height = height;
                if (*(*win1).w_prev).w_vsep_width == 1 as ::core::ffi::c_int {
                    (*(*win1).w_prev).w_vsep_width = 0 as ::core::ffi::c_int;
                    (*(*(*win1).w_prev).w_frame).fr_width -= 1 as ::core::ffi::c_int;
                    (*win1).w_vsep_width = 1 as ::core::ffi::c_int;
                    (*(*win1).w_frame).fr_width += 1 as ::core::ffi::c_int;
                }
            } else if win2 == lastwin.get() {
                let mut height_0: ::core::ffi::c_int = (*win1).w_status_height;
                (*win1).w_status_height = (*win2).w_status_height;
                (*win2).w_status_height = height_0;
                height_0 = (*win1).w_hsep_height;
                (*win1).w_hsep_height = (*win2).w_hsep_height;
                (*win2).w_hsep_height = height_0;
                if (*win1).w_vsep_width == 1 as ::core::ffi::c_int {
                    (*win2).w_vsep_width = 1 as ::core::ffi::c_int;
                    (*(*win2).w_frame).fr_width += 1 as ::core::ffi::c_int;
                    (*win1).w_vsep_width = 0 as ::core::ffi::c_int;
                    (*(*win1).w_frame).fr_width -= 1 as ::core::ffi::c_int;
                }
            }
            win_remove(win1, ::core::ptr::null_mut::<tabpage_T>());
            frame_remove((*win1).w_frame);
            win_append(win2, win1, ::core::ptr::null_mut::<tabpage_T>());
            frame_append((*win2).w_frame, (*win1).w_frame);
            win_comp_pos();
            redraw_later(curwin.get(), UPD_NOT_VALID);
        }
        (*win1).w_pos_changed = true_0 != 0;
        (*win2).w_pos_changed = true_0 != 0;
        win_enter(win1, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn get_maximum_wincount(
    mut fr: *mut frame_T,
    mut height: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*fr).fr_layout as ::core::ffi::c_int != FR_COL {
            return height
                / (p_wmh.get() as ::core::ffi::c_int
                    + STATUS_HEIGHT as ::core::ffi::c_int
                    + (*frame2win(fr)).w_winbar_height);
        } else if global_winbar_height() != 0 {
            return height
                / (p_wmh.get() as ::core::ffi::c_int
                    + STATUS_HEIGHT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int);
        }
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        let mut total_wincount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        frp = (*fr).fr_child;
        while !frp.is_null() {
            let mut wp: *mut win_T = frame2win(frp);
            if (height as OptInt)
                < p_wmh.get()
                    + STATUS_HEIGHT as ::core::ffi::c_int as OptInt
                    + (*wp).w_winbar_height as OptInt
            {
                break;
            }
            height -= p_wmh.get() as ::core::ffi::c_int
                + STATUS_HEIGHT as ::core::ffi::c_int
                + (*wp).w_winbar_height;
            total_wincount += 1 as ::core::ffi::c_int;
            frp = (*frp).fr_next;
        }
        total_wincount +=
            height / (p_wmh.get() as ::core::ffi::c_int + STATUS_HEIGHT as ::core::ffi::c_int);
        return total_wincount;
    }
}
