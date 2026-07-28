//! Setting and reading an option's value programmatically.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_option_sctx(mut opt_idx: OptIndex) -> *mut sctx_T {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                2008 as c_uint,
                b"sctx_T *get_option_sctx(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return &raw mut (*(options.ptr() as *mut vimoption_T).offset(opt_idx as isize)).script_ctx;
}

pub unsafe extern "C" fn set_option_sctx(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut script_ctx: sctx_T,
) {
    let mut both: bool = opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int;
    if opt_flags & OPT_MODELINE as c_int == 0 {
        script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
        .es_lnum;
    }
    nlua_set_sctx(&raw mut script_ctx);
    if both as c_int != 0
        || opt_flags & OPT_GLOBAL as c_int != 0
        || option_is_global_only(opt_idx) as c_int != 0
    {
        (*options.ptr())[opt_idx as usize].script_ctx = script_ctx;
    }
    if both as c_int != 0 || opt_flags & OPT_LOCAL as c_int != 0 {
        if option_has_scope(opt_idx, kOptScopeBuf) {
            (*curbuf.get()).b_p_script_ctx[option_scope_idx(opt_idx, kOptScopeBuf) as usize] =
                script_ctx;
        } else if option_has_scope(opt_idx, kOptScopeWin) {
            (*curwin.get()).w_onebuf_opt.wo_script_ctx
                [option_scope_idx(opt_idx, kOptScopeWin) as usize] = script_ctx;
            if both {
                (*curwin.get()).w_allbuf_opt.wo_script_ctx
                    [option_scope_idx(opt_idx, kOptScopeWin) as usize] = script_ctx;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn apply_optionset_autocmd(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut oldval: OptVal,
    mut oldval_g: OptVal,
    mut oldval_l: OptVal,
    mut newval: OptVal,
    mut errmsg: *const c_char,
) {
    if starting.get() != 0 || !errmsg.is_null() || *get_vim_var_str(VV_OPTION_TYPE) as c_int != NUL
    {
        return;
    }
    let mut buf_type: [c_char; 7] = [0; 7];
    let mut oldval_tv: typval_T = optval_as_tv(oldval, false_0 != 0);
    let mut oldval_g_tv: typval_T = optval_as_tv(oldval_g, false_0 != 0);
    let mut oldval_l_tv: typval_T = optval_as_tv(oldval_l, false_0 != 0);
    let mut newval_tv: typval_T = optval_as_tv(newval, false_0 != 0);
    set_vim_var_tv(VV_OPTION_OLD, &raw mut oldval_tv);
    set_vim_var_tv(VV_OPTION_NEW, &raw mut newval_tv);
    let mut typelen: size_t = vim_snprintf_safelen(
        &raw mut buf_type as *mut c_char,
        ::core::mem::size_of::<[c_char; 7]>(),
        b"%s\0".as_ptr() as *const c_char,
        if opt_flags & OPT_LOCAL as c_int != 0 {
            b"local\0".as_ptr() as *const c_char
        } else {
            b"global\0".as_ptr() as *const c_char
        },
    );
    set_vim_var_string(
        VV_OPTION_TYPE,
        &raw mut buf_type as *mut c_char,
        typelen as ptrdiff_t,
    );
    if opt_flags & OPT_LOCAL as c_int != 0 {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"setlocal\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_tv);
    }
    if opt_flags & OPT_GLOBAL as c_int != 0 {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"setglobal\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDGLOBAL, &raw mut oldval_tv);
    }
    if opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"set\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_l_tv);
        set_vim_var_tv(VV_OPTION_OLDGLOBAL, &raw mut oldval_g_tv);
    }
    if opt_flags & OPT_MODELINE as c_int != 0 {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"modeline\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_tv);
    }
    apply_autocmds(
        EVENT_OPTIONSET,
        (*options.ptr())[opt_idx as usize].fullname,
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    reset_v_option_vars();
}

pub unsafe extern "C" fn is_tty_option(mut name: *const c_char) -> bool {
    return !find_tty_option_end(name).is_null();
}

pub unsafe extern "C" fn get_tty_option(mut name: *const c_char) -> OptVal {
    let mut value: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if strequal(name, b"t_Co\0".as_ptr() as *const c_char) {
        if t_colors.get() <= 1 as c_int {
            value = xstrdup(b"\0".as_ptr() as *const c_char);
        } else {
            value = xmalloc(NUMBUFLEN as c_int as size_t) as *mut c_char;
            snprintf(
                value,
                NUMBUFLEN as c_int as size_t,
                b"%d\0".as_ptr() as *const c_char,
                t_colors.get(),
            );
        }
    } else if strequal(name, b"term\0".as_ptr() as *const c_char) {
        value = if !(*p_term.ptr()).is_null() {
            xstrdup(p_term.get())
        } else {
            xstrdup(b"nvim\0".as_ptr() as *const c_char)
        };
    } else if strequal(name, b"ttytype\0".as_ptr() as *const c_char) {
        value = if !(*p_ttytype.ptr()).is_null() {
            xstrdup(p_ttytype.get())
        } else {
            xstrdup(b"nvim\0".as_ptr() as *const c_char)
        };
    } else if is_tty_option(name) {
        value = xstrdup(b"\0".as_ptr() as *const c_char);
    }
    return if value.is_null() {
        OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        }
    } else {
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(value),
            },
        }
    };
}

pub unsafe extern "C" fn set_tty_option(mut name: *const c_char, mut value: *mut c_char) -> bool {
    if strequal(name, b"term\0".as_ptr() as *const c_char) {
        if !(*p_term.ptr()).is_null() {
            xfree(p_term.get() as *mut c_void);
        }
        p_term.set(value);
        return true_0 != 0;
    }
    if strequal(name, b"ttytype\0".as_ptr() as *const c_char) {
        if !(*p_ttytype.ptr()).is_null() {
            xfree(p_ttytype.get() as *mut c_void);
        }
        p_ttytype.set(value);
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub unsafe extern "C" fn find_option_len(name: *const c_char, len: size_t) -> OptIndex {
    if len == 0 {
        return kOptInvalid;
    }
    // SAFETY: the caller passes `len` readable bytes at `name`.
    find_option_index(unsafe { ::core::slice::from_raw_parts(name.cast::<u8>(), len) })
}

pub unsafe extern "C" fn find_option(name: *const c_char) -> OptIndex {
    return find_option_len(name, strlen(name));
}

pub unsafe extern "C" fn get_option_value(mut opt_idx: OptIndex, mut opt_flags: c_int) -> OptVal {
    if opt_idx as c_int == kOptInvalid as c_int {
        return OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
    }
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
    return optval_copy(optval_from_varp(opt_idx, varp));
}

pub unsafe extern "C" fn get_option(mut opt_idx: OptIndex) -> *mut vimoption_T {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3580 as c_uint,
                b"vimoption_T *get_option(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
}

pub(crate) unsafe extern "C" fn get_option_unset_value(mut opt_idx: OptIndex) -> OptVal {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3593 as c_uint,
                b"OptVal get_option_unset_value(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    if option_is_global_local(opt_idx) {
        if option_has_type(opt_idx, kOptValTypeString) {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 1]>().wrapping_sub(1 as size_t),
                    },
                },
            };
        }
        match opt_idx as c_int {
            6 | 10 | 118 => {
                return OptVal {
                    type_0: kOptValTypeBoolean,
                    data: OptValData { boolean: kNone },
                };
            }
            247 | 276 => {
                return OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData {
                        number: -1 as OptInt,
                    },
                };
            }
            333 => {
                return OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData {
                        number: -123456 as OptInt,
                    },
                };
            }
            _ => {
                abort();
            }
        }
    }
    return optval_from_varp(opt_idx, get_varp_scope(opt, OPT_GLOBAL as c_int));
}

