//! Where an option's default comes from, and the three startup passes
//! that install it.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_init_tablocal() {
    p_ch.set(
        (*options.ptr())[kOptCmdheight as c_int as usize]
            .def_val
            .data
            .number,
    );
}

pub(crate) unsafe extern "C" fn set_init_default_shell() {
    let mut shell: *mut c_char = os_getenv(b"SHELL\0".as_ptr() as *const c_char);
    if !shell.is_null() {
        if !vim_strchr(shell, ' ' as c_int).is_null() {
            let len: size_t = strlen(shell).wrapping_add(3 as size_t);
            let cmd: *mut c_char = xmalloc(len) as *mut c_char;
            snprintf(cmd, len, b"\"%s\"\0".as_ptr() as *const c_char, shell);
            set_string_default(kOptShell, cmd, true_0 != 0);
        } else {
            set_string_default(kOptShell, shell, false_0 != 0);
        }
        xfree(shell as *mut c_void);
    }
}

pub(crate) unsafe extern "C" fn set_init_default_backupskip() {
    static names: GlobalCell<[*mut c_char; 4]> = GlobalCell::new([
        b"\0".as_ptr() as *const c_char as *mut c_char,
        b"TMPDIR\0".as_ptr() as *const c_char as *mut c_char,
        b"TEMP\0".as_ptr() as *const c_char as *mut c_char,
        b"TMP\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    let mut opt_idx: OptIndex = kOptBackupskip;
    ga_init(&raw mut ga, 1 as c_int, 100 as c_int);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*mut c_char; 4]>()
        .wrapping_div(::core::mem::size_of::<*mut c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut c_char; 4]>()
                .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                == 0) as c_int as usize,
        )
    {
        let mut mustfree: bool = true_0 != 0;
        let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut plen: size_t = 0;
        if *(*names.ptr())[i as usize] as c_int == NUL {
            p = b"/tmp\0".as_ptr() as *const c_char as *mut c_char;
            plen = ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as size_t;
            mustfree = false_0 != 0;
        } else {
            p = vim_getenv((*names.ptr())[i as usize] as *const c_char);
            plen = 0 as size_t;
        }
        if !p.is_null() && *p as c_int != NUL {
            let mut has_trailing_path_sep: bool = false_0 != 0;
            if plen == 0 as size_t {
                plen = strlen(p);
                if after_pathsep(p, p.offset(plen as isize)) != 0 {
                    has_trailing_path_sep = true_0 != 0;
                }
            }
            let mut itemsize: size_t = plen
                .wrapping_add(
                    (if has_trailing_path_sep as c_int != 0 {
                        0 as c_int
                    } else {
                        1 as c_int
                    }) as size_t,
                )
                .wrapping_add(2 as size_t);
            let mut item: *mut c_char = xmalloc(itemsize) as *mut c_char;
            let mut itemseplen: size_t = (if ga.ga_len == 0 as c_int {
                0 as c_int
            } else {
                1 as c_int
            }) as size_t;
            let mut itemlen: size_t = vim_snprintf(
                item,
                itemsize,
                b"%s%s*\0".as_ptr() as *const c_char,
                p,
                if has_trailing_path_sep as c_int != 0 {
                    b"\0".as_ptr() as *const c_char
                } else {
                    PATHSEPSTR.as_ptr()
                },
            ) as size_t;
            if find_dup_item(
                ga.ga_data as *const c_char,
                item,
                itemlen,
                (*options.ptr())[opt_idx as usize].flags,
            )
            .is_null()
            {
                ga_grow(
                    &raw mut ga,
                    itemseplen.wrapping_add(itemlen).wrapping_add(1 as size_t) as c_int,
                );
                ga.ga_len += vim_snprintf(
                    (ga.ga_data as *mut c_char).offset(ga.ga_len as isize),
                    itemseplen.wrapping_add(itemlen).wrapping_add(1 as size_t),
                    b"%s%s\0".as_ptr() as *const c_char,
                    if itemseplen > 0 as size_t {
                        b",\0".as_ptr() as *const c_char
                    } else {
                        b"\0".as_ptr() as *const c_char
                    },
                    item,
                );
            }
            xfree(item as *mut c_void);
        }
        if mustfree {
            xfree(p as *mut c_void);
        }
        i = i.wrapping_add(1);
    }
    if !ga.ga_data.is_null() {
        set_string_default(kOptBackupskip, ga.ga_data as *mut c_char, true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn set_init_default_cdpath() {
    let mut cdpath: *mut c_char = vim_getenv(b"CDPATH\0".as_ptr() as *const c_char);
    if cdpath.is_null() {
        return;
    }
    let mut buf: *mut c_char = xmalloc(
        (2 as size_t)
            .wrapping_mul(strlen(cdpath))
            .wrapping_add(2 as size_t),
    ) as *mut c_char;
    *buf.offset(0 as c_int as isize) = ',' as c_char;
    let mut j: c_int = 1 as c_int;
    let mut i: c_int = 0 as c_int;
    while *cdpath.offset(i as isize) as c_int != NUL {
        if vim_ispathlistsep(*cdpath.offset(i as isize) as c_int) {
            let c2rust_fresh0 = j;
            j = j + 1;
            *buf.offset(c2rust_fresh0 as isize) = ',' as c_char;
        } else {
            if *cdpath.offset(i as isize) as c_int == ' ' as c_int
                || *cdpath.offset(i as isize) as c_int == ',' as c_int
            {
                let c2rust_fresh1 = j;
                j = j + 1;
                *buf.offset(c2rust_fresh1 as isize) = '\\' as c_char;
            }
            let c2rust_fresh2 = j;
            j = j + 1;
            *buf.offset(c2rust_fresh2 as isize) = *cdpath.offset(i as isize);
        }
        i += 1;
    }
    *buf.offset(j as isize) = NUL as c_char;
    change_option_default(
        kOptCdpath,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(buf),
            },
        },
    );
    xfree(cdpath as *mut c_void);
}

