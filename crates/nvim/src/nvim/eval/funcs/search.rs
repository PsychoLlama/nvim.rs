//! Searching the buffer: `search()`, `searchpair()` and their shared
//! machinery.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub const SP_NOMOVE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SP_REPEAT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SP_RETCOUNT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SP_SETPCMARK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SP_START: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const SP_SUBPAT: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SP_END: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SP_COLUMN: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
unsafe extern "C" fn get_search_arg(
    mut varp: *mut typval_T,
    mut flagsp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = FORWARD as ::core::ffi::c_int;
    if (*varp).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FORWARD as ::core::ffi::c_int;
    }
    let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flags: *const ::core::ffi::c_char =
        tv_get_string_buf_chk(varp, &raw mut nbuf as *mut ::core::ffi::c_char);
    if flags.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut mask: ::core::ffi::c_int = 0;
    while *flags as ::core::ffi::c_int != NUL {
        match *flags as ::core::ffi::c_int {
            98 => {
                dir = BACKWARD as ::core::ffi::c_int;
            }
            119 => {
                p_ws.set(true_0);
            }
            87 => {
                p_ws.set(false_0);
            }
            _ => {
                mask = 0 as ::core::ffi::c_int;
                if !flagsp.is_null() {
                    match *flags as ::core::ffi::c_int {
                        99 => {
                            mask = SP_START;
                        }
                        101 => {
                            mask = SP_END;
                        }
                        109 => {
                            mask = SP_RETCOUNT;
                        }
                        110 => {
                            mask = SP_NOMOVE;
                        }
                        112 => {
                            mask = SP_SUBPAT;
                        }
                        114 => {
                            mask = SP_REPEAT;
                        }
                        115 => {
                            mask = SP_SETPCMARK;
                        }
                        122 => {
                            mask = SP_COLUMN;
                        }
                        _ => {}
                    }
                }
                if mask == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        flags,
                    );
                    dir = 0 as ::core::ffi::c_int;
                } else {
                    *flagsp |= mask;
                }
            }
        }
        if dir == 0 as ::core::ffi::c_int {
            break;
        }
        flags = flags.offset(1);
    }
    return dir;
}
unsafe extern "C" fn search_cmn(
    mut argvars: *mut typval_T,
    mut match_pos: *mut pos_T,
    mut flagsp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0;
    let mut tm: proftime_T = 0;
    let mut save_cursor: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut firstpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut sia: searchit_arg_T = searchit_arg_T {
        sa_stop_lnum: 0,
        sa_tm: ::core::ptr::null_mut::<proftime_T>(),
        sa_timed_out: 0,
        sa_wrapped: 0,
    };
    let mut patlen: size_t = 0;
    let mut subpatnum: ::core::ffi::c_int = 0;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lnum_stop: linenr_T = 0 as linenr_T;
    let mut time_limit: int64_t = 0 as int64_t;
    let mut options: ::core::ffi::c_int = SEARCH_KEEP as ::core::ffi::c_int;
    let mut use_skip: bool = false_0 != 0;
    let pat: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut dir: ::core::ffi::c_int =
        get_search_arg(argvars.offset(1 as ::core::ffi::c_int as isize), flagsp);
    '_theend: {
        if dir != 0 as ::core::ffi::c_int {
            flags = *flagsp;
            if flags & SP_START != 0 {
                options |= SEARCH_START as ::core::ffi::c_int;
            }
            if flags & SP_END != 0 {
                options |= SEARCH_END as ::core::ffi::c_int;
            }
            if flags & SP_COLUMN != 0 {
                options |= SEARCH_COL as ::core::ffi::c_int;
            }
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lnum_stop = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<bool>(),
                ) as linenr_T;
                if lnum_stop < 0 as linenr_T {
                    break '_theend;
                } else if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                    as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    time_limit = tv_get_number_chk(
                        argvars.offset(3 as ::core::ffi::c_int as isize),
                        ::core::ptr::null_mut::<bool>(),
                    ) as int64_t;
                    if time_limit < 0 as int64_t {
                        break '_theend;
                    } else {
                        use_skip =
                            eval_expr_valid_arg(argvars.offset(4 as ::core::ffi::c_int as isize));
                    }
                }
            }
            tm = profile_setlimit(time_limit);
            if flags & (SP_REPEAT | SP_RETCOUNT) != 0 as ::core::ffi::c_int
                || flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0
            {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                );
            } else {
                save_cursor = pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                };
                save_cursor = (*curwin.get()).w_cursor;
                pos = save_cursor;
                firstpos = pos_T {
                    lnum: 0 as linenr_T,
                    col: 0,
                    coladd: 0,
                };
                sia = searchit_arg_T {
                    sa_stop_lnum: lnum_stop,
                    sa_tm: &raw mut tm,
                    sa_timed_out: 0,
                    sa_wrapped: 0,
                };
                patlen = strlen(pat);
                subpatnum = 0;
                loop {
                    subpatnum = searchit(
                        curwin.get(),
                        curbuf.get(),
                        &raw mut pos,
                        ::core::ptr::null_mut::<pos_T>(),
                        dir as Direction,
                        pat as *mut ::core::ffi::c_char,
                        patlen,
                        1 as ::core::ffi::c_int,
                        options,
                        RE_SEARCH as ::core::ffi::c_int,
                        &raw mut sia,
                    );
                    if firstpos.lnum != 0 as linenr_T
                        && equalpos(pos, firstpos) as ::core::ffi::c_int != 0
                    {
                        subpatnum = FAIL;
                    }
                    if subpatnum == FAIL || !use_skip {
                        break;
                    }
                    if firstpos.lnum == 0 as linenr_T {
                        firstpos = pos;
                    }
                    let save_pos: pos_T = (*curwin.get()).w_cursor;
                    (*curwin.get()).w_cursor = pos;
                    let mut err: bool = false_0 != 0;
                    let do_skip: bool = eval_expr_to_bool(
                        argvars.offset(4 as ::core::ffi::c_int as isize),
                        &raw mut err,
                    );
                    (*curwin.get()).w_cursor = save_pos;
                    if err {
                        subpatnum = FAIL;
                        break;
                    } else {
                        if !do_skip {
                            break;
                        }
                        options &= !(SEARCH_START as ::core::ffi::c_int);
                    }
                }
                if subpatnum != FAIL {
                    if flags & SP_SUBPAT != 0 {
                        retval = subpatnum;
                    } else {
                        retval = pos.lnum as ::core::ffi::c_int;
                    }
                    if flags & SP_SETPCMARK != 0 {
                        setpcmark();
                    }
                    (*curwin.get()).w_cursor = pos;
                    if !match_pos.is_null() {
                        (*match_pos).lnum = pos.lnum;
                        (*match_pos).col =
                            (pos.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
                    }
                    check_cursor(curwin.get());
                }
                if flags & SP_NOMOVE != 0 {
                    (*curwin.get()).w_cursor = save_cursor;
                } else {
                    (*curwin.get()).w_set_curswant = true_0;
                }
            }
        }
    }
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    return retval;
}
pub unsafe extern "C" fn f_search(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*rettv).vval.v_number =
        search_cmn(argvars, ::core::ptr::null_mut::<pos_T>(), &raw mut flags) as varnumber_T;
}
pub unsafe extern "C" fn f_searchdecl(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut locally: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut thisblock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let name: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        locally = (tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) == 0 as varnumber_T) as ::core::ffi::c_int;
        if !error
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            thisblock = (tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0 as varnumber_T) as ::core::ffi::c_int;
        }
    }
    if !error && !name.is_null() {
        (*rettv).vval.v_number = (find_decl(
            name as *mut ::core::ffi::c_char,
            strlen(name),
            locally != 0,
            thisblock != 0,
            SEARCH_KEEP as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            == FAIL) as ::core::ffi::c_int as varnumber_T;
    }
}
unsafe extern "C" fn searchpair_cmn(
    mut argvars: *mut typval_T,
    mut match_pos: *mut pos_T,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = 0;
    let mut skip: *const typval_T = ::core::ptr::null::<typval_T>();
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lnum_stop: linenr_T = 0 as linenr_T;
    let mut time_limit: int64_t = 0 as int64_t;
    let mut nbuf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut nbuf2: [::core::ffi::c_char; 65] = [0; 65];
    let mut spat: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut mpat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut nbuf1 as *mut ::core::ffi::c_char,
    );
    let mut epat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut nbuf2 as *mut ::core::ffi::c_char,
    );
    '_theend: {
        if !(spat.is_null() || mpat.is_null() || epat.is_null()) {
            dir = get_search_arg(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut flags,
            );
            if dir != 0 as ::core::ffi::c_int {
                if flags & (SP_END | SP_SUBPAT) != 0 as ::core::ffi::c_int
                    || flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        tv_get_string(argvars.offset(3 as ::core::ffi::c_int as isize)),
                    );
                } else {
                    if flags & SP_REPEAT != 0 {
                        p_ws.set(false_0);
                    }
                    skip = ::core::ptr::null::<typval_T>();
                    if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        skip = ::core::ptr::null::<typval_T>();
                    } else {
                        skip = argvars.offset(4 as ::core::ffi::c_int as isize);
                        if (*argvars.offset(5 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            lnum_stop = tv_get_number_chk(
                                argvars.offset(5 as ::core::ffi::c_int as isize),
                                ::core::ptr::null_mut::<bool>(),
                            ) as linenr_T;
                            if lnum_stop < 0 as linenr_T {
                                semsg(
                                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                    tv_get_string(argvars.offset(5 as ::core::ffi::c_int as isize)),
                                );
                                break '_theend;
                            } else if (*argvars.offset(6 as ::core::ffi::c_int as isize)).v_type
                                as ::core::ffi::c_uint
                                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                time_limit = tv_get_number_chk(
                                    argvars.offset(6 as ::core::ffi::c_int as isize),
                                    ::core::ptr::null_mut::<bool>(),
                                ) as int64_t;
                                if time_limit < 0 as int64_t {
                                    semsg(
                                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                        tv_get_string(
                                            argvars.offset(6 as ::core::ffi::c_int as isize),
                                        ),
                                    );
                                    break '_theend;
                                }
                            }
                        }
                    }
                    retval = do_searchpair(
                        spat, mpat, epat, dir, skip, flags, match_pos, lnum_stop, time_limit,
                    );
                }
            }
        }
    }
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    return retval;
}
pub unsafe extern "C" fn f_searchpair(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number =
        searchpair_cmn(argvars, ::core::ptr::null_mut::<pos_T>()) as varnumber_T;
}
pub unsafe extern "C" fn f_searchpairpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut match_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    if searchpair_cmn(argvars, &raw mut match_pos) > 0 as ::core::ffi::c_int {
        lnum = match_pos.lnum as ::core::ffi::c_int;
        col = match_pos.col as ::core::ffi::c_int;
    }
    tv_list_append_number((*rettv).vval.v_list, lnum as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, col as varnumber_T);
}
pub unsafe extern "C" fn do_searchpair(
    mut spat: *const ::core::ffi::c_char,
    mut mpat: *const ::core::ffi::c_char,
    mut epat: *const ::core::ffi::c_char,
    mut dir: ::core::ffi::c_int,
    mut skip: *const typval_T,
    mut flags: ::core::ffi::c_int,
    mut match_pos: *mut pos_T,
    mut lnum_stop: linenr_T,
    mut time_limit: int64_t,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut nest: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut use_skip: bool = false_0 != 0;
    let mut options: ::core::ffi::c_int = SEARCH_KEEP as ::core::ffi::c_int;
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut tm: proftime_T = profile_setlimit(time_limit);
    let spatlen: size_t = strlen(spat);
    let epatlen: size_t = strlen(epat);
    let pat2size: size_t = spatlen.wrapping_add(epatlen).wrapping_add(17 as size_t);
    let mut pat2: *mut ::core::ffi::c_char = xmalloc(pat2size) as *mut ::core::ffi::c_char;
    let pat3size: size_t = spatlen
        .wrapping_add(strlen(mpat))
        .wrapping_add(epatlen)
        .wrapping_add(25 as size_t);
    let mut pat3: *mut ::core::ffi::c_char = xmalloc(pat3size) as *mut ::core::ffi::c_char;
    let mut pat2len: ::core::ffi::c_int = snprintf(
        pat2,
        pat2size,
        b"\\m\\(%s\\m\\)\\|\\(%s\\m\\)\0".as_ptr() as *const ::core::ffi::c_char,
        spat,
        epat,
    );
    let mut pat3len: ::core::ffi::c_int = 0;
    if *mpat as ::core::ffi::c_int == NUL {
        strcpy(pat3, pat2);
        pat3len = pat2len;
    } else {
        pat3len = snprintf(
            pat3,
            pat3size,
            b"\\m\\(%s\\m\\)\\|\\(%s\\m\\)\\|\\(%s\\m\\)\0".as_ptr() as *const ::core::ffi::c_char,
            spat,
            epat,
            mpat,
        );
    }
    if flags & SP_START != 0 {
        options |= SEARCH_START as ::core::ffi::c_int;
    }
    if !skip.is_null() {
        use_skip = eval_expr_valid_arg(skip);
    }
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut firstpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    clearpos(&mut firstpos);
    let mut foundpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    clearpos(&mut foundpos);
    let mut pat: *mut ::core::ffi::c_char = pat3;
    '_c2rust_label: {
        if pat3len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"pat3len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/funcs.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                6178 as ::core::ffi::c_uint,
                b"int do_searchpair(const char *, const char *, const char *, int, const typval_T *, int, pos_T *, linenr_T, int64_t)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut patlen: size_t = pat3len as size_t;
    loop {
        let mut sia: searchit_arg_T = searchit_arg_T {
            sa_stop_lnum: lnum_stop,
            sa_tm: &raw mut tm,
            sa_timed_out: 0,
            sa_wrapped: 0,
        };
        let mut n: ::core::ffi::c_int = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut pos,
            ::core::ptr::null_mut::<pos_T>(),
            dir as Direction,
            pat,
            patlen,
            1 as ::core::ffi::c_int,
            options,
            RE_SEARCH as ::core::ffi::c_int,
            &raw mut sia,
        );
        if n == FAIL
            || firstpos.lnum != 0 as linenr_T && equalpos(pos, firstpos) as ::core::ffi::c_int != 0
        {
            break;
        }
        if firstpos.lnum == 0 as linenr_T {
            firstpos = pos;
        }
        if equalpos(pos, foundpos) {
            if dir == BACKWARD as ::core::ffi::c_int {
                decl(&raw mut pos);
            } else {
                incl(&raw mut pos);
            }
        }
        foundpos = pos;
        options &= !(SEARCH_START as ::core::ffi::c_int);
        if use_skip {
            let mut save_pos: pos_T = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = pos;
            let mut err: bool = false_0 != 0;
            let r: bool = eval_expr_to_bool(skip, &raw mut err);
            (*curwin.get()).w_cursor = save_pos;
            if err {
                (*curwin.get()).w_cursor = save_cursor;
                retval = -1 as ::core::ffi::c_int;
                break;
            } else if r {
                continue;
            }
        }
        if dir == BACKWARD as ::core::ffi::c_int && n == 3 as ::core::ffi::c_int
            || dir == FORWARD as ::core::ffi::c_int && n == 2 as ::core::ffi::c_int
        {
            nest += 1;
            pat = pat2;
        } else {
            nest -= 1;
            if nest == 1 as ::core::ffi::c_int {
                pat = pat3;
            }
        }
        if nest != 0 as ::core::ffi::c_int {
            continue;
        }
        if flags & SP_RETCOUNT != 0 {
            retval += 1;
        } else {
            retval = pos.lnum as ::core::ffi::c_int;
        }
        if flags & SP_SETPCMARK != 0 {
            setpcmark();
        }
        (*curwin.get()).w_cursor = pos;
        if flags & SP_REPEAT == 0 {
            break;
        }
        nest = 1 as ::core::ffi::c_int;
    }
    if !match_pos.is_null() {
        (*match_pos).lnum = (*curwin.get()).w_cursor.lnum;
        (*match_pos).col = ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as colnr_T;
    }
    if flags & SP_NOMOVE != 0 || retval == 0 as ::core::ffi::c_int {
        (*curwin.get()).w_cursor = save_cursor;
    }
    xfree(pat2 as *mut ::core::ffi::c_void);
    xfree(pat3 as *mut ::core::ffi::c_void);
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
    return retval;
}
pub unsafe extern "C" fn f_searchpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut match_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let n: ::core::ffi::c_int = search_cmn(argvars, &raw mut match_pos, &raw mut flags);
    tv_list_alloc_ret(
        rettv,
        (2 as ::core::ffi::c_int + (flags & SP_SUBPAT != 0) as ::core::ffi::c_int) as ptrdiff_t,
    );
    let lnum: ::core::ffi::c_int = if n > 0 as ::core::ffi::c_int {
        match_pos.lnum as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let col: ::core::ffi::c_int = if n > 0 as ::core::ffi::c_int {
        match_pos.col as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    tv_list_append_number((*rettv).vval.v_list, lnum as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, col as varnumber_T);
    if flags & SP_SUBPAT != 0 {
        tv_list_append_number((*rettv).vval.v_list, n as varnumber_T);
    }
}