pub(crate) unsafe extern "C" fn is_option_local_value_unset(mut opt_idx: OptIndex) -> bool {
    let mut opt: *mut vimoption_T = get_option(opt_idx);
    if !option_is_global_local(opt_idx) {
        return false_0 != 0;
    }
    let mut varp_local: *mut c_void = get_varp_scope(opt, OPT_LOCAL as c_int);
    let mut local_value: OptVal = optval_from_varp(opt_idx, varp_local);
    let mut unset_local_value: OptVal = get_option_unset_value(opt_idx);
    return optval_equal(local_value, unset_local_value);
}

pub(crate) unsafe extern "C" fn did_set_option(
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
    mut old_value: OptVal,
    mut new_value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut restore_chartab: bool = false_0 != 0;
    let mut value_changed: bool = false_0 != 0;
    let mut value_checked: bool = false_0 != 0;
    let mut did_set_cb_args: optset_T = optset_T {
        os_varp: varp,
        os_idx: opt_idx,
        os_flags: opt_flags,
        os_oldval: old_value.data,
        os_newval: new_value.data,
        os_value_checked: false_0 != 0,
        os_value_changed: false_0 != 0,
        os_restore_chartab: false_0 != 0,
        os_errbuf: errbuf,
        os_errbuflen: errbuflen,
        os_win: curwin.get() as *mut c_void,
        os_buf: curbuf.get() as *mut c_void,
    };
    if !direct {
        if (*opt).immutable as c_int != 0 && !optval_equal(old_value, new_value) {
            errmsg = &raw const e_unsupportedoption as *const c_char;
        } else if (secure.get() != 0 || sandbox.get() != 0 as c_int)
            && (*opt).flags & kOptFlagSecure as c_int as uint32_t != 0
        {
            errmsg = &raw const e_secure as *const c_char;
        } else if new_value.type_0 as c_int == kOptValTypeString as c_int
            && check_illegal_path_names(*(varp as *mut *mut c_char), (*opt).flags) as c_int != 0
        {
            errmsg = &raw const e_invarg as *const c_char;
        } else if (*opt).opt_did_set_cb.is_some() {
            errmsg =
                (*opt).opt_did_set_cb.expect("non-null function pointer")(&raw mut did_set_cb_args);
            value_changed = did_set_cb_args.os_value_changed;
            value_checked = did_set_cb_args.os_value_checked;
            restore_chartab = did_set_cb_args.os_restore_chartab;
        }
    }
    if !errmsg.is_null() {
        set_option_varp(opt_idx, varp, old_value, true_0 != 0);
        if restore_chartab {
            buf_init_chartab(curbuf.get(), true);
        }
        return errmsg;
    }
    new_value = optval_from_varp(opt_idx, varp);
    if set_sid != SID_NONE {
        let mut script_ctx: sctx_T = if set_sid == 0 as c_int {
            current_sctx.get()
        } else {
            sctx_T {
                sc_sid: set_sid,
                sc_seq: 0,
                sc_lnum: 0,
                sc_chan: 0,
            }
        };
        set_option_sctx(opt_idx, opt_flags, script_ctx);
    }
    optval_free(old_value);
    let scope_both: bool = opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int;
    if scope_both {
        if option_is_global_local(opt_idx) {
            let mut varp_local: *mut c_void = get_varp_scope(opt, OPT_LOCAL as c_int);
            let mut local_unset_value: OptVal = get_option_unset_value(opt_idx);
            set_option_varp(
                opt_idx,
                varp_local,
                optval_copy(local_unset_value),
                true_0 != 0,
            );
        } else {
            let mut varp_global: *mut c_void = get_varp_scope(opt, OPT_GLOBAL as c_int);
            set_option_varp(opt_idx, varp_global, optval_copy(new_value), true_0 != 0);
        }
    }
    if direct {
        return errmsg;
    }
    if varp == &raw mut (*curbuf.get()).b_p_syn as *mut c_void {
        do_syntax_autocmd(curbuf.get(), value_changed);
    } else if varp == &raw mut (*curbuf.get()).b_p_ft as *mut c_void {
        if opt_flags & OPT_MODELINE as c_int == 0 || value_changed as c_int != 0 {
            do_filetype_autocmd(curbuf.get(), value_changed);
        }
    } else if varp == &raw mut (*(*curwin.get()).w_s).b_p_spl as *mut c_void {
        do_spelllang_source(curwin.get());
    }
    comp_col();
    if varp == p_mouse.ptr() as *mut c_void {
        setmouse();
    } else if (varp == p_flp.ptr() as *mut c_void
        || varp == &raw mut (*curbuf.get()).b_p_flp as *mut c_void)
        && (*curwin.get()).w_briopt_list != 0
    {
        redraw_all_later(UPD_NOT_VALID as c_int);
    } else if varp == p_wbr.ptr() as *mut c_void
        || varp == &raw mut (*curwin.get()).w_onebuf_opt.wo_wbr as *mut c_void
    {
        set_winbar(true_0 != 0);
    }
    if (*curwin.get()).w_curswant != MAXCOL as c_int
        && (*opt).flags & (kOptFlagCurswant as c_int | kOptFlagRedrAll as c_int) as uint32_t
            != 0 as uint32_t
        && (*opt).flags & kOptFlagHLOnly as c_int as uint32_t == 0 as uint32_t
    {
        (*curwin.get()).w_set_curswant = true_0;
    }
    check_redraw((*opt).flags);
    if errmsg.is_null() {
        (*opt).flags |= kOptFlagWasSet as c_int as uint32_t;
        let mut flagsp: *mut uint32_t = insecure_flag(curwin.get(), opt_idx, opt_flags);
        let mut flagsp_local: *mut uint32_t = if scope_both as c_int != 0 {
            insecure_flag(curwin.get(), opt_idx, OPT_LOCAL as c_int)
        } else {
            ::core::ptr::null_mut::<uint32_t>()
        };
        if !value_checked
            && (secure.get() != 0
                || sandbox.get() != 0 as c_int
                || opt_flags & OPT_MODELINE as c_int != 0)
        {
            *flagsp |= kOptFlagInsecure as c_int as uint32_t;
            if !flagsp_local.is_null() {
                *flagsp_local |= kOptFlagInsecure as c_int as uint32_t;
            }
        } else if value_replaced {
            *flagsp = (*flagsp as c_uint & !(kOptFlagInsecure as c_int as c_uint)) as uint32_t;
            if !flagsp_local.is_null() {
                *flagsp_local =
                    (*flagsp_local as c_uint & !(kOptFlagInsecure as c_int as c_uint)) as uint32_t;
            }
        }
    }
    return errmsg;
}

