//! Matching a pattern against a string: the `match*()` family.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

unsafe extern "C" fn find_some_match(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    type_0: SomeMatchType,
) {
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut pat: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: int64_t = 0 as int64_t;
    let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut start: int64_t = 0 as int64_t;
    let mut nth: int64_t = 1 as int64_t;
    let mut startcol: colnr_T = 0 as colnr_T;
    let mut match_0: bool = false_0 != 0;
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    (*rettv).vval.v_number = -1 as varnumber_T;
    match type_0 as ::core::ffi::c_uint {
        2 => {
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        }
        4 => {
            tv_list_alloc_ret(rettv, 4 as ptrdiff_t);
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ssize_t,
            );
            tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
            tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
            tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
        }
        3 => {
            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        0 | 1 | _ => {}
    }
    let mut li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    '_theend: {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            l = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list;
            if l.is_null() {
                break '_theend;
            } else {
                li = tv_list_first(l);
            }
        } else {
            str = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char;
            expr = str;
            len = strlen(str) as int64_t;
        }
        patbuf = [0; 65];
        pat = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut patbuf as *mut ::core::ffi::c_char,
        );
        if !pat.is_null() {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut error: bool = false_0 != 0;
                start = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as int64_t;
                if error {
                    break '_theend;
                } else {
                    if !l.is_null() {
                        idx = tv_list_uidx(l, start as ::core::ffi::c_int);
                        if idx == -1 as ::core::ffi::c_int {
                            break '_theend;
                        } else {
                            li = tv_list_find(l, idx);
                        }
                    } else {
                        if start < 0 as int64_t {
                            start = 0 as int64_t;
                        }
                        if start > len {
                            break '_theend;
                        } else if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            startcol = start as colnr_T;
                        } else {
                            str = str.offset(start as isize);
                            len -= start;
                        }
                    }
                    if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        nth = tv_get_number_chk(
                            argvars.offset(3 as ::core::ffi::c_int as isize),
                            &raw mut error,
                        ) as int64_t;
                    }
                    if error {
                        break '_theend;
                    }
                }
            }
            regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            if !regmatch.regprog.is_null() {
                regmatch.rm_ic = p_ic.get() != 0;
                loop {
                    if !l.is_null() {
                        if li.is_null() {
                            match_0 = false_0 != 0;
                            break;
                        } else {
                            xfree(tofree as *mut ::core::ffi::c_void);
                            str = encode_tv2echo(
                                &raw mut (*li).li_tv,
                                ::core::ptr::null_mut::<size_t>(),
                            );
                            expr = str;
                            tofree = expr;
                            if str.is_null() {
                                break;
                            }
                        }
                    }
                    match_0 = vim_regexec_nl(&raw mut regmatch, str, startcol);
                    if match_0 as ::core::ffi::c_int != 0 && {
                        nth -= 1;
                        nth <= 0 as int64_t
                    } {
                        break;
                    }
                    if l.is_null() && !match_0 {
                        break;
                    }
                    if !l.is_null() {
                        li = (*li).li_next;
                        idx += 1;
                    } else {
                        startcol = regmatch.startp[0 as ::core::ffi::c_int as usize]
                            .offset(
                                utfc_ptr2len(regmatch.startp[0 as ::core::ffi::c_int as usize])
                                    as isize,
                            )
                            .offset_from(str) as colnr_T;
                        if !(startcol > len as colnr_T
                            || str.offset(startcol as isize)
                                <= regmatch.startp[0 as ::core::ffi::c_int as usize])
                        {
                            continue;
                        }
                        match_0 = false_0 != 0;
                        break;
                    }
                }
                if match_0 {
                    match type_0 as ::core::ffi::c_uint {
                        4 => {
                            let ret_l: *mut list_T = (*rettv).vval.v_list;
                            let mut li1: *mut listitem_T = tv_list_first(ret_l);
                            let mut li2: *mut listitem_T = (*li1).li_next;
                            let mut li3: *mut listitem_T = (*li2).li_next;
                            let mut li4: *mut listitem_T = (*li3).li_next;
                            xfree((*li1).li_tv.vval.v_string as *mut ::core::ffi::c_void);
                            let rd: size_t = regmatch.endp[0 as ::core::ffi::c_int as usize]
                                .offset_from(regmatch.startp[0 as ::core::ffi::c_int as usize])
                                as size_t;
                            (*li1).li_tv.vval.v_string = xmemdupz(
                                regmatch.startp[0 as ::core::ffi::c_int as usize]
                                    as *const ::core::ffi::c_void,
                                rd,
                            )
                                as *mut ::core::ffi::c_char;
                            (*li3).li_tv.vval.v_number =
                                regmatch.startp[0 as ::core::ffi::c_int as usize].offset_from(expr)
                                    as varnumber_T;
                            (*li4).li_tv.vval.v_number =
                                regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(expr)
                                    as varnumber_T;
                            if !l.is_null() {
                                (*li2).li_tv.vval.v_number = idx as varnumber_T;
                            }
                        }
                        2 => {
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < NSUBEXP as ::core::ffi::c_int {
                                if regmatch.endp[i as usize].is_null() {
                                    tv_list_append_string(
                                        (*rettv).vval.v_list,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        0 as ssize_t,
                                    );
                                } else {
                                    tv_list_append_string(
                                        (*rettv).vval.v_list,
                                        regmatch.startp[i as usize],
                                        regmatch.endp[i as usize]
                                            .offset_from(regmatch.startp[i as usize])
                                            as ssize_t,
                                    );
                                }
                                i += 1;
                            }
                        }
                        3 => {
                            if !l.is_null() {
                                tv_copy(&raw mut (*li).li_tv, rettv);
                            } else {
                                (*rettv).vval.v_string = xmemdupz(
                                    regmatch.startp[0 as ::core::ffi::c_int as usize]
                                        as *const ::core::ffi::c_void,
                                    regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(
                                        regmatch.startp[0 as ::core::ffi::c_int as usize],
                                    ) as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                        }
                        0 | 1 => {
                            if !l.is_null() {
                                (*rettv).vval.v_number = idx as varnumber_T;
                            } else {
                                if type_0 as ::core::ffi::c_uint
                                    == kSomeMatch as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    (*rettv).vval.v_number = regmatch.startp
                                        [0 as ::core::ffi::c_int as usize]
                                        .offset_from(str)
                                        as varnumber_T;
                                } else {
                                    (*rettv).vval.v_number = regmatch.endp
                                        [0 as ::core::ffi::c_int as usize]
                                        .offset_from(str)
                                        as varnumber_T;
                                }
                                (*rettv).vval.v_number += str.offset_from(expr) as varnumber_T;
                            }
                        }
                        _ => {}
                    }
                }
                vim_regfree(regmatch.regprog);
            }
        }
    }
    if type_0 as ::core::ffi::c_uint
        == kSomeMatchStrPos as ::core::ffi::c_int as ::core::ffi::c_uint
        && l.is_null()
        && !(*rettv).vval.v_list.is_null()
    {
        let ret_l_0: *mut list_T = (*rettv).vval.v_list;
        tv_list_item_remove(ret_l_0, (*tv_list_first(ret_l_0)).li_next);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    p_cpo.set(save_cpo);
}
unsafe extern "C" fn get_matches_in_str(
    mut str: *const ::core::ffi::c_char,
    mut rmp: *mut regmatch_T,
    mut mlist: *mut list_T,
    mut idx: ::core::ffi::c_int,
    mut submatches: bool,
    mut matchbuf: bool,
) {
    let mut len: size_t = strlen(str);
    let mut match_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut startidx: colnr_T = 0 as colnr_T;
    loop {
        match_0 = vim_regexec_nl(rmp, str, startidx) as ::core::ffi::c_int;
        if match_0 == 0 {
            break;
        }
        let mut d: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(mlist, d);
        if matchbuf {
            tv_dict_add_nr(
                d,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                idx as varnumber_T,
            );
        } else {
            tv_dict_add_nr(
                d,
                b"idx\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                idx as varnumber_T,
            );
        }
        tv_dict_add_nr(
            d,
            b"byteidx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            (*rmp).startp[0 as ::core::ffi::c_int as usize].offset_from(str) as colnr_T
                as varnumber_T,
        );
        tv_dict_add_str_len(
            d,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*rmp).startp[0 as ::core::ffi::c_int as usize],
            (*rmp).endp[0 as ::core::ffi::c_int as usize]
                .offset_from((*rmp).startp[0 as ::core::ffi::c_int as usize])
                as ::core::ffi::c_int,
        );
        if submatches {
            let mut sml: *mut list_T = tv_list_alloc(
                (NSUBEXP as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as ptrdiff_t,
            );
            tv_dict_add_list(
                d,
                b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                sml,
            );
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while i < NSUBEXP as ::core::ffi::c_int {
                if (*rmp).endp[i as usize].is_null() {
                    tv_list_append_string(
                        sml,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as ssize_t,
                    );
                } else {
                    tv_list_append_string(
                        sml,
                        (*rmp).startp[i as usize],
                        (*rmp).endp[i as usize].offset_from((*rmp).startp[i as usize]) as ssize_t,
                    );
                }
                i += 1;
            }
        }
        startidx = (*rmp).endp[0 as ::core::ffi::c_int as usize].offset_from(str) as colnr_T;
        if startidx >= len as colnr_T
            || str.offset(startidx as isize)
                <= (*rmp).startp[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char
        {
            break;
        }
    }
}
pub unsafe extern "C" fn f_matchbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut retlist: *mut list_T = (*rettv).vval.v_list;
    if tv_check_for_buffer_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_lnum_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        || tv_check_for_lnum_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 4 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let prev_did_emsg: ::core::ffi::c_int = did_emsg.get();
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        if did_emsg.get() == prev_did_emsg {
            semsg(
                gettext(&raw const e_invalid_buffer_name_str as *const ::core::ffi::c_char),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            );
        }
        return;
    }
    if (*buf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_buffer_is_not_loaded as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut pat: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let mut slnum: linenr_T =
        tv_get_lnum_buf(argvars.offset(2 as ::core::ffi::c_int as isize), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if slnum < 1 as linenr_T {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut elnum: linenr_T =
        tv_get_lnum_buf(argvars.offset(3 as ::core::ffi::c_int as isize), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if elnum < 1 as linenr_T || elnum < slnum {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"end_lnum\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if elnum > (*buf).b_ml.ml_line_count {
        elnum = (*buf).b_ml.ml_line_count;
    }
    let mut submatches: bool = false_0 != 0;
    if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(4 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !d.is_null() {
            let mut di: *mut dictitem_T = tv_dict_find(
                d,
                b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() {
                if (*di).di_tv.v_type as ::core::ffi::c_uint
                    != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(
                        gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                        b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return;
                }
                submatches = tv_get_bool(&raw mut (*di).di_tv) != 0;
            }
        }
    }
    let save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = p_ic.get() != 0;
        while slnum <= elnum {
            let mut str: *const ::core::ffi::c_char = ml_get_buf(buf, slnum);
            get_matches_in_str(
                str,
                &raw mut regmatch,
                retlist,
                slnum as ::core::ffi::c_int,
                submatches,
                true_0 != 0,
            );
            slnum += 1;
        }
        vim_regfree(regmatch.regprog);
    }
    p_cpo.set(save_cpo);
}
pub unsafe extern "C" fn f_match(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatch);
}
pub unsafe extern "C" fn f_matchend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchEnd);
}
pub unsafe extern "C" fn f_matchlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchList);
}
pub unsafe extern "C" fn f_matchstr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchStr);
}
pub unsafe extern "C" fn f_matchstrlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut idx: ::core::ffi::c_int = 0;
    let mut submatches: bool = false;
    (*rettv).vval.v_number = -1 as varnumber_T;
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut retlist: *mut list_T = (*rettv).vval.v_list;
    if tv_check_for_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    l = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if l.is_null() {
        return;
    }
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    if pat.is_null() {
        return;
    }
    let save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = p_ic.get() != 0;
        submatches = false_0 != 0;
        '_cleanup: {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut d: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict;
                if !d.is_null() {
                    let mut di: *mut dictitem_T = tv_dict_find(
                        d,
                        b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    );
                    if !di.is_null() {
                        if (*di).di_tv.v_type as ::core::ffi::c_uint
                            != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            semsg(
                                gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                                b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            break '_cleanup;
                        } else {
                            submatches = tv_get_bool(&raw mut (*di).di_tv) != 0;
                        }
                    }
                }
            }
            idx = 0 as ::core::ffi::c_int;
            let l_: *const list_T = l;
            if !l_.is_null() {
                let mut li: *const listitem_T = (*l_).lv_first;
                while !li.is_null() {
                    let li_tv: *const typval_T = &raw const (*li).li_tv;
                    if (*li_tv).v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(*li_tv).vval.v_string.is_null()
                    {
                        let mut str: *const ::core::ffi::c_char = (*li_tv).vval.v_string;
                        get_matches_in_str(str, &raw mut regmatch, retlist, idx, submatches, false);
                    }
                    idx += 1;
                    li = (*li).li_next;
                }
            }
        }
        vim_regfree(regmatch.regprog);
    }
    p_cpo.set(save_cpo);
}
pub unsafe extern "C" fn f_matchstrpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchStrPos);
}
