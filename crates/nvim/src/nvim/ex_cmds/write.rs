//! Putting the buffer on disk -- `:write`, `:update`, `:wall`, `:wq` and the
//! checks that guard them.
//!
//! `do_write` is the entry point every `:w` form funnels into; the risk it
//! manages is not the writing (that is `bufwrite.rs`) but *which file* and
//! *whether we may*: `check_overwrite` refuses an existing other file without
//! `!`, `check_readonly` handles 'readonly' and a read-only file mode, and
//! `check_writable`/`not_writing` cover 'write' and `:noautocmd`.  `ex_file` is
//! `:file`, which renames the buffer, and `getfile` is the shared "switch to
//! this file, writing or abandoning the current one first" helper that `:tag`
//! and friends call.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn rename_buffer(
    mut new_fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = curbuf.get();
        apply_autocmds(
            EVENT_BUFFILEPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if buf != curbuf.get() {
            return FAIL;
        }
        if aborting() {
            return FAIL;
        }
        let mut fname: *mut ::core::ffi::c_char = (*curbuf.get()).b_ffname;
        let mut sfname: *mut ::core::ffi::c_char = (*curbuf.get()).b_sfname;
        let mut xfname: *mut ::core::ffi::c_char = (*curbuf.get()).b_fname;
        (*curbuf.get()).b_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*curbuf.get()).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if setfname(
            curbuf.get(),
            new_fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
        ) == FAIL
        {
            (*curbuf.get()).b_ffname = fname;
            (*curbuf.get()).b_sfname = sfname;
            return FAIL;
        }
        (*curbuf.get()).b_flags |= BF_NOTEDITED;
        if !xfname.is_null() && *xfname as ::core::ffi::c_int != NUL {
            buf = buflist_new(
                fname,
                xfname,
                (*curwin.get()).w_cursor.lnum,
                0 as ::core::ffi::c_int,
            );
            if !buf.is_null()
                && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
            }
        }
        xfree(fname as *mut ::core::ffi::c_void);
        xfree(sfname as *mut ::core::ffi::c_void);
        apply_autocmds(
            EVENT_BUFFILEPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        do_autochdir();
        return OK;
    }
}

pub unsafe fn ex_file(mut eap: *mut exarg_T) {
    unsafe {
        if (*eap).addr_count > 0 as ::core::ffi::c_int
            && (*(*eap).arg as ::core::ffi::c_int != NUL
                || (*eap).line2 > 0 as linenr_T
                || (*eap).addr_count > 1 as ::core::ffi::c_int)
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        if *(*eap).arg as ::core::ffi::c_int != NUL || (*eap).addr_count == 1 as ::core::ffi::c_int
        {
            if rename_buffer((*eap).arg) == FAIL {
                return;
            }
            redraw_tabline.set(true_0 != 0);
        }
        if *(*eap).arg as ::core::ffi::c_int == NUL
            || !shortmess(SHM_FILEINFO as ::core::ffi::c_int)
        {
            fileinfo(false_0, false_0, (*eap).forceit != 0);
        }
    }
}

pub unsafe fn ex_update(mut eap: *mut exarg_T) {
    unsafe {
        if curbufIsChanged() as ::core::ffi::c_int != 0
            || !bt_nofilename(curbuf.get())
                && !(*curbuf.get()).b_ffname.is_null()
                && !os_path_exists((*curbuf.get()).b_ffname)
        {
            do_write(eap);
        }
    }
}

pub unsafe fn ex_write(mut eap: *mut exarg_T) {
    unsafe {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
            (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
        if (*eap).usefilter != 0 {
            do_bang(
                1 as ::core::ffi::c_int,
                eap,
                false_0 != 0,
                true_0 != 0,
                false_0 != 0,
            );
        } else {
            do_write(eap);
        };
    }
}

unsafe extern "C" fn check_writable(mut fname: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if os_nodetype(fname) == NODE_OTHER {
            semsg_c!(
                gettext(c"E503: \"%s\" is not a file or writable device".as_ptr()),
                fname,
            );
            return FAIL;
        }
        return OK;
    }
}

