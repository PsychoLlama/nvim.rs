//! The `:set` family: parsing one argument into an option, a prefix, an
//! operator and a new value.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ex_set(mut eap: *mut exarg_T) {
    let mut flags: c_int = 0 as c_int;
    if (*eap).cmdidx as c_int == CMD_setlocal as c_int {
        flags = OPT_LOCAL as c_int;
    } else if (*eap).cmdidx as c_int == CMD_setglobal as c_int {
        flags = OPT_GLOBAL as c_int;
    }
    if (*eap).forceit != 0 {
        flags |= OPT_ONECOLUMN as c_int;
    }
    do_set((*eap).arg, flags);
}

pub(crate) unsafe extern "C" fn get_op(mut arg: *const c_char) -> set_op_T {
    let mut op: set_op_T = OP_NONE;
    if *arg as c_int != NUL && *arg.offset(1 as c_int as isize) as c_int == '=' as c_int {
        if *arg as c_int == '+' as c_int {
            op = OP_ADDING;
        } else if *arg as c_int == '^' as c_int {
            op = OP_PREPENDING;
        } else if *arg as c_int == '-' as c_int {
            op = OP_REMOVING;
        }
    }
    return op;
}

pub(crate) unsafe extern "C" fn get_option_prefix(mut argp: *mut *mut c_char) -> set_prefix_T {
    if strncmp(*argp, b"no\0".as_ptr() as *const c_char, 2 as size_t) == 0 as c_int {
        *argp = (*argp).offset(2 as c_int as isize);
        return PREFIX_NO;
    } else if strncmp(*argp, b"inv\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        *argp = (*argp).offset(3 as c_int as isize);
        return PREFIX_INV;
    }
    return PREFIX_NONE;
}

