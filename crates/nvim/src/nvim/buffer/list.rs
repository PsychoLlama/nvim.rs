//! The buffer list -- creating an entry and finding one.
//!
//! [`buflist_new`] is the only way a buffer joins the list: reuse an existing
//! entry for the same file if there is one, otherwise allocate, assign the
//! next buffer number, copy the option defaults and fire `BufNew`/`BufAdd`.
//! [`buflist_findpat`] is the search the command line uses -- the four-attempt
//! match over full names, then tails, then patterns -- and the
//! `buflist_findname*` group the exact-name lookups.  [`buflist_getfile`]
//! switches to an entry and puts the cursor where it was.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::autocmd::{EVENT_BUFADD, EVENT_BUFNEW, apply_autocmds};
use crate::src::nvim::cursor::{check_cursor_col, check_cursor_lnum};
use crate::src::nvim::diff::diff_mode_buf;
use crate::src::nvim::digraph::keymap_ga_clear;
use crate::src::nvim::eval::typval::kCallbackNone;
use crate::src::nvim::eval::typval::{callback_free, tv_dict_alloc};
use crate::src::nvim::eval::vars::init_var_dict;
use crate::src::nvim::ex_cmds::getfile;
use crate::src::nvim::ex_docmd::tabpage_new;
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::text_or_buf_locked;
use crate::src::nvim::fileio::file_pat_to_reg_pat;
use crate::src::nvim::garray::ga_clear;
use crate::src::nvim::hashtab::hash_init;
use crate::src::nvim::insexpand::clear_cpt_callbacks;
use crate::src::nvim::main::{
    RedrawingDisabled, buffer_handles, curbuf, curtab, curwin, e_buffer_nr_not_found, e_noalt,
    emsg_silent, firstbuf, firstwin, in_assert_fails, jop_flags, lastbuf, p_sol, swb_flags,
};
use crate::src::nvim::mark::{clrallmarks, fmarks_check_names, mark_view_restore};
use crate::src::nvim::memory::{xcalloc, xfree, xrealloc, xstrdup};
use crate::src::nvim::message::{emsg, msg_delay};
use crate::src::nvim::option::{buf_copy_options, magic_isset};
use crate::src::nvim::options::{
    kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagSplit, kOptSwbFlagVsplit,
};
use crate::src::nvim::optionstr::clear_string_option;
use crate::src::nvim::os::fs::os_fileid;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::path::FullName_save;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::src::nvim::types::{
    AdditionalData, BufUpdateCallbacks, FileID, OptInt, Timestamp, VAR_SCOPE, WinInfo, buf_T,
    bufref_T, colnr_T, fmark_T, fmarkv_T, handle_T, int16_t, linenr_T, pos_T, ptr_t, regmatch_T,
    regprog_T, size_t, uint64_t, win_T,
};
use crate::src::nvim::undo::curbufIsChanged;
use crate::src::nvim::window::{WSP_VERT, swbuf_goto_win_with_buf, win_split};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buflist_new(
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> *mut buf_T {
    unsafe {
        let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
        let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        fname_expand(curbuf.get(), &raw mut ffname, &raw mut sfname);
        let mut file_id: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        let mut file_id_valid: bool =
            !sfname.is_null() && os_fileid(sfname, &raw mut file_id) as ::core::ffi::c_int != 0;
        if !ffname.is_null()
            && flags & (BLN_DUMMY as ::core::ffi::c_int | BLN_NEW as ::core::ffi::c_int) == 0
            && {
                buf = buflist_findname_file_id(ffname, &raw mut file_id, file_id_valid);
                !buf.is_null()
            }
        {
            xfree(ffname as *mut ::core::ffi::c_void);
            if lnum != 0 as linenr_T {
                buflist_setfpos(
                    buf,
                    if flags & BLN_NOCURWIN as ::core::ffi::c_int != 0 {
                        ::core::ptr::null_mut::<win_T>()
                    } else {
                        curwin.get()
                    },
                    lnum,
                    0 as colnr_T,
                    false_0 != 0,
                );
            }
            if flags & BLN_NOOPT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                buf_copy_options(buf, 0 as ::core::ffi::c_int);
            }
            if flags & BLN_LISTED as ::core::ffi::c_int != 0 && (*buf).b_p_bl == 0 {
                (*buf).b_p_bl = true_0;
                let mut bufref: bufref_T = bufref_T::default();
                set_bufref(&raw mut bufref, buf);
                if flags & BLN_DUMMY as ::core::ffi::c_int == 0 {
                    if apply_autocmds(
                        EVENT_BUFADD,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                        buf,
                    ) as ::core::ffi::c_int
                        != 0
                        && !bufref_valid(&raw mut bufref)
                    {
                        return ::core::ptr::null_mut::<buf_T>();
                    }
                }
            }
            return buf;
        }
        buf = ::core::ptr::null_mut::<buf_T>();
        if flags & BLN_CURBUF as ::core::ffi::c_int != 0
            && curbuf_reusable() as ::core::ffi::c_int != 0
        {
            let mut bufref_0: bufref_T = bufref_T::default();
            debug_assert!(!(*curbuf.ptr()).is_null(), "curbuf != NULL");
            buf = curbuf.get();
            set_bufref(&raw mut bufref_0, buf);
            trigger_undo_ftplugin(buf, curwin.get());
            buf_freeall(
                buf,
                BFA_WIPE as ::core::ffi::c_int | BFA_DEL as ::core::ffi::c_int,
            );
            if aborting() {
                xfree(ffname as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<buf_T>();
            }
            if !bufref_valid(&raw mut bufref_0) {
                buf = ::core::ptr::null_mut::<buf_T>();
            }
        }
        if buf != curbuf.get() || (*curbuf.ptr()).is_null() {
            buf = xcalloc(1 as size_t, ::core::mem::size_of::<buf_T>()) as *mut buf_T;
            (*buf).b_vars = tv_dict_alloc();
            init_var_dict((*buf).b_vars, &raw mut (*buf).b_bufvar, VAR_SCOPE);
            buf_init_changedtick(buf);
        }
        if !ffname.is_null() {
            (*buf).b_ffname = ffname;
            (*buf).b_sfname = xstrdup(sfname);
        }
        clear_wininfo(buf);
        let mut curwin_info: *mut WinInfo =
            xcalloc(1 as size_t, ::core::mem::size_of::<WinInfo>()) as *mut WinInfo;
        if (*buf).b_wininfo.size == (*buf).b_wininfo.capacity {
            (*buf).b_wininfo.capacity = if (*buf).b_wininfo.capacity != 0 {
                (*buf).b_wininfo.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*buf).b_wininfo.items = xrealloc(
                (*buf).b_wininfo.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<*mut WinInfo>().wrapping_mul((*buf).b_wininfo.capacity),
            ) as *mut *mut WinInfo;
        } else {
        };
        let c2rust_fresh0 = (*buf).b_wininfo.size;
        (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *(*buf).b_wininfo.items.add(c2rust_fresh0);
        *c2rust_lvalue_ptr = curwin_info;
        if buf == curbuf.get() {
            free_buffer_stuff(buf, kBffInitChangedtick as ::core::ffi::c_int);
            (*buf).b_p_initialized = false_0 != 0;
            buf_copy_options(buf, BCO_ENTER as ::core::ffi::c_int);
            (*curbuf.get()).b_kmap_state =
                ((*curbuf.get()).b_kmap_state as ::core::ffi::c_int | KEYMAP_INIT) as int16_t;
        } else {
            (*buf).b_next = ::core::ptr::null_mut::<buf_T>();
            if (*firstbuf.ptr()).is_null() {
                (*buf).b_prev = ::core::ptr::null_mut::<buf_T>();
                firstbuf.set(buf);
            } else {
                (*lastbuf.get()).b_next = buf;
                (*buf).b_prev = lastbuf.get();
            }
            lastbuf.set(buf);
            let c2rust_fresh1 = top_file_num.get();
            top_file_num.set(top_file_num.get() + 1);
            (*buf).handle = c2rust_fresh1 as handle_T;
            map_put_int_ptr_t(
                buffer_handles.ptr(),
                (*buf).handle as ::core::ffi::c_int,
                buf as ptr_t,
            );
            if top_file_num.get() < 0 as ::core::ffi::c_int {
                emsg(gettext(
                    c"W14: Warning: List of file names overflow".as_ptr(),
                ));
                if emsg_silent.get() == 0 as ::core::ffi::c_int && !in_assert_fails.get() {
                    msg_delay(3001 as uint64_t, true_0 != 0);
                }
                top_file_num.set(1 as ::core::ffi::c_int);
            }
            buf_copy_options(buf, BCO_ALWAYS as ::core::ffi::c_int);
        }
        (*curwin_info).wi_mark = fmark_T {
            mark: pos_T {
                lnum: 0 as linenr_T,
                col: 0 as colnr_T,
                coladd: 0 as colnr_T,
            },
            fnum: 0 as ::core::ffi::c_int,
            timestamp: 0 as Timestamp,
            view: fmarkv_T {
                topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                skipcol: 0 as colnr_T,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        (*curwin_info).wi_mark.mark.lnum = lnum;
        (*curwin_info).wi_win = curwin.get();
        hash_init(&raw mut (*buf).b_s.b_keywtab);
        hash_init(&raw mut (*buf).b_s.b_keywtab_ic);
        (*buf).b_fname = (*buf).b_sfname;
        if !file_id_valid {
            (*buf).file_id_valid = false_0 != 0;
        } else {
            (*buf).file_id_valid = true_0 != 0;
            (*buf).file_id = file_id;
        }
        (*buf).b_u_synced = true_0 != 0;
        (*buf).b_flags = BF_CHECK_RO | BF_NEVERLOADED;
        if flags & BLN_DUMMY as ::core::ffi::c_int != 0 {
            (*buf).b_flags |= BF_DUMMY;
        }
        buf_clear_file(buf);
        clrallmarks(buf, 0 as Timestamp);
        fmarks_check_names(buf);
        (*buf).b_p_bl = if flags & BLN_LISTED as ::core::ffi::c_int != 0 {
            true_0
        } else {
            false_0
        };
        xfree((*buf).update_channels.items as *mut ::core::ffi::c_void);
        (*buf).update_channels.capacity = 0 as size_t;
        (*buf).update_channels.size = (*buf).update_channels.capacity;
        (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
        (*buf).update_channels.capacity = 0 as size_t;
        (*buf).update_channels.size = (*buf).update_channels.capacity;
        (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
        xfree((*buf).update_callbacks.items as *mut ::core::ffi::c_void);
        (*buf).update_callbacks.capacity = 0 as size_t;
        (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
        (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
        (*buf).update_callbacks.capacity = 0 as size_t;
        (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
        (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
        if flags & BLN_DUMMY as ::core::ffi::c_int == 0 {
            let mut bufref_1: bufref_T = bufref_T::default();
            set_bufref(&raw mut bufref_1, buf);
            if apply_autocmds(
                EVENT_BUFNEW,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
                && !bufref_valid(&raw mut bufref_1)
            {
                return ::core::ptr::null_mut::<buf_T>();
            }
            if flags & BLN_LISTED as ::core::ffi::c_int != 0
                && apply_autocmds(
                    EVENT_BUFADD,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    buf,
                ) as ::core::ffi::c_int
                    != 0
                && !bufref_valid(&raw mut bufref_1)
            {
                return ::core::ptr::null_mut::<buf_T>();
            }
            if aborting() {
                return ::core::ptr::null_mut::<buf_T>();
            }
        }
        (*buf).b_prompt_callback.type_0 = kCallbackNone;
        (*buf).b_prompt_interrupt.type_0 = kCallbackNone;
        (*buf).b_prompt_text = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*buf).b_prompt_start = fmark_T {
            mark: pos_T {
                lnum: 0 as linenr_T,
                col: 0 as colnr_T,
                coladd: 0 as colnr_T,
            },
            fnum: 0 as ::core::ffi::c_int,
            timestamp: 0 as Timestamp,
            view: fmarkv_T {
                topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                skipcol: 0 as colnr_T,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        (*buf).b_prompt_start.mark.col = 2 as ::core::ffi::c_int as colnr_T;
        (*buf).b_prompt_append_new_line = true_0 != 0;
        return buf;
    }
}

pub unsafe extern "C" fn curbuf_reusable() -> bool {
    unsafe {
        return !(*curbuf.ptr()).is_null()
            && (*curbuf.get()).b_ffname.is_null()
            && (*curbuf.get()).b_nwindows <= 1 as ::core::ffi::c_int
            && (*curbuf.get()).terminal.is_null()
            && ((*curbuf.get()).b_ml.ml_mfp.is_null()
                || buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0)
            && !bt_quickfix(curbuf.get())
            && !curbufIsChanged();
    }
}

pub unsafe extern "C" fn free_buf_options(mut buf: *mut buf_T, mut free_p_ff: bool) {
    unsafe {
        if free_p_ff {
            clear_string_option(&raw mut (*buf).b_p_fenc);
            clear_string_option(&raw mut (*buf).b_p_ff);
            clear_string_option(&raw mut (*buf).b_p_bh);
            clear_string_option(&raw mut (*buf).b_p_bt);
        }
        clear_string_option(&raw mut (*buf).b_p_def);
        clear_string_option(&raw mut (*buf).b_p_inc);
        clear_string_option(&raw mut (*buf).b_p_inex);
        clear_string_option(&raw mut (*buf).b_p_inde);
        clear_string_option(&raw mut (*buf).b_p_indk);
        clear_string_option(&raw mut (*buf).b_p_fp);
        clear_string_option(&raw mut (*buf).b_p_fex);
        clear_string_option(&raw mut (*buf).b_p_kp);
        clear_string_option(&raw mut (*buf).b_p_mps);
        clear_string_option(&raw mut (*buf).b_p_fo);
        clear_string_option(&raw mut (*buf).b_p_flp);
        clear_string_option(&raw mut (*buf).b_p_isk);
        clear_string_option(&raw mut (*buf).b_p_vsts);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_p_vsts_nopaste as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_p_vsts_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        clear_string_option(&raw mut (*buf).b_p_vts);
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_p_vts_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL_0;
        let _ = *ptr__1;
        clear_string_option(&raw mut (*buf).b_p_keymap);
        keymap_ga_clear(&raw mut (*buf).b_kmap_ga);
        ga_clear(&raw mut (*buf).b_kmap_ga);
        clear_string_option(&raw mut (*buf).b_p_com);
        clear_string_option(&raw mut (*buf).b_p_cms);
        clear_string_option(&raw mut (*buf).b_p_nf);
        clear_string_option(&raw mut (*buf).b_p_syn);
        clear_string_option(&raw mut (*buf).b_s.b_syn_isk);
        clear_string_option(&raw mut (*buf).b_s.b_p_spc);
        clear_string_option(&raw mut (*buf).b_s.b_p_spf);
        vim_regfree((*buf).b_s.b_cap_prog);
        (*buf).b_s.b_cap_prog = ::core::ptr::null_mut::<regprog_T>();
        clear_string_option(&raw mut (*buf).b_s.b_p_spl);
        clear_string_option(&raw mut (*buf).b_s.b_p_spo);
        clear_string_option(&raw mut (*buf).b_p_sua);
        clear_string_option(&raw mut (*buf).b_p_ft);
        clear_string_option(&raw mut (*buf).b_p_cink);
        clear_string_option(&raw mut (*buf).b_p_cino);
        clear_string_option(&raw mut (*buf).b_p_lop);
        clear_string_option(&raw mut (*buf).b_p_cinsd);
        clear_string_option(&raw mut (*buf).b_p_cinw);
        clear_string_option(&raw mut (*buf).b_p_cot);
        clear_string_option(&raw mut (*buf).b_p_cpt);
        clear_string_option(&raw mut (*buf).b_p_cfu);
        callback_free(&raw mut (*buf).b_cfu_cb);
        clear_string_option(&raw mut (*buf).b_p_ofu);
        callback_free(&raw mut (*buf).b_ofu_cb);
        clear_string_option(&raw mut (*buf).b_p_tsrfu);
        callback_free(&raw mut (*buf).b_tsrfu_cb);
        clear_cpt_callbacks(&raw mut (*buf).b_p_cpt_cb, (*buf).b_p_cpt_count);
        (*buf).b_p_cpt_count = 0 as ::core::ffi::c_int;
        clear_string_option(&raw mut (*buf).b_p_gefm);
        clear_string_option(&raw mut (*buf).b_p_gp);
        clear_string_option(&raw mut (*buf).b_p_mp);
        clear_string_option(&raw mut (*buf).b_p_efm);
        clear_string_option(&raw mut (*buf).b_p_ep);
        clear_string_option(&raw mut (*buf).b_p_path);
        clear_string_option(&raw mut (*buf).b_p_tags);
        clear_string_option(&raw mut (*buf).b_p_tc);
        clear_string_option(&raw mut (*buf).b_p_tfu);
        callback_free(&raw mut (*buf).b_tfu_cb);
        clear_string_option(&raw mut (*buf).b_p_ffu);
        callback_free(&raw mut (*buf).b_ffu_cb);
        clear_string_option(&raw mut (*buf).b_p_dict);
        clear_string_option(&raw mut (*buf).b_p_dia);
        clear_string_option(&raw mut (*buf).b_p_tsr);
        clear_string_option(&raw mut (*buf).b_p_qe);
        (*buf).b_p_ac = -1 as ::core::ffi::c_int;
        (*buf).b_p_ar = -1 as ::core::ffi::c_int;
        (*buf).b_p_fs = -1 as ::core::ffi::c_int;
        (*buf).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
        clear_string_option(&raw mut (*buf).b_p_lw);
        clear_string_option(&raw mut (*buf).b_p_bkc);
        clear_string_option(&raw mut (*buf).b_p_menc);
    }
}

pub unsafe extern "C" fn buflist_getfile(
    mut n: ::core::ffi::c_int,
    mut lnum: linenr_T,
    mut options: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
        let mut buf: *mut buf_T = buflist_findnr(n);
        if buf.is_null() {
            if options & GETF_ALT as ::core::ffi::c_int != 0 && n == 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_noalt as *const ::core::ffi::c_char));
            } else {
                semsg_c!(
                    gettext(&raw const e_buffer_nr_not_found as *const ::core::ffi::c_char),
                    n,
                );
            }
            return FAIL;
        }
        if buf == curbuf.get() {
            return OK;
        }
        if text_or_buf_locked() {
            return FAIL;
        }
        let mut col: colnr_T = 0;
        let mut restore_view: bool = false_0 != 0;
        if lnum == 0 as linenr_T {
            fm = buflist_findfmark(buf);
            lnum = (*fm).mark.lnum;
            col = (*fm).mark.col;
            restore_view = true_0 != 0;
        } else {
            col = 0 as ::core::ffi::c_int as colnr_T;
        }
        if options & GETF_SWITCH as ::core::ffi::c_int != 0 {
            wp = swbuf_goto_win_with_buf(buf);
            if wp.is_null()
                && swb_flags.get()
                    & (kOptSwbFlagVsplit as ::core::ffi::c_int
                        | kOptSwbFlagSplit as ::core::ffi::c_int
                        | kOptSwbFlagNewtab as ::core::ffi::c_int)
                        as ::core::ffi::c_uint
                    != 0
                && !buf_is_empty(curbuf.get())
            {
                if swb_flags.get() & kOptSwbFlagNewtab as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    tabpage_new();
                } else if win_split(
                    0 as ::core::ffi::c_int,
                    if swb_flags.get()
                        & kOptSwbFlagVsplit as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        WSP_VERT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                ) == FAIL
                {
                    return FAIL;
                }
                (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
            }
        }
        (*RedrawingDisabled.ptr()) += 1;
        if getfile(
            (*buf).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            options & GETF_SETMARK as ::core::ffi::c_int != 0,
            lnum,
            forceit != 0,
        ) <= 0 as ::core::ffi::c_int
        {
            (*RedrawingDisabled.ptr()) -= 1;
            if p_sol.get() == 0 && col != 0 as ::core::ffi::c_int {
                (*curwin.get()).w_cursor.col = col;
                check_cursor_col(curwin.get());
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                (*curwin.get()).w_set_curswant = true_0;
            }
            if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                && restore_view as ::core::ffi::c_int != 0
            {
                mark_view_restore(fm);
            }
            return OK;
        }
        (*RedrawingDisabled.ptr()) -= 1;
        return FAIL;
    }
}

pub(crate) unsafe extern "C" fn buflist_getfpos() {
    unsafe {
        let mut fm: *mut fmark_T = buflist_findfmark(curbuf.get());
        let mut fpos: *const pos_T = &raw mut (*fm).mark;
        (*curwin.get()).w_cursor.lnum = (*fpos).lnum;
        check_cursor_lnum(curwin.get());
        if p_sol.get() != 0 {
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            (*curwin.get()).w_cursor.col = (*fpos).col;
            check_cursor_col(curwin.get());
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_set_curswant = true_0;
        }
        if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            mark_view_restore(fm);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buflist_findname_exp(mut fname: *mut ::core::ffi::c_char) -> *mut buf_T {
    unsafe {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut ffname: *mut ::core::ffi::c_char = FullName_save(fname, true_0 != 0);
        if !ffname.is_null() {
            buf = buflist_findname(ffname);
            xfree(ffname as *mut ::core::ffi::c_void);
        }
        return buf;
    }
}

pub unsafe extern "C" fn buflist_findname(mut ffname: *mut ::core::ffi::c_char) -> *mut buf_T {
    unsafe {
        let mut file_id: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        let mut file_id_valid: bool = os_fileid(ffname, &raw mut file_id);
        return buflist_findname_file_id(ffname, &raw mut file_id, file_id_valid);
    }
}

pub(crate) unsafe extern "C" fn buflist_findname_file_id(
    mut ffname: *mut ::core::ffi::c_char,
    mut file_id: *mut FileID,
    mut file_id_valid: bool,
) -> *mut buf_T {
    unsafe {
        let mut buf: *mut buf_T = lastbuf.get();
        while !buf.is_null() {
            if (*buf).b_flags & BF_DUMMY == 0 as ::core::ffi::c_int
                && !otherfile_buf(buf, ffname, file_id, file_id_valid)
            {
                return buf;
            }
            buf = (*buf).b_prev;
        }
        return ::core::ptr::null_mut::<buf_T>();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buflist_findpat(
    mut pattern: *const ::core::ffi::c_char,
    mut pattern_end: *const ::core::ffi::c_char,
    mut unlisted: bool,
    mut diffmode: bool,
    mut curtab_only: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut match_0: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if pattern_end == pattern.offset(1 as ::core::ffi::c_int as isize)
            && (*pattern as ::core::ffi::c_int == '%' as ::core::ffi::c_int
                || *pattern as ::core::ffi::c_int == '#' as ::core::ffi::c_int)
        {
            match_0 = if *pattern as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                (*curbuf.get()).handle as ::core::ffi::c_int
            } else {
                (*curwin.get()).w_alt_fnum
            };
            let mut found_buf: *mut buf_T = buflist_findnr(match_0);
            if diffmode as ::core::ffi::c_int != 0
                && !(!found_buf.is_null() && diff_mode_buf(found_buf) as ::core::ffi::c_int != 0)
            {
                match_0 = -1 as ::core::ffi::c_int;
            }
        } else {
            let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
                pattern,
                pattern_end,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0,
            );
            if pat.is_null() {
                return -1 as ::core::ffi::c_int;
            }
            let mut patend: *mut ::core::ffi::c_char = pat
                .add(strlen(pat))
                .offset(-(1 as ::core::ffi::c_int as isize));
            let mut toggledollar: bool =
                patend > pat && *patend as ::core::ffi::c_int == '$' as ::core::ffi::c_int;
            let mut find_listed: ::core::ffi::c_int = true_0;
            loop {
                let mut attempt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while attempt <= 3 as ::core::ffi::c_int {
                    if toggledollar {
                        *patend = (if attempt < 2 as ::core::ffi::c_int {
                            NUL
                        } else {
                            '$' as ::core::ffi::c_int
                        }) as ::core::ffi::c_char;
                    }
                    let mut p: *mut ::core::ffi::c_char = pat;
                    if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int
                        && attempt & 1 as ::core::ffi::c_int == 0
                    {
                        p = p.offset(1);
                    }
                    let mut regmatch: regmatch_T = regmatch_T {
                        regprog: ::core::ptr::null_mut::<regprog_T>(),
                        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                        rm_matchcol: 0,
                        rm_ic: false,
                    };
                    regmatch.regprog = vim_regcomp(
                        p,
                        if magic_isset() as ::core::ffi::c_int != 0 {
                            RE_MAGIC
                        } else {
                            0 as ::core::ffi::c_int
                        },
                    );
                    let mut buf: *mut buf_T = lastbuf.get();
                    's_171: while !buf.is_null() {
                        if regmatch.regprog.is_null() {
                            xfree(pat as *mut ::core::ffi::c_void);
                            return -1 as ::core::ffi::c_int;
                        }
                        's_92: {
                            if (*buf).b_p_bl == find_listed
                                && (!diffmode || diff_mode_buf(buf) as ::core::ffi::c_int != 0)
                                && !buflist_match(&raw mut regmatch, buf, false_0 != 0).is_null()
                            {
                                if curtab_only {
                                    let mut found_window: bool = false_0 != 0;
                                    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                                        firstwin.get()
                                    } else {
                                        (*curtab.get()).tp_firstwin
                                    };
                                    while !wp.is_null() {
                                        if (*wp).w_buffer == buf {
                                            found_window = true_0 != 0;
                                            break;
                                        } else {
                                            wp = (*wp).w_next;
                                        }
                                    }
                                    if !found_window {
                                        break 's_92;
                                    }
                                }
                                if match_0 >= 0 as ::core::ffi::c_int {
                                    match_0 = -2 as ::core::ffi::c_int;
                                    break 's_171;
                                } else {
                                    match_0 = (*buf).handle as ::core::ffi::c_int;
                                }
                            }
                        }
                        buf = (*buf).b_prev;
                    }
                    vim_regfree(regmatch.regprog);
                    if match_0 >= 0 as ::core::ffi::c_int {
                        break;
                    }
                    attempt += 1;
                }
                if !unlisted || find_listed == 0 || match_0 != -1 as ::core::ffi::c_int {
                    break;
                }
                find_listed = false_0;
            }
            xfree(pat as *mut ::core::ffi::c_void);
        }
        if match_0 == -2 as ::core::ffi::c_int {
            semsg_c!(
                gettext(c"E93: More than one match for %s".as_ptr()),
                pattern,
            );
        } else if match_0 < 0 as ::core::ffi::c_int {
            semsg_c!(gettext(c"E94: No matching buffer for %s".as_ptr()), pattern,);
        }
        return match_0;
    }
}

pub(crate) unsafe extern "C" fn buf_time_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf1: *mut buf_T = *(s1 as *mut *mut buf_T);
        let mut buf2: *mut buf_T = *(s2 as *mut *mut buf_T);
        if (*buf1).b_last_used == (*buf2).b_last_used {
            return 0 as ::core::ffi::c_int;
        }
        return if (*buf1).b_last_used > (*buf2).b_last_used {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
}