pub(crate) unsafe extern "C" fn set_option(
    opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr()
                    as *const c_char,
                3871 as c_uint,
                b"const char *set_option(const OptIndex, OptVal, int, scid_T, const _Bool, const _Bool, char *, size_t)\0"
                    .as_ptr() as *const c_char,
            );
        }
    };
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    if !direct {
        errmsg = validate_option_value(opt_idx, &mut value, opt_flags, errbuf, errbuflen);
        if !errmsg.is_null() {
            optval_free(value);
            return errmsg;
        }
    }
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let scope_local: bool = opt_flags & OPT_LOCAL as c_int != 0;
    let scope_global: bool = opt_flags & OPT_GLOBAL as c_int != 0;
    let scope_both: bool = !scope_local && !scope_global;
    let is_opt_local_unset: bool = is_option_local_value_unset(opt_idx);
    let mut varp: *mut c_void =
        if scope_both as c_int != 0 && option_is_global_local(opt_idx) as c_int != 0 {
            (*opt).var
        } else {
            get_varp_scope(opt, opt_flags)
        };
    let mut varp_local: *mut c_void = get_varp_scope(opt, OPT_LOCAL as c_int);
    let mut varp_global: *mut c_void = get_varp_scope(opt, OPT_GLOBAL as c_int);
    let mut old_value: OptVal = optval_from_varp(opt_idx, varp);
    let mut old_global_value: OptVal = optval_from_varp(opt_idx, varp_global);
    let mut old_local_value: OptVal = if is_opt_local_unset as c_int != 0 {
        old_global_value
    } else {
        optval_from_varp(opt_idx, varp_local)
    };
    let mut used_old_value: OptVal =
        if scope_local as c_int != 0 && is_opt_local_unset as c_int != 0 {
            optval_from_varp(opt_idx, get_varp(opt))
        } else {
            old_value
        };
    let mut saved_used_value: OptVal = optval_copy(used_old_value);
    let mut saved_old_global_value: OptVal = optval_copy(old_global_value);
    let mut saved_old_local_value: OptVal = optval_copy(old_local_value);
    let mut saved_new_value: OptVal = optval_copy(value);
    let mut p: *mut uint32_t = insecure_flag(curwin.get(), opt_idx, opt_flags);
    let secure_saved: c_int = secure.get();
    if opt_flags & OPT_MODELINE as c_int != 0
        || sandbox.get() != 0 as c_int
        || !value_replaced && *p & kOptFlagInsecure as c_int as uint32_t != 0
    {
        secure.set(1 as c_int);
    }
    set_option_varp(opt_idx, varp, value, false_0 != 0);
    errmsg = did_set_option(
        opt_idx,
        varp,
        old_value,
        value,
        opt_flags,
        set_sid,
        direct,
        value_replaced,
        errbuf,
        errbuflen,
    );
    secure.set(secure_saved);
    if errmsg.is_null() && !direct {
        if starting.get() == 0 {
            apply_optionset_autocmd(
                opt_idx,
                opt_flags,
                saved_used_value,
                saved_old_global_value,
                saved_old_local_value,
                saved_new_value,
                errmsg,
            );
        }
        if (*opt).flags & kOptFlagUIOption as c_int as uint32_t != 0 {
            ui_call_option_set(
                cstr_as_string((*opt).fullname),
                optval_as_object(saved_new_value),
            );
        }
    }
    optval_free(saved_used_value);
    optval_free(saved_old_local_value);
    optval_free(saved_old_global_value);
    optval_free(saved_new_value);
    return errmsg;
}