unsafe extern "C" fn handle_mkdir_p_arg(
    mut eap: *mut exarg_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if (*eap).mkdir_p != 0 && os_file_mkdir(fname, 0o755 as int32_t) < 0 as ::core::ffi::c_int {
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn do_write(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    unsafe {
        let mut other: bool = false;
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut free_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut alt_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if not_writing() {
            return FAIL;
        }
        let mut ffname: *mut ::core::ffi::c_char = (*eap).arg;
        '_theend: {
            if *ffname as ::core::ffi::c_int == NUL {
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
                    break '_theend;
                } else {
                    other = false_0 != 0;
                }
            } else {
                fname = ffname;
                free_fname = fix_fname(ffname);
                if !free_fname.is_null() {
                    ffname = free_fname;
                }
                other = otherfile(ffname);
            }
            if other {
                if !vim_strchr(p_cpo.get(), CPO_ALTWRITE).is_null()
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int
                {
                    alt_buf = setaltfname(ffname, fname, 1 as linenr_T);
                } else {
                    alt_buf = buflist_findname(ffname);
                }
                if !alt_buf.is_null() && !(*alt_buf).b_ml.ml_mfp.is_null() {
                    emsg(gettext(
                        &raw const e_bufloaded as *const ::core::ffi::c_char,
                    ));
                    break '_theend;
                }
            }
            if !(!other
                && (bt_dontwrite_msg(curbuf.get()) as ::core::ffi::c_int != 0
                    || check_fname() == FAIL
                    || check_writable((*curbuf.get()).b_ffname) == FAIL
                    || check_readonly(&raw mut (*eap).forceit, curbuf.get()) != 0))
            {
                if !other {
                    ffname = (*curbuf.get()).b_ffname;
                    fname = (*curbuf.get()).b_fname;
                    if ((*eap).line1 != 1 as linenr_T
                        || (*eap).line2 != (*curbuf.get()).b_ml.ml_line_count)
                        && (*eap).forceit == 0
                        && (*eap).append == 0
                        && p_wa.get() == 0
                    {
                        if p_confirm.get() != 0
                            || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                        {
                            if vim_dialog_yesno(
                                VIM_QUESTION as ::core::ffi::c_int,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                gettext(c"Write partial file?".as_ptr()),
                                2 as ::core::ffi::c_int,
                            ) != VIM_YES as ::core::ffi::c_int
                            {
                                break '_theend;
                            } else {
                                (*eap).forceit = true_0;
                            }
                        } else {
                            emsg(gettext(c"E140: Use ! to write partial buffer".as_ptr()));
                            break '_theend;
                        }
                    }
                }
                if check_overwrite(eap, curbuf.get(), fname, ffname, other) == OK {
                    if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int
                        && !alt_buf.is_null()
                    {
                        let mut was_curbuf: *mut buf_T = curbuf.get();
                        apply_autocmds(
                            EVENT_BUFFILEPRE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            curbuf.get(),
                        );
                        apply_autocmds(
                            EVENT_BUFFILEPRE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            alt_buf,
                        );
                        if curbuf.get() != was_curbuf || aborting() as ::core::ffi::c_int != 0 {
                            retval = FAIL;
                            break '_theend;
                        } else {
                            fname = (*alt_buf).b_fname;
                            (*alt_buf).b_fname = (*curbuf.get()).b_fname;
                            (*curbuf.get()).b_fname = fname;
                            fname = (*alt_buf).b_ffname;
                            (*alt_buf).b_ffname = (*curbuf.get()).b_ffname;
                            (*curbuf.get()).b_ffname = fname;
                            fname = (*alt_buf).b_sfname;
                            (*alt_buf).b_sfname = (*curbuf.get()).b_sfname;
                            (*curbuf.get()).b_sfname = fname;
                            buf_name_changed(curbuf.get());
                            apply_autocmds(
                                EVENT_BUFFILEPOST,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                curbuf.get(),
                            );
                            apply_autocmds(
                                EVENT_BUFFILEPOST,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                alt_buf,
                            );
                            if (*alt_buf).b_p_bl == 0 {
                                (*alt_buf).b_p_bl = true_0;
                                apply_autocmds(
                                    EVENT_BUFADD,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    false_0 != 0,
                                    alt_buf,
                                );
                            }
                            if curbuf.get() != was_curbuf || aborting() as ::core::ffi::c_int != 0 {
                                retval = FAIL;
                                break '_theend;
                            } else {
                                if *(*curbuf.get()).b_p_ft as ::core::ffi::c_int == NUL {
                                    if augroup_exists(c"filetypedetect".as_ptr()) {
                                        do_doautocmd(
                                            c"filetypedetect BufRead".as_ptr()
                                                as *mut ::core::ffi::c_char,
                                            true_0 != 0,
                                            ::core::ptr::null_mut::<bool>(),
                                        );
                                    }
                                    do_modelines(0 as ::core::ffi::c_int);
                                }
                                fname = (*curbuf.get()).b_sfname;
                            }
                        }
                    }
                    if handle_mkdir_p_arg(eap, fname) == FAIL {
                        retval = FAIL;
                    } else {
                        let mut name_was_missing: ::core::ffi::c_int =
                            (*curbuf.get()).b_ffname.is_null() as ::core::ffi::c_int;
                        retval = buf_write(
                            curbuf.get(),
                            ffname,
                            fname,
                            (*eap).line1,
                            (*eap).line2,
                            eap,
                            WriteRequest {
                                append: (*eap).append != 0,
                                forceit: (*eap).forceit != 0,
                                reset_changed: true,
                                filtering: false,
                            },
                        );
                        if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
                            if retval == OK {
                                (*curbuf.get()).b_p_ro = false_0;
                                redraw_tabline.set(true_0 != 0);
                            }
                        }
                        if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int
                            || name_was_missing != 0
                        {
                            do_autochdir();
                        }
                    }
                }
            }
        }
        xfree(free_fname as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn check_overwrite(
    mut eap: *mut exarg_T,
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut ffname: *mut ::core::ffi::c_char,
    mut other: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if (other as ::core::ffi::c_int != 0
            || !bt_nofilename(buf)
                && ((*buf).b_flags & BF_NOTEDITED != 0
                    || (*buf).b_flags & BF_NEW != 0
                        && vim_strchr(p_cpo.get(), CPO_OVERNEW).is_null()
                    || (*buf).b_flags & BF_READERR != 0))
            && p_wa.get() == 0
            && os_path_exists(ffname) as ::core::ffi::c_int != 0
        {
            if (*eap).forceit == 0 && (*eap).append == 0 {
                if os_isdir(ffname) {
                    semsg_c!(
                        gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
                        ffname,
                    );
                    return FAIL;
                }
                if p_confirm.get() != 0
                    || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                {
                    let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
                    dialog_msg(
                        &raw mut buff as *mut ::core::ffi::c_char,
                        gettext(c"Overwrite existing file \"%s\"?".as_ptr()),
                        fname,
                    );
                    if vim_dialog_yesno(
                        VIM_QUESTION as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut buff as *mut ::core::ffi::c_char,
                        2 as ::core::ffi::c_int,
                    ) != VIM_YES as ::core::ffi::c_int
                    {
                        return FAIL;
                    }
                    (*eap).forceit = true_0;
                } else {
                    emsg(gettext(&raw const e_exists as *const ::core::ffi::c_char));
                    return FAIL;
                }
            }
            if other as ::core::ffi::c_int != 0 && emsg_silent.get() == 0 {
                let mut dir: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if *p_dir.get() as ::core::ffi::c_int == NUL {
                    dir = xmalloc(5 as size_t) as *mut ::core::ffi::c_char;
                    strcpy(dir, c".".as_ptr() as *mut ::core::ffi::c_char);
                } else {
                    dir = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                    let mut p: *mut ::core::ffi::c_char = p_dir.get();
                    copy_option_part(
                        &raw mut p,
                        dir,
                        MAXPATHL as size_t,
                        c",".as_ptr() as *mut ::core::ffi::c_char,
                    );
                }
                let mut swapname: *mut ::core::ffi::c_char =
                    makeswapname(fname, ffname, curbuf.get(), dir);
                xfree(dir as *mut ::core::ffi::c_void);
                if os_path_exists(swapname) {
                    if p_confirm.get() != 0
                        || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                    {
                        let mut buff_0: [::core::ffi::c_char; 1000] = [0; 1000];
                        dialog_msg(
                            &raw mut buff_0 as *mut ::core::ffi::c_char,
                            gettext(c"Swap file \"%s\" exists, overwrite anyway?".as_ptr()),
                            swapname,
                        );
                        if vim_dialog_yesno(
                            VIM_QUESTION as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut buff_0 as *mut ::core::ffi::c_char,
                            2 as ::core::ffi::c_int,
                        ) != VIM_YES as ::core::ffi::c_int
                        {
                            xfree(swapname as *mut ::core::ffi::c_void);
                            return FAIL;
                        }
                        (*eap).forceit = true_0;
                    } else {
                        semsg_c!(
                            gettext(c"E768: Swap file exists: %s (:silent! overrides)".as_ptr(),),
                            swapname,
                        );
                        xfree(swapname as *mut ::core::ffi::c_void);
                        return FAIL;
                    }
                }
                xfree(swapname as *mut ::core::ffi::c_void);
            }
        }
        return OK;
    }
}

pub unsafe fn ex_wnext(mut eap: *mut exarg_T) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        if *(*eap).cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'n' as ::core::ffi::c_int
        {
            i = (*curwin.get()).w_arg_idx + (*eap).line2 as ::core::ffi::c_int;
        } else {
            i = (*curwin.get()).w_arg_idx - (*eap).line2 as ::core::ffi::c_int;
        }
        (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
        (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        if do_write(eap) != FAIL {
            do_argfile(eap, i);
        }
    }
}

pub unsafe fn do_wqall(mut eap: *mut exarg_T) {
    unsafe {
        let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut save_forceit: ::core::ffi::c_int = (*eap).forceit;
        let mut save_exiting: bool = exiting.get();
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_xall as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_wqall as ::core::ffi::c_int
        {
            if before_quit_all(eap) == FAIL {
                return;
            }
            exiting.set(true_0 != 0);
        }
        let mut buf: *mut buf_T = firstbuf.get();
        's_136: while !buf.is_null() {
            's_32: {
                if exiting.get() as ::core::ffi::c_int != 0
                    && (*eap).forceit == 0
                    && !(*buf).terminal.is_null()
                    && channel_job_running((*buf).b_p_channel as uint64_t) as ::core::ffi::c_int
                        != 0
                {
                    no_write_message_buf(buf);
                    error += 1;
                } else if !bufIsChanged(buf) || bt_dontwrite(buf) as ::core::ffi::c_int != 0 {
                    break 's_32;
                }
                if not_writing() {
                    error += 1;
                    break 's_136;
                } else {
                    if (*buf).b_ffname.is_null() {
                        semsg_c!(
                            gettext(c"E141: No file name for buffer %ld".as_ptr()),
                            (*buf).handle as int64_t,
                        );
                        error += 1;
                    } else if check_readonly(&raw mut (*eap).forceit, buf) != 0
                        || check_overwrite(eap, buf, (*buf).b_fname, (*buf).b_ffname, false_0 != 0)
                            == FAIL
                    {
                        error += 1;
                    } else {
                        let mut bufref: bufref_T = bufref_T::default();
                        set_bufref(&raw mut bufref, buf);
                        if handle_mkdir_p_arg(eap, (*buf).b_fname) == FAIL
                            || buf_write_all(buf, (*eap).forceit != 0) == FAIL
                        {
                            error += 1;
                        }
                        if !bufref_valid(&raw mut bufref) {
                            buf = firstbuf.get();
                        }
                    }
                    (*eap).forceit = save_forceit;
                }
            }
            buf = (*buf).b_next;
        }
        if exiting.get() {
            if error == 0 {
                getout(0 as ::core::ffi::c_int);
            }
            not_exiting(save_exiting);
        }
    }
}

