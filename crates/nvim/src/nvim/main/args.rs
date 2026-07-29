//! Command-line argument scanning, and the small initialisations
//! whose input is an argument.
//!
//! `command_line_scan` walks argv once. Anything it cannot place is a usage
//! error; anything that needs the next word sets `want_argument` and is
//! collected at the bottom of the loop.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_number_arg(
    mut p: *const c_char,
    mut idx: *mut c_int,
    mut def: c_int,
) -> c_int {
    if ascii_isdigit(*p.offset(*idx as isize) as c_int) {
        def = atoi(p.offset(*idx as isize));
        while ascii_isdigit(*p.offset(*idx as isize) as c_int) {
            *idx = *idx + 1 as c_int;
        }
    }
    return def;
}

pub(crate) unsafe extern "C" fn edit_stdin(mut parmp: *mut mparm_T) -> bool {
    let mut implicit: bool = !headless_mode.get()
        && !(embedded_mode.get() as c_int != 0 && stdin_fd.get() <= 0 as c_int)
        && (!exmode_active.get() || (*parmp).input_istext as c_int != 0)
        && !stdin_isatty.get()
        && (*parmp).edit_type <= EDIT_STDIN as c_int
        && (*parmp).scriptin.is_null();
    return (*parmp).had_stdin_file as c_int != 0 || implicit as c_int != 0;
}

