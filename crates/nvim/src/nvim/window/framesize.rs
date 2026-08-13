//! Frame arithmetic -- giving a frame a new height or width.
//!
//! [`frame_new_height`] and [`frame_new_width`] distribute a frame's new size
//! over its children, recursing into rows and columns and stopping at the
//! `'winfix{height,width}'` pins; [`frame_minheight`] and [`frame_minwidth`]
//! answer how small a frame may become given `'winminheight'`/`'winminwidth'`
//! and the status lines and separators it contains.  The
//! `frame_add_statusline`/`frame_add_hsep`/`frame_set_vsep` trio adjusts those
//! non-text rows and columns when the layout changes around them.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::main::{Rows, curwin, p_ch, p_wh, p_wiw, p_wmh, p_wmw};
use crate::src::nvim::option::set_option_value;
use crate::src::nvim::options::kOptCmdheight;
use crate::src::nvim::types::{OptInt, OptVal, OptValData, frame_T, win_T};

pub unsafe extern "C" fn frame_new_height(
    mut topfrp: *mut frame_T,
    mut height: ::core::ffi::c_int,
    mut topfirst: bool,
    mut wfh: bool,
    mut set_ch: bool,
) {
    unsafe {
        if (*topfrp).fr_parent.is_null() && set_ch as ::core::ffi::c_int != 0 {
            let mut new_ch: OptInt = if min_set_ch.get()
                > p_ch.get() + (*topfrp).fr_height as OptInt - height as OptInt
            {
                min_set_ch.get()
            } else {
                p_ch.get() + (*topfrp).fr_height as OptInt - height as OptInt
            };
            if new_ch != p_ch.get() {
                let save_ch: OptInt = min_set_ch.get();
                set_option_value(
                    kOptCmdheight,
                    OptVal {
                        type_0: kOptValTypeNumber,
                        data: OptValData { number: new_ch },
                    },
                    0 as ::core::ffi::c_int,
                );
                min_set_ch.set(save_ch);
            }
            height = (if (Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt)
                < height as OptInt
            {
                Rows.get() as OptInt
                    - p_ch.get()
                    - tabline_height() as OptInt
                    - global_stl_height() as OptInt
            } else {
                height as OptInt
            }) as ::core::ffi::c_int;
        }
        if !(*topfrp).fr_win.is_null() {
            let mut wp: *mut win_T = (*topfrp).fr_win;
            if is_bottom_win(wp) {
                (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
            }
            win_new_height(wp, height - (*wp).w_hsep_height - (*wp).w_status_height);
        } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            loop {
                frp = (*topfrp).fr_child;
                while !frp.is_null() {
                    frame_new_height(frp, height, topfirst, wfh, set_ch);
                    if (*frp).fr_height > height {
                        height = (*frp).fr_height;
                        break;
                    } else {
                        frp = (*frp).fr_next;
                    }
                }
                if frp.is_null() {
                    break;
                }
            }
        } else {
            let mut frp_0: *mut frame_T = (*topfrp).fr_child;
            if wfh {
                while frame_fixed_height(frp_0) {
                    frp_0 = (*frp_0).fr_next;
                    if frp_0.is_null() {
                        return;
                    }
                }
            }
            if !topfirst {
                while !(*frp_0).fr_next.is_null() {
                    frp_0 = (*frp_0).fr_next;
                }
                if wfh {
                    while frame_fixed_height(frp_0) {
                        frp_0 = (*frp_0).fr_prev;
                    }
                }
            }
            let mut extra_lines: ::core::ffi::c_int = height - (*topfrp).fr_height;
            if extra_lines < 0 as ::core::ffi::c_int {
                while !frp_0.is_null() {
                    let mut h: ::core::ffi::c_int =
                        frame_minheight(frp_0, ::core::ptr::null_mut::<win_T>());
                    if (*frp_0).fr_height + extra_lines < h {
                        extra_lines += (*frp_0).fr_height - h;
                        frame_new_height(frp_0, h, topfirst, wfh, set_ch);
                        if topfirst {
                            loop {
                                frp_0 = (*frp_0).fr_next;
                                if !(wfh as ::core::ffi::c_int != 0
                                    && !frp_0.is_null()
                                    && frame_fixed_height(frp_0) as ::core::ffi::c_int != 0)
                                {
                                    break;
                                }
                            }
                        } else {
                            loop {
                                frp_0 = (*frp_0).fr_prev;
                                if !(wfh as ::core::ffi::c_int != 0
                                    && !frp_0.is_null()
                                    && frame_fixed_height(frp_0) as ::core::ffi::c_int != 0)
                                {
                                    break;
                                }
                            }
                        }
                        if frp_0.is_null() {
                            height -= extra_lines;
                        }
                    } else {
                        frame_new_height(
                            frp_0,
                            (*frp_0).fr_height + extra_lines,
                            topfirst,
                            wfh,
                            set_ch,
                        );
                        break;
                    }
                }
            } else if extra_lines > 0 as ::core::ffi::c_int {
                frame_new_height(
                    frp_0,
                    (*frp_0).fr_height + extra_lines,
                    topfirst,
                    wfh,
                    set_ch,
                );
            }
        }
        (*topfrp).fr_height = height;
    }
}

