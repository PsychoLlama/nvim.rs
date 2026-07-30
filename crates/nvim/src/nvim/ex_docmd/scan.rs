//! Scanning the rest of a command line: the count, the register, the
//! `!`, the `:p`-style flags, and where the next command begins.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn skip_colon_white(
    mut p: *const c_char,
    mut skipleadingwhite: bool,
) -> *mut c_char {
    if skipleadingwhite {
        p = skipwhite(p);
    }
    while *p as c_int == ':' as c_int {
        p = skipwhite(p.offset(1 as c_int as isize));
    }
    return p as *mut c_char;
}

pub(crate) unsafe extern "C" fn parse_register(mut eap: *mut exarg_T) {
    if (*eap).argt & EX_REGSTR as uint32_t != 0
        && *(*eap).arg as c_int != NUL
        && (!(((*eap).cmdidx as c_int) < 0 as c_int) || *(*eap).arg as c_int != '=' as c_int)
        && !((*eap).argt & EX_COUNT as uint32_t != 0
            && ascii_isdigit(*(*eap).arg as c_int) as c_int != 0)
    {
        if valid_yank_reg(
            *(*eap).arg as c_int,
            !(((*eap).cmdidx as c_int) < 0 as c_int)
                && (*eap).cmdidx as c_int != CMD_put as c_int
                && (*eap).cmdidx as c_int != CMD_iput as c_int,
        ) {
            let c2rust_fresh25 = (*eap).arg;
            (*eap).arg = (*eap).arg.offset(1);
            (*eap).regname = *c2rust_fresh25 as uint8_t as c_int;
            if *(*eap).arg.offset(-1 as c_int as isize) as c_int == '=' as c_int
                && *(*eap).arg.offset(0 as c_int as isize) as c_int != NUL
            {
                if (*eap).skip == 0 {
                    set_expr_line(xstrdup((*eap).arg));
                }
                (*eap).arg = (*eap).arg.offset(strlen((*eap).arg) as isize);
            }
            (*eap).arg = skipwhite((*eap).arg);
        }
    }
}