pub(crate) unsafe extern "C" fn command_line_scan(mut parmp: *mut mparm_T) {
    let mut argc: c_int = (*parmp).argc;
    let mut argv: *mut *mut c_char = (*parmp).argv;
    let mut argv_idx: c_int = 0;
    let mut had_minmin: bool = false_0 != 0;
    let mut want_argument: bool = false;
    let mut n: c_int = 0;
    argc -= 1;
    argv = argv.offset(1);
    argv_idx = 1 as c_int;
    while argc > 0 as c_int {
        if *(*argv.offset(0 as c_int as isize)).offset(0 as c_int as isize) as c_int == '+' as c_int
            && !had_minmin
        {
            if (*parmp).n_commands >= MAX_ARG_CMDS {
                mainerr(
                    err_extra_cmd.get(),
                    ::core::ptr::null::<c_char>(),
                    ::core::ptr::null::<c_char>(),
                );
            }
            argv_idx = -1 as c_int;
            if *(*argv.offset(0 as c_int as isize)).offset(1 as c_int as isize) as c_int == NUL {
                let c2rust_fresh6 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr = &raw mut (*parmp).commands[c2rust_fresh6 as usize];
                *c2rust_lvalue_ptr = b"$\0".as_ptr() as *const c_char as *mut c_char;
            } else {
                let c2rust_fresh7 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr_0 = &raw mut (*parmp).commands[c2rust_fresh7 as usize];
                *c2rust_lvalue_ptr_0 =
                    (*argv.offset(0 as c_int as isize)).offset(1 as c_int as isize);
            }
        } else if *(*argv.offset(0 as c_int as isize)).offset(0 as c_int as isize) as c_int
            == '-' as c_int
            && !had_minmin
        {
            want_argument = false_0 != 0;
            let c2rust_fresh8 = argv_idx;
            argv_idx = argv_idx + 1;
            let mut c: c_char = *(*argv.offset(0 as c_int as isize)).offset(c2rust_fresh8 as isize);
            's_747: {
                'c_49604: {
                    match c as c_int {
                        NUL => {
                            if exmode_active.get() {
                                silent_mode.set(true_0 != 0);
                                (*parmp).no_swap_file = true_0;
                            } else {
                                if (*parmp).edit_type > EDIT_STDIN as c_int {
                                    mainerr(
                                        err_too_many_args.get(),
                                        *argv.offset(0 as c_int as isize),
                                        ::core::ptr::null::<c_char>(),
                                    );
                                }
                                (*parmp).had_stdin_file = true_0 != 0;
                                (*parmp).edit_type = EDIT_STDIN as c_int;
                            }
                            argv_idx = -1 as c_int;
                            break 's_747;
                        }
                        45 => {
                            if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"help\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                usage();
                                os_exit(0 as c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"version\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                version();
                                os_exit(0 as c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"api-info\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                let mut data: String_0 = api_metadata_raw();
                                let written_bytes: ptrdiff_t =
                                    os_write(STDOUT_FILENO, data.data, data.size, false_0 != 0);
                                if written_bytes < 0 as ptrdiff_t {
                                    semsg(
                                        gettext(b"E5420: Failed to write to file: %s\0".as_ptr()
                                            as *const c_char),
                                        uv_strerror(written_bytes as c_int),
                                    );
                                }
                                os_exit(0 as c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"headless\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                headless_mode.set(true_0 != 0);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"embed\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                embedded_mode.set(true_0 != 0);
                            } else if strncasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"listen\0".as_ptr() as *const c_char as *mut c_char,
                                6 as c_int as size_t,
                            ) == 0 as c_int
                            {
                                want_argument = true_0 != 0;
                                argv_idx += 6 as c_int;
                            } else if strncasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"literal\0".as_ptr() as *const c_char as *mut c_char,
                                7 as c_int as size_t,
                            ) != 0 as c_int
                            {
                                if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"remote\0".as_ptr() as *const c_char as *mut c_char,
                                    6 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    (*parmp).remote = (*parmp).argc - argc;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"server\0".as_ptr() as *const c_char as *mut c_char,
                                    6 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 6 as c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"noplugin\0".as_ptr() as *const c_char as *mut c_char,
                                    8 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    p_lpl.set(false_0);
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"cmd\0".as_ptr() as *const c_char as *mut c_char,
                                    3 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 3 as c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"startuptime\0".as_ptr() as *const c_char as *mut c_char,
                                    11 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 11 as c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"clean\0".as_ptr() as *const c_char as *mut c_char,
                                    5 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    (*parmp).use_vimrc =
                                        b"NONE\0".as_ptr() as *const c_char as *mut c_char;
                                    (*parmp).clean = true_0 != 0;
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr() as *const c_char
                                                        as *mut c_char,
                                                    size: ::core::mem::size_of::<[c_char; 5]>()
                                                        .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as c_int,
                                    );
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"luamod-dev\0".as_ptr() as *const c_char as *mut c_char,
                                    9 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    nlua_disable_preload.set(true_0 != 0);
                                } else {
                                    if *(*argv.offset(0 as c_int as isize))
                                        .offset(argv_idx as isize)
                                        != 0
                                    {
                                        mainerr(
                                            err_opt_unknown.get(),
                                            *argv.offset(0 as c_int as isize),
                                            ::core::ptr::null::<c_char>(),
                                        );
                                    }
                                    had_minmin = true_0 != 0;
                                }
                            }
                            if !want_argument {
                                argv_idx = -1 as c_int;
                            }
                            break 's_747;
                        }
                        65 => {
                            set_option_value_give_err(
                                kOptArabic,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as c_int,
                            );
                            break 's_747;
                        }
                        98 => {
                            set_options_bin((*curbuf.get()).b_p_bin, 1 as c_int, 0 as c_int);
                            (*curbuf.get()).b_p_bin = 1 as c_int;
                            break 's_747;
                        }
                        68 => {
                            (*parmp).use_debug_break_level = 9999 as c_int;
                            break 's_747;
                        }
                        100 => {
                            (*parmp).diff_mode = true_0;
                            break 's_747;
                        }
                        101 => {
                            exmode_active.set(true_0 != 0);
                            break 's_747;
                        }
                        69 => {
                            exmode_active.set(true_0 != 0);
                            (*parmp).input_istext = true_0 != 0;
                            break 's_747;
                        }
                        63 | 104 => {
                            usage();
                            os_exit(0 as c_int);
                        }
                        72 => {
                            set_option_value_give_err(
                                kOptKeymap,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: String_0 {
                                            data: b"hebrew\0".as_ptr() as *const c_char
                                                as *mut c_char,
                                            size: ::core::mem::size_of::<[c_char; 7]>()
                                                .wrapping_sub(1 as size_t),
                                        },
                                    },
                                },
                                0 as c_int,
                            );
                            set_option_value_give_err(
                                kOptRightleft,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as c_int,
                            );
                            break 's_747;
                        }
                        77 => {
                            reset_modifiable();
                        }
                        109 => {}
                        102 | 78 | 88 => {
                            break 's_747;
                        }
                        110 => {
                            (*parmp).no_swap_file = true_0;
                            break 's_747;
                        }
                        112 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                0 as c_int,
                            );
                            (*parmp).window_layout = WIN_TABS as c_int;
                            break 's_747;
                        }
                        111 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                0 as c_int,
                            );
                            (*parmp).window_layout = WIN_HOR as c_int;
                            break 's_747;
                        }
                        79 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                0 as c_int,
                            );
                            (*parmp).window_layout = WIN_VER as c_int;
                            break 's_747;
                        }
                        113 => {
                            if (*parmp).edit_type != EDIT_NONE as c_int {
                                mainerr(
                                    err_too_many_args.get(),
                                    *argv.offset(0 as c_int as isize),
                                    ::core::ptr::null::<c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_QF as c_int;
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) != 0 {
                                (*parmp).use_ef =
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize);
                                argv_idx = -1 as c_int;
                            } else if argc > 1 as c_int {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        82 => {
                            readonlymode.set(true_0 != 0);
                            (*curbuf.get()).b_p_ro = true_0;
                            p_uc.set(10000 as OptInt);
                            break 's_747;
                        }
                        114 | 76 => {
                            recoverymode.set(true);
                            break 's_747;
                        }
                        115 => {
                            if exmode_active.get() {
                                silent_mode.set(true_0 != 0);
                                (*parmp).no_swap_file = true_0;
                                if (*p_shadafile.ptr()).is_null()
                                    || *p_shadafile.get() as c_int == NUL
                                {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr() as *const c_char
                                                        as *mut c_char,
                                                    size: ::core::mem::size_of::<[c_char; 5]>()
                                                        .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as c_int,
                                    );
                                }
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        116 => {
                            if (*parmp).edit_type != EDIT_NONE as c_int {
                                mainerr(
                                    err_too_many_args.get(),
                                    *argv.offset(0 as c_int as isize),
                                    ::core::ptr::null::<c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_TAG as c_int;
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) != 0 {
                                (*parmp).tagname =
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize);
                                argv_idx = -1 as c_int;
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        118 => {
                            version();
                            os_exit(0 as c_int);
                        }
                        86 => {
                            p_verbose.set(get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                10 as c_int,
                            ) as OptInt);
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize)
                                as c_int
                                != NUL
                            {
                                set_option_value_give_err(
                                    kOptVerbosefile,
                                    OptVal {
                                        type_0: kOptValTypeString,
                                        data: OptValData {
                                            string: cstr_as_string(
                                                (*argv.offset(0 as c_int as isize))
                                                    .offset(argv_idx as isize),
                                            ),
                                        },
                                    },
                                    0 as c_int,
                                );
                                argv_idx = strlen(*argv.offset(0 as c_int as isize)) as c_int;
                            }
                            break 's_747;
                        }
                        119 => {
                            if ascii_isdigit(
                                *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize)
                                    as c_int,
                            ) {
                                n = get_number_arg(
                                    *argv.offset(0 as c_int as isize),
                                    &raw mut argv_idx,
                                    10 as c_int,
                                );
                                set_option_value_give_err(
                                    kOptWindow,
                                    OptVal {
                                        type_0: kOptValTypeNumber,
                                        data: OptValData {
                                            number: n as OptInt,
                                        },
                                    },
                                    0 as c_int,
                                );
                                break 's_747;
                            } else {
                                want_argument = true_0 != 0;
                                break 's_747;
                            }
                        }
                        99 => {
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize)
                                as c_int
                                != NUL
                            {
                                if (*parmp).n_commands >= MAX_ARG_CMDS {
                                    mainerr(
                                        err_extra_cmd.get(),
                                        ::core::ptr::null::<c_char>(),
                                        ::core::ptr::null::<c_char>(),
                                    );
                                }
                                let c2rust_fresh9 = (*parmp).n_commands;
                                (*parmp).n_commands = (*parmp).n_commands + 1;
                                let c2rust_lvalue_ptr_1 =
                                    &raw mut (*parmp).commands[c2rust_fresh9 as usize];
                                *c2rust_lvalue_ptr_1 = (*argv).offset(argv_idx as isize);
                                argv_idx = -1 as c_int;
                                break 's_747;
                            } else {
                                break 'c_49604;
                            }
                        }
                        83 | 105 | 108 | 117 | 85 | 87 => {
                            break 'c_49604;
                        }
                        _ => {
                            mainerr(
                                err_opt_unknown.get(),
                                *argv.offset(0 as c_int as isize),
                                ::core::ptr::null::<c_char>(),
                            );
                        }
                    }
                    p_write.set(false_0);
                    break 's_747;
                }
                want_argument = true_0 != 0;
            }
            if want_argument {
                if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) as c_int != NUL {
                    mainerr(
                        err_opt_garbage.get(),
                        *argv.offset(0 as c_int as isize),
                        ::core::ptr::null::<c_char>(),
                    );
                }
                argc -= 1;
                if argc < 1 as c_int && c as c_int != 'S' as c_int {
                    mainerr(
                        err_arg_missing.get(),
                        *argv.offset(0 as c_int as isize),
                        ::core::ptr::null::<c_char>(),
                    );
                }
                argv = argv.offset(1);
                argv_idx = -1 as c_int;
                's_1076: {
                    '_scripterror: {
                        's_1075: {
                            match c as c_int {
                                99 | 83 => {
                                    if (*parmp).n_commands >= MAX_ARG_CMDS {
                                        mainerr(
                                            err_extra_cmd.get(),
                                            ::core::ptr::null::<c_char>(),
                                            ::core::ptr::null::<c_char>(),
                                        );
                                    }
                                    if c as c_int == 'S' as c_int {
                                        let mut a: *mut c_char = ::core::ptr::null_mut::<c_char>();
                                        if argc < 1 as c_int {
                                            a = SESSION_FILE.as_ptr() as *mut c_char;
                                        } else if *(*argv.offset(0 as c_int as isize))
                                            .offset(0 as c_int as isize)
                                            as c_int
                                            == '-' as c_int
                                        {
                                            a = SESSION_FILE.as_ptr() as *mut c_char;
                                            argc += 1;
                                            argv = argv.offset(-1);
                                        } else {
                                            a = *argv.offset(0 as c_int as isize);
                                        }
                                        let mut s_size: size_t =
                                            strlen(a).wrapping_add(9 as size_t);
                                        let mut s: *mut c_char = xmalloc(s_size) as *mut c_char;
                                        snprintf(
                                            s,
                                            s_size,
                                            b"so %s\0".as_ptr() as *const c_char,
                                            a,
                                        );
                                        (*parmp).cmds_tofree[(*parmp).n_commands as usize] =
                                            true_0 as c_char;
                                        let c2rust_fresh10 = (*parmp).n_commands;
                                        (*parmp).n_commands = (*parmp).n_commands + 1;
                                        let c2rust_lvalue_ptr_2 =
                                            &raw mut (*parmp).commands[c2rust_fresh10 as usize];
                                        *c2rust_lvalue_ptr_2 = s;
                                    } else {
                                        let c2rust_fresh11 = (*parmp).n_commands;
                                        (*parmp).n_commands = (*parmp).n_commands + 1;
                                        let c2rust_lvalue_ptr_3 =
                                            &raw mut (*parmp).commands[c2rust_fresh11 as usize];
                                        *c2rust_lvalue_ptr_3 = *argv.offset(0 as c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                45 => {
                                    if strequal(
                                        *argv.offset(-1 as c_int as isize),
                                        b"--cmd\0".as_ptr() as *const c_char,
                                    ) {
                                        if (*parmp).n_pre_commands >= MAX_ARG_CMDS {
                                            mainerr(
                                                err_extra_cmd.get(),
                                                ::core::ptr::null::<c_char>(),
                                                ::core::ptr::null::<c_char>(),
                                            );
                                        }
                                        let c2rust_fresh12 = (*parmp).n_pre_commands;
                                        (*parmp).n_pre_commands = (*parmp).n_pre_commands + 1;
                                        let c2rust_lvalue_ptr_4 =
                                            &raw mut (*parmp).pre_commands[c2rust_fresh12 as usize];
                                        *c2rust_lvalue_ptr_4 = *argv.offset(0 as c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as c_int as isize),
                                        b"--listen\0".as_ptr() as *const c_char,
                                    ) {
                                        (*parmp).listen_addr = *argv.offset(0 as c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as c_int as isize),
                                        b"--server\0".as_ptr() as *const c_char,
                                    ) {
                                        (*parmp).server_addr = *argv.offset(0 as c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                113 => {
                                    (*parmp).use_ef = *argv.offset(0 as c_int as isize);
                                    break 's_1075;
                                }
                                105 => {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: cstr_as_string(
                                                    *argv.offset(0 as c_int as isize),
                                                ),
                                            },
                                        },
                                        0 as c_int,
                                    );
                                    break 's_1075;
                                }
                                108 => {
                                    headless_mode.set(true_0 != 0);
                                    silent_mode.set(true_0 != 0);
                                    p_verbose.set(1 as OptInt);
                                    (*parmp).no_swap_file = true_0;
                                    (*parmp).use_vimrc = (if !(*parmp).use_vimrc.is_null() {
                                        (*parmp).use_vimrc as *const c_char
                                    } else {
                                        b"NONE\0".as_ptr() as *const c_char
                                    })
                                        as *mut c_char;
                                    if (*p_shadafile.ptr()).is_null()
                                        || *p_shadafile.get() as c_int == NUL
                                    {
                                        set_option_value_give_err(
                                            kOptShadafile,
                                            OptVal {
                                                type_0: kOptValTypeString,
                                                data: OptValData {
                                                    string: String_0 {
                                                        data: b"NONE\0".as_ptr() as *const c_char
                                                            as *mut c_char,
                                                        size: ::core::mem::size_of::<[c_char; 5]>()
                                                            .wrapping_sub(1 as size_t),
                                                    },
                                                },
                                            },
                                            0 as c_int,
                                        );
                                    }
                                    (*parmp).luaf = *argv.offset(0 as c_int as isize);
                                    argc -= 1;
                                    if argc >= 0 as c_int {
                                        (*parmp).lua_arg0 = (*parmp).argc - argc;
                                        argc = 0 as c_int;
                                    }
                                    break 's_1075;
                                }
                                115 => {
                                    if !(*parmp).scriptin.is_null() {
                                        break '_scripterror;
                                    } else {
                                        (*parmp).scriptin = *argv.offset(0 as c_int as isize);
                                        break 's_1075;
                                    }
                                }
                                116 => {
                                    (*parmp).tagname = *argv.offset(0 as c_int as isize);
                                    break 's_1075;
                                }
                                117 => {
                                    (*parmp).use_vimrc = *argv.offset(0 as c_int as isize);
                                    break 's_1075;
                                }
                                119 => {
                                    if ascii_isdigit(**argv.offset(0 as c_int as isize) as c_int) {
                                        argv_idx = 0 as c_int;
                                        n = get_number_arg(
                                            *argv.offset(0 as c_int as isize),
                                            &raw mut argv_idx,
                                            10 as c_int,
                                        );
                                        set_option_value_give_err(
                                            kOptWindow,
                                            OptVal {
                                                type_0: kOptValTypeNumber,
                                                data: OptValData {
                                                    number: n as OptInt,
                                                },
                                            },
                                            0 as c_int,
                                        );
                                        argv_idx = -1 as c_int;
                                        break 's_1075;
                                    }
                                }
                                87 => {}
                                85 | _ => {
                                    break 's_1075;
                                }
                            }
                            if !(*parmp).scriptout.is_null() {
                                break '_scripterror;
                            } else {
                                (*parmp).scriptout = *argv.offset(0 as c_int as isize);
                                (*parmp).scriptout_append = c as c_int == 'w' as c_int;
                            }
                        }
                        break 's_1076;
                    }
                    vim_snprintf(
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        gettext(b"Attempt to open script file again: \"%s %s\"\n\0".as_ptr()
                            as *const c_char),
                        *argv.offset(-1 as c_int as isize),
                        *argv.offset(0 as c_int as isize),
                    );
                    fprintf(
                        stderr,
                        b"%s\0".as_ptr() as *const c_char,
                        IObuff.ptr() as *mut c_char,
                    );
                    os_exit(2 as c_int);
                }
            }
        } else {
            argv_idx = -1 as c_int;
            if (*parmp).edit_type > EDIT_STDIN as c_int {
                mainerr(
                    err_too_many_args.get(),
                    *argv.offset(0 as c_int as isize),
                    ::core::ptr::null::<c_char>(),
                );
            }
            (*parmp).edit_type = EDIT_FILE as c_int;
            ga_grow(&raw mut (*global_alist.ptr()).al_ga, 1 as c_int);
            let mut p: *mut c_char = xstrdup(*argv.offset(0 as c_int as isize));
            if (*parmp).diff_mode != 0
                && os_isdir(p) as c_int != 0
                && (*global_alist.ptr()).al_ga.ga_len > 0 as c_int
                && !os_isdir(alist_name(
                    ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                        .offset(0 as c_int as isize),
                ))
            {
                let mut r: *mut c_char = concat_fnames(
                    p,
                    path_tail(alist_name(
                        ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                            .offset(0 as c_int as isize),
                    )),
                    true_0 != 0,
                );
                xfree(p as *mut c_void);
                p = r;
            }
            let mut alist_fnum_flag: c_int = if edit_stdin(parmp) as c_int != 0 {
                1 as c_int
            } else {
                2 as c_int
            };
            alist_add(global_alist.ptr(), p, alist_fnum_flag);
        }
        if argv_idx <= 0 as c_int
            || *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) as c_int == NUL
        {
            argc -= 1;
            argv = argv.offset(1);
            argv_idx = 1 as c_int;
        }
    }
    if embedded_mode.get() as c_int != 0
        && (silent_mode.get() as c_int != 0 || !(*parmp).luaf.is_null())
    {
        mainerr(
            gettext(b"--embed conflicts with -es/-Es/-l\0".as_ptr() as *const c_char),
            ::core::ptr::null::<c_char>(),
            ::core::ptr::null::<c_char>(),
        );
    }
    if (*parmp).n_commands > 0 as c_int {
        let swcmd_len: size_t =
            strlen((*parmp).commands[0 as c_int as usize]).wrapping_add(2 as size_t);
        let swcmd: *mut c_char = xmalloc(swcmd_len.wrapping_add(1 as size_t)) as *mut c_char;
        snprintf(
            swcmd,
            swcmd_len.wrapping_add(1 as size_t),
            b":%s\r\0".as_ptr() as *const c_char,
            (*parmp).commands[0 as c_int as usize],
        );
        set_vim_var_string(VV_SWAPCOMMAND, swcmd, swcmd_len as ptrdiff_t);
        xfree(swcmd as *mut c_void);
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"parsing arguments\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn init_params(
    mut paramp: *mut mparm_T,
    mut argc: c_int,
    mut argv: *mut *mut c_char,
) {
    memset(
        paramp as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<mparm_T>(),
    );
    (*paramp).argc = argc;
    (*paramp).argv = argv;
    (*paramp).use_debug_break_level = -1 as c_int;
    (*paramp).window_count = -1 as c_int;
    (*paramp).listen_addr = ::core::ptr::null_mut::<c_char>();
    (*paramp).server_addr = ::core::ptr::null_mut::<c_char>();
    (*paramp).remote = 0 as c_int;
    (*paramp).luaf = ::core::ptr::null_mut::<c_char>();
    (*paramp).lua_arg0 = -1 as c_int;
}