pub(crate) unsafe extern "C" fn set_init_expand_env() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut opt: *mut vimoption_T =
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
        if (*opt).flags & kOptFlagNoDefExp as c_int as uint32_t == 0 {
            let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
            if (*opt).flags & kOptFlagGettext as c_int as uint32_t != 0 && !(*opt).var.is_null() {
                p = gettext(*((*opt).var as *mut *mut c_char));
            } else {
                p = option_expand(opt_idx, ::core::ptr::null::<c_char>());
            }
            if !p.is_null() {
                set_option_varp(
                    opt_idx,
                    (*opt).var,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_to_string(p),
                        },
                    },
                    true_0 != 0,
                );
                change_option_default(
                    opt_idx,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_to_string(p),
                        },
                    },
                );
            }
        }
        opt_idx += 1;
    }
}

pub(crate) unsafe extern "C" fn set_init_fenc_default() {
    let mut p: *mut c_char = enc_locale();
    if p.is_null() {
        p = xmemdupz(
            b"utf-8\0".as_ptr() as *const c_char as *const c_void,
            ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
        ) as *mut c_char;
    }
    fenc_default.set(p);
}

pub unsafe extern "C" fn set_init_1(mut clean_arg: bool) {
    langmap_init();
    alloc_options_default();
    set_init_default_shell();
    set_init_default_backupskip();
    set_init_default_cdpath();
    let mut backupdir: *mut c_char = stdpaths_user_state_subpath(
        b"backup\0".as_ptr() as *const c_char,
        2 as size_t,
        true_0 != 0,
    );
    let backupdir_len: size_t = strlen(backupdir);
    backupdir = xrealloc(
        backupdir as *mut c_void,
        backupdir_len.wrapping_add(3 as size_t),
    ) as *mut c_char;
    memmove(
        backupdir.offset(2 as c_int as isize) as *mut c_void,
        backupdir as *const c_void,
        backupdir_len.wrapping_add(1 as size_t),
    );
    memmove(
        backupdir as *mut c_void,
        b".,\0".as_ptr() as *const c_char as *const c_void,
        2 as size_t,
    );
    set_string_default(kOptBackupdir, backupdir, true_0 != 0);
    set_string_default(
        kOptViewdir,
        stdpaths_user_state_subpath(
            b"view\0".as_ptr() as *const c_char,
            2 as size_t,
            true_0 != 0,
        ),
        true_0 != 0,
    );
    set_string_default(
        kOptDirectory,
        stdpaths_user_state_subpath(
            b"swap\0".as_ptr() as *const c_char,
            2 as size_t,
            true_0 != 0,
        ),
        true_0 != 0,
    );
    set_string_default(
        kOptUndodir,
        stdpaths_user_state_subpath(
            b"undo\0".as_ptr() as *const c_char,
            2 as size_t,
            true_0 != 0,
        ),
        true_0 != 0,
    );
    let mut rtp: *mut c_char = runtimepath_default(clean_arg);
    if !rtp.is_null() {
        set_string_default(kOptRuntimepath, rtp, true_0 != 0);
        set_string_default(kOptPackpath, rtp, false_0 != 0);
        rtp = ::core::ptr::null_mut::<c_char>();
    }
    set_options_default(0 as c_int);
    (*curbuf.get()).b_p_initialized = true_0 != 0;
    (*curbuf.get()).b_p_ac = -1 as c_int;
    (*curbuf.get()).b_p_ar = -1 as c_int;
    (*curbuf.get()).b_p_fs = -1 as c_int;
    (*curbuf.get()).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
    check_buf_options(curbuf.get());
    check_win_options(curwin.get());
    check_options();
    last_status(false_0 != 0);
    didset_options();
    init_spell_chartab();
    set_init_expand_env();
    if os_env_exists(b"NVIM_NOTTYFAST\0".as_ptr() as *const c_char, false_0 != 0) {
        set_option_value_give_err(
            kOptTtyfast,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
            0 as c_int,
        );
    }
    save_file_ff(curbuf.get());
    if os_env_exists(b"MLTERM\0".as_ptr() as *const c_char, false_0 != 0) {
        set_option_value_give_err(
            kOptTermbidi,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kTrue },
            },
            0 as c_int,
        );
    }
    didset_options2();
    lang_init();
    set_init_fenc_default();
    bind_textdomain_codeset(PROJECT_NAME.as_ptr(), p_enc.get());
    set_helplang_default(get_mess_lang());
}