pub unsafe extern "C" fn set_option_direct(
    mut opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
) {
    static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
    if is_option_hidden(opt_idx) {
        return;
    }
    let mut errmsg: *const c_char = set_option(
        opt_idx,
        optval_copy(value),
        opt_flags,
        set_sid,
        true_0 != 0,
        true_0 != 0,
        errbuf.ptr() as *mut c_char,
        ::core::mem::size_of::<[c_char; 1025]>(),
    );
    '_c2rust_label: {
        if errmsg.is_null() {
        } else {
            __assert_fail(
                b"errmsg == NULL\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3975 as c_uint,
                b"void set_option_direct(OptIndex, OptVal, int, scid_T)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
}

pub unsafe extern "C" fn set_option_direct_for(
    mut opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
    mut scope: OptScope,
    from: *mut c_void,
) {
    let mut save_curbuf: *mut buf_T = curbuf.get();
    let mut save_curwin: *mut win_T = curwin.get();
    match scope as c_uint {
        1 => {
            curwin.set(from as *mut win_T);
            curbuf.set((*curwin.get()).w_buffer);
        }
        2 => {
            curbuf.set(from as *mut buf_T);
        }
        0 | _ => {}
    }
    set_option_direct(opt_idx, value, opt_flags, set_sid);
    curwin.set(save_curwin);
    curbuf.set(save_curbuf);
}

pub unsafe extern "C" fn set_option_value(
    opt_idx: OptIndex,
    value: OptVal,
    mut opt_flags: c_int,
) -> *const c_char {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                4025 as c_uint,
                b"const char *set_option_value(const OptIndex, const OptVal, int)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
    let mut flags: uint32_t = (*options.ptr())[opt_idx as usize].flags;
    if sandbox.get() > 0 as c_int && flags & kOptFlagSecure as c_int as uint32_t != 0 {
        return gettext(&raw const e_sandbox as *const c_char);
    }
    return set_option(
        opt_idx,
        optval_copy(value),
        opt_flags,
        0 as scid_T,
        false_0 != 0,
        true_0 != 0,
        errbuf.ptr() as *mut c_char,
        ::core::mem::size_of::<[c_char; 1025]>(),
    );
}

#[inline]
pub(crate) unsafe extern "C" fn unset_option_local_value(opt_idx: OptIndex) -> *const c_char {
    '_c2rust_label: {
        if option_is_global_local(opt_idx) {
        } else {
            __assert_fail(
                b"option_is_global_local(opt_idx)\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                4045 as c_uint,
                b"const char *unset_option_local_value(const OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return set_option_value(opt_idx, get_option_unset_value(opt_idx), OPT_LOCAL as c_int);
}

pub unsafe extern "C" fn set_option_value_handle_tty(
    mut name: *const c_char,
    mut opt_idx: OptIndex,
    value: OptVal,
    mut opt_flags: c_int,
) -> *const c_char {
    static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
    if opt_idx as c_int == kOptInvalid as c_int {
        if is_tty_option(name) {
            return ::core::ptr::null::<c_char>();
        }
        snprintf(
            errbuf.ptr() as *mut c_char,
            ::core::mem::size_of::<[c_char; 1025]>(),
            gettext(&raw const e_unknown_option2 as *const c_char),
            name,
        );
        return errbuf.ptr() as *mut c_char;
    }
    return set_option_value(opt_idx, value, opt_flags);
}

pub unsafe extern "C" fn set_option_value_give_err(
    opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
) {
    let mut errmsg: *const c_char = set_option_value(opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        emsg(gettext(errmsg));
    }
}

pub(crate) unsafe extern "C" fn switch_option_context(
    ctx: *mut c_void,
    mut scope: OptScope,
    from: *mut c_void,
    mut err: *mut Error,
) -> bool {
    match scope as c_uint {
        0 => return false_0 != 0,
        1 => {
            let win: *mut win_T = from as *mut win_T;
            let switchwin: *mut switchwin_T = ctx as *mut switchwin_T;
            if win == curwin.get() {
                return false_0 != 0;
            }
            if switch_win_noblock(switchwin, win, win_find_tabpage(win), true_0 != 0) == FAIL {
                restore_win_noblock(switchwin, true_0 != 0);
                if (*err).type_0 as c_int != kErrorTypeNone as c_int {
                    return false_0 != 0;
                }
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Problem while switching windows\0".as_ptr() as *const c_char,
                );
                return false_0 != 0;
            }
            return true_0 != 0;
        }
        2 => {
            let buf: *mut buf_T = from as *mut buf_T;
            let aco: *mut aco_save_T = ctx as *mut aco_save_T;
            if buf == curbuf.get() {
                return false_0 != 0;
            }
            aucmd_prepbuf(aco, buf);
            return true_0 != 0;
        }
        _ => {}
    }
    unreachable!();
}

pub(crate) unsafe extern "C" fn restore_option_context(ctx: *mut c_void, mut scope: OptScope) {
    match scope as c_uint {
        1 => {
            restore_win_noblock(ctx as *mut switchwin_T, true_0 != 0);
        }
        2 => {
            aucmd_restbuf(ctx as *mut aco_save_T);
        }
        0 | _ => {}
    };
}

pub unsafe extern "C" fn get_option_value_for(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    scope: OptScope,
    from: *mut c_void,
    mut err: *mut Error,
) -> OptVal {
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<c_char>(),
        globaldir: ::core::ptr::null_mut::<c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut ctx: *mut c_void = if scope as c_uint == kOptScopeWin as c_int as c_uint {
        &raw mut switchwin as *mut c_void
    } else if scope as c_uint == kOptScopeBuf as c_int as c_uint {
        &raw mut aco as *mut c_void
    } else {
        NULL
    };
    let mut switched: bool = switch_option_context(ctx, scope, from, err);
    if (*err).type_0 as c_int != kErrorTypeNone as c_int {
        return OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
    }
    let mut retv: OptVal = get_option_value(opt_idx, opt_flags);
    if switched {
        restore_option_context(ctx, scope);
    }
    return retv;
}

pub unsafe extern "C" fn set_option_value_for(
    mut name: *const c_char,
    mut opt_idx: OptIndex,
    mut value: OptVal,
    opt_flags: c_int,
    scope: OptScope,
    from: *mut c_void,
    mut err: *mut Error,
) {
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<c_char>(),
        globaldir: ::core::ptr::null_mut::<c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut ctx: *mut c_void = if scope as c_uint == kOptScopeWin as c_int as c_uint {
        &raw mut switchwin as *mut c_void
    } else if scope as c_uint == kOptScopeBuf as c_int as c_uint {
        &raw mut aco as *mut c_void
    } else {
        NULL
    };
    let mut switched: bool = switch_option_context(ctx, scope, from, err);
    if (*err).type_0 as c_int != kErrorTypeNone as c_int {
        return;
    }
    let errmsg: *const c_char = set_option_value_handle_tty(name, opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const c_char,
            errmsg,
        );
    }
    if switched {
        restore_option_context(ctx, scope);
    }
}

pub(crate) unsafe extern "C" fn didset_options_sctx(mut opt_flags: c_int, mut buf: *mut c_int) {
    let mut i: c_int = 0 as c_int;
    while *buf.offset(i as isize) != kOptInvalid as c_int {
        set_option_sctx(
            *buf.offset(i as isize) as OptIndex,
            opt_flags,
            current_sctx.get(),
        );
        i += 1;
    }
}
