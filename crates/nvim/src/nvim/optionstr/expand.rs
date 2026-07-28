//! Command-line completion of a string option's value.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn expand_set_str_generic(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let (values, values_len) = opt_values((*args).oe_idx);
    return expand_set_opt_string(args, values.cast_mut(), values_len, numMatches, matches);
}

pub(crate) unsafe extern "C" fn expand_set_opt_string(
    mut args: *mut optexpand_T,
    mut values: *mut *const c_char,
    mut numValues: size_t,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut regmatch: *mut regmatch_T = (*args).oe_regmatch;
    let mut include_orig_val: bool = (*args).oe_include_orig_val;
    let mut option_val: *mut c_char = (*args).oe_opt_value;
    *matches = xmalloc(
        ::core::mem::size_of::<*mut c_char>().wrapping_mul(numValues.wrapping_add(1 as size_t)),
    ) as *mut *mut c_char;
    let mut count: c_int = 0 as c_int;
    if include_orig_val as c_int != 0 && *option_val as c_int != NUL {
        let c2rust_fresh0 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh0 as isize);
        *c2rust_lvalue_ptr = xstrdup(option_val);
    }
    let mut val: *mut *const c_char = values;
    while !(*val).is_null() {
        's_27: {
            if **val as c_int != NUL {
                if include_orig_val as c_int != 0 && *option_val as c_int != NUL {
                    if strcmp(*val, option_val) == 0 as c_int {
                        break 's_27;
                    }
                }
                if vim_regexec(regmatch, *val, 0 as colnr_T) {
                    let c2rust_fresh1 = count;
                    count = count + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh1 as isize);
                    *c2rust_lvalue_ptr_0 = xstrdup(*val);
                }
            }
        }
        val = val.offset(1);
    }
    if count == 0 as c_int {
        let mut ptr_: *mut *mut c_void = matches as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return FAIL;
    }
    *numMatches = count;
    return OK;
}

pub(crate) unsafe extern "C" fn expand_set_opt_callback(
    mut xp: *mut expand_T,
    mut idx: c_int,
) -> *mut c_char {
    if idx == 0 as c_int {
        if !(*set_opt_callback_orig_option.ptr()).is_null() {
            return set_opt_callback_orig_option.get();
        } else {
            return b"\0".as_ptr() as *const c_char as *mut c_char;
        }
    }
    return (*set_opt_callback_func.ptr()).expect("non-null function pointer")(
        xp,
        idx - 1 as c_int,
    );
}

pub(crate) unsafe extern "C" fn expand_set_opt_generic(
    mut args: *mut optexpand_T,
    mut func: CompleteListItemGetter,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    set_opt_callback_orig_option.set(if (*args).oe_include_orig_val as c_int != 0 {
        (*args).oe_opt_value
    } else {
        ::core::ptr::null_mut::<c_char>()
    });
    set_opt_callback_func
        .set(func as Option<unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char>);
    ExpandGeneric(
        b"\0".as_ptr() as *const c_char,
        (*args).oe_xp,
        (*args).oe_regmatch,
        matches,
        numMatches,
        Some(expand_set_opt_callback as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        false_0 != 0,
    );
    set_opt_callback_orig_option.set(::core::ptr::null_mut::<c_char>());
    set_opt_callback_func.set(None);
    return OK;
}

pub(crate) unsafe extern "C" fn expand_set_opt_listflag(
    mut args: *mut optexpand_T,
    mut flags: *mut c_char,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut option_val: *mut c_char = (*args).oe_opt_value;
    let mut cmdline_val: *mut c_char = (*args).oe_set_arg;
    let mut append: bool = (*args).oe_append;
    let mut include_orig_val: bool =
        (*args).oe_include_orig_val as c_int != 0 && *option_val as c_int != NUL;
    let mut num_flags: size_t = strlen(flags);
    *matches = xmalloc(
        ::core::mem::size_of::<*mut c_char>().wrapping_mul(num_flags.wrapping_add(1 as size_t)),
    ) as *mut *mut c_char;
    let mut count: c_int = 0 as c_int;
    if include_orig_val {
        let c2rust_fresh7 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh7 as isize);
        *c2rust_lvalue_ptr = xstrdup(option_val);
    }
    let mut flag: *mut c_char = flags;
    while *flag as c_int != NUL {
        if !(append as c_int != 0 && !vim_strchr(option_val, *flag as c_int).is_null()) {
            if vim_strchr(cmdline_val, *flag as c_int).is_null() {
                if !(include_orig_val as c_int != 0
                    && *option_val.offset(1 as c_int as isize) as c_int == NUL
                    && *flag as c_int == *option_val.offset(0 as c_int as isize) as c_int)
                {
                    let c2rust_fresh8 = count;
                    count = count + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh8 as isize);
                    *c2rust_lvalue_ptr_0 =
                        xmemdupz(flag as *const c_void, 1 as size_t) as *mut c_char;
                }
            }
        }
        flag = flag.offset(1);
    }
    if count == 0 as c_int {
        let mut ptr_: *mut *mut c_void = matches as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return FAIL;
    }
    *numMatches = count;
    return OK;
}

