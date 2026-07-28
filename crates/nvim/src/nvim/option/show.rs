//! `:set` with no value, `:mkvimrc`, and the UI's option broadcast.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn showoptions(mut all: bool, mut opt_flags: c_int) {
    let mut items: *mut *mut vimoption_T =
        xmalloc(::core::mem::size_of::<*mut vimoption_T>().wrapping_mul(OPTION_COUNT))
            as *mut *mut vimoption_T;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
    if opt_flags & OPT_GLOBAL as c_int != 0 {
        msg_puts_title(gettext(
            b"\n--- Global option values ---\0".as_ptr() as *const c_char
        ));
    } else if opt_flags & OPT_LOCAL as c_int != 0 {
        msg_puts_title(gettext(
            b"\n--- Local option values ---\0".as_ptr() as *const c_char
        ));
    } else {
        msg_puts_title(gettext(b"\n--- Options ---\0".as_ptr() as *const c_char));
    }
    let mut run: c_int = 1 as c_int;
    while run <= 2 as c_int && !got_int.get() {
        let mut item_count: c_int = 0 as c_int;
        let mut opt: *mut vimoption_T = ::core::ptr::null_mut::<vimoption_T>();
        let mut opt_idx: OptIndex = kOptAleph;
        while (opt_idx as c_int) < kOptCount {
            opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
            if !message_filtered((*opt).fullname) {
                let mut varp: *mut c_void = NULL;
                if opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) != 0 as c_int {
                    if !option_is_global_only(opt_idx) {
                        varp = get_varp_scope(opt, opt_flags);
                    }
                } else {
                    varp = get_varp(opt);
                }
                if !varp.is_null() && (all as c_int != 0 || !optval_is_default(opt_idx, varp)) {
                    let mut len: c_int = 0;
                    if opt_flags & OPT_ONECOLUMN as c_int != 0 {
                        len = Columns.get();
                    } else if option_has_type(opt_idx, kOptValTypeBoolean) {
                        len = 1 as c_int;
                    } else {
                        option_value2string(opt, opt_flags);
                        len = strlen((*opt).fullname) as c_int
                            + vim_strsize(NameBuff.ptr() as *mut c_char)
                            + 1 as c_int;
                    }
                    if len <= INC - GAP && run == 1 as c_int || len > INC - GAP && run == 2 as c_int
                    {
                        let c2rust_fresh6 = item_count;
                        item_count = item_count + 1;
                        let c2rust_lvalue_ptr = &raw mut *items.offset(c2rust_fresh6 as isize);
                        *c2rust_lvalue_ptr = opt;
                    }
                }
            }
            opt_idx += 1;
        }
        let mut rows: c_int = 0;
        if run == 1 as c_int {
            '_c2rust_label: {
                if Columns.get() <= 2147483647 as c_int - 3 as c_int
                    && Columns.get() + 3 as c_int >= -2147483647 as c_int - 1 as c_int + 3 as c_int
                    && (Columns.get() + 3 as c_int - 3 as c_int) / 20 as c_int
                        >= -2147483647 as c_int - 1 as c_int
                    && (Columns.get() + 3 as c_int - 3 as c_int) / 20 as c_int
                        <= 2147483647 as c_int
                {
                } else {
                    __assert_fail(
                        b"Columns <= INT_MAX - GAP && Columns + GAP >= INT_MIN + 3 && (Columns + GAP - 3) / INC >= INT_MIN && (Columns + GAP - 3) / INC <= INT_MAX\0"
                            .as_ptr() as *const c_char,
                        b"src/nvim/option.rs\0"
                            .as_ptr() as *const c_char,
                        4288 as c_uint,
                        b"void showoptions(_Bool, int)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            let mut cols: c_int = (Columns.get() + GAP - 3 as c_int) / INC;
            if cols == 0 as c_int {
                cols = 1 as c_int;
            }
            rows = (item_count + cols - 1 as c_int) / cols;
        } else {
            rows = item_count;
        }
        let mut row: c_int = 0 as c_int;
        while row < rows && !got_int.get() {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                break;
            }
            let mut col: c_int = 0 as c_int;
            let mut i: c_int = row;
            while i < item_count {
                msg_advance(col);
                showoneopt(*items.offset(i as isize), opt_flags);
                col += INC;
                i += rows;
            }
            os_breakcheck();
            row += 1;
        }
        run += 1;
    }
    xfree(items as *mut c_void);
}

