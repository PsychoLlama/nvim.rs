//! `:vimgrep`, the built-in grep.
//!
//! [`ex_vimgrep`] loads each file named on the command line — into a real
//! buffer if one is already open, otherwise into a throwaway one
//! ([`load_dummy_buffer`]) — and [`vgr_match_buflines`] runs the pattern
//! over its lines, recording every match as an entry.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn vgr_get_auname(mut cmdidx: cmdidx_T) -> *mut ::core::ffi::c_char {
    match cmdidx as ::core::ffi::c_int {
        510 => {
            return b"vimgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        267 => {
            return b"lvimgrep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        511 => {
            return b"vimgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        268 => {
            return b"lvimgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        172 => {
            return b"grep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        239 => {
            return b"lgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        173 => {
            return b"grepadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        240 => {
            return b"lgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}

pub(crate) unsafe extern "C" fn vgr_init_regmatch(
    mut regmatch: *mut regmmatch_T,
    mut s: *mut ::core::ffi::c_char,
) {
    unsafe {
        (*regmatch).regprog = ::core::ptr::null_mut::<regprog_T>();
        if s.is_null() || *s as ::core::ffi::c_int == NUL {
            if last_search_pat().is_null() {
                emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                return;
            }
            (*regmatch).regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
        } else {
            (*regmatch).regprog = vim_regcomp(s, RE_MAGIC);
        }
        (*regmatch).rmm_ic = p_ic.get();
        (*regmatch).rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
    }
}

pub(crate) unsafe extern "C" fn vgr_display_fname(mut fname: *mut ::core::ffi::c_char) {
    unsafe {
        msg_start();
        let mut p: *mut ::core::ffi::c_char = msg_strtrunc(fname, true_0);
        if p.is_null() {
            msg_outtrans(fname, 0 as ::core::ffi::c_int, false_0 != 0);
        } else {
            msg_outtrans(p, 0 as ::core::ffi::c_int, false_0 != 0);
            xfree(p as *mut ::core::ffi::c_void);
        }
        msg_clr_eos();
        msg_didout.set(false_0 != 0);
        msg_nowait.set(true_0 != 0);
        msg_col.set(0 as ::core::ffi::c_int);
        ui_flush();
    }
}

pub(crate) unsafe extern "C" fn vgr_load_dummy_buf(
    mut fname: *mut ::core::ffi::c_char,
    mut dirname_start: *mut ::core::ffi::c_char,
    mut dirname_now: *mut ::core::ffi::c_char,
) -> *mut buf_T {
    unsafe {
        let mut save_ei: *mut ::core::ffi::c_char = au_event_disable(b",Filetype\0".as_ptr()
            as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char);
        let mut save_mls: OptInt = p_mls.get();
        p_mls.set(0 as OptInt);
        let mut buf: *mut buf_T = load_dummy_buffer(fname, dirname_start, dirname_now);
        p_mls.set(save_mls);
        au_event_restore(save_ei);
        return buf;
    }
}

pub(crate) unsafe extern "C" fn vgr_qflist_valid(
    mut wp: *mut win_T,
    mut qi: *mut qf_info_T,
    mut qfid: ::core::ffi::c_uint,
    mut title: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        if !qflist_valid(wp, qfid) {
            if !wp.is_null() {
                emsg(gettext(e_current_location_list_was_changed.get()));
                return false_0 != 0;
            }
            qf_new_list(qi, title);
            return true_0 != 0;
        }
        if qf_restore_list(qi, qfid) == FAIL {
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn vgr_match_buflines(
    mut qfl: *mut qf_list_T,
    mut fname: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut spat: *mut ::core::ffi::c_char,
    mut regmatch: *mut regmmatch_T,
    mut tomatch: *mut ::core::ffi::c_int,
    mut duplicate_name: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut found_match: bool = false_0 != 0;
        let mut pat_len: size_t = strlen(spat);
        pat_len = if pat_len < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t {
            pat_len
        } else {
            FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t
        };
        let mut lnum: linenr_T = 1 as linenr_T;
        while lnum <= (*buf).b_ml.ml_line_count && *tomatch > 0 as ::core::ffi::c_int {
            let mut col: colnr_T = 0 as colnr_T;
            if flags & VGR_FUZZY as ::core::ffi::c_int == 0 {
                while vim_regexec_multi(
                    regmatch,
                    curwin.get(),
                    buf,
                    lnum,
                    col,
                    ::core::ptr::null_mut::<proftime_T>(),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ) > 0 as ::core::ffi::c_int
                {
                    qf_add_entry(
                        qfl,
                        &NewEntry {
                            fname,
                            bufnum: if duplicate_name != 0 {
                                0 as ::core::ffi::c_int
                            } else {
                                (*buf).handle as ::core::ffi::c_int
                            },
                            lnum: (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum
                                + lnum,
                            end_lnum: (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum
                                + lnum,
                            col: (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col
                                as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int,
                            end_col: (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col
                                as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int,
                            ..NewEntry::new(ml_get_buf(
                                buf,
                                (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum + lnum,
                            ))
                        },
                    );
                    {
                        found_match = true_0 != 0;
                        *tomatch -= 1;
                        if *tomatch == 0 as ::core::ffi::c_int {
                            break;
                        }
                        if flags & VGR_GLOBAL as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                            || (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum
                                > 0 as linenr_T
                        {
                            break;
                        }
                        col = ((*regmatch).endpos[0 as ::core::ffi::c_int as usize].col
                            as ::core::ffi::c_int
                            + (col == (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col)
                                as ::core::ffi::c_int) as colnr_T;
                        if col > ml_get_buf_len(buf, lnum) {
                            break;
                        }
                    }
                }
            } else {
                let str: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
                let linelen: colnr_T = ml_get_buf_len(buf, lnum);
                let mut score: ::core::ffi::c_int = 0;
                let mut matches: [uint32_t; 1024] = [0; 1024];
                let sz: size_t = ::core::mem::size_of::<[uint32_t; 1024]>()
                    .wrapping_div(::core::mem::size_of::<uint32_t>());
                memset(
                    &raw mut matches as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<[uint32_t; 1024]>(),
                );
                while fuzzy_match(
                    str.offset(col as isize),
                    spat,
                    false_0 != 0,
                    &raw mut score,
                    &raw mut matches as *mut uint32_t,
                    sz as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    > 0 as ::core::ffi::c_int
                {
                    qf_add_entry(
                        qfl,
                        &NewEntry {
                            fname,
                            bufnum: if duplicate_name != 0 {
                                0 as ::core::ffi::c_int
                            } else {
                                (*buf).handle as ::core::ffi::c_int
                            },
                            lnum,
                            col: matches[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                                + col as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int,
                            ..NewEntry::new(str)
                        },
                    );
                    {
                        found_match = true_0 != 0;
                        *tomatch -= 1;
                        if *tomatch == 0 as ::core::ffi::c_int {
                            break;
                        }
                        if flags & VGR_GLOBAL as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            break;
                        }
                        col = (matches[pat_len.wrapping_sub(1 as size_t) as usize]
                            as ::core::ffi::c_int
                            + col as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int) as colnr_T;
                        if col > linelen {
                            break;
                        }
                    }
                }
            }
            line_breakcheck();
            if got_int.get() {
                break;
            }
            lnum += 1;
        }
        return found_match;
    }
}

pub(crate) unsafe extern "C" fn vgr_jump_to_match(
    mut qi: *mut qf_info_T,
    mut forceit: ::core::ffi::c_int,
    mut redraw_for_dummy: *mut bool,
    mut first_match_buf: *mut buf_T,
    mut target_dir: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut buf: *mut buf_T = curbuf.get();
        qf_jump(
            qi,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            forceit,
        );
        if buf != curbuf.get() {
            *redraw_for_dummy = false_0 != 0;
        }
        if curbuf.get() == first_match_buf && !target_dir.is_null() {
            let mut ea: exarg_T = exarg {
                arg: target_dir,
                args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                arglens: ::core::ptr::null_mut::<size_t>(),
                argc: 0,
                nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdidx: CMD_lcd,
                argt: 0,
                skip: 0,
                forceit: 0,
                addr_count: 0,
                line1: 0,
                line2: 0,
                addr_type: ADDR_LINES,
                flags: 0,
                do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                do_ecmd_lnum: 0,
                append: 0,
                usefilter: 0,
                amount: 0,
                regname: 0,
                force_bin: 0,
                read_edit: 0,
                mkdir_p: 0,
                force_ff: 0,
                force_enc: 0,
                bad_char: 0,
                useridx: 0,
                errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ea_getline: None,
                cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                cstack: ::core::ptr::null_mut::<cstack_T>(),
            };
            ex_cd(&raw mut ea);
        }
    }
}

pub(crate) unsafe extern "C" fn existing_swapfile(mut buf: *const buf_T) -> bool {
    unsafe {
        if !(*buf).b_ml.ml_mfp.is_null() && !mf_fname((*buf).b_ml.ml_mfp).is_null() {
            let fname: *const ::core::ffi::c_char = mf_fname((*buf).b_ml.ml_mfp);
            let len: size_t = strlen(fname);
            return *fname.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                != 'p' as ::core::ffi::c_int
                || *fname.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                    != 'w' as ::core::ffi::c_int;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn vgr_process_args(
    mut eap: *mut exarg_T,
    mut args: *mut vgr_args_T,
) -> ::core::ffi::c_int {
    unsafe {
        memset(
            args as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<vgr_args_T>(),
        );
        (*args).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
        (*args).qf_title = xstrdup(qf_cmdtitle(*(*eap).cmdlinep));
        (*args).tomatch = (if (*eap).addr_count > 0 as ::core::ffi::c_int {
            (*eap).line2
        } else {
            MAXLNUM as ::core::ffi::c_int as linenr_T
        }) as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char =
            skip_vimgrep_pat((*eap).arg, &raw mut (*args).spat, &raw mut (*args).flags);
        if p.is_null() {
            emsg(gettext(&raw const e_invalpat as *const ::core::ffi::c_char));
            return FAIL;
        }
        vgr_init_regmatch(&raw mut (*args).regmatch, (*args).spat);
        if (*args).regmatch.regprog.is_null() {
            return FAIL;
        }
        p = skipwhite(p);
        if *p as ::core::ffi::c_int == NUL {
            emsg(gettext(
                b"E683: File name missing or invalid pattern\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if get_arglist_exp(
            p,
            &raw mut (*args).fcount,
            &raw mut (*args).fnames,
            true_0 != 0,
        ) == FAIL
            || (*args).fcount == 0 as ::core::ffi::c_int
        {
            emsg(gettext(&raw const e_nomatch as *const ::core::ffi::c_char));
            return FAIL;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn vgr_process_files(
    mut wp: *mut win_T,
    mut qi: *mut qf_info_T,
    mut cmd_args: *mut vgr_args_T,
    mut redraw_for_dummy: *mut bool,
    mut first_match_buf: *mut *mut buf_T,
    mut target_dir: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = FAIL;
        let mut save_qfid: ::core::ffi::c_uint = (*qf_get_curlist(qi)).qf_id;
        let mut duplicate_name: bool = false_0 != 0;
        let mut dirname_start: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut dirname_now: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        os_dirname(dirname_start, MAXPATHL as size_t);
        let mut seconds: time_t = 0 as time_t;
        let mut fi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        '_theend: {
            while fi < (*cmd_args).fcount
                && !got_int.get()
                && (*cmd_args).tomatch > 0 as ::core::ffi::c_int
            {
                let mut fname: *mut ::core::ffi::c_char =
                    path_try_shorten_fname(*(*cmd_args).fnames.offset(fi as isize));
                if time(::core::ptr::null_mut::<time_t>()) > seconds {
                    seconds = time(::core::ptr::null_mut::<time_t>());
                    vgr_display_fname(fname);
                }
                let mut buf: *mut buf_T =
                    buflist_findname_exp(*(*cmd_args).fnames.offset(fi as isize));
                let mut using_dummy: bool = false;
                if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
                    duplicate_name = !buf.is_null();
                    using_dummy = true_0 != 0;
                    *redraw_for_dummy = true_0 != 0;
                    buf = vgr_load_dummy_buf(fname, dirname_start, dirname_now);
                } else {
                    using_dummy = false_0 != 0;
                }
                if !vgr_qflist_valid(wp, qi, save_qfid, (*cmd_args).qf_title) {
                    break '_theend;
                }
                save_qfid = (*qf_get_curlist(qi)).qf_id;
                if buf.is_null() {
                    if !got_int.get() {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Cannot open file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            fname,
                        );
                    }
                } else {
                    let mut found_match: bool = vgr_match_buflines(
                        qf_get_curlist(qi),
                        fname,
                        buf,
                        (*cmd_args).spat,
                        &raw mut (*cmd_args).regmatch,
                        &raw mut (*cmd_args).tomatch,
                        duplicate_name as ::core::ffi::c_int,
                        (*cmd_args).flags,
                    );
                    if using_dummy {
                        if found_match as ::core::ffi::c_int != 0 && (*first_match_buf).is_null() {
                            *first_match_buf = buf;
                        }
                        if duplicate_name {
                            wipe_dummy_buffer(buf, dirname_start);
                            buf = ::core::ptr::null_mut::<buf_T>();
                        } else if (*cmdmod.ptr()).cmod_flags & CMOD_HIDE as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                            || *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'u' as ::core::ffi::c_int
                            || *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'w' as ::core::ffi::c_int
                            || *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'd' as ::core::ffi::c_int
                        {
                            if !found_match {
                                wipe_dummy_buffer(buf, dirname_start);
                                buf = ::core::ptr::null_mut::<buf_T>();
                            } else if buf != *first_match_buf
                                || (*cmd_args).flags & VGR_NOJUMP as ::core::ffi::c_int != 0
                                || existing_swapfile(buf) as ::core::ffi::c_int != 0
                            {
                                unload_dummy_buffer(buf, dirname_start);
                                (*buf).b_flags &= !BF_DUMMY;
                                buf = ::core::ptr::null_mut::<buf_T>();
                            }
                        }
                        if !buf.is_null() {
                            (*buf).b_flags &= !BF_DUMMY;
                            if buf == *first_match_buf
                                && (*target_dir).is_null()
                                && strcmp(dirname_start, dirname_now) != 0 as ::core::ffi::c_int
                            {
                                *target_dir = xstrdup(dirname_now);
                            }
                            let mut aco: aco_save_T = aco_save_T::default();
                            aucmd_prepbuf(&raw mut aco, buf);
                            apply_autocmds(
                                EVENT_FILETYPE,
                                (*buf).b_p_ft,
                                (*buf).b_fname,
                                true_0 != 0,
                                buf,
                            );
                            do_modelines(OPT_NOWIN as ::core::ffi::c_int);
                            aucmd_restbuf(&raw mut aco);
                        }
                    }
                }
                fi += 1;
            }
            status = OK;
        }
        xfree(dirname_now as *mut ::core::ffi::c_void);
        xfree(dirname_start as *mut ::core::ffi::c_void);
        return status;
    }
}

pub unsafe fn ex_vimgrep(mut eap: *mut exarg_T) {
    unsafe {
        let mut redraw_for_dummy: bool = false;
        let mut first_match_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut status: ::core::ffi::c_int = 0;
        let mut qfl: *mut qf_list_T = ::core::ptr::null_mut::<qf_list_T>();
        let mut save_qfid: ::core::ffi::c_uint = 0;
        if !check_can_set_curbuf_forceit((*eap).forceit) {
            return;
        }
        let mut au_name: *mut ::core::ffi::c_char = vgr_get_auname((*eap).cmdidx);
        if !au_name.is_null()
            && apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            ) as ::core::ffi::c_int
                != 0
        {
            if aborting() {
                return;
            }
        }
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut qi: *mut qf_info_T = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
        let mut target_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut args: vgr_args_T = vgr_args_T {
            tomatch: 0,
            spat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            flags: 0,
            fnames: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            fcount: 0,
            regmatch: regmmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startpos: [lpos_T { lnum: 0, col: 0 }; 10],
                endpos: [lpos_T { lnum: 0, col: 0 }; 10],
                rmm_matchcol: 0,
                rmm_ic: 0,
                rmm_maxcol: 0,
            },
            qf_title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if vgr_process_args(eap, &raw mut args) != FAIL {
            if (*eap).cmdidx as ::core::ffi::c_int != CMD_grepadd as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_lgrepadd as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_vimgrepadd as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_lvimgrepadd as ::core::ffi::c_int
                || qf_stack_empty(qi) as ::core::ffi::c_int != 0
            {
                qf_new_list(qi, args.qf_title);
            }
            incr_quickfix_busy();
            redraw_for_dummy = false_0 != 0;
            first_match_buf = ::core::ptr::null_mut::<buf_T>();
            status = vgr_process_files(
                wp,
                qi,
                &raw mut args,
                &raw mut redraw_for_dummy,
                &raw mut first_match_buf,
                &raw mut target_dir,
            );
            if status != OK {
                FreeWild(args.fcount, args.fnames);
                decr_quickfix_busy();
            } else {
                FreeWild(args.fcount, args.fnames);
                qfl = qf_get_curlist(qi);
                (*qfl).qf_nonevalid = false_0 != 0;
                (*qfl).qf_ptr = (*qfl).qf_start;
                (*qfl).qf_index = 1 as ::core::ffi::c_int;
                qf_list_changed(qfl);
                qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
                save_qfid = (*qf_get_curlist(qi)).qf_id;
                if !au_name.is_null() {
                    apply_autocmds(
                        EVENT_QUICKFIXCMDPOST,
                        au_name,
                        (*curbuf.get()).b_fname,
                        true_0 != 0,
                        curbuf.get(),
                    );
                }
                if !qflist_valid(wp, save_qfid) || qf_restore_list(qi, save_qfid) == FAIL {
                    decr_quickfix_busy();
                } else {
                    if !qf_list_empty(qf_get_curlist(qi)) {
                        if args.flags & VGR_NOJUMP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        {
                            vgr_jump_to_match(
                                qi,
                                (*eap).forceit,
                                &raw mut redraw_for_dummy,
                                first_match_buf,
                                target_dir,
                            );
                        }
                    } else {
                        semsg(
                            gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                            args.spat,
                        );
                    }
                    decr_quickfix_busy();
                    if redraw_for_dummy {
                        foldUpdateAll(curwin.get());
                    }
                }
            }
        }
        xfree(args.qf_title as *mut ::core::ffi::c_void);
        xfree(target_dir as *mut ::core::ffi::c_void);
        vim_regfree(args.regmatch.regprog);
    }
}

pub(crate) unsafe extern "C" fn restore_start_dir(mut dirname_start: *mut ::core::ffi::c_char) {
    unsafe {
        let mut dirname_now: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        os_dirname(dirname_now, MAXPATHL as size_t);
        if strcmp(dirname_start, dirname_now) != 0 as ::core::ffi::c_int {
            let mut ea: exarg_T = exarg {
                arg: dirname_start,
                args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                arglens: ::core::ptr::null_mut::<size_t>(),
                argc: 0,
                nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdidx: (if (*curwin.get()).w_localdir.is_null() {
                    CMD_cd as ::core::ffi::c_int
                } else {
                    CMD_lcd as ::core::ffi::c_int
                }) as cmdidx_T,
                argt: 0,
                skip: 0,
                forceit: 0,
                addr_count: 0,
                line1: 0,
                line2: 0,
                addr_type: ADDR_LINES,
                flags: 0,
                do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                do_ecmd_lnum: 0,
                append: 0,
                usefilter: 0,
                amount: 0,
                regname: 0,
                force_bin: 0,
                read_edit: 0,
                mkdir_p: 0,
                force_ff: 0,
                force_enc: 0,
                bad_char: 0,
                useridx: 0,
                errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ea_getline: None,
                cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                cstack: ::core::ptr::null_mut::<cstack_T>(),
            };
            ex_cd(&raw mut ea);
        }
        xfree(dirname_now as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn load_dummy_buffer(
    mut fname: *mut ::core::ffi::c_char,
    mut dirname_start: *mut ::core::ffi::c_char,
    mut resulting_dir: *mut ::core::ffi::c_char,
) -> *mut buf_T {
    unsafe {
        let mut newbuf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            BLN_DUMMY as ::core::ffi::c_int,
        );
        if newbuf.is_null() {
            return ::core::ptr::null_mut::<buf_T>();
        }
        let mut failed: bool = true_0 != 0;
        let mut newbufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut newbufref, newbuf);
        buf_copy_options(
            newbuf,
            BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
        );
        if ml_open(newbuf) == OK {
            (*newbuf).b_locked += 1;
            let mut aco: aco_save_T = aco_save_T::default();
            aucmd_prepbuf(&raw mut aco, newbuf);
            setfname(
                curbuf.get(),
                fname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
            );
            check_need_swap(true_0 != 0);
            (*curbuf.get()).b_flags &= !BF_DUMMY;
            let mut newbuf_to_wipe: bufref_T = bufref_T::default();
            newbuf_to_wipe.br_buf = ::core::ptr::null_mut::<buf_T>();
            let mut readfile_result: ::core::ffi::c_int = readfile(
                fname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as linenr_T,
                0 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                ::core::ptr::null_mut::<exarg_T>(),
                READ_NEW as ::core::ffi::c_int | READ_DUMMY as ::core::ffi::c_int,
                false_0 != 0,
            );
            (*newbuf).b_locked -= 1;
            if readfile_result == OK && !got_int.get() && (*curbuf.get()).b_flags & BF_NEW == 0 {
                failed = false_0 != 0;
                if curbuf.get() != newbuf {
                    set_bufref(&raw mut newbuf_to_wipe, newbuf);
                    newbuf = curbuf.get();
                }
            }
            aucmd_restbuf(&raw mut aco);
            if !newbuf_to_wipe.br_buf.is_null()
                && bufref_valid(&raw mut newbuf_to_wipe) as ::core::ffi::c_int != 0
            {
                block_autocmds();
                wipe_dummy_buffer(
                    newbuf_to_wipe.br_buf,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                unblock_autocmds();
            }
            (*newbuf).b_flags |= BF_DUMMY;
        }
        os_dirname(resulting_dir, MAXPATHL as size_t);
        restore_start_dir(dirname_start);
        if !bufref_valid(&raw mut newbufref) {
            return ::core::ptr::null_mut::<buf_T>();
        }
        if failed {
            wipe_dummy_buffer(newbuf, dirname_start);
            return ::core::ptr::null_mut::<buf_T>();
        }
        return newbuf;
    }
}

pub(crate) unsafe extern "C" fn wipe_dummy_buffer(
    mut buf: *mut buf_T,
    mut dirname_start: *mut ::core::ffi::c_char,
) {
    unsafe {
        '_fail: {
            // Not immutable: win_close() drops (*buf).b_nwindows behind the raw pointer.
            #[allow(clippy::while_immutable_condition)]
            while (*buf).b_nwindows > 0 as ::core::ffi::c_int {
                let mut did_one: bool = false_0 != 0;
                if !(*firstwin.get()).w_next.is_null() {
                    let mut wp: *mut win_T = firstwin.get();
                    while !wp.is_null() {
                        if (*wp).w_buffer == buf {
                            if win_close(wp, false_0 != 0, false_0 != 0) == OK {
                                did_one = true_0 != 0;
                            }
                            break;
                        } else {
                            wp = (*wp).w_next;
                        }
                    }
                }
                if !did_one {
                    break '_fail;
                }
            }
            if curbuf.get() != buf && (*buf).b_nwindows == 0 as ::core::ffi::c_int {
                let mut cs: cleanup_T = cleanup_T {
                    pending: 0,
                    exception: ::core::ptr::null_mut::<except_T>(),
                };
                enter_cleanup(&raw mut cs);
                wipe_buffer(buf, true_0 != 0);
                leave_cleanup(&raw mut cs);
                if !dirname_start.is_null() {
                    restore_start_dir(dirname_start);
                }
                return;
            }
        }
        (*buf).b_flags &= !BF_DUMMY;
    }
}

pub(crate) unsafe extern "C" fn unload_dummy_buffer(
    mut buf: *mut buf_T,
    mut dirname_start: *mut ::core::ffi::c_char,
) {
    unsafe {
        if curbuf.get() == buf {
            return;
        }
        close_buffer(
            ::core::ptr::null_mut::<win_T>(),
            buf,
            DOBUF_UNLOAD as ::core::ffi::c_int,
            false_0 != 0,
            true_0 != 0,
        );
        restore_start_dir(dirname_start);
    }
}
