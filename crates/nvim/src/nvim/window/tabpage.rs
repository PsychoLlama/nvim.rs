//! Tab pages -- creating one, switching to it, and closing it.
//!
//! [`win_new_tabpage`] takes the current window out of the layout and gives
//! it a tab page of its own; [`leave_tabpage`] and [`enter_tabpage`] save and
//! restore the whole window layout around a switch, which is what makes a tab
//! page a layout rather than a list of windows.  [`goto_tabpage`] and
//! [`goto_tabpage_tp`] are the entry points, [`tabpage_move`] reorders them,
//! and [`valid_tabpage`]/[`find_tabpage`]/[`tabpage_index`] are the lookups
//! the rest of the editor asks.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_TABENTER, EVENT_TABLEAVE, EVENT_TABNEW,
    EVENT_TABNEWENTERED, EVENT_WINENTER, EVENT_WINLEAVE, EVENT_WINNEW, apply_autocmds,
    block_autocmds, unblock_autocmds,
};
use crate::src::nvim::diff::diff_clear;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::eval::typval::tv_dict_alloc;
use crate::src::nvim::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::src::nvim::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::src::nvim::ex_getln::{text_locked, text_locked_msg};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_init;
use crate::src::nvim::main::{
    Columns, Rows, cmdmod, cmdwin_type, curbuf, curtab, curwin, diff_need_scrollbind, e_cmdwin,
    first_tabpage, firstwin, lastused_tabpage, lastwin, p_ch, p_tpm, postponed_split_tab, prevwin,
    redraw_tabline, skip_win_fix_scroll, starting, tabpage_handles, tabpage_move_disallowed,
    topframe,
};
use crate::src::nvim::map::map_del_int_ptr_t;
use crate::src::nvim::memory::{xcalloc, xfree, xstrdup};
use crate::src::nvim::message::{emsg, set_keep_msg};
use crate::src::nvim::mouse::reset_dragwin;
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::option::set_option_value;
use crate::src::nvim::options::kOptCmdheight;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::terminal::terminal_check_size;
use crate::src::nvim::types::{
    CMD_tabnew, OptInt, OptVal, OptValData, VAR_SCOPE, buf_T, handle_T, int64_t, ptr_t, size_t,
    switchwin_T, tabpage_T, win_T,
};
use crate::src::nvim::ui_compositor::ui_comp_remove_grid;
use crate::src::nvim::winfloat::{win_config_float, win_float_update_statusline};

pub unsafe extern "C" fn unuse_tabpage(mut tp: *mut tabpage_T) {
    unsafe {
        (*tp).tp_topframe = topframe.get();
        (*tp).tp_firstwin = firstwin.get();
        (*tp).tp_lastwin = lastwin.get();
        (*tp).tp_curwin = curwin.get();
    }
}

pub unsafe extern "C" fn use_tabpage(mut tp: *mut tabpage_T) {
    unsafe {
        curtab.set(tp);
        topframe.set((*curtab.get()).tp_topframe);
        firstwin.set((*curtab.get()).tp_firstwin);
        lastwin.set((*curtab.get()).tp_lastwin);
        curwin.set((*curtab.get()).tp_curwin);
    }
}

pub(crate) unsafe extern "C" fn alloc_tabpage() -> *mut tabpage_T {
    unsafe {
        static last_tp_handle: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        let mut tp: *mut tabpage_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<tabpage_T>()) as *mut tabpage_T;
        (*last_tp_handle.ptr()) += 1;
        (*tp).handle = last_tp_handle.get() as handle_T;
        map_put_int_ptr_t(
            tabpage_handles.ptr(),
            (*tp).handle as ::core::ffi::c_int,
            tp as ptr_t,
        );
        (*tp).tp_vars = tv_dict_alloc();
        init_var_dict((*tp).tp_vars, &raw mut (*tp).tp_winvar, VAR_SCOPE);
        (*tp).tp_diff_invalid = true_0;
        (*tp).tp_ch_used = p_ch.get();
        return tp;
    }
}