pub unsafe extern "C" fn get_option_default(opt_idx: OptIndex, mut opt_flags: c_int) -> OptVal {
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut is_global_local_option: bool = option_is_global_local(opt_idx);
    if opt_idx as c_int == kOptModeline as c_int && getuid() == ROOT_UID as __uid_t {
        return OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        };
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && is_global_local_option as c_int != 0 {
        return get_option_unset_value(opt_idx);
    } else if option_has_type(opt_idx, kOptValTypeString) as c_int != 0
        && (*opt).flags & kOptFlagNoDefExp as c_int as uint32_t == 0
    {
        let mut s: *mut c_char = option_expand(opt_idx, (*opt).def_val.data.string.data);
        return if s.is_null() {
            (*opt).def_val
        } else {
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(s),
                },
            }
        };
    } else {
        return (*opt).def_val;
    };
}

pub(crate) unsafe extern "C" fn alloc_options_default() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        (*options.ptr())[opt_idx as usize].def_val =
            optval_copy((*options.ptr())[opt_idx as usize].def_val);
        opt_idx += 1;
    }
}

pub(crate) unsafe extern "C" fn change_option_default(opt_idx: OptIndex, mut value: OptVal) {
    optval_free((*options.ptr())[opt_idx as usize].def_val);
    (*options.ptr())[opt_idx as usize].def_val = value;
}