pub unsafe extern "C" fn ui_refresh_options() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut flags: uint32_t = (*options.ptr())[opt_idx as usize].flags;
        if flags & kOptFlagUIOption as c_int as uint32_t != 0 {
            let mut name: String_0 = cstr_as_string((*options.ptr())[opt_idx as usize].fullname);
            let mut value: Object = optval_as_object(optval_from_varp(
                opt_idx,
                (*options.ptr())[opt_idx as usize].var,
            ));
            ui_call_option_set(name, value);
        }
        opt_idx += 1;
    }
    if !(*p_mouse.ptr()).is_null() {
        setmouse();
    }
}

pub(crate) unsafe extern "C" fn showoneopt(mut opt: *mut vimoption_T, mut opt_flags: c_int) {
    let mut save_silent: c_int = silent_mode.get() as c_int;
    silent_mode.set(false_0 != 0);
    info_message.set(true_0 != 0);
    let mut opt_idx: OptIndex = get_opt_idx(opt);
    let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
    if option_has_type(opt_idx, kOptValTypeBoolean) as c_int != 0
        && (if varp as *mut c_int == &raw mut (*curbuf.get()).b_changed {
            !curbufIsChanged() as c_int
        } else {
            (*(varp as *mut c_int) == 0) as c_int
        }) != 0
    {
        msg_puts(b"no\0".as_ptr() as *const c_char);
    } else if option_has_type(opt_idx, kOptValTypeBoolean) as c_int != 0
        && *(varp as *mut c_int) < 0 as c_int
    {
        msg_puts(b"--\0".as_ptr() as *const c_char);
    } else {
        msg_puts(b"  \0".as_ptr() as *const c_char);
    }
    msg_puts((*opt).fullname);
    if !option_has_type(opt_idx, kOptValTypeBoolean) {
        msg_putchar('=' as c_int);
        option_value2string(opt, opt_flags);
        if *(NameBuff.ptr() as *mut c_char) as c_int != NUL {
            msg_outtrans(NameBuff.ptr() as *mut c_char, 0 as c_int, false_0 != 0);
        }
    }
    silent_mode.set(save_silent != 0);
    info_message.set(false_0 != 0);
}

