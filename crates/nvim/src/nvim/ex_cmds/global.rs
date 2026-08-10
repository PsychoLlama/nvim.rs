//! `:global` and `:vglobal` -- run a command on every line that matches.
//!
//! The two-pass shape is the whole point: `ex_global` marks every matching line
//! first, then `global_exe` runs the command on the marks, so that the command
//! may delete, add and move lines without the scan losing its place.  Each
//! execution re-enters `do_cmdline`, which is why an error, an interrupt or a
//! `:global` nested inside another has to be handled here rather than by the
//! caller.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn global_exe_one(cmd: *mut ::core::ffi::c_char, lnum: linenr_T) {
    unsafe {
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        if *cmd as ::core::ffi::c_int == NUL
            || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            do_cmdline(
                c"p".as_ptr() as *mut ::core::ffi::c_char,
                None,
                NULL_0,
                DOCMD_NOWAIT as ::core::ffi::c_int,
            );
        } else {
            do_cmdline(cmd, None, NULL_0, DOCMD_NOWAIT as ::core::ffi::c_int);
        };
    }
}

pub unsafe fn ex_global(mut eap: *mut exarg_T) {
    unsafe {
        let mut lnum: linenr_T = 0;
        let mut type_0: ::core::ffi::c_int = 0;
        let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut delim: ::core::ffi::c_char = 0;
        let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut patlen: size_t = 0;
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        if global_busy.get() != 0
            && ((*eap).line1 != 1 as linenr_T || (*eap).line2 != (*curbuf.get()).b_ml.ml_line_count)
        {
            emsg(gettext(
                c"E147: Cannot do :global recursive with a range".as_ptr(),
            ));
            return;
        }
        if (*eap).forceit != 0 {
            type_0 = 'v' as ::core::ffi::c_int;
        } else {
            type_0 = *(*eap).cmd as uint8_t as ::core::ffi::c_int;
        }
        cmd = (*eap).arg;
        let mut which_pat: ::core::ffi::c_int = RE_LAST as ::core::ffi::c_int;
        if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            cmd = cmd.offset(1);
            if vim_strchr(c"/?&".as_ptr(), *cmd as uint8_t as ::core::ffi::c_int).is_null() {
                emsg(gettext(
                    &raw const e_backslash as *const ::core::ffi::c_char,
                ));
                return;
            }
            if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
                which_pat = RE_SUBST as ::core::ffi::c_int;
            } else {
                which_pat = RE_SEARCH as ::core::ffi::c_int;
            }
            cmd = cmd.offset(1);
            pat = c"".as_ptr() as *mut ::core::ffi::c_char;
            patlen = 0 as size_t;
        } else if *cmd as ::core::ffi::c_int == NUL {
            emsg(gettext(
                c"E148: Regular expression missing from global".as_ptr(),
            ));
            return;
        } else if check_regexp_delim(*cmd as ::core::ffi::c_int) == FAIL {
            return;
        } else {
            delim = *cmd;
            cmd = cmd.offset(1);
            pat = cmd;
            cmd = skip_regexp_ex(
                cmd,
                delim as ::core::ffi::c_int,
                magic_isset() as ::core::ffi::c_int,
                &raw mut (*eap).arg,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<magic_T>(),
            );
            if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == delim as ::core::ffi::c_int
            {
                let c2rust_fresh5 = cmd;
                cmd = cmd.offset(1);
                *c2rust_fresh5 = NUL as ::core::ffi::c_char;
            }
            patlen = strlen(pat);
        }
        let mut used_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if search_regcomp(
            pat,
            patlen,
            &raw mut used_pat,
            RE_BOTH as ::core::ffi::c_int,
            which_pat,
            SEARCH_HIS as ::core::ffi::c_int,
            &raw mut regmatch,
        ) == FAIL
        {
            emsg(gettext(&raw const e_invcmd as *const ::core::ffi::c_char));
            return;
        }
        if global_busy.get() != 0 {
            lnum = (*curwin.get()).w_cursor.lnum;
            let mut match_0: ::core::ffi::c_int = vim_regexec_multi(
                &raw mut regmatch,
                curwin.get(),
                curbuf.get(),
                lnum,
                0 as colnr_T,
                ::core::ptr::null_mut::<proftime_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if type_0 == 'g' as ::core::ffi::c_int && match_0 != 0
                || type_0 == 'v' as ::core::ffi::c_int && match_0 == 0
            {
                global_exe_one(cmd, lnum);
            }
        } else {
            let mut ndone: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            lnum = (*eap).line1;
            while lnum <= (*eap).line2 && !got_int.get() {
                let mut match_1: ::core::ffi::c_int = vim_regexec_multi(
                    &raw mut regmatch,
                    curwin.get(),
                    curbuf.get(),
                    lnum,
                    0 as colnr_T,
                    ::core::ptr::null_mut::<proftime_T>(),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                );
                if regmatch.regprog.is_null() {
                    break;
                }
                if type_0 == 'g' as ::core::ffi::c_int && match_1 != 0
                    || type_0 == 'v' as ::core::ffi::c_int && match_1 == 0
                {
                    ml_setmarked(lnum);
                    ndone += 1;
                }
                line_breakcheck();
                lnum += 1;
            }
            if got_int.get() {
                msg(
                    gettext(&raw const e_interr as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
            } else if ndone == 0 as ::core::ffi::c_int {
                if type_0 == 'v' as ::core::ffi::c_int {
                    smsg_c!(
                        0 as ::core::ffi::c_int,
                        gettext(c"Pattern found in every line: %s".as_ptr()),
                        used_pat,
                    );
                } else {
                    smsg_c!(
                        0 as ::core::ffi::c_int,
                        gettext(c"Pattern not found: %s".as_ptr()),
                        used_pat,
                    );
                }
            } else {
                global_exe(cmd);
            }
            ml_clearmarked();
        }
        vim_regfree(regmatch.regprog);
    }
}

pub unsafe extern "C" fn global_exe(mut cmd: *mut ::core::ffi::c_char) {
    unsafe {
        let mut old_lcount: linenr_T = 0;
        let mut old_buf: *mut buf_T = curbuf.get();
        let mut lnum: linenr_T = 0;
        setpcmark();
        msg_didout.set(true_0 != 0);
        sub_nsubs.set(0 as ::core::ffi::c_int);
        sub_nlines.set(0 as ::core::ffi::c_int as linenr_T);
        global_need_msg_kind.set(true_0 != 0);
        global_need_beginline.set(false_0);
        global_busy.set(1 as ::core::ffi::c_int);
        old_lcount = (*curbuf.get()).b_ml.ml_line_count;
        while !got_int.get()
            && {
                lnum = ml_firstmarked();
                lnum != 0 as linenr_T
            }
            && global_busy.get() == 1 as ::core::ffi::c_int
        {
            global_exe_one(cmd, lnum);
            os_breakcheck();
        }
        global_busy.set(0 as ::core::ffi::c_int);
        if global_need_beginline.get() != 0 {
            beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        } else {
            check_cursor(curwin.get());
        }
        changed_line_abv_curs();
        if msg_col.get() == 0 as ::core::ffi::c_int && msg_scrolled.get() == 0 as ::core::ffi::c_int
        {
            msg_didout.set(false_0 != 0);
        }
        if !do_sub_msg(false_0 != 0) && curbuf.get() == old_buf {
            msgmore(
                (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int
                    - old_lcount as ::core::ffi::c_int,
            );
        }
    }
}
