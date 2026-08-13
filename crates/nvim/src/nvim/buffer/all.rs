//! `:ball` -- one window per buffer.
//!
//! [`ex_buffer_all`] opens a window for every listed buffer (or closes the
//! extra ones for `:unhide`), splitting until the count or `'winheight'` says
//! to stop, reusing windows that already show the right buffer, and loading
//! each buffer as its window is entered.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::ex_cmds2::autowrite;
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::getchar::vgetc;
use crate::src::nvim::main::{
    Columns, Rows, autocmd_no_enter, autocmd_no_leave, cmdmod, curtab, curwin, first_tabpage,
    firstbuf, firstwin, got_int, jop_flags, lastwin, p_ch, p_ea, p_tpm, swap_exists_action,
    swap_exists_did_quit,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::options::kOptJopFlagClean;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::types::{
    CMD_sunhide, CMD_unhide, OptInt, buf_T, bufref_T, cleanup_T, exarg_T, except_T, linenr_T,
    tabpage_T, win_T,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::{
    WSP_BELOW, WSP_ROOM, WSP_VERT, global_stl_height, goto_tabpage_tp, lastwin_nofloating,
    tabline_height, tabpage_index, win_close, win_enter, win_locked, win_move_after, win_split,
    win_valid,
};

pub unsafe fn ex_buffer_all(mut eap: *mut exarg_T) {
    unsafe {
        let mut wpnext: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut split_ret: ::core::ffi::c_int = OK;
        let mut open_wins: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut had_tab: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_tab;
        let mut count: linenr_T = if (*eap).addr_count == 0 as ::core::ffi::c_int {
            9999 as linenr_T
        } else {
            (*eap).line2
        };
        let mut all: ::core::ffi::c_int = ((*eap).cmdidx as ::core::ffi::c_int
            != CMD_unhide as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_sunhide as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        reset_VIsual_and_resel();
        setpcmark();
        if had_tab > 0 as ::core::ffi::c_int {
            goto_tabpage_tp(first_tabpage.get(), true_0 != 0, true_0 != 0);
        }
        loop {
            let mut tpnext: *mut tabpage_T = (*curtab.get()).tp_next;
            let mut wp: *mut win_T = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
                lastwin.get()
            } else {
                firstwin.get()
            };
            while !wp.is_null() {
                wpnext = if (*wp).w_floating as ::core::ffi::c_int != 0 {
                    if (*(*wp).w_prev).w_floating as ::core::ffi::c_int != 0 {
                        (*wp).w_prev
                    } else {
                        firstwin.get()
                    }
                } else if (*wp).w_next.is_null()
                    || (*(*wp).w_next).w_floating as ::core::ffi::c_int != 0
                {
                    ::core::ptr::null_mut::<win_T>()
                } else {
                    (*wp).w_next
                };
                if ((*(*wp).w_buffer).b_nwindows > 1 as ::core::ffi::c_int
                    || (*wp).w_floating as ::core::ffi::c_int != 0
                    || (if (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
                        ((((*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height) as OptInt)
                            < Rows.get() as OptInt
                                - p_ch.get()
                                - tabline_height() as OptInt
                                - global_stl_height() as OptInt)
                            as ::core::ffi::c_int
                    } else {
                        ((*wp).w_width != Columns.get()) as ::core::ffi::c_int
                    }) != 0
                    || had_tab > 0 as ::core::ffi::c_int && wp != firstwin.get())
                    && !(firstwin.get() == lastwin.get())
                    && !(win_locked(wp) != 0
                        || (*(*wp).w_buffer).b_locked > 0 as ::core::ffi::c_int)
                    && !is_aucmd_win(wp)
                {
                    if win_close(wp, false_0 != 0, false_0 != 0) == FAIL {
                        break;
                    }
                    wpnext = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
                        lastwin.get()
                    } else {
                        firstwin.get()
                    };
                    tpnext = first_tabpage.get();
                    open_wins = 0 as ::core::ffi::c_int;
                } else {
                    open_wins += 1;
                }
                wp = wpnext;
            }
            if had_tab == 0 as ::core::ffi::c_int || tpnext.is_null() {
                break;
            }
            goto_tabpage_tp(tpnext, true_0 != 0, true_0 != 0);
        }
        (*autocmd_no_enter.ptr()) += 1;
        win_enter(
            lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()),
            false_0 != 0,
        );
        (*autocmd_no_leave.ptr()) += 1;
        let mut buf: *mut buf_T = firstbuf.get();
        's_295: while !buf.is_null() && (open_wins as linenr_T) < count {
            's_111: {
                if !(all == 0 && (*buf).b_ml.ml_mfp.is_null() || (*buf).b_p_bl == 0) {
                    let mut wp_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
                    if had_tab != 0 as ::core::ffi::c_int {
                        wp_0 = if (*buf).b_nwindows > 0 as ::core::ffi::c_int {
                            lastwin.get()
                        } else {
                            ::core::ptr::null_mut::<win_T>()
                        };
                    } else {
                        wp_0 = firstwin.get();
                        while !wp_0.is_null() {
                            if !(*wp_0).w_floating && (*wp_0).w_buffer == buf {
                                break;
                            }
                            wp_0 = (*wp_0).w_next;
                        }
                        if !wp_0.is_null() {
                            win_move_after(wp_0, curwin.get());
                        }
                    }
                    if wp_0.is_null() && split_ret == OK {
                        let mut bufref: bufref_T = bufref_T::default();
                        set_bufref(&raw mut bufref, buf);
                        let mut p_ea_save: bool = p_ea.get() != 0;
                        p_ea.set(true_0);
                        split_ret = win_split(
                            0 as ::core::ffi::c_int,
                            WSP_ROOM as ::core::ffi::c_int | WSP_BELOW as ::core::ffi::c_int,
                        );
                        open_wins += 1;
                        p_ea.set(p_ea_save as ::core::ffi::c_int);
                        if split_ret == FAIL {
                            break 's_111;
                        } else {
                            swap_exists_action.set(SEA_DIALOG);
                            set_curbuf(
                                buf,
                                DOBUF_GOTO as ::core::ffi::c_int,
                                jop_flags.get()
                                    & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                                    == 0,
                            );
                            if !bufref_valid(&raw mut bufref) {
                                swap_exists_action.set(SEA_NONE);
                                break 's_295;
                            } else if swap_exists_action.get() == SEA_QUIT {
                                let mut cs: cleanup_T = cleanup_T {
                                    pending: 0,
                                    exception: ::core::ptr::null_mut::<except_T>(),
                                };
                                enter_cleanup(&raw mut cs);
                                win_close(curwin.get(), true_0 != 0, false_0 != 0);
                                open_wins -= 1;
                                swap_exists_action.set(SEA_NONE);
                                swap_exists_did_quit.set(true_0 != 0);
                                leave_cleanup(&raw mut cs);
                            } else {
                                handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
                            }
                        }
                    }
                    os_breakcheck();
                    if got_int.get() {
                        vgetc();
                        break 's_295;
                    } else {
                        if aborting() {
                            break 's_295;
                        }
                        if had_tab > 0 as ::core::ffi::c_int
                            && tabpage_index(::core::ptr::null_mut::<tabpage_T>()) as OptInt
                                <= p_tpm.get()
                        {
                            (*cmdmod.ptr()).cmod_tab = 9999 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            buf = (*buf).b_next;
        }
        (*autocmd_no_enter.ptr()) -= 1;
        win_enter(firstwin.get(), false_0 != 0);
        (*autocmd_no_leave.ptr()) -= 1;
        let mut wp_1: *mut win_T = lastwin.get();
        while open_wins as linenr_T > count {
            let mut r: bool = (buf_hide((*wp_1).w_buffer) as ::core::ffi::c_int != 0
                || !bufIsChanged((*wp_1).w_buffer)
                || autowrite((*wp_1).w_buffer, false_0 != 0) == OK)
                && !is_aucmd_win(wp_1);
            if !win_valid(wp_1) {
                wp_1 = lastwin.get();
            } else if r {
                win_close(wp_1, !buf_hide((*wp_1).w_buffer), false_0 != 0);
                open_wins -= 1;
                wp_1 = lastwin.get();
            } else {
                wp_1 = (*wp_1).w_prev;
                if wp_1.is_null() {
                    break;
                }
            }
        }
    }
}