pub(crate) unsafe extern "C" fn validate_opt_idx(
    mut win: *mut win_T,
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut flags: uint32_t,
    mut prefix: set_prefix_T,
    mut errmsg: *mut *const c_char,
) -> c_int {
    if !option_has_type(opt_idx, kOptValTypeBoolean)
        && prefix as c_uint != PREFIX_NONE as c_int as c_uint
    {
        *errmsg = &raw const e_invarg as *const c_char;
        return FAIL;
    }
    if opt_flags & OPT_WINONLY as c_int != 0 && !option_is_window_local(opt_idx) {
        return FAIL;
    }
    if opt_flags & OPT_NOWIN as c_int != 0 && option_is_window_local(opt_idx) as c_int != 0 {
        return FAIL;
    }
    if opt_flags & OPT_MODELINE as c_int != 0 {
        if flags & kOptFlagSecure as c_int as uint32_t != 0 {
            *errmsg = (e_not_allowed_in_modeline.ptr() as *const _) as *const c_char;
            return FAIL;
        }
        if flags & kOptFlagMLE as c_int as uint32_t != 0 && p_mle.get() == 0 {
            *errmsg = (e_not_allowed_in_modeline_when_modelineexpr_is_off.ptr() as *const _)
                as *const c_char;
            return FAIL;
        }
        if (*win).w_onebuf_opt.wo_diff != 0
            && (opt_idx as c_int == kOptFoldmethod as c_int
                || opt_idx as c_int == kOptWrap as c_int)
        {
            return FAIL;
        }
    }
    if sandbox.get() != 0 as c_int && flags & kOptFlagSecure as c_int as uint32_t != 0 {
        *errmsg = &raw const e_sandbox as *const c_char;
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn find_tty_option_end(mut arg: *const c_char) -> *const c_char {
    if strequal(arg, b"term\0".as_ptr() as *const c_char) {
        return arg
            .offset(::core::mem::size_of::<[c_char; 5]>() as isize)
            .offset(-(1 as c_int as isize));
    } else if strequal(arg, b"ttytype\0".as_ptr() as *const c_char) {
        return arg
            .offset(::core::mem::size_of::<[c_char; 8]>() as isize)
            .offset(-(1 as c_int as isize));
    }
    let mut p: *const c_char = arg;
    let mut delimit: bool = false_0 != 0;
    if *arg.offset(0 as c_int as isize) as c_int == '<' as c_int {
        delimit = true_0 != 0;
        p = p.offset(1);
    }
    if *p.offset(0 as c_int as isize) as c_int == 't' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '_' as c_int
        && *p.offset(2 as c_int as isize) as c_int != 0
        && *p.offset(3 as c_int as isize) as c_int != 0
    {
        p = p.offset(4 as c_int as isize);
    } else if delimit {
        while *p as c_int != NUL && *p as c_int != '>' as c_int {
            p = p.offset(1);
        }
    }
    if delimit {
        if *p as c_int != '>' as c_int {
            return ::core::ptr::null::<c_char>();
        }
        p = p.offset(1);
    }
    return if arg == p {
        ::core::ptr::null::<c_char>()
    } else {
        p
    };
}

pub unsafe extern "C" fn find_option_end(
    mut arg: *const c_char,
    mut opt_idxp: *mut OptIndex,
) -> *const c_char {
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = find_tty_option_end(arg);
    if !p.is_null() {
        *opt_idxp = kOptInvalid;
        return p;
    } else {
        p = arg;
    }
    if !(*p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint)
    {
        *opt_idxp = kOptInvalid;
        return ::core::ptr::null::<c_char>();
    }
    while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
    {
        p = p.offset(1);
    }
    *opt_idxp = find_option_len(arg, p.offset_from(arg) as size_t);
    return p;
}

pub(crate) unsafe extern "C" fn get_option_newval(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut prefix: set_prefix_T,
    mut argp: *mut *mut c_char,
    mut nextchar: c_int,
    mut op: set_op_T,
    mut flags: uint32_t,
    mut varp: *mut c_void,
    mut _errbuf: *mut c_char,
    _errbuflen: size_t,
    mut errmsg: *mut *const c_char,
) -> OptVal {
    '_c2rust_label: {
        if !varp.is_null() {
        } else {
            __assert_fail(
                b"varp != NULL\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr()
                    as *const c_char,
                1322 as c_uint,
                b"OptVal get_option_newval(OptIndex, int, set_prefix_T, char **, int, set_op_T, uint32_t, void *, char *, const size_t, const char **)\0"
                    .as_ptr() as *const c_char,
            );
        }
    };
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut arg: *mut c_char = *argp;
    let oldval_is_global: bool =
        option_is_global_local(opt_idx) as c_int != 0 && opt_flags & OPT_LOCAL as c_int != 0;
    let mut oldval: OptVal = optval_from_varp(
        opt_idx,
        if oldval_is_global as c_int != 0 {
            get_varp(opt)
        } else {
            varp
        },
    );
    let mut newval: OptVal = OptVal {
        type_0: kOptValTypeNil,
        data: OptValData { boolean: kFalse },
    };
    if nextchar == '&' as c_int {
        return optval_copy(get_option_default(opt_idx, OPT_GLOBAL as c_int));
    } else if nextchar == '<' as c_int {
        if option_is_global_local(opt_idx) as c_int != 0 && opt_flags & OPT_LOCAL as c_int == 0 {
            unset_option_local_value(opt_idx);
        }
        return get_option_value(opt_idx, OPT_GLOBAL as c_int);
    }
    match oldval.type_0 as c_int {
        -1 => {
            abort();
        }
        0 => {
            let mut newval_bool: TriState = kFalse;
            if nextchar == '!' as c_int {
                match oldval.data.boolean as c_int {
                    -1 => {
                        newval_bool = kNone;
                    }
                    1 => {
                        newval_bool = kFalse;
                    }
                    0 => {
                        newval_bool = kTrue;
                    }
                    _ => {}
                }
            } else if prefix as c_uint == PREFIX_INV as c_int as c_uint {
                newval_bool = (*(varp as *mut c_int) ^ 1 as c_int) as TriState;
            } else {
                newval_bool = (if prefix as c_uint == PREFIX_NO as c_int as c_uint {
                    0 as c_int
                } else {
                    1 as c_int
                }) as TriState;
            }
            newval = OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: newval_bool,
                },
            };
        }
        1 => {
            let mut oldval_num: OptInt = oldval.data.number;
            let mut newval_num: OptInt = 0;
            arg = arg.offset(1);
            if (varp as *mut OptInt == p_wc.ptr() || varp as *mut OptInt == p_wcm.ptr())
                && (*arg as c_int == '<' as c_int
                    || *arg as c_int == '^' as c_int
                    || *arg as c_int != NUL
                        && (*arg.offset(1 as c_int as isize) == 0
                            || ascii_iswhite(*arg.offset(1 as c_int as isize) as c_int) as c_int
                                != 0)
                        && !ascii_isdigit(*arg as c_int))
            {
                newval_num = string_to_key(arg) as OptInt;
                if newval_num == 0 as OptInt {
                    *errmsg = &raw const e_invarg as *const c_char;
                    return newval;
                }
            } else if *arg as c_int == '-' as c_int || ascii_isdigit(*arg as c_int) as c_int != 0 {
                let mut i: c_int = 0;
                vim_str2nr(
                    arg,
                    ::core::ptr::null_mut::<c_int>(),
                    &raw mut i,
                    STR2NR_ALL as c_int,
                    &raw mut newval_num,
                    ::core::ptr::null_mut::<uvarnumber_T>(),
                    0 as c_int,
                    true_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if i == 0 as c_int
                    || *arg.offset(i as isize) as c_int != NUL
                        && !ascii_iswhite(*arg.offset(i as isize) as c_int)
                {
                    *errmsg = (e_number_required_after_equal.ptr() as *const _) as *const c_char;
                    return newval;
                }
            } else {
                *errmsg = (e_number_required_after_equal.ptr() as *const _) as *const c_char;
                return newval;
            }
            if op as c_uint == OP_ADDING as c_int as c_uint {
                newval_num = oldval_num + newval_num;
            }
            if op as c_uint == OP_PREPENDING as c_int as c_uint {
                newval_num = oldval_num * newval_num;
            }
            if op as c_uint == OP_REMOVING as c_int as c_uint {
                newval_num = oldval_num - newval_num;
            }
            newval = OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData { number: newval_num },
            };
        }
        2 => {
            let mut oldval_str: *const c_char = oldval.data.string.data;
            let mut newval_str: *const c_char = stropt_get_newval(
                nextchar,
                opt_idx,
                argp,
                varp,
                oldval_str,
                &raw mut op,
                flags,
            );
            newval = OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(newval_str),
                },
            };
        }
        _ => {}
    }
    return newval;
}