unsafe extern "C" fn not_writing() -> bool {
    unsafe {
        if p_write.get() != 0 {
            return false_0 != 0;
        }
        emsg(gettext(
            c"E142: File not written: Writing is disabled by 'write' option".as_ptr(),
        ));
        return true_0 != 0;
    }
}

unsafe extern "C" fn check_readonly(
    mut forceit: *mut ::core::ffi::c_int,
    mut buf: *mut buf_T,
) -> ::core::ffi::c_int {
    unsafe {
        if *forceit == 0
            && ((*buf).b_p_ro != 0
                || os_path_exists((*buf).b_ffname) as ::core::ffi::c_int != 0
                    && os_file_is_writable((*buf).b_ffname) == 0)
        {
            if (p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
                && !(*buf).b_fname.is_null()
            {
                let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
                if (*buf).b_p_ro != 0 {
                    dialog_msg(
                        &raw mut buff as *mut ::core::ffi::c_char,
                        gettext(
                            c"'readonly' option is set for \"%s\".\nDo you wish to write anyway?"
                                .as_ptr(),
                        ),
                        (*buf).b_fname,
                    );
                } else {
                    dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        c"File permissions of \"%s\" are read-only.\nIt may still be possible to write it.\nDo you wish to try?"
                            .as_ptr(),
                    ),
                    (*buf).b_fname,
                );
                }
                if vim_dialog_yesno(
                    VIM_QUESTION as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut buff as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) == VIM_YES as ::core::ffi::c_int
                {
                    *forceit = true_0;
                    return false_0;
                }
                return true_0;
            } else if (*buf).b_p_ro != 0 {
                emsg(gettext(&raw const e_readonly as *const ::core::ffi::c_char));
            } else {
                semsg_c!(
                    gettext(c"E505: \"%s\" is read-only (add ! to override)".as_ptr()),
                    (*buf).b_fname,
                );
            }
            return true_0;
        }
        return false_0;
    }
}

