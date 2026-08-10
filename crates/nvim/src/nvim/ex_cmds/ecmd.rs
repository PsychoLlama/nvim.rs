//! `do_ecmd` -- the single function behind `:edit`, `:enew`, `:view`,
//! `:new`, `:read`, `:sview` and every other command that changes which file a
//! window shows.
//!
//! It has to decide whether the target is already in a buffer, whether the
//! current buffer can be abandoned (and if so whether to write, hide or wipe
//! it), fire BufLeave/BufUnload/BufEnter/BufWinEnter in the right order with
//! the right buffer current, survive an autocommand that deleted a buffer or
//! closed the window underneath it (`delbuf_msg`), and finally position the
//! cursor from the `+cmd` argument, a mark or the last-known position.
//! `set_swapcommand` is how the `+cmd` reaches the swap-file dialog.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_swapcommand(
    mut command: *mut ::core::ffi::c_char,
    mut newlnum: linenr_T,
) -> bool {
    unsafe {
        if command.is_null() && newlnum <= 0 as linenr_T
            || *get_vim_var_str(VV_SWAPCOMMAND) as ::core::ffi::c_int != NUL
        {
            return false_0 != 0;
        }
        let valsize: size_t = if !command.is_null() {
            strlen(command).wrapping_add(3 as size_t)
        } else {
            30 as size_t
        };
        let mut val: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        val.data = xmalloc(valsize) as *mut ::core::ffi::c_char;
        val.size = if !command.is_null() {
            vim_snprintf_safelen(val.data, valsize, c":%s\r".as_ptr(), command)
        } else {
            vim_snprintf_safelen(val.data, valsize, c"%ldG".as_ptr(), newlnum as int64_t)
        };
        set_vim_var_string(VV_SWAPCOMMAND, val.data, val.size as ptrdiff_t);
        xfree(val.data as *mut ::core::ffi::c_void);
        return true_0 != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn do_ecmd(
    mut fnum: ::core::ffi::c_int,
    mut ffname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut newlnum: linenr_T,
    mut flags: ::core::ffi::c_int,
    mut oldwin: *mut win_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut other_file: bool = false;
        let mut oldbuf: ::core::ffi::c_int = 0;
        let mut auto_buf: bool = false_0 != 0;
        let mut new_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut did_set_swapcommand: bool = false_0 != 0;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut bufref: bufref_T = bufref_T::default();
        let mut old_curbuf: bufref_T = bufref_T::default();
        let mut free_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut topline: linenr_T = 0 as linenr_T;
        let mut newcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut solcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut did_get_winopts: bool = false_0 != 0;
        let mut readfile_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut did_inc_redrawing_disabled: bool = false_0 != 0;
        let mut so_ptr: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_so >= 0 as OptInt {
            &raw mut (*curwin.get()).w_onebuf_opt.wo_so
        } else {
            p_so.ptr()
        };
        if !eap.is_null() {
            command = (*eap).do_ecmd_cmd;
        }
        set_bufref(&raw mut old_curbuf, curbuf.get());
        '_theend: {
            if fnum != 0 as ::core::ffi::c_int {
                if fnum == (*curbuf.get()).handle {
                    return OK;
                }
                other_file = true_0 != 0;
            } else {
                if sfname.is_null() {
                    sfname = ffname;
                }
                if flags & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                    != 0
                    && (ffname.is_null() || *ffname as ::core::ffi::c_int == NUL)
                {
                    break '_theend;
                } else if ffname.is_null() {
                    other_file = true_0 != 0;
                } else if *ffname as ::core::ffi::c_int == NUL && (*curbuf.get()).b_ffname.is_null()
                {
                    other_file = false_0 != 0;
                } else {
                    if *ffname as ::core::ffi::c_int == NUL {
                        ffname = (*curbuf.get()).b_ffname;
                        sfname = (*curbuf.get()).b_fname;
                    }
                    free_fname = fix_fname(ffname);
                    if !free_fname.is_null() {
                        ffname = free_fname;
                    }
                    other_file = otherfile(ffname);
                }
            }
            if !other_file && !(*curbuf.get()).terminal.is_null() {
                check_arg_idx(curwin.get());
                maketitle();
                retval = OK;
            } else if (!other_file && flags & ECMD_OLDBUF as ::core::ffi::c_int == 0
                || (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
                    && flags
                        & (ECMD_HIDE as ::core::ffi::c_int
                            | ECMD_ADDBUF as ::core::ffi::c_int
                            | ECMD_ALTBUF as ::core::ffi::c_int)
                        == 0)
                && check_changed(
                    curbuf.get(),
                    (if p_awa.get() != 0 {
                        CCGD_AW as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if other_file as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        CCGD_MULTWIN as ::core::ffi::c_int
                    }) | (if flags & ECMD_FORCEIT as ::core::ffi::c_int != 0 {
                        CCGD_FORCEIT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if eap.is_null() {
                        0 as ::core::ffi::c_int
                    } else {
                        CCGD_EXCMD as ::core::ffi::c_int
                    }),
                ) as ::core::ffi::c_int
                    != 0
            {
                if fnum == 0 as ::core::ffi::c_int
                    && other_file as ::core::ffi::c_int != 0
                    && !ffname.is_null()
                {
                    setaltfname(
                        ffname,
                        sfname,
                        if newlnum < 0 as linenr_T {
                            0 as linenr_T
                        } else {
                            newlnum
                        },
                    );
                }
            } else {
                reset_VIsual();
                if !oldwin.is_null() && !win_valid(oldwin) {
                    oldwin = ::core::ptr::null_mut::<win_T>();
                }
                did_set_swapcommand = set_swapcommand(command, newlnum);
                if other_file {
                    let prev_alt_fnum: ::core::ffi::c_int = (*curwin.get()).w_alt_fnum;
                    if flags
                        & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                        == 0
                    {
                        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_alt_fnum =
                                (*curbuf.get()).handle as ::core::ffi::c_int;
                        }
                        if !oldwin.is_null() {
                            buflist_altfpos(oldwin);
                        }
                    }
                    if fnum != 0 {
                        buf = buflist_findnr(fnum);
                    } else if flags
                        & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                        != 0
                    {
                        let mut tlnum: linenr_T = 0 as linenr_T;
                        if !command.is_null() {
                            tlnum = atol(command) as linenr_T;
                            if tlnum <= 0 as linenr_T {
                                tlnum = 1 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                        let newbuf: *const buf_T = buflist_new(
                            ffname,
                            sfname,
                            tlnum,
                            BLN_LISTED as ::core::ffi::c_int | BLN_NOCURWIN as ::core::ffi::c_int,
                        );
                        if !newbuf.is_null() && flags & ECMD_ALTBUF as ::core::ffi::c_int != 0 {
                            (*curwin.get()).w_alt_fnum = (*newbuf).handle as ::core::ffi::c_int;
                        }
                        break '_theend;
                    } else {
                        buf = buflist_new(
                            ffname,
                            sfname,
                            0 as linenr_T,
                            BLN_CURBUF as ::core::ffi::c_int
                                | (if flags & ECMD_SET_HELP as ::core::ffi::c_int != 0 {
                                    0 as ::core::ffi::c_int
                                } else {
                                    BLN_LISTED as ::core::ffi::c_int
                                }),
                        );
                        if !oldwin.is_null() {
                            oldwin = curwin.get();
                        }
                        set_bufref(&raw mut old_curbuf, curbuf.get());
                    }
                    if buf.is_null() {
                        break '_theend;
                    } else if (*buf).b_locked_split != 0 {
                        if oldwin.is_null()
                            && !(*curwin.get()).w_buffer.is_null()
                            && (*(*curwin.get()).w_buffer).b_nwindows > 1 as ::core::ffi::c_int
                        {
                            (*(*curwin.get()).w_buffer).b_nwindows -= 1;
                        }
                        emsg(gettext(
                            &raw const e_cannot_switch_to_a_closing_buffer
                                as *const ::core::ffi::c_char,
                        ));
                        break '_theend;
                    } else {
                        if (*curwin.get()).w_alt_fnum == (*buf).handle
                            && prev_alt_fnum != 0 as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_alt_fnum = prev_alt_fnum;
                        }
                        if (*buf).b_ml.ml_mfp.is_null() {
                            oldbuf = false_0;
                        } else {
                            oldbuf = true_0;
                            set_bufref(&raw mut bufref, buf);
                            buf_check_timestamp(buf);
                            if !bufref_valid(&raw mut bufref) || curbuf.get() != old_curbuf.br_buf {
                                break '_theend;
                            } else if aborting() {
                                break '_theend;
                            }
                        }
                        if oldbuf != 0 && newlnum == ECMD_LASTL as ::core::ffi::c_int as linenr_T
                            || newlnum == ECMD_LAST as ::core::ffi::c_int as linenr_T
                        {
                            let mut pos: *mut pos_T = &raw mut (*(buflist_findfmark
                                as unsafe extern "C" fn(*mut buf_T) -> *mut fmark_T)(
                                buf
                            ))
                            .mark;
                            newlnum = (*pos).lnum;
                            solcol = (*pos).col as ::core::ffi::c_int;
                        }
                        if buf != curbuf.get() {
                            debug_assert!((*cmdwin_buf.ptr()).is_null(), "cmdwin_buf == NULL");
                            let save_cmdwin_type: ::core::ffi::c_int = cmdwin_type.get();
                            let save_cmdwin_win: *mut win_T = cmdwin_win.get();
                            let save_cmdwin_old_curwin: *mut win_T = cmdwin_old_curwin.get();
                            cmdwin_type.set(0 as ::core::ffi::c_int);
                            cmdwin_win.set(::core::ptr::null_mut::<win_T>());
                            cmdwin_old_curwin.set(::core::ptr::null_mut::<win_T>());
                            if !(*buf).b_fname.is_null() {
                                new_name = xstrdup((*buf).b_fname);
                            }
                            let save_au_new_curbuf: bufref_T = au_new_curbuf.get();
                            set_bufref(au_new_curbuf.ptr(), buf);
                            apply_autocmds(
                                EVENT_BUFLEAVE,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                curbuf.get(),
                            );
                            cmdwin_type.set(save_cmdwin_type);
                            cmdwin_win.set(save_cmdwin_win);
                            cmdwin_old_curwin.set(save_cmdwin_old_curwin);
                            if !bufref_valid(au_new_curbuf.ptr()) {
                                delbuf_msg(new_name);
                                au_new_curbuf.set(save_au_new_curbuf);
                                break '_theend;
                            } else if aborting() {
                                xfree(new_name as *mut ::core::ffi::c_void);
                                au_new_curbuf.set(save_au_new_curbuf);
                                break '_theend;
                            } else {
                                if buf == curbuf.get() {
                                    auto_buf = true_0 != 0;
                                } else {
                                    let mut the_curwin: *mut win_T = curwin.get();
                                    let mut was_curbuf: *mut buf_T = curbuf.get();
                                    (*the_curwin).w_locked = true_0 != 0;
                                    (*buf).b_locked += 1;
                                    if curbuf.get() == old_curbuf.br_buf {
                                        buf_copy_options(buf, BCO_ENTER as ::core::ffi::c_int);
                                    }
                                    u_sync(false_0 != 0);
                                    let did_decrement: bool = close_buffer(
                                        oldwin,
                                        curbuf.get(),
                                        if flags & ECMD_HIDE as ::core::ffi::c_int != 0
                                            || !(*curbuf.get()).terminal.is_null()
                                                && terminal_running((*curbuf.get()).terminal)
                                                    as ::core::ffi::c_int
                                                    != 0
                                        {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            DOBUF_UNLOAD as ::core::ffi::c_int
                                        },
                                        false_0 != 0,
                                        false_0 != 0,
                                    );
                                    if win_valid(the_curwin) {
                                        (*the_curwin).w_locked = false_0 != 0;
                                    }
                                    (*buf).b_locked -= 1;
                                    if aborting() as ::core::ffi::c_int != 0
                                        && !(*curwin.get()).w_buffer.is_null()
                                    {
                                        xfree(new_name as *mut ::core::ffi::c_void);
                                        au_new_curbuf.set(save_au_new_curbuf);
                                        break '_theend;
                                    } else if !bufref_valid(au_new_curbuf.ptr()) {
                                        delbuf_msg(new_name);
                                        au_new_curbuf.set(save_au_new_curbuf);
                                        break '_theend;
                                    } else {
                                        if buf == curbuf.get() {
                                            if did_decrement as ::core::ffi::c_int != 0
                                                && buf_valid(was_curbuf) as ::core::ffi::c_int != 0
                                            {
                                                (*was_curbuf).b_nwindows += 1;
                                            }
                                            if win_valid_any_tab(oldwin) as ::core::ffi::c_int != 0
                                                && (*oldwin).w_buffer.is_null()
                                            {
                                                (*oldwin).w_buffer = was_curbuf;
                                            }
                                            auto_buf = true_0 != 0;
                                        } else {
                                            if (*curwin.get()).w_buffer.is_null()
                                                || (*curwin.get()).w_s
                                                    == &raw mut (*(*curwin.get()).w_buffer).b_s
                                            {
                                                (*curwin.get()).w_s = &raw mut (*buf).b_s;
                                            }
                                            (*curwin.get()).w_buffer = buf;
                                            curbuf.set(buf);
                                            (*curbuf.get()).b_nwindows += 1;
                                            if oldbuf == 0 && !eap.is_null() {
                                                set_file_options(true_0 != 0, eap);
                                                set_forced_fenc(eap);
                                            }
                                        }
                                        get_winopts(curbuf.get());
                                        did_get_winopts = true_0 != 0;
                                    }
                                }
                                xfree(new_name as *mut ::core::ffi::c_void);
                                au_new_curbuf.set(save_au_new_curbuf);
                            }
                        }
                        (*curwin.get()).w_pcmark.lnum = 1 as ::core::ffi::c_int as linenr_T;
                        (*curwin.get()).w_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                } else if flags
                    & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                    != 0
                    || check_fname() == FAIL
                {
                    break '_theend;
                } else {
                    oldbuf = flags & ECMD_OLDBUF as ::core::ffi::c_int;
                }
                (*RedrawingDisabled.ptr()) += 1;
                did_inc_redrawing_disabled = true_0 != 0;
                buf = curbuf.get();
                if flags & ECMD_SET_HELP as ::core::ffi::c_int != 0
                    || keep_help_flag.get() as ::core::ffi::c_int != 0
                {
                    prepare_help_buffer();
                } else if !(*curbuf.get()).b_help {
                    set_buflisted(true_0);
                }
                if buf == curbuf.get() {
                    if !aborting() {
                        (*curbuf.get()).b_did_filetype = false_0 != 0;
                        if !other_file && oldbuf == 0 {
                            set_last_cursor(curwin.get());
                            if newlnum == ECMD_LAST as ::core::ffi::c_int as linenr_T
                                || newlnum == ECMD_LASTL as ::core::ffi::c_int as linenr_T
                            {
                                newlnum = (*curwin.get()).w_cursor.lnum;
                                solcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                            }
                            buf = curbuf.get();
                            if !(*buf).b_fname.is_null() {
                                new_name = xstrdup((*buf).b_fname);
                            } else {
                                new_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            }
                            set_bufref(&raw mut bufref, buf);
                            if (*curbuf.get()).b_flags & BF_NEVERLOADED == 0
                                && (p_ur.get() < 0 as OptInt
                                    || (*curbuf.get()).b_ml.ml_line_count as OptInt <= p_ur.get())
                            {
                                u_sync(false_0 != 0);
                                if u_savecommon(
                                    curbuf.get(),
                                    0 as linenr_T,
                                    (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T,
                                    0 as linenr_T,
                                    true_0 != 0,
                                ) == FAIL
                                {
                                    xfree(new_name as *mut ::core::ffi::c_void);
                                    break '_theend;
                                } else {
                                    u_unchanged(curbuf.get());
                                    buf_freeall(curbuf.get(), BFA_KEEP_UNDO as ::core::ffi::c_int);
                                    readfile_flags = READ_KEEP_UNDO as ::core::ffi::c_int;
                                }
                            } else {
                                buf_freeall(curbuf.get(), 0 as ::core::ffi::c_int);
                            }
                            if !bufref_valid(&raw mut bufref) {
                                delbuf_msg(new_name);
                                break '_theend;
                            } else {
                                xfree(new_name as *mut ::core::ffi::c_void);
                                if buf != curbuf.get() {
                                    break '_theend;
                                } else if aborting() {
                                    break '_theend;
                                } else {
                                    buf_clear_file(curbuf.get());
                                    (*curbuf.get()).b_op_start.lnum =
                                        0 as ::core::ffi::c_int as linenr_T;
                                    (*curbuf.get()).b_op_end.lnum =
                                        0 as ::core::ffi::c_int as linenr_T;
                                }
                            }
                        }
                        retval = OK;
                        if !other_file {
                            (*curbuf.get()).b_flags &= !BF_NOTEDITED;
                        }
                        check_arg_idx(curwin.get());
                        if !auto_buf {
                            curwin_init();
                            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                            while !tp.is_null() {
                                let mut win: *mut win_T = if tp == curtab.get() {
                                    firstwin.get()
                                } else {
                                    (*tp).tp_firstwin
                                };
                                while !win.is_null() {
                                    if (*win).w_buffer == curbuf.get() {
                                        foldUpdateAll(win);
                                    }
                                    win = (*win).w_next;
                                }
                                tp = (*tp).tp_next as *mut tabpage_T;
                            }
                            do_autochdir();
                            let mut orig_pos: pos_T = (*curwin.get()).w_cursor;
                            topline = (*curwin.get()).w_topline;
                            if oldbuf == 0 {
                                swap_exists_action.set(SEA_DIALOG);
                                (*curbuf.get()).b_flags |= BF_CHECK_RO;
                                if flags & ECMD_NOWINENTER as ::core::ffi::c_int != 0 {
                                    readfile_flags |= READ_NOWINENTER as ::core::ffi::c_int;
                                }
                                if should_abort(open_buffer(false_0 != 0, eap, readfile_flags)) {
                                    retval = FAIL;
                                }
                                if swap_exists_action.get() == SEA_QUIT {
                                    retval = FAIL;
                                }
                                handle_swap_exists(&raw mut old_curbuf);
                            } else {
                                do_modelines(OPT_WINONLY as ::core::ffi::c_int);
                                apply_autocmds_retval(
                                    EVENT_BUFENTER,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    false_0 != 0,
                                    curbuf.get(),
                                    &raw mut retval,
                                );
                                if flags & ECMD_NOWINENTER as ::core::ffi::c_int
                                    == 0 as ::core::ffi::c_int
                                {
                                    apply_autocmds_retval(
                                        EVENT_BUFWINENTER,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        false_0 != 0,
                                        curbuf.get(),
                                        &raw mut retval,
                                    );
                                }
                            }
                            check_arg_idx(curwin.get());
                            if !equalpos((*curwin.get()).w_cursor, orig_pos) {
                                let mut text: *const ::core::ffi::c_char = get_cursor_line_ptr();
                                if (*curwin.get()).w_cursor.lnum != orig_pos.lnum
                                    || (*curwin.get()).w_cursor.col
                                        != skipwhite(text).offset_from(text) as ::core::ffi::c_int
                                {
                                    newlnum = (*curwin.get()).w_cursor.lnum;
                                    newcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                                }
                            }
                            if (*curwin.get()).w_topline == topline {
                                topline = 0 as ::core::ffi::c_int as linenr_T;
                            }
                            changed_line_abv_curs();
                            maketitle();
                        }
                        if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
                            diff_buf_add(curbuf.get());
                            diff_invalidate(curbuf.get());
                        }
                        if did_get_winopts as ::core::ffi::c_int != 0
                            && (*curwin.get()).w_onebuf_opt.wo_spell != 0
                            && *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int != NUL
                        {
                            parse_spelllang(curwin.get());
                        }
                        if command.is_null() {
                            if newcol >= 0 as ::core::ffi::c_int {
                                (*curwin.get()).w_cursor.lnum = newlnum;
                                (*curwin.get()).w_cursor.col = newcol as colnr_T;
                                check_cursor(curwin.get());
                            } else if newlnum > 0 as linenr_T {
                                (*curwin.get()).w_cursor.lnum = newlnum;
                                check_cursor_lnum(curwin.get());
                                if solcol >= 0 as ::core::ffi::c_int && p_sol.get() == 0 {
                                    (*curwin.get()).w_cursor.col = solcol as colnr_T;
                                    check_cursor_col(curwin.get());
                                    (*curwin.get()).w_cursor.coladd =
                                        0 as ::core::ffi::c_int as colnr_T;
                                    (*curwin.get()).w_set_curswant = true_0;
                                } else {
                                    beginline(
                                        BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                                    );
                                }
                            } else {
                                if exmode_active.get() {
                                    (*curwin.get()).w_cursor.lnum =
                                        (*curbuf.get()).b_ml.ml_line_count;
                                }
                                beginline(
                                    BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                                );
                            }
                        }
                        check_lnums(false_0 != 0);
                        if oldbuf != 0 && !auto_buf {
                            let mut msg_scroll_save: ::core::ffi::c_int = msg_scroll.get();
                            if shortmess(SHM_OVERALL as ::core::ffi::c_int) as ::core::ffi::c_int
                                != 0
                                && msg_listdo_overwrite.get() == 0
                                && !exiting.get()
                                && p_verbose.get() == 0 as OptInt
                            {
                                msg_scroll.set(false_0);
                            }
                            if msg_scroll.get() == 0 {
                                msg_check_for_delay(false_0 != 0);
                            }
                            msg_start();
                            msg_scroll.set(msg_scroll_save);
                            msg_scrolled_ign.set(true_0 != 0);
                            if !shortmess(SHM_FILEINFO as ::core::ffi::c_int) {
                                fileinfo(false_0, true_0, false_0 != 0);
                            }
                            msg_scrolled_ign.set(false_0 != 0);
                        }
                        (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
                        if !command.is_null() {
                            do_cmdline(command, None, NULL_0, DOCMD_VERBOSE as ::core::ffi::c_int);
                        }
                        if (*curbuf.get()).b_kmap_state as ::core::ffi::c_int & KEYMAP_INIT != 0 {
                            keymap_init();
                        }
                        (*RedrawingDisabled.ptr()) -= 1;
                        did_inc_redrawing_disabled = false_0 != 0;
                        if !skip_redraw.get() {
                            let mut n: OptInt = *so_ptr;
                            if topline == 0 as linenr_T && command.is_null() {
                                *so_ptr = 999 as OptInt;
                            }
                            update_topline(curwin.get());
                            (*curwin.get()).w_scbind_pos = plines_m_win_fill(
                                curwin.get(),
                                1 as linenr_T,
                                (*curwin.get()).w_topline,
                            );
                            *so_ptr = n;
                            redraw_curbuf_later(UPD_NOT_VALID);
                        }
                        do_autochdir();
                    }
                }
            }
        }
        if bufref_valid(&raw mut old_curbuf) as ::core::ffi::c_int != 0
            && !(*old_curbuf.br_buf).terminal.is_null()
        {
            terminal_check_size((*old_curbuf.br_buf).terminal);
        }
        if (!bufref_valid(&raw mut old_curbuf) || curbuf.get() != old_curbuf.br_buf)
            && !(*curbuf.get()).terminal.is_null()
        {
            terminal_check_size((*curbuf.get()).terminal);
        }
        if did_inc_redrawing_disabled {
            (*RedrawingDisabled.ptr()) -= 1;
        }
        if did_set_swapcommand {
            set_vim_var_string(
                VV_SWAPCOMMAND,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ptrdiff_t,
            );
        }
        xfree(free_fname as *mut ::core::ffi::c_void);
        return retval;
    }
}

unsafe extern "C" fn delbuf_msg(mut name: *mut ::core::ffi::c_char) {
    unsafe {
        semsg_c!(
            gettext(c"E143: Autocommands unexpectedly deleted new buffer %s".as_ptr(),),
            if name.is_null() {
                c"".as_ptr()
            } else {
                name as *const ::core::ffi::c_char
            },
        );
        xfree(name as *mut ::core::ffi::c_void);
        (*au_new_curbuf.ptr()).br_buf = ::core::ptr::null_mut::<buf_T>();
        (*au_new_curbuf.ptr()).br_buf_free_count = 0 as ::core::ffi::c_int;
    }
}