pub unsafe extern "C" fn set_cmd_count(
    mut eap: *mut exarg_T,
    mut count: linenr_T,
    mut validate: bool,
) {
    if (*eap).addr_type as c_uint != ADDR_LINES as c_int as c_uint {
        (*eap).line2 = count;
        if (*eap).addr_count == 0 as c_int {
            (*eap).addr_count = 1 as c_int;
        }
    } else {
        (*eap).line1 = (*eap).line2;
        if (*eap).line2 >= INT32_MAX as linenr_T - (count - 1 as linenr_T) {
            (*eap).line2 = INT32_MAX as linenr_T;
        } else {
            (*eap).line2 = ((*eap).line2 as c_int + (count - 1 as linenr_T) as c_int) as linenr_T;
        }
        (*eap).addr_count += 1;
        if validate as c_int != 0 && (*eap).line2 > (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
    };
}

pub(crate) unsafe extern "C" fn parse_count(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut validate: bool,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if (*eap).argt & EX_COUNT as uint32_t != 0
        && ascii_isdigit(*(*eap).arg as c_int) as c_int != 0
        && ((*eap).argt & EX_BUFNAME as uint32_t == 0
            || {
                p = skipdigits((*eap).arg.offset(1 as c_int as isize));
                *p as c_int == NUL
            }
            || ascii_iswhite(*p as c_int) as c_int != 0)
    {
        let mut n: linenr_T =
            getdigits_int32(&raw mut (*eap).arg, false_0 != 0, INT32_MAX as int32_t);
        (*eap).arg = skipwhite((*eap).arg);
        if !(*eap).args.is_null() {
            '_c2rust_label: {
                if (*eap).argc > 0 as size_t
                    && (*eap).arg >= *(*eap).args.offset(0 as c_int as isize)
                {
                } else {
                    __assert_fail(
                        b"eap->argc > 0 && eap->arg >= eap->args[0]\0".as_ptr() as *const c_char,
                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                        1467 as c_uint,
                        b"int parse_count(exarg_T *, const char **, _Bool)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            if (*eap).arg
                < (*(*eap).args.offset(0 as c_int as isize))
                    .offset(*(*eap).arglens.offset(0 as c_int as isize) as isize)
            {
                *(*eap).arglens.offset(0 as c_int as isize) =
                    (*(*eap).arglens.offset(0 as c_int as isize)).wrapping_sub(
                        (*eap)
                            .arg
                            .offset_from(*(*eap).args.offset(0 as c_int as isize))
                            as size_t,
                    );
                *(*eap).args.offset(0 as c_int as isize) = (*eap).arg;
            } else {
                shift_cmd_args(eap);
            }
        }
        if n <= 0 as linenr_T && (*eap).argt & EX_ZEROR as uint32_t == 0 as uint32_t {
            if !errormsg.is_null() {
                *errormsg = gettext(&raw const e_zerocount as *const c_char);
            }
            return FAIL;
        }
        set_cmd_count(eap, n, validate);
    }
    return OK;
}

pub(crate) unsafe extern "C" fn parse_bang(
    mut eap: *const exarg_T,
    mut p: *mut *mut c_char,
) -> bool {
    if **p as c_int == '!' as c_int
        && (*eap).cmdidx as c_int != CMD_substitute as c_int
        && (*eap).cmdidx as c_int != CMD_smagic as c_int
        && (*eap).cmdidx as c_int != CMD_snomagic as c_int
    {
        *p = (*p).offset(1);
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn get_flags(mut eap: *mut exarg_T) {
    while !vim_strchr(
        b"lp#\0".as_ptr() as *const c_char,
        *(*eap).arg as uint8_t as c_int,
    )
    .is_null()
    {
        if *(*eap).arg as c_int == 'l' as c_int {
            (*eap).flags |= EXFLAG_LIST;
        } else if *(*eap).arg as c_int == 'p' as c_int {
            (*eap).flags |= EXFLAG_PRINT;
        } else {
            (*eap).flags |= EXFLAG_NR;
        }
        (*eap).arg = skipwhite((*eap).arg.offset(1 as c_int as isize));
    }
}

pub(crate) unsafe extern "C" fn skip_grep_pat(mut eap: *mut exarg_T) -> *mut c_char {
    let mut p: *mut c_char = (*eap).arg;
    if *p as c_int != NUL
        && ((*eap).cmdidx as c_int == CMD_vimgrep as c_int
            || (*eap).cmdidx as c_int == CMD_lvimgrep as c_int
            || (*eap).cmdidx as c_int == CMD_vimgrepadd as c_int
            || (*eap).cmdidx as c_int == CMD_lvimgrepadd as c_int
            || grep_internal((*eap).cmdidx) != 0)
    {
        p = skip_vimgrep_pat(
            p,
            ::core::ptr::null_mut::<*mut c_char>(),
            ::core::ptr::null_mut::<c_int>(),
        );
        if p.is_null() {
            p = (*eap).arg;
        }
    }
    return p;
}

pub unsafe extern "C" fn separate_nextcmd(mut eap: *mut exarg_T) {
    let mut p: *mut c_char = skip_grep_pat(eap);
    while *p != 0 {
        if *p as c_int == Ctrl_V {
            if (*eap).argt & (EX_CTRLV as uint32_t | EX_XFILE as uint32_t) != 0 {
                p = p.offset(1);
            } else {
                memmove(
                    p as *mut c_void,
                    p.offset(1 as c_int as isize) as *const c_void,
                    strlen(p.offset(1 as c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            if *p as c_int == NUL {
                break;
            }
        } else if *p.offset(0 as c_int as isize) as c_int == '`' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '=' as c_int
            && (*eap).argt & EX_XFILE as uint32_t != 0
        {
            p = p.offset(2 as c_int as isize);
            skip_expr(&raw mut p, ::core::ptr::null_mut::<evalarg_T>());
            if *p as c_int == NUL {
                break;
            }
        } else if *p as c_int == '"' as c_int
            && (*eap).argt & EX_NOTRLCOM as uint32_t == 0
            && ((*eap).cmdidx as c_int != CMD_at as c_int || p != (*eap).arg)
            && ((*eap).cmdidx as c_int != CMD_redir as c_int
                || p != (*eap).arg.offset(1 as c_int as isize)
                || *p.offset(-1 as c_int as isize) as c_int != '@' as c_int)
            || *p as c_int == '|' as c_int
                && (*eap).cmdidx as c_int != CMD_append as c_int
                && (*eap).cmdidx as c_int != CMD_change as c_int
                && (*eap).cmdidx as c_int != CMD_insert as c_int
            || *p as c_int == '\n' as c_int
        {
            if (vim_strchr(p_cpo.get(), CPO_BAR).is_null()
                || (*eap).argt & EX_CTRLV as uint32_t == 0)
                && *p.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int
            {
                memmove(
                    p.offset(-(1 as c_int as isize)) as *mut c_void,
                    p as *const c_void,
                    strlen(p).wrapping_add(1 as size_t),
                );
                p = p.offset(-1);
            } else {
                (*eap).nextcmd = check_nextcmd(p);
                *p = NUL as c_char;
                break;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if (*eap).argt & EX_NOTRLCOM as uint32_t == 0 {
        del_trailing_spaces((*eap).arg);
    }
}

pub unsafe extern "C" fn skip_cmd_arg(mut p: *mut c_char, mut rembs: bool) -> *mut c_char {
    while *p as c_int != 0 && !ascii_isspace(*p as c_int) {
        if *p as c_int == '\\' as c_int && *p.offset(1 as c_int as isize) as c_int != NUL {
            if rembs {
                memmove(
                    p as *mut c_void,
                    p.offset(1 as c_int as isize) as *const c_void,
                    strlen(p.offset(1 as c_int as isize)).wrapping_add(1 as size_t),
                );
            } else {
                p = p.offset(1);
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return p;
}

pub unsafe extern "C" fn ends_excmd(mut c: c_int) -> c_int {
    return (c == NUL || c == '|' as c_int || c == '"' as c_int || c == '\n' as c_int) as c_int;
}

pub unsafe extern "C" fn find_nextcmd(mut p: *const c_char) -> *mut c_char {
    while *p as c_int != '|' as c_int && *p as c_int != '\n' as c_int {
        if *p as c_int == NUL {
            return ::core::ptr::null_mut::<c_char>();
        }
        p = p.offset(1);
    }
    return (p as *mut c_char).offset(1 as c_int as isize);
}

pub unsafe extern "C" fn check_nextcmd(mut p: *mut c_char) -> *mut c_char {
    let mut s: *mut c_char = skipwhite(p);
    if *s as c_int == '|' as c_int || *s as c_int == '\n' as c_int {
        return s.offset(1 as c_int as isize);
    }
    return ::core::ptr::null_mut::<c_char>();
}
