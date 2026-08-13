//! Saving and restoring a layout, revalidating it, and the odds and ends.
//!
//! The `*_snapshot*` family saves the frame tree before a `:diffsplit` or a
//! help window opens and puts it back afterwards, matching the saved shape
//! against the current one ([`check_snapshot_rec`]) before trusting it.
//! [`check_lnums`] and [`reset_lnums`] revalidate every window's cursor and
//! topline against a buffer whose line count changed.  [`frame_check_height`]
//! and [`frame_check_width`] are the tree's internal consistency assertions,
//! [`check_colorcolumn`] parses `'colorcolumn'`, and [`win_ui_flush`] pushes
//! the accumulated position and viewport changes to the UI.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::getdigits_int;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::eval::window::win_has_winnr;
use crate::src::nvim::main::{
    curbuf, curtab, curwin, e_invarg, empty_string_option, first_tabpage, firstwin, lastwin,
    topframe,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc};
use crate::src::nvim::message::msg_ui_flush;
use crate::src::nvim::os::libc::qsort;
use crate::src::nvim::popupmenu::pum_ui_flush;
use crate::src::nvim::pos::equalpos;
use crate::src::nvim::types::{
    Integer, OptInt, frame_T, handle_T, linenr_T, size_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::ui_call_win_hide;

unsafe extern "C" fn check_lnums_both(mut do_curwin: bool, mut nested: bool) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (do_curwin as ::core::ffi::c_int != 0 || wp != curwin.get())
                    && (*wp).w_buffer == curbuf.get()
                {
                    if !nested {
                        (*wp).w_save_cursor.w_cursor_save = (*wp).w_cursor;
                        (*wp).w_save_cursor.w_topline_save = (*wp).w_topline as ::core::ffi::c_int;
                    }
                    let mut need_adjust: bool =
                        (*wp).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count;
                    if need_adjust {
                        (*wp).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
                    }
                    if need_adjust as ::core::ffi::c_int != 0 || !nested {
                        (*wp).w_save_cursor.w_cursor_corr = (*wp).w_cursor;
                    }
                    need_adjust = (*wp).w_topline > (*curbuf.get()).b_ml.ml_line_count;
                    if need_adjust {
                        (*wp).w_topline = (*curbuf.get()).b_ml.ml_line_count;
                    }
                    if need_adjust as ::core::ffi::c_int != 0 || !nested {
                        (*wp).w_save_cursor.w_topline_corr = (*wp).w_topline as ::core::ffi::c_int;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

pub unsafe extern "C" fn check_lnums(mut do_curwin: bool) {
    unsafe {
        check_lnums_both(do_curwin, false_0 != 0);
    }
}

pub unsafe extern "C" fn check_lnums_nested(mut do_curwin: bool) {
    unsafe {
        check_lnums_both(do_curwin, true_0 != 0);
    }
}

pub unsafe extern "C" fn reset_lnums() {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == curbuf.get() {
                    if equalpos((*wp).w_save_cursor.w_cursor_corr, (*wp).w_cursor)
                        as ::core::ffi::c_int
                        != 0
                        && (*wp).w_save_cursor.w_cursor_save.lnum != 0 as linenr_T
                    {
                        (*wp).w_cursor = (*wp).w_save_cursor.w_cursor_save;
                    }
                    if (*wp).w_save_cursor.w_topline_corr as linenr_T == (*wp).w_topline
                        && (*wp).w_save_cursor.w_topline_save != 0 as ::core::ffi::c_int
                    {
                        (*wp).w_topline = (*wp).w_save_cursor.w_topline_save as linenr_T;
                    }
                    if (*wp).w_save_cursor.w_topline_save as linenr_T
                        > (*(*wp).w_buffer).b_ml.ml_line_count
                    {
                        (*wp).w_valid &= !VALID_TOPLINE;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

pub unsafe extern "C" fn make_snapshot(mut idx: ::core::ffi::c_int) {
    unsafe {
        clear_snapshot(curtab.get(), idx);
        make_snapshot_rec(
            topframe.get(),
            (&raw mut (*curtab.get()).tp_snapshot as *mut *mut frame_T).offset(idx as isize),
        );
    }
}

unsafe extern "C" fn make_snapshot_rec(mut fr: *mut frame_T, mut frp: *mut *mut frame_T) {
    unsafe {
        *frp = xcalloc(1 as size_t, ::core::mem::size_of::<frame_T>()) as *mut frame_T;
        (**frp).fr_layout = (*fr).fr_layout;
        (**frp).fr_width = (*fr).fr_width;
        (**frp).fr_height = (*fr).fr_height;
        if !(*fr).fr_next.is_null() {
            make_snapshot_rec((*fr).fr_next, &raw mut (**frp).fr_next);
        }
        if !(*fr).fr_child.is_null() {
            make_snapshot_rec((*fr).fr_child, &raw mut (**frp).fr_child);
        }
        if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF && (*fr).fr_win == curwin.get() {
            (**frp).fr_win = curwin.get();
        }
    }
}

pub(crate) unsafe extern "C" fn clear_snapshot(
    mut tp: *mut tabpage_T,
    mut idx: ::core::ffi::c_int,
) {
    unsafe {
        clear_snapshot_rec((*tp).tp_snapshot[idx as usize] as *mut frame_T);
        (*tp).tp_snapshot[idx as usize] = ::core::ptr::null_mut::<frame_T>();
    }
}

unsafe extern "C" fn clear_snapshot_rec(mut fr: *mut frame_T) {
    unsafe {
        if fr.is_null() {
            return;
        }
        clear_snapshot_rec((*fr).fr_next);
        clear_snapshot_rec((*fr).fr_child);
        xfree(fr as *mut ::core::ffi::c_void);
    }
}

unsafe extern "C" fn get_snapshot_curwin_rec(mut ft: *mut frame_T) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if !(*ft).fr_next.is_null() {
            wp = get_snapshot_curwin_rec((*ft).fr_next);
            if !wp.is_null() {
                return wp;
            }
        }
        if !(*ft).fr_child.is_null() {
            wp = get_snapshot_curwin_rec((*ft).fr_child);
            if !wp.is_null() {
                return wp;
            }
        }
        return (*ft).fr_win;
    }
}

pub(crate) unsafe extern "C" fn get_snapshot_curwin(mut idx: ::core::ffi::c_int) -> *mut win_T {
    unsafe {
        if (*curtab.get()).tp_snapshot[idx as usize].is_null() {
            return ::core::ptr::null_mut::<win_T>();
        }
        return get_snapshot_curwin_rec((*curtab.get()).tp_snapshot[idx as usize] as *mut frame_T);
    }
}

pub unsafe extern "C" fn restore_snapshot(
    mut idx: ::core::ffi::c_int,
    mut close_curwin: ::core::ffi::c_int,
) {
    unsafe {
        if !(*curtab.get()).tp_snapshot[idx as usize].is_null()
            && (*(*curtab.get()).tp_snapshot[idx as usize]).fr_width == (*topframe.get()).fr_width
            && (*(*curtab.get()).tp_snapshot[idx as usize]).fr_height == (*topframe.get()).fr_height
            && check_snapshot_rec(
                (*curtab.get()).tp_snapshot[idx as usize] as *mut frame_T,
                topframe.get(),
            ) == OK
        {
            let mut wp: *mut win_T = restore_snapshot_rec(
                (*curtab.get()).tp_snapshot[idx as usize] as *mut frame_T,
                topframe.get(),
            );
            win_comp_pos();
            if !wp.is_null() && close_curwin != 0 {
                win_goto(wp);
            }
            redraw_all_later(UPD_NOT_VALID);
        }
        clear_snapshot(curtab.get(), idx);
    }
}

unsafe extern "C" fn check_snapshot_rec(
    mut sn: *mut frame_T,
    mut fr: *mut frame_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*sn).fr_layout as ::core::ffi::c_int != (*fr).fr_layout as ::core::ffi::c_int
            || (*sn).fr_next.is_null() as ::core::ffi::c_int
                != (*fr).fr_next.is_null() as ::core::ffi::c_int
            || (*sn).fr_child.is_null() as ::core::ffi::c_int
                != (*fr).fr_child.is_null() as ::core::ffi::c_int
            || !(*sn).fr_next.is_null() && check_snapshot_rec((*sn).fr_next, (*fr).fr_next) == FAIL
            || !(*sn).fr_child.is_null()
                && check_snapshot_rec((*sn).fr_child, (*fr).fr_child) == FAIL
            || !(*sn).fr_win.is_null() && !win_valid((*sn).fr_win)
        {
            return FAIL;
        }
        return OK;
    }
}

unsafe extern "C" fn restore_snapshot_rec(
    mut sn: *mut frame_T,
    mut fr: *mut frame_T,
) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        (*fr).fr_height = (*sn).fr_height;
        (*fr).fr_width = (*sn).fr_width;
        if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
            frame_new_height(
                fr,
                (*fr).fr_height,
                false_0 != 0,
                false_0 != 0,
                false_0 != 0,
            );
            frame_new_width(fr, (*fr).fr_width, false_0 != 0, false_0 != 0);
            wp = (*sn).fr_win;
        }
        if !(*sn).fr_next.is_null() {
            let mut wp2: *mut win_T = restore_snapshot_rec((*sn).fr_next, (*fr).fr_next);
            if !wp2.is_null() {
                wp = wp2;
            }
        }
        if !(*sn).fr_child.is_null() {
            let mut wp2_0: *mut win_T = restore_snapshot_rec((*sn).fr_child, (*fr).fr_child);
            if !wp2_0.is_null() {
                wp = wp2_0;
            }
        }
        return wp;
    }
}

