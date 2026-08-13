//! `win_close()` -- closing one window.
//!
//! The re-entrant half of closing: free the window's buffer if nothing else
//! shows it, fire `WinClosed`, remove the frame and give its room to a
//! neighbour, pick the window to enter next, and cope with the fact that any
//! of those autocommands may have closed further windows or freed the buffer
//! in hand.  [`win_close_othertab`] is the same for a window that is not on
//! the current tab page, and cannot simply enter it to do the work.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_TABCLOSED, EVENT_TABCLOSEDPRE, EVENT_TABLEAVE,
    EVENT_WINCLOSED, EVENT_WINLEAVE, EVENT_WINNEWPRE, apply_autocmds, has_event, is_aucmd_win,
};
use crate::src::nvim::buffer::{
    bt_help, bt_quickfix, buf_hide, bufref_valid, close_buffer, set_bufref,
};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::diff::diffopt_closeoff;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    curbuf, curtab, curwin, e_autocmd_close, e_floatonly, first_tabpage, firstwin, getout, lastwin,
    p_ea, p_ead, p_ru, redraw_cmdline, redraw_tabline,
};
use crate::src::nvim::message::{emsg, internal_error};
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::terminal::terminal_check_size;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    CMD_SIZE, CMD_close, Integer, buf_T, bufref_T, frame_T, size_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::{ui_call_win_close, ui_has};
use crate::src::nvim::ui_compositor::ui_comp_remove_grid;
use crate::src::nvim::winfloat::win_float_find_altwin;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_close(
    mut win: *mut win_T,
    mut free_buf: bool,
    mut force: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut prev_curtab: *mut tabpage_T = curtab.get();
        let mut win_frame: *mut frame_T = if (*win).w_floating as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<frame_T>()
        } else {
            (*(*win).w_frame).fr_parent
        };
        let had_diffmode: bool = (*win).w_onebuf_opt.wo_diff != 0;
        if last_window(win) {
            emsg(gettext(e_cannot_close_last_window.as_ptr()));
            return FAIL;
        }
        if !(*win).w_floating && window_layout_locked(CMD_close) as ::core::ffi::c_int != 0 {
            return FAIL;
        }
        if win_locked(win) != 0
            || !(*win).w_buffer.is_null() && (*(*win).w_buffer).b_locked > 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if is_aucmd_win(win) {
            emsg(gettext(
                &raw const e_autocmd_close as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0
            && one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        {
            if is_aucmd_win(lastwin.get()) {
                emsg(gettext(
                    c"E814: Cannot close window, only autocmd window would remain".as_ptr(),
                ));
                return FAIL;
            }
            if force as ::core::ffi::c_int != 0
                || can_close_floating_windows(::core::ptr::null_mut::<tabpage_T>())
                    as ::core::ffi::c_int
                    != 0
            {
                while (*lastwin.get()).w_floating {
                    if win_close(
                        lastwin.get(),
                        !buf_hide((*lastwin.get()).w_buffer),
                        true_0 != 0,
                    ) == FAIL
                    {
                        return FAIL;
                    }
                }
                if !win_valid_any_tab(win) {
                    return FAIL;
                }
                if last_window(win) {
                    emsg(gettext(e_cannot_close_last_window.as_ptr()));
                    return FAIL;
                }
            } else {
                emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
                return FAIL;
            }
        }
        if close_last_window_tabpage(win, free_buf, prev_curtab) {
            return FAIL;
        }
        let mut help_window: bool = false_0 != 0;
        let mut quickfix_window: bool = false_0 != 0;
        if bt_help((*win).w_buffer) {
            help_window = true_0 != 0;
        } else {
            clear_snapshot(curtab.get(), SNAP_HELP_IDX);
        }
        if bt_quickfix((*win).w_buffer) {
            quickfix_window = true_0 != 0;
        } else {
            clear_snapshot(curtab.get(), SNAP_QUICKFIX_IDX);
        }
        let mut other_buffer: bool = false_0 != 0;
        if win == curwin.get() {
            leaving_window(curwin.get());
            let mut wp: *mut win_T = if (*win).w_floating as ::core::ffi::c_int != 0 {
                win_float_find_altwin(win, ::core::ptr::null::<tabpage_T>())
            } else {
                frame2win(win_altframe(win, ::core::ptr::null_mut::<tabpage_T>()))
            };
            if (*wp).w_buffer != curbuf.get() {
                reset_VIsual_and_resel();
                other_buffer = true_0 != 0;
                if !win_valid(win) {
                    return FAIL;
                }
                (*win).w_locked = true_0 != 0;
                apply_autocmds(
                    EVENT_BUFLEAVE,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
                if !win_valid(win) {
                    return FAIL;
                }
                (*win).w_locked = false_0 != 0;
                if last_window(win) {
                    return FAIL;
                }
            }
            (*win).w_locked = true_0 != 0;
            apply_autocmds(
                EVENT_WINLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if !win_valid(win) {
                return FAIL;
            }
            (*win).w_locked = false_0 != 0;
            if last_window(win) {
                return FAIL;
            }
            if aborting() {
                return FAIL;
            }
        }
        do_autocmd_winclosed(win);
        if !win_valid_any_tab(win) {
            return OK;
        }
        let mut bufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut bufref, (*win).w_buffer);
        let mut did_decrement: bool = win_close_buffer(
            win,
            if free_buf as ::core::ffi::c_int != 0 {
                DOBUF_UNLOAD as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            true_0 != 0,
        );
        if win_valid(win) as ::core::ffi::c_int != 0
            && (*win).w_buffer.is_null()
            && !(*win).w_floating
            && last_window(win) as ::core::ffi::c_int != 0
        {
            if (*curwin.get()).w_buffer.is_null() {
                (*curwin.get()).w_buffer = curbuf.get();
            }
            getout(0 as ::core::ffi::c_int);
        }
        if curtab.get() != prev_curtab
            && win_valid_any_tab(win) as ::core::ffi::c_int != 0
            && (*win).w_buffer.is_null()
        {
            win_close_othertab(win, false_0, prev_curtab, force);
            return FAIL;
        }
        if !win_valid(win) {
            return FAIL;
        }
        if one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
            && ((*first_tabpage.get()).tp_next.is_null()
                || (*lastwin.get()).w_floating as ::core::ffi::c_int != 0)
        {
            if !(*first_tabpage.get()).tp_next.is_null() {
                emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
            }
            win_unclose_buffer(win, &raw mut bufref, did_decrement);
            return FAIL;
        }
        if close_last_window_tabpage(win, free_buf, prev_curtab) {
            return FAIL;
        }
        (*split_disallowed.ptr()) += 1;
        let mut was_floating: bool = (*win).w_floating;
        if ui_has(kUIMultigrid) {
            ui_call_win_close((*win).w_grid_alloc.handle as Integer);
        }
        if (*win).w_floating {
            ui_comp_remove_grid(&raw mut (*win).w_grid_alloc);
            debug_assert!(!(*first_tabpage.ptr()).is_null(), "first_tabpage != NULL");
            if (*win).w_config.external {
                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tp.is_null() {
                    if tp != curtab.get() && (*tp).tp_curwin == win {
                        (*tp).tp_curwin = (*tp).tp_firstwin;
                    }
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
            }
        }
        set_bufref(&raw mut bufref, (*win).w_buffer);
        let mut had_cmdline_ruler: bool = p_ru.get() != 0
            && win == curwin.get()
            && (*win).w_status_height == 0 as ::core::ffi::c_int;
        let mut dir: ::core::ffi::c_int = 0;
        let mut wp_0: *mut win_T =
            win_free_mem(win, &raw mut dir, ::core::ptr::null_mut::<tabpage_T>());
        if help_window as ::core::ffi::c_int != 0 || quickfix_window as ::core::ffi::c_int != 0 {
            let mut prev_win: *mut win_T =
                get_snapshot_curwin(if help_window as ::core::ffi::c_int != 0 {
                    SNAP_HELP_IDX
                } else {
                    SNAP_QUICKFIX_IDX
                });
            if win_valid(prev_win) {
                wp_0 = prev_win;
            }
        }
        let mut close_curwin: bool = false_0 != 0;
        if win == curwin.get() {
            curwin.set(wp_0);
            if (*wp_0).w_onebuf_opt.wo_pvw != 0
                || bt_quickfix((*wp_0).w_buffer) as ::core::ffi::c_int != 0
            {
                loop {
                    if (*wp_0).w_next.is_null() {
                        wp_0 = firstwin.get();
                    } else {
                        wp_0 = (*wp_0).w_next;
                    }
                    if wp_0 == curwin.get() {
                        break;
                    }
                    if !((*wp_0).w_onebuf_opt.wo_pvw == 0
                        && !bt_quickfix((*wp_0).w_buffer)
                        && !((*wp_0).w_floating as ::core::ffi::c_int != 0
                            && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                || !(*wp_0).w_config.focusable)))
                    {
                        continue;
                    }
                    curwin.set(wp_0);
                    break;
                }
            }
            curbuf.set((*curwin.get()).w_buffer);
            close_curwin = true_0 != 0;
            check_cursor(curwin.get());
        }
        if !was_floating {
            last_status(false_0 != 0);
            if !(*curwin.get()).w_floating
                && p_ea.get() != 0
                && (*p_ead.get() as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                    || *p_ead.get() as ::core::ffi::c_int == dir)
            {
                win_equal(
                    curwin.get(),
                    (*(*curwin.get()).w_frame).fr_parent == win_frame,
                    dir,
                );
            } else {
                win_comp_pos();
                win_fix_scroll(false_0 != 0);
            }
        } else if had_cmdline_ruler as ::core::ffi::c_int != 0
            && (*wp_0).w_status_height > 0 as ::core::ffi::c_int
        {
            redraw_cmdline.set(true_0 != 0);
        }
        if !bufref.br_buf.is_null()
            && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
            && !(*bufref.br_buf).terminal.is_null()
        {
            terminal_check_size((*bufref.br_buf).terminal);
        }
        if close_curwin {
            win_enter_ext(
                wp_0,
                WEE_CURWIN_INVALID as ::core::ffi::c_int
                    | WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
                    | WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int,
            );
            if other_buffer {
                apply_autocmds(
                    EVENT_BUFENTER,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
            }
        }
        if firstwin.get() == lastwin.get()
            && (*curwin.get()).w_locked as ::core::ffi::c_int != 0
            && (*curbuf.get()).b_locked_split != 0
            && !(*first_tabpage.get()).tp_next.is_null()
        {
            apply_autocmds(
                EVENT_TABLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        (*split_disallowed.ptr()) -= 1;
        if help_window as ::core::ffi::c_int != 0 || quickfix_window as ::core::ffi::c_int != 0 {
            restore_snapshot(
                if help_window as ::core::ffi::c_int != 0 {
                    SNAP_HELP_IDX
                } else {
                    SNAP_QUICKFIX_IDX
                },
                close_curwin as ::core::ffi::c_int,
            );
        }
        if diffopt_closeoff() as ::core::ffi::c_int != 0
            && had_diffmode as ::core::ffi::c_int != 0
            && curtab.get() == prev_curtab
        {
            let mut diffcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut dwin: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !dwin.is_null() {
                if (*dwin).w_onebuf_opt.wo_diff != 0 {
                    diffcount += 1;
                }
                dwin = (*dwin).w_next;
            }
            if diffcount == 1 as ::core::ffi::c_int {
                do_cmdline_cmd(c"diffoff!".as_ptr());
            }
        }
        (*curwin.get()).w_pos_changed = true_0 != 0;
        if !was_floating {
            redraw_all_later(UPD_NOT_VALID);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn trigger_winnewpre() {
    unsafe {
        window_layout_lock();
        apply_autocmds(
            EVENT_WINNEWPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<buf_T>(),
        );
        window_layout_unlock();
    }
}

unsafe extern "C" fn do_autocmd_winclosed(mut win: *mut win_T) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if recursive.get() as ::core::ffi::c_int != 0 || !has_event(EVENT_WINCLOSED) {
            return;
        }
        recursive.set(true_0 != 0);
        let mut winid: [::core::ffi::c_char; 65] = [0; 65];
        vim_snprintf(
            &raw mut winid as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
            c"%d".as_ptr(),
            (*win).handle,
        );
        apply_autocmds(
            EVENT_WINCLOSED,
            &raw mut winid as *mut ::core::ffi::c_char,
            &raw mut winid as *mut ::core::ffi::c_char,
            false_0 != 0,
            (*win).w_buffer,
        );
        recursive.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn trigger_tabclosedpre(mut tp: *mut tabpage_T) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut ptp: *mut tabpage_T = curtab.get();
        if !has_event(EVENT_TABCLOSEDPRE) || recursive.get() as ::core::ffi::c_int != 0 {
            return;
        }
        if valid_tabpage(tp) {
            goto_tabpage_tp(tp, false_0 != 0, false_0 != 0);
        }
        recursive.set(true_0 != 0);
        window_layout_lock();
        apply_autocmds(
            EVENT_TABCLOSEDPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<buf_T>(),
        );
        window_layout_unlock();
        recursive.set(false_0 != 0);
        if valid_tabpage(ptp) {
            goto_tabpage_tp(ptp, false_0 != 0, false_0 != 0);
        } else {
            goto_tabpage_tp(first_tabpage.get(), false_0 != 0, false_0 != 0);
        };
    }
}

pub unsafe extern "C" fn win_close_othertab(
    mut win: *mut win_T,
    mut free_buf: ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut force: bool,
) -> bool {
    unsafe {
        let mut bufref: bufref_T = bufref_T::default();
        let mut free_tp_idx: ::core::ffi::c_int = 0;
        let mut dir: ::core::ffi::c_int = 0;
        debug_assert!(tp != curtab.get(), "tp != curtab");
        let mut did_decrement: bool = false_0 != 0;
        if window_layout_locked(CMD_SIZE) {
            return false_0 != 0;
        }
        if win_locked(win) != 0
            || !(*win).w_buffer.is_null() && (*(*win).w_buffer).b_locked > 0 as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        if is_aucmd_win(win) {
            emsg(gettext(
                &raw const e_autocmd_close as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        '_leave_open: {
            if (*(*tp).tp_lastwin).w_floating as ::core::ffi::c_int != 0
                && one_window(win, tp) as ::core::ffi::c_int != 0
            {
                if force as ::core::ffi::c_int != 0
                    || can_close_floating_windows(tp) as ::core::ffi::c_int != 0
                {
                    // Not immutable: win_close_othertab() updates tp_lastwin behind the raw pointer.
                    #[allow(clippy::while_immutable_condition)]
                    while (*(*tp).tp_lastwin).w_floating {
                        if !win_close_othertab(
                            (*tp).tp_lastwin,
                            !buf_hide((*(*tp).tp_lastwin).w_buffer) as ::core::ffi::c_int,
                            tp,
                            true_0 != 0,
                        ) {
                            break '_leave_open;
                        }
                    }
                    if !win_valid_any_tab(win) {
                        return false_0 != 0;
                    }
                } else {
                    emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
                    break '_leave_open;
                }
            }
            if !(*win).w_buffer.is_null() {
                do_autocmd_winclosed(win);
                if !win_valid_any_tab(win) {
                    return false_0 != 0;
                }
            }
            if (*tp).tp_firstwin == (*tp).tp_lastwin && !(*tp).tp_did_tabclosedpre {
                trigger_tabclosedpre(tp);
                if !win_valid_any_tab(win) {
                    return false_0 != 0;
                }
            }
            bufref = bufref_T::default();
            set_bufref(&raw mut bufref, (*win).w_buffer);
            if !(*win).w_buffer.is_null() {
                did_decrement = close_buffer(
                    win,
                    (*win).w_buffer,
                    if free_buf != 0 {
                        DOBUF_UNLOAD as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    false_0 != 0,
                    true_0 != 0,
                );
            }
            if !(!valid_tabpage(tp) || tp == curtab.get()) {
                if tabpage_win_valid(tp, win) {
                    if (*(*tp).tp_lastwin).w_floating as ::core::ffi::c_int != 0
                        && one_window(win, tp) as ::core::ffi::c_int != 0
                    {
                        emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
                    } else {
                        free_tp_idx = 0 as ::core::ffi::c_int;
                        if (*tp).tp_firstwin == (*tp).tp_lastwin {
                            free_tp_idx = tabpage_index(tp);
                            let mut h: ::core::ffi::c_int = tabline_height();
                            if tp == first_tabpage.get() {
                                first_tabpage.set((*tp).tp_next);
                            } else {
                                let mut ptp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
                                ptp = first_tabpage.get();
                                while !ptp.is_null() && (*ptp).tp_next != tp {
                                    ptp = (*ptp).tp_next;
                                }
                                if ptp.is_null() {
                                    internal_error(c"win_close_othertab()".as_ptr());
                                    return false_0 != 0;
                                }
                                (*ptp).tp_next = (*tp).tp_next;
                            }
                            redraw_tabline.set(true_0 != 0);
                            if h != tabline_height() {
                                win_new_screen_rows();
                            }
                        }
                        set_bufref(&raw mut bufref, (*win).w_buffer);
                        dir = 0;
                        win_free_mem(win, &raw mut dir, tp);
                        if !bufref.br_buf.is_null()
                            && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                            && !(*bufref.br_buf).terminal.is_null()
                        {
                            terminal_check_size((*bufref.br_buf).terminal);
                        }
                        if free_tp_idx > 0 as ::core::ffi::c_int {
                            free_tabpage(tp);
                            if has_event(EVENT_TABCLOSED) {
                                let mut prev_idx: [::core::ffi::c_char; 65] = [0; 65];
                                vim_snprintf(
                                    &raw mut prev_idx as *mut ::core::ffi::c_char,
                                    NUMBUFLEN as ::core::ffi::c_int as size_t,
                                    c"%i".as_ptr(),
                                    free_tp_idx,
                                );
                                apply_autocmds(
                                    EVENT_TABCLOSED,
                                    &raw mut prev_idx as *mut ::core::ffi::c_char,
                                    &raw mut prev_idx as *mut ::core::ffi::c_char,
                                    false_0 != 0,
                                    if !bufref.br_buf.is_null()
                                        && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                                    {
                                        bufref.br_buf
                                    } else {
                                        curbuf.get()
                                    },
                                );
                            }
                        }
                        return true_0 != 0;
                    }
                }
            }
        }
        if win_valid_any_tab(win) {
            win_unclose_buffer(win, &raw mut bufref, did_decrement);
        }
        return false_0 != 0;
    }
}