pub(crate) unsafe extern "C" fn set_option_default(opt_idx: OptIndex, mut opt_flags: c_int) {
    let mut both: bool = opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int;
    let mut def_val: OptVal = get_option_default(opt_idx, opt_flags);
    set_option_direct(opt_idx, def_val, opt_flags, (*current_sctx.ptr()).sc_sid);
    if opt_idx as c_int == kOptScroll as c_int {
        win_comp_scroll(curwin.get());
    }
    let mut flagsp: *mut uint32_t = insecure_flag(curwin.get(), opt_idx, opt_flags);
    *flagsp = *flagsp & !(kOptFlagInsecure as c_int as uint32_t);
    if both {
        flagsp = insecure_flag(curwin.get(), opt_idx, OPT_LOCAL as c_int);
        *flagsp = *flagsp & !(kOptFlagInsecure as c_int as uint32_t);
    }
}

pub(crate) unsafe extern "C" fn set_options_default(mut opt_flags: c_int) {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        if (*options.ptr())[opt_idx as usize].flags & kOptFlagNoDefault as c_int as uint32_t == 0 {
            set_option_default(opt_idx, opt_flags);
        }
        opt_idx += 1;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            win_comp_scroll(wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    parse_cino(curbuf.get());
}

pub(crate) unsafe extern "C" fn set_string_default(
    mut opt_idx: OptIndex,
    mut val: *mut c_char,
    mut allocated: bool,
) {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                546 as c_uint,
                b"void set_string_default(OptIndex, char *, _Bool)\0".as_ptr() as *const c_char,
            );
        }
    };
    change_option_default(
        opt_idx,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(if allocated as c_int != 0 {
                    val
                } else {
                    xstrdup(val)
                }),
            },
        },
    );
}

