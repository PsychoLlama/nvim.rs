//! Deciding whether a window may close, and closing all the others.
//!
//! [`close_windows`] closes every window showing a given buffer,
//! [`close_others`] is `:only`, and the predicates around them --
//! [`last_window`], [`one_window`], [`can_close_floating_windows`],
//! [`can_close_in_cmdwin`] -- are the questions asked before any of it.
//! [`close_last_window_tabpage`] handles the case where the window being
//! closed is the last one on its tab page, and the
//! `leaving_window`/`entering_window` pair keeps the cmdline window's saved
//! state consistent across the move.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::api_set_error;
use crate::src::nvim::autocmd::{
    EVENT_BUFENTER, EVENT_TABENTER, EVENT_WINENTER, apply_autocmds, is_aucmd_win,
};
use crate::src::nvim::buffer::{
    bt_prompt, bt_quickfix, buf_hide, buf_valid, bufref_valid, close_buffer, set_bufref,
};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::ex_cmds2::{can_abandon, dialog_changed};
use crate::src::nvim::keycodes::Ctrl_C;
use crate::src::nvim::main::{
    RedrawingDisabled, State, autocmd_busy, clear_cmdline, cmdmod, cmdwin_old_curwin,
    cmdwin_result, cmdwin_type, cmdwin_win, curbuf, curtab, curwin, e_cmdwin, e_floatonly,
    first_tabpage, firstbuf, firstwin, lastwin, mode_displayed, p_confirm, p_write, restart_edit,
    stop_insert_mode,
};
use crate::src::nvim::message::{emsg, msg};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::syntax::reset_synblock;
use crate::src::nvim::types::{
    CMD_SIZE, CMOD_CONFIRM, Error, Terminal, buf_T, bufref_T, colnr_T, kErrorTypeException,
    linenr_T, tabpage_T, win_T,
};
use crate::src::nvim::undo::bufIsChanged;

pub unsafe extern "C" fn leaving_window(win: *mut win_T) {
    unsafe {
        if !bt_prompt((*win).w_buffer) || is_aucmd_win(win) as ::core::ffi::c_int != 0 {
            return;
        }
        (*(*win).w_buffer).b_prompt_insert = restart_edit.get();
        if restart_edit.get() != NUL && mode_displayed.get() as ::core::ffi::c_int != 0 {
            clear_cmdline.set(true_0 != 0);
        }
        restart_edit.set(NUL);
        if State.get() & MODE_INSERT != 0 && !stop_insert_mode.get() {
            stop_insert_mode.set(true_0 != 0);
            if (*(*win).w_buffer).b_prompt_insert == NUL {
                (*(*win).w_buffer).b_prompt_insert = 'A' as ::core::ffi::c_int;
            }
        }
    }
}

pub unsafe extern "C" fn entering_window(win: *mut win_T) {
    unsafe {
        if !bt_prompt((*win).w_buffer) || is_aucmd_win(win) as ::core::ffi::c_int != 0 {
            return;
        }
        if (*(*win).w_buffer).b_prompt_insert != NUL {
            stop_insert_mode.set(false_0 != 0);
        }
        if State.get() & MODE_INSERT == 0 as ::core::ffi::c_int {
            restart_edit.set((*(*win).w_buffer).b_prompt_insert);
        }
    }
}

