//! Unloading, deleting and wiping a buffer -- `close_buffer()`.
//!
//! [`close_buffer`] is the one entry point for all three: fire
//! `BufUnload`/`BufDelete`/`BufWipeout`, free the memline, the undo tree, the
//! marks, the folds and the extmarks, and -- for a wipe -- unlink the buffer
//! from the list and free it.  Every one of those autocommands may have freed
//! the buffer in hand, which is why so much of this is written around
//! `bufref_T` re-validation.  [`buf_freeall`] is the loaded-state teardown
//! the reload path shares.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::autocmd::{
    EVENT_BUFDELETE, EVENT_BUFHIDDEN, EVENT_BUFUNLOAD, EVENT_BUFWINLEAVE, EVENT_BUFWIPEOUT,
    apply_autocmds, aubuflocal_remove, block_autocmds, unblock_autocmds,
};
use crate::src::nvim::buffer_updates::{buf_free_callbacks, buf_updates_unload};
use crate::src::nvim::change::{deleted_lines_mark, unchanged};
use crate::src::nvim::diff::{diff_buf_delete, diffopt_hiddenoff};
use crate::src::nvim::eval::typval::{callback_free, tv_dict_add, tv_dict_item_copy};
use crate::src::nvim::eval::vars::{unref_var_dict, vars_clear};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::extmark::extmark_free_all;
use crate::src::nvim::fold::clearFolding;
use crate::src::nvim::garray::ga_clear;
use crate::src::nvim::hashtab::{hash_find, hash_init, hash_remove};
use crate::src::nvim::main::{
    VIsual_active, au_pending_free_buf, autocmd_busy, buffer_handles, curbuf, curtab, curwin,
    e_auabort, exiting, first_tabpage, firstbuf, firstwin, lastbuf, updating_screen,
};
use crate::src::nvim::map::map_del_int_ptr_t;
use crate::src::nvim::mapping::map_clear_mode;
use crate::src::nvim::mark::{
    clear_fmark, free_fmark, mark_adjust_buf, mark_forget_file, set_last_cursor,
};
use crate::src::nvim::memline::{ml_close, ml_delete};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::emsg;
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::os::libc::{gettext, memset};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::state::MAP_ALL_MODES;
use crate::src::nvim::syntax::{reset_synblock, syntax_clear};
use crate::src::nvim::terminal::terminal_close;
use crate::src::nvim::types::{
    Timestamp, WinInfo, buf_T, bufref_T, dictitem_T, fmark_T, hashitem_T, linenr_T, memfile_T,
    size_t, tabpage_T, win_T,
};
use crate::src::nvim::undo::u_clearallandblockfree;
use crate::src::nvim::usercmd::uc_clear;
use crate::src::nvim::window::{free_wininfo, goto_tabpage_win, one_window, win_valid_any_tab};

pub(crate) unsafe extern "C" fn can_unload_buffer(mut buf: *mut buf_T) -> bool {
    unsafe {
        let mut can_unload: bool = (*buf).b_locked == 0;
        if can_unload as ::core::ffi::c_int != 0 && updating_screen.get() as ::core::ffi::c_int != 0
        {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    can_unload = false_0 != 0;
                    break;
                } else {
                    wp = (*wp).w_next;
                }
            }
        }
        if can_unload as ::core::ffi::c_int != 0 && (*buf).b_saving as ::core::ffi::c_int != 0 {
            can_unload = false_0 != 0;
        }
        if !can_unload {
            let mut fname: *mut ::core::ffi::c_char = if !(*buf).b_fname.is_null() {
                (*buf).b_fname
            } else {
                (*buf).b_ffname
            };
            semsg_c!(
                gettext(e_attempt_to_delete_buffer_that_is_in_use_str.as_ptr(),),
                if !fname.is_null() {
                    fname as *const ::core::ffi::c_char
                } else {
                    c"[No Name]".as_ptr()
                },
            );
        }
        return can_unload;
    }
}