pub unsafe extern "C" fn getfile(
    mut fnum: ::core::ffi::c_int,
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut setpm: bool,
    mut lnum: linenr_T,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if !check_can_set_curbuf_forceit(forceit as ::core::ffi::c_int) {
            return GETFILE_ERROR as ::core::ffi::c_int;
        }
        let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
        let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
        let mut other: bool = false;
        let mut retval: ::core::ffi::c_int = 0;
        let mut free_me: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if text_locked() {
            return GETFILE_ERROR as ::core::ffi::c_int;
        }
        if curbuf_locked() {
            return GETFILE_ERROR as ::core::ffi::c_int;
        }
        if fnum == 0 as ::core::ffi::c_int {
            fname_expand(curbuf.get(), &raw mut ffname, &raw mut sfname);
            other = otherfile(ffname);
            free_me = ffname;
        } else {
            other = fnum != (*curbuf.get()).handle;
        }
        if other {
            (*no_wait_return.ptr()) += 1;
        }
        '_theend: {
            if other as ::core::ffi::c_int != 0
                && !forceit
                && (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
                && !buf_hide(curbuf.get())
                && curbufIsChanged() as ::core::ffi::c_int != 0
                && autowrite(curbuf.get(), forceit) == FAIL
            {
                if p_confirm.get() != 0 && p_write.get() != 0 {
                    dialog_changed(curbuf.get(), false_0 != 0);
                }
                if curbufIsChanged() {
                    (*no_wait_return.ptr()) -= 1;
                    no_write_message();
                    retval = GETFILE_NOT_WRITTEN as ::core::ffi::c_int;
                    break '_theend;
                }
            }
            if other {
                (*no_wait_return.ptr()) -= 1;
            }
            if setpm {
                setpcmark();
            }
            if !other {
                if lnum != 0 as linenr_T {
                    (*curwin.get()).w_cursor.lnum = lnum;
                }
                check_cursor_lnum(curwin.get());
                beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                retval = GETFILE_SAME_FILE as ::core::ffi::c_int;
            } else if do_ecmd(
                fnum,
                ffname,
                sfname,
                ::core::ptr::null_mut::<exarg_T>(),
                lnum,
                (if buf_hide(curbuf.get()) as ::core::ffi::c_int != 0 {
                    ECMD_HIDE as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) + (if forceit as ::core::ffi::c_int != 0 {
                    ECMD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
                curwin.get(),
            ) == OK
            {
                retval = GETFILE_OPEN_OTHER as ::core::ffi::c_int;
            } else {
                retval = GETFILE_ERROR as ::core::ffi::c_int;
            }
        }
        xfree(free_me as *mut ::core::ffi::c_void);
        return retval;
    }
}