pub(crate) unsafe extern "C" fn find_dup_item(
    mut origval: *const c_char,
    mut newval: *const c_char,
    newvallen: size_t,
    mut flags: uint32_t,
) -> *const c_char {
    if origval.is_null() {
        return ::core::ptr::null::<c_char>();
    }
    let mut bs: c_int = 0 as c_int;
    let mut s: *const c_char = origval;
    while *s as c_int != NUL {
        if (flags & kOptFlagComma as c_int as uint32_t == 0
            || s == origval
            || *s.offset(-1 as c_int as isize) as c_int == ',' as c_int && bs & 1 as c_int == 0)
            && strncmp(s, newval, newvallen) == 0 as c_int
            && (flags & kOptFlagComma as c_int as uint32_t == 0
                || *s.offset(newvallen as isize) as c_int == ',' as c_int
                || *s.offset(newvallen as isize) as c_int == NUL)
        {
            return s;
        }
        if s > origval.offset(1 as c_int as isize)
            && *s.offset(-1 as c_int as isize) as c_int == '\\' as c_int
            && *s.offset(-2 as c_int as isize) as c_int != ',' as c_int
            || s == origval.offset(1 as c_int as isize)
                && *s.offset(-1 as c_int as isize) as c_int == '\\' as c_int
        {
            bs += 1;
        } else {
            bs = 0 as c_int;
        }
        s = s.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn set_init_2(mut _headless: bool) {
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<c_char>(),
        b"set_init_2\0".as_ptr() as *const c_char,
        613 as c_int,
        true_0 != 0,
        b"startup runtimepath/packpath value: %s\0".as_ptr() as *const c_char,
        p_rtp.get(),
    );
    if (*options.ptr())[kOptScroll as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t
        == 0
    {
        set_option_default(kOptScroll, OPT_LOCAL as c_int);
    }
    comp_col();
    if !option_was_set(kOptWindow) {
        p_window.set((Rows.get() - 1 as c_int) as OptInt);
    }
    change_option_default(
        kOptWindow,
        OptVal {
            type_0: kOptValTypeNumber,
            data: OptValData {
                number: (Rows.get() - 1 as c_int) as OptInt,
            },
        },
    );
}

pub unsafe extern "C" fn set_init_3() {
    parse_shape_opt(SHAPE_CURSOR);
    let mut do_srr: bool = (*options.ptr())[kOptShellredir as c_int as usize].flags
        & kOptFlagWasSet as c_int as uint32_t
        == 0;
    let mut do_sp: bool = (*options.ptr())[kOptShellpipe as c_int as usize].flags
        & kOptFlagWasSet as c_int as uint32_t
        == 0;
    let mut len: size_t = 0 as size_t;
    let mut p: *mut c_char = invocation_path_tail(p_sh.get(), &raw mut len) as *mut c_char;
    p = xmemdupz(p as *const c_void, len) as *mut c_char;
    let mut is_csh: bool = path_fnamecmp(p, b"csh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"tcsh\0".as_ptr() as *const c_char) == 0 as c_int;
    let mut is_known_shell: bool = path_fnamecmp(p, b"sh\0".as_ptr() as *const c_char)
        == 0 as c_int
        || path_fnamecmp(p, b"ksh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"mksh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"pdksh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"zsh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"zsh-beta\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"bash\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"fish\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"ash\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"dash\0".as_ptr() as *const c_char) == 0 as c_int;
    if is_csh as c_int != 0 || is_known_shell as c_int != 0 {
        if do_sp {
            let sp: OptVal = if is_csh as c_int != 0 {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"|& tee\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            } else {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"2>&1| tee\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            };
            set_option_direct(kOptShellpipe, sp, 0 as c_int, SID_NONE);
            change_option_default(kOptShellpipe, optval_copy(sp));
        }
        if do_srr {
            let srr: OptVal = if is_csh as c_int != 0 {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b">&\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            } else {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b">%s 2>&1\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            };
            set_option_direct(kOptShellredir, srr, 0 as c_int, SID_NONE);
            change_option_default(kOptShellredir, optval_copy(srr));
        }
    }
    xfree(p as *mut c_void);
    if buf_is_empty(curbuf.get()) {
        if (*options.ptr())[kOptFileformats as c_int as usize].flags
            & kOptFlagWasSet as c_int as uint32_t
            != 0
        {
            set_fileformat(default_fileformat(), OPT_LOCAL as c_int);
        }
    }
    set_title_defaults();
}

pub unsafe extern "C" fn set_helplang_default(mut lang: *const c_char) {
    if lang.is_null() {
        return;
    }
    let lang_len: size_t = strlen(lang);
    if lang_len < 2 as size_t {
        return;
    }
    if (*options.ptr())[kOptHelplang as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t
        != 0
    {
        return;
    }
    free_string_option(p_hlg.get());
    p_hlg.set(xmemdupz(lang as *const c_void, lang_len) as *mut c_char);
    if strncasecmp(
        p_hlg.get(),
        b"zh_\0".as_ptr() as *const c_char as *mut c_char,
        3 as c_int as size_t,
    ) == 0 as c_int
        && lang_len >= 5 as size_t
    {
        *(*p_hlg.ptr()).offset(0 as c_int as isize) =
            (if (*(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int) < 'A' as c_int
                || *(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int > 'Z' as c_int
            {
                *(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int
            } else {
                *(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int + ('a' as c_int - 'A' as c_int)
            }) as c_char;
        *(*p_hlg.ptr()).offset(1 as c_int as isize) =
            (if (*(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int) < 'A' as c_int
                || *(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int > 'Z' as c_int
            {
                *(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int
            } else {
                *(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int + ('a' as c_int - 'A' as c_int)
            }) as c_char;
    } else if lang_len != 0 && *p_hlg.get() as c_int == 'C' as c_int {
        *(*p_hlg.ptr()).offset(0 as c_int as isize) = 'e' as c_char;
        *(*p_hlg.ptr()).offset(1 as c_int as isize) = 'n' as c_char;
    }
    *(*p_hlg.ptr()).offset(2 as c_int as isize) = NUL as c_char;
}

pub unsafe extern "C" fn set_title_defaults() {
    if (*options.ptr())[kOptTitle as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t
        == 0
    {
        change_option_default(
            kOptTitle,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
        );
        p_title.set(0 as c_int);
    }
    if (*options.ptr())[kOptIcon as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t == 0
    {
        change_option_default(
            kOptIcon,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
        );
        p_icon.set(0 as c_int);
    }
}
