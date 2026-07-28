//! Running a compiled machine: try a start position, then the entry
//! points the engine table names.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn nfa_regtry(
    mut prog: *mut nfa_regprog_T,
    mut col: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut subs: regsubs_T = regsubs_T {
        norm: regsub_T {
            in_use: 0,
            list: C2Rust_Unnamed_19 {
                multi: [multipos {
                    start_lnum: 0,
                    end_lnum: 0,
                    start_col: 0,
                    end_col: 0,
                }; 10],
            },
            orig_start_col: 0,
        },
        synt: regsub_T {
            in_use: 0,
            list: C2Rust_Unnamed_19 {
                multi: [multipos {
                    start_lnum: 0,
                    end_lnum: 0,
                    start_col: 0,
                    end_col: 0,
                }; 10],
            },
            orig_start_col: 0,
        },
    };
    let mut m: regsubs_T = regsubs_T {
        norm: regsub_T {
            in_use: 0,
            list: C2Rust_Unnamed_19 {
                multi: [multipos {
                    start_lnum: 0,
                    end_lnum: 0,
                    start_col: 0,
                    end_col: 0,
                }; 10],
            },
            orig_start_col: 0,
        },
        synt: regsub_T {
            in_use: 0,
            list: C2Rust_Unnamed_19 {
                multi: [multipos {
                    start_lnum: 0,
                    end_lnum: 0,
                    start_col: 0,
                    end_col: 0,
                }; 10],
            },
            orig_start_col: 0,
        },
    };
    let mut start: *mut nfa_state_T = (*prog).start;
    (*rex.ptr()).input = (*rex.ptr()).line.offset(col as isize);
    nfa_time_limit.set(tm);
    nfa_timed_out.set(timed_out);
    nfa_time_count.set(0 as ::core::ffi::c_int);
    clear_sub(&raw mut subs.norm);
    clear_sub(&raw mut m.norm);
    clear_sub(&raw mut subs.synt);
    clear_sub(&raw mut m.synt);
    let mut result: ::core::ffi::c_int = nfa_regmatch(prog, start, &raw mut subs, &raw mut m);
    if result == 0 {
        return 0 as ::core::ffi::c_int;
    } else if result == NFA_TOO_EXPENSIVE as ::core::ffi::c_int {
        return result;
    }
    cleanup_subexpr();
    if (*rex.ptr()).reg_match.is_null() {
        i = 0 as ::core::ffi::c_int;
        while i < subs.norm.in_use {
            (*(*rex.ptr()).reg_startpos.offset(i as isize)).lnum =
                subs.norm.list.multi[i as usize].start_lnum;
            (*(*rex.ptr()).reg_startpos.offset(i as isize)).col =
                subs.norm.list.multi[i as usize].start_col;
            (*(*rex.ptr()).reg_endpos.offset(i as isize)).lnum =
                subs.norm.list.multi[i as usize].end_lnum;
            (*(*rex.ptr()).reg_endpos.offset(i as isize)).col =
                subs.norm.list.multi[i as usize].end_col;
            i += 1;
        }
        if !(*rex.ptr()).reg_mmatch.is_null() {
            (*(*rex.ptr()).reg_mmatch).rmm_matchcol = subs.norm.orig_start_col;
        }
        if (*(*rex.ptr())
            .reg_startpos
            .offset(0 as ::core::ffi::c_int as isize))
        .lnum
            < 0 as linenr_T
        {
            (*(*rex.ptr())
                .reg_startpos
                .offset(0 as ::core::ffi::c_int as isize))
            .lnum = 0 as ::core::ffi::c_int as linenr_T;
            (*(*rex.ptr())
                .reg_startpos
                .offset(0 as ::core::ffi::c_int as isize))
            .col = col;
        }
        if (*(*rex.ptr())
            .reg_endpos
            .offset(0 as ::core::ffi::c_int as isize))
        .lnum
            < 0 as linenr_T
        {
            (*(*rex.ptr())
                .reg_endpos
                .offset(0 as ::core::ffi::c_int as isize))
            .lnum = (*rex.ptr()).lnum;
            (*(*rex.ptr())
                .reg_endpos
                .offset(0 as ::core::ffi::c_int as isize))
            .col =
                (*rex.ptr()).input.offset_from((*rex.ptr()).line) as ::core::ffi::c_int as colnr_T;
        } else {
            (*rex.ptr()).lnum = (*(*rex.ptr())
                .reg_endpos
                .offset(0 as ::core::ffi::c_int as isize))
            .lnum;
        }
    } else {
        i = 0 as ::core::ffi::c_int;
        while i < subs.norm.in_use {
            *(*rex.ptr()).reg_startp.offset(i as isize) = subs.norm.list.line[i as usize].start;
            *(*rex.ptr()).reg_endp.offset(i as isize) = subs.norm.list.line[i as usize].end;
            i += 1;
        }
        if (*(*rex.ptr())
            .reg_startp
            .offset(0 as ::core::ffi::c_int as isize))
        .is_null()
        {
            *(*rex.ptr())
                .reg_startp
                .offset(0 as ::core::ffi::c_int as isize) = (*rex.ptr()).line.offset(col as isize);
        }
        if (*(*rex.ptr())
            .reg_endp
            .offset(0 as ::core::ffi::c_int as isize))
        .is_null()
        {
            *(*rex.ptr())
                .reg_endp
                .offset(0 as ::core::ffi::c_int as isize) = (*rex.ptr()).input;
        }
    }
    unref_extmatch(re_extmatch_out.get());
    re_extmatch_out.set(::core::ptr::null_mut::<reg_extmatch_T>());
    if (*prog).reghasz == REX_SET {
        cleanup_zsubexpr();
        re_extmatch_out.set(make_extmatch());
        i = 1 as ::core::ffi::c_int;
        while i < subs.synt.in_use {
            if (*rex.ptr()).reg_match.is_null() {
                let mut mpos: *mut multipos = (&raw mut subs.synt.list.multi as *mut multipos)
                    .offset(i as isize)
                    as *mut multipos;
                if (*mpos).start_lnum >= 0 as linenr_T
                    && (*mpos).start_lnum == (*mpos).end_lnum
                    && (*mpos).end_col >= (*mpos).start_col
                {
                    (*re_extmatch_out.get()).matches[i as usize] = xstrnsave(
                        reg_getline((*mpos).start_lnum).offset((*mpos).start_col as isize),
                        ((*mpos).end_col - (*mpos).start_col) as size_t,
                    )
                        as *mut uint8_t;
                }
            } else {
                let mut lpos: *mut linepos = (&raw mut subs.synt.list.line as *mut linepos)
                    .offset(i as isize)
                    as *mut linepos;
                if !(*lpos).start.is_null() && !(*lpos).end.is_null() {
                    (*re_extmatch_out.get()).matches[i as usize] = xstrnsave(
                        (*lpos).start as *mut ::core::ffi::c_char,
                        (*lpos).end.offset_from((*lpos).start) as size_t,
                    )
                        as *mut uint8_t;
                }
            }
            i += 1;
        }
    }
    return 1 as ::core::ffi::c_int + (*rex.ptr()).lnum as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn nfa_regexec_both(
    mut line: *mut uint8_t,
    mut startcol: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut prog: *mut nfa_regprog_T = ::core::ptr::null_mut::<nfa_regprog_T>();
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut col: colnr_T = startcol;
    if (*rex.ptr()).reg_match.is_null() {
        prog = (*(*rex.ptr()).reg_mmatch).regprog as *mut nfa_regprog_T;
        line = reg_getline(0 as linenr_T) as *mut uint8_t;
        (*rex.ptr()).reg_startpos = &raw mut (*(*rex.ptr()).reg_mmatch).startpos as *mut lpos_T;
        (*rex.ptr()).reg_endpos = &raw mut (*(*rex.ptr()).reg_mmatch).endpos as *mut lpos_T;
    } else {
        prog = (*(*rex.ptr()).reg_match).regprog as *mut nfa_regprog_T;
        (*rex.ptr()).reg_startp = &raw mut (*(*rex.ptr()).reg_match).startp
            as *mut *mut ::core::ffi::c_char as *mut *mut uint8_t;
        (*rex.ptr()).reg_endp = &raw mut (*(*rex.ptr()).reg_match).endp
            as *mut *mut ::core::ffi::c_char as *mut *mut uint8_t;
    }
    if prog.is_null() || line.is_null() {
        iemsg(gettext(&raw const e_null as *const ::core::ffi::c_char));
    } else {
        if (*prog).regflags & RF_ICASE as ::core::ffi::c_uint != 0 {
            (*rex.ptr()).reg_ic = true_0 != 0;
        } else if (*prog).regflags & RF_NOICASE as ::core::ffi::c_uint != 0 {
            (*rex.ptr()).reg_ic = false_0 != 0;
        }
        if (*prog).regflags & RF_ICOMBINE as ::core::ffi::c_uint != 0 {
            (*rex.ptr()).reg_icombine = true_0 != 0;
        }
        (*rex.ptr()).line = line;
        (*rex.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*rex.ptr()).nfa_has_zend = (*prog).has_zend;
        (*rex.ptr()).nfa_has_backref = (*prog).has_backref;
        (*rex.ptr()).nfa_nsubexpr = (*prog).nsubexp;
        (*rex.ptr()).nfa_listid = 1 as ::core::ffi::c_int;
        (*rex.ptr()).nfa_alt_listid = 2 as ::core::ffi::c_int;
        if (*prog).reganch != 0 && col > 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        (*rex.ptr()).need_clear_subexpr = true_0;
        if (*prog).reghasz == REX_SET {
            (*rex.ptr()).nfa_has_zsubexpr = true_0;
            (*rex.ptr()).need_clear_zsubexpr = true_0;
        } else {
            (*rex.ptr()).nfa_has_zsubexpr = false_0;
            (*rex.ptr()).need_clear_zsubexpr = false_0;
        }
        if (*prog).regstart != NUL {
            if skip_to_start((*prog).regstart, &raw mut col) == FAIL {
                return 0 as ::core::ffi::c_int;
            }
            if !(*prog).match_text.is_null()
                && *(*prog).match_text as ::core::ffi::c_int != NUL
                && !(*rex.ptr()).reg_icombine
            {
                retval = find_match_text(&raw mut col, (*prog).regstart, (*prog).match_text);
                if (*rex.ptr()).reg_match.is_null() {
                    (*(*rex.ptr()).reg_mmatch).rmm_matchcol = col;
                } else {
                    (*(*rex.ptr()).reg_match).rm_matchcol = col;
                }
                return retval;
            }
        }
        if !((*rex.ptr()).reg_maxcol > 0 as ::core::ffi::c_int && col >= (*rex.ptr()).reg_maxcol) {
            nstate.set(0 as ::core::ffi::c_int);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*prog).nstate {
                (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).id = i;
                (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).lastlist
                    [0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
                (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).lastlist
                    [1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
                i += 1;
            }
            retval = nfa_regtry(prog, col, tm, timed_out);
        }
    }
    if retval > 0 as ::core::ffi::c_int {
        if (*rex.ptr()).reg_match.is_null() {
            let start: *const lpos_T = (&raw mut (*(*rex.ptr()).reg_mmatch).startpos
                as *mut lpos_T)
                .offset(0 as ::core::ffi::c_int as isize);
            let end: *const lpos_T = (&raw mut (*(*rex.ptr()).reg_mmatch).endpos as *mut lpos_T)
                .offset(0 as ::core::ffi::c_int as isize);
            if (*end).lnum < (*start).lnum
                || (*end).lnum == (*start).lnum && (*end).col < (*start).col
            {
                (*(*rex.ptr()).reg_mmatch).endpos[0 as ::core::ffi::c_int as usize] =
                    (*(*rex.ptr()).reg_mmatch).startpos[0 as ::core::ffi::c_int as usize];
            }
        } else {
            if (*(*rex.ptr()).reg_match).endp[0 as ::core::ffi::c_int as usize]
                < (*(*rex.ptr()).reg_match).startp[0 as ::core::ffi::c_int as usize]
            {
                (*(*rex.ptr()).reg_match).endp[0 as ::core::ffi::c_int as usize] =
                    (*(*rex.ptr()).reg_match).startp[0 as ::core::ffi::c_int as usize];
            }
            (*(*rex.ptr()).reg_match).rm_matchcol = col;
        }
    }
    return retval;
}
pub(crate) unsafe extern "C" fn nfa_regcomp(
    mut expr: *mut uint8_t,
    mut re_flags: ::core::ffi::c_int,
) -> *mut regprog_T {
    let mut prog_size: size_t = 0;
    let mut prog: *mut nfa_regprog_T = ::core::ptr::null_mut::<nfa_regprog_T>();
    let mut postfix: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    if expr.is_null() {
        return ::core::ptr::null_mut::<regprog_T>();
    }
    nfa_re_flags.set(re_flags);
    init_class_tab();
    nfa_regcomp_start(expr, re_flags);
    postfix = re2post();
    '_out: {
        if !postfix.is_null() {
            post2nfa(postfix, post_ptr.get(), true_0);
            prog_size = (80 as size_t).wrapping_add(
                ::core::mem::size_of::<nfa_state_T>().wrapping_mul(nstate.get() as size_t),
            );
            prog = xmalloc(prog_size) as *mut nfa_regprog_T;
            state_ptr.set(&raw mut (*prog).state as *mut nfa_state_T);
            (*prog).re_in_use = false_0 != 0;
            (*prog).start = post2nfa(postfix, post_ptr.get(), false_0);
            if !(*prog).start.is_null() {
                (*prog).regflags = regflags.get();
                (*prog).engine = nfa_regengine.ptr();
                (*prog).nstate = nstate.get();
                (*prog).has_zend = (*rex.ptr()).nfa_has_zend;
                (*prog).has_backref = (*rex.ptr()).nfa_has_backref;
                (*prog).nsubexp = regnpar.get();
                nfa_postprocess(prog);
                (*prog).reganch = nfa_get_reganch((*prog).start, 0 as ::core::ffi::c_int);
                (*prog).regstart = nfa_get_regstart((*prog).start, 0 as ::core::ffi::c_int);
                (*prog).match_text = nfa_get_match_text((*prog).start);
                (*prog).reghasz = re_has_z.get();
                (*prog).pattern = xstrdup(expr as *mut ::core::ffi::c_char);
                break '_out;
            }
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut prog as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    xfree(post_start.get() as *mut ::core::ffi::c_void);
    post_end.set(::core::ptr::null_mut::<::core::ffi::c_int>());
    post_ptr.set(post_end.get());
    post_start.set(post_ptr.get());
    state_ptr.set(::core::ptr::null_mut::<nfa_state_T>());
    return prog as *mut regprog_T;
}
pub(crate) unsafe extern "C" fn nfa_regfree(mut prog: *mut regprog_T) {
    if prog.is_null() {
        return;
    }
    xfree((*(prog as *mut nfa_regprog_T)).match_text as *mut ::core::ffi::c_void);
    xfree((*(prog as *mut nfa_regprog_T)).pattern as *mut ::core::ffi::c_void);
    xfree(prog as *mut ::core::ffi::c_void);
}
pub(crate) unsafe extern "C" fn nfa_regexec_nl(
    mut rmp: *mut regmatch_T,
    mut line: *mut uint8_t,
    mut col: colnr_T,
    mut line_lbr: bool,
) -> ::core::ffi::c_int {
    (*rex.ptr()).reg_match = rmp;
    (*rex.ptr()).reg_mmatch = ::core::ptr::null_mut::<regmmatch_T>();
    (*rex.ptr()).reg_maxline = 0 as ::core::ffi::c_int as linenr_T;
    (*rex.ptr()).reg_line_lbr = line_lbr;
    (*rex.ptr()).reg_buf = curbuf.get();
    (*rex.ptr()).reg_win = ::core::ptr::null_mut::<win_T>();
    (*rex.ptr()).reg_ic = (*rmp).rm_ic;
    (*rex.ptr()).reg_icombine = false_0 != 0;
    (*rex.ptr()).reg_nobreak = (*(*rmp).regprog).re_flags & RE_NOBREAK as ::core::ffi::c_uint != 0;
    (*rex.ptr()).reg_maxcol = 0 as ::core::ffi::c_int as colnr_T;
    return nfa_regexec_both(
        line,
        col,
        ::core::ptr::null_mut::<proftime_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
}
pub(crate) unsafe extern "C" fn nfa_regexec_multi(
    mut rmp: *mut regmmatch_T,
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    init_regexec_multi(rmp, win, buf, lnum);
    return nfa_regexec_both(::core::ptr::null_mut::<uint8_t>(), col, tm, timed_out);
}