pub(crate) unsafe extern "C" fn frame_check_height(
    mut topfrp: *const frame_T,
    mut height: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if (*topfrp).fr_height != height {
            return false_0 != 0;
        }
        if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
            let mut frp: *const frame_T = ::core::ptr::null::<frame_T>();
            frp = (*topfrp).fr_child;
            while !frp.is_null() {
                if (*frp).fr_height != height {
                    return false_0 != 0;
                }
                frp = (*frp).fr_next;
            }
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn frame_check_width(
    mut topfrp: *const frame_T,
    mut width: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if (*topfrp).fr_width != width {
            return false_0 != 0;
        }
        if (*topfrp).fr_layout as ::core::ffi::c_int == FR_COL {
            let mut frp: *const frame_T = ::core::ptr::null::<frame_T>();
            frp = (*topfrp).fr_child;
            while !frp.is_null() {
                if (*frp).fr_width != width {
                    return false_0 != 0;
                }
                frp = (*frp).fr_next;
            }
        }
        return true_0 != 0;
    }
}

unsafe extern "C" fn int_cmp(
    mut pa: *const ::core::ffi::c_void,
    mut pb: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let a: ::core::ffi::c_int = *(pa as *const ::core::ffi::c_int);
        let b: ::core::ffi::c_int = *(pb as *const ::core::ffi::c_int);
        return if a == b {
            0 as ::core::ffi::c_int
        } else if a < b {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn check_colorcolumn(
    mut cc: *mut ::core::ffi::c_char,
    mut wp: *mut win_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        if !wp.is_null() && (*wp).w_buffer.is_null() {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        let mut s: *mut ::core::ffi::c_char = empty_string_option.ptr() as *mut ::core::ffi::c_char;
        if !cc.is_null() {
            s = cc;
        } else if !wp.is_null() {
            s = (*wp).w_onebuf_opt.wo_cc;
        }
        let mut tw: OptInt = 0;
        if !wp.is_null() {
            tw = (*(*wp).w_buffer).b_p_tw;
        } else {
            tw = 0 as OptInt;
        }
        let mut count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        let mut color_cols: [::core::ffi::c_int; 256] = [0; 256];
        while *s as ::core::ffi::c_int != NUL && count < 255 as ::core::ffi::c_uint {
            let mut col: ::core::ffi::c_int = 0;
            '_skip: {
                if *s as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                {
                    col = if *s as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        -1 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                    s = s.offset(1);
                    if !ascii_isdigit(*s as ::core::ffi::c_int) {
                        return &raw const e_invarg as *const ::core::ffi::c_char;
                    }
                    col = col * getdigits_int(&raw mut s, true_0 != 0, 0 as ::core::ffi::c_int);
                    if tw == 0 as OptInt {
                        break '_skip;
                    } else {
                        debug_assert!(
                            col >= 0 as ::core::ffi::c_int
                                && tw <= (2147483647 as ::core::ffi::c_int - col) as OptInt
                                && tw + col as OptInt
                                    >= (-2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                        as OptInt
                                || col < 0 as ::core::ffi::c_int
                                    && tw
                                        >= (-2147483647 as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int
                                            - col)
                                            as OptInt
                                    && tw + col as OptInt <= 2147483647 as OptInt,
                            "(col >= 0 && tw <= INT_MAX - col && tw + col >= INT_MIN) || (col < 0 && tw >= INT_MIN - col && tw + col <= INT_MAX)"
                        );
                        col += tw as ::core::ffi::c_int;
                        if col < 0 as ::core::ffi::c_int {
                            break '_skip;
                        }
                    }
                } else if ascii_isdigit(*s as ::core::ffi::c_int) {
                    col = getdigits_int(&raw mut s, true_0 != 0, 0 as ::core::ffi::c_int);
                } else {
                    return &raw const e_invarg as *const ::core::ffi::c_char;
                }
                let c2rust_fresh8 = count;
                count = count.wrapping_add(1);
                color_cols[c2rust_fresh8 as usize] = col - 1 as ::core::ffi::c_int;
            }
            if *s as ::core::ffi::c_int == NUL {
                break;
            }
            if *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            s = s.offset(1);
            if *s as ::core::ffi::c_int == NUL {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
        }
        if wp.is_null() {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        xfree((*wp).w_p_cc_cols as *mut ::core::ffi::c_void);
        if count == 0 as ::core::ffi::c_uint {
            (*wp).w_p_cc_cols = ::core::ptr::null_mut::<::core::ffi::c_int>();
        } else {
            (*wp).w_p_cc_cols = xmalloc(
                ::core::mem::size_of::<::core::ffi::c_int>()
                    .wrapping_mul(count.wrapping_add(1 as ::core::ffi::c_uint) as size_t),
            ) as *mut ::core::ffi::c_int;
            qsort(
                &raw mut color_cols as *mut ::core::ffi::c_int as *mut ::core::ffi::c_void,
                count as size_t,
                ::core::mem::size_of::<::core::ffi::c_int>(),
                Some(
                    int_cmp
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
            while i < count {
                if j == 0 as ::core::ffi::c_int
                    || *(*wp)
                        .w_p_cc_cols
                        .offset((j - 1 as ::core::ffi::c_int) as isize)
                        != color_cols[i as usize]
                {
                    let c2rust_fresh9 = j;
                    j = j + 1;
                    *(*wp).w_p_cc_cols.offset(c2rust_fresh9 as isize) = color_cols[i as usize];
                }
                i = i.wrapping_add(1);
            }
            *(*wp).w_p_cc_cols.offset(j as isize) = -1 as ::core::ffi::c_int;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn get_last_winid() -> ::core::ffi::c_int {
    return last_win_id.get();
}

pub unsafe extern "C" fn win_locked(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        return (*wp).w_locked as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn win_get_tabwin(
    mut id: handle_T,
    mut tabnr: *mut ::core::ffi::c_int,
    mut winnr: *mut ::core::ffi::c_int,
) {
    unsafe {
        *tabnr = 0 as ::core::ffi::c_int;
        *winnr = 0 as ::core::ffi::c_int;
        let mut tnum: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut wnum: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).handle == id {
                    if win_has_winnr(wp, tp as *mut tabpage_T) {
                        *winnr = wnum;
                        *tabnr = tnum;
                    }
                    return;
                }
                wnum += win_has_winnr(wp, tp as *mut tabpage_T) as ::core::ffi::c_int;
                wp = (*wp).w_next;
            }
            tnum += 1;
            wnum = 1 as ::core::ffi::c_int;
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

pub unsafe extern "C" fn win_ui_flush(mut validate: bool) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if ((*wp).w_pos_changed as ::core::ffi::c_int != 0
                    || (*wp).w_grid_alloc.pending_comp_index_update as ::core::ffi::c_int != 0)
                    && !(*wp).w_grid_alloc.chars.is_null()
                {
                    if tp == curtab.get() {
                        ui_ext_win_position(wp, validate);
                    } else {
                        ui_call_win_hide((*wp).w_grid_alloc.handle as Integer);
                        (*wp).w_pos_changed = false_0 != 0;
                    }
                    (*wp).w_grid_alloc.pending_comp_index_update = false_0 != 0;
                }
                if tp == curtab.get() {
                    ui_ext_win_viewport(wp);
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        pum_ui_flush();
        msg_ui_flush();
    }
}

pub unsafe extern "C" fn lastwin_nofloating(mut tp: *mut tabpage_T) -> *mut win_T {
    unsafe {
        debug_assert!(tp != curtab.get() || tp.is_null(), "tp != curtab || !tp");
        let mut res: *mut win_T = if !tp.is_null() {
            (*tp).tp_lastwin
        } else {
            lastwin.get()
        };
        while (*res).w_floating {
            res = (*res).w_prev;
        }
        return res;
    }
}