pub unsafe extern "C" fn free_tabpage(mut tp: *mut tabpage_T) {
    unsafe {
        map_del_int_ptr_t(
            tabpage_handles.ptr(),
            (*tp).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        diff_clear(tp);
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < SNAP_COUNT {
            clear_snapshot(tp, idx);
            idx += 1;
        }
        vars_clear(&raw mut (*(*tp).tp_vars).dv_hashtab);
        hash_init(&raw mut (*(*tp).tp_vars).dv_hashtab);
        unref_var_dict((*tp).tp_vars);
        if tp == lastused_tabpage.get() {
            lastused_tabpage.set(::core::ptr::null_mut::<tabpage_T>());
        }
        xfree((*tp).tp_localdir as *mut ::core::ffi::c_void);
        xfree((*tp).tp_prevdir as *mut ::core::ffi::c_void);
        xfree(tp as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn win_new_tabpage(
    mut after: ::core::ffi::c_int,
    mut filename: *mut ::core::ffi::c_char,
    mut enter: bool,
    mut first: *mut *mut win_T,
) -> *mut tabpage_T {
    unsafe {
        let mut old_curtab: *mut tabpage_T = curtab.get();
        if enter as ::core::ffi::c_int != 0 && cmdwin_type.get() != 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
            return ::core::ptr::null_mut::<tabpage_T>();
        }
        if window_layout_locked(CMD_tabnew) {
            return ::core::ptr::null_mut::<tabpage_T>();
        }
        let mut newtp: *mut tabpage_T = alloc_tabpage();
        if enter {
            if leave_tabpage(curbuf.get(), true_0 != 0) == FAIL {
                xfree(newtp as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<tabpage_T>();
            }
        } else {
            unuse_tabpage(curtab.get());
            (*curtab.get()).tp_old_Rows_avail = (Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt)
                as int64_t;
            firstwin.set(::core::ptr::null_mut::<win_T>());
            lastwin.set(::core::ptr::null_mut::<win_T>());
        }
        (*newtp).tp_localdir = if !(*old_curtab).tp_localdir.is_null() {
            xstrdup((*old_curtab).tp_localdir)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        curtab.set(newtp);
        let result: ::core::ffi::c_int = win_alloc_firstwin((*old_curtab).tp_curwin);
        debug_assert!(result == 1 as ::core::ffi::c_int, "result == OK");
        if !first.is_null() {
            *first = curwin.get();
        }
        if after == 1 as ::core::ffi::c_int {
            (*newtp).tp_next = first_tabpage.get();
            first_tabpage.set(newtp);
        } else {
            let mut tp: *mut tabpage_T = old_curtab;
            if after > 0 as ::core::ffi::c_int {
                let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                tp = first_tabpage.get();
                while !(*tp).tp_next.is_null() && n < after {
                    n += 1;
                    tp = (*tp).tp_next;
                }
            }
            (*newtp).tp_next = (*tp).tp_next;
            (*tp).tp_next = newtp;
        }
        (*newtp).tp_curwin = curwin.get();
        (*newtp).tp_lastwin = (*newtp).tp_curwin;
        (*newtp).tp_firstwin = (*newtp).tp_lastwin;
        win_init_size();
        (*firstwin.get()).w_winrow = tabline_height();
        (*firstwin.get()).w_prev_winrow = (*firstwin.get()).w_winrow;
        win_comp_scroll(curwin.get());
        (*newtp).tp_topframe = topframe.get();
        last_status(false_0 != 0);
        if !(*curbuf.get()).terminal.is_null() {
            terminal_check_size((*curbuf.get()).terminal);
        }
        if enter {
            redraw_all_later(UPD_NOT_VALID);
            tabpage_check_windows(old_curtab);
            lastused_tabpage.set(old_curtab);
            entering_window(curwin.get());
            apply_autocmds(
                EVENT_WINNEW,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            apply_autocmds(
                EVENT_WINENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            apply_autocmds(EVENT_TABNEW, filename, filename, false_0 != 0, curbuf.get());
            apply_autocmds(
                EVENT_TABENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        } else {
            unuse_tabpage(curtab.get());
            use_tabpage(old_curtab);
            redraw_tabline.set(true_0 != 0);
            if (*curtab.get()).tp_old_Rows_avail
                != Rows.get() as OptInt
                    - p_ch.get()
                    - tabline_height() as OptInt
                    - global_stl_height() as OptInt
            {
                win_new_screen_rows();
            }
            let mut switchwin: switchwin_T = switchwin_T {
                sw_curwin: ::core::ptr::null_mut::<win_T>(),
                sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                sw_same_win: false,
                sw_visual_active: false,
            };
            let sw_result: ::core::ffi::c_int =
                switch_win_noblock(&raw mut switchwin, (*newtp).tp_curwin, newtp, true_0 != 0);
            debug_assert!(sw_result == 1 as ::core::ffi::c_int, "sw_result == OK");
            apply_autocmds(
                EVENT_WINNEW,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            apply_autocmds(EVENT_TABNEW, filename, filename, false_0 != 0, curbuf.get());
            restore_win_noblock(&raw mut switchwin, true_0 != 0);
        }
        return newtp;
    }
}

pub(crate) unsafe extern "C" fn may_open_tabpage() -> ::core::ffi::c_int {
    unsafe {
        let mut n: ::core::ffi::c_int = if (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int {
            postponed_split_tab.get()
        } else {
            (*cmdmod.ptr()).cmod_tab
        };
        if n == 0 as ::core::ffi::c_int {
            return FAIL;
        }
        (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
        postponed_split_tab.set(0 as ::core::ffi::c_int);
        let mut status: ::core::ffi::c_int = if !win_new_tabpage(
            n,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            ::core::ptr::null_mut::<*mut win_T>(),
        )
        .is_null()
        {
            OK
        } else {
            FAIL
        };
        if status == OK {
            apply_autocmds(
                EVENT_TABNEWENTERED,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        return status;
    }
}

pub unsafe extern "C" fn make_tabpages(mut maxcount: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut count: ::core::ffi::c_int = maxcount;
        count = if count < p_tpm.get() as ::core::ffi::c_int {
            count
        } else {
            p_tpm.get() as ::core::ffi::c_int
        };
        block_autocmds();
        let mut todo: ::core::ffi::c_int = 0;
        todo = count - 1 as ::core::ffi::c_int;
        while todo > 0 as ::core::ffi::c_int {
            if win_new_tabpage(
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                true_0 != 0,
                ::core::ptr::null_mut::<*mut win_T>(),
            )
            .is_null()
            {
                break;
            }
            todo -= 1;
        }
        unblock_autocmds();
        return count - todo;
    }
}

pub unsafe extern "C" fn valid_tabpage(mut tpc: *mut tabpage_T) -> bool {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp == tpc {
                return true_0 != 0;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn valid_tabpage_win(mut tpc: *mut tabpage_T) -> ::core::ffi::c_int {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp == tpc {
                let mut wp: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    if win_valid_any_tab(wp) {
                        return true_0;
                    }
                    wp = (*wp).w_next;
                }
                return false_0;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return false_0;
    }
}

pub unsafe extern "C" fn close_tabpage(mut tab: *mut tabpage_T) {
    unsafe {
        let mut ptp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if tab == first_tabpage.get() {
            first_tabpage.set((*tab).tp_next);
            ptp = first_tabpage.get();
        } else {
            ptp = first_tabpage.get();
            while !ptp.is_null() && (*ptp).tp_next != tab {
                ptp = (*ptp).tp_next;
            }
            debug_assert!(!ptp.is_null(), "ptp != NULL");
            (*ptp).tp_next = (*tab).tp_next;
        }
        goto_tabpage_tp(ptp, false_0 != 0, false_0 != 0);
        free_tabpage(tab);
    }
}

pub unsafe extern "C" fn find_tabpage(mut n: ::core::ffi::c_int) -> *mut tabpage_T {
    unsafe {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if n == 0 as ::core::ffi::c_int {
            return curtab.get();
        }
        tp = first_tabpage.get();
        while !tp.is_null() && i != n {
            i += 1;
            tp = (*tp).tp_next;
        }
        return tp;
    }
}

pub unsafe extern "C" fn tabpage_index(mut ftp: *mut tabpage_T) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        tp = first_tabpage.get();
        while !tp.is_null() && tp != ftp {
            i += 1;
            tp = (*tp).tp_next;
        }
        return i;
    }
}

unsafe extern "C" fn leave_tabpage(
    mut new_curbuf: *mut buf_T,
    mut trigger_leave_autocmds: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut tp: *mut tabpage_T = curtab.get();
        leaving_window(curwin.get());
        reset_VIsual_and_resel();
        if trigger_leave_autocmds {
            if new_curbuf != curbuf.get() {
                apply_autocmds(
                    EVENT_BUFLEAVE,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
                if curtab.get() != tp {
                    return FAIL;
                }
            }
            apply_autocmds(
                EVENT_WINLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if curtab.get() != tp {
                return FAIL;
            }
            apply_autocmds(
                EVENT_TABLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if curtab.get() != tp {
                return FAIL;
            }
        }
        reset_dragwin();
        (*tp).tp_curwin = curwin.get();
        (*tp).tp_prevwin = prevwin.get();
        (*tp).tp_firstwin = firstwin.get();
        (*tp).tp_lastwin = lastwin.get();
        (*tp).tp_old_Rows_avail = (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt) as int64_t;
        if (*tp).tp_old_Columns != -1 as int64_t {
            (*tp).tp_old_Columns = Columns.get() as int64_t;
        }
        firstwin.set(::core::ptr::null_mut::<win_T>());
        lastwin.set(::core::ptr::null_mut::<win_T>());
        return OK;
    }
}

unsafe extern "C" fn enter_tabpage(
    mut tp: *mut tabpage_T,
    mut old_curbuf: *mut buf_T,
    mut trigger_enter_autocmds: bool,
    mut trigger_leave_autocmds: bool,
) {
    unsafe {
        let mut old_off: ::core::ffi::c_int = (*(*tp).tp_firstwin).w_winrow;
        let mut next_prevwin: *mut win_T = (*tp).tp_prevwin;
        let mut old_curtab: *mut tabpage_T = curtab.get();
        use_tabpage(tp);
        if old_curtab != curtab.get() {
            tabpage_check_windows(old_curtab);
            if p_ch.get() != (*curtab.get()).tp_ch_used {
                let mut new_ch: OptInt = (*curtab.get()).tp_ch_used;
                (*curtab.get()).tp_ch_used = p_ch.get();
                command_frame_height.set(false_0 != 0);
                set_option_value(
                    kOptCmdheight,
                    OptVal {
                        type_0: kOptValTypeNumber,
                        data: OptValData { number: new_ch },
                    },
                    0 as ::core::ffi::c_int,
                );
                command_frame_height.set(true_0 != 0);
            }
        }
        win_enter_ext(
            (*tp).tp_curwin,
            WEE_CURWIN_INVALID as ::core::ffi::c_int
                | (if trigger_enter_autocmds as ::core::ffi::c_int != 0 {
                    WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if trigger_leave_autocmds as ::core::ffi::c_int != 0 {
                    WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
        );
        prevwin.set(next_prevwin);
        last_status(false_0 != 0);
        win_float_update_statusline();
        win_comp_pos();
        diff_need_scrollbind.set(true_0 != 0);
        reset_dragwin();
        if (*curtab.get()).tp_old_Rows_avail
            != Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt
            || old_off != (*firstwin.get()).w_winrow
        {
            win_new_screen_rows();
        }
        if (*curtab.get()).tp_old_Columns != Columns.get() as int64_t {
            if starting.get() == 0 as ::core::ffi::c_int {
                win_new_screen_cols();
                (*curtab.get()).tp_old_Columns = Columns.get() as int64_t;
            } else {
                (*curtab.get()).tp_old_Columns = -1 as int64_t;
            }
        }
        lastused_tabpage.set(old_curtab);
        if trigger_enter_autocmds {
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
        }
        redraw_all_later(UPD_NOT_VALID);
    }
}

unsafe extern "C" fn tabpage_check_windows(mut old_curtab: *mut tabpage_T) {
    unsafe {
        let mut next_wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wp: *mut win_T = (*old_curtab).tp_firstwin;
        while !wp.is_null() {
            next_wp = (*wp).w_next;
            if (*wp).w_floating {
                if (*wp).w_config.external {
                    win_remove(wp, old_curtab);
                    win_append(
                        lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()),
                        wp,
                        ::core::ptr::null_mut::<tabpage_T>(),
                    );
                } else {
                    ui_comp_remove_grid(&raw mut (*wp).w_grid_alloc);
                }
            }
            (*wp).w_pos_changed = true_0 != 0;
            wp = next_wp;
        }
        let mut wp_0: *mut win_T = firstwin.get();
        while !wp_0.is_null() {
            if (*wp_0).w_floating as ::core::ffi::c_int != 0 && !(*wp_0).w_config.external {
                win_config_float(wp_0, (*wp_0).w_config);
            }
            (*wp_0).w_pos_changed = true_0 != 0;
            wp_0 = (*wp_0).w_next;
        }
    }
}

pub unsafe extern "C" fn goto_tabpage(mut n: ::core::ffi::c_int) {
    unsafe {
        if text_locked() {
            text_locked_msg();
            return;
        }
        if (*first_tabpage.get()).tp_next.is_null() {
            if n > 1 as ::core::ffi::c_int {
                beep_flush();
            }
            return;
        }
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if n == 0 as ::core::ffi::c_int {
            if (*curtab.get()).tp_next.is_null() {
                tp = first_tabpage.get();
            } else {
                tp = (*curtab.get()).tp_next;
            }
        } else if n < 0 as ::core::ffi::c_int {
            let mut ttp: *mut tabpage_T = curtab.get();
            let mut i: ::core::ffi::c_int = n;
            while i < 0 as ::core::ffi::c_int {
                tp = first_tabpage.get();
                while (*tp).tp_next != ttp && !(*tp).tp_next.is_null() {
                    tp = (*tp).tp_next;
                }
                ttp = tp;
                i += 1;
            }
        } else if n == 9999 as ::core::ffi::c_int {
            tp = first_tabpage.get();
            while !(*tp).tp_next.is_null() {
                tp = (*tp).tp_next;
            }
        } else {
            tp = find_tabpage(n);
            if tp.is_null() {
                beep_flush();
                return;
            }
        }
        goto_tabpage_tp(tp, true_0 != 0, true_0 != 0);
    }
}

pub unsafe extern "C" fn goto_tabpage_tp(
    mut tp: *mut tabpage_T,
    mut trigger_enter_autocmds: bool,
    mut trigger_leave_autocmds: bool,
) {
    unsafe {
        if trigger_enter_autocmds as ::core::ffi::c_int != 0
            || trigger_leave_autocmds as ::core::ffi::c_int != 0
        {
            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                return;
            }
        }
        set_keep_msg(
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
        );
        skip_win_fix_scroll.set(true_0 != 0);
        if tp != curtab.get()
            && leave_tabpage((*(*tp).tp_curwin).w_buffer, trigger_leave_autocmds) == OK
        {
            if valid_tabpage(tp) {
                enter_tabpage(
                    tp,
                    curbuf.get(),
                    trigger_enter_autocmds,
                    trigger_leave_autocmds,
                );
            } else {
                enter_tabpage(
                    curtab.get(),
                    curbuf.get(),
                    trigger_enter_autocmds,
                    trigger_leave_autocmds,
                );
            }
        }
        skip_win_fix_scroll.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn goto_tabpage_lastused() -> bool {
    unsafe {
        if !valid_tabpage(lastused_tabpage.get()) {
            return false_0 != 0;
        }
        goto_tabpage_tp(lastused_tabpage.get(), true_0 != 0, true_0 != 0);
        return true_0 != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn goto_tabpage_win(mut tp: *mut tabpage_T, mut wp: *mut win_T) {
    unsafe {
        goto_tabpage_tp(tp, true_0 != 0, true_0 != 0);
        if curtab.get() == tp && win_valid(wp) as ::core::ffi::c_int != 0 {
            win_enter(wp, true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn tabpage_move(mut nr: ::core::ffi::c_int) {
    unsafe {
        debug_assert!(!(*curtab.ptr()).is_null(), "curtab != NULL");
        if (*first_tabpage.get()).tp_next.is_null() {
            return;
        }
        if tabpage_move_disallowed.get() != 0 {
            return;
        }
        let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        tp = first_tabpage.get();
        while !(*tp).tp_next.is_null() && n < nr {
            n += 1;
            tp = (*tp).tp_next;
        }
        if tp == curtab.get()
            || nr > 0 as ::core::ffi::c_int
                && !(*tp).tp_next.is_null()
                && (*tp).tp_next == curtab.get()
        {
            return;
        }
        let mut tp_dst: *mut tabpage_T = tp;
        if curtab.get() == first_tabpage.get() {
            first_tabpage.set((*curtab.get()).tp_next);
        } else {
            tp = ::core::ptr::null_mut::<tabpage_T>();
            let mut tp2: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp2.is_null() {
                if (*tp2).tp_next == curtab.get() {
                    tp = tp2 as *mut tabpage_T;
                    break;
                } else {
                    tp2 = (*tp2).tp_next as *mut tabpage_T;
                }
            }
            if tp.is_null() {
                return;
            }
            (*tp).tp_next = (*curtab.get()).tp_next;
        }
        if nr <= 0 as ::core::ffi::c_int {
            (*curtab.get()).tp_next = first_tabpage.get();
            first_tabpage.set(curtab.get());
        } else {
            (*curtab.get()).tp_next = (*tp_dst).tp_next;
            (*tp_dst).tp_next = curtab.get();
        }
        redraw_tabline.set(true_0 != 0);
    }
}
