//! Command-line completion of option names and values.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn option_expand(
    mut opt_idx: OptIndex,
    mut val: *const c_char,
) -> *mut c_char {
    if (*options.ptr())[opt_idx as usize].flags & kOptFlagExpand as c_int as uint32_t == 0
        || is_option_hidden(opt_idx) as c_int != 0
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    if val.is_null() {
        val = *((*options.ptr())[opt_idx as usize].var as *mut *mut c_char);
    }
    if val.is_null() || strlen(val) > MAXPATHL as size_t {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut var: *mut *mut c_char = (*options.ptr())[opt_idx as usize].var as *mut *mut c_char;
    let mut esc: bool = var == p_tags.ptr() || var == p_path.ptr();
    expand_env_esc(
        val,
        NameBuff.ptr() as *mut c_char,
        MAXPATHL,
        esc,
        false_0 != 0,
        (if (*options.ptr())[opt_idx as usize].var as *mut *mut c_char == p_sps.ptr() {
            b"file:\0".as_ptr() as *const c_char
        } else {
            ::core::ptr::null::<c_char>()
        }) as *mut c_char,
    );
    if strcmp(NameBuff.ptr() as *mut c_char, val) == 0 as c_int {
        return ::core::ptr::null_mut::<c_char>();
    }
    return NameBuff.ptr() as *mut c_char;
}

pub unsafe extern "C" fn set_context_in_set_cmd(
    mut xp: *mut expand_T,
    mut arg: *mut c_char,
    mut opt_flags: c_int,
) {
    expand_option_flags.set(opt_flags);
    (*xp).xp_context = EXPAND_SETTINGS as c_int;
    if *arg as c_int == NUL {
        (*xp).xp_pattern = arg;
        return;
    }
    let argend: *mut c_char = arg.offset(strlen(arg) as isize);
    let mut p: *mut c_char = argend.offset(-(1 as c_int as isize));
    if *p as c_int == ' ' as c_int && *p.offset(-(1 as c_int as isize)) as c_int != '\\' as c_int {
        (*xp).xp_pattern = p.offset(1 as c_int as isize);
        return;
    }
    while p > arg {
        let mut s: *mut c_char = p;
        if *p as c_int == ' ' as c_int || *p as c_int == ',' as c_int {
            while s > arg && *s.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int {
                s = s.offset(-1);
            }
        }
        if *p as c_int == ' ' as c_int && p.offset_from(s) & 1 as isize == 0 as isize {
            p = p.offset(1);
            break;
        } else {
            p = p.offset(-1);
        }
    }
    if strncmp(p, b"no\0".as_ptr() as *const c_char, 2 as size_t) == 0 as c_int {
        (*xp).xp_context = EXPAND_BOOL_SETTINGS as c_int;
        (*xp).xp_prefix = XP_PREFIX_NO;
        p = p.offset(2 as c_int as isize);
    } else if strncmp(p, b"inv\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        (*xp).xp_context = EXPAND_BOOL_SETTINGS as c_int;
        (*xp).xp_prefix = XP_PREFIX_INV;
        p = p.offset(3 as c_int as isize);
    }
    (*xp).xp_pattern = p;
    arg = p;
    let mut nextchar: c_char = 0;
    let mut flags: uint32_t = 0 as uint32_t;
    let mut opt_idx: OptIndex = kOptAleph;
    let mut is_term_option: bool = false_0 != 0;
    if *arg as c_int == '<' as c_int {
        while *p as c_int != '>' as c_int {
            let c2rust_fresh10 = p;
            p = p.offset(1);
            if *c2rust_fresh10 as c_int == NUL {
                return;
            }
        }
        let mut key: c_int = get_special_key_code(arg.offset(1 as c_int as isize));
        if key == 0 as c_int {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return;
        }
        p = p.offset(1);
        nextchar = *p;
        is_term_option = true_0 != 0;
        (*expand_option_name.ptr())[2 as c_int as usize] =
            (-key & 0xff as c_int) as uint8_t as c_char;
        (*expand_option_name.ptr())[3 as c_int as usize] =
            (-key as c_uint >> 8 as c_int & 0xff as c_uint) as uint8_t as c_char;
    } else if *p.offset(0 as c_int as isize) as c_int == 't' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '_' as c_int
    {
        p = p.offset(2 as c_int as isize);
        if *p as c_int != NUL {
            p = p.offset(1);
        }
        if *p as c_int == NUL {
            return;
        }
        p = p.offset(1);
        nextchar = *p;
        is_term_option = true_0 != 0;
        (*expand_option_name.ptr())[2 as c_int as usize] = *p.offset(-2 as c_int as isize);
        (*expand_option_name.ptr())[3 as c_int as usize] = *p.offset(-1 as c_int as isize);
    } else {
        while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
            || ascii_isdigit(*p as c_int) as c_int != 0
            || *p as c_int == '_' as c_int
            || *p as c_int == '*' as c_int
        {
            p = p.offset(1);
        }
        if *p as c_int == NUL {
            return;
        }
        nextchar = *p;
        opt_idx = find_option_len(arg, p.offset_from(arg) as size_t);
        if opt_idx as c_int == kOptInvalid as c_int || is_option_hidden(opt_idx) as c_int != 0 {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return;
        }
        flags = (*options.ptr())[opt_idx as usize].flags;
        if option_has_type(opt_idx, kOptValTypeBoolean) {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return;
        }
    }
    expand_option_append.set(false_0 != 0);
    let mut expand_option_subtract: bool = false_0 != 0;
    if (nextchar as c_int == '-' as c_int
        || nextchar as c_int == '+' as c_int
        || nextchar as c_int == '^' as c_int)
        && *p.offset(1 as c_int as isize) as c_int == '=' as c_int
    {
        if nextchar as c_int == '-' as c_int {
            expand_option_subtract = true_0 != 0;
        }
        if nextchar as c_int == '+' as c_int || nextchar as c_int == '^' as c_int {
            expand_option_append.set(true_0 != 0);
        }
        p = p.offset(1);
        nextchar = '=' as c_char;
    }
    if nextchar as c_int != '=' as c_int && nextchar as c_int != ':' as c_int
        || (*xp).xp_context == EXPAND_BOOL_SETTINGS as c_int
    {
        (*xp).xp_context = EXPAND_UNSUCCESSFUL as c_int;
        return;
    }
    if is_term_option {
        expand_option_idx.set(kOptInvalid);
    } else {
        expand_option_idx.set(opt_idx);
    }
    (*xp).xp_pattern = p.offset(1 as c_int as isize);
    expand_option_start_col.set(p.offset(1 as c_int as isize).offset_from((*xp).xp_line) as c_int);
    if (*options.ptr())[opt_idx as usize].var == p_syn.ptr() as *mut c_void {
        (*xp).xp_context = EXPAND_OWNSYNTAX as c_int;
        return;
    }
    if (*options.ptr())[opt_idx as usize].var == p_ft.ptr() as *mut c_void {
        (*xp).xp_context = EXPAND_FILETYPE as c_int;
        return;
    }
    if (*options.ptr())[opt_idx as usize].var == p_keymap.ptr() as *mut c_void {
        (*xp).xp_context = EXPAND_KEYMAP as c_int;
        return;
    }
    if expand_option_subtract {
        (*xp).xp_context = EXPAND_SETTING_SUBTRACT as c_int;
        return;
    } else if expand_option_idx.get() as c_int != kOptInvalid as c_int
        && (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .is_some()
    {
        (*xp).xp_context = EXPAND_STRING_SETTING as c_int;
    } else if *(*xp).xp_pattern as c_int == NUL {
        (*xp).xp_context = EXPAND_OLD_SETTING as c_int;
        return;
    } else {
        (*xp).xp_context = EXPAND_NOTHING as c_int;
    }
    if is_term_option as c_int != 0 || option_has_type(opt_idx, kOptValTypeNumber) as c_int != 0 {
        return;
    }
    if flags & kOptFlagExpand as c_int as uint32_t != 0 {
        p = (*options.ptr())[opt_idx as usize].var as *mut c_char;
        if p == p_bdir.ptr() as *mut c_char
            || p == p_dir.ptr() as *mut c_char
            || p == p_path.ptr() as *mut c_char
            || p == p_pp.ptr() as *mut c_char
            || p == p_rtp.ptr() as *mut c_char
            || p == p_cdpath.ptr() as *mut c_char
            || p == p_vdir.ptr() as *mut c_char
        {
            (*xp).xp_context = EXPAND_DIRECTORIES as c_int;
            if p == p_path.ptr() as *mut c_char || p == p_cdpath.ptr() as *mut c_char {
                (*xp).xp_backslash = XP_BS_THREE as c_int;
            } else {
                (*xp).xp_backslash = XP_BS_ONE as c_int;
            }
        } else {
            (*xp).xp_context = EXPAND_FILES as c_int;
            if p == p_tags.ptr() as *mut c_char {
                (*xp).xp_backslash = XP_BS_THREE as c_int;
            } else {
                (*xp).xp_backslash = XP_BS_ONE as c_int;
            }
        }
        if flags & kOptFlagComma as c_int as uint32_t != 0 {
            (*xp).xp_backslash |= XP_BS_COMMA as c_int;
        }
    }
    if flags & kOptFlagExpand as c_int as uint32_t != 0
        || flags & kOptFlagComma as c_int as uint32_t != 0
        || flags & kOptFlagColon as c_int as uint32_t != 0
    {
        p = argend.offset(-(1 as c_int as isize));
        while p > (*xp).xp_pattern {
            if *p as c_int == ' ' as c_int
                || *p as c_int == ',' as c_int
                || *p as c_int == ':' as c_int && flags & kOptFlagColon as c_int as uint32_t != 0
            {
                let mut s_0: *mut c_char = p;
                while s_0 > (*xp).xp_pattern
                    && *s_0.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int
                {
                    s_0 = s_0.offset(-1);
                }
                if *p as c_int == ' ' as c_int
                    && ((*xp).xp_backslash & XP_BS_THREE as c_int != 0
                        && p.offset_from(s_0) < 3 as isize)
                    || *p as c_int == ',' as c_int
                        && flags & kOptFlagComma as c_int as uint32_t != 0
                        && p.offset_from(s_0) < 2 as isize
                    || *p as c_int == ':' as c_int
                        && flags & kOptFlagColon as c_int as uint32_t != 0
                {
                    (*xp).xp_pattern = p.offset(1 as c_int as isize);
                    break;
                }
            }
            p = p.offset(-1);
        }
    }
    if flags & kOptFlagFlagList as c_int as uint32_t != 0 {
        (*xp).xp_pattern = argend;
    }
    if (*options.ptr())[opt_idx as usize].var == p_sps.ptr() as *mut c_void {
        if strncmp(
            (*xp).xp_pattern,
            b"file:\0".as_ptr() as *const c_char,
            5 as size_t,
        ) == 0 as c_int
        {
            (*xp).xp_pattern = (*xp).xp_pattern.offset(5 as c_int as isize);
            return;
        } else if (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .is_some()
        {
            (*xp).xp_context = EXPAND_STRING_SETTING as c_int;
        }
    }
}

pub(crate) unsafe extern "C" fn match_str(
    str: *mut c_char,
    regmatch: *mut regmatch_T,
    matches: *mut *mut c_char,
    idx: c_int,
    test_only: bool,
    fuzzy: bool,
    fuzzystr: *const c_char,
    fuzmatch: *mut fuzmatch_str_T,
) -> bool {
    if !fuzzy {
        if vim_regexec(regmatch, str, 0 as colnr_T) {
            if !test_only {
                *matches.offset(idx as isize) = xstrdup(str);
            }
            return true_0 != 0;
        }
    } else {
        let score: c_int = fuzzy_match_str(str, fuzzystr);
        if score != FUZZY_SCORE_NONE as c_int {
            if !test_only {
                (*fuzmatch.offset(idx as isize)).idx = idx;
                (*fuzmatch.offset(idx as isize)).str = xstrdup(str);
                (*fuzmatch.offset(idx as isize)).score = score;
            }
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}

pub unsafe extern "C" fn ExpandSettings(
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut fuzzystr: *mut c_char,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
    can_fuzzy: bool,
) -> c_int {
    let mut num_normal: c_int = 0 as c_int;
    let mut count: c_int = 0 as c_int;
    static names: GlobalCell<[*mut c_char; 1]> =
        GlobalCell::new([b"all\0".as_ptr() as *const c_char as *mut c_char]);
    let mut ic: c_int = (*regmatch).rm_ic as c_int;
    let mut fuzmatch: *mut fuzmatch_str_T = ::core::ptr::null_mut::<fuzmatch_str_T>();
    let fuzzy: bool = can_fuzzy as c_int != 0 && cmdline_fuzzy_complete(fuzzystr) as c_int != 0;
    let mut loop_0: c_int = 0 as c_int;
    while loop_0 <= 1 as c_int {
        (*regmatch).rm_ic = ic != 0;
        if (*xp).xp_context != EXPAND_BOOL_SETTINGS as c_int {
            let mut match_0: c_int = 0 as c_int;
            while match_0
                < ::core::mem::size_of::<[*mut c_char; 1]>()
                    .wrapping_div(::core::mem::size_of::<*mut c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut c_char; 1]>()
                            .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                            == 0) as c_int as usize,
                    ) as c_int
            {
                if match_str(
                    (*names.ptr())[match_0 as usize] as *mut c_char,
                    regmatch,
                    *matches,
                    count,
                    loop_0 == 0 as c_int,
                    fuzzy,
                    fuzzystr,
                    fuzmatch,
                ) {
                    if loop_0 == 0 as c_int {
                        num_normal += 1;
                    } else {
                        count += 1;
                    }
                }
                match_0 += 1;
            }
        }
        let mut str: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut opt_idx: OptIndex = kOptAleph;
        while (opt_idx as c_int) < kOptCount {
            str = (*options.ptr())[opt_idx as usize].fullname;
            if !is_option_hidden(opt_idx) {
                if !((*xp).xp_context == EXPAND_BOOL_SETTINGS as c_int
                    && !option_has_type(opt_idx, kOptValTypeBoolean))
                {
                    if match_str(
                        str,
                        regmatch,
                        *matches,
                        count,
                        loop_0 == 0 as c_int,
                        fuzzy,
                        fuzzystr,
                        fuzmatch,
                    ) {
                        if loop_0 == 0 as c_int {
                            num_normal += 1;
                        } else {
                            count += 1;
                        }
                    } else if !fuzzy
                        && !(*options.ptr())[opt_idx as usize].shortname.is_null()
                        && vim_regexec(
                            regmatch,
                            (*options.ptr())[opt_idx as usize].shortname,
                            0 as colnr_T,
                        ) as c_int
                            != 0
                    {
                        if loop_0 == 0 as c_int {
                            num_normal += 1;
                        } else {
                            let c2rust_fresh11 = count;
                            count = count + 1;
                            let c2rust_lvalue_ptr =
                                &raw mut *(*matches).offset(c2rust_fresh11 as isize);
                            *c2rust_lvalue_ptr = xstrdup(str);
                        }
                    }
                }
            }
            opt_idx += 1;
        }
        if loop_0 == 0 as c_int {
            if num_normal > 0 as c_int {
                *numMatches = num_normal;
            } else {
                return OK;
            }
            if !fuzzy {
                *matches = xmalloc(
                    (*numMatches as size_t).wrapping_mul(::core::mem::size_of::<*mut c_char>()),
                ) as *mut *mut c_char;
            } else {
                fuzmatch = xmalloc(
                    (*numMatches as size_t).wrapping_mul(::core::mem::size_of::<fuzmatch_str_T>()),
                ) as *mut fuzmatch_str_T;
            }
        }
        loop_0 += 1;
    }
    if fuzzy {
        fuzzymatches_to_strmatches(fuzmatch, matches, count, false_0 != 0);
    }
    return OK;
}

pub(crate) unsafe extern "C" fn escape_option_str_cmdline(mut var: *mut c_char) -> *mut c_char {
    let mut buf: *mut c_char = vim_strsave_escaped(var, escape_chars.get());
    return buf;
}

pub unsafe extern "C" fn ExpandOldSetting(
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut var: *mut c_char = ::core::ptr::null_mut::<c_char>();
    *numMatches = 0 as c_int;
    *matches = xmalloc(::core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if expand_option_idx.get() as c_int == kOptInvalid as c_int {
        expand_option_idx.set(find_option(expand_option_name.ptr() as *mut c_char));
    }
    if expand_option_idx.get() as c_int != kOptInvalid as c_int {
        option_value2string(
            (options.ptr() as *mut vimoption_T).offset(expand_option_idx.get() as isize),
            expand_option_flags.get(),
        );
        var = NameBuff.ptr() as *mut c_char;
    } else {
        var = b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    let mut buf: *mut c_char = escape_option_str_cmdline(var);
    *(*matches).offset(0 as c_int as isize) = buf;
    *numMatches = 1 as c_int;
    return OK;
}

pub unsafe extern "C" fn ExpandStringSetting(
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    if expand_option_idx.get() as c_int == kOptInvalid as c_int
        || (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .is_none()
    {
        return FAIL;
    }
    let mut args: optexpand_T = optexpand_T {
        oe_varp: get_varp_scope(
            (options.ptr() as *mut vimoption_T).offset(expand_option_idx.get() as isize),
            expand_option_flags.get(),
        ) as *mut c_char,
        oe_idx: expand_option_idx.get(),
        oe_opt_value: ::core::ptr::null_mut::<c_char>(),
        oe_append: expand_option_append.get(),
        oe_include_orig_val: false,
        oe_regmatch: regmatch,
        oe_xp: xp,
        oe_set_arg: (*xp).xp_line.offset(expand_option_start_col.get() as isize),
    };
    args.oe_include_orig_val = !expand_option_append.get() && *args.oe_set_arg as c_int == NUL;
    option_value2string(
        (options.ptr() as *mut vimoption_T).offset(expand_option_idx.get() as isize),
        expand_option_flags.get(),
    );
    let mut var: *mut c_char = NameBuff.ptr() as *mut c_char;
    let mut buf: *mut c_char = escape_option_str_cmdline(var);
    args.oe_opt_value = buf;
    let mut num_ret: c_int =
        (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .expect("non-null function pointer")(&raw mut args, numMatches, matches);
    xfree(buf as *mut c_void);
    return num_ret;
}

pub unsafe extern "C" fn ExpandSettingSubtract(
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    if expand_option_idx.get() as c_int == kOptInvalid as c_int {
        return ExpandOldSetting(numMatches, matches);
    }
    let mut option_val: *mut c_char = *(get_option_varp_scope_from(
        expand_option_idx.get(),
        expand_option_flags.get(),
        curbuf.get(),
        curwin.get(),
    ) as *mut *mut c_char);
    let mut option_flags: uint32_t = (*options.ptr())[expand_option_idx.get() as usize].flags;
    if option_has_type(expand_option_idx.get(), kOptValTypeNumber) {
        return ExpandOldSetting(numMatches, matches);
    } else if option_flags & kOptFlagComma as c_int as uint32_t != 0 {
        if *option_val as c_int == NUL {
            return FAIL;
        }
        let mut option_copy: *mut c_char = xstrdup(option_val);
        let mut next_val: *mut c_char = option_copy;
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut c_char>() as c_int,
            10 as c_int,
        );
        loop {
            let mut item: *mut c_char = next_val;
            let mut comma: *mut c_char = vim_strchr(next_val, ',' as c_int);
            while !comma.is_null()
                && comma != next_val
                && *comma.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int
            {
                comma = vim_strchr(comma.offset(1 as c_int as isize), ',' as c_int);
            }
            if !comma.is_null() {
                *comma = NUL as c_char;
                next_val = comma.offset(1 as c_int as isize);
            } else {
                next_val = ::core::ptr::null_mut::<c_char>();
            }
            if *item as c_int != NUL {
                if vim_regexec(regmatch, item, 0 as colnr_T) {
                    let mut buf: *mut c_char = escape_option_str_cmdline(item);
                    ga_grow(&raw mut ga, 1 as c_int);
                    *(ga.ga_data as *mut *mut c_char).offset(ga.ga_len as isize) = buf;
                    ga.ga_len += 1;
                }
            }
            if next_val.is_null() {
                break;
            }
        }
        xfree(option_copy as *mut c_void);
        *matches = ga.ga_data as *mut *mut c_char;
        *numMatches = ga.ga_len;
        return OK;
    } else if option_flags & kOptFlagFlagList as c_int as uint32_t != 0 {
        if *(*xp).xp_pattern as c_int != NUL {
            return FAIL;
        }
        let mut num_flags: size_t = strlen(option_val);
        if num_flags == 0 as size_t {
            return FAIL;
        }
        *matches = xmalloc(
            ::core::mem::size_of::<*mut c_char>().wrapping_mul(num_flags.wrapping_add(1 as size_t)),
        ) as *mut *mut c_char;
        let mut count: c_int = 0 as c_int;
        let c2rust_fresh12 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh12 as isize);
        *c2rust_lvalue_ptr = xmemdupz(option_val as *const c_void, num_flags) as *mut c_char;
        if num_flags > 1 as size_t {
            let mut flag: *mut c_char = option_val;
            while *flag as c_int != NUL {
                let c2rust_fresh13 = count;
                count = count + 1;
                let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh13 as isize);
                *c2rust_lvalue_ptr_0 = xmemdupz(flag as *const c_void, 1 as size_t) as *mut c_char;
                flag = flag.offset(1);
            }
        }
        *numMatches = count;
        return OK;
    }
    return ExpandOldSetting(numMatches, matches);
}
