//! The frame tree -- removing a window's frame and giving its room away.
//!
//! [`winframe_remove`] unlinks a window's leaf and returns the frame that
//! inherits its space, [`winframe_find_altwin`] picks that neighbour (which
//! is what decides where the cursor goes after `:close`), [`frame_flatten`]
//! collapses a row or column left with a single child, and
//! [`winframe_restore`] puts one back when the close is undone.
//! [`win_altframe`], [`frame2win`], [`frame_has_win`] and [`is_bottom_win`]
//! are the small queries over the tree the rest of the family asks.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::main::{
    cmdline_win, curtab, first_tabpage, lastused_tabpage, p_sb, p_spr, tcl_flags, topframe,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::options::{kOptTclFlagLeft, kOptTclFlagUselast};
use crate::src::nvim::types::{frame_T, tabpage_T, win_T};
use crate::src::nvim::winfloat::win_float_find_altwin;

pub(crate) unsafe extern "C" fn win_free_mem(
    mut win: *mut win_T,
    mut dirp: *mut ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut win_tp: *mut tabpage_T = if tp.is_null() { curtab.get() } else { tp };
        if !(*win).w_floating {
            let mut frp: *mut frame_T = (*win).w_frame;
            wp = winframe_remove(win, dirp, tp, ::core::ptr::null_mut::<*mut frame_T>());
            xfree(frp as *mut ::core::ffi::c_void);
        } else {
            *dirp = 'h' as ::core::ffi::c_int;
            wp = win_float_find_altwin(win, tp);
        }
        win_free(win, tp);
        if win == (*win_tp).tp_curwin {
            (*win_tp).tp_curwin = wp;
        }
        if win == cmdline_win.get() {
            cmdline_win.set(::core::ptr::null_mut::<win_T>());
        }
        return wp;
    }
}