pub unsafe extern "C" fn win_init_empty(mut wp: *mut win_T) {
    unsafe {
        redraw_later(wp, UPD_NOT_VALID);
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        (*wp).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*wp).w_curswant = (*wp).w_cursor.col;
        (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*wp).w_pcmark.lnum = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
        (*wp).w_prev_pcmark.lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_prev_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
        (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_topfill = 0 as ::core::ffi::c_int;
        (*wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
        (*wp).w_valid = 0 as ::core::ffi::c_int;
        (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s;
    }
}

pub unsafe extern "C" fn curwin_init() {
    unsafe {
        win_init_empty(curwin.get());
    }
}

pub unsafe extern "C" fn close_windows(mut buf: *mut buf_T, mut keep_curwin: bool) {
    unsafe {
        let mut nexttp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        (*RedrawingDisabled.ptr()) += 1;
        let mut wp: *mut win_T = lastwin.get();
        '_theend: {
            while !wp.is_null()
                && (is_aucmd_win(lastwin.get()) as ::core::ffi::c_int != 0
                    || !one_window(wp, ::core::ptr::null_mut::<tabpage_T>()))
            {
                if (*wp).w_buffer == buf
                    && (!keep_curwin || wp != curwin.get())
                    && !(win_locked(wp) != 0
                        || (*(*wp).w_buffer).b_locked > 0 as ::core::ffi::c_int)
                {
                    if window_layout_locked(CMD_SIZE) {
                        break '_theend;
                    }
                    if win_close(wp, false_0 != 0, false_0 != 0) == FAIL {
                        break;
                    }
                    wp = lastwin.get();
                } else {
                    wp = (*wp).w_prev;
                }
            }
            nexttp = ::core::ptr::null_mut::<tabpage_T>();
            let mut tp: *mut tabpage_T = first_tabpage.get();
            loop {
                if tp.is_null() {
                    break '_theend;
                }
                nexttp = (*tp).tp_next;
                's_53: {
                    if tp != curtab.get() {
                        let mut wp_0: *mut win_T = (*tp).tp_lastwin;
                        loop {
                            if wp_0.is_null() {
                                break 's_53;
                            }
                            if (*wp_0).w_buffer == buf
                                && !(win_locked(wp_0) != 0
                                    || (*(*wp_0).w_buffer).b_locked > 0 as ::core::ffi::c_int)
                            {
                                if window_layout_locked(CMD_SIZE) {
                                    break '_theend;
                                }
                                if !win_close_othertab(wp_0, false_0, tp, false_0 != 0) {
                                    break 's_53;
                                }
                                nexttp = first_tabpage.get();
                                break 's_53;
                            } else {
                                wp_0 = (*wp_0).w_prev;
                            }
                        }
                    }
                }
                tp = nexttp;
            }
        }
        (*RedrawingDisabled.ptr()) -= 1;
    }
}

pub unsafe extern "C" fn last_window(mut win: *mut win_T) -> bool {
    unsafe {
        return one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
            && (*first_tabpage.get()).tp_next.is_null();
    }
}

pub unsafe extern "C" fn one_window(mut win: *mut win_T, mut tp: *mut tabpage_T) -> bool {
    unsafe {
        let mut first: *mut win_T = if !tp.is_null() {
            (*tp).tp_firstwin
        } else {
            firstwin.get()
        };
        debug_assert!(
            (tp.is_null() || tp != curtab.get()) && !(*first).w_floating,
            "(!tp || tp != curtab) && !first->w_floating"
        );
        return first == win
            && ((*win).w_next.is_null() || (*(*win).w_next).w_floating as ::core::ffi::c_int != 0);
    }
}