pub(crate) unsafe extern "C" fn do_one_set_option(
    mut opt_flags: c_int,
    mut argp: *mut *mut c_char,
    mut did_show: *mut bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut errmsg: *mut *const c_char,
) {
    let mut prefix: set_prefix_T = get_option_prefix(argp);
    let mut arg: *mut c_char = *argp;
    let mut opt_idx: OptIndex = kOptAleph;
    let option_end: *const c_char = find_option_end(arg, &raw mut opt_idx);
    if opt_idx as c_int != kOptInvalid as c_int {
        '_c2rust_label: {
            if option_end >= arg as *const c_char {
            } else {
                __assert_fail(
                    b"option_end >= arg\0".as_ptr() as *const c_char,
                    b"src/nvim/option.rs\0".as_ptr()
                        as *const c_char,
                    1448 as c_uint,
                    b"void do_one_set_option(int, char **, _Bool *, char *, size_t, const char **)\0"
                        .as_ptr() as *const c_char,
                );
            }
        };
    } else if is_tty_option(arg) {
        return;
    } else {
        *errmsg = (e_unknown_option.ptr() as *const _) as *const c_char;
        return;
    }
    let mut afterchar: uint8_t = *option_end as uint8_t;
    let mut p: *mut c_char = option_end as *mut c_char;
    while ascii_iswhite(*p as c_int) {
        p = p.offset(1);
    }
    let mut op: set_op_T = get_op(p);
    if op as c_uint != OP_NONE as c_int as c_uint {
        p = p.offset(1);
    }
    let mut nextchar: uint8_t = *p as uint8_t;
    let mut flags: uint32_t = (*options.ptr())[opt_idx as usize].flags;
    let mut varp: *mut c_void = get_varp_scope(
        (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
        opt_flags,
    );
    if validate_opt_idx(curwin.get(), opt_idx, opt_flags, flags, prefix, errmsg) == FAIL {
        return;
    }
    if !vim_strchr(b"?=:!&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null() {
        *argp = p;
        if nextchar as c_int == '&' as c_int
            && *(*argp).offset(1 as c_int as isize) as c_int == 'v' as c_int
            && *(*argp).offset(2 as c_int as isize) as c_int == 'i' as c_int
        {
            if *(*argp).offset(3 as c_int as isize) as c_int == 'm' as c_int {
                *argp = (*argp).offset(3 as c_int as isize);
            } else {
                *argp = (*argp).offset(2 as c_int as isize);
            }
        }
        if !vim_strchr(b"?!&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null()
            && *(*argp).offset(1 as c_int as isize) as c_int != NUL
            && !ascii_iswhite(*(*argp).offset(1 as c_int as isize) as c_int)
        {
            *errmsg = &raw const e_trailing as *const c_char;
            return;
        }
    }
    if nextchar as c_int == '?' as c_int
        || prefix as c_uint == PREFIX_NONE as c_int as c_uint
            && vim_strchr(b"=:&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null()
            && !option_has_type(opt_idx, kOptValTypeBoolean)
    {
        if *did_show {
            msg_putchar('\n' as c_int);
        } else {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
            gotocmdline(true_0 != 0);
            *did_show = true_0 != 0;
        }
        showoneopt(
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
            opt_flags,
        );
        if p_verbose.get() > 0 as OptInt {
            if varp == (*options.ptr())[opt_idx as usize].var {
                last_set_msg((*options.ptr())[opt_idx as usize].script_ctx);
            } else if option_has_scope(opt_idx, kOptScopeWin) {
                last_set_msg(
                    (*curwin.get()).w_onebuf_opt.wo_script_ctx
                        [option_scope_idx(opt_idx, kOptScopeWin) as usize],
                );
            } else if option_has_scope(opt_idx, kOptScopeBuf) {
                last_set_msg(
                    (*curbuf.get()).b_p_script_ctx
                        [option_scope_idx(opt_idx, kOptScopeBuf) as usize],
                );
            }
        }
        if nextchar as c_int != '?' as c_int
            && nextchar as c_int != NUL
            && !ascii_iswhite(afterchar as c_int)
        {
            *errmsg = &raw const e_trailing as *const c_char;
        }
        return;
    }
    if option_has_type(opt_idx, kOptValTypeBoolean) {
        if !vim_strchr(b"=:\0".as_ptr() as *const c_char, nextchar as c_int).is_null() {
            *errmsg = &raw const e_invarg as *const c_char;
            return;
        }
        if vim_strchr(b"!&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null()
            && nextchar as c_int != NUL
            && !ascii_iswhite(afterchar as c_int)
        {
            *errmsg = &raw const e_trailing as *const c_char;
            return;
        }
    } else if vim_strchr(b"=:&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null() {
        *errmsg = &raw const e_invarg as *const c_char;
        return;
    }
    let mut newval: OptVal = get_option_newval(
        opt_idx,
        opt_flags,
        prefix,
        argp,
        nextchar as c_int,
        op,
        flags,
        varp,
        errbuf,
        errbuflen,
        errmsg,
    );
    if newval.type_0 as c_int == kOptValTypeNil as c_int || !(*errmsg).is_null() {
        return;
    }
    *errmsg = set_option(
        opt_idx,
        newval,
        opt_flags,
        0 as scid_T,
        false_0 != 0,
        op as c_uint == OP_NONE as c_int as c_uint,
        errbuf,
        errbuflen,
    );
}

pub unsafe extern "C" fn do_set(mut arg: *mut c_char, mut opt_flags: c_int) -> c_int {
    let mut did_show: bool = false_0 != 0;
    if *arg as c_int == NUL {
        showoptions(false_0 != 0, opt_flags);
        did_show = true_0 != 0;
    } else {
        while *arg as c_int != NUL {
            if strncmp(arg, b"all\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int
                && !(*arg.offset(3 as c_int as isize) as c_uint >= 'A' as c_uint
                    && *arg.offset(3 as c_int as isize) as c_uint <= 'Z' as c_uint
                    || *arg.offset(3 as c_int as isize) as c_uint >= 'a' as c_uint
                        && *arg.offset(3 as c_int as isize) as c_uint <= 'z' as c_uint)
                && opt_flags & OPT_MODELINE as c_int == 0
            {
                arg = arg.offset(3 as c_int as isize);
                if *arg as c_int == '&' as c_int {
                    arg = arg.offset(1);
                    set_options_default(opt_flags);
                    didset_options();
                    didset_options2();
                    ui_refresh_options();
                    redraw_all_later(UPD_CLEAR as c_int);
                } else {
                    showoptions(true_0 != 0, opt_flags);
                    did_show = true_0 != 0;
                }
            } else {
                let mut startarg: *mut c_char = arg;
                let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
                let mut errbuf: [c_char; 80] = [0; 80];
                do_one_set_option(
                    opt_flags,
                    &raw mut arg,
                    &raw mut did_show,
                    &raw mut errbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 80]>(),
                    &raw mut errmsg,
                );
                let mut i: c_int = 0 as c_int;
                while i < 2 as c_int {
                    arg = skiptowhite_esc(arg);
                    arg = skipwhite(arg);
                    if *arg as c_int != '=' as c_int {
                        break;
                    }
                    i += 1;
                }
                if !errmsg.is_null() {
                    let mut i_0: c_int = vim_snprintf(
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        b"%s\0".as_ptr() as *const c_char,
                        gettext(errmsg),
                    ) + 2 as c_int;
                    if i_0 as isize + arg.offset_from(startarg) < IOSIZE as isize {
                        xstrlcpy(
                            (IObuff.ptr() as *mut c_char)
                                .offset(i_0 as isize)
                                .offset(-(2 as c_int as isize)),
                            b": \0".as_ptr() as *const c_char,
                            (IOSIZE - i_0 + 2 as c_int) as size_t,
                        );
                        '_c2rust_label: {
                            if arg >= startarg {
                            } else {
                                __assert_fail(
                                    b"arg >= startarg\0".as_ptr() as *const c_char,
                                    b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                                    1620 as c_uint,
                                    b"int do_set(char *, int)\0".as_ptr() as *const c_char,
                                );
                            }
                        };
                        memmove(
                            (IObuff.ptr() as *mut c_char).offset(i_0 as isize) as *mut c_void,
                            startarg as *const c_void,
                            arg.offset_from(startarg) as size_t,
                        );
                        (*IObuff.ptr())[(i_0 as isize + arg.offset_from(startarg)) as usize] =
                            NUL as c_char;
                    }
                    trans_characters(IObuff.ptr() as *mut c_char, IOSIZE);
                    (*no_wait_return.ptr()) += 1;
                    emsg(IObuff.ptr() as *mut c_char);
                    (*no_wait_return.ptr()) -= 1;
                    return FAIL;
                }
            }
            arg = skipwhite(arg);
        }
    }
    if silent_mode.get() as c_int != 0 && did_show as c_int != 0 {
        silent_mode.set(false_0 != 0);
        info_message.set(true_0 != 0);
        msg_putchar('\n' as c_int);
        silent_mode.set(true_0 != 0);
        info_message.set(false_0 != 0);
    }
    return OK;
}

pub(crate) unsafe extern "C" fn find_key_len(
    mut arg_arg: *const c_char,
    mut len: size_t,
    mut has_lt: bool,
) -> c_int {
    let mut key: c_int = 0 as c_int;
    let mut arg: *const c_char = arg_arg;
    if len >= 4 as size_t
        && *arg.offset(0 as c_int as isize) as c_int == 't' as c_int
        && *arg.offset(1 as c_int as isize) as c_int == '_' as c_int
    {
        if !has_lt || *arg.offset(4 as c_int as isize) as c_int == '>' as c_int {
            key = -(*arg.offset(2 as c_int as isize) as uint8_t as c_int
                + ((*arg.offset(3 as c_int as isize) as uint8_t as c_int) << 8 as c_int));
        }
    } else if has_lt {
        arg = arg.offset(-1);
        let mut modifiers: c_int = 0 as c_int;
        key = find_special_key(
            &raw mut arg,
            len.wrapping_add(1 as size_t),
            &raw mut modifiers,
            FSK_KEYCODE as c_int | FSK_KEEP_X_KEY as c_int | FSK_SIMPLIFY as c_int,
            ::core::ptr::null_mut::<bool>(),
        );
        if modifiers != 0 {
            key = 0 as c_int;
        }
    }
    return key;
}

pub unsafe extern "C" fn string_to_key(mut arg: *mut c_char) -> c_int {
    if *arg as c_int == '<' as c_int && *arg.offset(1 as c_int as isize) as c_int != 0 {
        return find_key_len(arg.offset(1 as c_int as isize), strlen(arg), true_0 != 0);
    }
    if *arg as c_int == '^' as c_int && *arg.offset(1 as c_int as isize) as c_int != 0 {
        let mut key: c_int = (if (*arg.offset(1 as c_int as isize) as uint8_t as c_int)
            < 'a' as c_int
            || *arg.offset(1 as c_int as isize) as uint8_t as c_int > 'z' as c_int
        {
            *arg.offset(1 as c_int as isize) as uint8_t as c_int
        } else {
            *arg.offset(1 as c_int as isize) as uint8_t as c_int - ('a' as c_int - 'A' as c_int)
        }) ^ 0x40 as c_int;
        if key == 0 as c_int {
            key = K_ZERO;
        }
        return key;
    }
    return *arg as uint8_t as c_int;
}
