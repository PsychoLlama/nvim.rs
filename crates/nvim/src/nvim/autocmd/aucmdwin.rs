//! The autocommand window: running a command "in" a buffer.
//!
//! `aucmd_prepbuf` makes `buf` current for the duration of an autocommand
//! -- entering a real window if one already shows the buffer, and otherwise
//! borrowing the hidden autocommand window and pointing it at the buffer --
//! and `aucmd_restbuf` puts everything back, which is the harder half: the
//! command may have closed windows, changed buffers or deleted the very
//! buffer it was given.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn is_aucmd_win(mut win: *mut win_T) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int {
            if (*(*aucmd_win_vec.ptr()).items.offset(i as isize)).auc_win_used as ::core::ffi::c_int
                != 0
                && (*(*aucmd_win_vec.ptr()).items.offset(i as isize)).auc_win == win
            {
                return true_0 != 0;
            }
            i += 1;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn aucmd_prepbuf(mut aco: *mut aco_save_T, mut buf: *mut buf_T) {
    unsafe {
        let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut need_append: bool = true_0 != 0;
        let same_buffer: bool = buf == curbuf.get();
        if same_buffer {
            win = curwin.get();
        } else {
            win = ::core::ptr::null_mut::<win_T>();
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    win = wp;
                    break;
                } else {
                    wp = (*wp).w_next;
                }
            }
        }
        let mut auc_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut auc_idx: ::core::ffi::c_int = (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int;
        if win.is_null() {
            auc_idx = 0 as ::core::ffi::c_int;
            while auc_idx < (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int {
                if !(*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize)).auc_win_used {
                    break;
                }
                auc_idx += 1;
            }
            if auc_idx == (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int {
                if (*aucmd_win_vec.ptr()).size == (*aucmd_win_vec.ptr()).capacity {
                    (*aucmd_win_vec.ptr()).capacity = if (*aucmd_win_vec.ptr()).capacity != 0 {
                        (*aucmd_win_vec.ptr()).capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*aucmd_win_vec.ptr()).items = xrealloc(
                        (*aucmd_win_vec.ptr()).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<aucmdwin_T>()
                            .wrapping_mul((*aucmd_win_vec.ptr()).capacity),
                    ) as *mut aucmdwin_T;
                } else {
                };
                let c2rust_fresh12 = (*aucmd_win_vec.ptr()).size;
                (*aucmd_win_vec.ptr()).size = (*aucmd_win_vec.ptr()).size.wrapping_add(1);
                *(*aucmd_win_vec.ptr()).items.offset(c2rust_fresh12 as isize) = aucmdwin_T {
                    auc_win: ::core::ptr::null_mut::<win_T>(),
                    auc_win_used: false,
                };
            }
            if (*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize))
                .auc_win
                .is_null()
            {
                win_alloc_aucmd_win(auc_idx);
                need_append = false_0 != 0;
            }
            auc_win = (*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize)).auc_win;
            (*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize)).auc_win_used = true_0 != 0;
        }
        (*aco).save_curwin_handle = (*curwin.get()).handle;
        (*aco).save_prevwin_handle = (if (*prevwin.ptr()).is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*prevwin.get()).handle as ::core::ffi::c_int
        }) as handle_T;
        if bt_prompt(curbuf.get()) {
            (*aco).save_prompt_insert = (*curbuf.get()).b_prompt_insert;
        }
        if !win.is_null() {
            (*aco).use_aucmd_win_idx = -1 as ::core::ffi::c_int;
            curwin.set(win);
        } else {
            (*aco).use_aucmd_win_idx = auc_idx;
            (*auc_win).w_buffer = buf;
            (*auc_win).w_s = &raw mut (*buf).b_s;
            (*buf).b_nwindows += 1;
            win_init_empty(auc_win);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*auc_win).w_localdir as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            (*aco).tp_localdir = (*curtab.get()).tp_localdir;
            (*curtab.get()).tp_localdir = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*aco).globaldir = globaldir.get();
            globaldir.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            block_autocmds();
            if need_append {
                win_append(lastwin.get(), auc_win, ::core::ptr::null_mut::<tabpage_T>());
                map_put_int_ptr_t(
                    window_handles.ptr(),
                    (*auc_win).handle as ::core::ffi::c_int,
                    auc_win as ptr_t,
                );
                win_config_float(auc_win, (*auc_win).w_config);
            }
            let save_acd: ::core::ffi::c_int = p_acd.get();
            p_acd.set(false_0);
            (*RedrawingDisabled.ptr()) += 1;
            win_enter(auc_win, false_0 != 0);
            (*RedrawingDisabled.ptr()) -= 1;
            p_acd.set(save_acd);
            unblock_autocmds();
            curwin.set(auc_win);
        }
        curbuf.set(buf);
        (*aco).new_curwin_handle = (*curwin.get()).handle;
        set_bufref(&raw mut (*aco).new_curbuf, curbuf.get());
        (*aco).save_VIsual_active = VIsual_active.get();
        if !same_buffer {
            VIsual_active.set(false_0 != 0);
        }
    }
}

