//! Running a compiled program: try a start position, then the entry
//! points the engine table names.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn regtry(
    mut prog: *mut bt_regprog_T,
    mut col: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (*rex.ptr()).input = (*rex.ptr()).line.offset(col as isize);
    (*rex.ptr()).need_clear_subexpr = true_0;
    (*rex.ptr()).need_clear_zsubexpr =
        ((*prog).reghasz as ::core::ffi::c_int == REX_SET) as ::core::ffi::c_int;
    if regmatch(
        (&raw mut (*prog).program as *mut uint8_t).offset(1 as ::core::ffi::c_int as isize),
        tm,
        timed_out,
    ) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    cleanup_subexpr();
    if (*rex.ptr()).reg_match.is_null() {
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
    if (*prog).reghasz as ::core::ffi::c_int == REX_SET {
        let mut i: ::core::ffi::c_int = 0;
        cleanup_zsubexpr();
        re_extmatch_out.set(make_extmatch());
        i = 0 as ::core::ffi::c_int;
        while i < NSUBEXP as ::core::ffi::c_int {
            if (*rex.ptr()).reg_match.is_null() {
                if (*reg_startzpos.ptr())[i as usize].lnum >= 0 as linenr_T
                    && (*reg_endzpos.ptr())[i as usize].lnum
                        == (*reg_startzpos.ptr())[i as usize].lnum
                    && (*reg_endzpos.ptr())[i as usize].col
                        >= (*reg_startzpos.ptr())[i as usize].col
                {
                    (*re_extmatch_out.get()).matches[i as usize] = xstrnsave(
                        reg_getline((*reg_startzpos.ptr())[i as usize].lnum)
                            .offset((*reg_startzpos.ptr())[i as usize].col as isize),
                        ((*reg_endzpos.ptr())[i as usize].col
                            - (*reg_startzpos.ptr())[i as usize].col)
                            as size_t,
                    )
                        as *mut uint8_t;
                }
            } else if !(*reg_startzp.ptr())[i as usize].is_null()
                && !(*reg_endzp.ptr())[i as usize].is_null()
            {
                (*re_extmatch_out.get()).matches[i as usize] = xstrnsave(
                    (*reg_startzp.ptr())[i as usize] as *mut ::core::ffi::c_char,
                    (*reg_endzp.ptr())[i as usize].offset_from((*reg_startzp.ptr())[i as usize])
                        as size_t,
                ) as *mut uint8_t;
            }
            i += 1;
        }
    }
    return 1 as ::core::ffi::c_int + (*rex.ptr()).lnum as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn bt_regexec_both(
    mut line: *mut uint8_t,
    mut startcol: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut prog: *mut bt_regprog_T = ::core::ptr::null_mut::<bt_regprog_T>();
    let mut s: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut col: colnr_T = startcol;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*regstack.ptr()).ga_data.is_null() {
        ga_init(regstack.ptr(), 1 as ::core::ffi::c_int, REGSTACK_INITIAL);
        ga_grow(regstack.ptr(), REGSTACK_INITIAL);
        ga_set_growsize(regstack.ptr(), REGSTACK_INITIAL * 8 as ::core::ffi::c_int);
    }
    if (*backpos.ptr()).ga_data.is_null() {
        ga_init(
            backpos.ptr(),
            ::core::mem::size_of::<backpos_T>() as ::core::ffi::c_int,
            BACKPOS_INITIAL,
        );
        ga_grow(backpos.ptr(), BACKPOS_INITIAL);
        ga_set_growsize(backpos.ptr(), BACKPOS_INITIAL * 8 as ::core::ffi::c_int);
    }
    if (*rex.ptr()).reg_match.is_null() {
        prog = (*(*rex.ptr()).reg_mmatch).regprog as *mut bt_regprog_T;
        line = reg_getline(0 as linenr_T) as *mut uint8_t;
        (*rex.ptr()).reg_startpos = &raw mut (*(*rex.ptr()).reg_mmatch).startpos as *mut lpos_T;
        (*rex.ptr()).reg_endpos = &raw mut (*(*rex.ptr()).reg_mmatch).endpos as *mut lpos_T;
    } else {
        prog = (*(*rex.ptr()).reg_match).regprog as *mut bt_regprog_T;
        (*rex.ptr()).reg_startp = &raw mut (*(*rex.ptr()).reg_match).startp
            as *mut *mut ::core::ffi::c_char as *mut *mut uint8_t;
        (*rex.ptr()).reg_endp = &raw mut (*(*rex.ptr()).reg_match).endp
            as *mut *mut ::core::ffi::c_char as *mut *mut uint8_t;
    }
    '_theend: {
        if prog.is_null() || line.is_null() {
            iemsg(gettext(&raw const e_null as *const ::core::ffi::c_char));
        } else if prog_magic_wrong() == 0 {
            if !((*rex.ptr()).reg_maxcol > 0 as ::core::ffi::c_int
                && col >= (*rex.ptr()).reg_maxcol)
            {
                if (*prog).regflags & RF_ICASE as ::core::ffi::c_uint != 0 {
                    (*rex.ptr()).reg_ic = true_0 != 0;
                } else if (*prog).regflags & RF_NOICASE as ::core::ffi::c_uint != 0 {
                    (*rex.ptr()).reg_ic = false_0 != 0;
                }
                if (*prog).regflags & RF_ICOMBINE as ::core::ffi::c_uint != 0 {
                    (*rex.ptr()).reg_icombine = true_0 != 0;
                }
                if !(*prog).regmust.is_null() {
                    let mut c: ::core::ffi::c_int =
                        utf_ptr2char((*prog).regmust as *mut ::core::ffi::c_char);
                    s = line.offset(col as isize);
                    if !(*rex.ptr()).reg_ic {
                        loop {
                            s = vim_strchr(s as *mut ::core::ffi::c_char, c) as *mut uint8_t;
                            if s.is_null() {
                                break;
                            }
                            if cstrncmp(
                                s as *mut ::core::ffi::c_char,
                                (*prog).regmust as *mut ::core::ffi::c_char,
                                &raw mut (*prog).regmlen,
                            ) == 0 as ::core::ffi::c_int
                            {
                                break;
                            }
                            s = s.offset(utfc_ptr2len(s as *mut ::core::ffi::c_char) as isize);
                        }
                    } else {
                        loop {
                            s = cstrchr(s as *mut ::core::ffi::c_char, c) as *mut uint8_t;
                            if s.is_null() {
                                break;
                            }
                            if cstrncmp(
                                s as *mut ::core::ffi::c_char,
                                (*prog).regmust as *mut ::core::ffi::c_char,
                                &raw mut (*prog).regmlen,
                            ) == 0 as ::core::ffi::c_int
                            {
                                break;
                            }
                            s = s.offset(utfc_ptr2len(s as *mut ::core::ffi::c_char) as isize);
                        }
                    }
                    if s.is_null() {
                        break '_theend;
                    }
                }
                (*rex.ptr()).line = line;
                (*rex.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
                reg_toolong.set(false_0);
                if (*prog).reganch != 0 {
                    let mut c_0: ::core::ffi::c_int = utf_ptr2char(
                        ((*rex.ptr()).line as *mut ::core::ffi::c_char).offset(col as isize),
                    );
                    if (*prog).regstart == NUL
                        || (*prog).regstart == c_0
                        || (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                            && (utf_fold((*prog).regstart) == utf_fold(c_0)
                                || c_0 < 255 as ::core::ffi::c_int
                                    && (*prog).regstart < 255 as ::core::ffi::c_int
                                    && mb_tolower((*prog).regstart) == mb_tolower(c_0))
                    {
                        retval = regtry(prog, col, tm, timed_out);
                    } else {
                        retval = 0 as ::core::ffi::c_int;
                    }
                } else {
                    let mut tm_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while !got_int.get() {
                        if (*prog).regstart != NUL {
                            s = cstrchr(
                                ((*rex.ptr()).line as *mut ::core::ffi::c_char)
                                    .offset(col as isize),
                                (*prog).regstart,
                            ) as *mut uint8_t;
                            if s.is_null() {
                                retval = 0 as ::core::ffi::c_int;
                                break;
                            } else {
                                col = s.offset_from((*rex.ptr()).line) as ::core::ffi::c_int
                                    as colnr_T;
                            }
                        }
                        if (*rex.ptr()).reg_maxcol > 0 as ::core::ffi::c_int
                            && col >= (*rex.ptr()).reg_maxcol
                        {
                            retval = 0 as ::core::ffi::c_int;
                            break;
                        } else {
                            retval = regtry(prog, col, tm, timed_out);
                            if retval > 0 as ::core::ffi::c_int {
                                break;
                            }
                            if (*rex.ptr()).lnum != 0 as linenr_T {
                                (*rex.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
                                (*rex.ptr()).line = reg_getline(0 as linenr_T) as *mut uint8_t;
                            }
                            if *(*rex.ptr()).line.offset(col as isize) as ::core::ffi::c_int == NUL
                            {
                                break;
                            }
                            col += utfc_ptr2len(
                                ((*rex.ptr()).line as *mut ::core::ffi::c_char)
                                    .offset(col as isize),
                            );
                            if !(!tm.is_null() && {
                                tm_count += 1;
                                tm_count == 20 as ::core::ffi::c_int
                            }) {
                                continue;
                            }
                            tm_count = 0 as ::core::ffi::c_int;
                            if !profile_passed_limit(*tm) {
                                continue;
                            }
                            if !timed_out.is_null() {
                                *timed_out = true_0;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    if reg_tofreelen.get() > 400 as ::core::ffi::c_uint {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            reg_tofree.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    if (*regstack.ptr()).ga_maxlen > REGSTACK_INITIAL {
        ga_clear(regstack.ptr());
    }
    if (*backpos.ptr()).ga_maxlen > BACKPOS_INITIAL {
        ga_clear(backpos.ptr());
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
            (*(*rex.ptr()).reg_mmatch).rmm_matchcol = col;
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
pub(crate) unsafe extern "C" fn bt_regexec_nl(
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
    let mut r: int64_t = bt_regexec_both(
        line,
        col,
        ::core::ptr::null_mut::<proftime_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) as int64_t;
    assert!(r <= 2147483647 as int64_t, "r <= INT_MAX");
    return r as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn bt_regexec_multi(
    mut rmp: *mut regmmatch_T,
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    init_regexec_multi(rmp, win, buf, lnum);
    return bt_regexec_both(::core::ptr::null_mut::<uint8_t>(), col, tm, timed_out);
}
pub(crate) unsafe extern "C" fn re_num_cmp(
    mut val: uint32_t,
    mut scan: *const uint8_t,
) -> ::core::ffi::c_int {
    let mut n: uint32_t = (((*scan.offset(3 as ::core::ffi::c_int as isize) as int64_t)
        << 24 as ::core::ffi::c_int)
        + ((*scan.offset(4 as ::core::ffi::c_int as isize) as int64_t) << 16 as ::core::ffi::c_int)
        + ((*scan.offset(5 as ::core::ffi::c_int as isize) as int64_t) << 8 as ::core::ffi::c_int)
        + *scan.offset(6 as ::core::ffi::c_int as isize) as int64_t)
        as uint32_t;
    if *scan.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '>' as ::core::ffi::c_int
    {
        return (val > n) as ::core::ffi::c_int;
    }
    if *scan.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '<' as ::core::ffi::c_int
    {
        return (val < n) as ::core::ffi::c_int;
    }
    return (val == n) as ::core::ffi::c_int;
}
