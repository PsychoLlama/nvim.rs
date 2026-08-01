//! Going to the position an entry names.
//!
//! [`qf_jump_newwin`] is the entry point: it picks a window
//! ([`qf_jump_open_window`] and the `qf_goto_*`/`qf_find_*` helpers, which
//! implement `'switchbuf'`), opens the buffer ([`qf_jump_edit_buffer`]),
//! moves the cursor ([`qf_jump_goto_line`]) and reports what it did
//! ([`qf_jump_print_msg`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn qf_find_help_win() -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if bt_help((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_config.hide
                && (*wp).w_config.focusable as ::core::ffi::c_int != 0
            {
                return wp;
            }
            wp = (*wp).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub(crate) unsafe extern "C" fn win_set_loclist(mut wp: *mut win_T, mut qi: *mut qf_info_T) {
    unsafe {
        (*wp).w_llist = qi;
        (*qi).qf_refcount += 1;
    }
}

pub(crate) unsafe extern "C" fn jump_to_help_window(
    mut qi: *mut qf_info_T,
    mut newwin: bool,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut win_T = if (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int
            || newwin as ::core::ffi::c_int != 0
        {
            ::core::ptr::null_mut::<win_T>()
        } else {
            qf_find_help_win()
        };
        if !wp.is_null() && (*(*wp).w_buffer).b_nwindows > 0 as ::core::ffi::c_int {
            win_enter(wp, true_0 != 0);
        } else {
            let mut flags: ::core::ffi::c_int = WSP_HELP as ::core::ffi::c_int;
            if (*cmdmod.ptr()).cmod_split == 0 as ::core::ffi::c_int
                && (*curwin.get()).w_width != Columns.get()
                && (*curwin.get()).w_width < 80 as ::core::ffi::c_int
            {
                flags |= WSP_TOP as ::core::ffi::c_int;
            }
            if (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
                && !newwin
            {
                flags |= WSP_NEWLOC as ::core::ffi::c_int;
            }
            if win_split(0 as ::core::ffi::c_int, flags) == FAIL {
                return FAIL;
            }
            *opened_window = true_0 != 0;
            if ((*curwin.get()).w_height as OptInt) < p_hh.get() {
                win_setheight(p_hh.get() as ::core::ffi::c_int);
            }
            if (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
                && !newwin
            {
                win_set_loclist(curwin.get(), qi);
            }
        }
        restart_edit.set(0 as ::core::ffi::c_int);
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_find_win_with_loclist(mut ll: *const qf_info_T) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_llist == ll as *mut qf_info_T && !bt_quickfix((*wp).w_buffer) {
                return wp;
            }
            wp = (*wp).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub(crate) unsafe extern "C" fn qf_find_win_with_normal_buf() -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if bt_normal((*wp).w_buffer) {
                return wp;
            }
            wp = (*wp).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub(crate) unsafe extern "C" fn qf_goto_tabwin_with_file(mut fnum: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*(*wp).w_buffer).handle == fnum {
                    goto_tabpage_win(tp as *mut tabpage_T, wp);
                    return true_0 != 0;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn qf_open_new_file_win(
    mut ll_ref: *mut qf_info_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut flags: ::core::ffi::c_int = WSP_ABOVE as ::core::ffi::c_int;
        if !ll_ref.is_null() {
            flags |= WSP_NEWLOC as ::core::ffi::c_int;
        }
        if win_split(0 as ::core::ffi::c_int, flags) == FAIL {
            return FAIL;
        }
        p_swb.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
        swb_flags.set(0 as ::core::ffi::c_uint);
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        if !ll_ref.is_null() {
            win_set_loclist(curwin.get(), ll_ref);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_goto_win_with_ll_file(
    mut use_win: *mut win_T,
    mut qf_fnum: ::core::ffi::c_int,
    mut ll_ref: *mut qf_info_T,
) {
    unsafe {
        let mut win: *mut win_T = use_win;
        if win.is_null() {
            let mut win2: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !win2.is_null() {
                if (*(*win2).w_buffer).handle == qf_fnum {
                    win = win2;
                    break;
                } else {
                    win2 = (*win2).w_next;
                }
            }
            if win.is_null() {
                win = curwin.get();
                while !bt_normal((*win).w_buffer) {
                    if (*win).w_prev.is_null() {
                        win = lastwin.get();
                    } else {
                        win = (*win).w_prev;
                    }
                    if win == curwin.get() {
                        break;
                    }
                }
            }
        }
        win_goto(win);
        if (*win).w_llist.is_null() && !ll_ref.is_null() {
            win_set_loclist(win, ll_ref);
        }
    }
}

pub(crate) unsafe extern "C" fn qf_goto_win_with_qfl_file(mut qf_fnum: ::core::ffi::c_int) {
    unsafe {
        let mut win: *mut win_T = curwin.get();
        let mut altwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
        while (*(*win).w_buffer).handle != qf_fnum {
            if (*win).w_prev.is_null() {
                win = lastwin.get();
            } else {
                win = (*win).w_prev;
            }
            if bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
                && (*win).w_llist_ref.is_null()
            {
                if swb_flags.get() & kOptSwbFlagUselast as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                    && win_valid(prevwin.get()) as ::core::ffi::c_int != 0
                    && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
                {
                    win = prevwin.get();
                } else if !altwin.is_null() {
                    win = altwin;
                } else if !(*curwin.get()).w_prev.is_null() {
                    win = (*curwin.get()).w_prev;
                } else {
                    win = (*curwin.get()).w_next;
                }
                break;
            } else if altwin.is_null()
                && (*win).w_onebuf_opt.wo_pvw == 0
                && (*win).w_onebuf_opt.wo_wfb == 0
                && bt_normal((*win).w_buffer) as ::core::ffi::c_int != 0
            {
                altwin = win;
            }
        }
        win_goto(win);
    }
}

pub(crate) unsafe extern "C" fn qf_jump_to_usable_window(
    mut qf_fnum: ::core::ffi::c_int,
    mut newwin: bool,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut usable_wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut usable_win: bool = false_0 != 0;
        let mut ll_ref: *mut qf_info_T = if newwin as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<qf_info_T>()
        } else {
            (*curwin.get()).w_llist_ref
        };
        if !ll_ref.is_null() {
            usable_wp = qf_find_win_with_loclist(ll_ref);
            if !usable_wp.is_null() {
                usable_win = true_0 != 0;
            }
        }
        if !usable_win {
            let mut win: *mut win_T = qf_find_win_with_normal_buf();
            if !win.is_null() {
                usable_win = true_0 != 0;
            }
        }
        if !usable_win
            && swb_flags.get() & kOptSwbFlagUsetab as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            usable_win = qf_goto_tabwin_with_file(qf_fnum);
        }
        if firstwin.get() == lastwin.get() && bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
            || !usable_win
            || newwin as ::core::ffi::c_int != 0
        {
            if qf_open_new_file_win(ll_ref) != OK {
                return FAIL;
            }
            *opened_window = true_0 != 0;
        } else if !(*curwin.get()).w_llist_ref.is_null() {
            qf_goto_win_with_ll_file(usable_wp, qf_fnum, ll_ref);
        } else {
            qf_goto_win_with_qfl_file(qf_fnum);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_jump_edit_buffer(
    mut qi: *mut qf_info_T,
    mut qf_ptr: *mut qfline_T,
    mut forceit: ::core::ffi::c_int,
    mut prev_winid: ::core::ffi::c_int,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        let mut old_changetick: ::core::ffi::c_int = (*qfl).qf_changedtick;
        let mut old_qf_curlist: ::core::ffi::c_int = (*qi).qf_curlist;
        let mut qfl_type: qfltype_T = (*qfl).qfl_type;
        let mut retval: ::core::ffi::c_int = OK;
        let mut save_qfid: ::core::ffi::c_uint = (*qfl).qf_id;
        if (*qf_ptr).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            if !can_abandon(curbuf.get(), forceit != 0) {
                no_write_message();
                return FAIL;
            }
            retval = do_ecmd(
                (*qf_ptr).qf_fnum,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<exarg_T>(),
                1 as linenr_T,
                ECMD_HIDE as ::core::ffi::c_int + ECMD_SET_HELP as ::core::ffi::c_int,
                if prev_winid == (*curwin.get()).handle {
                    curwin.get()
                } else {
                    ::core::ptr::null_mut::<win_T>()
                },
            );
        } else {
            let mut fnum: ::core::ffi::c_int = (*qf_ptr).qf_fnum;
            if forceit == 0
                && (*curwin.get()).w_onebuf_opt.wo_wfb != 0
                && (*curbuf.get()).handle != fnum
            {
                if (*qi).qfl_type as ::core::ffi::c_uint
                    == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    emsg(gettext(
                        &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
                    ));
                    return FAIL;
                }
                if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
                    && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
                    && !bt_quickfix((*prevwin.get()).w_buffer)
                {
                    win_goto(prevwin.get());
                }
                if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
                    if win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == OK {
                        *opened_window = true_0 != 0;
                    }
                    if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
                        emsg(gettext(
                            &raw const e_winfixbuf_cannot_go_to_buffer
                                as *const ::core::ffi::c_char,
                        ));
                        retval = FAIL;
                    }
                }
            }
            if retval == OK {
                retval = buflist_getfile(
                    fnum,
                    1 as linenr_T,
                    GETF_SETMARK as ::core::ffi::c_int | GETF_SWITCH as ::core::ffi::c_int,
                    forceit,
                );
            }
        }
        if qfl_type as ::core::ffi::c_uint
            == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut wp: *mut win_T = win_id2wp(prev_winid);
            if wp.is_null() && (*curwin.get()).w_llist != qi {
                emsg(gettext(
                    b"E924: Current window was closed\0".as_ptr() as *const ::core::ffi::c_char
                ));
                *opened_window = false_0 != 0;
                return QF_ABORT as ::core::ffi::c_int;
            }
        }
        if qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            && !qflist_valid(::core::ptr::null_mut::<win_T>(), save_qfid)
        {
            emsg(gettext(e_current_quickfix_list_was_changed.get()));
            return QF_ABORT as ::core::ffi::c_int;
        }
        if old_qf_curlist != (*qi).qf_curlist
            || old_changetick != (*qfl).qf_changedtick
            || !is_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(e_current_quickfix_list_was_changed.get()));
            } else {
                emsg(gettext(e_current_location_list_was_changed.get()));
            }
            return QF_ABORT as ::core::ffi::c_int;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn qf_jump_goto_line(
    mut qf_lnum: linenr_T,
    mut qf_col: ::core::ffi::c_int,
    mut qf_viscol: ::core::ffi::c_char,
    mut qf_pattern: *mut ::core::ffi::c_char,
) {
    unsafe {
        if qf_pattern.is_null() {
            let mut i: linenr_T = qf_lnum;
            if i > 0 as linenr_T {
                i = if i < (*curbuf.get()).b_ml.ml_line_count {
                    i
                } else {
                    (*curbuf.get()).b_ml.ml_line_count
                };
                (*curwin.get()).w_cursor.lnum = i;
            }
            if qf_col > 0 as ::core::ffi::c_int {
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                if qf_viscol as ::core::ffi::c_int == true_0 {
                    coladvance(curwin.get(), qf_col as colnr_T - 1 as colnr_T);
                } else {
                    (*curwin.get()).w_cursor.col = (qf_col - 1 as ::core::ffi::c_int) as colnr_T;
                }
                (*curwin.get()).w_set_curswant = true_0;
                check_cursor(curwin.get());
            } else {
                beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            }
        } else {
            let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
            if do_search(
                ::core::ptr::null_mut::<oparg_T>(),
                '/' as ::core::ffi::c_int,
                '/' as ::core::ffi::c_int,
                qf_pattern,
                strlen(qf_pattern),
                1 as ::core::ffi::c_int,
                SEARCH_KEEP as ::core::ffi::c_int,
                ::core::ptr::null_mut::<searchit_arg_T>(),
            ) == 0
            {
                (*curwin.get()).w_cursor = save_cursor;
            }
        };
    }
}

pub(crate) unsafe extern "C" fn qf_jump_print_msg(
    mut qi: *mut qf_info_T,
    mut qf_index: ::core::ffi::c_int,
    mut qf_ptr: *mut qfline_T,
    mut old_curbuf: *mut buf_T,
    mut old_lnum: linenr_T,
) {
    unsafe {
        let gap: *mut garray_T = qfga_get();
        if msg_scrolled.get() == 0 {
            update_topline(curwin.get());
            if must_redraw.get() != 0 {
                update_screen();
            }
        }
        let mut IObufflen: size_t = vim_snprintf_safelen(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            gettext(b"(%d of %d)%s%s: \0".as_ptr() as *const ::core::ffi::c_char),
            qf_index,
            (*qf_get_curlist(qi)).qf_count,
            if (*qf_ptr).qf_cleared as ::core::ffi::c_int != 0 {
                gettext(b" (line deleted)\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            qf_types((*qf_ptr).qf_type as ::core::ffi::c_int, (*qf_ptr).qf_nr),
        );
        ga_concat_len(gap, IObuff.ptr() as *mut ::core::ffi::c_char, IObufflen);
        qf_fmt_text(gap, skipwhite((*qf_ptr).qf_text));
        ga_append(gap, NUL as uint8_t);
        let mut i: linenr_T = msg_scroll.get() as linenr_T;
        if curbuf.get() == old_curbuf && (*curwin.get()).w_cursor.lnum == old_lnum {
            msg_scroll.set(true_0);
        } else if (msg_scrolled.get() == 0 as ::core::ffi::c_int
            || p_ch.get() == 0 as OptInt && msg_scrolled.get() == 1 as ::core::ffi::c_int)
            && shortmess(SHM_OVERALL as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            msg_scroll.set(false_0);
        }
        msg_ext_set_kind(b"quickfix\0".as_ptr() as *const ::core::ffi::c_char);
        msg_keep(
            (*gap).ga_data as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
        );
        msg_scroll.set(i as ::core::ffi::c_int);
        qfga_clear();
    }
}

pub(crate) unsafe extern "C" fn qf_jump_open_window(
    mut qi: *mut qf_info_T,
    mut qf_ptr: *mut qfline_T,
    mut newwin: bool,
    mut opened_window: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        let mut old_changetick: ::core::ffi::c_int = (*qfl).qf_changedtick;
        let mut old_qf_curlist: ::core::ffi::c_int = (*qi).qf_curlist;
        let mut qfl_type: qfltype_T = (*qfl).qfl_type;
        if (*qf_ptr).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (!bt_help((*curwin.get()).w_buffer)
                || (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int)
        {
            if jump_to_help_window(qi, newwin, opened_window) == FAIL {
                return FAIL;
            }
        }
        if old_qf_curlist != (*qi).qf_curlist
            || old_changetick != (*qfl).qf_changedtick
            || !is_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(e_current_quickfix_list_was_changed.get()));
            } else {
                emsg(gettext(e_current_location_list_was_changed.get()));
            }
            return QF_ABORT as ::core::ffi::c_int;
        }
        if bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0 && !*opened_window {
            if (*qf_ptr).qf_fnum == 0 as ::core::ffi::c_int {
                return NOTDONE;
            }
            if qf_jump_to_usable_window((*qf_ptr).qf_fnum, newwin, opened_window) == FAIL {
                return FAIL;
            }
        }
        if old_qf_curlist != (*qi).qf_curlist
            || old_changetick != (*qfl).qf_changedtick
            || !is_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(e_current_quickfix_list_was_changed.get()));
            } else {
                emsg(gettext(e_current_location_list_was_changed.get()));
            }
            return QF_ABORT as ::core::ffi::c_int;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_jump_to_buffer(
    mut qi: *mut qf_info_T,
    mut qf_index: ::core::ffi::c_int,
    mut qf_ptr: *mut qfline_T,
    mut forceit: ::core::ffi::c_int,
    mut prev_winid: ::core::ffi::c_int,
    mut opened_window: *mut bool,
    mut openfold: ::core::ffi::c_int,
    mut print_message: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut old_curbuf: *mut buf_T = curbuf.get();
        let mut old_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut retval: ::core::ffi::c_int = OK;
        if (*qf_ptr).qf_fnum != 0 as ::core::ffi::c_int {
            retval = qf_jump_edit_buffer(qi, qf_ptr, forceit, prev_winid, opened_window);
            if retval != OK {
                return retval;
            }
        }
        if curbuf.get() == old_curbuf {
            setpcmark();
        }
        qf_jump_goto_line(
            (*qf_ptr).qf_lnum,
            (*qf_ptr).qf_col,
            (*qf_ptr).qf_viscol,
            (*qf_ptr).qf_pattern,
        );
        if fdo_flags.get() & kOptFdoFlagQuickfix as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && openfold != 0
        {
            foldOpenCursor();
        }
        if print_message {
            qf_jump_print_msg(qi, qf_index, qf_ptr, old_curbuf, old_lnum);
        }
        return retval;
    }
}

