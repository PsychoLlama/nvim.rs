//! The `:syntax` item options, and the containment test.
//!
//! [`get_syn_options`] parses the flag words (`contained`, `oneline`, `keepend`,
//! `conceal`, `nextgroup=`, ...) that may follow any item definition, and
//! [`get_id_list`] parses a group list (`contains=a,b,@cl,ALLBUT,TOP`) into the
//! `int16_t` id array the state machine tests against. [`in_id_list`] is that
//! test -- it runs once per candidate pattern per column, so it is on the
//! per-cell path even though the rest of this module is not.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_group_name(
    mut arg: *mut ::core::ffi::c_char,
    mut name_end: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        *name_end = skiptowhite(arg);
        let mut rest: *mut ::core::ffi::c_char = skipwhite(*name_end);
        if ends_excmd(*arg as ::core::ffi::c_int) != 0 || *rest as ::core::ffi::c_int == NUL {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return rest;
    }
}

pub(crate) unsafe extern "C" fn get_syn_options(
    mut arg: *mut ::core::ffi::c_char,
    mut opt: *mut syn_opt_arg_T,
    mut conceal_char: *mut ::core::ffi::c_int,
    mut skip: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fidx: ::core::ffi::c_int = 0;
        static flagtab: GlobalCell<[flag; 19]> = GlobalCell::new([
            flag {
                name: b"cCoOnNtTaAiInNeEdD\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_CONTAINED,
            },
            flag {
                name: b"oOnNeElLiInNeE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_ONELINE,
            },
            flag {
                name: b"kKeEeEpPeEnNdD\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_KEEPEND,
            },
            flag {
                name: b"eExXtTeEnNdD\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_EXTEND,
            },
            flag {
                name: b"eExXcClLuUdDeEnNlL\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_EXCLUDENL,
            },
            flag {
                name: b"tTrRaAnNsSpPaArReEnNtT\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_TRANSP,
            },
            flag {
                name: b"sSkKiIpPnNlL\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_SKIPNL,
            },
            flag {
                name: b"sSkKiIpPwWhHiItTeE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_SKIPWHITE,
            },
            flag {
                name: b"sSkKiIpPeEmMpPtTyY\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_SKIPEMPTY,
            },
            flag {
                name: b"gGrRoOuUpPhHeErReE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_SYNC_HERE,
            },
            flag {
                name: b"gGrRoOuUpPtThHeErReE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_SYNC_THERE,
            },
            flag {
                name: b"dDiIsSpPlLaAyY\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_DISPLAY,
            },
            flag {
                name: b"fFoOlLdD\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_FOLD,
            },
            flag {
                name: b"cCoOnNcCeEaAlL\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_CONCEAL,
            },
            flag {
                name: b"cCoOnNcCeEaAlLeEnNdDsS\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 0 as ::core::ffi::c_int,
                flags: HL_CONCEALENDS,
            },
            flag {
                name: b"cCcChHaArR\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 11 as ::core::ffi::c_int,
                flags: 0 as ::core::ffi::c_int,
            },
            flag {
                name: b"cCoOnNtTaAiInNsS\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 1 as ::core::ffi::c_int,
                flags: 0 as ::core::ffi::c_int,
            },
            flag {
                name: b"cCoOnNtTaAiInNeEdDiInN\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 2 as ::core::ffi::c_int,
                flags: 0 as ::core::ffi::c_int,
            },
            flag {
                name: b"nNeExXtTgGrRoOuUpP\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                argtype: 3 as ::core::ffi::c_int,
                flags: 0 as ::core::ffi::c_int,
            },
        ]);
        static first_letters: GlobalCell<*const ::core::ffi::c_char> =
            GlobalCell::new(b"cCoOkKeEtTsSgGdDfFnN\0".as_ptr() as *const ::core::ffi::c_char);
        if arg.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if (*(*curwin.get()).w_s).b_syn_conceal != 0 {
            (*opt).flags |= HL_CONCEAL;
        }
        while !strchr(first_letters.get(), *arg as ::core::ffi::c_int).is_null() {
            fidx = ::core::mem::size_of::<[flag; 19]>()
                .wrapping_div(::core::mem::size_of::<flag>())
                .wrapping_div(
                    (::core::mem::size_of::<[flag; 19]>()
                        .wrapping_rem(::core::mem::size_of::<flag>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int;
            loop {
                fidx -= 1;
                if fidx < 0 as ::core::ffi::c_int {
                    break;
                }
                let mut p: *mut ::core::ffi::c_char = (*flagtab.ptr())[fidx as usize].name;
                let mut i: ::core::ffi::c_int = 0;
                i = 0 as ::core::ffi::c_int;
                len = 0 as ::core::ffi::c_int;
                while *p.offset(i as isize) as ::core::ffi::c_int != NUL {
                    if *arg.offset(len as isize) as ::core::ffi::c_int
                        != *p.offset(i as isize) as ::core::ffi::c_int
                        && *arg.offset(len as isize) as ::core::ffi::c_int
                            != *p.offset((i + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                    {
                        break;
                    }
                    i += 2 as ::core::ffi::c_int;
                    len += 1;
                }
                if !(*p.offset(i as isize) as ::core::ffi::c_int == NUL
                    && (ascii_iswhite(*arg.offset(len as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                        || (if (*flagtab.ptr())[fidx as usize].argtype > 0 as ::core::ffi::c_int {
                            (*arg.offset(len as isize) as ::core::ffi::c_int
                                == '=' as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                        } else {
                            ends_excmd(*arg.offset(len as isize) as ::core::ffi::c_int)
                        }) != 0))
                {
                    continue;
                }
                if (*opt).keyword as ::core::ffi::c_int != 0
                    && ((*flagtab.ptr())[fidx as usize].flags == HL_DISPLAY
                        || (*flagtab.ptr())[fidx as usize].flags == HL_FOLD
                        || (*flagtab.ptr())[fidx as usize].flags == HL_EXTEND)
                {
                    fidx = -1 as ::core::ffi::c_int;
                }
                break;
            }
            if fidx < 0 as ::core::ffi::c_int {
                break;
            }
            if (*flagtab.ptr())[fidx as usize].argtype == 1 as ::core::ffi::c_int {
                if !(*opt).has_cont_list {
                    emsg(gettext(
                        (e_contains_argument_not_accepted_here.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                if get_id_list(
                    &raw mut arg,
                    8 as ::core::ffi::c_int,
                    &raw mut (*opt).cont_list,
                    skip != 0,
                ) == FAIL
                {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else if (*flagtab.ptr())[fidx as usize].argtype == 2 as ::core::ffi::c_int {
                if get_id_list(
                    &raw mut arg,
                    11 as ::core::ffi::c_int,
                    &raw mut (*opt).cont_in_list,
                    skip != 0,
                ) == FAIL
                {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else if (*flagtab.ptr())[fidx as usize].argtype == 3 as ::core::ffi::c_int {
                if get_id_list(
                    &raw mut arg,
                    9 as ::core::ffi::c_int,
                    &raw mut (*opt).next_list,
                    skip != 0,
                ) == FAIL
                {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else if (*flagtab.ptr())[fidx as usize].argtype == 11 as ::core::ffi::c_int
                && *arg.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int
            {
                *conceal_char = utf_ptr2char(arg.offset(6 as ::core::ffi::c_int as isize));
                arg = arg.offset(
                    (utfc_ptr2len(arg.offset(6 as ::core::ffi::c_int as isize))
                        - 1 as ::core::ffi::c_int) as isize,
                );
                if !vim_isprintc(*conceal_char) {
                    emsg(gettext(
                        (e_invalid_cchar_value.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
            } else {
                (*opt).flags |= (*flagtab.ptr())[fidx as usize].flags;
                arg = skipwhite(arg.offset(len as isize));
                if (*flagtab.ptr())[fidx as usize].flags == HL_SYNC_HERE
                    || (*flagtab.ptr())[fidx as usize].flags == HL_SYNC_THERE
                {
                    if (*opt).sync_idx.is_null() {
                        emsg(gettext(b"E393: group[t]here not accepted here\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    let mut gname_start: *mut ::core::ffi::c_char = arg;
                    arg = skiptowhite(arg);
                    if gname_start == arg {
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    let mut gname: *mut ::core::ffi::c_char =
                        xstrnsave(gname_start, arg.offset_from(gname_start) as size_t);
                    if strcmp(gname, b"NONE\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                    {
                        *(*opt).sync_idx = NONE_IDX;
                    } else {
                        let mut syn_id: ::core::ffi::c_int = syn_name2id(gname);
                        let mut i_0: ::core::ffi::c_int = 0;
                        i_0 = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
                        loop {
                            i_0 -= 1;
                            if i_0 < 0 as ::core::ffi::c_int {
                                break;
                            }
                            if !((*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(i_0 as isize))
                            .sp_syn
                            .id as ::core::ffi::c_int
                                == syn_id
                                && (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data
                                    as *mut synpat_T)
                                    .offset(i_0 as isize))
                                .sp_type as ::core::ffi::c_int
                                    == SPTYPE_START)
                            {
                                continue;
                            }
                            *(*opt).sync_idx = i_0;
                            break;
                        }
                        if i_0 < 0 as ::core::ffi::c_int {
                            semsg(
                                gettext(b"E394: Didn't find region item for %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                gname,
                            );
                            xfree(gname as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                    }
                    xfree(gname as *mut ::core::ffi::c_void);
                    arg = skipwhite(arg);
                } else if (*flagtab.ptr())[fidx as usize].flags == HL_FOLD
                    && foldmethodIsSyntax(curwin.get()) as ::core::ffi::c_int != 0
                {
                    foldUpdateAll(curwin.get());
                }
            }
        }
        return arg;
    }
}

pub(crate) unsafe extern "C" fn get_id_list(
    arg: *mut *mut ::core::ffi::c_char,
    keylen: ::core::ffi::c_int,
    list: *mut *mut int16_t,
    skip: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut total_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut retval: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut id: ::core::ffi::c_int = 0;
        let mut failed: bool = false_0 != 0;
        let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while round <= 2 as ::core::ffi::c_int {
            p = skipwhite((*arg).offset(keylen as isize));
            if *p as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"E405: Missing equal sign: %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    *arg,
                );
                break;
            } else {
                p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                if ends_excmd(*p as ::core::ffi::c_int) != 0 {
                    semsg(
                        gettext(
                            b"E406: Empty argument: %s\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        *arg,
                    );
                    break;
                } else {
                    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    loop {
                        end = p;
                        while *end as ::core::ffi::c_int != 0
                            && !ascii_iswhite(*end as ::core::ffi::c_int)
                            && *end as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        {
                            end = end.offset(1);
                        }
                        let name: *mut ::core::ffi::c_char =
                            xmalloc((end.offset_from(p) as size_t).wrapping_add(3 as size_t))
                                as *mut ::core::ffi::c_char;
                        xmemcpyz(
                            name.offset(1 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            p as *const ::core::ffi::c_void,
                            end.offset_from(p) as size_t,
                        );
                        if strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"ALLBUT\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                            || strcmp(
                                name.offset(1 as ::core::ffi::c_int as isize),
                                b"ALL\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            || strcmp(
                                name.offset(1 as ::core::ffi::c_int as isize),
                                b"TOP\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            || strcmp(
                                name.offset(1 as ::core::ffi::c_int as isize),
                                b"CONTAINED\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                        {
                            if (if (**arg as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                                || **arg as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                            {
                                **arg as ::core::ffi::c_int
                            } else {
                                **arg as ::core::ffi::c_int
                                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            }) != 'C' as ::core::ffi::c_int
                            {
                                semsg(
                                    gettext(b"E407: %s not allowed here\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    name.offset(1 as ::core::ffi::c_int as isize),
                                );
                                failed = true_0 != 0;
                                xfree(name as *mut ::core::ffi::c_void);
                                break;
                            } else if count != 0 as ::core::ffi::c_int {
                                semsg(
                                    gettext(b"E408: %s must be first in contains list\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    name.offset(1 as ::core::ffi::c_int as isize),
                                );
                                failed = true_0 != 0;
                                xfree(name as *mut ::core::ffi::c_void);
                                break;
                            } else {
                                if *name.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'A' as ::core::ffi::c_int
                                {
                                    id = MAX_HL_ID as ::core::ffi::c_int;
                                } else if *name.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'T' as ::core::ffi::c_int
                                {
                                    id = SYNID_TOP;
                                } else {
                                    id = SYNID_CONTAINED;
                                }
                                id += current_syn_inc_tag.get();
                            }
                        } else if *name.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == '@' as ::core::ffi::c_int
                        {
                            if skip {
                                id = -1 as ::core::ffi::c_int;
                            } else {
                                id = syn_check_cluster(
                                    name.offset(2 as ::core::ffi::c_int as isize),
                                    (end.offset_from(p) - 1 as isize) as ::core::ffi::c_int,
                                );
                            }
                        } else if strpbrk(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"\\.*^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                        )
                        .is_null()
                        {
                            id = syn_check_group(
                                name.offset(1 as ::core::ffi::c_int as isize),
                                end.offset_from(p) as size_t,
                            );
                        } else {
                            *name = '^' as ::core::ffi::c_char;
                            strcat(name, b"$\0".as_ptr() as *const ::core::ffi::c_char);
                            regmatch.regprog = vim_regcomp(name, RE_MAGIC);
                            if regmatch.regprog.is_null() {
                                failed = true_0 != 0;
                                xfree(name as *mut ::core::ffi::c_void);
                                break;
                            } else {
                                regmatch.rm_ic = true_0 != 0;
                                id = 0 as ::core::ffi::c_int;
                                let mut i: ::core::ffi::c_int = highlight_num_groups();
                                loop {
                                    i -= 1;
                                    if i < 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                    if vim_regexec(
                                        &raw mut regmatch,
                                        highlight_group_name(i),
                                        0 as colnr_T,
                                    ) {
                                        if round == 2 as ::core::ffi::c_int {
                                            if count >= total_count {
                                                xfree(retval as *mut ::core::ffi::c_void);
                                                round = 1 as ::core::ffi::c_int;
                                            } else {
                                                *retval.offset(count as isize) =
                                                    (i + 1 as ::core::ffi::c_int) as int16_t;
                                            }
                                        }
                                        count += 1;
                                        id = -1 as ::core::ffi::c_int;
                                    }
                                }
                                vim_regfree(regmatch.regprog);
                            }
                        }
                        xfree(name as *mut ::core::ffi::c_void);
                        if id == 0 as ::core::ffi::c_int {
                            semsg(
                                gettext(b"E409: Unknown group name: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                p,
                            );
                            failed = true_0 != 0;
                            break;
                        } else {
                            if id > 0 as ::core::ffi::c_int {
                                if round == 2 as ::core::ffi::c_int {
                                    if count >= total_count {
                                        xfree(retval as *mut ::core::ffi::c_void);
                                        round = 1 as ::core::ffi::c_int;
                                    } else {
                                        *retval.offset(count as isize) = id as int16_t;
                                    }
                                }
                                count += 1;
                            }
                            p = skipwhite(end);
                            if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                                break;
                            }
                            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                            if ends_excmd(*p as ::core::ffi::c_int) != 0 {
                                break;
                            }
                        }
                    }
                    if failed {
                        break;
                    }
                    if round == 1 as ::core::ffi::c_int {
                        retval = xmalloc(
                            (count as size_t)
                                .wrapping_add(1 as size_t)
                                .wrapping_mul(::core::mem::size_of::<int16_t>()),
                        ) as *mut int16_t;
                        *retval.offset(count as isize) = 0 as int16_t;
                        total_count = count;
                    }
                    round += 1;
                }
            }
        }
        *arg = p;
        if failed as ::core::ffi::c_int != 0 || retval.is_null() {
            xfree(retval as *mut ::core::ffi::c_void);
            return FAIL;
        }
        if (*list).is_null() {
            *list = retval;
        } else {
            xfree(retval as *mut ::core::ffi::c_void);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn copy_id_list(list: *const int16_t) -> *mut int16_t {
    unsafe {
        if list.is_null() {
            return ::core::ptr::null_mut::<int16_t>();
        }
        let mut count: ::core::ffi::c_int = 0;
        count = 0 as ::core::ffi::c_int;
        while *list.offset(count as isize) != 0 {
            count += 1;
        }
        let len: size_t = (count as size_t)
            .wrapping_add(1 as size_t)
            .wrapping_mul(::core::mem::size_of::<int16_t>());
        let retval: *mut int16_t = xmalloc(len) as *mut int16_t;
        memmove(
            retval as *mut ::core::ffi::c_void,
            list as *const ::core::ffi::c_void,
            len,
        );
        return retval;
    }
}

pub(crate) unsafe extern "C" fn in_id_list(
    mut cur_si: *mut stateitem_T,
    mut list: *mut int16_t,
    mut ssp: *mut sp_syn,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0;
        let mut id: int16_t = (*ssp).id;
        static depth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if !cur_si.is_null() && !(*ssp).cont_in_list.is_null() && (*cur_si).si_flags & HL_MATCH == 0
        {
            while (*cur_si).si_flags & HL_TRANS_CONT != 0
                && cur_si > (*current_state.ptr()).ga_data as *mut stateitem_T
            {
                cur_si = cur_si.offset(-1);
            }
            if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && in_id_list(
                    ::core::ptr::null_mut::<stateitem_T>(),
                    (*ssp).cont_in_list,
                    &raw mut (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset((*cur_si).si_idx as isize))
                    .sp_syn,
                    (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset((*cur_si).si_idx as isize))
                    .sp_flags,
                ) != 0
            {
                return true_0;
            }
        }
        if list.is_null() {
            return false_0;
        }
        if list == ID_LIST_ALL {
            return (flags & HL_CONTAINED == 0) as ::core::ffi::c_int;
        }
        let mut toplevel: bool = flags & HL_CONTAINED == 0 || flags & HL_INCLUDED_TOPLEVEL != 0;
        let mut item: int16_t = *list;
        if item as ::core::ffi::c_int >= MAX_HL_ID as ::core::ffi::c_int
            && (item as ::core::ffi::c_int) < SYNID_CLUSTER
        {
            if (item as ::core::ffi::c_int) < SYNID_TOP {
                if item as ::core::ffi::c_int - MAX_HL_ID as ::core::ffi::c_int != (*ssp).inc_tag {
                    return false_0;
                }
            } else if (item as ::core::ffi::c_int) < SYNID_CONTAINED {
                if item as ::core::ffi::c_int - SYNID_TOP != (*ssp).inc_tag || !toplevel {
                    return false_0;
                }
            } else if item as ::core::ffi::c_int - SYNID_CONTAINED != (*ssp).inc_tag
                || toplevel as ::core::ffi::c_int != 0
            {
                return false_0;
            }
            list = list.offset(1);
            item = *list;
            retval = false_0;
        } else {
            retval = true_0;
        }
        while item as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if item as ::core::ffi::c_int == id as ::core::ffi::c_int {
                return retval;
            }
            if item as ::core::ffi::c_int >= SYNID_CLUSTER {
                let mut scl_list: *mut int16_t = (*((*syn_block.get()).b_syn_clusters.ga_data
                    as *mut syn_cluster_T)
                    .offset((item as ::core::ffi::c_int - SYNID_CLUSTER) as isize))
                .scl_list;
                if !scl_list.is_null() && depth.get() < 30 as ::core::ffi::c_int {
                    (*depth.ptr()) += 1;
                    let mut r: ::core::ffi::c_int =
                        in_id_list(::core::ptr::null_mut::<stateitem_T>(), scl_list, ssp, flags);
                    (*depth.ptr()) -= 1;
                    if r != 0 {
                        return retval;
                    }
                }
            }
            list = list.offset(1);
            item = *list;
        }
        return (retval == 0) as ::core::ffi::c_int;
    }
}
