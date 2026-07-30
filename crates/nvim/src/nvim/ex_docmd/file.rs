//! Commands that name a file or a buffer: reading, editing, finding,
//! recovering, and the buffer list.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn is_other_file(mut fnum: c_int, mut ffname: *mut c_char) -> bool {
    if fnum != 0 as c_int {
        if fnum == (*curbuf.get()).handle {
            return false_0 != 0;
        }
        return true_0 != 0;
    }
    if ffname.is_null() {
        return true_0 != 0;
    }
    if *ffname as c_int == NUL {
        return false_0 != 0;
    }
    if !(*curbuf.get()).file_id_valid
        && !(*curbuf.get()).b_sfname.is_null()
        && *(*curbuf.get()).b_sfname as c_int != NUL
    {
        return path_fnamecmp(ffname, (*curbuf.get()).b_sfname) != 0 as c_int;
    }
    return otherfile(ffname);
}

pub(crate) unsafe extern "C" fn ex_buffer(mut eap: *mut exarg_T) {
    do_exbuffer(eap);
}

pub(crate) unsafe extern "C" fn do_exbuffer(mut eap: *mut exarg_T) {
    if *(*eap).arg != 0 {
        (*eap).errmsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, (*eap).arg);
    } else {
        if (*eap).addr_count == 0 as c_int {
            goto_buffer(eap, DOBUF_CURRENT as c_int, FORWARD as c_int, 0 as c_int);
        } else {
            goto_buffer(
                eap,
                DOBUF_FIRST as c_int,
                FORWARD as c_int,
                (*eap).line2 as c_int,
            );
        }
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
    };
}

pub(crate) unsafe extern "C" fn ex_bmodified(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_MOD as c_int,
        FORWARD as c_int,
        (*eap).line2 as c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}

pub(crate) unsafe extern "C" fn ex_bnext(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_CURRENT as c_int,
        FORWARD as c_int,
        (*eap).line2 as c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}

pub(crate) unsafe extern "C" fn ex_bprevious(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_CURRENT as c_int,
        BACKWARD as c_int,
        (*eap).line2 as c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}

pub(crate) unsafe extern "C" fn ex_brewind(mut eap: *mut exarg_T) {
    goto_buffer(eap, DOBUF_FIRST as c_int, FORWARD as c_int, 0 as c_int);
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}

pub(crate) unsafe extern "C" fn ex_blast(mut eap: *mut exarg_T) {
    goto_buffer(eap, DOBUF_LAST as c_int, BACKWARD as c_int, 0 as c_int);
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}

pub(crate) unsafe extern "C" fn ex_preserve(mut _eap: *mut exarg_T) {
    ml_preserve(curbuf.get(), true_0 != 0, true_0 != 0);
}

pub(crate) unsafe extern "C" fn ex_recover(mut eap: *mut exarg_T) {
    recoverymode.set(true_0 != 0);
    if !check_changed(
        curbuf.get(),
        (if p_awa.get() != 0 {
            CCGD_AW as c_int
        } else {
            0 as c_int
        }) | CCGD_MULTWIN as c_int
            | (if (*eap).forceit != 0 {
                CCGD_FORCEIT as c_int
            } else {
                0 as c_int
            })
            | CCGD_EXCMD as c_int,
    ) && (*(*eap).arg as c_int == NUL
        || setfname(
            curbuf.get(),
            (*eap).arg,
            ::core::ptr::null_mut::<c_char>(),
            true_0 != 0,
        ) == OK)
    {
        ml_recover(true_0 != 0);
    }
    recoverymode.set(false_0 != 0);
}