pub(crate) unsafe extern "C" fn can_close_floating_windows(mut tp: *mut tabpage_T) -> bool {
    unsafe {
        debug_assert!(
            tp != curtab.get() && (!tp.is_null() || !is_aucmd_win(lastwin.get())),
            "tp != curtab && (tp || !is_aucmd_win(lastwin))"
        );
        let mut wp: *mut win_T = if !tp.is_null() {
            (*tp).tp_lastwin
        } else {
            lastwin.get()
        };
        while (*wp).w_floating {
            let mut buf: *mut buf_T = (*wp).w_buffer;
            let mut need_hide: ::core::ffi::c_int = (bufIsChanged(buf) as ::core::ffi::c_int != 0
                && (*buf).b_nwindows <= 1 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            if need_hide != 0 && !buf_hide(buf) {
                return false_0 != 0;
            }
            wp = (*wp).w_prev;
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn can_close_in_cmdwin(mut win: *mut win_T, mut err: *mut Error) -> bool {
    unsafe {
        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
            if win == cmdwin_win.get() {
                cmdwin_result.set(Ctrl_C);
                return false_0 != 0;
            } else if win == cmdwin_old_curwin.get() {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"%s".as_ptr(),
                    &raw const e_cmdwin as *const ::core::ffi::c_char,
                );
                return false_0 != 0;
            }
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn close_last_window_tabpage(
    mut win: *mut win_T,
    mut free_buf: bool,
    mut prev_curtab: *mut tabpage_T,
) -> bool {
    unsafe {
        if !(firstwin.get() == lastwin.get()) {
            return false_0 != 0;
        }
        let mut old_curbuf: *mut buf_T = curbuf.get();
        let mut term: *mut Terminal = if !(*win).w_buffer.is_null() {
            (*(*win).w_buffer).terminal
        } else {
            ::core::ptr::null_mut::<Terminal>()
        };
        if !term.is_null() {
            free_buf = false_0 != 0;
        }
        goto_tabpage_tp(alt_tabpage(), false_0 != 0, !(*win).w_buffer.is_null());
        if curtab.get() != prev_curtab
            && valid_tabpage(prev_curtab) as ::core::ffi::c_int != 0
            && (*prev_curtab).tp_firstwin == win
        {
            win_close_othertab(
                win,
                free_buf as ::core::ffi::c_int,
                prev_curtab,
                false_0 != 0,
            );
        }
        entering_window(curwin.get());
        apply_autocmds(
            EVENT_WINENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(
            EVENT_TABENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if old_curbuf != curbuf.get() {
            apply_autocmds(
                EVENT_BUFENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn win_close_buffer(
    mut win: *mut win_T,
    mut action: ::core::ffi::c_int,
    mut abort_if_last: bool,
) -> bool {
    unsafe {
        if !(*win).w_buffer.is_null() {
            reset_synblock(win);
        }
        if !(*win).w_buffer.is_null()
            && bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
            && (*(*win).w_buffer).b_nwindows == 1 as ::core::ffi::c_int
        {
            (*(*win).w_buffer).b_p_bl = false_0;
        }
        let mut retval: bool = false_0 != 0;
        if !(*win).w_buffer.is_null() {
            let mut bufref: bufref_T = bufref_T::default();
            set_bufref(&raw mut bufref, curbuf.get());
            (*win).w_locked = true_0 != 0;
            retval = close_buffer(win, (*win).w_buffer, action, abort_if_last, true_0 != 0);
            if win_valid_any_tab(win) {
                (*win).w_locked = false_0 != 0;
            }
            if !bufref_valid(&raw mut bufref) {
                curbuf.set(firstbuf.get());
            }
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn win_unclose_buffer(
    mut win: *mut win_T,
    mut bufref: *mut bufref_T,
    mut did_decrement: bool,
) {
    unsafe {
        if (*win).w_buffer.is_null() {
            (*win).w_buffer = firstbuf.get();
            (*firstbuf.get()).b_nwindows += 1;
            if win == curwin.get() {
                curbuf.set((*curwin.get()).w_buffer);
            }
            win_init_empty(win);
        } else if did_decrement as ::core::ffi::c_int != 0
            && (*win).w_buffer == (*bufref).br_buf
            && bufref_valid(bufref) as ::core::ffi::c_int != 0
        {
            (*(*win).w_buffer).b_nwindows += 1;
        }
    }
}

pub unsafe extern "C" fn close_others(
    mut message: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) {
    unsafe {
        let old_curwin: *mut win_T = curwin.get();
        if (*curwin.get()).w_floating {
            if message != 0 && !autocmd_busy.get() {
                emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
            }
            return;
        }
        if one_window(firstwin.get(), ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int
            != 0
            && !(*lastwin.get()).w_floating
        {
            if message != 0 && !autocmd_busy.get() {
                msg(gettext(m_onlyone.get()), 0 as ::core::ffi::c_int);
            }
            return;
        }
        let mut nextwp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wp: *mut win_T = firstwin.get();
        while win_valid(wp) {
            nextwp = (*wp).w_next;
            if old_curwin != curwin.get() && win_valid(old_curwin) as ::core::ffi::c_int != 0 {
                curwin.set(old_curwin);
                curbuf.set((*curwin.get()).w_buffer);
            }
            's_52: {
                if wp != curwin.get() {
                    if !buf_valid((*wp).w_buffer) && win_valid(wp) as ::core::ffi::c_int != 0 {
                        (*wp).w_buffer = ::core::ptr::null_mut::<buf_T>();
                        win_close(wp, false_0 != 0, false_0 != 0);
                    } else {
                        let mut r: ::core::ffi::c_int =
                            can_abandon((*wp).w_buffer, forceit != 0) as ::core::ffi::c_int;
                        if !win_valid(wp) {
                            nextwp = firstwin.get();
                        } else {
                            if r == 0 {
                                if message != 0
                                    && (p_confirm.get() != 0
                                        || (*cmdmod.ptr()).cmod_flags
                                            & CMOD_CONFIRM as ::core::ffi::c_int
                                            != 0)
                                    && p_write.get() != 0
                                {
                                    dialog_changed((*wp).w_buffer, false_0 != 0);
                                    if !win_valid(wp) {
                                        nextwp = firstwin.get();
                                        break 's_52;
                                    }
                                }
                                if bufIsChanged((*wp).w_buffer) {
                                    break 's_52;
                                }
                            }
                            win_close(
                                wp,
                                !buf_hide((*wp).w_buffer) && !bufIsChanged((*wp).w_buffer),
                                false_0 != 0,
                            );
                        }
                    }
                }
            }
            wp = nextwp;
        }
        if message != 0 && !(firstwin.get() == lastwin.get()) {
            emsg(gettext(c"E445: Other window contains changes".as_ptr()));
        }
    }
}