pub unsafe extern "C" fn makeset(
    mut fd: *mut FILE,
    mut opt_flags: c_int,
    mut local_only: c_int,
) -> c_int {
    let mut pri: c_int = 1 as c_int;
    while pri >= 0 as c_int {
        let mut opt: *mut vimoption_T = ::core::ptr::null_mut::<vimoption_T>();
        let mut opt_idx: OptIndex = kOptAleph;
        while (opt_idx as c_int) < kOptCount {
            opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
            's_14: {
                if (*opt).flags & kOptFlagNoMkrc as c_int as uint32_t == 0
                    && (pri == 1 as c_int) as c_int
                        == ((*opt).flags & kOptFlagPriMkrc as c_int as uint32_t != 0 as uint32_t)
                            as c_int
                {
                    if !(option_is_global_only(opt_idx) as c_int != 0
                        && opt_flags & OPT_GLOBAL as c_int == 0)
                    {
                        if !(opt_flags & OPT_GLOBAL as c_int != 0
                            && (*opt).flags & kOptFlagNoGlob as c_int as uint32_t != 0)
                        {
                            let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
                            if !varp.is_null() {
                                if !(opt_flags & OPT_GLOBAL as c_int != 0
                                    && optval_is_default(opt_idx, varp))
                                {
                                    if !(opt_flags & OPT_SKIPRTP as c_int != 0
                                        && ((*opt).var == p_rtp.ptr() as *mut c_void
                                            || (*opt).var == p_pp.ptr() as *mut c_void))
                                    {
                                        let mut round: c_int = 2 as c_int;
                                        let mut varp_local: *mut c_void = NULL;
                                        if option_is_window_local(opt_idx) {
                                            if opt_flags & OPT_LOCAL as c_int == 0 {
                                                break 's_14;
                                            } else if opt_flags & OPT_GLOBAL as c_int == 0
                                                && local_only == 0
                                            {
                                                let mut varp_fresh: *mut c_void =
                                                    get_varp_scope(opt, OPT_GLOBAL as c_int);
                                                if !optval_is_default(opt_idx, varp_fresh) {
                                                    round = 1 as c_int;
                                                    varp_local = varp;
                                                    varp = varp_fresh;
                                                }
                                            }
                                        }
                                        while round <= 2 as c_int {
                                            let mut cmd: *mut c_char =
                                                ::core::ptr::null_mut::<c_char>();
                                            if round == 1 as c_int
                                                || opt_flags & OPT_GLOBAL as c_int != 0
                                            {
                                                cmd = b"set\0".as_ptr() as *const c_char
                                                    as *mut c_char;
                                            } else {
                                                cmd = b"setlocal\0".as_ptr() as *const c_char
                                                    as *mut c_char;
                                            }
                                            let mut do_endif: bool = false_0 != 0;
                                            if opt_idx as c_int == kOptSyntax as c_int
                                                || opt_idx as c_int == kOptFiletype as c_int
                                            {
                                                if fprintf(
                                                    fd,
                                                    b"if &%s != '%s'\0".as_ptr() as *const c_char,
                                                    (*opt).fullname,
                                                    *(varp as *mut *mut c_char),
                                                ) < 0 as c_int
                                                    || put_eol(fd) < 0 as c_int
                                                {
                                                    return FAIL;
                                                }
                                                do_endif = true_0 != 0;
                                            }
                                            if put_set(fd, cmd, opt_idx, varp) == FAIL {
                                                return FAIL;
                                            }
                                            if do_endif {
                                                if put_line(
                                                    fd,
                                                    b"endif\0".as_ptr() as *const c_char
                                                        as *mut c_char,
                                                ) == FAIL
                                                {
                                                    return FAIL;
                                                }
                                            }
                                            varp = varp_local;
                                            round += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            opt_idx += 1;
        }
        pri -= 1;
    }
    return OK;
}

pub unsafe extern "C" fn makefoldset(mut fd: *mut FILE) -> c_int {
    if put_set(
        fd,
        b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
        kOptFoldmethod,
        &raw mut (*curwin.get()).w_onebuf_opt.wo_fdm as *mut c_void,
    ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldexpr,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fde as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldmarker,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fmr as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldignore,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fdi as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldlevel,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fdl as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldminlines,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fml as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldnestmax,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fdn as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldenable,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fen as *mut c_void,
        ) == FAIL
    {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn put_set(
    mut fd: *mut FILE,
    mut cmd: *mut c_char,
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
) -> c_int {
    let mut value: OptVal = optval_from_varp(opt_idx, varp);
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut name: *mut c_char = (*opt).fullname;
    let mut flags: uint64_t = (*opt).flags as uint64_t;
    if option_is_global_local(opt_idx) as c_int != 0
        && varp != (*opt).var
        && optval_equal(value, get_option_unset_value(opt_idx)) as c_int != 0
    {
        return OK;
    }
    match value.type_0 as c_int {
        -1 => {
            abort();
        }
        0 => {
            '_c2rust_label: {
                if value.data.boolean as c_int != kNone as c_int {
                } else {
                    __assert_fail(
                        b"value.data.boolean != kNone\0".as_ptr() as *const c_char,
                        b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                        4544 as c_uint,
                        b"int put_set(FILE *, char *, OptIndex, void *)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            let mut value_bool: bool = if value.data.boolean as c_int == kTrue as c_int {
                true_0
            } else if value.data.boolean as c_int == kFalse as c_int {
                false_0
            } else {
                0 as c_int
            } != 0;
            if fprintf(
                fd,
                b"%s %s%s\0".as_ptr() as *const c_char,
                cmd,
                if value_bool as c_int != 0 {
                    b"\0".as_ptr() as *const c_char
                } else {
                    b"no\0".as_ptr() as *const c_char
                },
                name,
            ) < 0 as c_int
            {
                return FAIL;
            }
        }
        1 => {
            if fprintf(fd, b"%s %s=\0".as_ptr() as *const c_char, cmd, name) < 0 as c_int {
                return FAIL;
            }
            let mut value_num: OptInt = value.data.number;
            let mut wc: OptInt = 0;
            if wc_use_keyname(varp, &raw mut wc) != 0 {
                if fputs(get_special_key_name(wc as c_int, 0 as c_int), fd) < 0 as c_int {
                    return FAIL;
                }
            } else if fprintf(fd, b"%ld\0".as_ptr() as *const c_char, value_num) < 0 as c_int {
                return FAIL;
            }
        }
        2 => {
            if fprintf(fd, b"%s %s=\0".as_ptr() as *const c_char, cmd, name) < 0 as c_int {
                return FAIL;
            }
            let mut value_str: *const c_char = value.data.string.data;
            let mut buf: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut part: *mut c_char = ::core::ptr::null_mut::<c_char>();
            if !value_str.is_null() {
                if flags & kOptFlagExpand as c_int as uint64_t != 0 as uint64_t {
                    let mut size: size_t = strlen(value_str).wrapping_add(1 as size_t);
                    buf = xmalloc(size) as *mut c_char;
                    home_replace(
                        ::core::ptr::null::<buf_T>(),
                        value_str,
                        buf,
                        size,
                        false_0 != 0,
                    );
                    if size >= MAXPATHL as size_t
                        && flags & kOptFlagComma as c_int as uint64_t != 0 as uint64_t
                        && !vim_strchr(value_str, ',' as c_int).is_null()
                    {
                        part = xmalloc(size) as *mut c_char;
                        '_fail: {
                            if put_eol(fd) != FAIL {
                                let mut p: *mut c_char = buf;
                                while *p as c_int != NUL {
                                    if fprintf(
                                        fd,
                                        b"%s %s+=\0".as_ptr() as *const c_char,
                                        cmd,
                                        name,
                                    ) < 0 as c_int
                                    {
                                        break '_fail;
                                    }
                                    copy_option_part(
                                        &raw mut p,
                                        part,
                                        size,
                                        b",\0".as_ptr() as *const c_char as *mut c_char,
                                    );
                                    if put_escstr(fd, part, 2 as c_int) == FAIL
                                        || put_eol(fd) == FAIL
                                    {
                                        break '_fail;
                                    }
                                }
                                xfree(buf as *mut c_void);
                                xfree(part as *mut c_void);
                                return OK;
                            }
                        }
                        xfree(buf as *mut c_void);
                        xfree(part as *mut c_void);
                        return FAIL;
                    } else {
                        if put_escstr(fd, buf, 2 as c_int) == FAIL {
                            xfree(buf as *mut c_void);
                            return FAIL;
                        }
                        xfree(buf as *mut c_void);
                    }
                } else if put_escstr(fd, value_str, 2 as c_int) == FAIL {
                    return FAIL;
                }
            }
        }
        _ => {}
    }
    if put_eol(fd) < 0 as c_int {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn option_value2string(
    mut opt: *mut vimoption_T,
    mut opt_flags: c_int,
) {
    let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
    '_c2rust_label: {
        if !varp.is_null() {
        } else {
            __assert_fail(
                b"varp != NULL\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                6126 as c_uint,
                b"void option_value2string(vimoption_T *, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    if option_has_type(get_opt_idx(opt), kOptValTypeNumber) {
        let mut wc: OptInt = 0 as OptInt;
        if wc_use_keyname(varp, &raw mut wc) != 0 {
            xstrlcpy(
                NameBuff.ptr() as *mut c_char,
                get_special_key_name(wc as c_int, 0 as c_int),
                ::core::mem::size_of::<[c_char; 4096]>(),
            );
        } else if wc != 0 as OptInt {
            xstrlcpy(
                NameBuff.ptr() as *mut c_char,
                transchar(wc as c_int),
                ::core::mem::size_of::<[c_char; 4096]>(),
            );
        } else {
            snprintf(
                NameBuff.ptr() as *mut c_char,
                ::core::mem::size_of::<[c_char; 4096]>(),
                b"%ld\0".as_ptr() as *const c_char,
                *(varp as *mut OptInt),
            );
        }
    } else {
        varp = *(varp as *mut *mut c_char) as *mut c_void;
        if (*opt).flags & kOptFlagExpand as c_int as uint32_t != 0 {
            home_replace(
                ::core::ptr::null::<buf_T>(),
                varp as *const c_char,
                NameBuff.ptr() as *mut c_char,
                MAXPATHL as size_t,
                false_0 != 0,
            );
        } else {
            xstrlcpy(
                NameBuff.ptr() as *mut c_char,
                varp as *const c_char,
                MAXPATHL as size_t,
            );
        }
    };
}

pub(crate) unsafe extern "C" fn wc_use_keyname(
    mut varp: *const c_void,
    mut wcp: *mut OptInt,
) -> c_int {
    if varp as *mut OptInt == p_wc.ptr() || varp as *mut OptInt == p_wcm.ptr() {
        *wcp = *(varp as *mut OptInt);
        if *wcp < 0 as OptInt || find_special_key_in_table(*wcp as c_int) >= 0 as c_int {
            return true_0;
        }
    }
    return false_0;
}