pub unsafe extern "C" fn expand_set_chars_option(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut varp: *mut *mut c_char = (*args).oe_varp as *mut *mut c_char;
    let mut is_lcs: bool =
        varp == p_lcs.ptr() || varp == &raw mut (*curwin.get()).w_onebuf_opt.wo_lcs;
    return expand_set_opt_generic(
        args,
        if is_lcs as c_int != 0 {
            Some(get_listchars_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char)
        } else {
            Some(get_fillchars_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char)
        },
        numMatches,
        matches,
    );
}

pub unsafe extern "C" fn expand_set_concealcursor(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, COCU_ALL.as_ptr() as *mut c_char, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_cpoptions(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, CPO_VI.as_ptr() as *mut c_char, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_diffopt(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut xp: *mut expand_T = (*args).oe_xp;
    if (*xp).xp_pattern > (*args).oe_set_arg
        && *(*xp).xp_pattern.offset(-(1 as c_int as isize)) as c_int == ':' as c_int
    {
        let algo_len: size_t = strlen(b"algorithm:\0".as_ptr() as *const c_char);
        if (*xp).xp_pattern.offset_from((*args).oe_set_arg) >= algo_len as c_int as isize
            && strncmp(
                (*xp).xp_pattern.offset(-(algo_len as isize)),
                b"algorithm:\0".as_ptr() as *const c_char,
                algo_len,
            ) == 0 as c_int
        {
            return expand_set_opt_string(
                args,
                opt_dip_algorithm_values.ptr() as *mut *const c_char,
                ::core::mem::size_of::<[*const c_char; 5]>()
                    .wrapping_div(::core::mem::size_of::<*const c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*const c_char; 5]>()
                            .wrapping_rem(::core::mem::size_of::<*const c_char>())
                            == 0) as c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t),
                numMatches,
                matches,
            );
        }
        let inline_len: size_t = strlen(b"inline:\0".as_ptr() as *const c_char);
        if (*xp).xp_pattern.offset_from((*args).oe_set_arg) >= inline_len as c_int as isize
            && strncmp(
                (*xp).xp_pattern.offset(-(inline_len as isize)),
                b"inline:\0".as_ptr() as *const c_char,
                inline_len,
            ) == 0 as c_int
        {
            return expand_set_opt_string(
                args,
                opt_dip_inline_values.ptr() as *mut *const c_char,
                ::core::mem::size_of::<[*const c_char; 5]>()
                    .wrapping_div(::core::mem::size_of::<*const c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*const c_char; 5]>()
                            .wrapping_rem(::core::mem::size_of::<*const c_char>())
                            == 0) as c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t),
                numMatches,
                matches,
            );
        }
        return FAIL;
    }
    return expand_set_str_generic(args, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_encoding(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_generic(
        args,
        Some(get_encoding_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        numMatches,
        matches,
    );
}

pub(crate) unsafe extern "C" fn get_eventignore_name(
    mut xp: *mut expand_T,
    mut idx: c_int,
) -> *mut c_char {
    let mut subtract: bool = *(*xp).xp_pattern as c_int == '-' as c_int;
    if !subtract && idx == 0 as c_int {
        return b"all\0".as_ptr() as *const c_char as *mut c_char;
    }
    let mut name: *mut c_char =
        get_event_name_no_group(xp, idx - 1 as c_int + subtract as c_int, expand_eiw.get());
    if name.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    snprintf(
        IObuff.ptr() as *mut c_char,
        IOSIZE as size_t,
        b"%s%s\0".as_ptr() as *const c_char,
        if subtract as c_int != 0 {
            b"-\0".as_ptr() as *const c_char
        } else {
            b"\0".as_ptr() as *const c_char
        },
        name,
    );
    return IObuff.ptr() as *mut c_char;
}

pub unsafe extern "C" fn expand_set_eventignore(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    expand_eiw.set((*args).oe_varp != p_ei.ptr() as *mut c_char);
    return expand_set_opt_generic(
        args,
        Some(get_eventignore_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        numMatches,
        matches,
    );
}

pub unsafe extern "C" fn get_fileformat_name(
    mut _xp: *mut expand_T,
    mut idx: c_int,
) -> *mut c_char {
    if idx
        >= ::core::mem::size_of::<[*const c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*const c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*const c_char>())
                    == 0) as c_int as usize,
            ) as c_int
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    return (*opt_ff_values.ptr())[idx as usize] as *mut c_char;
}

pub unsafe extern "C" fn expand_set_formatoptions(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, FO_ALL.as_ptr() as *mut c_char, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_mouse(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, MOUSE_ALL.as_ptr() as *mut c_char, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_shortmess(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, SHM_ALL.ptr() as *mut c_char, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_whichwrap(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, WW_ALL.as_ptr() as *mut c_char, numMatches, matches);
}

pub unsafe extern "C" fn expand_set_winhighlight(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_generic(
        args,
        Some(get_highlight_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        numMatches,
        matches,
    );
}
