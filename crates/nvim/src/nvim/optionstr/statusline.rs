//! The callbacks for the options holding a format string, and for the
//! session/history/shell specs alongside them.
//!
//! They are `pub` only so the generated option table can name them.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_iconstring(mut args: *mut optset_T) -> *const c_char {
    return did_set_titleiconstring(args, STL_IN_ICON);
}

pub unsafe extern "C" fn did_set_rulerformat(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, true_0 != 0, false_0 != 0);
}

pub unsafe extern "C" fn did_set_sessionoptions(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if ssop_flags.get() & kOptSsopFlagCurdir as c_int as c_uint != 0
        && ssop_flags.get() & kOptSsopFlagSesdir as c_int as c_uint != 0
    {
        let mut oldval: *const c_char = (*args).os_oldval.string.data;
        opt_strings_flags(
            oldval,
            opt_ssop_values.ptr() as *mut *const c_char,
            ssop_flags.ptr(),
            true_0 != 0,
        );
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_shada(mut args: *mut optset_T) -> *const c_char {
    let mut errbuf: *mut c_char = (*args).os_errbuf;
    let mut errbuflen: size_t = (*args).os_errbuflen;
    let mut s: *mut c_char = p_shada.get();
    while *s != 0 {
        if vim_strchr(
            b"!\"%'/:<@cfhnrs\0".as_ptr() as *const c_char,
            *s as uint8_t as c_int,
        )
        .is_null()
        {
            return illegal_char(errbuf, errbuflen, *s as uint8_t as c_int);
        }
        if *s as c_int == 'n' as c_int {
            break;
        }
        if *s as c_int == 'r' as c_int {
            loop {
                s = s.offset(1);
                if !(*s as c_int != 0 && *s as c_int != ',' as c_int) {
                    break;
                }
            }
        } else if *s as c_int == '%' as c_int {
            loop {
                s = s.offset(1);
                if !ascii_isdigit(*s as c_int) {
                    break;
                }
            }
        } else if *s as c_int == '!' as c_int
            || *s as c_int == 'h' as c_int
            || *s as c_int == 'c' as c_int
        {
            s = s.offset(1);
        } else {
            loop {
                s = s.offset(1);
                if !ascii_isdigit(*s as c_int) {
                    break;
                }
            }
            if !ascii_isdigit(*s.offset(-(1 as c_int as isize)) as c_int) {
                if !errbuf.is_null() {
                    vim_snprintf(
                        errbuf,
                        errbuflen,
                        gettext(b"E526: Missing number after <%s>\0".as_ptr() as *const c_char),
                        transchar_byte(*s.offset(-(1 as c_int as isize)) as uint8_t as c_int),
                    );
                    return errbuf;
                } else {
                    return b"\0".as_ptr() as *const c_char;
                }
            }
        }
        if *s as c_int == ',' as c_int {
            s = s.offset(1);
        } else if *s != 0 {
            if !errbuf.is_null() {
                return b"E527: Missing comma\0".as_ptr() as *const c_char;
            } else {
                return b"\0".as_ptr() as *const c_char;
            }
        }
    }
    if *p_shada.get() as c_int != 0 && get_shada_parameter('\'' as c_int) < 0 as c_int {
        return b"E528: Must specify a ' value\0".as_ptr() as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_shellpipe_redir(mut args: *mut optset_T) -> *const c_char {
    let mut seen: bool = false_0 != 0;
    let mut p: *mut c_char = (*args).os_newval.string.data;
    while *p as c_int != NUL {
        if *p as c_int == '%' as c_int {
            if *p.offset(1 as c_int as isize) as c_int == NUL {
                return &raw const e_invalid_format_string_single_percent_s as *const c_char;
            }
            if *p.offset(1 as c_int as isize) as c_int == '%' as c_int {
                p = p.offset(1);
            } else if *p.offset(1 as c_int as isize) as c_int == 's' as c_int {
                if seen {
                    return &raw const e_invalid_format_string_single_percent_s as *const c_char;
                }
                seen = true_0 != 0;
                p = p.offset(1);
            } else {
                return &raw const e_invalid_format_string_single_percent_s as *const c_char;
            }
        }
        p = p.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_shortmess(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        SHM_ALL.ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}

pub unsafe extern "C" fn did_set_statuscolumn(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, true_0 != 0);
}

pub unsafe extern "C" fn did_set_statusline(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}

pub(crate) unsafe extern "C" fn did_set_statustabline_rulerformat(
    mut args: *mut optset_T,
    mut rulerformat: bool,
    mut statuscolumn: bool,
) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if rulerformat {
        ru_wid.set(0 as c_int);
    } else if statuscolumn {
        (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    }
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut s: *mut c_char = *varp;
    let mut is_stl: bool = (*args).os_idx as c_int == kOptStatusline as c_int;
    if is_stl as c_int != 0
        && ((*args).os_flags & OPT_GLOBAL as c_int != 0
            || (*args).os_flags & OPT_LOCAL as c_int == 0)
        && *s.offset(0 as c_int as isize) as c_int == NUL
    {
        xfree(*varp as *mut c_void);
        *varp = xstrdup(
            get_option_default((*args).os_idx, (*args).os_flags)
                .data
                .string
                .data,
        );
        s = *varp;
    }
    if is_stl as c_int != 0 && !win.is_null() && (*win).w_floating as c_int != 0 {
        win_config_float(win, (*win).w_config);
    }
    if rulerformat as c_int != 0 && *s as c_int == '%' as c_int {
        s = s.offset(1);
        if *s as c_int == '-' as c_int {
            s = s.offset(1);
        }
        let mut wid: c_int = getdigits_int(&raw mut s, true_0 != 0, 0 as c_int);
        if wid != 0 && *s as c_int == '(' as c_int && {
            errmsg = check_stl_option(p_ruf.get());
            errmsg.is_null()
        } {
            ru_wid.set(wid);
        } else if *(*varp).offset(1 as c_int as isize) as c_int != '!' as c_int {
            errmsg = check_stl_option(p_ruf.get());
        }
    } else if rulerformat as c_int != 0
        || *s.offset(0 as c_int as isize) as c_int != '%' as c_int
        || *s.offset(1 as c_int as isize) as c_int != '!' as c_int
    {
        errmsg = check_stl_option(s);
    }
    if rulerformat as c_int != 0 && errmsg.is_null() {
        comp_col();
    }
    return errmsg;
}

pub unsafe extern "C" fn did_set_tabline(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}

pub(crate) unsafe extern "C" fn did_set_titleiconstring(
    mut args: *mut optset_T,
    mut flagval: c_int,
) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !vim_strchr(*varp, '%' as c_int).is_null() && check_stl_option(*varp).is_null() {
        (*stl_syntax.ptr()) |= flagval;
    } else {
        (*stl_syntax.ptr()) &= !flagval;
    }
    did_set_title();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_titlestring(mut args: *mut optset_T) -> *const c_char {
    return did_set_titleiconstring(args, STL_IN_TITLE);
}

pub unsafe extern "C" fn did_set_verbosefile(mut _args: *mut optset_T) -> *const c_char {
    verbose_stop();
    if *p_vfile.get() as c_int != NUL && verbose_open() == FAIL {
        return &raw const e_invarg as *const c_char as *mut c_char;
    }
    return ::core::ptr::null::<c_char>();
}