pub unsafe extern "C" fn aucmd_restbuf(mut aco: *mut aco_save_T) {
    unsafe {
        if (*aco).use_aucmd_win_idx >= 0 as ::core::ffi::c_int {
            let mut awp: *mut win_T = (*(*aucmd_win_vec.ptr())
                .items
                .offset((*aco).use_aucmd_win_idx as isize))
            .auc_win;
            block_autocmds();
            '_win_found: {
                if curwin.get() != awp {
                    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                    loop {
                        if tp.is_null() {
                            break '_win_found;
                        }
                        let mut wp: *mut win_T = if tp == curtab.get() {
                            firstwin.get()
                        } else {
                            (*tp).tp_firstwin
                        };
                        while !wp.is_null() {
                            if wp == awp {
                                if tp != curtab.get() {
                                    goto_tabpage_tp(tp as *mut tabpage_T, true_0 != 0, true_0 != 0);
                                }
                                win_goto(awp);
                                break '_win_found;
                            } else {
                                wp = (*wp).w_next;
                            }
                        }
                        tp = (*tp).tp_next as *mut tabpage_T;
                    }
                }
            }
            (*curbuf.get()).b_nwindows -= 1;
            win_remove(curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
            map_del_int_ptr_t(
                window_handles.ptr(),
                (*curwin.get()).handle as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if !(*curwin.get()).w_grid_alloc.chars.is_null() {
                ui_comp_remove_grid(&raw mut (*curwin.get()).w_grid_alloc);
                ui_call_win_hide((*curwin.get()).w_grid_alloc.handle as Integer);
                grid_free(&raw mut (*curwin.get()).w_grid_alloc);
            }
            (*(*aucmd_win_vec.ptr())
                .items
                .offset((*aco).use_aucmd_win_idx as isize))
            .auc_win_used = false_0 != 0;
            if valid_tabpage_win(curtab.get()) == 0 {
                close_tabpage(curtab.get());
            }
            unblock_autocmds();
            let save_curwin: *mut win_T = win_find_by_handle((*aco).save_curwin_handle);
            if !save_curwin.is_null() {
                curwin.set(save_curwin);
            } else {
                curwin.set(firstwin.get());
            }
            curbuf.set((*curwin.get()).w_buffer);
            entering_window(curwin.get());
            if bt_prompt(curbuf.get()) {
                (*curbuf.get()).b_prompt_insert = (*aco).save_prompt_insert;
            }
            prevwin.set(win_find_by_handle((*aco).save_prevwin_handle));
            vars_clear(&raw mut (*(*awp).w_vars).dv_hashtab);
            hash_init(&raw mut (*(*awp).w_vars).dv_hashtab);
            if !(*awp).w_localdir.is_null() {
                win_fix_current_dir();
            }
            xfree((*curtab.get()).tp_localdir as *mut ::core::ffi::c_void);
            (*curtab.get()).tp_localdir = (*aco).tp_localdir;
            xfree(globaldir.get() as *mut ::core::ffi::c_void);
            globaldir.set((*aco).globaldir);
            VIsual_active.set((*aco).save_VIsual_active);
            check_cursor(curwin.get());
            if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
                (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
            }
        } else {
            let save_curwin_0: *mut win_T = win_find_by_handle((*aco).save_curwin_handle);
            if !save_curwin_0.is_null() {
                if (*curwin.get()).handle == (*aco).new_curwin_handle
                    && curbuf.get() != (*aco).new_curbuf.br_buf
                    && bufref_valid(&raw mut (*aco).new_curbuf) as ::core::ffi::c_int != 0
                    && !(*(*aco).new_curbuf.br_buf).b_ml.ml_mfp.is_null()
                {
                    if (*curwin.get()).w_s == &raw mut (*curbuf.get()).b_s {
                        (*curwin.get()).w_s = &raw mut (*(*aco).new_curbuf.br_buf).b_s;
                    }
                    (*curbuf.get()).b_nwindows -= 1;
                    curbuf.set((*aco).new_curbuf.br_buf);
                    (*curwin.get()).w_buffer = curbuf.get();
                    (*curbuf.get()).b_nwindows += 1;
                }
                curwin.set(save_curwin_0);
                curbuf.set((*curwin.get()).w_buffer);
                prevwin.set(win_find_by_handle((*aco).save_prevwin_handle));
                VIsual_active.set((*aco).save_VIsual_active);
                check_cursor(curwin.get());
            }
        }
        VIsual_active.set((*aco).save_VIsual_active);
        check_cursor(curwin.get());
        if VIsual_active.get() {
            check_pos(curbuf.get(), VIsual.ptr());
        }
    }
}
