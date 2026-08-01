//! `:helpgrep`, which searches the help files.
//!
//! [`ex_helpgrep`] walks every `doc/` directory in `'runtimepath'`
//! ([`hgr_search_in_rtp`]) and matches the pattern against each help file's
//! lines ([`hgr_search_file`]), building a list without ever loading a
//! buffer.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn hgr_get_ll(mut new_ll: *mut bool) -> *mut qf_info_T {
    unsafe {
        let mut wp: *mut win_T = if bt_help((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0 {
            curwin.get()
        } else {
            qf_find_help_win()
        };
        let mut qi: *mut qf_info_T = if wp.is_null() {
            ::core::ptr::null_mut::<qf_info_T>()
        } else {
            (*wp).w_llist
        };
        if qi.is_null() {
            qi = qf_alloc_stack(QFLT_LOCATION, 1 as ::core::ffi::c_int);
            *new_ll = true_0 != 0;
        }
        return qi;
    }
}

pub(crate) unsafe extern "C" fn hgr_search_file(
    mut qfl: *mut qf_list_T,
    mut fname: *mut ::core::ffi::c_char,
    mut p_regmatch: *mut regmatch_T,
) {
    unsafe {
        let fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
        if fd.is_null() {
            return;
        }
        let mut lnum: linenr_T = 1 as linenr_T;
        while !vim_fgets(IObuff.ptr() as *mut ::core::ffi::c_char, IOSIZE, fd) && !got_int.get() {
            let mut line: *mut ::core::ffi::c_char = IObuff.ptr() as *mut ::core::ffi::c_char;
            if vim_regexec(p_regmatch, line, 0 as colnr_T) {
                let mut l: ::core::ffi::c_int = strlen(line) as ::core::ffi::c_int;
                while l > 0 as ::core::ffi::c_int
                    && *line.offset((l - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        <= ' ' as ::core::ffi::c_int
                {
                    l -= 1;
                    *line.offset(l as isize) = NUL as ::core::ffi::c_char;
                }
                qf_add_entry(
                    qfl,
                    &NewEntry {
                        fname,
                        lnum,
                        col: (*p_regmatch).startp[0 as ::core::ffi::c_int as usize]
                            .offset_from(line) as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int,
                        end_col: (*p_regmatch).endp[0 as ::core::ffi::c_int as usize]
                            .offset_from(line)
                            as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int,
                        // A help entry, which `qf_jump` opens as help.
                        kind: 1 as ::core::ffi::c_char,
                        ..NewEntry::new(line)
                    },
                );
            }
            if line != IObuff.ptr() as *mut ::core::ffi::c_char {
                xfree(line as *mut ::core::ffi::c_void);
            }
            lnum += 1;
            line_breakcheck();
        }
        fclose(fd);
    }
}

pub(crate) unsafe extern "C" fn hgr_search_files_in_dir(
    mut qfl: *mut qf_list_T,
    mut dirname: *mut ::core::ffi::c_char,
    mut p_regmatch: *mut regmatch_T,
    mut lang: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut fcount: ::core::ffi::c_int = 0;
        let mut fnames: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        add_pathsep(dirname);
        strcat(
            dirname,
            b"doc/*.\\(txt\\|??x\\)\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if gen_expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut dirname,
            &raw mut fcount,
            &raw mut fnames,
            EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
        ) == OK
            && fcount > 0 as ::core::ffi::c_int
        {
            let mut fi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while fi < fcount && !got_int.get() {
                if !(!lang.is_null()
                    && strncasecmp(
                        lang as *mut ::core::ffi::c_char,
                        (*fnames.offset(fi as isize))
                            .offset(strlen(*fnames.offset(fi as isize)) as isize)
                            .offset(-(3 as ::core::ffi::c_int as isize)),
                        2 as ::core::ffi::c_int as size_t,
                    ) != 0 as ::core::ffi::c_int
                    && !(strncasecmp(
                        lang as *mut ::core::ffi::c_char,
                        b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        2 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                        && strncasecmp(
                            b"txt\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fnames.offset(fi as isize))
                                .offset(strlen(*fnames.offset(fi as isize)) as isize)
                                .offset(-(3 as ::core::ffi::c_int as isize)),
                            3 as ::core::ffi::c_int as size_t,
                        ) == 0 as ::core::ffi::c_int))
                {
                    hgr_search_file(qfl, *fnames.offset(fi as isize), p_regmatch);
                }
                fi += 1;
            }
            FreeWild(fcount, fnames);
        }
    }
}

pub(crate) unsafe extern "C" fn hgr_search_in_rtp(
    mut qfl: *mut qf_list_T,
    mut p_regmatch: *mut regmatch_T,
    mut lang: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = p_rtp.get();
        while *p as ::core::ffi::c_int != NUL && !got_int.get() {
            copy_option_part(
                &raw mut p,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            hgr_search_files_in_dir(
                qfl,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                p_regmatch,
                lang,
            );
        }
    }
}

pub unsafe fn ex_helpgrep(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    7575 as ::core::ffi::c_uint,
                    b"void ex_helpgrep(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut au_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match (*eap).cmdidx as ::core::ffi::c_int {
            178 => {
                au_name = b"helpgrep\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            241 => {
                au_name = b"lhelpgrep\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            _ => {}
        }
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
        let mut updated: bool = false_0 != 0;
        let save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
        p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
        let mut new_qi: bool = false_0 != 0;
        if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
            qi = hgr_get_ll(&raw mut new_qi);
        }
        incr_quickfix_busy();
        let lang: *mut ::core::ffi::c_char = check_help_lang((*eap).arg);
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: vim_regcomp((*eap).arg, RE_MAGIC + RE_STRING),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false_0 != 0,
        };
        if !regmatch.regprog.is_null() {
            qf_new_list(qi, qf_cmdtitle(*(*eap).cmdlinep));
            let qfl: *mut qf_list_T = qf_get_curlist(qi);
            hgr_search_in_rtp(qfl, &raw mut regmatch, lang);
            vim_regfree(regmatch.regprog);
            (*qfl).qf_nonevalid = false_0 != 0;
            (*qfl).qf_ptr = (*qfl).qf_start;
            (*qfl).qf_index = 1 as ::core::ffi::c_int;
            qf_list_changed(qfl);
            updated = true_0 != 0;
        }
        if p_cpo.get() == empty_string_option.ptr() as *mut ::core::ffi::c_char {
            p_cpo.set(save_cpo);
        } else {
            if *p_cpo.get() as ::core::ffi::c_int == NUL {
                set_option_value_give_err(
                    kOptCpoptions,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(save_cpo),
                        },
                    },
                    0 as ::core::ffi::c_int,
                );
            }
            free_string_option(save_cpo);
        }
        if updated {
            qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
        }
        if !au_name.is_null() {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                au_name,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            );
            if !new_qi
                && (*qi).qfl_type as ::core::ffi::c_uint
                    == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
                && qf_find_win_with_loclist(qi).is_null()
            {
                decr_quickfix_busy();
                return;
            }
        }
        if !qf_list_empty(qf_get_curlist(qi)) {
            qf_jump(
                qi,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0,
            );
        } else {
            semsg(
                gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        }
        decr_quickfix_busy();
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_lhelpgrep as ::core::ffi::c_int {
            if !bt_help((*curwin.get()).w_buffer) || (*curwin.get()).w_llist == qi {
                if new_qi {
                    ll_free_all(&raw mut qi);
                }
            } else if (*curwin.get()).w_llist.is_null() && new_qi as ::core::ffi::c_int != 0 {
                (*curwin.get()).w_llist = qi;
            }
        }
    }
}