pub(crate) unsafe extern "C" fn ex_find(mut eap: *mut exarg_T) {
    if !check_can_set_curbuf_forceit((*eap).forceit) {
        return;
    }
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *get_findfunc() as c_int != NUL {
        fname = findfunc_find_file(
            (*eap).arg,
            strlen((*eap).arg),
            if (*eap).addr_count > 0 as c_int {
                (*eap).line2 as c_int
            } else {
                1 as c_int
            },
        );
    } else {
        let mut file_to_find: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut search_ctx: *mut c_char = ::core::ptr::null_mut::<c_char>();
        fname = find_file_in_path(
            (*eap).arg,
            strlen((*eap).arg),
            FNAME_MESS as c_int,
            true_0,
            (*curbuf.get()).b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        if (*eap).addr_count > 0 as c_int {
            let mut count: linenr_T = (*eap).line2;
            while !fname.is_null() && {
                count -= 1;
                count > 0 as linenr_T
            } {
                xfree(fname as *mut c_void);
                fname = find_file_in_path(
                    ::core::ptr::null_mut::<c_char>(),
                    0 as size_t,
                    FNAME_MESS as c_int,
                    false_0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
        }
        xfree(file_to_find as *mut c_void);
        vim_findfile_cleanup(search_ctx as *mut c_void);
    }
    if fname.is_null() {
        return;
    }
    (*eap).arg = fname;
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
    xfree(fname as *mut c_void);
}

pub(crate) unsafe extern "C" fn ex_edit(mut eap: *mut exarg_T) {
    let mut ffname: *mut c_char = if (*eap).cmdidx as c_int == CMD_enew as c_int {
        ::core::ptr::null_mut::<c_char>()
    } else {
        (*eap).arg
    };
    if (*eap).cmdidx as c_int != CMD_badd as c_int
        && (*eap).cmdidx as c_int != CMD_balt as c_int
        && (is_other_file(0 as c_int, ffname) as c_int != 0
            && !check_can_set_curbuf_forceit((*eap).forceit))
    {
        return;
    }
    if bt_prompt(curbuf.get()) as c_int != 0
        && (*eap).cmdidx as c_int == CMD_edit as c_int
        && *(*eap).arg as c_int == NUL
    {
        emsg(b"cannot :edit a prompt buffer\0".as_ptr() as *const c_char);
        return;
    }
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
}

pub unsafe extern "C" fn do_exedit(mut eap: *mut exarg_T, mut old_curwin: *mut win_T) {
    if exmode_active.get() as c_int != 0
        && ((*eap).cmdidx as c_int == CMD_visual as c_int
            || (*eap).cmdidx as c_int == CMD_view as c_int)
    {
        exmode_active.set(false_0 != 0);
        ex_pressedreturn.set(false_0 != 0);
        if ui_has(kUICmdline) {
            ui_ext_cmdline_block_leave();
        }
        if *(*eap).arg as c_int == NUL {
            if global_busy.get() != 0 {
                if !(*eap).nextcmd.is_null() {
                    stuffReadbuff((*eap).nextcmd);
                    (*eap).nextcmd = ::core::ptr::null_mut::<c_char>();
                }
                let save_rd: c_int = RedrawingDisabled.get();
                RedrawingDisabled.set(0 as c_int);
                let save_nwr: c_int = no_wait_return.get();
                no_wait_return.set(0 as c_int);
                need_wait_return.set(false_0 != 0);
                let save_ms: c_int = msg_scroll.get();
                msg_scroll.set(0 as c_int);
                redraw_all_later(UPD_NOT_VALID as c_int);
                pending_exmode_active.set(true_0 != 0);
                normal_enter(false_0 != 0, true_0 != 0);
                pending_exmode_active.set(false_0 != 0);
                RedrawingDisabled.set(save_rd);
                no_wait_return.set(save_nwr);
                msg_scroll.set(save_ms);
            }
            return;
        }
    }
    if ((*eap).cmdidx as c_int == CMD_new as c_int
        || (*eap).cmdidx as c_int == CMD_tabnew as c_int
        || (*eap).cmdidx as c_int == CMD_tabedit as c_int
        || (*eap).cmdidx as c_int == CMD_vnew as c_int)
        && *(*eap).arg as c_int == NUL
    {
        setpcmark();
        do_ecmd(
            0 as c_int,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            eap,
            ECMD_ONE as c_int as linenr_T,
            ECMD_HIDE as c_int
                + (if (*eap).forceit != 0 {
                    ECMD_FORCEIT as c_int
                } else {
                    0 as c_int
                }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        );
    } else if (*eap).cmdidx as c_int != CMD_split as c_int
        && (*eap).cmdidx as c_int != CMD_vsplit as c_int
        || *(*eap).arg as c_int != NUL
    {
        if *(*eap).arg as c_int != NUL && text_or_buf_locked() as c_int != 0 {
            return;
        }
        let mut n: c_int = readonlymode.get() as c_int;
        if (*eap).cmdidx as c_int == CMD_view as c_int
            || (*eap).cmdidx as c_int == CMD_sview as c_int
        {
            readonlymode.set(true_0 != 0);
        } else if (*eap).cmdidx as c_int == CMD_enew as c_int {
            readonlymode.set(false_0 != 0);
        }
        if (*eap).cmdidx as c_int != CMD_balt as c_int
            && (*eap).cmdidx as c_int != CMD_badd as c_int
        {
            setpcmark();
        }
        if do_ecmd(
            0 as c_int,
            if (*eap).cmdidx as c_int == CMD_enew as c_int {
                ::core::ptr::null_mut::<c_char>()
            } else {
                (*eap).arg
            },
            ::core::ptr::null_mut::<c_char>(),
            eap,
            (*eap).do_ecmd_lnum,
            (if buf_hide(curbuf.get()) as c_int != 0 {
                ECMD_HIDE as c_int
            } else {
                0 as c_int
            }) + (if (*eap).forceit != 0 {
                ECMD_FORCEIT as c_int
            } else {
                0 as c_int
            }) + (if !old_curwin.is_null() {
                ECMD_OLDBUF as c_int
            } else {
                0 as c_int
            }) + (if (*eap).cmdidx as c_int == CMD_badd as c_int {
                ECMD_ADDBUF as c_int
            } else {
                0 as c_int
            }) + (if (*eap).cmdidx as c_int == CMD_balt as c_int {
                ECMD_ALTBUF as c_int
            } else {
                0 as c_int
            }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        ) == FAIL
        {
            if !old_curwin.is_null() {
                let mut need_hide: bool =
                    curbufIsChanged() as c_int != 0 && (*curbuf.get()).b_nwindows <= 1 as c_int;
                if !need_hide || buf_hide(curbuf.get()) as c_int != 0 {
                    let mut cs: cleanup_T = cleanup_T {
                        pending: 0,
                        exception: ::core::ptr::null_mut::<except_T>(),
                    };
                    enter_cleanup(&raw mut cs);
                    win_close(
                        curwin.get(),
                        !need_hide && !buf_hide(curbuf.get()),
                        false_0 != 0,
                    );
                    leave_cleanup(&raw mut cs);
                }
            }
        } else if readonlymode.get() as c_int != 0 && (*curbuf.get()).b_nwindows == 1 as c_int {
            (*curbuf.get()).b_p_ro = true_0;
        }
        readonlymode.set(n != 0);
    } else {
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
        let mut n_0: c_int = (*curwin.get()).w_arg_idx_invalid;
        check_arg_idx(curwin.get());
        if n_0 != (*curwin.get()).w_arg_idx_invalid {
            maketitle();
        }
    }
    if !old_curwin.is_null()
        && *(*eap).arg as c_int != NUL
        && curwin.get() != old_curwin
        && win_valid(old_curwin) as c_int != 0
        && (*old_curwin).w_buffer != curbuf.get()
        && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as c_int == 0 as c_int
    {
        (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as c_int;
    }
    ex_no_reprint.set(true_0 != 0);
}

pub(crate) unsafe extern "C" fn ex_swapname(mut _eap: *mut exarg_T) {
    if (*curbuf.get()).b_ml.ml_mfp.is_null() || (*(*curbuf.get()).b_ml.ml_mfp).mf_fname.is_null() {
        msg(
            gettext(b"No swap file\0".as_ptr() as *const c_char),
            0 as c_int,
        );
    } else {
        msg((*(*curbuf.get()).b_ml.ml_mfp).mf_fname, 0 as c_int);
    };
}

pub(crate) unsafe extern "C" fn ex_read(mut eap: *mut exarg_T) {
    let mut empty: c_int = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY;
    if (*eap).usefilter != 0 {
        do_bang(1 as c_int, eap, false_0 != 0, false_0 != 0, true_0 != 0);
        return;
    }
    if u_save((*eap).line2, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    let mut i: c_int = 0;
    if *(*eap).arg as c_int == NUL {
        if check_fname() == FAIL {
            return;
        }
        i = readfile(
            (*curbuf.get()).b_ffname,
            (*curbuf.get()).b_fname,
            (*eap).line2,
            0 as linenr_T,
            MAXLNUM as c_int as linenr_T,
            eap,
            0 as c_int,
            false_0 != 0,
        );
    } else {
        if !vim_strchr(p_cpo.get(), CPO_ALTREAD).is_null() {
            setaltfname((*eap).arg, (*eap).arg, 1 as linenr_T);
        }
        i = readfile(
            (*eap).arg,
            ::core::ptr::null_mut::<c_char>(),
            (*eap).line2,
            0 as linenr_T,
            MAXLNUM as c_int as linenr_T,
            eap,
            0 as c_int,
            false_0 != 0,
        );
    }
    if i != OK {
        if !aborting() {
            semsg(gettext(&raw const e_notopen as *const c_char), (*eap).arg);
        }
    } else {
        if empty != 0 && exmode_active.get() as c_int != 0 {
            let mut lnum: linenr_T = 0;
            if (*eap).line2 == 0 as linenr_T {
                lnum = (*curbuf.get()).b_ml.ml_line_count;
            } else {
                lnum = 1 as c_int as linenr_T;
            }
            if *ml_get(lnum) as c_int == NUL && u_savedel(lnum, 1 as linenr_T) == OK {
                ml_delete(lnum);
                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                    && (*curwin.get()).w_cursor.lnum >= lnum
                {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                deleted_lines_mark(lnum, 1 as c_int);
            }
        }
        redraw_curbuf_later(UPD_VALID as c_int);
    };
}

pub(crate) unsafe extern "C" fn ex_bang(mut eap: *mut exarg_T) {
    do_bang(
        (*eap).addr_count,
        eap,
        (*eap).forceit != 0,
        true_0 != 0,
        true_0 != 0,
    );
}

pub(crate) unsafe extern "C" fn ex_wundo(mut eap: *mut exarg_T) {
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
    u_write_undo(
        (*eap).arg,
        (*eap).forceit != 0,
        curbuf.get(),
        &raw mut hash as *mut uint8_t,
    );
}

pub(crate) unsafe extern "C" fn ex_rundo(mut eap: *mut exarg_T) {
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
    u_read_undo(
        (*eap).arg,
        &raw mut hash as *mut uint8_t,
        ::core::ptr::null::<c_char>(),
    );
}

pub(crate) unsafe extern "C" fn ex_checkpath(mut eap: *mut exarg_T) {
    find_pattern_in_path(
        ::core::ptr::null_mut::<c_char>(),
        kDirectionNotSet,
        0 as size_t,
        false_0 != 0,
        false_0 != 0,
        CHECK_PATH as c_int,
        1 as c_int,
        if (*eap).forceit != 0 {
            ACTION_SHOW_ALL as c_int
        } else {
            ACTION_SHOW as c_int
        },
        1 as linenr_T,
        MAXLNUM as c_int as linenr_T,
        (*eap).forceit != 0,
        false_0 != 0,
    );
}

pub(crate) unsafe extern "C" fn ex_shada(mut eap: *mut exarg_T) {
    let mut save_shada: *mut c_char = p_shada.get();
    if *p_shada.get() as c_int == NUL {
        p_shada.set(b"'100\0".as_ptr() as *const c_char as *mut c_char);
    }
    if (*eap).cmdidx as c_int == CMD_rviminfo as c_int
        || (*eap).cmdidx as c_int == CMD_rshada as c_int
    {
        shada_read_everything((*eap).arg, (*eap).forceit != 0, false_0 != 0);
    } else {
        shada_write_file((*eap).arg, (*eap).forceit != 0);
    }
    p_shada.set(save_shada);
}

pub(crate) unsafe extern "C" fn ex_fclose(mut eap: *mut exarg_T) {
    win_float_remove((*eap).forceit != 0, (*eap).line1 as c_int);
}