pub unsafe extern "C" fn winframe_remove(
    mut win: *mut win_T,
    mut dirp: *mut ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut unflat_altfr: *mut *mut frame_T,
) -> *mut win_T {
    unsafe {
        let mut altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        let mut wp: *mut win_T = winframe_find_altwin(win, dirp, tp, &raw mut altfr);
        if wp.is_null() {
            return ::core::ptr::null_mut::<win_T>();
        }
        let mut frp_close: *mut frame_T = (*win).w_frame;
        (*frame_locked.ptr()) += 1;
        let topleft: *const win_T = frame2win((*frp_close).fr_parent);
        let mut row: ::core::ffi::c_int = (*topleft).w_winrow;
        let mut col: ::core::ffi::c_int = (*topleft).w_wincol;
        if (*win).w_vsep_width == 0 as ::core::ffi::c_int
            && (*(*frp_close).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && !(*frp_close).fr_prev.is_null()
        {
            frame_set_vsep((*frp_close).fr_prev, false_0 != 0);
        }
        frame_remove(frp_close);
        if *dirp == 'v' as ::core::ffi::c_int {
            frame_new_height(
                altfr,
                (*altfr).fr_height + (*frp_close).fr_height,
                altfr == (*frp_close).fr_next,
                false_0 != 0,
                false_0 != 0,
            );
        } else {
            debug_assert!(*dirp == 'h' as ::core::ffi::c_int, "*dirp == 'h'");
            frame_new_width(
                altfr,
                (*altfr).fr_width + (*frp_close).fr_width,
                altfr == (*frp_close).fr_next,
                false_0 != 0,
            );
        }
        if altfr != (*frp_close).fr_prev {
            frame_comp_pos((*frp_close).fr_parent, &raw mut row, &raw mut col);
        }
        if unflat_altfr.is_null() {
            frame_flatten(altfr);
        } else {
            *unflat_altfr = altfr;
        }
        (*frame_locked.ptr()) -= 1;
        return wp;
    }
}

pub unsafe extern "C" fn winframe_find_altwin(
    mut win: *mut win_T,
    mut dirp: *mut ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut altfr: *mut *mut frame_T,
) -> *mut win_T {
    unsafe {
        debug_assert!(
            tp.is_null() || tp != curtab.get(),
            "tp == NULL || tp != curtab"
        );
        if one_window(win, tp) {
            return ::core::ptr::null_mut::<win_T>();
        }
        let mut frp_close: *mut frame_T = (*win).w_frame;
        let mut frp2: *mut frame_T = win_altframe(win, tp);
        let mut wp: *mut win_T = frame2win(frp2);
        if (*(*frp_close).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL {
            if !(*frp2).fr_win.is_null() && (*(*frp2).fr_win).w_onebuf_opt.wo_wfh != 0 {
                let mut frp: *mut frame_T = (*frp_close).fr_prev;
                let mut frp3: *mut frame_T = (*frp_close).fr_next;
                while !frp.is_null() || !frp3.is_null() {
                    if !frp.is_null() {
                        if !frame_fixed_height(frp) {
                            frp2 = frp;
                            wp = frame2win(frp2);
                            break;
                        } else {
                            frp = (*frp).fr_prev;
                        }
                    }
                    if frp3.is_null() {
                        continue;
                    }
                    if !(*frp3).fr_win.is_null() && (*(*frp3).fr_win).w_onebuf_opt.wo_wfh == 0 {
                        frp2 = frp3;
                        wp = (*frp3).fr_win;
                        break;
                    } else {
                        frp3 = (*frp3).fr_next;
                    }
                }
            }
            *dirp = 'v' as ::core::ffi::c_int;
        } else {
            if !(*frp2).fr_win.is_null() && (*(*frp2).fr_win).w_onebuf_opt.wo_wfw != 0 {
                let mut frp_0: *mut frame_T = (*frp_close).fr_prev;
                let mut frp3_0: *mut frame_T = (*frp_close).fr_next;
                while !frp_0.is_null() || !frp3_0.is_null() {
                    if !frp_0.is_null() {
                        if !frame_fixed_width(frp_0) {
                            frp2 = frp_0;
                            wp = frame2win(frp2);
                            break;
                        } else {
                            frp_0 = (*frp_0).fr_prev;
                        }
                    }
                    if frp3_0.is_null() {
                        continue;
                    }
                    if !(*frp3_0).fr_win.is_null() && (*(*frp3_0).fr_win).w_onebuf_opt.wo_wfw == 0 {
                        frp2 = frp3_0;
                        wp = (*frp3_0).fr_win;
                        break;
                    } else {
                        frp3_0 = (*frp3_0).fr_next;
                    }
                }
            }
            *dirp = 'h' as ::core::ffi::c_int;
        }
        debug_assert!(
            wp != win && frp2 != frp_close,
            "wp != win && frp2 != frp_close"
        );
        if !altfr.is_null() {
            *altfr = frp2;
        }
        return wp;
    }
}

pub(crate) unsafe extern "C" fn frame_flatten(mut frp: *mut frame_T) {
    unsafe {
        if !(*frp).fr_next.is_null() || !(*frp).fr_prev.is_null() {
            return;
        }
        (*(*frp).fr_parent).fr_layout = (*frp).fr_layout;
        (*(*frp).fr_parent).fr_child = (*frp).fr_child;
        let mut frp2: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp2 = (*frp).fr_child;
        while !frp2.is_null() {
            (*frp2).fr_parent = (*frp).fr_parent;
            frp2 = (*frp2).fr_next;
        }
        (*(*frp).fr_parent).fr_win = (*frp).fr_win;
        if !(*frp).fr_win.is_null() {
            (*(*frp).fr_win).w_frame = (*frp).fr_parent;
        }
        frp2 = (*frp).fr_parent;
        if (*topframe.get()).fr_child == frp {
            (*topframe.get()).fr_child = frp2;
        }
        xfree(frp as *mut ::core::ffi::c_void);
        frp = (*frp2).fr_parent;
        if !frp.is_null()
            && (*frp).fr_layout as ::core::ffi::c_int == (*frp2).fr_layout as ::core::ffi::c_int
        {
            if (*frp).fr_child == frp2 {
                (*frp).fr_child = (*frp2).fr_child;
            }
            debug_assert!(!(*frp2).fr_child.is_null(), "frp2->fr_child");
            (*(*frp2).fr_child).fr_prev = (*frp2).fr_prev;
            if !(*frp2).fr_prev.is_null() {
                (*(*frp2).fr_prev).fr_next = (*frp2).fr_child;
            }
            let mut frp3: *mut frame_T = (*frp2).fr_child;
            loop {
                (*frp3).fr_parent = frp;
                if (*frp3).fr_next.is_null() {
                    (*frp3).fr_next = (*frp2).fr_next;
                    if !(*frp2).fr_next.is_null() {
                        (*(*frp2).fr_next).fr_prev = frp3;
                    }
                    break;
                } else {
                    frp3 = (*frp3).fr_next;
                }
            }
            if (*topframe.get()).fr_child == frp2 {
                (*topframe.get()).fr_child = frp;
            }
            xfree(frp2 as *mut ::core::ffi::c_void);
        }
    }
}

pub unsafe extern "C" fn winframe_restore(
    mut wp: *mut win_T,
    mut dir: ::core::ffi::c_int,
    mut unflat_altfr: *mut frame_T,
) {
    unsafe {
        let mut frp: *mut frame_T = (*wp).w_frame;
        if !(*frp).fr_prev.is_null() {
            frame_append((*frp).fr_prev, frp);
        } else {
            frame_insert((*frp).fr_next, frp);
        }
        if (*wp).w_vsep_width == 0 as ::core::ffi::c_int
            && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && !(*frp).fr_prev.is_null()
        {
            frame_set_vsep((*frp).fr_prev, true_0 != 0);
        }
        if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && !(*frp).fr_prev.is_null()
        {
            if global_stl_height() == 0 as ::core::ffi::c_int
                && (*wp).w_status_height == 0 as ::core::ffi::c_int
            {
                frame_add_statusline((*frp).fr_prev);
            } else if global_stl_height() > 0 as ::core::ffi::c_int
                && (*wp).w_hsep_height == 0 as ::core::ffi::c_int
            {
                frame_add_hsep((*frp).fr_prev);
            }
        }
        if dir == 'v' as ::core::ffi::c_int {
            frame_new_height(
                unflat_altfr,
                (*unflat_altfr).fr_height - (*frp).fr_height,
                unflat_altfr == (*frp).fr_next,
                false_0 != 0,
                false_0 != 0,
            );
        } else if dir == 'h' as ::core::ffi::c_int {
            frame_new_width(
                unflat_altfr,
                (*unflat_altfr).fr_width - (*frp).fr_width,
                unflat_altfr == (*frp).fr_next,
                false_0 != 0,
            );
        }
        if unflat_altfr != (*frp).fr_prev {
            let topleft: *const win_T = frame2win((*frp).fr_parent);
            let mut row: ::core::ffi::c_int = (*topleft).w_winrow;
            let mut col: ::core::ffi::c_int = (*topleft).w_wincol;
            frame_comp_pos((*frp).fr_parent, &raw mut row, &raw mut col);
        }
    }
}

pub(crate) unsafe extern "C" fn win_altframe(
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
) -> *mut frame_T {
    unsafe {
        debug_assert!(
            tp.is_null() || tp != curtab.get(),
            "tp == NULL || tp != curtab"
        );
        if one_window(win, tp) {
            return (*(*alt_tabpage()).tp_curwin).w_frame;
        }
        let mut frp: *mut frame_T = (*win).w_frame;
        if (*frp).fr_prev.is_null() {
            return (*frp).fr_next;
        }
        if (*frp).fr_next.is_null() {
            return (*frp).fr_prev;
        }
        let mut target_fr: *mut frame_T = (*frp).fr_next;
        let mut other_fr: *mut frame_T = (*frp).fr_prev;
        if !(*frp).fr_parent.is_null()
            && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && p_sb.get() != 0
        {
            target_fr = (*frp).fr_prev;
            other_fr = (*frp).fr_next;
        }
        if !(*frp).fr_parent.is_null()
            && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && p_spr.get() != 0
        {
            target_fr = (*frp).fr_prev;
            other_fr = (*frp).fr_next;
        }
        if !(*frp).fr_parent.is_null()
            && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
        {
            if frame_fixed_width(target_fr) as ::core::ffi::c_int != 0
                && !frame_fixed_width(other_fr)
            {
                target_fr = other_fr;
            }
        } else if frame_fixed_height(target_fr) as ::core::ffi::c_int != 0
            && !frame_fixed_height(other_fr)
        {
            target_fr = other_fr;
        }
        return target_fr;
    }
}

pub(crate) unsafe extern "C" fn alt_tabpage() -> *mut tabpage_T {
    unsafe {
        if tcl_flags.get() & kOptTclFlagUselast as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && valid_tabpage(lastused_tabpage.get()) as ::core::ffi::c_int != 0
        {
            return lastused_tabpage.get();
        }
        let mut forward: bool = !(*curtab.get()).tp_next.is_null()
            && (tcl_flags.get() & kOptTclFlagLeft as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
                || curtab.get() == first_tabpage.get());
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if forward {
            tp = (*curtab.get()).tp_next;
        } else {
            tp = first_tabpage.get();
            while (*tp).tp_next != curtab.get() {
                tp = (*tp).tp_next;
            }
        }
        return tp;
    }
}

pub unsafe extern "C" fn frame2win(mut frp: *mut frame_T) -> *mut win_T {
    unsafe {
        while (*frp).fr_win.is_null() {
            frp = (*frp).fr_child;
        }
        return (*frp).fr_win;
    }
}

pub(crate) unsafe extern "C" fn frame_has_win(
    mut frp: *const frame_T,
    mut wp: *const win_T,
) -> bool {
    unsafe {
        if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
            return (*frp).fr_win == wp as *mut win_T;
        }
        let mut p: *const frame_T = ::core::ptr::null::<frame_T>();
        p = (*frp).fr_child;
        while !p.is_null() {
            if frame_has_win(p, wp) {
                return true_0 != 0;
            }
            p = (*p).fr_next;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn is_bottom_win(mut wp: *mut win_T) -> bool {
    unsafe {
        let mut frp: *mut frame_T = (*wp).w_frame;
        while !(*frp).fr_parent.is_null() {
            if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
                && !(*frp).fr_next.is_null()
            {
                return false_0 != 0;
            }
            frp = (*frp).fr_parent;
        }
        return true_0 != 0;
    }
}