pub unsafe fn qf_jump(
    mut qi: *mut qf_info_T,
    mut dir: ::core::ffi::c_int,
    mut errornr: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) {
    unsafe {
        qf_jump_newwin(qi, dir, errornr, forceit, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn qf_jump_newwin(
    mut qi: *mut qf_info_T,
    mut dir: ::core::ffi::c_int,
    mut errornr: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
    mut newwin: bool,
) {
    unsafe {
        let mut print_message: bool = false;
        let mut prev_winid: ::core::ffi::c_int = 0;
        let mut opened_window: bool = false;
        let mut retval: ::core::ffi::c_int = 0;
        let mut old_swb: *mut ::core::ffi::c_char = p_swb.get();
        let mut old_swb_flags: ::core::ffi::c_uint = swb_flags.get();
        let old_KeyTyped: bool = KeyTyped.get();
        if qi.is_null() {
            '_c2rust_label: {
                if !(*ql_info.ptr()).is_null() {
                } else {
                    __assert_fail(
                        b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3240 as ::core::ffi::c_uint,
                        b"void qf_jump_newwin(qf_info_T *, int, int, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            qi = ql_info.get();
        }
        if qf_stack_empty(qi) as ::core::ffi::c_int != 0
            || qf_list_empty(qf_get_curlist(qi)) as ::core::ffi::c_int != 0
        {
            emsg(gettext(
                &raw const e_no_errors as *const ::core::ffi::c_char,
            ));
            return;
        }
        incr_quickfix_busy();
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
        let mut old_qf_ptr: *mut qfline_T = qf_ptr;
        let mut qf_index: ::core::ffi::c_int = (*qfl).qf_index;
        let mut old_qf_index: ::core::ffi::c_int = qf_index;
        qf_ptr = qf_get_entry(qfl, errornr, dir, &mut qf_index);
        '_theend: {
            if qf_ptr.is_null() {
                qf_ptr = old_qf_ptr;
                qf_index = old_qf_index;
            } else {
                (*qfl).qf_index = qf_index;
                (*qfl).qf_ptr = qf_ptr;
                print_message = !qf_win_pos_update(qi, old_qf_index);
                prev_winid = (*curwin.get()).handle as ::core::ffi::c_int;
                opened_window = false_0 != 0;
                retval = qf_jump_open_window(qi, qf_ptr, newwin, &raw mut opened_window);
                if retval != FAIL {
                    if retval == QF_ABORT as ::core::ffi::c_int {
                        qi = ::core::ptr::null_mut::<qf_info_T>();
                        qf_ptr = ::core::ptr::null_mut::<qfline_T>();
                        break '_theend;
                    } else if retval == NOTDONE {
                        break '_theend;
                    } else {
                        retval = qf_jump_to_buffer(
                            qi,
                            qf_index,
                            qf_ptr,
                            forceit,
                            prev_winid,
                            &raw mut opened_window,
                            old_KeyTyped as ::core::ffi::c_int,
                            print_message,
                        );
                        if retval == QF_ABORT as ::core::ffi::c_int {
                            qi = ::core::ptr::null_mut::<qf_info_T>();
                            qf_ptr = ::core::ptr::null_mut::<qfline_T>();
                        }
                        if retval != OK {
                            if opened_window {
                                win_close(curwin.get(), true_0 != 0, false_0 != 0);
                            }
                            if !(!qf_ptr.is_null() && (*qf_ptr).qf_fnum != 0 as ::core::ffi::c_int)
                            {
                                break '_theend;
                            }
                        } else {
                            break '_theend;
                        }
                    }
                }
                qf_ptr = old_qf_ptr;
                qf_index = old_qf_index;
            }
        }
        if !qi.is_null() {
            (*qfl).qf_ptr = qf_ptr;
            (*qfl).qf_index = qf_index;
        }
        if p_swb.get() != old_swb
            && p_swb.get() == empty_string_option.ptr() as *mut ::core::ffi::c_char
        {
            p_swb.set(old_swb);
            swb_flags.set(old_swb_flags);
        }
        decr_quickfix_busy();
    }
}

pub(crate) unsafe extern "C" fn qf_jump_first(
    mut qi: *mut qf_info_T,
    mut save_qfid: ::core::ffi::c_uint,
    mut forceit: ::core::ffi::c_int,
) {
    unsafe {
        if qf_restore_list(qi, save_qfid) == FAIL {
            return;
        }
        if !check_can_set_curbuf_forceit(forceit) {
            return;
        }
        if !qf_list_empty(qf_get_curlist(qi)) {
            qf_jump(
                qi,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                forceit,
            );
        }
    }
}