pub(crate) unsafe extern "C" fn frame_fixed_height(mut frp: *mut frame_T) -> bool {
    unsafe {
        if !(*frp).fr_win.is_null() {
            return (*(*frp).fr_win).w_onebuf_opt.wo_wfh != 0;
        }
        if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
            frp = (*frp).fr_child;
            while !frp.is_null() {
                if frame_fixed_height(frp) {
                    return true_0 != 0;
                }
                frp = (*frp).fr_next;
            }
            return false_0 != 0;
        }
        frp = (*frp).fr_child;
        while !frp.is_null() {
            if !frame_fixed_height(frp) {
                return false_0 != 0;
            }
            frp = (*frp).fr_next;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn frame_fixed_width(mut frp: *mut frame_T) -> bool {
    unsafe {
        if !(*frp).fr_win.is_null() {
            return (*(*frp).fr_win).w_onebuf_opt.wo_wfw != 0;
        }
        if (*frp).fr_layout as ::core::ffi::c_int == FR_COL {
            frp = (*frp).fr_child;
            while !frp.is_null() {
                if frame_fixed_width(frp) {
                    return true_0 != 0;
                }
                frp = (*frp).fr_next;
            }
            return false_0 != 0;
        }
        frp = (*frp).fr_child;
        while !frp.is_null() {
            if !frame_fixed_width(frp) {
                return false_0 != 0;
            }
            frp = (*frp).fr_next;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn frame_add_statusline(mut frp: *mut frame_T) {
    unsafe {
        if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
            let mut wp: *mut win_T = (*frp).fr_win;
            (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
        } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
            frp = (*frp).fr_child;
            while !frp.is_null() {
                frame_add_statusline(frp);
                frp = (*frp).fr_next;
            }
        } else {
            debug_assert!(
                (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int,
                "frp->fr_layout == FR_COL"
            );
            frp = (*frp).fr_child;
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            frame_add_statusline(frp);
        };
    }
}

pub(crate) unsafe extern "C" fn frame_new_width(
    mut topfrp: *mut frame_T,
    mut width: ::core::ffi::c_int,
    mut leftfirst: bool,
    mut wfw: bool,
) {
    unsafe {
        if (*topfrp).fr_layout as ::core::ffi::c_int == FR_LEAF {
            let mut wp: *mut win_T = (*topfrp).fr_win;
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp = topfrp;
            while !(*frp).fr_parent.is_null() {
                if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
                    && !(*frp).fr_next.is_null()
                {
                    break;
                }
                frp = (*frp).fr_parent;
            }
            if (*frp).fr_parent.is_null() {
                (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
            }
            win_new_width(wp, width - (*wp).w_vsep_width);
        } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_COL {
            let mut frp_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            loop {
                frp_0 = (*topfrp).fr_child;
                while !frp_0.is_null() {
                    frame_new_width(frp_0, width, leftfirst, wfw);
                    if (*frp_0).fr_width > width {
                        width = (*frp_0).fr_width;
                        break;
                    } else {
                        frp_0 = (*frp_0).fr_next;
                    }
                }
                if frp_0.is_null() {
                    break;
                }
            }
        } else {
            let mut frp_1: *mut frame_T = (*topfrp).fr_child;
            if wfw {
                while frame_fixed_width(frp_1) {
                    frp_1 = (*frp_1).fr_next;
                    if frp_1.is_null() {
                        return;
                    }
                }
            }
            if !leftfirst {
                while !(*frp_1).fr_next.is_null() {
                    frp_1 = (*frp_1).fr_next;
                }
                if wfw {
                    while frame_fixed_width(frp_1) {
                        frp_1 = (*frp_1).fr_prev;
                    }
                }
            }
            let mut extra_cols: ::core::ffi::c_int = width - (*topfrp).fr_width;
            if extra_cols < 0 as ::core::ffi::c_int {
                while !frp_1.is_null() {
                    let mut w: ::core::ffi::c_int =
                        frame_minwidth(frp_1, ::core::ptr::null_mut::<win_T>());
                    if (*frp_1).fr_width + extra_cols < w {
                        extra_cols += (*frp_1).fr_width - w;
                        frame_new_width(frp_1, w, leftfirst, wfw);
                        if leftfirst {
                            loop {
                                frp_1 = (*frp_1).fr_next;
                                if !(wfw as ::core::ffi::c_int != 0
                                    && !frp_1.is_null()
                                    && frame_fixed_width(frp_1) as ::core::ffi::c_int != 0)
                                {
                                    break;
                                }
                            }
                        } else {
                            loop {
                                frp_1 = (*frp_1).fr_prev;
                                if !(wfw as ::core::ffi::c_int != 0
                                    && !frp_1.is_null()
                                    && frame_fixed_width(frp_1) as ::core::ffi::c_int != 0)
                                {
                                    break;
                                }
                            }
                        }
                        if frp_1.is_null() {
                            width -= extra_cols;
                        }
                    } else {
                        frame_new_width(frp_1, (*frp_1).fr_width + extra_cols, leftfirst, wfw);
                        break;
                    }
                }
            } else if extra_cols > 0 as ::core::ffi::c_int {
                frame_new_width(frp_1, (*frp_1).fr_width + extra_cols, leftfirst, wfw);
            }
        }
        (*topfrp).fr_width = width;
    }
}

pub(crate) unsafe extern "C" fn frame_set_vsep(mut frp: *const frame_T, mut add: bool) {
    unsafe {
        if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
            let mut wp: *mut win_T = (*frp).fr_win;
            if add as ::core::ffi::c_int != 0 && (*wp).w_vsep_width == 0 as ::core::ffi::c_int {
                if (*wp).w_width > 0 as ::core::ffi::c_int {
                    win_new_width(wp, (*wp).w_width - 1 as ::core::ffi::c_int);
                }
                (*wp).w_vsep_width = 1 as ::core::ffi::c_int;
            } else if !add && (*wp).w_vsep_width == 1 as ::core::ffi::c_int {
                win_new_width(wp, (*wp).w_width + 1 as ::core::ffi::c_int);
                (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
            }
        } else if (*frp).fr_layout as ::core::ffi::c_int == FR_COL {
            frp = (*frp).fr_child;
            while !frp.is_null() {
                frame_set_vsep(frp, add);
                frp = (*frp).fr_next;
            }
        } else {
            debug_assert!(
                (*frp).fr_layout as ::core::ffi::c_int == 1 as ::core::ffi::c_int,
                "frp->fr_layout == FR_ROW"
            );
            frp = (*frp).fr_child;
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            frame_set_vsep(frp, add);
        };
    }
}

pub(crate) unsafe extern "C" fn frame_add_hsep(mut frp: *const frame_T) {
    unsafe {
        if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
            let mut wp: *mut win_T = (*frp).fr_win;
            (*wp).w_hsep_height = 1 as ::core::ffi::c_int;
        } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
            frp = (*frp).fr_child;
            while !frp.is_null() {
                frame_add_hsep(frp);
                frp = (*frp).fr_next;
            }
        } else {
            debug_assert!(
                (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int,
                "frp->fr_layout == FR_COL"
            );
            frp = (*frp).fr_child;
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            frame_add_hsep(frp);
        };
    }
}

pub(crate) unsafe extern "C" fn frame_fix_width(mut wp: *mut win_T) {
    unsafe {
        (*(*wp).w_frame).fr_width = (*wp).w_width + (*wp).w_vsep_width;
    }
}

pub(crate) unsafe extern "C" fn frame_fix_height(mut wp: *mut win_T) {
    unsafe {
        (*(*wp).w_frame).fr_height = (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
    }
}

pub(crate) unsafe extern "C" fn frame_minheight(
    mut topfrp: *mut frame_T,
    mut next_curwin: *mut win_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut m: ::core::ffi::c_int = 0;
        if !(*topfrp).fr_win.is_null() {
            let mut extra_height: ::core::ffi::c_int = (*(*topfrp).fr_win).w_winbar_height
                + (*(*topfrp).fr_win).w_hsep_height
                + (*(*topfrp).fr_win).w_status_height;
            if (*topfrp).fr_win == next_curwin {
                m = p_wh.get() as ::core::ffi::c_int + extra_height;
            } else {
                m = p_wmh.get() as ::core::ffi::c_int + extra_height;
                if (*topfrp).fr_win == curwin.get() && next_curwin.is_null() {
                    if p_wmh.get() == 0 as OptInt {
                        m += 1;
                    }
                }
            }
        } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
            m = 0 as ::core::ffi::c_int;
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp = (*topfrp).fr_child;
            while !frp.is_null() {
                let mut n: ::core::ffi::c_int = frame_minheight(frp, next_curwin);
                if n > m {
                    m = n;
                }
                frp = (*frp).fr_next;
            }
        } else {
            m = 0 as ::core::ffi::c_int;
            let mut frp_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp_0 = (*topfrp).fr_child;
            while !frp_0.is_null() {
                m += frame_minheight(frp_0, next_curwin);
                frp_0 = (*frp_0).fr_next;
            }
        }
        return m;
    }
}

pub(crate) unsafe extern "C" fn frame_minwidth(
    mut topfrp: *mut frame_T,
    mut next_curwin: *mut win_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut m: ::core::ffi::c_int = 0;
        if !(*topfrp).fr_win.is_null() {
            if (*topfrp).fr_win == next_curwin {
                m = p_wiw.get() as ::core::ffi::c_int + (*(*topfrp).fr_win).w_vsep_width;
            } else {
                m = p_wmw.get() as ::core::ffi::c_int + (*(*topfrp).fr_win).w_vsep_width;
                if p_wmw.get() == 0 as OptInt
                    && (*topfrp).fr_win == curwin.get()
                    && next_curwin.is_null()
                {
                    m += 1;
                }
            }
        } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_COL {
            m = 0 as ::core::ffi::c_int;
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp = (*topfrp).fr_child;
            while !frp.is_null() {
                let mut n: ::core::ffi::c_int = frame_minwidth(frp, next_curwin);
                m = if m > n { m } else { n };
                frp = (*frp).fr_next;
            }
        } else {
            m = 0 as ::core::ffi::c_int;
            let mut frp_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp_0 = (*topfrp).fr_child;
            while !frp_0.is_null() {
                m += frame_minwidth(frp_0, next_curwin);
                frp_0 = (*frp_0).fr_next;
            }
        }
        return m;
    }
}