pub(crate) unsafe extern "C" fn init_startuptime(mut paramp: *mut mparm_T) {
    let mut is_embed: bool = false_0 != 0;
    let mut i: c_int = 1 as c_int;
    while i < (*paramp).argc - 1 as c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i as isize),
            b"--embed\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
        {
            is_embed = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    let mut i_0: c_int = 1 as c_int;
    while i_0 < (*paramp).argc - 1 as c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i_0 as isize),
            b"--startuptime\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
        {
            time_init(
                *(*paramp).argv.offset((i_0 + 1 as c_int) as isize),
                if is_embed as c_int != 0 {
                    b"Embedded\0".as_ptr() as *const c_char
                } else {
                    b"Primary (or UI client)\0".as_ptr() as *const c_char
                },
            );
            time_start(b"--- NVIM STARTING ---\0".as_ptr() as *const c_char);
            break;
        } else {
            i_0 += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn check_and_set_isatty(mut _paramp: *mut mparm_T) {
    stdin_isatty.set(os_isatty(STDIN_FILENO));
    stdout_isatty.set(os_isatty(STDOUT_FILENO));
    stderr_isatty.set(os_isatty(STDERR_FILENO));
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"window checked\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn init_path(mut exename: *const c_char) {
    let mut exepath: [c_char; 4096] = [0 as c_char; 4096];
    let mut exepathlen: size_t = MAXPATHL as size_t;
    if os_exepath(&raw mut exepath as *mut c_char, &raw mut exepathlen) != 0 as c_int {
        path_guess_exepath(
            exename,
            &raw mut exepath as *mut c_char,
            ::core::mem::size_of::<[c_char; 4096]>(),
        );
    }
    set_vim_var_string(
        VV_PROGPATH,
        &raw mut exepath as *mut c_char,
        -1 as ptrdiff_t,
    );
    set_vim_var_string(VV_PROGNAME, path_tail(exename), -1 as ptrdiff_t);
}

pub(crate) unsafe extern "C" fn set_window_layout(mut paramp: *mut mparm_T) {
    if (*paramp).diff_mode != 0 && (*paramp).window_layout == 0 as c_int {
        if diffopt_horizontal() {
            (*paramp).window_layout = WIN_HOR as c_int;
        } else {
            (*paramp).window_layout = WIN_VER as c_int;
        }
    }
}

pub(crate) unsafe extern "C" fn execute_env(mut env: *mut c_char) -> c_int {
    let mut initstr: *mut c_char = os_getenv(env);
    if initstr.is_null() {
        return FAIL;
    }
    estack_push(ETYPE_ENV, env, 0 as linenr_T);
    let save_current_sctx: sctx_T = current_sctx.get();
    (*current_sctx.ptr()).sc_sid = SID_ENV as scid_T;
    (*current_sctx.ptr()).sc_seq = 0 as c_int;
    (*current_sctx.ptr()).sc_lnum = 0 as c_int as linenr_T;
    do_cmdline_cmd(initstr);
    estack_pop();
    current_sctx.set(save_current_sctx);
    xfree(initstr as *mut c_void);
    return OK;
}