pub unsafe extern "C" fn buf_close_terminal(mut buf: *mut buf_T) {
    unsafe {
        debug_assert!(!(*buf).terminal.is_null(), "buf->terminal");
        (*buf).b_locked += 1;
        terminal_close(&raw mut (*buf).terminal, -1 as ::core::ffi::c_int);
        (*buf).b_locked -= 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn close_buffer(
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut action: ::core::ffi::c_int,
    mut abort_if_last: bool,
    mut ignore_abort: bool,
) -> bool {
    unsafe {
        let mut unload_buf: bool = action != 0 as ::core::ffi::c_int;
        let mut del_buf: bool =
            action == DOBUF_DEL as ::core::ffi::c_int || action == DOBUF_WIPE as ::core::ffi::c_int;
        let mut wipe_buf: bool = action == DOBUF_WIPE as ::core::ffi::c_int;
        let mut is_curwin: bool = !(*curwin.ptr()).is_null() && (*curwin.get()).w_buffer == buf;
        let mut the_curwin: *mut win_T = curwin.get();
        let mut the_curtab: *mut tabpage_T = curtab.get();
        if (*buf).terminal.is_null() {
            if *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'd' as ::core::ffi::c_int
            {
                del_buf = true_0 != 0;
                unload_buf = true_0 != 0;
            } else if *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'w' as ::core::ffi::c_int
            {
                del_buf = true_0 != 0;
                unload_buf = true_0 != 0;
                wipe_buf = true_0 != 0;
            } else if *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'u' as ::core::ffi::c_int
            {
                unload_buf = true_0 != 0;
            }
        }
        if !(*buf).terminal.is_null()
            && (unload_buf as ::core::ffi::c_int != 0
                || del_buf as ::core::ffi::c_int != 0
                || wipe_buf as ::core::ffi::c_int != 0)
        {
            unload_buf = true_0 != 0;
            del_buf = true_0 != 0;
            wipe_buf = true_0 != 0;
        }
        if (del_buf as ::core::ffi::c_int != 0 || wipe_buf as ::core::ffi::c_int != 0)
            && !can_unload_buffer(buf)
        {
            return false_0 != 0;
        }
        if !win.is_null() && win_valid_any_tab(win) as ::core::ffi::c_int != 0 {
            if (*buf).b_nwindows == 1 as ::core::ffi::c_int {
                set_last_cursor(win);
            }
            buflist_setfpos(
                buf,
                win,
                if (*win).w_cursor.lnum == 1 as linenr_T {
                    0 as linenr_T
                } else {
                    (*win).w_cursor.lnum
                },
                (*win).w_cursor.col,
                true_0 != 0,
            );
        }
        let mut bufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut bufref, buf);
        if (*buf).b_nwindows == 1 as ::core::ffi::c_int {
            (*buf).b_locked += 1;
            (*buf).b_locked_split += 1;
            if apply_autocmds(
                EVENT_BUFWINLEAVE,
                (*buf).b_fname,
                (*buf).b_fname,
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
                && !bufref_valid(&raw mut bufref)
            {
                emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
                return false_0 != 0;
            }
            (*buf).b_locked -= 1;
            (*buf).b_locked_split -= 1;
            if abort_if_last as ::core::ffi::c_int != 0
                && !win.is_null()
                && one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
            {
                emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
                return false_0 != 0;
            }
            if !unload_buf {
                (*buf).b_locked += 1;
                (*buf).b_locked_split += 1;
                if apply_autocmds(
                    EVENT_BUFHIDDEN,
                    (*buf).b_fname,
                    (*buf).b_fname,
                    false_0 != 0,
                    buf,
                ) as ::core::ffi::c_int
                    != 0
                    && !bufref_valid(&raw mut bufref)
                {
                    emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
                    return false_0 != 0;
                }
                (*buf).b_locked -= 1;
                (*buf).b_locked_split -= 1;
                if abort_if_last as ::core::ffi::c_int != 0
                    && !win.is_null()
                    && one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int
                        != 0
                {
                    emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
                    return false_0 != 0;
                }
            }
            if !ignore_abort && aborting() as ::core::ffi::c_int != 0 {
                return false_0 != 0;
            }
        }
        if is_curwin as ::core::ffi::c_int != 0
            && curwin.get() != the_curwin
            && win_valid_any_tab(the_curwin) as ::core::ffi::c_int != 0
        {
            block_autocmds();
            goto_tabpage_win(the_curtab, the_curwin);
            unblock_autocmds();
        }
        let mut nwindows: ::core::ffi::c_int = (*buf).b_nwindows;
        if (*buf).b_nwindows > 0 as ::core::ffi::c_int {
            (*buf).b_nwindows -= 1;
        }
        if diffopt_hiddenoff() as ::core::ffi::c_int != 0
            && !unload_buf
            && (*buf).b_nwindows == 0 as ::core::ffi::c_int
        {
            diff_buf_delete(buf);
        }
        if (*buf).b_nwindows > 0 as ::core::ffi::c_int || !unload_buf {
            return true_0 != 0;
        }
        if (*buf).b_ffname.is_null() {
            del_buf = true_0 != 0;
        }
        let mut is_curbuf: bool = buf == curbuf.get();
        if is_curbuf as ::core::ffi::c_int != 0 && VIsual_active.get() as ::core::ffi::c_int != 0 {
            end_visual_mode();
        }
        (*buf).b_nwindows = nwindows;
        buf_freeall(
            buf,
            (if del_buf as ::core::ffi::c_int != 0 {
                BFA_DEL as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if wipe_buf as ::core::ffi::c_int != 0 {
                BFA_WIPE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if ignore_abort as ::core::ffi::c_int != 0 {
                BFA_IGNORE_ABORT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
        );
        if !bufref_valid(&raw mut bufref) {
            return false_0 != 0;
        }
        if !ignore_abort && aborting() as ::core::ffi::c_int != 0 {
            return false_0 != 0;
        }
        if buf == curbuf.get() && !is_curbuf {
            return false_0 != 0;
        }
        let mut clear_w_buf: bool = false_0 != 0;
        if !win.is_null()
            && win_valid_any_tab(win) as ::core::ffi::c_int != 0
            && (*win).w_buffer == buf
        {
            clear_w_buf = true_0 != 0;
        }
        if nwindows > 0 as ::core::ffi::c_int && (*buf).b_nwindows > 0 as ::core::ffi::c_int {
            (*buf).b_nwindows -= 1;
        }
        if wipe_buf as ::core::ffi::c_int != 0
            && (*buf).b_nwindows <= 0 as ::core::ffi::c_int
            && (!(*buf).b_prev.is_null() || !(*buf).b_next.is_null())
        {
            if clear_w_buf {
                (*win).w_buffer = ::core::ptr::null_mut::<buf_T>();
            }
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut wp: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    mark_forget_file(wp, (*buf).handle as ::core::ffi::c_int);
                    wp = (*wp).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            if (*buf).b_sfname != (*buf).b_ffname {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            } else {
                (*buf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_ffname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
            if (*buf).b_prev.is_null() {
                firstbuf.set((*buf).b_next);
            } else {
                (*(*buf).b_prev).b_next = (*buf).b_next;
            }
            if (*buf).b_next.is_null() {
                lastbuf.set((*buf).b_prev);
            } else {
                (*(*buf).b_next).b_prev = (*buf).b_prev;
            }
            free_buffer(buf);
        } else {
            if del_buf {
                free_buffer_stuff(
                    buf,
                    kBffClearWinInfo as ::core::ffi::c_int
                        | kBffInitChangedtick as ::core::ffi::c_int,
                );
                (*buf).b_flags = BF_CHECK_RO | BF_NEVERLOADED;
                (*buf).b_p_initialized = false_0 != 0;
            }
            buf_clear_file(buf);
            if clear_w_buf {
                (*win).w_buffer = ::core::ptr::null_mut::<buf_T>();
            }
            if del_buf {
                (*buf).b_p_bl = false_0;
            }
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn buf_clear_file(mut buf: *mut buf_T) {
    unsafe {
        (*buf).b_ml.ml_line_count = 1 as ::core::ffi::c_int as linenr_T;
        unchanged(buf, true_0 != 0, true_0 != 0);
        (*buf).b_p_eof = false_0;
        (*buf).b_start_eof = false_0;
        (*buf).b_p_eol = true_0;
        (*buf).b_start_eol = true_0;
        (*buf).b_p_bomb = false_0;
        (*buf).b_start_bomb = false_0;
        (*buf).b_ml.ml_mfp = ::core::ptr::null_mut::<memfile_T>();
        (*buf).b_ml.ml_flags = ML_EMPTY;
    }
}

pub unsafe extern "C" fn buf_clear() {
    unsafe {
        let mut line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
        extmark_free_all(curbuf.get());
        while (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 {
            ml_delete(1 as linenr_T);
        }
        deleted_lines_mark(1 as linenr_T, line_count as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn buf_freeall(mut buf: *mut buf_T, mut flags: ::core::ffi::c_int) {
    unsafe {
        let mut is_curbuf: bool = buf == curbuf.get();
        let mut is_curwin: ::core::ffi::c_int =
            (!(*curwin.ptr()).is_null() && (*curwin.get()).w_buffer == buf) as ::core::ffi::c_int;
        let mut the_curwin: *mut win_T = curwin.get();
        let mut the_curtab: *mut tabpage_T = curtab.get();
        (*buf).b_locked += 1;
        (*buf).b_locked_split += 1;
        let mut bufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut bufref, buf);
        if !(*buf).terminal.is_null() {
            buf_close_terminal(buf);
        }
        buf_updates_unload(buf, false_0 != 0);
        if !(*buf).b_ml.ml_mfp.is_null()
            && apply_autocmds(
                EVENT_BUFUNLOAD,
                (*buf).b_fname,
                (*buf).b_fname,
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
            && !bufref_valid(&raw mut bufref)
        {
            return;
        }
        if flags & BFA_DEL as ::core::ffi::c_int != 0
            && (*buf).b_p_bl != 0
            && apply_autocmds(
                EVENT_BUFDELETE,
                (*buf).b_fname,
                (*buf).b_fname,
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
            && !bufref_valid(&raw mut bufref)
        {
            return;
        }
        if flags & BFA_WIPE as ::core::ffi::c_int != 0
            && apply_autocmds(
                EVENT_BUFWIPEOUT,
                (*buf).b_fname,
                (*buf).b_fname,
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
            && !bufref_valid(&raw mut bufref)
        {
            return;
        }
        (*buf).b_locked -= 1;
        (*buf).b_locked_split -= 1;
        if is_curwin != 0
            && curwin.get() != the_curwin
            && win_valid_any_tab(the_curwin) as ::core::ffi::c_int != 0
        {
            block_autocmds();
            goto_tabpage_win(the_curtab, the_curwin);
            unblock_autocmds();
        }
        if flags & BFA_IGNORE_ABORT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && aborting() as ::core::ffi::c_int != 0
        {
            return;
        }
        if buf == curbuf.get() && !is_curbuf {
            return;
        }
        diff_buf_delete(buf);
        if !(*curwin.ptr()).is_null() && (*curwin.get()).w_buffer == buf {
            reset_synblock(curwin.get());
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut win: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !win.is_null() {
                if (*win).w_buffer == buf {
                    clearFolding(win);
                }
                win = (*win).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        if !(*buf).terminal.is_null() {
            block_autocmds();
            buf_close_terminal(buf);
            unblock_autocmds();
        }
        let mut count: linenr_T = (*buf).b_ml.ml_line_count;
        ml_close(buf, true_0);
        (*buf).b_ml.ml_line_count = 0 as ::core::ffi::c_int as linenr_T;
        if bt_nofilename(buf) as ::core::ffi::c_int != 0 && !exiting.get() {
            mark_adjust_buf(
                buf,
                1 as linenr_T,
                count,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                -count,
                false_0 != 0,
                kMarkAdjustNormal,
                kExtmarkNoUndo,
            );
        }
        if flags & BFA_KEEP_UNDO as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            u_clearallandblockfree(buf);
        }
        syntax_clear(&raw mut (*buf).b_s);
        (*buf).b_flags &= !BF_READERR;
    }
}

unsafe extern "C" fn free_buffer(mut buf: *mut buf_T) {
    unsafe {
        map_del_int_ptr_t(
            buffer_handles.ptr(),
            (*buf).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        (*buf_free_count.ptr()) += 1;
        free_buffer_stuff(buf, kBffClearWinInfo as ::core::ffi::c_int);
        if (*(*buf).b_vars).dv_refcount > DO_NOT_FREE_CNT as ::core::ffi::c_int {
            tv_dict_add(
                (*buf).b_vars,
                tv_dict_item_copy(&raw mut (*buf).changedtick_di as *mut dictitem_T),
            );
        }
        unref_var_dict((*buf).b_vars);
        aubuflocal_remove(buf);
        xfree((*buf).additional_data as *mut ::core::ffi::c_void);
        xfree((*buf).b_prompt_text as *mut ::core::ffi::c_void);
        xfree((*buf).b_wininfo.items as *mut ::core::ffi::c_void);
        (*buf).b_wininfo.capacity = 0 as size_t;
        (*buf).b_wininfo.size = (*buf).b_wininfo.capacity;
        (*buf).b_wininfo.items = ::core::ptr::null_mut::<*mut WinInfo>();
        callback_free(&raw mut (*buf).b_prompt_callback);
        callback_free(&raw mut (*buf).b_prompt_interrupt);
        clear_fmark(&raw mut (*buf).b_last_cursor, 0 as Timestamp);
        clear_fmark(&raw mut (*buf).b_last_insert, 0 as Timestamp);
        clear_fmark(&raw mut (*buf).b_last_change, 0 as Timestamp);
        clear_fmark(&raw mut (*buf).b_prompt_start, 0 as Timestamp);
        let mut i: size_t = 0 as size_t;
        while i < NMARKS as size_t {
            free_fmark((*buf).b_namedm[i as usize]);
            i = i.wrapping_add(1);
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*buf).b_changelistlen {
            free_fmark((*buf).b_changelist[i_0 as usize]);
            i_0 += 1;
        }
        if autocmd_busy.get() {
            memset(
                &raw mut (*buf).b_namedm as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[fmark_T; 26]>(),
            );
            memset(
                &raw mut (*buf).b_changelist as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[fmark_T; 100]>(),
            );
            (*buf).b_next = au_pending_free_buf.get();
            au_pending_free_buf.set(buf);
        } else {
            xfree(buf as *mut ::core::ffi::c_void);
            if curbuf.get() == buf {
                curbuf.set(::core::ptr::null_mut::<buf_T>());
            }
        };
    }
}

pub(crate) unsafe extern "C" fn clear_wininfo(mut buf: *mut buf_T) {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < (*buf).b_wininfo.size {
            free_wininfo(*(*buf).b_wininfo.items.add(i));
            i = i.wrapping_add(1);
        }
        (*buf).b_wininfo.size = 0 as size_t;
    }
}

pub(crate) unsafe extern "C" fn free_buffer_stuff(
    mut buf: *mut buf_T,
    mut free_flags: ::core::ffi::c_int,
) {
    unsafe {
        if free_flags & kBffClearWinInfo as ::core::ffi::c_int != 0 {
            clear_wininfo(buf);
            free_buf_options(buf, true_0 != 0);
            ga_clear(&raw mut (*buf).b_s.b_langp);
        }
        let changedtick_hi: *mut hashitem_T = hash_find(
            &raw mut (*(*buf).b_vars).dv_hashtab,
            c"changedtick".as_ptr(),
        );
        debug_assert!(!changedtick_hi.is_null(), "changedtick_hi != NULL");
        hash_remove(&raw mut (*(*buf).b_vars).dv_hashtab, changedtick_hi);
        vars_clear(&raw mut (*(*buf).b_vars).dv_hashtab);
        hash_init(&raw mut (*(*buf).b_vars).dv_hashtab);
        if free_flags & kBffInitChangedtick as ::core::ffi::c_int != 0 {
            buf_init_changedtick(buf);
        }
        uc_clear(&raw mut (*buf).b_ucmds);
        extmark_free_all(buf);
        map_clear_mode(buf, MAP_ALL_MODES, true_0 != 0, false_0 != 0);
        map_clear_mode(buf, MAP_ALL_MODES, true_0 != 0, true_0 != 0);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_start_fenc as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        buf_free_callbacks(buf);
    }
}

pub unsafe extern "C" fn wipe_buffer(mut buf: *mut buf_T, mut aucmd: bool) {
    unsafe {
        if !aucmd {
            block_autocmds();
        }
        close_buffer(
            ::core::ptr::null_mut::<win_T>(),
            buf,
            DOBUF_WIPE as ::core::ffi::c_int,
            false_0 != 0,
            true_0 != 0,
        );
        if !aucmd {
            unblock_autocmds();
        }
    }
}
